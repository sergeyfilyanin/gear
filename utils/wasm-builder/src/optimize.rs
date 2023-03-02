use crate::builder_error::BuilderError;
use anyhow::{Context, Result};
use gear_wasm_instrument::STACK_END_EXPORT_NAME;
use pwasm_utils::{
    parity_wasm,
    parity_wasm::elements::{Internal, Module, Section, Serialize},
};
use std::{
    ffi::OsStr,
    fs::metadata,
    path::{Path, PathBuf},
};
use wasm_opt::{OptimizationOptions, Pass};

const OPTIMIZED_EXPORTS: [&str; 7] = [
    "handle",
    "handle_reply",
    "handle_signal",
    "init",
    "state",
    "metahash",
    STACK_END_EXPORT_NAME,
];

/// Type of the output wasm.
#[derive(PartialEq, Eq)]
pub enum OptType {
    Meta,
    Opt,
}

#[derive(Debug, thiserror::Error)]
#[error("Optimizer failed: {0:?}")]
pub struct OptimizerError(pwasm_utils::OptimizerError);

pub struct Optimizer {
    module: Module,
    file: PathBuf,
}

impl Optimizer {
    pub fn new(file: PathBuf) -> Result<Self> {
        let module = parity_wasm::deserialize_file(&file)?;
        Ok(Self { module, file })
    }

    pub fn insert_stack_and_export(&mut self) {
        let _ = crate::insert_stack_end_export(&mut self.module).map_err(|s| log::debug!("{}", s));
    }

    /// Strips all custom sections.
    ///
    /// Presently all custom sections are not required so they can be stripped safely.
    /// The name section is already stripped by `wasm-opt`.
    pub fn strip_custom_sections(&mut self) {
        self.module
            .sections_mut()
            .retain(|section| !matches!(section, Section::Reloc(_) | Section::Custom(_)))
    }

    /// Process optimization.
    pub fn optimize(&self, ty: OptType) -> Result<Vec<u8>> {
        let mut module = self.module.clone();

        let exports = if ty == OptType::Opt {
            OPTIMIZED_EXPORTS.to_vec()
        } else {
            self.module
                .export_section()
                .ok_or_else(|| anyhow::anyhow!("Export section not found"))?
                .entries()
                .iter()
                .flat_map(|entry| {
                    if let Internal::Function(_) = entry.internal() {
                        let entry = entry.field();
                        (!OPTIMIZED_EXPORTS.contains(&entry)).then_some(entry)
                    } else {
                        None
                    }
                })
                .collect()
        };

        pwasm_utils::optimize(&mut module, exports)
            .map_err(OptimizerError)
            .with_context(|| {
                format!(
                    "unable to optimize the WASM file `{0}`",
                    self.file.display()
                )
            })?;

        // Post check exports if optimizing program binary.
        if ty == OptType::Opt {
            check_exports(&module, &self.file)?;
        }

        let mut code = vec![];
        module.serialize(&mut code)?;

        Ok(code)
    }
}

pub struct OptimizationResult {
    pub dest_wasm: PathBuf,
    pub original_size: f64,
    pub optimized_size: f64,
}

/// Attempts to perform optional Wasm optimization using `binaryen`.
///
/// The intention is to reduce the size of bloated Wasm binaries as a result of missing
/// optimizations (or bugs?) between Rust and Wasm.
pub fn optimize_wasm(
    source: PathBuf,
    optimization_passes: &str,
    keep_debug_symbols: bool,
) -> Result<OptimizationResult> {
    let mut dest_optimized = source.clone();

    dest_optimized.set_file_name(format!(
        "{}-opt.wasm",
        source
            .file_name()
            .unwrap_or_else(|| OsStr::new("program"))
            .to_str()
            .unwrap()
    ));

    do_optimization(
        source.as_os_str(),
        dest_optimized.as_os_str(),
        optimization_passes,
        keep_debug_symbols,
    )?;

    if !dest_optimized.exists() {
        return Err(anyhow::anyhow!(
            "Optimization failed, optimized wasm output file `{}` not found.",
            dest_optimized.display()
        ));
    }

    let original_size = metadata(&source)?.len() as f64 / 1000.0;
    let optimized_size = metadata(&dest_optimized)?.len() as f64 / 1000.0;

    // overwrite existing destination wasm file with the optimized version
    std::fs::rename(&dest_optimized, &source)?;
    Ok(OptimizationResult {
        dest_wasm: source,
        original_size,
        optimized_size,
    })
}

/// Optimizes the Wasm supplied as `crate_metadata.dest_wasm` using
/// the `wasm-opt` binary.
///
/// The supplied `optimization_level` denotes the number of optimization passes,
/// resulting in potentially a lot of time spent optimizing.
///
/// If successful, the optimized Wasm is written to `dest_optimized`.
pub fn do_optimization(
    dest_wasm: &OsStr,
    dest_optimized: &OsStr,
    optimization_level: &str,
    keep_debug_symbols: bool,
) -> Result<()> {
    log::info!(
        "Optimization level passed to wasm-opt: {}",
        optimization_level
    );

    match optimization_level {
        "0" => OptimizationOptions::new_opt_level_0(),
        "1" => OptimizationOptions::new_opt_level_1(),
        "2" => OptimizationOptions::new_opt_level_2(),
        "3" => OptimizationOptions::new_opt_level_3(),
        "4" => OptimizationOptions::new_opt_level_4(),
        "s" => OptimizationOptions::new_optimize_for_size(),
        "z" => OptimizationOptions::new_optimize_for_size_aggressively(),
        _ => panic!("Invalid optimization level {}", optimization_level),
    }
    .add_pass(Pass::Dae)
    .add_pass(Pass::Vacuum)
    .zero_filled_memory(true)
    .debug_info(keep_debug_symbols)
    .run(dest_wasm, dest_optimized)
    .expect("The wasm-opt optimization failed");

    Ok(())
}

fn check_exports(module: &Module, path: &Path) -> Result<()> {
    if module
        .export_section()
        .ok_or_else(|| BuilderError::ExportSectionNotFound(path.to_path_buf()))?
        .entries()
        .iter()
        .any(|entry| {
            matches!(entry.internal(), Internal::Function(_))
                && matches!(entry.field(), "init" | "handle")
        })
    {
        Ok(())
    } else {
        Err(BuilderError::RequiredExportFnNotFound(path.to_path_buf()).into())
    }
}
