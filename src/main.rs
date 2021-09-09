#![no_std]
#![no_main]
extern crate heapless;

use panic_itm as _;

use cortex_m_log::destination;
use cortex_m_log::log::Logger;
use cortex_m_log::printer::itm::InterruptSync;
use feather_f405::hal as stm32f4xx_hal;
use heapless::String;
use log::info;
use rtic::app;
use stm32f4xx_hal::prelude::*;

static mut LOGGER: Option<Logger<InterruptSync>> = None;
const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Debug;

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

        let logger: &Logger<InterruptSync> = unsafe {
            LOGGER.replace(Logger {
                inner: InterruptSync::new(destination::itm::Itm::new(core.ITM)),
                level: LOG_LEVEL,
            });
            LOGGER.as_ref().expect("to have a logger")
        };
        cortex_m_log::log::init(logger).expect("To set logger");
        cx.spawn.say_hello().expect("To start say hello task");
        init::LateResources {
            message: "Hello world".into(),
        }
    }

    #[allow(clippy::empty_loop)]
    #[idle()]
    fn idle(_cx: idle::Context) -> ! {
        info!("idle");
        loop {}
    }

    #[task(resources=[message])]
    fn say_hello(cx: say_hello::Context) {
        info!("{}", cx.resources.message);
    }
    extern "C" {
        fn UART4();
    }
};
