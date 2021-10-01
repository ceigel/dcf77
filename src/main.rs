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

use cast::u16;
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
use stm32f4xx_hal::{rcc::*, rtc::Rtc};
use time_display::{display_error, show_rtc_time};

type SegmentDisplay =
    HT16K33<I2c<pac::I2C1, (gpiob::PB6<AlternateOD<AF4>>, gpiob::PB7<AlternateOD<AF4>>)>>;

fn sync_rtc(rtc: &mut Rtc, dt: &NaiveDateTime) {
    rtc.set_datetime(dt).expect("To be able to set datetime");
}

fn enable_tim1(rcc: &mut pac::RCC) {
    pac::TIM1::enable(rcc);
    pac::TIM1::reset(rcc);
}

const ARR_MULTIPL: usize = 1;
fn start_tim1(tim1: pac::TIM1, clocks: &Clocks) -> pac::TIM1 {
    // pause
    tim1.cr1.modify(|_, w| w.cen().clear_bit());
    // reset counter
    tim1.cnt.reset();

    let ticks = clocks.pclk2().0; // for 1.hz() = 84 * 1E+6

    // let arr = u16(ticks / u32(psc + 1)).unwrap();
    let arr: u32 = 999; // 1000 bins

    let arr = arr << ARR_MULTIPL; // we can't fit more into psc
    let psc = u16((ticks / arr) - 1).unwrap(); // 42000
    tim1.psc.write(|w| w.psc().bits(psc));

    tim1.arr.write(|w| unsafe { w.bits(arr) });

    // Trigger update event to load the registers
    tim1.cr1.modify(|_, w| w.urs().set_bit());
    tim1.egr.write(|w| w.ug().set_bit());
    tim1.cr1.modify(|_, w| w.urs().clear_bit());

    // start counter
    tim1.cr1.modify(|_, w| w.cen().set_bit());

    tim1
}

const DISP_I2C_ADDR: u8 = 0x77;
#[app(device = feather_f405::hal::stm32, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        segment_display: SegmentDisplay,
        dcf_pin: gpioa::PA6<Input<PullUp>>,
        tim1: pac::TIM1,
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
        let mut device = cx.device;
        core.DCB.enable_trace();
        DWT::unlock();
        core.DWT.enable_cycle_counter();
        //core.SCB.set_sleepdeep();

        enable_tim1(&mut device.RCC);
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
        let mut pin = gpioa.pa6.into_pull_up_input();
        pin.make_interrupt_source(&mut syscfg);
        pin.trigger_on_edge(&mut exti, Edge::RisingFalling);
        pin.enable_interrupt(&mut exti);

        let tim1 = start_tim1(device.TIM1, &clocks);
        let rtc = Rtc::new(device.RTC, 255, 127, false, &mut pwr);
        rprintln!("Init successful");
        let cc = CyclesComputer::new(clocks.sysclk());
        init::LateResources {
            segment_display: ht16k33,
            dcf_pin: pin,
            tim1,
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

    #[task(binds = EXTI9_5, priority=2, resources=[dcf_pin, tim1, cycles_computer, decoder])]
    fn exti9_5(cx: exti9_5::Context) {
        let tim1 = cx.resources.tim1;
        let dcf_pin = cx.resources.dcf_pin;
        let dcf_interrupted = dcf_pin.check_interrupt();
        dcf_pin.clear_interrupt_pending_bit();
        if !dcf_interrupted {
            return;
        }
        let now = Instant::now();
        let tim1_val: u32 = tim1.cnt.read().bits() >> ARR_MULTIPL;
        let res = cx
            .resources
            .decoder
            .register_transition(dcf_pin.is_high(), now, tim1_val);
        if let Err(e) = res {
            rprintln!("Err: {:?}", e);
        }
    }

    extern "C" {
        fn UART4();
    }
};
