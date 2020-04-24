extern crate core;

pub mod ttf;
pub mod woff;
// pub mod otf;
// pub mod font2;

pub mod check {
    //
    const SFNT_VERSION_TRUE_TYPE1: u32 = 0x00010000;
    // true
    const SFNT_VERSION_TRUE_TYPE2: u32 = 0x74727565;
    // typ1
    const SFNT_VERSION_TRUE_TYPE3: u32 = 0x74797031;
    // OTTO
    const SFNT_VERSION_OPEN_TYPE: u32 = 0x4F54544F;
    // wOFF
    const SFNT_VERSION_WOFF: u32 = 0x774f4646;

    fn get_data(data: &[u8], offset: usize) -> Option<u32> {
        let r = data.get(offset..offset + 4)?;
        Some(u32::from_be_bytes([r[0], r[1], r[2], r[3]]))
    }

    pub fn check_type(data: &[u8]) -> Option<(String, bool)> {
        let signature = get_data(data, 0)?;
        if signature == SFNT_VERSION_TRUE_TYPE1 || signature == SFNT_VERSION_TRUE_TYPE2 || signature == SFNT_VERSION_TRUE_TYPE3 {
            Some(("ttf".to_string(), false))
        } else if signature == SFNT_VERSION_OPEN_TYPE {
            Some(("otf".to_string(), false))
        } else if signature == SFNT_VERSION_WOFF {
            let tag = get_data(data, 4)?;
            if tag == SFNT_VERSION_TRUE_TYPE1 {
                Some(("ttf".to_string(), true))
            } else if tag == SFNT_VERSION_OPEN_TYPE {
                Some(("otf".to_string(), true))
            } else {
                None
            }
        } else {
            None
        }
    }
}
