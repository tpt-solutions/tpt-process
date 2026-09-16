//! General process data store: a typed, unit-annotated key-value store
//! for process data with CSV import/export.
//!
//! # Example
//!
//! ```
//! use tpt_proc_database::DataStore;
//!
//! let mut store = DataStore::new();
//! store.set_number("feed.flow", 125.0, "kg/h");
//! store.set_text("feed.name", "crude-40");
//!
//! assert_eq!(store.number("feed.flow"), Some((125.0, "kg/h")));
//! assert_eq!(store.text("feed.name"), Some("crude-40"));
//! ```
//!
//! CSV round-trip:
//!
//! ```
//! use tpt_proc_database::DataStore;
//!
//! let csv = "key,value,unit\nfeed.flow,125.5,kg/h\n";
//! let store = DataStore::from_csv(csv).unwrap();
//! assert_eq!(store.number("feed.flow"), Some((125.5, "kg/h")));
//! assert!(store.to_csv().contains("feed.flow,125.5,kg/h"));
//! ```

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// A stored entry: value plus unit annotation.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    /// Numeric value with unit.
    Number(f64, String),
    /// Text value.
    Text(String),
}

/// A process data store with deterministic (ordered) iteration.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataStore {
    entries: BTreeMap<String, Entry>,
}

impl DataStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a numeric value with a unit.
    pub fn set_number(&mut self, key: &str, value: f64, unit: &str) {
        self.entries
            .insert(key.to_string(), Entry::Number(value, unit.to_string()));
    }

    /// Sets a text value.
    pub fn set_text(&mut self, key: &str, value: &str) {
        self.entries
            .insert(key.to_string(), Entry::Text(value.to_string()));
    }

    /// Fetches a numeric value with its unit.
    #[must_use]
    pub fn number(&self, key: &str) -> Option<(f64, &str)> {
        match self.entries.get(key) {
            Some(Entry::Number(v, unit)) => Some((*v, unit.as_str())),
            _ => None,
        }
    }

    /// Fetches a text value.
    #[must_use]
    pub fn text(&self, key: &str) -> Option<&str> {
        match self.entries.get(key) {
            Some(Entry::Text(t)) => Some(t.as_str()),
            _ => None,
        }
    }

    /// Number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// All keys, ordered.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    /// Serializes to CSV (`key,value,unit`); text entries carry an empty
    /// unit column.
    #[must_use]
    pub fn to_csv(&self) -> String {
        let mut out = String::from("key,value,unit\n");
        for (key, entry) in &self.entries {
            match entry {
                Entry::Number(v, unit) => {
                    out.push_str(&format!("{key},{v},{unit}\n"));
                }
                Entry::Text(t) => {
                    out.push_str(&format!("{key},{t},\n"));
                }
            }
        }
        out
    }

    /// Parses CSV in the [`DataStore::to_csv`] format.
    ///
    /// # Errors
    /// Returns `Err` with the offending line number for malformed rows.
    pub fn from_csv(text: &str) -> Result<Self, String> {
        let mut store = Self::new();
        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.trim_end_matches('\r');
            if line_no == 0 && line.starts_with("key,") {
                continue; // header
            }
            if line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(3, ',').collect();
            if parts.len() < 2 {
                return Err(format!("line {}: expected key,value[,unit]", line_no + 1));
            }
            let key = parts[0].trim();
            let value = parts[1].trim();
            let unit = parts.get(2).map(|u| u.trim()).unwrap_or("");
            if key.is_empty() {
                return Err(format!("line {}: empty key", line_no + 1));
            }
            match value.parse::<f64>() {
                Ok(v) => store.set_number(key, v, unit),
                Err(_) => store.set_text(key, value),
            }
        }
        Ok(store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_roundtrip_types() {
        let mut store = DataStore::new();
        store.set_number("p1.pressure", 12.5, "bar");
        store.set_text("note", "checked");
        assert_eq!(store.number("p1.pressure"), Some((12.5, "bar")));
        // A text entry does not answer number queries.
        assert_eq!(store.number("note"), None);
        assert_eq!(store.text("note"), Some("checked"));
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn csv_roundtrip_preserves_order_and_types() {
        let mut store = DataStore::new();
        store.set_number("feed.flow", 125.5, "kg/h");
        store.set_text("unit.name", "100-D");
        store.set_number("temp", 373.15, "K");
        let csv = store.to_csv();
        let back = DataStore::from_csv(&csv).unwrap();
        assert_eq!(back, store);
        // Deterministic key order.
        let keys: Vec<&str> = store.keys().collect();
        assert_eq!(keys, vec!["feed.flow", "temp", "unit.name"]);
    }

    #[test]
    fn csv_reports_bad_lines() {
        assert!(DataStore::from_csv("key,value,unit\n").is_ok());
        assert!(DataStore::from_csv("no commas at all\n").is_err());
    }
}
