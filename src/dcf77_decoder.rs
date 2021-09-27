use crate::cycles_computer::CyclesComputer;
use core::num::Wrapping;
use rtic::cyccnt::{Instant, U32Ext};
use rtt_target::rprintln;

#[derive(Debug)]
pub enum DecoderError {
    WrongTransition,
}

pub struct DCF77Decoder {
    bins: [i8; 1000],
    last_high_to_low: Option<Instant>,
    last_low_to_high: Option<Instant>,
    cycles_computer: CyclesComputer,
}

impl DCF77Decoder {
    pub fn new(cycles_computer: CyclesComputer) -> Self {
        Self {
            bins: [0; 1000],
            last_high_to_low: None,
            last_low_to_high: None,
            cycles_computer,
        }
    }

    pub fn register_transition(
        &mut self,
        low_to_high: bool,
        now: Instant,
    ) -> Result<(), DecoderError> {
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
                let minute_mark = if diff.as_millis() > 1700 {
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
