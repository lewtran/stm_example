// 01. Defining Transaction Struct
// 02. Implementation read, write, commit functions
// 03. Implementation STM struct
// 04. Basic usage

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crossbeam::thread;

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

struct STM<T> {
    data: Arc<Mutex<HashMap<String, T>>>,
}

impl<T: Clone> STM<T> {
    fn new() -> Self {
        STM {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn start_transaction(&self) -> Transaction<T> {
        Transaction::new(Arc::clone(&self.data))
    }
}

fn main() {
    let stm = STM::new();
    
    thread::scope(|s| {
        for _ in 0..10 {
            let stm = Arc::clone(&stm.data);
            s.spawn(move |_| {
                let mut txn = Transaction::new(stm);
                let value = txn.read("key").unwrap_or(0);
                txn.write("key", value + 1);
                txn.commit();
            });
        }
    }).unwrap();

    let value = STM {
        data: Arc::clone(&stm.data),
    }.start_transaction().read("key").unwrap_or(0);
    println!("Final value: {}", value);
}