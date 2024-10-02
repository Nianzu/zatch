
use core::convert::TryInto;
use std::fmt::DebugList;

use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};

// use esp_idf_svc::hal::{spi};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{self, AsyncWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop,  nvs::EspDefaultNvsPartition};

use esp_idf_svc::sntp;

use esp_idf_hal::gpio::{self, OutputPin, PinDriver};
use esp_idf_hal::spi::{
  self,
  config::{Config, Mode, Phase, Polarity},
  SpiDeviceDriver,
};
use esp_idf_hal::cpu::Core;
use esp_idf_hal::delay::Delay;
use gc9a01::{prelude::*, Gc9a01, SPIDisplayInterface};
use esp_idf_hal::prelude::*;
use std::sync;
use std::time::Instant;

use esp_idf_svc::hal::spi::Operation::DelayNs;
use std::time::Duration;

// use lvgl::input_device::{
//   pointer::{Pointer, PointerInputData},
//   InputDriver,
// };
// use lvgl::style::Style;
// use lvgl::widgets::Arc;
// use lvgl::{Align, Color, Display, DrawBuffer, Part, Widget};

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
    let mosi = pins.gpio10;
    let lcd_cs = pins.gpio3;
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
    std::thread::sleep(core::time::Duration::from_secs(1));


    let mut backlight_output = PinDriver::output(backlight.downgrade_output())?;
    let dc_output = PinDriver::output(dc.downgrade_output())?;

    let config: Config = Config::new().baudrate(40.MHz().into()).data_mode(Mode {
      polarity: Polarity::IdleLow,
      phase: Phase::CaptureOnFirstTransition,
  });
  let spi_device = SpiDeviceDriver::new(driver, Some(lcd_cs), &config)?;
  let interface = SPIDisplayInterface::new(spi_device, dc_output);
  
  

  let mut display_driver = Box::new(Gc9a01::new(
    interface,
    DisplayResolution240x240,
    DisplayRotation::Rotate180,
  ));
  // .into_buffered_graphics();
  

  log::info!("init: {:?}",display_driver.init_with_addr_mode(&mut Delay::new_default()));
  log::info!("clear: {:?}",display_driver.clear());
  // log::info!("brightness: {:?}",display_driver.set_brightness(brightness));
  log::info!("dim: {:?}", display_driver.dimensions());

  backlight_output.set_low()?;
  std::thread::sleep(core::time::Duration::from_secs(1));
  backlight_output.set_high()?;
  std::thread::sleep(core::time::Duration::from_secs(1));

  log::info!("dim: {:?}", display_driver.draw_buffer(&vec![0; 20000]));
  std::thread::sleep(core::time::Duration::from_secs(5));

  // display_driver.set_write_mode().unwrap();
  // loop
  // {
  //   log::info!("Low");
  //   display_driver.set_pixel(100, 100, 0).unwrap();
  //   std::thread::sleep(core::time::Duration::from_secs(1));
  //   log::info!("High");
  //   display_driver.set_pixel(100, 100, 65000).unwrap();
  //   std::thread::sleep(core::time::Duration::from_secs(1));
  // }
  // display_driver.init(&mut Delay::new_default()).ok();
  // display_driver.clear();
  // display_driver.fill(16);

  // display_driver.flush().ok();


    // Backlight
    // loop
    // {
    //   log::info!("Low");
    //   backlight_output.set_low()?;
    //   std::thread::sleep(core::time::Duration::from_secs(1));
    //   log::info!("High");
    //   backlight_output.set_high()?;
    //   std::thread::sleep(core::time::Duration::from_secs(1));
    // }
    
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