/// A structure to facilitate the decoding of a DCF77 signal which consists of 59 consecutive bits
/// of data
pub struct DCF77Time(pub u64);

impl DCF77Time {
    /// Generate an empty value for the storage of the DCF77 data
    pub fn new(dcf77bits: u64) -> Self {
        DCF77Time { 0: dcf77bits }
    }

    /// Validate the correct value of the start bit
    pub fn validate_start(&self) -> Result<(), ()> {
        if (self.0 & (1 << 0)) != 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Return whether summer time is signalled (without verifying the information)
    pub fn cest_unchecked(&self) -> bool {
        if (self.0 & (1 << 17)) != 0 {
            return true;
        }

        false
    }

    /// Return whether summer time is signalled with verification of the counter bit
    pub fn cest(&self) -> Result<bool, ()> {
        let cest = self.cest_unchecked();

        if ((self.0 & (1 << 18)) != 0) == cest {
            Err(())
        } else {
            Ok(cest)
        }
    }

    /// Return the current minutes of the hour (without verifying the information)
    pub fn minutes_unchecked(&self) -> u8 {
        let mut minutes = 0;
        if (self.0 & (1 << 21)) != 0 {
            minutes += 1;
        }

        if (self.0 & (1 << 22)) != 0 {
            minutes += 2;
        }

        if (self.0 & (1 << 23)) != 0 {
            minutes += 4;
        }

        if (self.0 & (1 << 24)) != 0 {
            minutes += 8;
        }

        if (self.0 & (1 << 25)) != 0 {
            minutes += 10;
        }

        if (self.0 & (1 << 26)) != 0 {
            minutes += 20;
        }

        if (self.0 & (1 << 27)) != 0 {
            minutes += 40;
        }

        minutes
    }

    /// Return the current minutes of the hour and verify parity and value < 60
    pub fn minutes(&self) -> Result<u8, ()> {
        let mut parity = false;
        if (self.0 & (1 << 21)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 22)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 23)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 24)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 25)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 26)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 27)) != 0 {
            parity ^= true;
        }

        let minutes = self.minutes_unchecked();
        if minutes > 59 {
            return Err(());
        }

        if ((self.0 & (1 << 28)) != 0) != parity {
            Err(())
        } else {
            Ok(minutes)
        }
    }

    /// Return the current hours of the day (without verifying the information)
    pub fn hours_unchecked(&self) -> u8 {
        let mut hours = 0;
        if (self.0 & (1 << 29)) != 0 {
            hours += 1;
        }

        if (self.0 & (1 << 30)) != 0 {
            hours += 2;
        }

        if (self.0 & (1 << 31)) != 0 {
            hours += 4;
        }

        if (self.0 & (1 << 32)) != 0 {
            hours += 8;
        }

        if (self.0 & (1 << 33)) != 0 {
            hours += 10;
        }

        if (self.0 & (1 << 34)) != 0 {
            hours += 20;
        }

        hours
    }

    /// Return the current hours of the day and verify parity and value < 23
    pub fn hours(&self) -> Result<u8, ()> {
        let mut parity = false;
        if (self.0 & (1 << 29)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 30)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 31)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 32)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 33)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 34)) != 0 {
            parity ^= true;
        }

        let hours = self.hours_unchecked();
        if hours > 23 {
            return Err(());
        }

        if ((self.0 & (1 << 35)) != 0) != parity {
            Err(())
        } else {
            Ok(hours)
        }
    }

    /// Return the current day of month (without verifying the information)
    pub fn day_unchecked(&self) -> u8 {
        let mut day = 0;
        if (self.0 & (1 << 36)) != 0 {
            day += 1;
        }

        if (self.0 & (1 << 37)) != 0 {
            day += 2;
        }

        if (self.0 & (1 << 38)) != 0 {
            day += 4;
        }

        if (self.0 & (1 << 39)) != 0 {
            day += 8;
        }

        if (self.0 & (1 << 40)) != 0 {
            day += 10;
        }

        if (self.0 & (1 << 41)) != 0 {
            day += 20;
        }

        day
    }

    /// Return the current day of month and do a basic value check
    pub fn day(&self) -> Result<u8, ()> {
        let day = self.day_unchecked();
        if day > 31 {
            Err(())
        } else {
            Ok(day)
        }
    }

    /// Return the current day of the week (without verifying the information)
    /// 0 meaning Monday
    pub fn weekday_unchecked(&self) -> u8 {
        let mut weekday = 0;
        if (self.0 & (1 << 42)) != 0 {
            weekday += 1;
        }

        if (self.0 & (1 << 43)) != 0 {
            weekday += 2;
        }

        if (self.0 & (1 << 44)) != 0 {
            weekday += 4;
        }
        weekday
    }

    /// Return the current month of the year (without verifying the information)
    pub fn month_unchecked(&self) -> u8 {
        let mut month = 0;
        if (self.0 & (1 << 45)) != 0 {
            month += 1;
        }

        if (self.0 & (1 << 46)) != 0 {
            month += 2;
        }

        if (self.0 & (1 << 47)) != 0 {
            month += 4;
        }

        if (self.0 & (1 << 48)) != 0 {
            month += 8;
        }

        if (self.0 & (1 << 49)) != 0 {
            month += 10;
        }

        month
    }

    /// Return the current year (without verifying the information)
    pub fn year_unchecked(&self) -> u16 {
        let mut year = 2000;
        if (self.0 & (1 << 50)) != 0 {
            year += 1;
        }

        if (self.0 & (1 << 51)) != 0 {
            year += 2;
        }

        if (self.0 & (1 << 52)) != 0 {
            year += 4;
        }

        if (self.0 & (1 << 53)) != 0 {
            year += 8;
        }

        if (self.0 & (1 << 54)) != 0 {
            year += 10;
        }

        if (self.0 & (1 << 55)) != 0 {
            year += 20;
        }

        if (self.0 & (1 << 56)) != 0 {
            year += 40;
        }

        if (self.0 & (1 << 57)) != 0 {
            year += 80;
        }

        year
    }

    /// Return a tuple of (year, month, day, weekday) if it passes a parity check
    pub fn date(&self) -> Result<(u16, u8, u8, u8), ()> {
        let mut parity = false;
        if (self.0 & (1 << 36)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 37)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 38)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 39)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 40)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 41)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 42)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 43)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 44)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 45)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 46)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 47)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 48)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 49)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 50)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 51)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 52)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 53)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 54)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 55)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 56)) != 0 {
            parity ^= true;
        }

        if (self.0 & (1 << 57)) != 0 {
            parity ^= true;
        }

        if ((self.0 & (1 << 58)) != 0) != parity {
            return Err(());
        }

        let year = self.year_unchecked();
        let month = self.month_unchecked();
        let day = self.day_unchecked();
        let weekday = self.weekday_unchecked();

        if year > 2100 || month > 12 || day > 31 || weekday > 7 {
            Err(())
        } else {
            Ok((year, month, day, weekday))
        }
    }
}

