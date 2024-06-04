// 01. Defining Transaction Struct
// 02. Implementation read, write, commit functions
// 03. Implementation STM struct
// 04. Basic usage

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crossbeam::thread;

// Struct representing a transaction
struct Transaction<T> {
    data: Arc<Mutex<HashMap<String, T>>>,
    log: HashMap<String, T>,
}

impl<T: Clone> Transaction<T> {
    // Creates a new transaction with a reference to shared data and an empty log
    fn new(data: Arc<Mutex<HashMap<String, T>>>) -> Self {
        Transaction {
            data,
            log: HashMap::new(),
        }
    }
    
    // Reads a value from the transaction log or shared data
    fn read(&mut self, key: &str) -> Option<T> {
        if let Some(value) = self.log.get(key) {
            return Some(value.clone());
        }
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }

    // Writes a value to the transaction log
    fn write(&mut self, key: &str, value: T) {
        self.log.insert(key.to_string(), value);
    }

    // Commits the transaction by writing all logged changes to the shared data
    fn commit(self) -> bool {
        let mut data = self.data.lock().unwrap();
        for (key, value) in self.log {
            data.insert(key, value);
        }
        true
    }
}

// Struct representing the STM system
struct STM<T> {
    data: Arc<Mutex<HashMap<String, T>>>,
}

impl<T: Clone> STM<T> {
    // Creates a new STM system with an empty data store
    fn new() -> Self {
        STM {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Starts a new transaction
    fn start_transaction(&self) -> Transaction<T> {
        Transaction::new(Arc::clone(&self.data))
    }
}

fn main() {
    let stm = STM::new();
    let counter = 10;

    // Create a scope for threads
    thread::scope(|s| {
        for _ in 0..counter {
            // Clone the shared data for each thread
            let stm = Arc::clone(&stm.data);
            s.spawn(move |_| {
                // Start a new transaction
                let mut txn = Transaction::new(stm);
                // Read the current value of "key"
                let value = txn.read("key").unwrap_or(0);
                // Increment the value and write it back
                txn.write("key", value + 1);
                // Commit the transaction
                txn.commit();
            });
        }
    }).unwrap();

    // Read the "key" final value after all transactions
    let value = STM {
        data: Arc::clone(&stm.data),
    }.start_transaction().read("key").unwrap_or(0);
    
    // Print the final value
    println!("Final value: {}", value);
}