#![no_std]
#![no_main]

use core::ops::Range;

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use sequential_storage::{cache::NoCache, map::StorageItem};
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
        &ConfigItem::A,
    )
    .await
    .ok();

    sequential_storage::map::store_item(
        &mut flash,
        FLASH_RANGE,
        &mut NoCache::new(),
        &mut data_buffer,
        &ConfigItem::B(123),
    )
    .await
    .ok();

    sequential_storage::map::store_item(
        &mut flash,
        FLASH_RANGE,
        &mut NoCache::new(),
        &mut data_buffer,
        &ConfigItem::C(0.123),
    )
    .await
    .ok();

    let _fetched = sequential_storage::map::fetch_item::<ConfigItem, _>(
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

enum ConfigItem {
    A,
    B(u8),
    C(f32),
}

impl StorageItem for ConfigItem {
    type Key = u8;
    type Error = ();

    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        buffer[0] = self.key();

        let data_len = match self {
            ConfigItem::A => 0,
            ConfigItem::B(val) => {
                buffer[1] = *val;
                1
            }
            ConfigItem::C(val) => {
                buffer[1..][..4].copy_from_slice(&val.to_le_bytes());
                4
            }
        };

        Ok(1 + data_len)
    }

    fn deserialize_from(buffer: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match buffer[0] {
            0 => Ok(ConfigItem::A),
            1 => Ok(ConfigItem::B(buffer[1])),
            2 => Ok(ConfigItem::C(f32::from_le_bytes(
                buffer[1..][..4].try_into().unwrap(),
            ))),
            _ => unreachable!(),
        }
    }

    fn key(&self) -> Self::Key {
        match self {
            ConfigItem::A => 0,
            ConfigItem::B(_) => 1,
            ConfigItem::C(_) => 2,
        }
    }

    fn deserialize_key_only(buffer: &[u8]) -> Result<Self::Key, Self::Error>
    where
        Self: Sized,
    {
        Ok(buffer[0])
    }
}
