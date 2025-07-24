#![no_std]
#![no_main]

use defmt_serial as _;
use esp_backtrace as _;
use embassy_executor::Spawner;

use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use embassy_time::{Duration, Timer};

use esp_wifi::wifi::{Configuration, ClientConfiguration, WifiEvent, WifiState};
use esp_wifi::wifi::WifiController;
use esp_wifi::wifi::WifiDevice;
use esp_wifi::EspWifiController;

use static_cell::StaticCell;

use esp_alloc::heap_allocator;

use embassy_net::Config;
use embassy_net::StackResources;
use embassy_net::DhcpConfig;
use embassy_net::Runner;
use embassy_net::Stack;
use embassy_net::IpEndpoint;

use esp_hal::peripherals::RADIO_CLK;
use esp_hal::peripherals::TIMG0;
use esp_hal::peripherals::WIFI;

use esp_hal::timer::timg::TimerGroup;

use esp_hal::rng::Rng;
use rand_core::{RngCore, CryptoRng};

use wasmbed_protocol_client::{Client};

static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

static WIFI_CONTROLLER: StaticCell<EspWifiController<'static>> =
    StaticCell::new();

pub static STOP_WIFI_SIGNAL: Signal<CriticalSectionRawMutex, ()> =
    Signal::new();

const HEAP_MEMORY_SIZE: usize = 72 * 1024;

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

esp_bootloader_esp_idf::esp_app_desc!();

#[derive(Clone)]
struct Esp32c3RngWrapper(Rng);

impl From<Rng> for Esp32c3RngWrapper {
    fn from(rng: Rng) -> Self {
        Self(rng)
    }
}

impl RngCore for Esp32c3RngWrapper {
    fn next_u32(&mut self) -> u32 {
        self.0.random()
    }

    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | (self.next_u32() as u64)
    }

    fn fill_bytes(&mut self, dst: &mut [u8]) {
        for chunk in dst.chunks_mut(4) {
            let bytes = self.next_u32().to_le_bytes();
            let (head, _) = bytes.split_at(chunk.len());
            chunk.copy_from_slice(head);
        }
    }

    fn try_fill_bytes(
        &mut self,
        dest: &mut [u8],
    ) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for Esp32c3RngWrapper {}

#[embassy_executor::task]
async fn run(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}

#[embassy_executor::task]
async fn wifi_connection(mut controller: WifiController<'static>) {
    loop {
        let is_started = match controller.is_started() {
            Ok(started) => started,
            Err(e) => {
                esp_println::println!(
                    "Error checking Wi-Fi controller status: {e:?}"
                );
                false
            },
        };
        if !is_started {
            let cfg = Configuration::Client(ClientConfiguration {
                ssid: SSID.into(),
                password: PASSWORD.into(),
                ..Default::default()
            });
            if let Err(e) = controller.set_configuration(&cfg) {
                esp_println::println!(
                    "Error setting Wi-Fi configuration: {e:?}"
                );
                let _ = Timer::after(Duration::from_secs(5)).await;
                continue;
            }
            match controller.start_async().await {
                Ok(()) => {
                    esp_println::println!("[Log] Wi-Fi Controller started")
                },
                Err(e) => {
                    esp_println::println!(
                        "Error starting Wi-Fi controller: {e:?}"
                    );
                    let _ = Timer::after(Duration::from_secs(5)).await;
                    continue;
                },
            }
        }

        if !matches!(esp_wifi::wifi::wifi_state(), WifiState::StaConnected) {
            match controller.connect_async().await {
                Ok(()) => esp_println::println!("Connected to {:?}!", SSID),
                Err(e) => {
                    esp_println::println!("Wi-Fi connect error: {e:?}");
                    let _ = Timer::after(Duration::from_secs(5)).await;
                    continue;
                },
            }
        }
        controller.wait_for_event(WifiEvent::StaDisconnected).await;
    }
}

async fn init_wifi(
    timg0: TimerGroup<'static, TIMG0<'static>>,
    mut rng: Rng,
    wifi: WIFI<'static>,
    radio_clk: RADIO_CLK<'static>,
    spawner: Spawner,
) -> Result<Stack<'static>, Error> {
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    esp_println::println!("Using random seed {:?}", seed);

    let wifi_controller = esp_wifi::init(timg0.timer0, rng, radio_clk)?;
    let wifi_controller: &'static mut _ = WIFI_CONTROLLER.init(wifi_controller);

    let (controller, wifi_interfaces) =
        esp_wifi::wifi::new(wifi_controller, wifi)?;
    let wifi_interface = wifi_interfaces.sta;

    let stack_resources: &'static mut _ =
        STACK_RESOURCES.init(StackResources::new());

    let config = Config::dhcpv4(DhcpConfig::default());
    let (stack, runner) =
        embassy_net::new(wifi_interface, config, stack_resources, seed);

    spawner.must_spawn(wifi_connection(controller));
    spawner.must_spawn(run(runner));

    esp_println::println!("Initialized stack resources");

    Ok(stack)
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    heap_allocator!(size: HEAP_MEMORY_SIZE);

    let timg1 = TimerGroup::new(peripherals.TIMG1);
    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_hal_embassy::init(timg1.timer0);

    esp_println::println!("Firmware initialized");
    esp_println::println!("SSID: {SSID:?}");

    let rng = Rng::new(peripherals.RNG);

    let stack = match init_wifi(
        timg0,
        rng,
        peripherals.WIFI,
        peripherals.RADIO_CLK,
        spawner,
    )
    .await
    {
        Ok(stack) => stack,
        Err(e) => {
            esp_println::println!("WiFi init error: {:?}", e);
            return;
        },
    };

    loop {
        if stack.is_link_up() {
            break;
        }
        embassy_time::Timer::after_millis(500).await;
    }

    let mut hal_rng = Esp32c3RngWrapper::from(rng);
    let mut client = Client::new(&stack);
    esp_println::println!("Wasmbed Client created");
    esp_println::println!("Test Tcp Connection with gateway");

    let endpoint = IpEndpoint::new(
        embassy_net::IpAddress::Ipv4(embassy_net::Ipv4Address::new(0, 0, 0, 0)),
        4423,
    );
    if let Err(e) = client.connect_tls(endpoint, &mut hal_rng, "", "").await {
        esp_println::println!("[ERR] TLS connect: {e:?}");
        loop {
            embassy_time::Timer::after_secs(5).await;
        }
    }
    esp_println::println!("[OK] TLS handshake");
    match client.send_heartbeat().await {
        Ok(n) => esp_println::println!("[OK] Heartbeat ACK – {n:?} bytes"),
        Err(e) => esp_println::println!("[ERR] Heartbeat: {e:?}"),
    }

    loop {
        embassy_time::Timer::after_secs(30).await;
        if let Err(e) = client.send_heartbeat().await {
            esp_println::println!("[ERR] HB: {e:?}");
        }
    }
}

#[derive(Debug)]
pub enum Error {
    WifiInitialization(esp_wifi::InitializationError),
    Wifi(esp_wifi::wifi::WifiError),
}

impl From<esp_wifi::InitializationError> for Error {
    fn from(error: esp_wifi::InitializationError) -> Self {
        Self::WifiInitialization(error)
    }
}

impl From<esp_wifi::wifi::WifiError> for Error {
    fn from(error: esp_wifi::wifi::WifiError) -> Self {
        Self::Wifi(error)
    }
}
