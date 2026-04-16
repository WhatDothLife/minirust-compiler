use std::fs;
use std::process::Command;

// Helper: Runs a command and returns trimmed stdout.
// Panics with stderr if the process fails to start or exits with an error.
fn run_cmd(cmd: &str, args: &[&str]) -> String {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute command: {} {:?}", cmd, args));

    if !out.status.success() {
        panic!(
            "Command '{}' exited with an error.\nStderr: {}",
            cmd,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn test_compiler_suite() {
    let mut paths: Vec<_> = fs::read_dir("tests/suite")
        .expect("Directory 'tests/suite' not found")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "mrs"))
        .collect();

    paths.sort();

    for path in paths {
        let path_str = path.to_str().unwrap();
        println!("Testing: {}", path_str);

        // --- REFERENCE (rustc) ---
        let rustc_bin = "target/ref_bin";
        let rustc_status = Command::new("rustc")
            .args([path_str, "-o", rustc_bin])
            .status()
            .expect("Failed to run 'rustc'.");

        assert!(
            rustc_status.success(),
            "Reference compilation failed for '{}'",
            path_str
        );
        let expected = run_cmd(format!("./{}", rustc_bin).as_str(), &[]);

        // --- COMPILER PIPELINE ---
        let my_compiler = env!("CARGO_BIN_EXE_minirust_compiler");
        let asm_path = "target/output.S";
        let riscv_bin = "target/riscv_bin";

        // A: Run compiler
        let comp_status = Command::new(my_compiler)
            .args(["-i", path_str, "-o", asm_path])
            .status()
            .expect("Failed to run your compiler binary");

        assert!(
            comp_status.success(),
            "Your compiler crashed on source '{}'",
            path_str
        );

        // B: Run GCC (RISC-V Cross-Compiler)
        let gcc_status = Command::new("riscv64-linux-gnu-gcc")
            .args(["-static", asm_path, "runtime/runtime.c", "-o", riscv_bin])
            .status()
            .expect("Failed to run 'riscv64-linux-gnu-gcc'. Is the cross-toolchain installed?");

        assert!(
            gcc_status.success(),
            "GCC failed to assemble/link '{}'",
            path_str
        );

        // C: Run QEMU
        let actual = run_cmd("qemu-riscv64-static", &[riscv_bin]);

        // --- COMPARE OUTPUTS ---
        assert_eq!(
            expected, actual,
            "\nOutput mismatch in file: {}\nExpected (rustc): {}\nActual (minirust): {}\n",
            path_str, expected, actual
        );

        let _ = fs::remove_file(rustc_bin);
    }
}
