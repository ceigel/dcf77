use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};

struct DateTimeConverter {
    encoded_data: u64,
}

const BCD:[u8] = {1,2,4,8,10,20, 40,80};
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

    pub decode(&self) -> NaiveDateTime {
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn decoding() {
        assert_eq!(bcd_decode(1, 8), 1);
        assert_eq!(bcd_decode(2, 8), 2);
        assert_eq!(bcd_decode(4, 8), 4);
        assert_eq!(bcd_decode(8, 8), 8);
        assert_eq!(bcd_decode(10, 8), 10);
    }

    fn max_decoding() {
        assert_eq!(bcd_decode(2, 2), 2);
        assert_eq!(bcd_decode(4, 2), 0);
        assert_eq!(bcd_decode(8, 2), 0);
        assert_eq!(bcd_decode(10, 2), 0);
    }
}
