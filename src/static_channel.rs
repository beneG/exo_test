//! global hash_map.

use exonum::crypto::{Hash};


use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::Mutex;

use std::collections::HashMap;

lazy_static! {
    static ref CHANNELS: Mutex<HashMap<Hash, Sender<Hash>>> = Mutex::new(HashMap::new());
}
    
pub fn register_callback(key: Hash) -> Receiver<Hash>  {
    let (tx, rx): (Sender<Hash>, Receiver<Hash>) = mpsc::channel();
    CHANNELS.lock().unwrap().insert(key, tx);
    rx
}

pub fn unregister(key: Hash) {
    println!("unregister called!");

    CHANNELS.lock().unwrap().remove(&key);
}

pub fn notify() {
    println!("notiry called!");
    for (hash, sender) in CHANNELS.lock().unwrap().iter() {
        sender.send(*hash).unwrap();
    }
}