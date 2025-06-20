use crate::title_id::TitleId;
use byteorder::{BE, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Seek, SeekFrom, Write};
use thiserror::Error;
use util::StreamPin;

use crate::ticket::PreSwitchTicketError;

// WARNING! HAZMAT! ACHTUNG! PELIGRO! THIS FORMAT IS REALLY SHITTY SO THIS IS
// THE CLEANEST WAY TO WRITE THIS AND PRESERVE PROPER TYPING.
//
// As a side note, this extension has barely seen any usage outside some DLC management on
// [Wii no Ma](https://en.wikipedia.org/wiki/Wii_no_Ma), so unless someone requests better support
// don't waste time improving this.
//
// "When I wrote this code, only god and I know how it worked.
// Now, only god knows it" - Kutu 2025-06-19 21:12:52Z

// TODO: Remove "ExtraData" on type names

#[derive(Debug)]
pub struct PreSwitchTicketV1ExtraData {
    sections: Vec<PreSwitchTicketV1ExtraDataSection>,
    flags: u32,
}

impl PreSwitchTicketV1ExtraData {
    const HEADER_SIZE: u16 = 20;
    const SECTION_HEADER_SIZE: u16 = 20;

    pub fn new<T: Read + Seek>(
        mut stream: T,
    ) -> Result<PreSwitchTicketV1ExtraData, PreSwitchTicketV1Error> {
        let mut stream = StreamPin::new(stream)?;

        let version = stream.read_u16::<BE>()?;
        if version != 1 {
            return Err(PreSwitchTicketV1Error::UnknownTicketV1Version(version));
        };

        let header_size = stream.read_u16::<BE>()?;
        if header_size != Self::HEADER_SIZE {
            return Err(PreSwitchTicketV1Error::UnknownTicketV1HeaderSize(
                header_size,
            ));
        }

        // TODO: Verify with this
        let _v1_data_size = stream.read_u32::<BE>()?;

        let first_section_header_offset = stream.read_u32::<BE>()?;
        let number_of_sections = stream.read_u16::<BE>()?;

        let section_header_size = stream.read_u16::<BE>()?;
        if section_header_size != Self::SECTION_HEADER_SIZE {
            return Err(PreSwitchTicketV1Error::UnknownTicketV1SectionHeaderSize(
                section_header_size,
            ));
        }

        let flags = stream.read_u32::<BE>()?;

        let mut sections = Vec::new();

        stream.seek_from_pin(first_section_header_offset.into())?;

        for _ in 0..number_of_sections {
            sections.push(PreSwitchTicketV1ExtraDataSection::new(&mut stream)?);
        }

        Ok(PreSwitchTicketV1ExtraData { sections, flags })
    }

    pub(crate) fn dump<T: Write + Seek>(&self, mut stream: T) -> io::Result<()> {
        let mut stream = StreamPin::new(stream)?;

        // Ticket V1 version
        stream.write_u16::<BE>(1)?;

        stream.write_u16::<BE>(Self::HEADER_SIZE)?;
        stream.write_u32::<BE>(self.size());

        // Skip this for now as we cannot know the position of the first section yet
        let first_section_byte_header_position = stream.relative_position()? as u64;
        stream.seek_relative(4)?;

        stream.write_u16::<BE>(self.sections.len() as u16)?;
        stream.write_u16::<BE>(Self::SECTION_HEADER_SIZE)?;
        stream.write_u32::<BE>(self.flags)?;

        let mut start_of_records = vec![];
        for section in &self.sections {
            start_of_records.push(stream.relative_position()? as u32);
            section.records.dump(&mut stream);
        }

        for (i, section) in self.sections.iter().enumerate() {
            if i == 0 {
                let first_section_byte_position = stream.relative_position()? as u64;

                stream.seek_from_pin(first_section_byte_header_position)?;
                stream.write_u32::<BE>(first_section_byte_position as u32)?;

                stream.seek_from_pin(first_section_byte_position)?;
            }

            stream.write_u32::<BE>(start_of_records[i])?;

            stream.write_u32::<BE>(section.records.len())?;
            stream.write_u32::<BE>(section.records.size_of_one_record())?;
            stream.write_u32::<BE>(Self::SECTION_HEADER_SIZE.into())?;

            stream.write_u16::<BE>(match section.records {
                PreSwitchTicketV1ExtraDataRecords::Permanent(_) => 1,
                PreSwitchTicketV1ExtraDataRecords::Subscription(_) => 2,
                PreSwitchTicketV1ExtraDataRecords::Content(_) => 3,
                PreSwitchTicketV1ExtraDataRecords::ContentConsumption(_) => 4,
                PreSwitchTicketV1ExtraDataRecords::AccessTitle(_) => 5,
            })?;

            stream.write_u16::<BE>(section.flags)?;
        }

        Ok(())
    }

