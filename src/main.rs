#![no_std]
#![no_main]

use core::ops::Range;

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use sequential_storage::cache::NoCache;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);

    let flash = embassy_nrf::nvmc::Nvmc::new(p.NVMC);
    let mut flash = embassy_embedded_hal::adapter::BlockingAsync::new(flash);

    const FLASH_RANGE: Range<u32> = 0xFC000..0x100000;
    let mut data_buffer = [0; 32];

    sequential_storage::map::store_item(
        &mut flash,
        FLASH_RANGE,
        &mut NoCache::new(),
        &mut data_buffer,
        0u8,
        &0u8,
    )
    .await
    .ok();

    sequential_storage::map::store_item(
        &mut flash,
        FLASH_RANGE,
        &mut NoCache::new(),
        &mut data_buffer,
        1u8,
        &123u32,
    )
    .await
    .ok();

    sequential_storage::map::store_item(
        &mut flash,
        FLASH_RANGE,
        &mut NoCache::new(),
        &mut data_buffer,
        2u8,
        &0.123f32,
    )
    .await
    .ok();

    let _fetched = sequential_storage::map::fetch_item::<u8, u32, _>(
        &mut flash,
        FLASH_RANGE,
        &mut NoCache::new(),
        &mut data_buffer,
        1,
    )
    .await
    .ok();

    loop {
        led.set_high();
        Timer::after_millis(300).await;
        led.set_low();
        Timer::after_millis(300).await;
    }
}

