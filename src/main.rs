#![no_std]
#![no_main]

use ch32_hal as hal;
use ch32_hal::spi::Spi;
use hal::delay::Delay;
use hal::gpio::{Level, Output};
use hal::mode::Blocking;
use hal::peripherals::SPI1;

pub mod font_blokus;
pub mod font_simple;
pub mod font_zypix;

#[qingke_rt::entry]
fn main() -> ! {
    hal::debug::SDIPrint::enable();

    let mut config = hal::Config::default();
    config.rcc = hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSE;
    let p = hal::init(config);

    let mut delay = Delay;
    let mut led = Output::new(p.PD4, Level::Low, Default::default());
    led.set_high();

    let spi_config = hal::spi::Config::default();
    let spi = Spi::new_blocking_txonly::<0>(p.SPI1, p.PC5, p.PC6, spi_config);
    let cs_load = Output::new(p.PC1, Level::High, Default::default());

    let mut display = MAX7219::new(spi, cs_load);

    display.init();

    for _ in 0..1 {
        display.all_on();
        delay.delay_ms(500);
        display.all_off();
        delay.delay_ms(500);
    }

    let mut offset = 33;
    let message = "Hello, CH32V003!";
    let message_size: isize = ((message.len() * 6) + 32).try_into().unwrap();
    loop {
        hal::println!("{}", offset);
        if offset < 33 - message_size {
            hal::println!("reset");
            offset = 33;
        }
        display.put_string(message, 6, offset);
        display.write();
        offset -= 1;
        delay.delay_ms(50);
        display.clear();
    }
}

struct MAX7219 {
    spi: Spi<'static, SPI1, Blocking>,
    cs_load: Output<'static>,
    display_buffer: [u8; 32],
}

impl MAX7219 {
    fn new(spi: Spi<'static, SPI1, Blocking>, cs_load: Output<'static>) -> Self {
        Self {
            spi,
            cs_load,
            display_buffer: [0_u8; 32],
        }
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
                self.spi
                    .blocking_write(&[row, self.display_buffer[(8 * d + row - 1) as usize]])
                    .unwrap();
            }
            self.cs_load.set_high();
        }
    }

    fn set_pixel(&mut self, x: isize, y: isize, value: bool) {
        // x is left to right, 0-31
        // y is top to bottom, 0-7
        if x > 31 || y > 7 || x < 0 || y < 0 {
            return;
        }
        let display_number = 3 - (x / 8); // 0 is rightmost
        let byte_bit = x % 8;
        let shifted = 1_u8 << byte_bit;
        let display_index: usize = (display_number * 8 + (7 - y)).try_into().unwrap();
        if value {
            self.display_buffer[display_index] |= shifted;
        } else {
            self.display_buffer[display_index] &= !shifted;
        }
    }

    fn clear(&mut self) {
        self.display_buffer = [0_u8; 32];
    }

    fn put_char(&mut self, c: char, x: isize) {
        // x, y, is top left corner
        let character_bits = font_simple::FONT[c as usize - 32];
        for col in 0..6 {
            for row in 0..8 {
                let col_u: usize = col.try_into().unwrap();
                self.set_pixel(x + col, row, 0 != (character_bits[col_u] >> row & 1));
            }
        }
    }

    fn put_string(&mut self, s: &str, width: isize, x: isize) {
        for (i, c) in s.chars().enumerate() {
            let i_i: isize = i.try_into().unwrap();
            self.put_char(c, width * i_i + x);
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
