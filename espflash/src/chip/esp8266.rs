use std::ops::Range;

use super::{bytes_to_mac_addr, ChipType};
use crate::{
    chip::{ReadEFuse, SpiRegisters},
    connection::Connection,
    elf::FirmwareImage,
    error::UnsupportedImageFormatError,
    image_format::{Esp8266Format, ImageFormat, ImageFormatId},
    Chip, Error, PartitionTable,
};

const IROM_MAP_START: u32 = 0x40200000;
const IROM_MAP_END: u32 = 0x40300000;

pub struct Esp8266;

impl ChipType for Esp8266 {
    const CHIP_DETECT_MAGIC_VALUE: u32 = 0xfff0c101;

    const UART_CLKDIV_REG: u32 = 0x60000014;
    const XTAL_CLK_DIVIDER: u32 = 2;

    const SPI_REGISTERS: SpiRegisters = SpiRegisters {
        base: 0x60000200,
        usr_offset: 0x1c,
        usr1_offset: 0x20,
        usr2_offset: 0x24,
        w0_offset: 0x40,
        mosi_length_offset: None,
        miso_length_offset: None,
    };

    const FLASH_RANGES: &'static [Range<u32>] = &[IROM_MAP_START..IROM_MAP_END];

    const DEFAULT_IMAGE_FORMAT: ImageFormatId = ImageFormatId::Bootloader;
    const SUPPORTED_IMAGE_FORMATS: &'static [ImageFormatId] = &[ImageFormatId::Bootloader];

    const SUPPORTED_TARGETS: &'static [&'static str] = &["xtensa-esp8266-none-elf"];

    fn chip_features(&self, _connection: &mut Connection) -> Result<Vec<&str>, Error> {
        Ok(vec!["WiFi"])
    }

    fn get_flash_segments<'a>(
        image: &'a FirmwareImage,
        _bootloader: Option<Vec<u8>>,
        _partition_table: Option<PartitionTable>,
        image_format: ImageFormatId,
        _chip_revision: Option<u32>,
    ) -> Result<Box<dyn ImageFormat<'a> + 'a>, Error> {
        match image_format {
            ImageFormatId::Bootloader => Ok(Box::new(Esp8266Format::new(image)?)),
            _ => Err(UnsupportedImageFormatError::new(image_format, Chip::Esp8266, None).into()),
        }
    }

    fn mac_address(&self, connection: &mut Connection) -> Result<String, Error> {
        let word0 = self.read_efuse(connection, 0)?;
        let word1 = self.read_efuse(connection, 1)?;
        let word3 = self.read_efuse(connection, 3)?;

        // First determine the OUI portion of the MAC address
        let mut bytes = if word3 != 0 {
            vec![
                ((word3 >> 16) & 0xff) as u8,
                ((word3 >> 8) & 0xff) as u8,
                (word3 & 0xff) as u8,
            ]
        } else if ((word1 >> 16) & 0xff) == 0 {
            vec![0x18, 0xfe, 0x34]
        } else {
            vec![0xac, 0xd0, 0x74]
        };

        // Add the remaining NIC portion of the MAC address
        bytes.push(((word1 >> 8) & 0xff) as u8);
        bytes.push((word1 & 0xff) as u8);
        bytes.push(((word0 >> 24) & 0xff) as u8);

        Ok(bytes_to_mac_addr(&bytes))
    }
}

impl ReadEFuse for Esp8266 {
    const EFUSE_REG_BASE: u32 = 0x3ff00050;
}

#[test]
fn test_esp8266_rom() {
    use std::fs::read;

    let input_bytes = read("./tests/data/esp8266").unwrap();
    let expected_bin = read("./tests/data/esp8266.bin").unwrap();

    let image = FirmwareImage::from_data(&input_bytes).unwrap();
    let flash_image = Esp8266Format::new(&image).unwrap();

    let segments = flash_image.flash_segments().collect::<Vec<_>>();

    assert_eq!(1, segments.len());
    let buff = segments[0].data.as_ref();
    assert_eq!(expected_bin.len(), buff.len());
    assert_eq!(expected_bin.as_slice(), buff);
}
