#![no_std]
#![no_main]
extern crate heapless;

use crate::stm32f4xx_hal::i2c::I2c;
use panic_rtt_target as _;

use adafruit_7segment::{Index, SevenSegment};
use feather_f405::hal as stm32f4xx_hal;
use feather_f405::{hal::prelude::*, pac, setup_clocks};
use heapless::String;
use ht16k33::{Dimming, Display, HT16K33};
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::gpio::{gpioa, Edge, ExtiPin, Input, PullUp};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::timer::{Event, Timer};

pub struct Timing {}

impl Timing {
    pub fn hello(&self) {
        rprintln!("Hello");
    }
}

#[app(device = feather_f405::hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        message: String<100>,
        input_pin: gpioa::PA<Input<PullUp>>,
        timer: Timer<pac::TIM2>,
        timing: Timing,
    }
    #[init(spawn=[say_hello])]
    fn init(cx: init::Context) -> init::LateResources {
        let mut core = cx.core;
        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();
        core.SCB.set_sleepdeep();

        let device = cx.device;
        let clocks = setup_clocks(device.RCC);
        let mut syscfg = device.SYSCFG.constrain();
        let mut exti = device.EXTI;

        rtt_init_print!();
        let gpiob = device.GPIOB.split();
        let scl = gpiob.pb8.into_alternate_af4().set_open_drain();
        let sda = gpiob.pb7.into_alternate_af4().set_open_drain();
        let i2c = I2c::new(device.I2C1, (scl, sda), 400.khz(), clocks);
        const DISP_I2C_ADDR: u8 = 112;
        let mut ht16k33 = HT16K33::new(i2c, DISP_I2C_ADDR);
        ht16k33.initialize().expect("Failed to initialize ht16k33");
        ht16k33
            .set_display(Display::ON)
            .expect("Could not turn on the display!");
        ht16k33
            .set_dimming(Dimming::BRIGHTNESS_MIN)
            .expect("Could not set dimming!");

        // Sending individual digits
        ht16k33.update_buffer_with_digit(Index::One, 1);
        ht16k33.update_buffer_with_digit(Index::Two, 2);
        ht16k33.update_buffer_with_digit(Index::Three, 3);
        ht16k33.update_buffer_with_digit(Index::Four, 4);

        let gpioa = device.GPIOA.split();
        let mut pa6 = gpioa.pa6.into_pull_up_input().downgrade();
        pa6.make_interrupt_source(&mut syscfg);
        pa6.trigger_on_edge(&mut exti, Edge::RISING_FALLING);
        pa6.enable_interrupt(&mut exti);

        let mut timer = Timer::tim2(device.TIM2, 10.mhz(), clocks);
        timer.listen(Event::TimeOut);
        let timing = Timing {};
        rprintln!("Init successful");
        cx.spawn.say_hello().expect("To start say hello task");
        init::LateResources {
            message: "Hello world".into(),
            input_pin: pa6,
            timer,
            timing,
        }
    }

    #[allow(clippy::empty_loop)]
    #[idle()]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {}
    }

    #[task(resources=[message])]
    fn say_hello(cx: say_hello::Context) {
        rprintln!("{}", cx.resources.message);
    }

    #[task(binds = EXTI9_5, priority=8, resources=[input_pin, timing])]
    fn exti9_5(cx: exti9_5::Context) {
        cx.resources.input_pin.clear_interrupt_pending_bit();
        cx.resources.timing.hello();
        //rprintln!("SIGNAL");
    }

    #[task(binds = TIM2, priority=8, resources=[timer, timing])]
    fn tim2(cx: tim2::Context) {
        cx.resources.timer.clear_interrupt(Event::TimeOut);
        cx.resources.timing.hello();
        //rprintln!("TIMER2");
    }

    extern "C" {
        fn UART4();
    }
};
