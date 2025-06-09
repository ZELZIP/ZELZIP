use crate::certificate_chain::{CertificateChain, CertificateChainError};
use crate::wad::InstallableWad;
use std::io::{Read, Seek, SeekFrom};
use util::View;

impl InstallableWad {
    pub fn seek_certificate_chain<T: Seek>(
        &self,
        stream: &mut T,
    ) -> Result<(), CertificateChainError> {
        // The header is always aligned to the boundary
        stream.seek(SeekFrom::Start(Self::HEADER_SIZE))?;

        Ok(())
    }

    pub fn take_certificate_chain<T: Read + Seek>(
        &self,
        mut stream: T,
    ) -> Result<View<T>, CertificateChainError> {
        self.seek_certificate_chain(&mut stream)?;

        Ok(View::new(stream, self.certificate_chain_size as usize)?)
    }

    pub fn certificate_chain<T: Read + Seek>(
        &self,
        stream: &mut T,
    ) -> Result<CertificateChain, CertificateChainError> {
        self.seek_certificate_chain(stream)?;

        CertificateChain::new(stream, Self::NUMBER_OF_CERTIFICATES_STORED)
    }

    // TODO(IMPLEMENT): Add write certificate chain.
}