    pub fn size(&self) -> u32 {
        let mut size = Self::HEADER_SIZE as u32
            + (Self::SECTION_HEADER_SIZE as u32 * self.sections.len() as u32);

        for section in &self.sections {
            size += section.records.size();
        }

        size
    }
}

#[derive(Error, Debug)]
pub enum PreSwitchTicketV1Error {
    #[error("Unknown ticket v1 version: {0}")]
    UnknownTicketV1Version(u16),

    #[error("Unknown ticket v1 header size: {0}")]
    UnknownTicketV1HeaderSize(u16),

    #[error("Unknown ticket v1 section header size: {0}")]
    UnknownTicketV1SectionHeaderSize(u16),

    #[error("Unknown ticket v1 section type: {0}")]
    UnknownTicketV1SectionKind(u16),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

#[derive(Debug)]
pub struct PreSwitchTicketV1ExtraDataSection {
    records: PreSwitchTicketV1ExtraDataRecords,
    flags: u16,
}

#[derive(Debug)]
pub enum PreSwitchTicketV1ExtraDataRecords {
    Permanent(Vec<PreSwitchTicketV1ExtraDataRecordPermanent>),
    Subscription(Vec<PreSwitchTicketV1ExtraDataRecordSubscription>),
    Content(Vec<PreSwitchTicketV1ExtraDataRecordContent>),
    ContentConsumption(Vec<PreSwitchTicketV1ExtraDataRecordContentConsumption>),
    AccessTitle(Vec<PreSwitchTicketV1ExtraDataRecordAccessTitle>),
}

impl PreSwitchTicketV1ExtraDataRecords {
    fn size(&self) -> u32 {
        self.size_of_one_record() * self.len()
    }

    fn size_of_one_record(&self) -> u32 {
        match self {
            Self::Permanent(_) => 16 + 4,
            Self::Subscription(_) => 16 + 4 + 4,
            Self::Content(_) => 128 + 4,
            Self::ContentConsumption(_) => 2 + 2 + 4,
            Self::AccessTitle(_) => 8 + 8,
        }
    }

    fn len(&self) -> u32 {
        (match self {
            Self::Permanent(data) => data.len(),
            Self::Subscription(data) => data.len(),
            Self::Content(data) => data.len(),
            Self::ContentConsumption(data) => data.len(),
            Self::AccessTitle(data) => data.len(),
        }) as u32
    }

