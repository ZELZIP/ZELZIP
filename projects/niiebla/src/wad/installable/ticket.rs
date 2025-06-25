// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use crate::TitleMetadata;
use crate::ticket::{PreSwitchTicket, PreSwitchTicketError};
use crate::wad::InstallableWad;
use crate::wad::InstallableWadError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use util::StreamPin;
use util::View;

impl InstallableWad {
    /// Seek the stream of the WAD to the start of the ticket.
    pub fn seek_ticket<T: Seek>(&self, mut stream: T) -> Result<(), PreSwitchTicketError> {
        // The header is always aligned to the boundary
        let ticket_offset = Self::HEADER_SIZE + Self::align_u64(self.certificate_chain_size);

        stream.seek(SeekFrom::Start(ticket_offset))?;
        Ok(())
    }

    /// Create a [View] into the ticket stored inside the WAD stream.
    pub fn ticket_view<T: Read + Seek>(
        &self,
        mut stream: T,
    ) -> Result<View<T>, PreSwitchTicketError> {
        self.seek_ticket(&mut stream)?;

        Ok(View::new(stream, self.ticket_size as usize)?)
    }

    /// Parse the ticket stored inside the WAD stream.
    pub fn ticket<T: Read + Seek>(
        &self,
        mut stream: T,
    ) -> Result<PreSwitchTicket, PreSwitchTicketError> {
        self.seek_ticket(&mut stream)?;

        PreSwitchTicket::new(stream)
    }

    /// Write a new ticket into the stream of a WAD. The internal WAD data will be modified to
    /// match the new size of the ticket.
    ///
    /// # Safety
    /// Data after the ticket (title metadata and content blobs) may be unaligned or overwritten. Using
    /// [Self::write_ticket_safe] or [Self::write_ticket_safe_file]
    /// may be preferred.
    pub unsafe fn write_ticket_raw<T: Write + Seek>(
        &mut self,
        new_ticket: &PreSwitchTicket,
        stream: T,
    ) -> Result<(), PreSwitchTicketError> {
        let mut stream = StreamPin::new(stream)?;

        self.seek_ticket(&mut stream)?;

        new_ticket.dump(&mut stream)?;
        stream.align_zeroed(64)?;

        self.ticket_size = new_ticket.size();

        stream.rewind()?;
        self.dump(stream)?;

        Ok(())
    }

    /// Like [Self::write_ticket_raw] but will make a in-memory copy off all the trailing data to
    /// realign it.
    pub fn write_ticket_safe<T: Read + Write + Seek>(
        &mut self,
        stream: T,
        new_ticket: &PreSwitchTicket,
        title_metadata: &TitleMetadata,
    ) -> Result<(), InstallableWadError> {
        let mut stream = StreamPin::new(stream)?;

        let contents = self.store_contents(&mut stream, title_metadata, 0)?;

        unsafe {
            self.write_ticket_raw(new_ticket, &mut stream)?;
            self.write_title_metadata_raw(title_metadata, &mut stream)?;
        }

        self.restore_contents(&mut stream, title_metadata, &contents)?;

        Ok(())
    }

    /// Like [Self::write_ticket_safe] but will also trim the size of the file to avoid garbage
    /// data or useless zeroes.
    pub fn write_ticket_safe_file(
        &mut self,
        file: &mut File,
        new_ticket: &PreSwitchTicket,
        title_metadata: &TitleMetadata,
    ) -> Result<(), InstallableWadError> {
        self.write_ticket_safe(&mut *file, new_ticket, title_metadata)?;

        let new_file_size = file.stream_position()?;
        file.set_len(new_file_size)?;

        Ok(())
    }
}
