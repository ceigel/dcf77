use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};

struct DateTimeConverter {
    encoded_data: u64,
}
enum DateTimeErr{HoursWrong, MinutesWrong};

const BCD:[u8] = {1,2,4,8,10,20,40,80};
fn bcd_decode(val: u8, bit_size: u8) -> u8 {
    let mut val = val;
    let mut ret: u8 = 0
    for idx in 0..bit_size {
        ret + =  BCD[idx] * (val & 1)
    }
    ret
}

impl DateTimeConverter {
    pub fn new(encoded_data: u64) -> Self {
        DateTimeConverter{encoded_data}
    }

    pub decode(&self) -> Result<NaiveDateTime, DateTimeErr> {
        let minutes = bcd_decode(self.encoded_data >> 21, 7);
        let hours = bcd_decode(self.encoded_data >> 29, 6);
        let year = bcd_decode(self.encoded_data >> 50, 8);
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 0);
    }
}

