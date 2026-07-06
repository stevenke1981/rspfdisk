use rspfdisk_core::PartitionType;
use uuid::Uuid;

/// Well-known partition type GUID constants (compile-time validated).
mod guids {
    use uuid::{uuid, Uuid};

    pub const ESP: Uuid = uuid!("C12A7328-F81F-11D2-BA4B-00A0C93EC93B");
    pub const MSR: Uuid = uuid!("E3C9E316-0B5C-4DB8-817D-F92DF00215AE");
    pub const MICROSOFT_BASIC_DATA: Uuid = uuid!("EBD0A0A2-B9E5-4433-87C0-68B6B72699C7");
    pub const WINDOWS_RECOVERY: Uuid = uuid!("DE94BBA4-06D1-4D40-A16A-BFD50179D6AD");
    pub const LINUX_FILESYSTEM: Uuid = uuid!("0FC63DAF-8483-4772-8E79-3D69D8477DE4");
    pub const LINUX_SWAP: Uuid = uuid!("0657FD6D-A4AB-43C4-84E5-0933C84B4F4F");
    pub const APPLE_APFS: Uuid = uuid!("7C3457EF-0000-11AA-AA11-0030654EC401");
    pub const BIOS_BOOT: Uuid = uuid!("21686148-6449-6E6F-744E-656564454649");
    pub const NIL: Uuid = uuid!("00000000-0000-0000-0000-000000000000");
}

use guids::*;

pub fn partition_type_guid(pt: &PartitionType) -> Uuid {
    match pt {
        PartitionType::Esp => ESP,
        PartitionType::Msr => MSR,
        PartitionType::MicrosoftBasicData => MICROSOFT_BASIC_DATA,
        PartitionType::WindowsRecovery => WINDOWS_RECOVERY,
        PartitionType::LinuxFilesystem => LINUX_FILESYSTEM,
        PartitionType::LinuxSwap => LINUX_SWAP,
        PartitionType::AppleApfs => APPLE_APFS,
        PartitionType::BiosBoot => BIOS_BOOT,
        PartitionType::ProtectiveMbr | PartitionType::Unknown | PartitionType::Custom(_) => NIL,
    }
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
