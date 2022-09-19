use gear_program::api::Api;

use super::{ExtrinsicStates, Executable};
use std::sync::{Arc, Mutex};


pub struct UploadProgramTask{
    pub status: ExtrinsicStates,
}

impl UploadProgramTask{}

impl Executable for UploadProgramTask{
    type Output = Self;

    async fn generate(gear_api: Api, seed: Arc<Mutex<u32>>, seed_gen: Arc<Mutex<SmallRng>>) -> Output {
        let signer = gear_api.try_signer(None).unwrap();

        println!("==============================================");

        let (seed, salt) = if let Some(seed) = params.only_seed {
            *salt.lock() += 1;
            (seed, *salt.lock())
        } else {
            (seed_gen.lock().next_u64(), 0)
        };
        let salt = format!("{:02}", salt);
        println!("Run with seed = {}, salt = {}", seed, salt);

        let code = gen_code_for_seed(seed);
        println!("Gen code size = {}", code.len());

        let params = UploadProgram {
            code: code.clone(),
            salt: hex::decode(salt.as_str()).unwrap(),
            init_payload: hex::decode("00").unwrap(),
            gas_limit: 250_000_000_000,
            value: 0,
        };
        
        signer.submit_program(params)
}
