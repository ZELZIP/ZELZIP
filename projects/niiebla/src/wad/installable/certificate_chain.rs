use crate::certificate_chain::{CertificateChain, CertificateChainError};
use crate::wad::InstallableWad;
use std::io::{Read, Seek, SeekFrom};
use takes::{Ext, Takes};

impl InstallableWad {
    pub fn seek_certificate_chain<S: Seek>(
        &self,
        seeker: &mut S,
    ) -> Result<(), CertificateChainError> {
        // The header is always aligned to the boundary
        seeker.seek(SeekFrom::Start(InstallableWad::HEADER_SIZE))?;

        Ok(())
    }

    pub fn take_certificate_chain<'a, T: Read + Seek>(
        &self,
        reader: &'a mut T,
    ) -> Result<Takes<&'a mut T>, CertificateChainError> {
        self.seek_certificate_chain(reader)?;

        Ok(reader.takes(self.certificate_chain_size as u64)?)
    }

    pub fn certificate_chain<T: Read + Seek>(
        &self,
        reader: &mut T,
    ) -> Result<CertificateChain, CertificateChainError> {
        Ok(unsafe {
            self.seek_certificate_chain(reader)?;

            CertificateChain::from_reader(reader, InstallableWad::NUMBER_OF_CERTIFICATES_STORED)?
        })
    }
}
