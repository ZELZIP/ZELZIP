use crate::certificate_chain::{CertificateChain, CertificateChainError};
use crate::wad::InstallableWad;
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
    pub fn write_certificate_chain<T: Write + Seek>(
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
}
