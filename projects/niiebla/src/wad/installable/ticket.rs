use crate::ticket::{PreSwitchTicket, PreSwitchTicketError};
use crate::wad::InstallableWad;
use std::io::{Read, Seek, SeekFrom, Write};
use util::View;

impl InstallableWad {
    pub fn seek_ticket<T: Seek>(&self, stream: &mut T) -> Result<(), PreSwitchTicketError> {
        // The header is always aligned to the boundary
        let ticket_offset = Self::HEADER_SIZE + Self::align(self.certificate_chain_size);

        stream.seek(SeekFrom::Start(ticket_offset))?;
        Ok(())
    }

    pub fn ticket_view<T: Read + Seek>(
        &self,
        mut stream: T,
    ) -> Result<View<T>, PreSwitchTicketError> {
        self.seek_ticket(&mut stream)?;

        Ok(View::new(stream, self.ticket_size as usize)?)
    }

    pub fn ticket<T: Read + Seek>(
        &self,
        stream: &mut T,
    ) -> Result<PreSwitchTicket, PreSwitchTicketError> {
        self.seek_ticket(stream)?;

        PreSwitchTicket::new(stream)
    }

    pub fn write_ticket<T: Write + Seek>(
        &mut self,
        new_ticket: &PreSwitchTicket,
        stream: &mut T,
    ) -> Result<(), PreSwitchTicketError> {
        self.seek_ticket(stream)?;

        // TODO(IMPROVE): The size of a ticket should change if Ticket goes V0 <-> V1
        new_ticket.dump(stream)?;

        Ok(())
    }
}
