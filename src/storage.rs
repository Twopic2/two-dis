
/* 
    * Credit
    * https://redis.io/docs/latest/develop/data-types/
*/

use std::{collections::HashMap, time::{Duration, Instant}};

pub enum DataTypes {
    String(String),
    Array(Vec<String>)
}

pub struct StorageDataType {
    pub data_value: DataTypes,
    pub expiry: Option<Duration>

}

impl StorageDataType {
    pub fn new(data_value: DataTypes, expiry: Option<Duration>) -> Self {
        Self { data_value, expiry }
    }
}

#[derive(Default)]
pub struct Storage {
    storage: HashMap<String, DataTypes>,
    expire_storage: HashMap<String, StorageDataType>,
    key_creation_time: HashMap<String, Instant>,
}

impl Storage {
    pub fn set(&mut self, key: String, value: String) -> Result<String, ()> {
        self.storage.insert(key, DataTypes::String(value));
        Ok(String::from("OK"))
    }

    // TODO: Make sure to set up array of keys
    pub fn del(&mut self, key: String) -> usize {
        self.key_creation_time.remove(&key);
        if let Some(_) = self.storage.remove(&key) {
           1 
        } else {
           0    
        }
    }

    pub fn append(&mut self, key: String, value: String) -> usize {
        match self.storage.get_mut(&key) {
            Some(v) => {
                match v {
                    DataTypes::String(s) => {
                        s.push_str(&value);
                        s.len()
                    }
                    _ => 0,
                }
            }
            None => {
                let len = value.len();

                self.storage.insert(
                    key,
                    DataTypes::String(value),
                );

                len
            }
        }
    }

    pub fn get(&mut self, key:  String) -> Option<String> {
        match self.storage.get(&key) {
            Some(DataTypes::String(value)) => {
               Some(value.clone()) 
            }
            None => {
                None
            }
            _ => {
                todo!()
            }
        }
    }
}
