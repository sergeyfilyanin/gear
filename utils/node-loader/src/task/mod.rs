pub mod upload_code;
pub mod upload_program;

use std::sync::{Arc, Mutex};
use gear_program::api::Api;
use rand::rngs::SmallRng;
use async_trait::async_trait;

#[derive(Debug)]
pub enum ExtrinsicStates {
    CodeUploaded,
    ProgramUploaded,
    MsgSended,
    RplSended,
}

#[async_trait]
pub trait Executable{
    type Output;
    async fn generate(gear_api: Api, seed: Arc<Mutex<u32>>, seed_gen: Arc<Mutex<SmallRng>>) -> Self::Output;
}

