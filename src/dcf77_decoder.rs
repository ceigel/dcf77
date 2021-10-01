use core::num::Wrapping;
use rtt_target::rprintln;

pub struct SignalSmoother<const X: usize> {
    buf: [bool; X],
    last_val: bool,
}

impl<const X: usize> SignalSmoother<X> {
    pub fn new() -> Self {
        Self {
            buf: [true; X],
            last_val: true,
        }
    }
    pub fn add_signal(&mut self, sig: bool) -> bool {
        self.buf.rotate_left(1);
        self.buf[X - 1] = sig;
        if self.buf.iter().all(|x| *x != self.last_val) {
            self.last_val = !self.last_val;
        }
        self.last_val
    }
}

pub struct DCF77Decoder {
    current_count: Wrapping<u64>,
    current_level: bool,
    last_transition: Wrapping<u64>,
    current_pause: u64,
    smoother: SignalSmoother<7>,
    start_detected: bool,
    current_bits: u64,
    last_bits: Option<u64>,
    bit_pos: usize,
}

impl DCF77Decoder {
    pub fn new() -> Self {
        Self {
            current_count: Wrapping(0),
            current_level: false,
            last_transition: Wrapping(0),
            current_pause: 0,
            smoother: SignalSmoother::new(),
            start_detected: false,
            current_bits: 0,
            last_bits: None,
            bit_pos: 0,
        }
    }

    pub fn current_level(&self) -> bool {
        self.current_level
    }

    pub fn reset_last_bits(&mut self) {
        self.last_bits.take();
    }

    pub fn last_bits(&self) -> Option<u64> {
        self.last_bits
    }

    pub fn read_bit(&mut self, level: bool) {
        let level = self.smoother.add_signal(level);
        if level != self.current_level {
            if self.current_pause > 0 {
                if self.current_level == true && self.current_pause > 150 {
                    rprintln!("Minute mark {}", self.current_pause);
                    if self.start_detected {
                        self.last_bits.replace(self.current_bits);
                        rprintln!("Data: {:060b}", self.current_bits);
                    } else {
                        self.start_detected = true;
                    }
                    self.bit_pos = 0;
                    self.current_bits = 0;
                } else if self.start_detected && self.current_level == false {
                    if self.current_pause >= 15 {
                        self.current_bits |= 1 << self.bit_pos
                    } else {
                        self.current_bits &= !(1 << self.bit_pos)
                    }
                    if self.bit_pos == 59 {
                        self.bit_pos = 0;
                        self.last_bits.replace(self.current_bits);
                        rprintln!("Data: {:060b}", self.current_bits);
                        self.current_bits = 0;
                        self.start_detected = false
                    } else {
                        self.bit_pos += 1;
                    }
                }
            }
            self.current_pause = 0;
            self.current_level = level;
            self.last_transition = self.current_count;
        } else {
            let diff = self.current_count - self.last_transition;
            let Wrapping(d) = diff;
            self.current_pause = d;
        }
        self.current_count += Wrapping(1);
    }
}
