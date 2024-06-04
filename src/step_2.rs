// 01. Defining Transaction Struct
// 02. Implementation transaction read, write, commit functions

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

struct Transaction<T> {
    data: Arc<Mutex<HashMap<String, T>>>,
    log: HashMap<String, T>,
}

impl<T: Clone> Transaction<T> {
    fn new(data: Arc<Mutex<HashMap<String, T>>>) -> Self {
        Transaction {
            data,
            log: HashMap::new(),
        }
    }
    
    fn read(&mut self, key: &str) -> Option<T> {
        if let Some(value) = self.log.get(key) {
            return Some(value.clone());
        }
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }

    fn write(&mut self, key: &str, value: T) {
        self.log.insert(key.to_string(), value);
    }

    fn commit(self) -> bool {
        let mut data = self.data.lock().unwrap();
        for (key, value) in self.log {
            data.insert(key, value);
        }
        true
    }
}