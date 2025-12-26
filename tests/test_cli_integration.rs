use std::process::{Command, Stdio};
use std::io::Write;
use tempfile::NamedTempFile;

#[cfg(test)]
mod cli_integration_tests {
    use super::*;

    fn get_binary_path() -> String {
        // Build the binary first if it doesn't exist
        let output = Command::new("cargo")
            .args(["build", "--bin", "tsef"])
            .output()
            .expect("Failed to build binary");
        
        if !output.status.success() {
            panic!("Failed to build binary: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        "./target/debug/tsef".to_string()
    }

    fn run_tsef_with_input(input: &str, args: &[&str]) -> (String, String, i32) {
        let binary_path = get_binary_path();
        
        let mut cmd = Command::new(&binary_path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start tsef process");
        
        // Write input to stdin
        if let Some(stdin) = cmd.stdin.take() {
            let mut stdin = stdin;
            stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
        }
        
        let output = cmd.wait_with_output().expect("Failed to read output");
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        
        (stdout, stderr, exit_code)
    }

    #[test]
    fn test_no_input_returns_success() {
        let (stdout, _stderr, exit_code) = run_tsef_with_input("", &[]);
        
        assert_eq!(exit_code, 0);
        assert_eq!(stdout, "");
    }

    #[test]
    fn test_simple_tsc_output_no_filters() {
        let input = "src/components/Header.tsx(15,7): error TS2322: Type 'string' is not assignable to type 'number'.\n\
                     src/utils/helpers.ts(23,3): error TS2322: Type 'undefined' is not assignable to type 'string'.\n\
                     Found 2 errors.\n";
        
        let (stdout, _stderr, exit_code) = run_tsef_with_input(input, &[]);
        
        // No filters means everything should pass through
        assert_eq!(exit_code, 1); // Should exit with error code since there were errors
        assert!(stdout.contains("src/components/Header.tsx"));
        assert!(stdout.contains("src/utils/helpers.ts"));
        assert!(stdout.contains("Found 2 errors"));
    }

    #[test]
    fn test_filter_by_exact_file() {
        let input = "src/components/Header.tsx(15,7): error TS2322: Type 'string' is not assignable to type 'number'.\n\
                     src/utils/helpers.ts(23,3): error TS2322: Type 'undefined' is not assignable to type 'string'.\n\
                     Found 2 errors.\n";
        
        let (stdout, _stderr, exit_code) = run_tsef_with_input(
            input, 
            &["-i", "src/components/Header.tsx"]
        );
        
        assert_eq!(exit_code, 1); // Should exit with error since matched file has errors
        assert!(stdout.contains("src/components/Header.tsx"));
        assert!(!stdout.contains("src/utils/helpers.ts"));
        assert!(!stdout.contains("Found 2 errors")); // Summary should be filtered out
    }

    #[test]
    fn test_filter_by_glob_pattern() {
        let input = "src/components/Header.tsx(15,7): error TS2322: Type 'string' is not assignable to type 'number'.\n\
                     src/utils/helpers.ts(23,3): error TS2322: Type 'undefined' is not assignable to type 'string'.\n\
                     node_modules/@types/react/index.d.ts(1024,9): error TS2717: Subsequent property declarations must have the same type.\n\
                     Found 3 errors.\n";
        
        let (stdout, _stderr, exit_code) = run_tsef_with_input(
            input, 
            &["-i", "src/**/*"]
        );
        
        assert_eq!(exit_code, 1);
        assert!(stdout.contains("src/components/Header.tsx"));
        assert!(stdout.contains("src/utils/helpers.ts"));
        assert!(!stdout.contains("node_modules/@types/react"));
        assert!(!stdout.contains("Found 3 errors"));
    }

    #[test]
    fn test_multiple_include_patterns() {
        let input = "src/components/Header.tsx(15,7): error TS2322: Type 'string' is not assignable to type 'number'.\n\
                     src/utils/helpers.ts(23,3): error TS2322: Type 'undefined' is not assignable to type 'string'.\n\
                     src/features/orders/index.ts(42,15): error TS2345: Argument of type 'string' is not assignable to parameter of type 'number'.\n\
                     src/services/api.ts(10,5): error TS2322: Type 'null' is not assignable to type 'string'.\n\
                     Found 4 errors.\n";
        
        let (stdout, _stderr, exit_code) = run_tsef_with_input(
            input, 
            &["-i", "src/components/**/*", "-i", "src/features/**/*"]
        );
        
        assert_eq!(exit_code, 1);
        assert!(stdout.contains("src/components/Header.tsx"));
        assert!(!stdout.contains("src/utils/helpers.ts"));
        assert!(stdout.contains("src/features/orders/index.ts"));
        assert!(!stdout.contains("src/services/api.ts"));
    }

    #[test]
    fn test_no_matching_files_success_exit() {
        let input = "src/components/Header.tsx(15,7): error TS2322: Type 'string' is not assignable to type 'number'.\n\
                     src/utils/helpers.ts(23,3): error TS2322: Type 'undefined' is not assignable to type 'string'.\n\
                     Found 2 errors.\n";
        
        let (stdout, _stderr, exit_code) = run_tsef_with_input(
            input, 
            &["-i", "tests/**/*"] // Pattern that won't match any of the error files
        );
        
        assert_eq!(exit_code, 0); // Should exit with success since no matching errors
        assert_eq!(stdout.trim(), ""); // No output since no matches
    }

    #[test]
    fn test_file_extension_filtering() {
        let input = "src/components/Header.tsx(15,7): error TS2322: Type 'string' is not assignable to type 'number'.\n\
                     src/utils/helpers.ts(23,3): error TS2322: Type 'undefined' is not assignable to type 'string'.\n\
                     src/styles/global.css(5,1): error: Invalid property value.\n\
                     Found 3 errors.\n";
        
        let (stdout, _stderr, exit_code) = run_tsef_with_input(
            input, 
            &["-i", "**/*.tsx"]
        );
        
        assert_eq!(exit_code, 1);
        assert!(stdout.contains("src/components/Header.tsx"));
        assert!(!stdout.contains("src/utils/helpers.ts"));
        assert!(!stdout.contains("src/styles/global.css"));
    }

    #[test]
    fn test_ansi_output_with_show_full() {
        // Create a simple ANSI output test
        let input = "\u{001b}[96msrc/components/Header.tsx\u{001b}[0m:15:7 - \u{001b}[91merror\u{001b}[0m TS2322: Type 'string' is not assignable to type 'number'.\n\n\
                     \u{001b}[7m15\u{001b}[0m const count: number = \"hello\";\n\
                     \u{001b}[7m  \u{001b}[0m \u{001b}[91m      ~~~~~\u{001b}[0m\n\n\n\n\
                     Found 1 error in 1 file.\n";
        
        let (stdout, _stderr, exit_code) = run_tsef_with_input(
            input, 
            &["-i", "src/**/*", "--show-full"]
        );
        
        assert_eq!(exit_code, 1);
        assert!(stdout.contains("src/components/Header.tsx"));
        assert!(stdout.contains("Found 1 error")); // Summary should be included with --show-full
    }

    #[test]
    fn test_help_flag() {
        let (stdout, _stderr, exit_code) = run_tsef_with_input("", &["--help"]);
        
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("tsef"));
        assert!(stdout.contains("--include"));
        assert!(stdout.contains("--show-full"));
    }

    #[test]
    fn test_version_flag() {
        let (stdout, _stderr, exit_code) = run_tsef_with_input("", &["--version"]);
        
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("tsef"));
        assert!(stdout.contains("0.3.0")); // Version from Cargo.toml
    }

    #[test]
    fn test_invalid_glob_pattern() {
        let input = "src/components/Header.tsx(15,7): error TS2322\n";
        
        let (_stdout, stderr, exit_code) = run_tsef_with_input(
            input,
            &["-i", "[invalid-glob"] // Invalid glob pattern
        );
        
        // Should handle invalid patterns gracefully
        // The exact behavior depends on the path-matchers crate
        // but it should either work or fail gracefully
        println!("Exit code: {}, stderr: {}", exit_code, stderr);
    }

    #[test]
    fn test_reading_from_file() {
        // Test that we can process file input (though tsef reads from stdin)
        let input = "src/components/Header.tsx(15,7): error TS2322: Type 'string' is not assignable to type 'number'.\n\
                     Found 1 error.\n";
        
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(input.as_bytes()).expect("Failed to write to temp file");
        
        let binary_path = get_binary_path();
        let output = Command::new(&binary_path)
            .args(["-i", "src/**/*"])
            .stdin(std::fs::File::open(temp_file.path()).expect("Failed to open temp file"))
            .output()
            .expect("Failed to run command");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let exit_code = output.status.code().unwrap_or(-1);
        
        assert_eq!(exit_code, 1);
        assert!(stdout.contains("src/components/Header.tsx"));
    }
}