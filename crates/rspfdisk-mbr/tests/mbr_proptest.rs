//! Property-based tests for MBR parser.
//!
//! Fuzzes the MBR entry parser with random byte buffers to ensure
//! no panics on any input.

use rspfdisk_mbr::parse_mbr_sector;

proptest::proptest! {
    // -----------------------------------------------------------------------
    // MBR sector parsing never panics on any random 512-byte buffer
    // -----------------------------------------------------------------------
    #[test]
    fn mbr_parse_never_panics_on_random_input(buffer: [u8; 512]) {
        let _result = parse_mbr_sector(&buffer);
    }

    // -----------------------------------------------------------------------
    // MBR parsing handles large buffers gracefully
    // -----------------------------------------------------------------------
    #[test]
    fn mbr_parse_never_panics_on_large_buffers(data: Vec<u8>) {
        if data.len() >= 512 {
            let _result = parse_mbr_sector(&data);
        }
        // Skip if too short — handled by next test
    }

    // -----------------------------------------------------------------------
    // A buffer that is too short (< 512 bytes) must be rejected
    // -----------------------------------------------------------------------
    #[test]
    fn mbr_parse_rejects_short_buffers(short in 0u32..512u32) {
        let buf = vec![0u8; short as usize];
        let result = parse_mbr_sector(&buf);
        assert!(result.is_err(), "expected error for {} byte buffer", short);
    }
}
