// Inventory management

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

fn update_stock(stm: &STM<i32>, product: &str, quantity: i32) -> bool {
    let mut txn = stm.start_transaction();
    let current_stock = txn.read(product).unwrap_or(0);
    txn.write(product, current_stock + quantity);
    txn.commit()
}

fn main() {
    let stm = STM::new();
    let mut init_txn = stm.start_transaction();
    init_txn.write("product_1", 100);
    init_txn.write("product_2", 50);
    init_txn.commit();

    thread::scope(|s| {
        for _ in 0..10 {
            let stm = STM { data: Arc::clone(&stm.data) };
            s.spawn(move |_| {
                update_stock(&stm, "product_1", -5);
                update_stock(&stm, "product_2", 10);
            });
        }
    }).unwrap();

    let final_product_1_stock = stm.start_transaction().read("product_1").unwrap_or(0);
    let final_product_2_stock = stm.start_transaction().read("product_2").unwrap_or(0);
    println!("Final product_1 stock: {}", final_product_1_stock);
    println!("Final product_2 stock: {}", final_product_2_stock);
}