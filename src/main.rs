#![no_std]
#![no_main]
extern crate heapless;
mod frequency;

use crate::frequency::ClockEvent;
use crate::frequency::Timing;
mod cycles_computer;
use crate::stm32f4xx_hal::i2c::I2c;
use panic_rtt_target as _;

use adafruit_7segment::{Index, SevenSegment};
use core::time;
use cortex_m::peripheral::DWT;
use cycles_computer::CyclesComputer;
use feather_f405::hal as stm32f4xx_hal;
use feather_f405::{hal::prelude::*, pac, setup_clocks};
use ht16k33::{Dimming, Display, HT16K33};
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::gpio::{gpioa, gpiob, AlternateOD, Edge, ExtiPin, Input, PullUp, AF4};
use stm32f4xx_hal::timer::{Event, Timer};

const DISP_I2C_ADDR: u8 = 0x77;
#[app(device = feather_f405::hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        segment_display:
            HT16K33<I2c<pac::I2C1, (gpiob::PB6<AlternateOD<AF4>>, gpiob::PB7<AlternateOD<AF4>>)>>,
        input_pin: gpioa::PA<Input<PullUp>>,
        timer: Timer<pac::TIM2>,
        timing: Timing,
        cycles_computer: CyclesComputer,
        val: u16,
    }
    #[init(spawn=[show_time])]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        let mut core = cx.core;
        let device = cx.device;
        core.DCB.enable_trace();
        DWT::unlock();
        core.DWT.enable_cycle_counter();
        //core.SCB.set_sleepdeep();

        let clocks = setup_clocks(device.RCC);
        let mut syscfg = device.SYSCFG.constrain();
        let mut exti = device.EXTI;

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
        let gpioa = device.GPIOA.split();
        let mut pa6 = gpioa.pa6.into_pull_up_input().downgrade();
        pa6.make_interrupt_source(&mut syscfg);
        pa6.trigger_on_edge(&mut exti, Edge::RISING_FALLING);
        pa6.enable_interrupt(&mut exti);

        let mut timer = Timer::tim2(device.TIM2, 10.hz(), clocks);
        timer.listen(Event::TimeOut);
        let timing = Timing::new();
        rprintln!("Init successful");
        cx.spawn.show_time().expect("To start show_time task");
        init::LateResources {
            segment_display: ht16k33,
            input_pin: pa6,
            timer,
            timing,
            cycles_computer: CyclesComputer::new(clocks.sysclk()),
            val: 0,
        }
    }

    #[allow(clippy::empty_loop)]
    #[idle()]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {}
    }

    #[task(resources=[val,cycles_computer, segment_display], schedule=[show_time])]
    fn show_time(mut cx: show_time::Context) {
        let display = cx.resources.segment_display;
        // Sending individual digits
        let val = *cx.resources.val;
        *cx.resources.val = val + 1;
        let colon = (val & 1) == 1;
        let d4 = (val % 10) as u8;
        let val = (val / 10) as u8;
        let d3 = (val % 10) as u8;
        let val = (val / 10) as u8;
        let d2 = (val % 10) as u8;
        let val = (val / 10) as u8;
        let d1 = (val % 10) as u8;
        let val = (val / 10) as u8;

        display.update_buffer_with_digit(Index::One, d1);
        display.update_buffer_with_dot(Index::One, true);

        display.update_buffer_with_digit(Index::Two, d2);
        display.update_buffer_with_dot(Index::Two, true);

        display.update_buffer_with_digit(Index::Three, d3);
        display.update_buffer_with_dot(Index::Three, true);

        display.update_buffer_with_digit(Index::Four, d4);
        display.update_buffer_with_dot(Index::Four, true);
        display.update_buffer_with_colon(colon);
        display
            .write_display_buffer()
            .expect("Could not write 7-segment display");
        rprintln!("time {}", val);
        let delay = cx
            .resources
            .cycles_computer
            .lock(|cc| cc.to_cycles(time::Duration::from_millis(500)));
        cx.schedule
            .show_time(cx.scheduled + delay)
            .expect("To be able to reschedule show_time");
    }

    #[task(binds = EXTI9_5, priority=2, resources=[input_pin, timing, cycles_computer])]
    fn exti9_5(cx: exti9_5::Context) {
        cx.resources.input_pin.clear_interrupt_pending_bit();
        cx.resources.timing.event(ClockEvent::SignalDetected(42));
    }

    #[task(binds = TIM2, priority=2, resources=[timer, timing])]
    fn tim2(cx: tim2::Context) {
        cx.resources.timer.clear_interrupt(Event::TimeOut);
        cx.resources.timing.event(ClockEvent::TimerExpired);
        //rprintln!("TIMER2");
    }

    extern "C" {
        fn UART4();
    }
};