/*
enum SimpleDCF77DecoderState {
    WaitingForPhase,
    PhaseFound,
    BitReceived,
    FaultyBit,
    EndOfCycle,
    Idle,
}

/// A structure for a simple timeslot based DCF77 decoder
pub struct SimpleDCF77Decoder {
    scancount: u8,
    lowcount: u8,
    highcount: u8,
    idlecount: u8,
    state: SimpleDCF77DecoderState,
    data: u64,
    datapos: usize,
}

/// The SimpleDCF77Decoder implements a simple state machine to decode a DCF77 signal from a fed-in
/// readout of a GPIO pin connected to a DCF77 receiver. To use this, create the structure, set up
/// the GPIO pin the receiver is connected to as an input and call the `read_bit` method every
/// 10ms with a parameter value of `true` for a high signal level or `false` for a low signal level
impl SimpleDCF77Decoder {
    /// Create a new decoder state machine
    pub fn new() -> Self {
        Self {
            scancount: 0,
            lowcount: 0,
            highcount: 0,
            idlecount: 0,
            state: SimpleDCF77DecoderState::WaitingForPhase,
            data: 0,
            datapos: 0,
        }
    }

    /// Return the raw data as `u64` value for decoding of the current date/time
    pub fn raw_data(&self) -> u64 {
        self.data
    }

    /// Returns true as soon as an individual bit was received
    pub fn bit_complete(&self) -> bool {
        match self.state {
            SimpleDCF77DecoderState::BitReceived => true,
            _ => false,
        }
    }

    /// Returns true if the last bit couldn't be identified as high/low
    pub fn bit_faulty(&self) -> bool {
        match self.state {
            SimpleDCF77DecoderState::FaultyBit => true,
            _ => false,
        }
    }

    /// Returns true if the end of a 59s cycle was detected
    pub fn end_of_cycle(&self) -> bool {
        match self.state {
            SimpleDCF77DecoderState::EndOfCycle => true,
            _ => false,
        }
    }

    /// Returns the value of the latest received bit. Mainly useful for live display of the
    /// received bits
    pub fn latest_bit(&self) -> bool {
        (self.data & (1 << (self.datapos - 1))) != 0
    }

    /// Return the current position of the bit counter after the latest recognized end of a cycle
    /// which is identical to the current second of the minute
    pub fn seconds(&self) -> usize {
        self.datapos
    }

    /// Ingest the latest sample of the GPIO input the DCF77 receiver is connected to judge the /
    /// current position and value of the DCF77 signal bitstream
    pub fn read_bit(&mut self, bit: bool) {
        self.state = match self.state {
            SimpleDCF77DecoderState::EndOfCycle
            | SimpleDCF77DecoderState::WaitingForPhase
            | SimpleDCF77DecoderState::FaultyBit => {
                if bit {
                    self.lowcount = 1;
                    self.highcount = 0;
                    self.scancount = 0;
                    SimpleDCF77DecoderState::PhaseFound
                } else {
                    if self.scancount > 180 {
                        self.datapos = 0;
                        self.scancount = 0;

                        SimpleDCF77DecoderState::EndOfCycle
                    } else {
                        SimpleDCF77DecoderState::WaitingForPhase
                    }
                }
            }
            SimpleDCF77DecoderState::PhaseFound => {
                if self.scancount < 20 {
                    if bit {
                        if self.scancount < 10 {
                            self.lowcount += 1;
                        } else {
                            self.highcount += 1;
                        }
                    }
                    SimpleDCF77DecoderState::PhaseFound
                } else {
                    let datapos = self.datapos;
                    self.datapos += 1;
                    if self.highcount > 3 {
                        self.data |= 1 << datapos;
                        SimpleDCF77DecoderState::BitReceived
                    } else if self.lowcount > 3 {
                        self.data &= !(1 << datapos);
                        SimpleDCF77DecoderState::BitReceived
                    } else {
                        // Bad signal, let's continue with the next bit
                        SimpleDCF77DecoderState::FaultyBit
                    }
                }
            }
            SimpleDCF77DecoderState::BitReceived | SimpleDCF77DecoderState::Idle => {
                if bit {
                    self.idlecount += 1;
                }

                if self.scancount >= 90 {
                    if self.idlecount > 10 {
                        self.idlecount = 0;
                        self.scancount = 0;
                    }
                    SimpleDCF77DecoderState::WaitingForPhase
                } else {
                    SimpleDCF77DecoderState::Idle
                }
            }
        };

        self.scancount += 1;
    }
}
*/
