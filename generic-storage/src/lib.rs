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
