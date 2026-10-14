#![no_std]
#![no_main]

use ch32_hal as hal;
use ch32_hal::spi::Spi;
use hal::delay::Delay;
use hal::gpio::{Level, Output};
use hal::peripherals::SPI1;
use hal::mode::Blocking;

#[qingke_rt::entry]
fn main() -> ! {
    hal::debug::SDIPrint::enable();

    hal::println!();

    let mut config = hal::Config::default();
    config.rcc = hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSE;
    let p = hal::init(config);

    let mut delay = Delay;
    let mut led = Output::new(p.PD4, Level::Low, Default::default());
    led.set_high();

    let spi_config = hal::spi::Config::default();
    let mut spi = Spi::new_blocking_txonly::<0>(p.SPI1, p.PC5, p.PC6, spi_config);
    let mut cs_load = Output::new(p.PC1, Level::High, Default::default());

    let mut display = MAX7219::new(spi, cs_load);

    display.init();

    for _ in 0..2 {
        display.all_on();
        delay.delay_ms(500);
        display.all_off();
        delay.delay_ms(500);    
    }

    loop {
        for y in 0..8 {
            for x in 0..32 {
                display.set_pixel(x, y, true);
                display.write();
                delay.delay_ms(100);
                display.set_pixel(x, y, false);
            }
        }
    }
}

struct MAX7219 {
    spi: Spi<'static, SPI1, Blocking>,
    cs_load: Output<'static>,
    display_buffer: [u8; 32],
}

impl MAX7219 {
    fn new(spi: Spi<'static, SPI1, Blocking>, cs_load: Output<'static>) -> Self {
        Self { spi, cs_load, display_buffer: [0_u8; 32] }
    }
    fn init(&mut self) {
        self.write_all(0x0F, 0x00);
        self.write_all(0x0B, 0x07);
        self.write_all(0x09, 0x00);
        for y in 1..=8 {
            self.write_all(y, 0x00);        
        }
        self.write_all(0x0c, 0x01);
    }

    fn write_all(&mut self, reg: u8, data: u8) {
        self.cs_load.set_low();
        for _ in 0..4 {
            self.spi.blocking_write(&[reg, data]).unwrap();
        }
        self.cs_load.set_high();
    }

    fn all_on(&mut self) {
        for row in 1..=8_u8 {
            self.write_all(row, 0xFF);
        }
    }
    
    fn all_off(&mut self) {
        for row in 1..=8_u8 {
            self.write_all(row, 0x00);
        }
    }

    fn write(&mut self) {
        for row in 1..=8_u8 {
            self.cs_load.set_low();
            for d in 0..4 {
                self.spi.blocking_write(&[row, self.display_buffer[(8 * d + row-1) as usize]]).unwrap();
            }
            self.cs_load.set_high();
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, value: bool) {
        // x is left to right, 0-31
        // y is top to bottom, 0-7
        let display_number = 3 - (x / 8); // 0 is rightmost
        let byte_bit = x % 8;
        let shifted = 1_u8 << byte_bit;
        if value {
            self.display_buffer[display_number * 8 + (7 - y)] |= shifted;
        } else {
            self.display_buffer[display_number * 8 + (7 - y)] &= !shifted;
        }
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
