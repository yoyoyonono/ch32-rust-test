#![no_std]
#![no_main]

use ch32_hal as hal;
use hal::delay::Delay;
use hal::gpio::{Level, Output};

#[qingke_rt::entry]
fn main() -> ! {
    #[cfg(debug_assertions)] 
    {
        hal::debug::SDIPrint::enable();
    }  
    
    let mut config = hal::Config::default();
    config.rcc = hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSE;
    let p = hal::init(config);

    let mut delay = Delay;
    let mut led = Output::new(p.PD4, Level::Low, Default::default());

    loop {
        led.toggle();

        delay.delay_ms(1000);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(debug_assertions)]
    {
        let _ = hal::println!("\n\n\n{}", info);
    }
    loop {}
}
