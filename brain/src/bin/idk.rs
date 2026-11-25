#![no_std]
#![no_main]
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{AnyPin, Io, Output, OutputConfig, Pin},
    main,
    time::Instant,
};

esp_bootloader_esp_idf::esp_app_desc!();
macro_rules! unwrap_or_report {
    ($expr:expr, $esp_now:expr, $module:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                send_error_msg($esp_now, $module, &e, line!());
                continue;
            }
        }
    };
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let mut peripherals =
        esp_hal::init(esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::_80MHz));

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);
    let io = Io::new(peripherals.IO_MUX);
    let delay = Delay::new();

    let mut pins: [Output; 11] = [
        Output::new(
            peripherals.GPIO2.degrade(),
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
        Output::new(
            peripherals.GPIO3.degrade(),
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
        Output::new(
            peripherals.GPIO4.degrade(),
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
        Output::new(
            peripherals.GPIO5.degrade(),
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
        Output::new(
            peripherals.GPIO6.degrade(),
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
        Output::new(
            peripherals.GPIO7.degrade(),
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
        Output::new(
            peripherals.GPIO8.degrade(),
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
        Output::new(
            peripherals.GPIO9.degrade(),
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
        Output::new(
            peripherals.GPIO10.degrade(),
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
        Output::new(
            peripherals.GPIO20.degrade(),
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
        Output::new(
            peripherals.GPIO21.degrade(),
            esp_hal::gpio::Level::Low,
            OutputConfig::default(),
        ),
    ];
    let mut pin = 0;
    loop {
        let output = get_two_mut_wrapped(&mut pins, pin);
        pin += 1;
        pin %= 11;
        output[0].set_low();
        output[1].set_high();
        delay.delay_millis(1000);
    }
}

fn get_two_mut_wrapped<T>(slice: &mut [T], index: usize) -> [&mut T; 2] {
    let len = slice.len();
    let i1 = index % len;
    let i2 = (index + 1) % len;

    assert!(i1 != i2, "Array must have at least 2 elements");

    let ptr = slice.as_mut_ptr();
    unsafe { [&mut *ptr.add(i1), &mut *ptr.add(i2)] }
}
