use crate::Rtc;
use crate::SegmentDisplay;
use adafruit_7segment::{AsciiChar, Index, SevenSegment};
use rtcc::{Hours, Rtcc};

fn display_time(display: &mut SegmentDisplay, hours: u8, minutes: u8, seconds: u8, dots: u8) {
    let d1 = (hours / 10) as u8;
    let d2 = (hours % 10) as u8;
    let d3 = (minutes / 10) as u8;
    let d4 = (minutes % 10) as u8;
    display.update_buffer_with_digit(Index::One, d1);
    display.update_buffer_with_digit(Index::Two, d2);
    display.update_buffer_with_digit(Index::Three, d3);
    display.update_buffer_with_digit(Index::Four, d4);
    display.update_buffer_with_colon(seconds % 2 == 1);
    display.update_buffer_with_dot(Index::One, dots & 1 == 1);
    display.update_buffer_with_dot(Index::Two, dots & 2 == 1);
    display.update_buffer_with_dot(Index::Three, dots & 4 == 1);
    display.update_buffer_with_dot(Index::Four, dots & 8 == 1);
    display
        .write_display_buffer()
        .expect("Could not write 7-segment display");
}

fn display_date(display: &mut SegmentDisplay, months: u8, days: u8) {
    let d1 = (days / 10) as u8;
    let d2 = (days % 10) as u8;
    let d3 = (months / 10) as u8;
    let d4 = (months % 10) as u8;
    display.update_buffer_with_digit(Index::One, d1);
    display.update_buffer_with_digit(Index::Two, d2);
    display.update_buffer_with_digit(Index::Three, d3);
    display.update_buffer_with_digit(Index::Four, d4);
    display.update_buffer_with_colon(false);
    display.update_buffer_with_dot(Index::Two, true);
    display
        .write_display_buffer()
        .expect("Could not write 7-segment display");
}

fn display_year(display: &mut SegmentDisplay, year: u16) {
    let d4 = (year % 10) as u8;
    let year = year / 10;
    let d3 = (year % 10) as u8;
    let year = year / 10;
    let d2 = (year % 10) as u8;
    let year = year / 10;
    let d1 = (year % 10) as u8;
    display.update_buffer_with_digit(Index::One, d1);
    display.update_buffer_with_digit(Index::Two, d2);
    display.update_buffer_with_digit(Index::Three, d3);
    display.update_buffer_with_digit(Index::Four, d4);
    display.update_buffer_with_colon(false);
    display.update_buffer_with_dot(Index::Two, false);
    display
        .write_display_buffer()
        .expect("Could not write 7-segment display");
}

pub(crate) fn display_error(display: &mut SegmentDisplay, dots: u8) {
    display
        .update_buffer_with_char(Index::One, AsciiChar::Minus)
        .expect("display minus");
    display
        .update_buffer_with_char(Index::Two, AsciiChar::Minus)
        .expect("display minus");
    display
        .update_buffer_with_char(Index::Three, AsciiChar::Minus)
        .expect("display minus");
    display
        .update_buffer_with_char(Index::Four, AsciiChar::Minus)
        .expect("display minus");
    display.update_buffer_with_dot(Index::One, dots & 1 == 1);
    display.update_buffer_with_dot(Index::Two, dots & 2 == 1);
    display.update_buffer_with_dot(Index::Three, dots & 4 == 1);
    display.update_buffer_with_dot(Index::Four, dots & 8 == 1);
    display.update_buffer_with_colon(false);
}

pub(crate) fn show_rtc_time(
    rtc: &mut Rtc,
    display: &mut SegmentDisplay,
    idx: u8,
    synchronized: bool,
    dots: u8,
) {
    let s = rtc.get_seconds().expect("to read seconds");
    if synchronized == false {
        display_error(display, dots);
    } else {
        match idx {
            0 | 1 => {
                let h = rtc.get_hours().expect("to read hours");
                let m = rtc.get_minutes().expect("to read minutes");
                let hours = match h {
                    Hours::AM(hours) => hours,
                    Hours::PM(hours) => hours,
                    Hours::H24(hours) => hours,
                };
                display_time(display, hours, m, s, dots);
            }
            2 => {
                let d = rtc.get_day().expect("to read days");
                let m = rtc.get_month().expect("to read months");
                display_date(display, m, d);
            }
            _ => {
                let y = rtc.get_year().expect("to read the year");
                display_year(display, y);
            }
        }
    }
}
