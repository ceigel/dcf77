use rtt_target::rprintln;

const TIMEOUTS_PER_SEC: usize = 10;
const CYCLE_ROUNDS: usize = 3;

#[derive(Debug)]
pub struct Timing {
    counter: u32,
    state: [[Option<u32>; TIMEOUTS_PER_SEC]; CYCLE_ROUNDS],
    last_signal: Option<u32>,
}

#[derive(Clone, Debug)]
pub enum ClockEvent {
    TimerExpired,
    SignalDetected(u32),
}

impl Default for Timing {
    fn default() -> Self {
        Self::new()
    }
}

impl Timing {
    pub fn new() -> Self {
        Self {
            counter: 0,
            state: [[None; TIMEOUTS_PER_SEC]; CYCLE_ROUNDS],
            last_signal: None,
        }
    }
    pub fn event(&mut self, e: ClockEvent) {
        match e {
            ClockEvent::SignalDetected(ticks) => self.last_signal = Some(ticks),
            ClockEvent::TimerExpired => {
                if self.counter < (TIMEOUTS_PER_SEC * CYCLE_ROUNDS) as u32 {
                    let cycle = self.counter as usize / TIMEOUTS_PER_SEC;
                    let cycle_index = self.counter as usize % TIMEOUTS_PER_SEC;
                    rprintln!(
                        "Hello from {:?} {}, cycle: {}, index: {}",
                        e,
                        self.counter,
                        cycle,
                        cycle_index
                    );
                    self.state[cycle][cycle_index] = self.last_signal;
                    self.last_signal = None;
                    self.counter += 1;
                    rprintln!("{:?}", self);
                }
            }
        }
    }

    pub fn identify_slot(&self) -> Option<usize> {
        let mut summed = [0u8; TIMEOUTS_PER_SEC];
        for i in 0..TIMEOUTS_PER_SEC {
            let mut sum = 0u8;
            for j in 0..CYCLE_ROUNDS {
                if self.state[j][i].is_some() {
                    sum += 1
                }
            }
            summed[i] = sum;
        }
        rprintln!("{:?}", summed);
        index_of_max(&summed)
    }
}

fn index_of_max(values: &[u8]) -> Option<usize> {
    values
        .iter()
        .enumerate()
        .max_by_key(|(_idx, &val)| val)
        .map(|(idx, _val)| idx)
}
