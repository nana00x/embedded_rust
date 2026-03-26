#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32l4xx_hal::{
    delay::Delay,
    pac,
    prelude::*,
};

struct Lpuart1;

impl Lpuart1 {
    fn init(sysclk_hz: u32, baud: u32) {
        unsafe {
            let rcc   = &(*pac::RCC::ptr());
            let gpioc = &(*pac::GPIOC::ptr());
            let lp    = &(*pac::LPUART1::ptr());

            rcc.ahb2enr.modify(|_, w| w.gpiocen().set_bit());
            rcc.apb1enr2.modify(|_, w| w.lpuart1en().set_bit());

            gpioc.moder.modify(|_, w| {
                w.moder0().alternate()
                 .moder1().alternate()
            });
            gpioc.afrl.modify(|_, w| {
                w.afrl0().bits(8)
                 .afrl1().bits(8)
            });

            let brr = (256_u64 * sysclk_hz as u64 / baud as u64) as u32;
            lp.brr.write(|w| w.bits(brr));

            lp.cr1.write(|w| {
                w.ue().set_bit()
                 .te().set_bit()
            });
        }
    }

    fn write_byte(byte: u8) {
        unsafe {
            let lp = &(*pac::LPUART1::ptr());
            while lp.isr.read().txe().bit_is_clear() {}
            lp.tdr.write(|w| w.tdr().bits(byte as u16));
        }
    }
}

impl Write for Lpuart1 {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            Lpuart1::write_byte(byte);
        }
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    Lpuart1::init(80_000_000, 115200);
    let mut serial = Lpuart1;

    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc   = dp.RCC.constrain();
    let mut pwr   = dp.PWR.constrain(&mut rcc.apb1r1);

    let clocks = rcc.cfgr.sysclk(80.MHz()).freeze(&mut flash.acr, &mut pwr);
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let mut led = gpioa.pa8.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

    let mut counter: u32 = 0;

    loop {
        led.set_high();
        writeln!(serial, "[{}] LED ON\r", counter).unwrap();
        delay.delay_ms(500u32);

        led.set_low();
        writeln!(serial, "[{}] LED OFF\r", counter).unwrap();
        delay.delay_ms(500u32);

        counter = counter.wrapping_add(1);
    }
}