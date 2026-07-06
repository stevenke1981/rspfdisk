use rspfdisk_core::SectorSize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorBuf {
    data: Vec<u8>,
    sector_size: SectorSize,
}

impl SectorBuf {
    pub fn new(sector_size: SectorSize, sector_count: usize) -> Self {
        Self {
            data: vec![0u8; sector_size.bytes() as usize * sector_count],
            sector_size,
        }
    }

    pub fn from_bytes(sector_size: SectorSize, data: Vec<u8>) -> Self {
        Self { data, sector_size }
    }

    pub fn sector_size(&self) -> SectorSize {
        self.sector_size
    }

    pub fn sector_count(&self) -> usize {
        self.data.len() / self.sector_size.bytes() as usize
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn sector(&self, index: usize) -> &[u8] {
        let size = self.sector_size.bytes() as usize;
        &self.data[index * size..(index + 1) * size]
    }

    pub fn sector_mut(&mut self, index: usize) -> &mut [u8] {
        let size = self.sector_size.bytes() as usize;
        let start = index * size;
        let end = start + size;
        &mut self.data[start..end]
    }
}
