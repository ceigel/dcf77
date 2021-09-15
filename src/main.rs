#![no_std]
#![no_main]
extern crate heapless;
mod decoder;
mod frequency;

use crate::frequency::ClockEvent;
use crate::frequency::Timing;
mod cycles_computer;
use crate::stm32f4xx_hal::i2c::I2c;
use dcf77::DCF77Time;
use panic_rtt_target as _;

use adafruit_7segment::{AsciiChar, Index, SevenSegment};
use core::num::Wrapping;
use core::time;
use cortex_m::peripheral::DWT;
use cycles_computer::CyclesComputer;
use feather_f405::hal as stm32f4xx_hal;
use feather_f405::{hal::prelude::*, pac, setup_clocks};
use ht16k33::{Dimming, Display, HT16K33};
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::gpio::{gpioa, gpiob, AlternateOD, Input, PullUp, AF4};
use stm32f4xx_hal::timer::{Event, Timer};

fn display_time(
    display: &mut HT16K33<
        I2c<pac::I2C1, (gpiob::PB6<AlternateOD<AF4>>, gpiob::PB7<AlternateOD<AF4>>)>,
    >,
    hours: u8,
    minutes: u8,
) {
    let d1 = (hours / 10) as u8;
    let d2 = (hours % 10) as u8;
    let d3 = (minutes / 10) as u8;
    let d4 = (minutes % 10) as u8;
    display.update_buffer_with_digit(Index::One, d1);
    display.update_buffer_with_digit(Index::Two, d2);
    display.update_buffer_with_digit(Index::Three, d3);
    display.update_buffer_with_digit(Index::Four, d4);
    display.update_buffer_with_colon(true);
}

fn display_error(
    display: &mut HT16K33<
        I2c<pac::I2C1, (gpiob::PB6<AlternateOD<AF4>>, gpiob::PB7<AlternateOD<AF4>>)>,
    >,
) {
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
    display.update_buffer_with_colon(false);
}

fn show_new_time(
    data: Option<u64>,
    display: &mut HT16K33<
        I2c<pac::I2C1, (gpiob::PB6<AlternateOD<AF4>>, gpiob::PB7<AlternateOD<AF4>>)>,
    >,
) {
    match data {
        None => {
            display_error(display);
        }
        Some(data) => {
            let time_decoder = DCF77Time::new(data);
            if time_decoder.validate_start().is_ok() {
                rprintln!("No start");
            } else {
                match (time_decoder.hours(), time_decoder.minutes()) {
                    (Err(_), Err(_)) => {
                        rprintln!("hours and minutes error");
                        display_error(display);
                    }
                    (Err(_), _) => {
                        rprintln!("hours error");
                        display_error(display);
                    }
                    (_, Err(_)) => {
                        rprintln!("minutes error");
                        display_error(display);
                    }
                    (Ok(hours), Ok(minutes)) => {
                        rprintln!("Time: {}:{}", hours, minutes);
                        display_time(display, hours, minutes);
                    }
                }
            }
        }
    }
    display
        .write_display_buffer()
        .expect("Could not write 7-segment display");
}

const VALID_TRANSITION_TIME: Wrapping<u64> = Wrapping(20);
struct TransitionCandidate {
    pub(crate) level: bool,
    pub(crate) time: Wrapping<u64>,
}

impl TransitionCandidate {
    pub fn new(level: bool, time: u64) -> Self {
        Self {
            level,
            time: Wrapping(time),
        }
    }
    pub fn valid(&self, current: u64) -> bool {
        if self.time - Wrapping(current) > VALID_TRANSITION_TIME {
            true
        } else {
            false
        }
    }
}

pub struct MyDecoder {
    bits: u64,
    seconds: u8,
    seconds_changed: bool,
    data_pos: u16,
    current_count: Wrapping<u64>,
    current_level: bool,
    last_transition: Wrapping<u64>,
    last_bits: Option<u64>,
}

impl MyDecoder {
    pub fn new() -> Self {
        Self {
            bits: 0,
            seconds: 0,
            seconds_changed: false,
            data_pos: 0,
            current_count: Wrapping(0),
            current_level: true,
            last_transition: Wrapping(0),
            last_bits: None,
        }
    }
    pub fn read_seconds(&mut self) -> u8 {
        self.seconds_changed = false;
        self.seconds
    }

    pub fn seconds_changed(&self) -> bool {
        self.seconds_changed
    }

    pub fn last_bits(&self) -> Option<u64> {
        self.last_bits
    }

