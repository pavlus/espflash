mod esp32bootloader;
mod esp8266;

use crate::elf::RomSegment;

use bytemuck::{Pod, Zeroable};
pub use esp32bootloader::*;
pub use esp8266::*;

const ESP_MAGIC: u8 = 0xE9;
const WP_PIN_DISABLED: u8 = 0xEE;

#[derive(Copy, Clone, Zeroable, Pod, Debug)]
#[repr(C)]
struct EspCommonHeader {
    magic: u8,
    segment_count: u8,
    flash_mode: u8,
    flash_config: u8,
    entry: u32,
}

#[derive(Copy, Clone, Zeroable, Pod, Debug)]
#[repr(C)]
struct SegmentHeader {
    addr: u32,
    length: u32,
}

pub trait ImageFormat<'a>: Sized {
    fn segments(self) -> Box<dyn Iterator<Item = RomSegment<'a>> + 'a>;
}
