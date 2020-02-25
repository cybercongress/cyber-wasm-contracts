use named_type::NamedType;
use named_type_derive::NamedType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm::traits::{ReadonlyStorage, Storage};
use cosmwasm::types::CanonicalAddr;
use cw_storage::{
    singleton, singleton_read, ReadonlySingleton, Singleton,
    bucket, bucket_read, Bucket, ReadonlyBucket,
};

pub static EVANGELIST_RESOLVER_KEY: &[u8] = b"evangelistresolver";
pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, NamedType)]
pub struct State {
    pub owner: CanonicalAddr,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, NamedType)]
pub struct EvangelistRecord {
    pub cyber: CanonicalAddr,
    pub nickname: String,
    pub keybase: String,
    pub github: String,
    pub accepted: bool,
}

pub fn resolver<S: Storage>(storage: &mut S) -> Bucket<S, EvangelistRecord> {
    bucket(EVANGELIST_RESOLVER_KEY, storage)
}

pub fn resolver_read<S: ReadonlyStorage>(storage: &S) -> ReadonlyBucket<S, EvangelistRecord> {
    bucket_read(EVANGELIST_RESOLVER_KEY, storage)
}
