
use core::convert::TryInto;
use std::fmt::DebugList;

use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};

use esp_idf_svc::hal::{spi};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{self, AsyncWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop,  nvs::EspDefaultNvsPartition};

use esp_idf_svc::sntp;

use esp_idf_hal::gpio::{self, OutputPin, PinDriver};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PWD");


fn main()  -> anyhow::Result<()>{
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let pins = peripherals.pins;

    let sck = pins.gpio8;
    let mosi = pins.gpio9;
    let lcd_cs = pins.gpio4;
    let dc = pins.gpio5;
    // let reset = None;
    let backlight = pins.gpio21;

    let driver: spi::SpiDriver<'_> = spi::SpiDriver::new(
      peripherals.spi2,
      sck,
      mosi,
      None::<gpio::AnyIOPin>,
      &spi::SpiDriverConfig::new(),
    )?;

    log::info!("Driver configured!");


    let mut backlight_output = PinDriver::output(backlight.downgrade_output())?;

    loop
    {

      log::info!("Low");
      backlight_output.set_low()?;
      std::thread::sleep(core::time::Duration::from_secs(1));
      log::info!("High");
      backlight_output.set_high()?;
      std::thread::sleep(core::time::Duration::from_secs(1));

    }
    
    // WIFI
    log::info!("SSID: {} PWD: {}",SSID, PASSWORD);
    // let mut wifi = AsyncWifi::wrap(
    //   EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
    //   sys_loop,
    //   timer_service,
    // )?;
    // block_on(connect_wifi(&mut wifi))?;
    // let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    // log::info!("WiFi DHCP info: {:?}", ip_info);
    
    // SNTP
    // let _sntp = sntp::EspSntp::new_default()?;
    // log::info!("SNTP initialized");
    // log::info!("Current time: {:?}", std::time::SystemTime::now());
    // log::info!("Shutting down in 5...");
    // std::thread::sleep(core::time::Duration::from_secs(5));

    Ok(())
}

async fn connect_wifi(wifi: &mut AsyncWifi<EspWifi<'static>>) -> anyhow::Result<()>{
  let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration{
    ssid: SSID.try_into().unwrap(),
    bssid: None,
    auth_method: AuthMethod::WPA2Personal,
    password: PASSWORD.try_into().unwrap(),
    channel: None,
    ..Default::default()
  });

  wifi.set_configuration(&wifi_configuration)?;

  wifi.start().await?;
  log::info!("Wifi started");
  wifi.connect().await?;
  log::info!("Wifi connected");
  wifi.wait_netif_up().await?;
  log::info!("Wifi netif up");

  Ok(())
}