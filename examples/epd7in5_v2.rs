use std::error::Error;

// This example tests rotations and draws analog clock, tests default fonts of embedded-graphics crate and displays an image of Ferris from examples/assets/ directory.
use embedded_graphics::{
    image::Image,
    image::ImageRaw,
    mono_font::ascii::*,
    mono_font::MonoTextStyleBuilder,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyleBuilder},
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_hal::delay::DelayNs;
#[cfg(feature = "graphics")]
use epd_waveshare::{color::Color, epd7in5_v2::*, graphics::DisplayRotation, prelude::*};
use linux_embedded_hal::{
    gpio_cdev::{Chip, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, Delay, SpidevDevice,
};

// GPIO pin definitions (BCM numbering - no offset needed for cdev)
const EPD_RST_PIN: u32 = 17;
const EPD_DC_PIN: u32 = 25;
const EPD_BUSY_PIN: u32 = 24;
const EPD_PWR_PIN: u32 = 18;

fn main() -> Result<(), Box<dyn Error>> {
    // Set up the device
    // Open the GPIO chip (usually gpiochip0 on Raspberry Pi)
    let mut chip = Chip::new("/dev/gpiochip0")?;

    // Get GPIO lines and configure them
    let rst_line = chip.get_line(EPD_RST_PIN)?;
    let rst_handle = rst_line.request(LineRequestFlags::OUTPUT, 0, "epd-rst")?;
    let rst_pin = CdevPin::new(rst_handle)?;

    let dc_line = chip.get_line(EPD_DC_PIN)?;
    let dc_handle = dc_line.request(LineRequestFlags::OUTPUT, 0, "epd-dc")?;
    let dc_pin = CdevPin::new(dc_handle)?;

    let busy_line = chip.get_line(EPD_BUSY_PIN)?;
    let busy_handle = busy_line.request(LineRequestFlags::INPUT, 0, "epd-busy")?;
    let busy_pin = CdevPin::new(busy_handle)?;

    let pwr_line = chip.get_line(EPD_PWR_PIN)?;
    let pwr_handle = pwr_line.request(LineRequestFlags::OUTPUT, 1, "epd-pwr")?;
    let pwr_pin = CdevPin::new(pwr_handle)?;

    // Initialize SPI
    let mut spi = SpidevDevice::open("/dev/spidev0.0")?;
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(10_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options)?;

    let mut delay = Delay {};

    let mut epd7in5 =
        Epd7in5::new(&mut spi, busy_pin, dc_pin, rst_pin, &mut delay, None).expect("epd new");
    let mut display = Display7in5::default();
    println!("Device successfully initialized!");

    // Test graphics display

    println!("Test all the rotations");

    display.set_rotation(DisplayRotation::Rotate0);
    draw_text(&mut display, "Rotate 0!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate90);
    draw_text(&mut display, "Rotate 90!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate180);
    draw_text(&mut display, "Rotate 180!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate270);
    draw_text(&mut display, "Rotate 270!", 5, 50);

    epd7in5.update_and_display_frame(&mut spi, display.buffer(), &mut delay)?;
    delay.delay_ms(5000);

    // Draw an analog clock
    println!("Draw a clock");
    display.clear(Color::White).ok();
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .build();

    let _ = Circle::with_center(Point::new(64, 64), 80)
        .into_styled(style)
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(0, 64))
        .into_styled(style)
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(80, 80))
        .into_styled(style)
        .draw(&mut display);
    epd7in5.update_and_display_frame(&mut spi, display.buffer(), &mut delay)?;
    delay.delay_ms(5000);

    // Draw some text
    println!("Print text in all sizes");
    // Color is inverted - black means white, white means black; the output will be black text on white background
    display.clear(Color::Black).ok();
    let fonts = [
        &FONT_4X6,
        &FONT_5X7,
        &FONT_5X8,
        &FONT_6X9,
        &FONT_6X10,
        &FONT_6X12,
        &FONT_6X13,
        &FONT_6X13_BOLD,
        &FONT_6X13_ITALIC,
        &FONT_7X13,
        &FONT_7X13_BOLD,
        &FONT_7X13_ITALIC,
        &FONT_7X14,
        &FONT_7X14_BOLD,
        &FONT_8X13,
        &FONT_8X13_BOLD,
        &FONT_8X13_ITALIC,
        &FONT_9X15,
        &FONT_9X15_BOLD,
        &FONT_9X18,
        &FONT_9X18_BOLD,
        &FONT_10X20,
    ];
    for (n, font) in fonts.iter().enumerate() {
        let style = MonoTextStyleBuilder::new()
            .font(font)
            .text_color(Color::White)
            .background_color(Color::Black)
            .build();
        let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
        let y = 10 + n * 30;
        let _ = Text::with_text_style(
            "Rust is awesome!",
            Point::new(20, y.try_into().unwrap()),
            style,
            text_style,
        )
        .draw(&mut display);
    }
    epd7in5.update_and_display_frame(&mut spi, display.buffer(), &mut delay)?;
    delay.delay_ms(5000);

    // Draw an image
    println!("Draw Ferris");
    display.clear(Color::Black).ok();
    let data = include_bytes!("./assets/ferris.raw");
    let raw_image = ImageRaw::<Color>::new(data, 460);
    let image = Image::new(&raw_image, Point::zero());
    image.draw(&mut display).unwrap();
    epd7in5.update_and_display_frame(&mut spi, display.buffer(), &mut delay)?;

    // Clear and sleep
    println!("Clear the display");
    display.clear(Color::Black).ok();
    epd7in5.update_and_display_frame(&mut spi, display.buffer(), &mut delay)?;
    println!("Finished tests - going to sleep");
    epd7in5.sleep(&mut spi, &mut delay)?;
    Ok(())
}

fn draw_text(display: &mut Display7in5, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(Color::White)
        .background_color(Color::Black)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}
