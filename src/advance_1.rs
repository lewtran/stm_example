// Banking account transfers

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

fn transfer(stm: &STM<i32>, from: &str, to: &str, amount: i32) -> bool {
    let mut txn = stm.start_transaction();
    let from_balance = txn.read(from).unwrap_or(0);
    let to_balance = txn.read(to).unwrap_or(0);

    if from_balance >= amount {
        txn.write(from, from_balance - amount);
        txn.write(to, to_balance + amount);
        txn.commit()
    } else {
        false
    }
}

fn main() {
    let stm = STM::new();
    let mut init_txn = stm.start_transaction();
    init_txn.write("Alice", 100);
    init_txn.write("Bob", 50);
    init_txn.commit();

    thread::scope(|s| {
        for _ in 0..10 {
            let stm = STM { data: Arc::clone(&stm.data) };
            s.spawn(move |_| {
                transfer(&stm, "Alice", "Bob", 10);
            });
        }
    }).unwrap();

    let final_alice_balance = stm.start_transaction().read("Alice").unwrap_or(0);
    let final_bob_balance = stm.start_transaction().read("Bob").unwrap_or(0);
    println!("Final Alice balance: {}", final_alice_balance);
    println!("Final Bob balance: {}", final_bob_balance);
}