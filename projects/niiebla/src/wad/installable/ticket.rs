use crate::ticket::{Ticket, TicketError};
use crate::wad::InstallableWad;
use std::io::{Read, Seek, SeekFrom, Take, Write};
use takes::{Ext, Takes};

impl InstallableWad {
    pub fn seek_ticket<S: Seek>(&self, seeker: &mut S) -> Result<(), TicketError> {
        // The header is always aligned to the boundary
        let ticket_offset =
            InstallableWad::HEADER_SIZE + InstallableWad::align(self.certificate_chain_size);

        seeker.seek(SeekFrom::Start(ticket_offset))?;
        Ok(())
    }

    pub fn take_ticket<'a, T: Read + Seek>(
        &self,
        reader: &'a mut T,
    ) -> Result<Takes<&'a mut T>, TicketError> {
        self.seek_ticket(reader)?;

        Ok(reader.takes(self.ticket_size as u64)?)
    }

    pub fn ticket<T: Read + Seek>(&self, reader: &mut T) -> Result<Ticket, TicketError> {
        self.seek_ticket(reader)?;
        Ok(unsafe { Ticket::from_reader(reader)? })
    }

    pub fn write_ticket<W: Write + Seek>(
        &mut self,
        new_ticket: &Ticket,
        writer: &mut W,
    ) -> Result<(), TicketError> {
        self.seek_ticket(writer)?;

        // TODO(IMPROVE): The size of a ticket should change if Ticket goes V1 <-> V2
        new_ticket.dump(writer)?;

        Ok(())
    }
}
