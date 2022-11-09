//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use embedded_time::fixed_point::FixedPoint;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use pimoroni_badger2040 as bsp;

use bsp::hal::clocks::Clock;

// Bring in all the rest of our dependencies from the BSP
use bsp::hal;
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};
use embedded_time::duration::*;
use embedded_time::rate::units::Extensions;
use pimoroni_badger2040::prelude::*;
use uc8151::WIDTH;

use tinybmp::Bmp;
#[entry]
fn main() -> ! {
    info!("Program start");
    // Get all the basic peripherals, and init clocks/timers
    let mut board = bsp::Board::take().unwrap();
    // Enable 3.3V power or you won't see anything
    let mut power = board.pins.p3v3_en.into_push_pull_output();
    let _ = power.set_high();

    let mut count_down = board.timer.count_down();
    let mut led_pin = board.pins.led.into_push_pull_output();

    // TODO: use buttons somehow
    let _buttons = bsp::Buttons {
        a: board.pins.sw_a.into_floating_input(),
        b: board.pins.sw_b.into_floating_input(),
        c: board.pins.sw_c.into_floating_input(),
        up: board.pins.sw_up.into_floating_input(),
        down: board.pins.sw_down.into_floating_input(),
    };

    // Set up the pins for the e-ink display
    let _spi_sclk = board.pins.sclk.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_mosi = board.pins.mosi.into_mode::<hal::gpio::FunctionSpi>();
    let spi = hal::Spi::<_, _, 8>::new(board.SPI0);
    let mut dc = board.pins.inky_dc.into_push_pull_output();
    let mut cs = board.pins.inky_cs_gpio.into_push_pull_output();
    let busy = board.pins.inky_busy.into_pull_up_input();
    let reset = board.pins.inky_res.into_push_pull_output();
    let spi = spi.init(
        &mut board.RESETS,
        board.clocks.peripheral_clock.freq(),
        1_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );

    let _ = dc.set_high();
    let _ = cs.set_high();

    let mut display = uc8151::Uc8151::new(spi, cs, dc, busy, reset);
    // Reset display
    display.disable();
    count_down.start(<Milliseconds>::new(10));
    let _ = nb::block!(count_down.wait());
    display.enable();
    count_down.start(<Milliseconds>::new(10));
    let _ = nb::block!(count_down.wait());
    // Wait for the screen to finish reset
    while display.is_busy() {}

    let mut delay =
        cortex_m::delay::Delay::new(board.SYST, board.clocks.system_clock.freq().integer());

    // Initialise display. Using the default LUT speed setting
    let _ = display.setup(&mut delay, uc8151::LUT::Internal);
    let text = "Hi! I'm 9names.\nTalk to\nme about\nEmbedded Rust!";
    // Note we're setting the Text color to `Off`. The driver is set up to treat Off as Black so that BMPs work as expected.
    let character_style = MonoTextStyle::new(&FONT_9X18_BOLD, BinaryColor::Off);
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Center)
        //.vertical_alignment(embedded_text::alignment::VerticalAlignment::Top)
        .paragraph_spacing(6)
        .build();
    // Bounding box for our text. Fill it with the opposite color so we can read the text.
    let bounds = Rectangle::new(Point::new(157, 10), Size::new(WIDTH - 157, 0));
    bounds
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display)
        .unwrap();
    // Create the text box and apply styling options.
    let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);
    // Draw the text box.
    text_box.draw(&mut display).unwrap();

    // Draw ferris
    let data = include_bytes!("../assets/ferris_intent_1bpp.bmp");
    // Draw ferris backwards!
    let data2 = include_bytes!("../assets/ferris_intent_1bpp_reverse.bmp");
    let tga: Bmp<BinaryColor> = Bmp::from_slice(data).unwrap();
    let image = Image::new(&tga, Point::zero());
    let tga2: Bmp<BinaryColor> = Bmp::from_slice(data2).unwrap();
    let image2 = Image::new(&tga2, Point::zero());
    let _ = image.draw(&mut display);
    let _ = display.update();
    let mut counter = 0;
    loop {
        // blink once a second
        led_pin.toggle().unwrap();
        count_down.start(1000_u32.milliseconds());
        let _ = nb::block!(count_down.wait());
        counter += 1;
        // every two minutes, reverse ferris
        if counter == 120 {
            let _ = display.clear(BinaryColor::On);
            let _ = image2.draw(&mut display);
            text_box.draw(&mut display).unwrap();
            let _ = display.update();
        }
        // at 4 minutes, put ferris back the right way.
        if counter == 240 {
            let _ = display.clear(BinaryColor::On);
            let _ = image.draw(&mut display);
            text_box.draw(&mut display).unwrap();
            let _ = display.update();
            counter = 0;
        }
    }
}

// End of file
