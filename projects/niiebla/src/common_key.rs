use thiserror::Error;

#[derive(Debug)]
pub enum CommonKeyKind {
    Normal,
    Korean,
    WiiU,
}

#[derive(Error, Debug)]
pub enum CommonKeyKindError {
    #[error("Unknown common key index: {0}")]
    UnknownCommonKeyIndex(u8),
}

impl CommonKeyKind {
    pub const fn from_identifier(identifier: u8) -> Result<CommonKeyKind, CommonKeyKindError> {
        Ok(match identifier {
            0 => CommonKeyKind::Normal,
            1 => CommonKeyKind::Korean,
            2 => CommonKeyKind::WiiU,

            identifier => return Err(CommonKeyKindError::UnknownCommonKeyIndex(identifier)),
        })
    }

    /// Get the bytes of the correct kind of common key.
    pub const fn bytes(&self) -> [u8; 16] {
        match self {
            CommonKeyKind::Normal => [
                0xeb, 0xe4, 0x2a, 0x22, 0x5e, 0x85, 0x93, 0xe4, 0x48, 0xd9, 0xc5, 0x45, 0x73, 0x81,
                0xaa, 0xf7,
            ],
            CommonKeyKind::Korean => [
                0x63, 0xb8, 0x2b, 0xb4, 0xf4, 0x61, 0x4e, 0x2e, 0x13, 0xf2, 0xfe, 0xfb, 0xba, 0x4c,
                0x9b, 0x7e,
            ],
            CommonKeyKind::WiiU => [
                0x30, 0xbf, 0xc7, 0x6e, 0x7c, 0x19, 0xaf, 0xbb, 0x23, 0x16, 0x33, 0x30, 0xce, 0xd7,
                0xc2, 0x8d,
            ],
        }
    }
}
