#![no_std]
#![no_main]
extern crate heapless;
mod cycles_computer;
mod decoder;
mod frequency;
mod time_display;

use crate::frequency::ClockEvent;
use crate::frequency::Timing;
use crate::stm32f4xx_hal::i2c::I2c;
use panic_rtt_target as _;

use core::num::Wrapping;
use core::time;
use cortex_m::peripheral::DWT;
use cycles_computer::CyclesComputer;
use feather_f405::hal as stm32f4xx_hal;
use feather_f405::{hal::prelude::*, pac, setup_clocks};
use ht16k33::{Dimming, Display, HT16K33};
use rtcc::{NaiveDate, NaiveTime, Rtcc};
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::gpio::{gpioa, gpiob, AlternateOD, Floating, Input, AF4};
use stm32f4xx_hal::rtc::Rtc;
use stm32f4xx_hal::timer::{Event, Timer};
use time_display::{display_error, show_new_time, show_rtc_time};

type SegmentDisplay =
    HT16K33<I2c<pac::I2C1, (gpiob::PB6<AlternateOD<AF4>>, gpiob::PB7<AlternateOD<AF4>>)>>;

pub struct MyDecoder {
    current_count: Wrapping<u64>,
    current_level: bool,
    last_transition: Wrapping<u64>,
    last_pause: u64,
}

impl MyDecoder {
    pub fn new() -> Self {
        Self {
            current_count: Wrapping(0),
            current_level: false,
            last_transition: Wrapping(0),
            last_pause: 0,
        }
    }

    pub fn read_bit(&mut self, level: bool) {
        if self.current_count % Wrapping(100) == Wrapping(0) {
            rprintln!("{}", self.current_count / Wrapping(100));
        }
        if level != self.current_level {
            if self.last_pause > 0 {
                rprintln!("Level: {}, pause: {}", self.current_level, self.last_pause);
            }
            self.last_pause = 0;
            self.current_level = level;
            self.last_transition = self.current_count;
        } else {
            let diff = self.current_count - self.last_transition;
            if diff >= Wrapping(30) {
                if let Wrapping(d) = diff {
                    self.last_pause = d;
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
        segment_display: SegmentDisplay,
        dcf_pin: gpioa::PA<Input<Floating>>,
        timer: Timer<pac::TIM2>,
        timing: Timing,
        cycles_computer: CyclesComputer,
        val: u16,
        decoder: MyDecoder,
        rtc: Rtc,
        timer_count: u16,
        synchronized: bool,
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
        let mut pwr = device.PWR;

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
        ht16k33
            .write_display_buffer()
            .expect("Could not write 7-segment display");
        let gpioa = device.GPIOA.split();
        let pin = gpioa.pa6.into_floating_input().downgrade();
        //pa6.make_interrupt_source(&mut syscfg);
        //pa6.trigger_on_edge(&mut exti, Edge::RISING_FALLING);
        //pa6.enable_interrupt(&mut exti);

        let mut timer = Timer::tim2(device.TIM2, 100.hz(), clocks);
        timer.listen(Event::TimeOut);
        let timing = Timing::new();
        let mut rtc = Rtc::new(device.RTC, 255, 127, false, &mut pwr);
        rtc.set_time(&NaiveTime::from_hms(21, 50, 0))
            .expect("to set time");
        rtc.set_date(&NaiveDate::from_ymd(2021, 09, 15))
            .expect("to set date");
        rprintln!("Init successful");
        init::LateResources {
            segment_display: ht16k33,
            dcf_pin: pin,
            timer,
            timing,
            cycles_computer: CyclesComputer::new(clocks.sysclk()),
            val: 0,
            decoder: MyDecoder::new(),
            rtc,
            timer_count: 0,
            synchronized: false,
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

    #[task(binds = TIM2, priority=2, resources=[timer, timing, decoder, dcf_pin, segment_display, rtc, timer_count, synchronized])]
    fn tim2(cx: tim2::Context) {
        cx.resources.timer.clear_interrupt(Event::TimeOut);
        //        cx.resources.timing.event(ClockEvent::TimerExpired);
        let pin_high = cx.resources.dcf_pin.is_high().unwrap();
        let decoder = cx.resources.decoder;
        decoder.read_bit(pin_high);

        let display = cx.resources.segment_display;
        //show_new_time(decoder.last_bits(), display);
        let time_display_idx = ((*cx.resources.timer_count / 300) % 4) as u8;
        show_rtc_time(
            cx.resources.rtc,
            display,
            time_display_idx,
            *cx.resources.synchronized,
        );
        *cx.resources.timer_count += 1;
        //rprintln!("TIMER2");
    }

    extern "C" {
        fn UART4();
    }
};
