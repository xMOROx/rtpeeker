pub(super) const PACKET_START_CODE_PREFIX: u32 = 0x000001;
pub(super) const REQUIRED_FIELDS_SIZE: usize = 6; // in bytes
pub(super) const HEADER_REQUIRED_FIELDS_SIZE: usize = 3; // in bytes
pub(super) const HEADER_MANDATORY_BITS_MASK: u8 = 0xc0;
pub(super) const HEADER_MANDATORY_BITS_VALUE: u8 = 0b10;
pub(super) const SCRAMBLING_CONTROL_MASK: u8 = 0x30;
pub(super) const PTS_DTS_FLAGS_MASK: u8 = 0xC0;
pub(super) const PTS_DTS_REQUIRED_BITS_MASK: u8 = 0xF0;
pub(super) const ONLY_PTS_REQUIRED_BITS_VALUE: u8 = 0b0010;
pub(super) const PTS_AND_DTS_REQUIRED_BITS_FIRST_VALUE: u8 = 0b0011;
pub(super) const PTS_AND_DTS_REQUIRED_BITS_SECOND_VALUE: u8 = 0b0001;
pub(super) const STUFFING_BYTE: u8 = 0xFF;
pub(super) const MAXIMUM_NO_OF_STUFFING_BYTES: usize = 32;
