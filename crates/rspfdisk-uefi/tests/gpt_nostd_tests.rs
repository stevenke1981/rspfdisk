use rspfdisk_uefi::parse_gpt_from_disk_sectors;

fn load_fixture(name: &str) -> (Vec<u8>, Vec<u8>) {
    let path = format!("../../tests/fixtures/uefi/{name}");
    let data = std::fs::read(&path).unwrap_or_else(|_| {
        // Synthesize minimal valid GPT for unit test when fixture absent.
        synthesize_minimal_gpt()
    });
    let header = data[..512].to_vec();
    let entries = data[512..].to_vec();
    (header, entries)
}

fn synthesize_minimal_gpt() -> Vec<u8> {
    let mut header = vec![0u8; 512];
    header[0..8].copy_from_slice(b"EFI PART");
    header[8..12].copy_from_slice(&0x0001_0000u32.to_le_bytes());
    header[12..16].copy_from_slice(&92u32.to_le_bytes());
    header[80..88].copy_from_slice(&2u64.to_le_bytes());
    header[88..92].copy_from_slice(&1u32.to_le_bytes());
    header[92..96].copy_from_slice(&128u32.to_le_bytes());

    let mut entry = vec![0u8; 128];
    entry[0] = 0x28; // non-zero type GUID
    entry[32..40].copy_from_slice(&2048u64.to_le_bytes());
    entry[40..48].copy_from_slice(&4096u64.to_le_bytes());
    let name_utf16: Vec<u8> = "EFI".encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    entry[56..56 + name_utf16.len()].copy_from_slice(&name_utf16);

    let entries_crc = rspfdisk_uefi::gpt::gpt_crc32(&entry);
    header[96..100].copy_from_slice(&entries_crc.to_le_bytes());

    let mut crc_buf = header.clone();
    crc_buf[16..20].copy_from_slice(&0u32.to_le_bytes());
    let hdr_crc = rspfdisk_uefi::gpt::gpt_crc32(&crc_buf[..92]);
    header[16..20].copy_from_slice(&hdr_crc.to_le_bytes());

    let mut out = header;
    out.extend(entry);
    out
}

#[test]
fn parse_minimal_gpt() {
    let (header, entries) = load_fixture("gpt-minimal.bin");
    let table = parse_gpt_from_disk_sectors(&header, &entries).expect("parse gpt");
    assert_eq!(table.partitions.len(), 1);
    assert_eq!(table.partitions[0].start_lba, 2048);
}

#[test]
fn reject_bad_signature() {
    let (mut header, entries) = load_fixture("gpt-minimal.bin");
    header[0] = b'X';
    assert!(parse_gpt_from_disk_sectors(&header, &entries).is_err());
}
