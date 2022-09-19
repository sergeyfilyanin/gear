//! Gear node loader.
//!
//! This tool sends semi-random data to the gear node with one main purpose - crash it.
//! The sent data is not completely random as it is usually in fuzz-kind tests. The tool
//! gets properly structured data acceptable by the gear node and randomizes it's "fields".
//! That's why generated data is called semi-random.

use std::{fs::File, io::Write, sync::Arc, collections::HashMap};

use arbitrary::Unstructured;
use gear_program::{
    api::{generated::api::gear::calls::{UploadProgram, UploadCode}, Api},
    result::Result,
};
use gear_wasm_gen::GearConfig;
use parking_lot::Mutex;
use rand::{rngs::SmallRng, distributions::{Distribution, Standard}, Rng, RngCore, SeedableRng };

mod args;
pub mod task;

use args::{parse_cli_params, Params};
use task::{Executable, upload_code::UploadCodeTask, upload_program::UploadProgramTask, ExtrinsicStates};


#[derive(Debug)]
pub enum Extrinsics{
    UploadCode,
    UploadProgram,
    Others,
}

impl Distribution<Extrinsics> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Extrinsics {
        match rng.gen_range(0..=2) { 
            0 => Extrinsics::UploadCode,
            1 => Extrinsics::UploadProgram,
            _ => Extrinsics::Others
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let params = parse_cli_params();

    if let Some(seed) = params.dump_seed {
        let code = gen_code_for_seed(seed);
        let mut file = File::create("out.wasm").expect("Cannot create out.wasm file");
        file.write_all(&code).expect("Cannot write bytes into file");
        return Ok(());
    }

    load_node(params).await;

    Ok(())
}

async fn load_node(params: Params) {
    gear_program::keystore::login(&params.user, None).unwrap();

    let params = Arc::new(params);
    let gear_api = gear_program::api::Api::new(Some(&params.endpoint))
        .await
        .unwrap();
    let salt = Arc::new(Mutex::new(0));
    let seed_gen = Arc::new(Mutex::new(SmallRng::seed_from_u64(params.seed)));

    let workers_num = params.workers as usize;
    let mut tasks = Vec::with_capacity(workers_num);
    
    let mut states: HashMap<u32 , ExtrinsicStates> = HashMap::new();
    
    loop {
        tasks.clear();
        while tasks.len() != workers_num {
            let task = tokio::spawn(generate(
                gear_api.clone(),
                Arc::clone(&salt),
                Arc::clone(&seed_gen),
            ));
            tasks.push(task);
        }
        for task in &mut tasks {
            states.insert();
            //states.push(task.await.unwrap());
        }
    }
}

pub fn generate(gear_api: Api, seed: Arc<Mutex<u32>>, seed_gen: Arc<Mutex<SmallRng>>) -> impl Executable {
    let ext_rnd: Extrinsics = rand::random();
    let task = match ext_rnd {
        Extrinsics::UploadCode => UploadCodeTask::generate(gear_api, seed, seed_gen),
        Extrinsics::UploadProgram => UploadProgramTask::generate(gear_api, seed, seed_gen),
        Extrinsics::Others => todo!(),
    }
}

async fn load_node_task(
    gear_api: Api,
    salt: Arc<Mutex<u32>>,
    seed_gen: Arc<Mutex<SmallRng>>,
    params: Arc<Params>,
) {
}

fn gen_code_for_seed(seed: u64) -> Vec<u8> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut buf = vec![0; 100_000];
    rng.fill_bytes(&mut buf);
    let mut u = Unstructured::new(&buf);
    gear_wasm_gen::gen_gear_program_code(&mut u, GearConfig::default())
}
