use crate::cycles_computer::CyclesComputer;
use core::num::Wrapping;
use rtic::cyccnt::{Instant, U32Ext};
use rtt_target::rprintln;

#[derive(Debug)]
pub enum DecoderError {
    WrongTransition,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Edge {
    Up,
    Down,
}

impl Edge {
    pub fn new(low_to_high: bool) -> Self {
        if low_to_high {
            Edge::Up
        } else {
            Edge::Down
        }
    }
}

struct Binning {
    bins: [i8; 1000],
    max_index: Option<usize>,
    max_val: i8,
    min_index: Option<usize>,
    min_val: i8,
}

impl Default for Binning {
    fn default() -> Self {
        Binning {
            bins: [0; 1000],
            max_index: None,
            max_val: 0,
            min_index: None,
            min_val: 0,
        }
    }
}

impl Binning {
    const MARGIN: usize = 15;
    pub fn add_edge(&mut self, bin: u32, edge: Edge) -> Option<Edge> {
        let bin = bin as usize;
        if edge == Edge::Up {
            self.bins[bin] += 1;
            if self.bins[bin] > self.max_val {
                self.max_val = self.bins[bin];
                self.max_index.replace(bin);
            }
        } else {
            self.bins[bin] -= 1;
            if self.bins[bin] < self.min_val {
                self.min_val = self.bins[bin];
                self.min_index.replace(bin);
            }
            self.scale_bins_if_needed();
        }
        self.rate_edge(bin, edge)
    }

    fn scale_bins_if_needed(&mut self) {
        if self.max_val >= i8::MAX || self.min_val <= (i8::MIN >> 1) {
            self.max_val = self.max_val >> 1;
            self.min_val = self.min_val >> 1;
            for mut b in self.bins {
                b = b >> 1;
            }
        }
    }

    fn rate_edge(&self, bin: usize, edge: Edge) -> Option<Edge> {
        if let (Some(max_index), Some(min_index)) = (self.max_index, self.min_index) {
            if edge == Edge::Up {
                if (((max_index as i32) - (bin as i32)).abs() as usize) < Self::MARGIN {
                    // up-edge close to min (15ms)
                    return Some(edge);
                }
            } else {
                if (((min_index as i32) - (bin as i32)).abs() as usize) < Self::MARGIN {
                    // down-edge close to min (15ms)
                    return Some(edge);
                }
            }
        }
        None
    }
}

pub struct DCF77Decoder {
    last_high_to_low: Option<Instant>,
    last_low_to_high: Option<Instant>,
    cycles_computer: CyclesComputer,
    bins: Binning,
}

impl DCF77Decoder {
    pub fn new(cycles_computer: CyclesComputer) -> Self {
        Self {
            last_high_to_low: None,
            last_low_to_high: None,
            cycles_computer,
            bins: Binning::default(),
        }
    }

    pub fn register_transition(
        &mut self,
        low_to_high: bool,
        now: Instant,
        bin_idx: u32,
    ) -> Result<(), DecoderError> {
        let edge = Edge::new(low_to_high);
        if self.bins.add_edge(bin_idx, edge).is_none() {
            return Ok(());
        }
        rprintln!("tick: {:?} {}", edge, bin_idx);
        if low_to_high {
            self.last_low_to_high.replace(now);
            if let Some(last) = self.last_high_to_low.take() {
                let diff = now - last;
                let diff = self.cycles_computer.from_cycles(diff);
                let bit = match diff.as_millis() {
                    60..=140 => 0,
                    160..=240 => 1,
                    _ => 3,
                };
                rprintln!("Edge: low->high: {}ms {}", diff.as_millis(), bit);
            } else {
                return Err(DecoderError::WrongTransition);
            }
        } else {
            self.last_high_to_low.replace(now);
            if let Some(last) = self.last_low_to_high.take() {
                let diff = now - last;
                let diff = self.cycles_computer.from_cycles(diff);
                let minute_mark = if diff.as_millis() > 1500 {
                    " MINUTE MARK"
                } else {
                    ""
                };
                rprintln!("Edge: high->low: {}ms{}", diff.as_millis(), minute_mark);
            } else {
                return Err(DecoderError::WrongTransition);
            }
        }
        Ok(())
    }
}
