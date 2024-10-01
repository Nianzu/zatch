
use core::convert::TryInto;
use std::fmt::DebugList;

use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};

use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{self, AsyncWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PWD");


fn main()  -> anyhow::Result<()>{
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let timer_service = EspTaskTimerService::new().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi = AsyncWifi::wrap(
      EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
      sys_loop,
      timer_service,
    ).unwrap();

    block_on(connect_wifi(&mut wifi)).unwrap();

    let ip_info = wifi.wifi().sta_netif().get_ip_info().unwrap();

    log::info!("WiFi DHCP info: {:?}", ip_info);

    log::info!("Shutting down in 5...");
    std::thread::sleep(core::time::Duration::from_secs(5));

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

  wifi.set_configuration(&wifi_configuration).unwrap();

  wifi.start().await.unwrap();
  log::info!("Wifi started");
  wifi.connect().await.unwrap();
  log::info!("Wifi connected");
  wifi.wait_netif_up().await.unwrap();
  log::info!("Wifi netif up");

  Ok(())
}