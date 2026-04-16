use std::path::PathBuf;
use std::{fs, io};

use clap::{Arg, ArgAction, Command};
use minirust_compiler::{ast, codegen, ir, parse, semant, util::Pretty};

fn error(message: &str) -> ! {
    eprintln!("{}", message);
    std::process::exit(1);
}

fn main() -> io::Result<()> {
    let matches = Command::new("mrscomp")
        .about("Compiles a subset of Rust to RISC-V assembly.")
        .arg(
            Arg::new("src")
                .short('i')
                .long("src")
                .value_name("PATH")
                .value_parser(src_parser)
                .help("Source file to compile")
                .required(true),
        )
        .arg(
            Arg::new("tgt")
                .short('o')
                .long("tgt")
                .value_name("PATH")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Target file to save the assembly to (default: print to stdout)"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Debug print the output of all passes")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let src_path: &PathBuf = matches.get_one::<PathBuf>("src").unwrap();

    let verbose: bool = *matches.get_one::<bool>("verbose").unwrap_or(&false);

    if verbose {
        println!("\n===== READING SOURCE FILE =====\n");
    }
    let src_str = fs::read_to_string(src_path).unwrap_or_else(|e| {
        error(&format!(
            "Failed reading from source file {:?}: {}",
            src_path, e
        ))
    });

    if verbose {
        println!("{}", src_str);
    }

    let ir_program = lower(&src_str, verbose).unwrap_or_else(|e| error(&e.render(&src_str, true)));

    if verbose {
        println!("{}", ir_program.pretty(4));
    }

    let asm_program = codegen::select(ir_program);

    if verbose {
        println!("\n===== WRITING OUTPUT ASSEMBLY =====\n");
    }

    let tgt_str = codegen::emit(&asm_program);

    let tgt_path = matches.get_one::<PathBuf>("tgt");

    match tgt_path {
        None => {
            println!("{}", tgt_str);
        }
        Some(path) => {
            if let Err(err) = fs::write(&path, tgt_str) {
                eprintln!("Failed writing to target file {:?}: {}", path, err);
                std::process::exit(1);
            }
            if verbose {
                println!("Wrote output assembly to file {:?}.", path);
            }
        }
    }

    Ok(())
}

fn lower(src_str: &str, verbose: bool) -> ast::Result<ir::Program> {
    if verbose {
        println!("\n===== PARSING PROGRAM =====\n");
    }

    let ast_program = parse::program(src_str)?;

    if verbose {
        println!("{}", ast_program.pretty(4));
    }

    if verbose {
        println!("\n===== TYPE CHECKING =====\n");
    }

    let ir_program = semant::check(&ast_program)?;

    Ok(ir_program)
}

fn src_parser(s: &str) -> Result<PathBuf, String> {
    if s.trim().is_empty() {
        return Err("src must not be empty".into());
    }

    let p = PathBuf::from(s);

    if !p.exists() {
        return Err(format!("file does not exist: {}", s));
    }

    if !p.is_file() {
        return Err(format!("not a file: {}", s));
    }

    Ok(p)
}
