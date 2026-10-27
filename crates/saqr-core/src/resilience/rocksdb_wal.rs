use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalOrderRecord {
    pub client_order_id: String,
    pub payload_json: String,
    pub epoch: u64,
    pub pre_send: bool,
}

pub struct RocksDbWalJournal {
    pending: Vec<WalOrderRecord>,
    file_path: PathBuf,
    #[cfg(feature = "rocksdb-wal")]
    db: Option<Arc<rocksdb::DB>>,
}

impl RocksDbWalJournal {
    pub fn new() -> Self {
        Self::open_default()
    }

    pub fn open_default() -> Self {
        let file_path = std::env::var("SAQR_WAL_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/saqr.wal.jsonl"));
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut journal = Self {
            pending: Vec::with_capacity(256),
            file_path: file_path.clone(),
            #[cfg(feature = "rocksdb-wal")]
            db: None,
        };
        journal.load_file_lines();
        #[cfg(feature = "rocksdb-wal")]
        {
            let rocks_path = std::env::var("SAQR_ROCKSDB_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("data/saqr_rocksdb"));
            if let Ok(db) = rocksdb::DB::open_default(&rocks_path) {
                journal.db = Some(Arc::new(db));
                journal.replay_rocksdb();
            }
        }
        journal
    }

    fn load_file_lines(&mut self) {
        let Ok(data) = fs::read_to_string(&self.file_path) else {
            return;
        };
        for line in data.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(rec) = serde_json::from_str::<WalOrderRecord>(line) {
                self.pending.push(rec);
            }
        }
    }

    #[cfg(feature = "rocksdb-wal")]
    fn replay_rocksdb(&mut self) {
        let Some(db) = &self.db else {
            return;
        };
        let iter = db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            if let Ok((_k, v)) = item {
                if let Ok(rec) = serde_json::from_slice::<WalOrderRecord>(&v) {
                    self.pending.push(rec);
                }
            }
        }
    }

    pub fn append_pre_send(&mut self, record: WalOrderRecord) {
        self.persist(&record);
        self.pending.push(record);
    }

    fn persist(&self, record: &WalOrderRecord) {
        if let Ok(line) = serde_json::to_string(record) {
            if let Ok(mut f) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.file_path)
            {
                let _ = writeln!(f, "{line}");
            }
        }
        #[cfg(feature = "rocksdb-wal")]
        if let Some(db) = &self.db {
            let key = format!("{}:{}", record.epoch, record.client_order_id);
            if let Ok(bytes) = serde_json::to_vec(record) {
                let _ = db.put(key.as_bytes(), bytes);
            }
        }
    }

    pub fn drain_for_recovery(&mut self) -> Vec<WalOrderRecord> {
        std::mem::take(&mut self.pending)
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

pub type SharedWalJournal = Arc<Mutex<RocksDbWalJournal>>;

pub fn shared_wal() -> SharedWalJournal {
    Arc::new(Mutex::new(RocksDbWalJournal::open_default()))
}
