#![no_std]
#![no_main]

use defmt::info;
use defmt_serial as _;
use embassy_executor::Spawner;
use hifive1::{
    pin, hal::DeviceResources, hal::delay::Sleep, hal::e310x::Uart0,
    hal::serial::Serial,
};
use panic_halt as _;
use static_cell::StaticCell;

/// UART0 configured with IOF0 pins 17 (TX) and 16 (RX).
type SerialPort = Serial<
    Uart0,
    hifive1::hal::gpio::gpio0::Pin17<
        hifive1::hal::gpio::IOF0<hifive1::hal::gpio::NoInvert>,
    >,
    hifive1::hal::gpio::gpio0::Pin16<
        hifive1::hal::gpio::IOF0<hifive1::hal::gpio::NoInvert>,
    >,
>;

static SERIAL: StaticCell<SerialPort> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    use hifive1::hal::time::U32Ext;
    use hifive1::hal::prelude::DelayNs;

    let cpu_frequency = 320.mhz();
    let serial_baud_rate = 115_200.bps();

    let Some(device_resources) = DeviceResources::take() else {
        // This panic won't show because UART isn't initialized yet.
        defmt::panic!("DeviceResources already taken");
    };

    let peripherals = device_resources.peripherals;
    let pins = device_resources.pins;

    let clocks = hifive1::clock::configure(
        peripherals.PRCI,
        peripherals.AONCLK,
        cpu_frequency.into(),
    );

    let mut sleep = Sleep::new(clocks);

    let tx = pin!(pins, uart0_tx).into_iof0();
    let rx = pin!(pins, uart0_rx).into_iof0();

    let serial = SERIAL.init(Serial::new(
        peripherals.UART0,
        (tx, rx),
        serial_baud_rate,
        clocks,
    ));
    defmt_serial::defmt_serial(serial);

    sleep.delay_ms(100);
    info!("Hello from RISC-V on QEMU!");
}
