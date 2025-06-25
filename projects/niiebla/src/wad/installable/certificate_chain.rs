// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use crate::certificate_chain::{CertificateChain, CertificateChainError};
use crate::wad::InstallableWad;
use crate::wad::InstallableWadError;
use crate::{PreSwitchTicket, TitleMetadata};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use util::{StreamPin, View};

impl InstallableWad {
    /// Seek the stream of the WAD to the start of the certificate chain.
    pub fn seek_certificate_chain<T: Seek>(
        &self,
        mut stream: T,
    ) -> Result<(), CertificateChainError> {
        // The header is always aligned to the boundary
        stream.seek(SeekFrom::Start(Self::HEADER_SIZE))?;

        Ok(())
    }

    /// Crate a [View] into the certificate chain stored inside the WAD stream.
    pub fn take_certificate_chain<T: Read + Seek>(
        &self,
        mut stream: T,
    ) -> Result<View<T>, CertificateChainError> {
        self.seek_certificate_chain(&mut stream)?;

        Ok(View::new(stream, self.certificate_chain_size as usize)?)
    }

    /// Parse the certificate chain stored inside the WAD stream.
    pub fn certificate_chain<T: Read + Seek>(
        &self,
        mut stream: T,
    ) -> Result<CertificateChain, CertificateChainError> {
        self.seek_certificate_chain(&mut stream)?;

        CertificateChain::new(&mut stream, Self::NUMBER_OF_CERTIFICATES_STORED)
    }

    /// Write a new certificate chain into the stream of a WAD.
    ///
    /// # Safety
    /// Data after the certificate chain (ticket, title metadata and content blobs) may be unaligned or overwritten. Using
    /// [Self::write_certificate_chain_safe] or [Self::write_certificate_chain_safe_file]
    /// may be preferred.
    pub unsafe fn write_certificate_chain_raw<T: Write + Seek>(
        &mut self,
        new_certificate_chain: &CertificateChain,
        stream: T,
    ) -> Result<(), CertificateChainError> {
        let mut stream = StreamPin::new(stream)?;

        self.seek_certificate_chain(&mut stream)?;

        new_certificate_chain.dump(&mut stream)?;
        stream.align_zeroed(64)?;

        self.certificate_chain_size = new_certificate_chain.size();

        stream.rewind()?;
        self.dump(stream)?;

        Ok(())
    }

    /// Like [Self::write_certificate_chain_raw] but will make a in-memory copy off all the trailing data to
    /// realign it.
    pub fn write_certificate_chain_safe<T: Read + Write + Seek>(
        &mut self,
        stream: T,
        new_certificate_chain: &CertificateChain,
        ticket: &PreSwitchTicket,
        title_metadata: &TitleMetadata,
    ) -> Result<(), InstallableWadError> {
        let mut stream = StreamPin::new(stream)?;

        let contents = self.store_contents(&mut stream, title_metadata, 0)?;

        unsafe {
            self.write_certificate_chain_raw(new_certificate_chain, &mut stream)?;
            self.write_ticket_raw(ticket, &mut stream)?;
            self.write_title_metadata_raw(title_metadata, &mut stream)?;
        }

        self.restore_contents(&mut stream, title_metadata, &contents)?;

        Ok(())
    }

    /// Like [Self::write_certificate_chain_safe] but will also trim the size of the file to avoid garbage
    /// data or useless zeroes.
    pub fn write_certificate_chain_safe_file(
        &mut self,
        file: &mut File,
        new_certificate_chain: &CertificateChain,
        ticket: &PreSwitchTicket,
        title_metadata: &TitleMetadata,
    ) -> Result<(), InstallableWadError> {
        self.write_certificate_chain_safe(
            &mut *file,
            new_certificate_chain,
            ticket,
            title_metadata,
        )?;

        let new_file_size = file.stream_position()?;
        file.set_len(new_file_size)?;

        Ok(())
    }
}