    pub(crate) fn dump<T: Write>(&self, mut stream: T) -> io::Result<()> {
        match self {
            PreSwitchTicketV1ExtraDataRecords::Permanent(data) => {
                for record in data {
                    record.reference_id.dump(&mut stream)?;
                }
            }

            PreSwitchTicketV1ExtraDataRecords::Subscription(data) => {
                for record in data {
                    stream.write_u32::<BE>(record.expiration_time)?;
                    record.reference_id.dump(&mut stream)?;
                }
            }

            PreSwitchTicketV1ExtraDataRecords::Content(data) => {
                for record in data {
                    stream.write_u32::<BE>(record.offset_content_index)?;
                    stream.write_all(&record.access_mask)?;
                }
            }

            PreSwitchTicketV1ExtraDataRecords::ContentConsumption(data) => {
                for record in data {
                    stream.write_u16::<BE>(record.content_index)?;
                    stream.write_u16::<BE>(record.limit_code)?;
                    stream.write_u32::<BE>(record.limit_value)?;
                }
            }

            PreSwitchTicketV1ExtraDataRecords::AccessTitle(data) => {
                for record in data {
                    record.title_id.dump(&mut stream)?;
                    stream.write_u64::<BE>(record.title_mask)?;
                }
            }
        }

        Ok(())
    }
}

impl PreSwitchTicketV1ExtraDataSection {
    fn new<T: Read + Seek>(
        mut stream: &mut StreamPin<T>,
    ) -> Result<PreSwitchTicketV1ExtraDataSection, PreSwitchTicketV1Error> {
        let section_records_offset = stream.read_u32::<BE>()?;
        let number_of_records = stream.read_u32::<BE>()?;
        // TODO: Verify this?
        let size_of_a_record = stream.read_u32::<BE>()?;
        // TODO: Verify this
        let section_header_size = stream.read_u32::<BE>()?;
        let section_kind = stream.read_u16::<BE>()?;
        let flags = stream.read_u16::<BE>()?;

        let next_section_position = stream.stream_position()?;

        let mut records = match section_kind {
            1 => PreSwitchTicketV1ExtraDataRecords::Permanent(vec![]),
            2 => PreSwitchTicketV1ExtraDataRecords::Subscription(vec![]),
            3 => PreSwitchTicketV1ExtraDataRecords::Content(vec![]),
            4 => PreSwitchTicketV1ExtraDataRecords::ContentConsumption(vec![]),
            5 => PreSwitchTicketV1ExtraDataRecords::AccessTitle(vec![]),

            kind => return Err(PreSwitchTicketV1Error::UnknownTicketV1SectionKind(kind)),
        };

        stream.seek_from_pin(section_records_offset.into())?;

        for _ in 0..number_of_records {
            match records {
                PreSwitchTicketV1ExtraDataRecords::Permanent(ref mut data) => {
                    let reference_id = PreSwitchTicketV1ExtraDataRefereceId::new(&mut *stream)?;

                    data.push(PreSwitchTicketV1ExtraDataRecordPermanent { reference_id });
                }

                PreSwitchTicketV1ExtraDataRecords::Subscription(ref mut data) => {
                    let expiration_time = stream.read_u32::<BE>()?;
                    let reference_id = PreSwitchTicketV1ExtraDataRefereceId::new(&mut *stream)?;

                    data.push(PreSwitchTicketV1ExtraDataRecordSubscription {
                        expiration_time,
                        reference_id,
                    })
                }

                PreSwitchTicketV1ExtraDataRecords::Content(ref mut data) => {
                    let offset_content_index = stream.read_u32::<BE>()?;
                    let access_mask = util::read_exact!(stream, 128)?;

                    data.push(PreSwitchTicketV1ExtraDataRecordContent {
                        offset_content_index,
                        access_mask,
                    })
                }

                PreSwitchTicketV1ExtraDataRecords::ContentConsumption(ref mut data) => {
                    let content_index = stream.read_u16::<BE>()?;
                    let limit_code = stream.read_u16::<BE>()?;
                    let limit_value = stream.read_u32::<BE>()?;

                    data.push(PreSwitchTicketV1ExtraDataRecordContentConsumption {
                        content_index,
                        limit_code,
                        limit_value,
                    })
                }

                PreSwitchTicketV1ExtraDataRecords::AccessTitle(ref mut data) => {
                    let title_id = TitleId::new(stream.read_u64::<BE>()?);
                    let title_mask = stream.read_u64::<BE>()?;

                    data.push(PreSwitchTicketV1ExtraDataRecordAccessTitle {
                        title_id,
                        title_mask,
                    })
                }
            }
        }

        stream.seek(SeekFrom::Start(next_section_position))?;
        Ok(PreSwitchTicketV1ExtraDataSection { records, flags })
    }
}

#[derive(Debug)]
// TODO(DISCOVER)
pub struct PreSwitchTicketV1ExtraDataRefereceId {
    id: [u8; 16],
    attributes: u32,
}

impl PreSwitchTicketV1ExtraDataRefereceId {
    fn new<T: Read>(
        mut stream: T,
    ) -> Result<PreSwitchTicketV1ExtraDataRefereceId, PreSwitchTicketV1Error> {
        let id = util::read_exact!(stream, 16)?;
        let attributes = stream.read_u32::<BE>()?;

        Ok(PreSwitchTicketV1ExtraDataRefereceId { id, attributes })
    }

    fn dump<T: Write>(&self, mut stream: T) -> io::Result<()> {
        stream.write_all(&self.id)?;
        stream.write_u32::<BE>(self.attributes)?;

        Ok(())
    }
}

#[derive(Debug)]
// TODO(DISCOVER)
pub struct PreSwitchTicketV1ExtraDataRecordPermanent {
    reference_id: PreSwitchTicketV1ExtraDataRefereceId,
}

#[derive(Debug)]
// TODO(DISCOVER)
pub struct PreSwitchTicketV1ExtraDataRecordSubscription {
    expiration_time: u32,
    reference_id: PreSwitchTicketV1ExtraDataRefereceId,
}

#[derive(Debug)]
// TODO(DISCOVER)
pub struct PreSwitchTicketV1ExtraDataRecordContent {
    // TODO(DISCOVER)
    offset_content_index: u32,

    // TODO(DISCOVER)
    access_mask: [u8; 128],
}

#[derive(Debug)]
// TODO(DISCOVER)
pub struct PreSwitchTicketV1ExtraDataRecordContentConsumption {
    // TODO(DISCOVER)
    content_index: u16,

    // TODO(DISCOVER)
    limit_code: u16,

    // TODO(DISCOVER)
    limit_value: u32,
}

#[derive(Debug)]
// TODO(DISCOVER)
pub struct PreSwitchTicketV1ExtraDataRecordAccessTitle {
    title_id: TitleId,
    title_mask: u64,
}
