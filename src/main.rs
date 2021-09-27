#![no_std]
#![no_main]
extern crate heapless;
mod cycles_computer;
mod datetime_converter;
mod dcf77_decoder;
mod time_display;

use rtic::cyccnt::{Instant, U32Ext};

use crate::stm32f4xx_hal::i2c::I2c;
use datetime_converter::DCF77DateTimeConverter;
use dcf77_decoder::DCF77Decoder;
use panic_rtt_target as _;

use chrono::naive::NaiveDateTime;
use cortex_m::peripheral::DWT;
use cycles_computer::CyclesComputer;
use feather_f405::hal as stm32f4xx_hal;
use feather_f405::{hal::prelude::*, pac, setup_clocks};
use ht16k33::{Dimming, Display, HT16K33};
use rtcc::Rtcc;
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::gpio::{gpioa, gpiob, AlternateOD, Edge, Input, PullUp, AF4};
use stm32f4xx_hal::rtc::Rtc;
use time_display::{display_error, show_rtc_time};

type SegmentDisplay =
    HT16K33<I2c<pac::I2C1, (gpiob::PB6<AlternateOD<AF4>>, gpiob::PB7<AlternateOD<AF4>>)>>;

fn sync_rtc(rtc: &mut Rtc, dt: &NaiveDateTime) {
    rtc.set_datetime(dt).expect("To be able to set datetime");
}

const DISP_I2C_ADDR: u8 = 0x77;
#[app(device = feather_f405::hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        segment_display: SegmentDisplay,
        dcf_pin: gpioa::PA<Input<PullUp>>,
        cycles_computer: CyclesComputer,
        val: u16,
        decoder: DCF77Decoder,
        rtc: Rtc,
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
        let mut syscfg = device.SYSCFG.constrain();
        let mut exti = device.EXTI;
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
        display_error(&mut ht16k33, 0);
        ht16k33
            .write_display_buffer()
            .expect("Could not write 7-segment display");
        let gpioa = device.GPIOA.split();
        let mut pin = gpioa.pa6.into_pull_up_input().downgrade();
        pin.make_interrupt_source(&mut syscfg);
        pin.trigger_on_edge(&mut exti, Edge::RISING_FALLING);
        pin.enable_interrupt(&mut exti);

        let rtc = Rtc::new(device.RTC, 255, 127, false, &mut pwr);
        rprintln!("Init successful");
        let cc = CyclesComputer::new(clocks.sysclk());
        init::LateResources {
            segment_display: ht16k33,
            dcf_pin: pin,
            cycles_computer: cc.clone(),
            val: 0,
            decoder: DCF77Decoder::new(cc),
            rtc,
            synchronized: false,
        }
    }

    #[allow(clippy::empty_loop)]
    #[idle()]
    fn idle(_cx: idle::Context) -> ! {
        rprintln!("idle");
        loop {}
    }

    #[task(binds = EXTI9_5, priority=2, resources=[dcf_pin, cycles_computer, decoder])]
    fn exti9_5(cx: exti9_5::Context) {
        let dcf_pin = cx.resources.dcf_pin;
        let dcf_interrupted = dcf_pin.check_interrupt();
        dcf_pin.clear_interrupt_pending_bit();
        if !dcf_interrupted {
            return;
        }
        let now = Instant::now();
        let res = cx
            .resources
            .decoder
            .register_transition(dcf_pin.is_high().unwrap(), now);
        if let Err(e) = res {
            rprintln!("Err: {:?}", e);
        }
    }

    extern "C" {
        fn UART4();
    }
};
