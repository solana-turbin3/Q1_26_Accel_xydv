use std::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use wincode::{SchemaRead, SchemaWrite, config::DefaultConfig};

trait Serializer<T> {
    fn to_bytes(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>;
}

struct Borsh;
struct Wincode;
struct Json;

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

struct Storage<T, S> {
    bytes: Option<Vec<u8>>, // option, since it can be empty
    serializer: S,
    phantom: PhantomData<T>,
}

trait StorageMethods<T, S> {
    fn new(&self, serializer: S) -> Self;
    fn save(&mut self, data: &T) -> Result<(), Box<dyn std::error::Error>>;
    fn load(&self) -> Result<T, Box<dyn std::error::Error>>;
    fn has_data(&self) -> bool;
}

impl<T, S> StorageMethods<T, S> for Storage<T, S>
where
    S: Serializer<T>,
{
    fn new(&self, serializer: S) -> Self {
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
