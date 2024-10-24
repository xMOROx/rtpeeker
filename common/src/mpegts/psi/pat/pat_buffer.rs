use crate::mpegts::psi::pat::fragmentary_pat::FragmentaryProgramAssociationTable;
use crate::mpegts::psi::pat::ProgramAssociationTable;
use crate::mpegts::psi::psi_buffer::PsiBuffer;

pub struct PatBuffer {
    last_section_number: u8,
    pat_fragments: Vec<FragmentaryProgramAssociationTable>,
}


impl PsiBuffer<ProgramAssociationTable, FragmentaryProgramAssociationTable> for PatBuffer {
    fn new(last_section_number: u8) -> Self {
        PatBuffer {
            last_section_number,
            pat_fragments: Vec::new(),
        }
    }

    fn is_complete(&self) -> bool {
        self.pat_fragments.len() as u8 == self.last_section_number + 1
    }

    fn last_section_number(&self) -> u8 {
        self.last_section_number
    }

    fn add_fragment(&mut self, fragment: FragmentaryProgramAssociationTable) {
        self.pat_fragments.push(fragment);
    }

    fn get_fragments(&self) -> &Vec<FragmentaryProgramAssociationTable> {
        &self.pat_fragments
    }

    fn build(&self) -> Option<ProgramAssociationTable> {
        if !self.is_complete() {
            return None;
        }

        let accumulated_payload = self.pat_fragments.iter().fold(Vec::new(), |mut acc, fragment| {
            acc.extend_from_slice(&fragment.payload);
            acc
        });

        ProgramAssociationTable::build(self.pat_fragments[0].transport_stream_id, &accumulated_payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mpegts::psi::pat::ProgramAssociationItem;
    use crate::mpegts::psi::psi_buffer::FragmentaryPsi;

    #[test]
    fn test_pat_buffer_with_one_fragment() {
        let data: Vec<u8> = vec![
            0x00, 0x00, 0xb0, 0x0d, 0x00, 0x03, 0xdf, 0x00, 0x00, 0x00, 0x23, 0xed, 0xad, 0x5a, 0xe9, 0x7d,
            0xda, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];

        let fragment = FragmentaryProgramAssociationTable::unmarshall(&data, true).unwrap();
        let mut buffer = PatBuffer::new(fragment.header.last_section_number);

        buffer.add_fragment(fragment);

        assert_eq!(buffer.is_complete(), true);
        assert_eq!(buffer.build(), Some(ProgramAssociationTable {
            transport_stream_id: 3,
            programs: vec![
                ProgramAssociationItem {
                    program_number: 0x0023,
                    network_pid: None,
                    program_map_pid: Some(0x0dad),
                },
            ],
            crc_32: 0x5ae97dda,
        }));
    }

    #[test]
    fn test_pat_buffer_with_two_fragments() {
        let data1: Vec<u8> = vec![
            // pointer_field, table_id, section_syntax_indicator, section_length
            0x00, /* header */ 0x00, 0xb0, 0xc4 /* header */, /*section*/ 0x12, 0x95, 0xc7, 0x00, 0x01 /*section*/,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed, 0xad,
            0x00, 0x23, 0xed,
        ];

        let data2: Vec<u8> = vec![
            /* header */ 0x00, 0xb0, 0xc4 /* header */, /*section*/ 0x12, 0x95, 0xc7, 0x01, 0x01 /*section*/, 0xad, 0x00, 0x23, 0xed, 0xad, 0x00,
            0x23, 0xed, 0xad, 0x00, 0x23, 0xed, 0xad, 0x5a, 0xe9, 0x7d, 0xda, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];

        let fragment1 = FragmentaryProgramAssociationTable::unmarshall(&data1, true).unwrap();
        let fragment2 = FragmentaryProgramAssociationTable::unmarshall(&data2, false).unwrap();
        let mut buffer = PatBuffer::new(fragment1.header.last_section_number);

        let frag1 = fragment1.clone();
        let frag2 = fragment2.clone();

        buffer.add_fragment(fragment1);
        buffer.add_fragment(fragment2);

        let pat = buffer.build().unwrap();

        let mut programs = Vec::new();

        for _ in 0..48 {
            programs.push(ProgramAssociationItem {
                program_number: 35,
                network_pid: None,
                program_map_pid: Some(3501),
            });
        }

        assert_eq!(buffer.is_complete(), true);
        assert_eq!(buffer.last_section_number, 1);
        assert_eq!(pat.transport_stream_id, 4757);
        assert_eq!(frag1.header.section_length, 196);
        assert_eq!(frag2.header.section_length, 196);
        assert_eq!(frag1.header.section_number, 0);
        assert_eq!(frag1.header.last_section_number, 1);
        assert_eq!(frag2.header.section_number, 1);
        assert_eq!(frag2.header.last_section_number, 1);
        assert_eq!(pat.crc_32, 0x5ae97dda);
        assert_eq!(pat.programs.len(), programs.len());
        assert_eq!(pat.programs, programs);
    }
}