    pub fn read_bit(&mut self, level: bool) {
        if level != self.current_level {
            if level == true {
                if self.current_count - self.last_transition >= Wrapping(9) {
                    rprintln!("Transition up {:?}", self.current_count / Wrapping(100));
                    let datapos = self.data_pos;
                    self.data_pos = if self.data_pos < 60 {
                        self.data_pos + 1
                    } else {
                        0
                    };

                    if self.current_count - self.last_transition > Wrapping(18) {
                        self.bits = self.bits | (1 << datapos); // bit == 1
                    } else if self.current_count - self.last_transition >= Wrapping(9) {
                        self.bits = self.bits & !(1 << datapos); // bit == 0
                    } else {
                        rprintln!("Transition too short");
                    }
                    self.seconds += 1;
                    self.seconds_changed = true;

                    self.current_level = level;
                    self.last_transition = self.current_count;
                }
            } else {
                if self.current_count - self.last_transition >= Wrapping(90) {
                    rprintln!("Transition down {:?}", self.current_count / Wrapping(100));
                    if self.current_count - self.last_transition > Wrapping(180) {
                        // minute end
                        rprintln!("minute ended");
                        self.data_pos = 0;
                        self.seconds = 0;
                        self.last_bits.replace(self.bits);
                    }
                    self.current_level = level;
                    self.last_transition = self.current_count;
                }
            }
        }
        self.current_count += Wrapping(1);
    }
}
const DISP_I2C_ADDR: u8 = 0x77;
#[app(device = feather_f405::hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        segment_display:
            HT16K33<I2c<pac::I2C1, (gpiob::PB6<AlternateOD<AF4>>, gpiob::PB7<AlternateOD<AF4>>)>>,
        dcf_pin: gpioa::PA<Input<PullUp>>,
        timer: Timer<pac::TIM2>,
        timing: Timing,
        cycles_computer: CyclesComputer,
        val: u16,
        decoder: MyDecoder,
    }
    #[init(spawn=[])]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        let mut core = cx.core;
        let device = cx.device;
        core.DCB.enable_trace();
        DWT::unlock();
        core.DWT.enable_cycle_counter();
        //core.SCB.set_sleepdeep();

        let clocks = setup_clocks(device.RCC);
        let _syscfg = device.SYSCFG.constrain();
        let _exti = device.EXTI;

        let gpiob = device.GPIOB.split();
        let scl = gpiob.pb6.into_alternate_af4().set_open_drain();
        let sda = gpiob.pb7.into_alternate_af4().set_open_drain();
        let i2c = I2c::new(device.I2C1, (scl, sda), 400.khz(), clocks);
        let mut ht16k33 = HT16K33::new(i2c, DISP_I2C_ADDR);
        ht16k33.initialize().expect("Failed to initialize ht16k33");
        ht16k33
            .set_display(Display::ON)
            .expect("Could not turn on the display!");
        ht16k33
            .set_dimming(Dimming::BRIGHTNESS_MAX)
            .expect("Could not set dimming!");
        display_error(&mut ht16k33);
        let gpioa = device.GPIOA.split();
        let pin = gpioa.pa6.into_pull_up_input().downgrade();
        //pa6.make_interrupt_source(&mut syscfg);
        //pa6.trigger_on_edge(&mut exti, Edge::RISING_FALLING);
        //pa6.enable_interrupt(&mut exti);

        let mut timer = Timer::tim2(device.TIM2, 100.hz(), clocks);
        timer.listen(Event::TimeOut);
        let timing = Timing::new();
        rprintln!("Init successful");
        init::LateResources {
            segment_display: ht16k33,
            dcf_pin: pin,
            timer,
            timing,
            cycles_computer: CyclesComputer::new(clocks.sysclk()),
            val: 0,
            decoder: MyDecoder::new(),
        }
    }

    #[allow(clippy::empty_loop)]
    #[idle()]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {}
    }

    /*
    #[task(binds = EXTI9_5, priority=2, resources=[dcf_pin, timing, cycles_computer])]
    fn exti9_5(cx: exti9_5::Context) {
        cx.resources.dcf_pin.clear_interrupt_pending_bit();
        cx.resources.timing.event(ClockEvent::SignalDetected(42));
    }
    */

    #[task(binds = TIM2, priority=2, resources=[timer, timing, decoder, dcf_pin, segment_display])]
    fn tim2(cx: tim2::Context) {
        cx.resources.timer.clear_interrupt(Event::TimeOut);
        //        cx.resources.timing.event(ClockEvent::TimerExpired);
        let pin_high = cx.resources.dcf_pin.is_high().unwrap();
        let decoder = cx.resources.decoder;
        decoder.read_bit(pin_high);

        if decoder.seconds_changed() {
            rprintln!("{}", decoder.read_seconds());
        }
        let display = cx.resources.segment_display;
        show_new_time(decoder.last_bits(), display);
        //rprintln!("TIMER2");
    }

    extern "C" {
        fn UART4();
    }
};
