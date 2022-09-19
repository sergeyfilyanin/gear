use rand::rngs::SmallRng;

use super::{ExtrinsicStates, Executable};
use std::sync::{Arc,Mutex};


pub struct UploadCodeTask{
    pub status: ExtrinsicStates,
}

impl UploadCodeTask{}

impl Executable for UploadCodeTask{
    type Output = Self;

    fn generate(seed: Arc<Mutex<u32>>, seed_gen: Arc<Mutex<SmallRng>>) -> Output {
        todo!()
    }
}
