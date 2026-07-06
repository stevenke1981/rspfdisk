use alloc::format;

use uefi::boot::{self, SearchType};
use uefi::proto::console::text::Input;
use uefi::proto::console::text::Output;
use uefi::proto::media::block::BlockIO;
use uefi::system::{with_stdin, with_stdout};
use uefi::CString16;
use uefi::{cstr16, Handle, Identify, Status};

use rspfdisk_uefi::parse_gpt_from_disk_sectors;

pub fn run() -> Status {
    uefi::helpers::init().unwrap();

    match run_viewer() {
        Ok(()) => Status::SUCCESS,
        Err(msg) => {
            print_line(msg);
            print_line("Press any key...");
            wait_for_key();
            Status::ABORTED
        }
    }
}

fn run_viewer() -> Result<(), &'static str> {
    print_line("Rust SPFDisk UEFI GPT Viewer (read-only PoC)");
    print_line("");

    let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&BlockIO::GUID))
        .map_err(|_| "no BlockIo handles")?;

    if handles.is_empty() {
        print_line("No block devices found.");
        wait_for_key();
        return Ok(());
    }

    let mut shown = 0usize;
    for (i, handle) in handles.iter().enumerate() {
        match inspect_handle(*handle) {
            Ok(table) => {
                print_line(&format!("--- Disk {} ---", i + 1));
                print_line(&format!("Partitions: {}", table.partitions.len()));
                for part in &table.partitions {
                    print_line(&format!("  {part}"));
                }
                print_line("");
                shown += 1;
            }
            Err(_) => continue,
        }
    }

    if shown == 0 {
        print_line("No GPT partitions found on block devices.");
    }

    print_line("Press any key to exit...");
    wait_for_key();
    Ok(())
}

fn inspect_handle(handle: Handle) -> Result<rspfdisk_uefi::GptTable, &'static str> {
    let block =
        boot::open_protocol_exclusive::<BlockIO>(handle).map_err(|_| "BlockIo open failed")?;

    let media = block.media();
    if media.is_logical_partition() {
        return Err("logical partition");
    }

    let block_size = media.block_size() as usize;
    if block_size < 512 {
        return Err("block size too small");
    }
    let media_id = media.media_id();

    let mut header = alloc::vec![0u8; block_size];
    block
        .read_blocks(media_id, 1, &mut header)
        .map_err(|_| "read header failed")?;

    let entry_lba = u64::from_le_bytes(header[80..88].try_into().unwrap_or([0; 8]));
    let entry_count = u32::from_le_bytes(header[88..92].try_into().unwrap_or([0; 4]));
    let entry_size = u32::from_le_bytes(header[92..96].try_into().unwrap_or([0; 4])) as usize;
    let total_entry_bytes = entry_count as usize * entry_size;
    let sectors = total_entry_bytes.div_ceil(block_size);
    let mut entries = alloc::vec![0u8; sectors * block_size];

    block
        .read_blocks(media_id, entry_lba, &mut entries)
        .map_err(|_| "read entries failed")?;
    entries.truncate(total_entry_bytes);

    parse_gpt_from_disk_sectors(&header, &entries).map_err(|_| "GPT parse failed")
}

fn print_line(text: &str) {
    let _ = with_stdout(|stdout: &mut Output| {
        if let Ok(s) = CString16::try_from(text) {
            let _ = stdout.output_string(&s);
        }
        let _ = stdout.output_string(cstr16!("\r\n"));
    });
}

fn wait_for_key() {
    let _ = with_stdin(|stdin: &mut Input| {
        let _ = stdin.read_key();
    });
}
