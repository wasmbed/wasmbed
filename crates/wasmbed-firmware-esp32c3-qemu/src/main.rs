#![no_std]
#![no_main]

use esp_backtrace as _;
use defmt_serial as _;

use defmt::info;
use defmt::debug;


use embassy_executor::Spawner;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::rng::Rng;

use static_cell::StaticCell;

use esp_alloc::heap_allocator;
use embassy_net::Config;
use embassy_net::StackResources;
use embassy_net::DhcpConfig;

use esp_hal::{
    Async,
    uart::{AtCmdConfig, RxConfig, Uart},
};
static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

const HEAP_MEMORY_SIZE: usize = 72 * 1024;

const READ_BUF_SIZE: usize = 64;

const AT_CMD: u8 = 0x04;

type SerialPort = esp_hal::uart::Uart<'static, Async>;

static SERIAL: StaticCell<SerialPort> = StaticCell::new();

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {

    let peripherals = esp_hal::init(esp_hal::Config::default());

    heap_allocator!(size: HEAP_MEMORY_SIZE);

    let timg1 = TimerGroup::new(peripherals.TIMG1);

    esp_hal_embassy::init(timg1.timer0);

    esp_println::println!(">>> Hello from esp_println");

    let (tx_pin, rx_pin) = (peripherals.GPIO21, peripherals.GPIO20);

    let config = esp_hal::uart::Config::default()
        .with_rx(RxConfig::default().with_fifo_full_threshold(READ_BUF_SIZE as u16));

    let mut uart0 = Uart::new(peripherals.UART0, config)
        .unwrap()
        .with_tx(tx_pin)
        .with_rx(rx_pin)
        .into_async();

    uart0.set_at_cmd(AtCmdConfig::default().with_cmd_char(AT_CMD));

    let uart: &'static mut _ = SERIAL.init(uart0);

    defmt_serial::defmt_serial(uart);    

    embassy_time::Timer::after_secs(5).await;

    esp_println::println!("Initializing the firmware..");

    let mut rng = Rng::new(peripherals.RNG);

    let _seed = (rng.random() as u64) << 32 | rng.random() as u64;

    info!("Using random seed");

    let _stack_resources: &'static mut _ = STACK_RESOURCES.init(StackResources::new());

    info!("Initialized stack resources");

    let _config = Config::dhcpv4(DhcpConfig::default());

    debug!("Wait for network link");
}
