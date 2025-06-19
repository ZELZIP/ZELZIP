use std::io::{self, Read, Write};

use crate::ticket::PreSwitchTicketError;

#[derive(Debug)]
pub struct PreSwitchTitleV1ExtraData {}

impl PreSwitchTitleV1ExtraData {
    pub fn new<T: Read>(stream: T) -> Result<PreSwitchTitleV1ExtraData, PreSwitchTicketError> {
        Ok(PreSwitchTitleV1ExtraData {})
    }

    pub fn dump<T: Write>(&self, stream: T) -> io::Result<()> {
        Ok(())
    }

    pub fn size(&self) -> u32 {
        0
    }
}
