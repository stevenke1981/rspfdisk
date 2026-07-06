use rspfdisk_core::{CoreError, CoreResult, SectorSize, ALIGN_1MIB};

#[derive(Debug, Clone)]
pub enum SizeExpr {
    Fixed(u64),
    Fill,
    FillMinus(u64),
    AutoSwap,
}

pub fn parse_size_expr(s: &str) -> CoreResult<SizeExpr> {
    let s = s.trim();
    if s == "fill" {
        return Ok(SizeExpr::Fill);
    }
    if let Some(rest) = s.strip_prefix("fill-minus:") {
        return Ok(SizeExpr::FillMinus(parse_byte_size(rest)?));
    }
    if s == "auto:swap" {
        return Ok(SizeExpr::AutoSwap);
    }
    Ok(SizeExpr::Fixed(parse_byte_size(s)?))
}

pub fn parse_byte_size(s: &str) -> CoreResult<u64> {
    let s = s.trim();
    let units: &[(&str, u64)] = &[
        ("GiB", 1024 * 1024 * 1024),
        ("MiB", 1024 * 1024),
        ("KiB", 1024),
        ("G", 1000 * 1000 * 1000),
        ("M", 1000 * 1000),
        ("K", 1000),
    ];
    for (unit, mult) in units {
        if let Some(num) = s.strip_suffix(unit) {
            let value: f64 = num
                .trim()
                .parse()
                .map_err(|_| CoreError::InvalidSizeExpression(s.to_string()))?;
            return Ok((value * *mult as f64) as u64);
        }
    }
    s.parse::<u64>()
        .map_err(|_| CoreError::InvalidSizeExpression(s.to_string()))
}

pub fn auto_swap_size_bytes() -> u64 {
    // Default 8 GiB when RAM unknown (per spec).
    8 * 1024 * 1024 * 1024
}

pub fn align_size(bytes: u64, _sector_size: SectorSize) -> u64 {
    let align = ALIGN_1MIB;
    bytes.div_ceil(align) * align
}

pub fn bytes_to_sectors(bytes: u64, sector_size: SectorSize) -> u64 {
    align_size(bytes, sector_size) / sector_size.bytes() as u64
}
