//! Command-line interface for the Links Notation transformer (requirement R15).
//!
//! Commands:
//!   check  Compile a program and report diagnostics.
//!   run    Apply a program to input and print the resulting LiNo graph.
//!   step   Apply a single rewrite step and print the resulting graph.
//!
//! Input is read from a file (`--input`) or standard input. Exit codes:
//!   0  success (run reached normal form or a terminal rule; check compiled)
//!   2  usage, compile, or parse error
//!   3  the run stopped at a bound (step/node/deadline/cycle/cancel)

use lino_transformer::{canonical, compile_program, Execution, Options, StepOutcome, StopReason};
use std::io::{Read, Write};
use std::process::ExitCode;

const USAGE: &str = "\
lino-transformer — a rule-based transformer for Links Notation

USAGE:
    lino-transformer <COMMAND> --program <FILE> [OPTIONS]

COMMANDS:
    check    Compile a program and report diagnostics
    run      Apply a program to input and print the resulting graph
    step     Apply a single rewrite step and print the resulting graph

OPTIONS:
    -p, --program <FILE>   Program file written in Links Notation (required)
    -i, --input <FILE>     Input data file (default: standard input)
        --max-steps <N>    Step limit for run (default: 100000)
        --max-nodes <N>    Node-count limit for run
        --unbounded        Remove the step limit (trusted programs only)
        --detect-cycles    Stop run when a graph state repeats
        --trace            Print structured trace events to stderr
    -h, --help             Print this help
    -V, --version          Print version

EXIT CODES:
    0  success            2  usage/compile/parse error            3  stopped at a bound
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {}", message);
            ExitCode::from(2)
        }
    }
}

struct Cli {
    command: String,
    program: Option<String>,
    input: Option<String>,
    max_steps: Option<u64>,
    max_nodes: Option<usize>,
    unbounded: bool,
    detect_cycles: bool,
    trace: bool,
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() {
        print!("{}", USAGE);
        return Ok(ExitCode::from(2));
    }

    // Global flags that short-circuit before a command is required.
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{}", USAGE);
        return Ok(ExitCode::SUCCESS);
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("lino-transformer {}", env!("CARGO_PKG_VERSION"));
        return Ok(ExitCode::SUCCESS);
    }

    let cli = parse_args(args)?;

    match cli.command.as_str() {
        "check" => cmd_check(&cli),
        "run" => cmd_run(&cli),
        "step" => cmd_step(&cli),
        other => Err(format!(
            "unknown command '{}' (expected check, run, or step)",
            other
        )),
    }
}

fn parse_args(args: &[String]) -> Result<Cli, String> {
    let command = args[0].clone();
    let mut cli = Cli {
        command,
        program: None,
        input: None,
        max_steps: None,
        max_nodes: None,
        unbounded: false,
        detect_cycles: false,
        trace: false,
    };

    let mut i = 1;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-p" | "--program" => cli.program = Some(take_value(args, &mut i, arg)?),
            "-i" | "--input" => cli.input = Some(take_value(args, &mut i, arg)?),
            "--max-steps" => {
                let value = take_value(args, &mut i, arg)?;
                cli.max_steps = Some(value.parse().map_err(|_| {
                    format!(
                        "--max-steps expects a non-negative integer, found '{}'",
                        value
                    )
                })?);
            }
            "--max-nodes" => {
                let value = take_value(args, &mut i, arg)?;
                cli.max_nodes = Some(value.parse().map_err(|_| {
                    format!(
                        "--max-nodes expects a non-negative integer, found '{}'",
                        value
                    )
                })?);
            }
            "--unbounded" => cli.unbounded = true,
            "--detect-cycles" => cli.detect_cycles = true,
            "--trace" => cli.trace = true,
            other => return Err(format!("unknown option '{}'", other)),
        }
        i += 1;
    }

    Ok(cli)
}

fn take_value(args: &[String], i: &mut usize, flag: &str) -> Result<String, String> {
    *i += 1;
    args.get(*i)
        .cloned()
        .ok_or_else(|| format!("option '{}' requires a value", flag))
}

fn read_program(cli: &Cli) -> Result<String, String> {
    let path = cli
        .program
        .as_ref()
        .ok_or_else(|| "missing required option --program <FILE>".to_string())?;
    std::fs::read_to_string(path).map_err(|e| format!("cannot read program '{}': {}", path, e))
}

fn read_input(cli: &Cli) -> Result<String, String> {
    match &cli.input {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read input '{}': {}", path, e)),
        None => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|e| format!("cannot read standard input: {}", e))?;
            Ok(buffer)
        }
    }
}

fn build_options(cli: &Cli) -> Options {
    let max_steps = if cli.unbounded {
        None
    } else {
        Some(cli.max_steps.unwrap_or(Options::DEFAULT_MAX_STEPS))
    };
    Options {
        max_steps,
        max_nodes: cli.max_nodes,
        deadline: None,
        detect_cycles: cli.detect_cycles,
        trace: cli.trace,
        cancel: None,
    }
}

fn cmd_check(cli: &Cli) -> Result<ExitCode, String> {
    let source = read_program(cli)?;
    match compile_program(&source) {
        Ok(program) => {
            println!(
                "ok: {} rule{}",
                program.rules.len(),
                if program.rules.len() == 1 { "" } else { "s" }
            );
            Ok(ExitCode::SUCCESS)
        }
        Err(diagnostics) => {
            eprintln!("{}", diagnostics);
            Ok(ExitCode::from(2))
        }
    }
}

fn cmd_run(cli: &Cli) -> Result<ExitCode, String> {
    let source = read_program(cli)?;
    let program =
        compile_program(&source).map_err(|d| format!("program did not compile:\n{}", d))?;
    let input = read_input(cli)?;
    let graph =
        lino_transformer::parse_lino(&input).map_err(|e| format!("input did not parse: {}", e))?;

    let mut execution = Execution::new(program, graph, build_options(cli));
    let result = execution.run();

    let mut stdout = std::io::stdout();
    writeln!(stdout, "{}", canonical(&result.graph)).map_err(|e| e.to_string())?;

    if cli.trace {
        for event in &result.trace {
            eprintln!("{}", event);
        }
    }
    eprintln!(
        "stopped: {} after {} step{}",
        result.stop_reason,
        result.steps,
        if result.steps == 1 { "" } else { "s" }
    );

    Ok(exit_for(&result.stop_reason))
}

fn cmd_step(cli: &Cli) -> Result<ExitCode, String> {
    let source = read_program(cli)?;
    let program =
        compile_program(&source).map_err(|d| format!("program did not compile:\n{}", d))?;
    let input = read_input(cli)?;
    let graph =
        lino_transformer::parse_lino(&input).map_err(|e| format!("input did not parse: {}", e))?;

    let mut execution = Execution::new(program, graph, build_options(cli));
    let outcome = execution.step();

    let mut stdout = std::io::stdout();
    writeln!(stdout, "{}", canonical(execution.graph())).map_err(|e| e.to_string())?;

    match outcome {
        StepOutcome::Applied {
            rule_label,
            terminal,
            ..
        } => {
            eprintln!(
                "applied: {}{}",
                rule_label,
                if terminal { " [terminal]" } else { "" }
            );
            Ok(ExitCode::SUCCESS)
        }
        StepOutcome::NormalForm => {
            eprintln!("normal form: no rule applied");
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn exit_for(stop_reason: &StopReason) -> ExitCode {
    if stop_reason.is_complete() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}
