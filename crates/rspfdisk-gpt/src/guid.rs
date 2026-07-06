use rspfdisk_core::PartitionType;
use uuid::Uuid;

pub fn partition_type_guid(pt: &PartitionType) -> Uuid {
    let s = match pt {
        PartitionType::Esp => "C12A7328-F81F-11D2-BA4B-00A0C93EC93B",
        PartitionType::Msr => "E3C9E316-0B5C-4DB8-817D-F92DF00215AE",
        PartitionType::MicrosoftBasicData => "EBD0A0A2-B9E5-4433-87C0-68B6B72699C7",
        PartitionType::WindowsRecovery => "DE94BBA4-06D1-4D40-A16A-BFD50179D6AD",
        PartitionType::LinuxFilesystem => "0FC63DAF-8483-4772-8E79-3D69D8477DE4",
        PartitionType::LinuxSwap => "0657FD6D-A4AB-43C4-84E5-0933C84B4F4F",
        PartitionType::AppleApfs => "7C3457EF-0000-11AA-AA11-0030654EC401",
        PartitionType::BiosBoot => "21686148-6449-6E6F-744E-656564454649",
        PartitionType::ProtectiveMbr => "00000000-0000-0000-0000-000000000000",
        PartitionType::Unknown | PartitionType::Custom(_) => "00000000-0000-0000-0000-000000000000",
    };
    Uuid::parse_str(s).expect("valid guid")
}

pub fn guid_from_type_slug(slug: &str) -> Uuid {
    partition_type_guid(&PartitionType::from_slug(slug))
}

pub fn decode_utf16le_name(data: &[u8]) -> String {
    let chars: Vec<u16> = data
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&c| c != 0)
        .collect();
    String::from_utf16_lossy(&chars)
}

pub fn encode_utf16le_name(name: &str, out: &mut [u8]) {
    out.fill(0);
    let encoded: Vec<u16> = name.encode_utf16().collect();
    let max_chars = out.len() / 2;
    for (i, ch) in encoded.iter().take(max_chars).enumerate() {
        out[i * 2..i * 2 + 2].copy_from_slice(&ch.to_le_bytes());
    }
}

pub fn write_guid_le(guid: &Uuid, out: &mut [u8]) {
    out.copy_from_slice(guid.as_bytes());
}
