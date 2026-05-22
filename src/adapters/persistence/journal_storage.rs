use async_trait::async_trait;
use redb::{Database, TableDefinition, ReadableTable};
use std::path::Path;
use std::sync::Arc;

use crate::strategy_core::domain::errors::StrategyError;
use crate::strategy_core::domain::trade_intent::TradeIntent;
use crate::strategy_core::ports::journal_port::IJournalPort;

const TRADE_INTENTS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("trade_intents");
const CONTROL_EVENTS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("control_events");

pub struct JournalStorage {
    db: Arc<Database>,
}

impl JournalStorage {
    pub fn open(path: &str) -> Result<Self, StrategyError> {
        let db = Database::create(Path::new(path))
            .map_err(|e| StrategyError::Journal(format!("Failed to open journal DB: {}", e)))?;

        let write_txn = db.begin_write()
            .map_err(|e| StrategyError::Journal(format!("Begin write error: {}", e)))?;
        {
            let _ = write_txn.open_table(TRADE_INTENTS)
                .map_err(|e| StrategyError::Journal(format!("Create trade_intents table: {}", e)))?;
            let _ = write_txn.open_table(CONTROL_EVENTS)
                .map_err(|e| StrategyError::Journal(format!("Create control_events table: {}", e)))?;
        }
        write_txn.commit()
            .map_err(|e| StrategyError::Journal(format!("Commit error: {}", e)))?;

        Ok(Self { db: Arc::new(db) })
    }

    pub async fn append_intent(&self, intent: &TradeIntent) -> Result<(), StrategyError> {
        let key = format!("{}:{}:{}", intent.timestamp_ns, intent.symbol, intent.sequence_number);
        let value = serde_json::to_vec(intent)
            .map_err(|e| StrategyError::Journal(format!("Serialize intent: {}", e)))?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StrategyError::Journal(format!("Begin write: {}", e)))?;
        {
            let mut table = write_txn.open_table(TRADE_INTENTS)
                .map_err(|e| StrategyError::Journal(format!("Open table: {}", e)))?;
            table.insert(key.as_bytes(), value.as_slice())
                .map_err(|e| StrategyError::Journal(format!("Insert: {}", e)))?;
        }
        write_txn.commit()
            .map_err(|e| StrategyError::Journal(format!("Commit: {}", e)))?;

        Ok(())
    }

    pub async fn append_control_event(&self, event_type: &str, data: &[u8]) -> Result<(), StrategyError> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let key = format!("{}:{}", now_ms, event_type);

        let write_txn = self.db.begin_write()
            .map_err(|e| StrategyError::Journal(format!("Begin write: {}", e)))?;
        {
            let mut table = write_txn.open_table(CONTROL_EVENTS)
                .map_err(|e| StrategyError::Journal(format!("Open table: {}", e)))?;
            table.insert(key.as_bytes(), data)
                .map_err(|e| StrategyError::Journal(format!("Insert: {}", e)))?;
        }
        write_txn.commit()
            .map_err(|e| StrategyError::Journal(format!("Commit: {}", e)))?;

        Ok(())
    }

    pub fn get_last_control_event(&self, event_type: &str) -> Result<Option<Vec<u8>>, StrategyError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StrategyError::Journal(format!("Begin read: {}", e)))?;
        let table = read_txn.open_table(CONTROL_EVENTS)
            .map_err(|e| StrategyError::Journal(format!("Open table: {}", e)))?;

        let prefix = format!(":{}", event_type);
        let mut result: Option<Vec<u8>> = None;

        for entry in table.iter().map_err(|e| StrategyError::Journal(format!("Iter: {}", e)))? {
            let (key, value) = entry.map_err(|e| StrategyError::Journal(format!("Entry: {}", e)))?;
            let key_str = String::from_utf8_lossy(key.value());
            if key_str.ends_with(&prefix) {
                result = Some(value.value().to_vec());
            }
        }

        Ok(result)
    }
}

#[async_trait]
impl IJournalPort for JournalStorage {
    async fn append_intent(&self, intent: &TradeIntent) -> Result<(), StrategyError> {
        JournalStorage::append_intent(self, intent).await
    }

    async fn append_control_event(&self, event_type: &str, data: &[u8]) -> Result<(), StrategyError> {
        JournalStorage::append_control_event(self, event_type, data).await
    }

    fn get_last_control_event(&self, event_type: &str) -> Result<Option<Vec<u8>>, StrategyError> {
        JournalStorage::get_last_control_event(self, event_type)
    }
}
