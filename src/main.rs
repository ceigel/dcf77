#![no_std]
#![no_main]
extern crate heapless;
mod decoder;
mod frequency;

use crate::frequency::ClockEvent;
use crate::frequency::Timing;
mod cycles_computer;
use crate::stm32f4xx_hal::i2c::I2c;
use dcf77::{DCF77Time, SimpleDCF77Decoder};
use panic_rtt_target as _;

use adafruit_7segment::{AsciiChar, Index, SevenSegment};
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
        decoder: SimpleDCF77Decoder,
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
        let pa6 = gpioa.pa6.into_pull_up_input().downgrade();
        //pa6.make_interrupt_source(&mut syscfg);
        //pa6.trigger_on_edge(&mut exti, Edge::RISING_FALLING);
        //pa6.enable_interrupt(&mut exti);

        let mut timer = Timer::tim2(device.TIM2, 100.hz(), clocks);
        timer.listen(Event::TimeOut);
        let timing = Timing::new();
        rprintln!("Init successful");
        init::LateResources {
            segment_display: ht16k33,
            dcf_pin: pa6,
            timer,
            timing,
            cycles_computer: CyclesComputer::new(clocks.sysclk()),
            val: 0,
            decoder: SimpleDCF77Decoder::new(),
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
        decoder.read_bit(!pin_high);

        if decoder.bit_faulty() {
            rprintln("bit faulty");
        }
        if decoder.bit_complete() {
            rprintln!("{}", decoder.seconds());
        }
        let mut data: Option<u64> = None;
        if decoder.end_of_cycle() {
            data.replace(decoder.raw_data());
            rprintln!("end of cycle");
        } else {
            data.take();
        }
        let display = cx.resources.segment_display;
        show_new_time(data, display);
        //rprintln!("TIMER2");
    }

    extern "C" {
        fn UART4();
    }
};
