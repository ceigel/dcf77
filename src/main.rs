#![no_std]
#![no_main]
extern crate heapless;

use panic_rtt_target as _;

use feather_f405::hal as stm32f4xx_hal;
use heapless::String;
use ht16k33::{Dimming, Display, HT16K33};
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::prelude::*;

#[app(device = feather_f405::hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        message: String<100>,
    }
    #[init(spawn=[say_hello])]
    fn init(cx: init::Context) -> init::LateResources {
        let mut core = cx.core;
        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();
        core.SCB.set_sleepdeep();

        let device = cx.device;
        let mut rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        rtt_init_print!();
        let gpiob = device.GPIOB.split();
        let scl = gpiob.pb8.into_alternate_af4().set_open_drain();
        let sda = gpiob.pb7.into_alternate_af4().set_open_drain();
        let i2c = I2c::i2c1(device.I2C1, (scl, sda), 400.khz(), clocks);
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
        cx.spawn.say_hello().expect("To start say hello task");
        init::LateResources {
            message: "Hello world".into(),
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
    extern "C" {
        fn UART4();
    }
};
