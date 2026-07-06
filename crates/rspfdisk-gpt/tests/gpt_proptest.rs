//! Property-based tests for GPT parser.
//!
//! Uses proptest to fuzz the GPT header and partition entry parsers
//! with random byte buffers. Ensures no panics and consistent error
//! handling on invalid input.

use rspfdisk_disk::open_read_only;
use rspfdisk_disk::test_helpers::create_test_image;
use rspfdisk_gpt::{parse_gpt, GptHeader};

proptest::proptest! {
    // -----------------------------------------------------------------------
    // GPT header parsing never panics on any random 512-byte buffer
    // -----------------------------------------------------------------------
    #[test]
    fn gpt_header_never_panics_on_random_input(buffer: [u8; 512]) {
        let _result = GptHeader::parse(&buffer);
    }

    // -----------------------------------------------------------------------
    // CRC32 of random data never panics
    // -----------------------------------------------------------------------
    #[test]
    fn gpt_crc32_never_panics(data: Vec<u8>) {
        let _crc = rspfdisk_gpt::crc::gpt_crc32(&data);
    }

    // -----------------------------------------------------------------------
    // A buffer that is too short (< 92 bytes) must be rejected
    // -----------------------------------------------------------------------
    #[test]
    fn gpt_header_rejects_short_buffers(short in 0u32..92u32) {
        let buf = vec![0u8; short as usize];
        let result = GptHeader::parse(&buf);
        assert!(result.is_err(), "expected error for {} byte buffer", short);
    }


}

// ---------------------------------------------------------------------------
// Integration: parse_gpt on random sized images never panics
// ---------------------------------------------------------------------------
#[test]
fn parse_gpt_on_various_sized_images() {
    let dir = std::env::temp_dir();
    let sizes = [4096u64, 16384, 65536, 131072, 524288, 1_048_576];

    for size in sizes {
        let path = dir.join(format!("proptest-gpt-{size}.img"));
        let _ = create_test_image(&path, size);
        if let Ok(device) = open_read_only(&path) {
            let _result = parse_gpt(&device);
        }
        let _ = std::fs::remove_file(&path);
    }
}
