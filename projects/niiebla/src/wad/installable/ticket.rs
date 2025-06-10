use crate::ticket::{PreSwitchTicket, PreSwitchTicketError};
use crate::wad::InstallableWad;
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

    /// Write a new ticket into the stream of a WAD.
    pub fn write_ticket<T: Write + Seek>(
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
}
