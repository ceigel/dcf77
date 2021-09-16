use chrono::naive::{NaiveDate, NaiveDateTime};
enum DateTimeErr{MinutesWrong, HoursWrong, DatesWrong}

/// Decode DCF77 binary, and output as chrono::naive::NaiveDateTime 
/// Sample data:
/// * 23:14 15.Sep.2021 Deutschland (CEST)
/// * `let test_data = 0b00000000000000001000_1_0010100_0_110001_1_101010_110_10010_10000100_1_0;`
/// * Zero-padded 64bit intgeger: `000000000000000010001001010001100011101010110100101000010010`
/// ```
/// // 23:14 15.Sep.2021 Deutschland (CEST)
/// // Write me!
/// ```
struct DCF77DateTimeConverter{
    encoded_data: u64,
    bcd: [u32;8],
}

impl DCF77DateTimeConverter {
    pub fn new(dcf77_data: u64) -> Self {
        DCF77DateTimeConverter {
            encoded_data: dcf77_data,
            bcd: [1,2,4,8,10,20,40,80],
        }
    }

    pub fn dcf77_decoder(&self) -> Result<NaiveDateTime, DateTimeErr> {
        let year = ((self.encoded_data >> 2) & 0b11111111)as u32;
        let month = ((self.encoded_data >> 10) & 0b11111) as u32;
        let weekday = ((self.encoded_data >> 15) & 0b111) as u32;
        let day = ((self.encoded_data >> 18) & 0b111111) as u32;
        let datetime_frame = ((self.encoded_data >> 2) & 0b1111111111111111111111) as u32;
        let hours = ((self.encoded_data >> 25) & 0b111111) as u32;
        let minutes = ((self.encoded_data >> 32) & 0b1111111) as u32;
 
        let check_datetime_parity :bool = DCF77DateTimeConverter::check_parity(datetime_frame) == ((self.encoded_data >> 1 )&1) as u32;
        let check_hours_parity :bool    = DCF77DateTimeConverter::check_parity(hours)          == ((self.encoded_data >> 24)&1) as u32;
        let check_minutes_parity :bool  = DCF77DateTimeConverter::check_parity(minutes)        == ((self.encoded_data >> 31)&1) as u32;

        if !check_datetime_parity { println!("There is a datetime prity error.");}
        if !check_hours_parity { println!("There is a houur prity error.");}
        if !check_minutes_parity { println!("There is a minutes prity error.");}
    
        //let dt: NaiveDateTime = NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11);
        //println!("{:?}", dt);
    
    
        //let dt: NaiveDateTime = NaiveDate::
        //                        from_ymd(DCF77DateTimeConverter::naive_year(self, year) as i32,
        //                            DCF77DateTimeConverter::naive_month(self, month),
        //                            DCF77DateTimeConverter::naive_day_or_hours(self, day)
        //                        )
        //                        .and_hms(DCF77DateTimeConverter::naive_day_or_hours(self, hours),
        //                            DCF77DateTimeConverter::naive_minutes(self, minutes),
        //                            0
        //                        );
        //let dt: NaiveDateTime = NaiveDate::from_ymd(2020, 8, 14).and_hms(22, 32, 0);
        NaiveDate::from_ymd(DCF77DateTimeConverter::naive_year(self, year) as i32,
                       DCF77DateTimeConverter::naive_month(self, month),
                       DCF77DateTimeConverter::naive_day_or_hours(self, day)
                   )
                   .and_hms(DCF77DateTimeConverter::naive_day_or_hours(self, hours),
                       DCF77DateTimeConverter::naive_minutes(self, minutes),
                       0
                   )
    }

    fn naive_year(&self, year_dcf77: u32) -> u32 {
       let mut naive_year = 2000;
       for bit in 0..8 {
           naive_year += self.bcd[bit]*((year_dcf77 >> 7-bit)&1)
       }
       naive_year
    }
    fn naive_month(&self, month_dcf77: u32) -> u32 {
        let mut naive_month = 0;
        for bit in 0..5 {
            naive_month += self.bcd[bit]*((month_dcf77 >> 4-bit)&1)
        }
        naive_month
    }
    fn naive_day_or_hours(&self, day_dcf77: u32) -> u32 {
        let mut naive_day = 0;
        for bit in 0..6 {
            naive_day += self.bcd[bit]*((day_dcf77 >> 5-bit)&1)
        }
        naive_day
    }
    
    fn naive_minutes(&self, minutes_dcf77: u32) -> u32 {
        let mut naive_minutes= 0;
        for bit in 0..7 {
            naive_minutes += self.bcd[bit]*((minutes_dcf77 >> 6-bit)&1)
        }
        naive_minutes
    }

    /// Check bit-parity of a u64 integer
    /// ```
    /// assert_eq!(check_parity(13), 1)
    /// assert_eq!(check_parity(2806404), 1)
    /// ```
    fn check_parity(i: u32) -> u32 {
        let mut j = i ^ (i >> 1);
        j = j ^ (j >> 2);
        j = j ^ (j >> 4);
        j = j ^ (j >> 8);
        j = j ^ (j >> 16);
        return j & 1;
    }
}
