use std::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use wincode::{SchemaRead, SchemaWrite, config::DefaultConfig};

pub trait Serializer<T> {
    fn to_bytes(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>;
}

pub struct Borsh;
pub struct Wincode;
pub struct Json;

impl<T> Serializer<T> for Borsh
where
    T: BorshSerialize + BorshDeserialize,
{
    fn to_bytes(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        borsh::to_vec(data).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        borsh::from_slice(bytes).map_err(|e| e.into())
    }
}

impl<T> Serializer<T> for Wincode
where
    T: for<'a> SchemaRead<'a, DefaultConfig, Dst = T> + SchemaWrite<DefaultConfig, Src = T>,
{
    fn to_bytes(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        wincode::serialize(data).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        wincode::deserialize(bytes).map_err(|e| e.into())
    }
}

impl<T> Serializer<T> for Json
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    fn to_bytes(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        serde_json::to_vec(data).map_err(|e| e.into())
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        serde_json::from_slice(bytes).map_err(|e| e.into())
    }
}

pub struct Storage<T, S> {
    pub bytes: Option<Vec<u8>>, // option, since it can be empty
    pub serializer: S,
    pub phantom: PhantomData<T>,
}

pub trait StorageMethods<T, S> {
    fn new(serializer: S) -> Self;
    fn save(&mut self, data: &T) -> Result<(), Box<dyn std::error::Error>>;
    fn load(&self) -> Result<T, Box<dyn std::error::Error>>;
    fn has_data(&self) -> bool;
}

impl<T, S> StorageMethods<T, S> for Storage<T, S>
where
    S: Serializer<T>,
{
    fn new(serializer: S) -> Self {
        Self {
            bytes: None,
            serializer,
            phantom: PhantomData,
        }
    }

    fn save(&mut self, data: &T) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = self.serializer.to_bytes(data)?;
        self.bytes = Some(bytes);
        Ok(())
    }

    fn load(&self) -> Result<T, Box<dyn std::error::Error>> {
        if let Some(bytes) = &self.bytes {
            return Ok(self.serializer.from_bytes(bytes)?);
        }

        Err("error".into())
    }

    fn has_data(&self) -> bool {
        self.bytes.is_some()
    }
}

#[derive(
    Debug,
    PartialEq,
    BorshSerialize,
    BorshDeserialize,
    SchemaRead,
    SchemaWrite,
    Serialize,
    Deserialize,
)]
pub struct Person {
    pub name: String,
    pub age: u8,
    pub gender: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borsh() {
        let person = Person {
            name: "aditya".to_string(),
            age: 21,
            gender: true,
        };

        let mut storage = Storage::new(Borsh);
        storage.save(&person).unwrap();

        assert!(storage.has_data());
        assert_eq!(storage.load().unwrap(), person);
    }

    #[test]
    fn test_wincode() {
        let person = Person {
            name: "aditya".to_string(),
            age: 21,
            gender: true,
        };

        let mut storage = Storage::new(Wincode);
        storage.save(&person).unwrap();

        assert!(storage.has_data());
        assert_eq!(storage.load().unwrap(), person);
    }

    #[test]
    fn test_json() {
        let person = Person {
            name: "aditya".to_string(),
            age: 21,
            gender: true,
        };

        let mut storage = Storage::new(Json);
        storage.save(&person).unwrap();

        assert!(storage.has_data());
        assert_eq!(storage.load().unwrap(), person);
    }
}
