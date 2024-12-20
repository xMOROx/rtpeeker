use netpix_common::mpegts::descriptors::Descriptors;
use netpix_common::mpegts::header::PIDTable;
use netpix_common::mpegts::psi::pat::ProgramAssociationTable;
use netpix_common::mpegts::psi::pmt::ProgramMapTable;
use std::cmp::Ordering;

pub const LINE_HEIGHT: f32 = 32.0;

pub struct MpegTsInfo {
    pub pat: Option<ProgramAssociationTable>,
    pub pmt: Option<ProgramMapTable>,
}

#[derive(Default)]
pub struct OpenModal {
    pub descriptor: Option<(usize, Descriptors)>,
    pub is_open: bool,
    pub active_descriptor: Option<(usize, Descriptors)>,
}

#[derive(Hash, Eq, PartialEq, Ord, Clone)]
pub struct RowKey {
    pub pid: PIDTable,
    pub alias: String,
}

impl PartialOrd for RowKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.alias.cmp(&other.alias) == Ordering::Equal {
            return self.pid.partial_cmp(&other.pid);
        }
        self.alias.partial_cmp(&other.alias)
    }
}
