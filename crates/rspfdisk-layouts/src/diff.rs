use rspfdisk_core::{DiffReport, LayoutDraft, PartitionTable};

pub fn build_diff_report(current: &PartitionTable, draft: &LayoutDraft) -> DiffReport {
    let creates_gpt =
        current.partitions.is_empty() || draft.table == rspfdisk_core::PartitionTableKind::Gpt;
    let creates_mbr = draft.table == rspfdisk_core::PartitionTableKind::Mbr;

    let mut summary_lines = vec![
        format!("Template: {}", draft.display_name),
        format!("Table: {:?}", draft.table),
    ];

    if current.partitions.is_empty() {
        summary_lines.push("+ Create GPT".to_string());
    }

    for part in &draft.partitions {
        let fs = part.filesystem.as_deref().unwrap_or("none");
        let size_mib = part.size_bytes / (1024 * 1024);
        summary_lines.push(format!(
            "+ Partition: {}  {} MiB  {}  {:?}",
            part.name, size_mib, fs, part.partition_type
        ));
    }

    summary_lines.push("No write performed.".to_string());

    DiffReport {
        creates_gpt,
        creates_mbr,
        added_partitions: draft.partitions.iter().map(|p| p.name.clone()).collect(),
        removed_partitions: Vec::new(),
        modified_partitions: Vec::new(),
        summary_lines,
    }
}
