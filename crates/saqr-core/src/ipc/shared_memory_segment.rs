use std::fs::OpenOptions;
use std::path::PathBuf;

use memmap2::{MmapMut, MmapOptions};

use crate::core::unified_market_state::UnifiedMarketState;

/// POSIX-style segment — ملف mmap قابل للمشاركة بين عمليات (Linux/Windows).
pub struct PosixSharedMemorySegment {
    pub segment_name: String,
    pub byte_size: usize,
    path: PathBuf,
}

impl PosixSharedMemorySegment {
    pub fn new(segment_name: String, byte_size: usize) -> Self {
        let path = std::env::var("SAQR_SHM_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(format!("data/shm_{segment_name}.bin")));
        Self {
            segment_name,
            byte_size,
            path,
        }
    }

    pub fn layout_valid(&self) -> bool {
        self.byte_size >= std::mem::size_of::<UnifiedMarketState>()
    }

    pub fn open_or_create_map(&self) -> Result<MmapMut, String> {
        if !self.layout_valid() {
            return Err("SHM layout too small".into());
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.path)
            .map_err(|e| e.to_string())?;
        file.set_len(self.byte_size as u64)
            .map_err(|e| e.to_string())?;
        unsafe {
            MmapOptions::new()
                .len(self.byte_size)
                .map_mut(&file)
                .map_err(|e| e.to_string())
        }
    }

    pub fn write_market_state(
        &self,
        state: &UnifiedMarketState,
    ) -> Result<(), String> {
        let mut map = self.open_or_create_map()?;
        let bytes =
            unsafe { std::slice::from_raw_parts(state as *const _ as *const u8, std::mem::size_of::<UnifiedMarketState>()) };
        map[..bytes.len()].copy_from_slice(bytes);
        Ok(())
    }
}
