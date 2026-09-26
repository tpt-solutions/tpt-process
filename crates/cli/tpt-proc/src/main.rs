//! `tpt-proc` — config-driven flowsheet runner.
//!
//! ```text
//! tpt-proc run <config.toml> [-o results.json]
//! tpt-proc components
//! ```
//!
//! `run` reads a TOML flowsheet definition, solves it with the
//! sequential-modular engine, and prints a JSON report (or writes it to
//! `-o <file>`). `components` lists the built-in chemical database.

use std::io::Write as _;
use std::process::ExitCode;

use serde::Serialize;
use tpt_proc::CliError;
use tpt_proc_thermo_database::ChemicalDatabase;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match dispatch(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(args: &[String]) -> Result<(), CliError> {
    match args.split_first() {
        Some((cmd, rest)) if cmd == "run" => run(rest),
        Some((cmd, rest)) if cmd == "components" => {
            let _ = rest;
            list_components()
        }
        Some((cmd, _)) if cmd == "--help" || cmd == "-h" || cmd == "help" => {
            print_help();
            Ok(())
        }
        _ => {
            print_help();
            Err(CliError::Config(
                "expected `tpt-proc run <config.toml>` or `tpt-proc components`".into(),
            ))
        }
    }
}

fn run(args: &[String]) -> Result<(), CliError> {
    let mut config_path: Option<&String> = None;
    let mut output_path: Option<&String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                output_path = args.get(i + 1);
                i += 2;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            other if config_path.is_none() && !other.starts_with('-') => {
                config_path = Some(&args[i]);
                i += 1;
            }
            other => {
                return Err(CliError::Config(format!("unrecognized argument {other:?}")));
            }
        }
    }
    let config_path = config_path.ok_or_else(|| {
        CliError::Config("usage: tpt-proc run <config.toml> [-o results.json]".into())
    })?;

    let text = std::fs::read_to_string(config_path)
        .map_err(|e| CliError::Config(format!("cannot read {config_path}: {e}")))?;
    let (result, components) = tpt_proc::run_config_str(&text)?;

    let report = Report {
        converged: result.converged,
        iterations: result.iterations,
        tear_residual: result.tear_residual,
        components,
        streams: result
            .streams
            .values()
            .map(|s| StreamReport {
                id: s.id().value(),
                name: s.name().to_string(),
                temperature_k: finite_or_null(s.temperature()),
                pressure_pa: finite_or_null(s.pressure()),
                molar_flow_mol_s: s.total_flow().ok(),
                phase: s.phase().map(phase_name),
                mole_fractions: s.composition().map(|c| c.as_slice().to_vec()),
            })
            .collect(),
    };
    let mut json =
        serde_json::to_string_pretty(&report).map_err(|e| CliError::Config(e.to_string()))?;
    json.push('\n');

    match output_path {
        Some(path) => std::fs::write(path, &json)
            .map_err(|e| CliError::Config(format!("cannot write {path}: {e}")))?,
        None => {
            std::io::stdout()
                .write_all(json.as_bytes())
                .map_err(|e| CliError::Config(e.to_string()))?;
        }
    }
    Ok(())
}

fn list_components() -> Result<(), CliError> {
    let db = ChemicalDatabase::builtin();
    let mut names = db.component_names();
    names.sort();
    for name in &names {
        println!("{name}");
    }
    Ok(())
}

fn print_help() {
    println!(
        "tpt-proc — config-driven flowsheet runner for tpt-process\n\
         \n\
         USAGE:\n\
         \x20 tpt-proc run <config.toml> [-o results.json]\n\
         \x20 tpt-proc components\n\
         \n\
         A config TOML defines [flowsheet], optional [thermo], [[streams]],\n\
         [[units]] (mixer, splitter, flash, heater, cooler, valve, pump), and\n\
         [[connections]]. See configs/recycle-demo.toml in the repository."
    );
}

fn finite_or_null(v: f64) -> Option<f64> {
    v.is_finite().then_some(v)
}

fn phase_name(phase: tpt_proc_core::PhaseState) -> String {
    use tpt_proc_core::PhaseState::*;
    match phase {
        Liquid => "liquid".into(),
        Vapor => "vapor".into(),
        TwoPhase { vapor_fraction } => format!("two-phase (β = {vapor_fraction:.4})"),
        Supercritical => "supercritical".into(),
        Solid => "solid".into(),
    }
}

/// Top-level JSON report shape.
#[derive(Serialize)]
struct Report {
    converged: bool,
    iterations: u32,
    tear_residual: f64,
    components: Vec<String>,
    streams: Vec<StreamReport>,
}

/// Per-stream JSON report shape.
#[derive(Serialize)]
struct StreamReport {
    id: u64,
    name: String,
    temperature_k: Option<f64>,
    pressure_pa: Option<f64>,
    molar_flow_mol_s: Option<f64>,
    phase: Option<String>,
    mole_fractions: Option<Vec<f64>>,
}
