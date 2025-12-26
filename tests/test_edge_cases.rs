use tsef::{path_matches, StateMachine, simple_state_machine::SimpleStateMachine, ansi_state_machine::AnsiStateMachine, State};
use ansi_parser::AnsiSequence;
use heapless::Vec as HeaplessVec;

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_path_matches_edge_cases() {
        // Test with empty path
        let include = vec!["src/**/*".to_string()];
        assert!(!path_matches(&include, ""));
        
        // Test with whitespace-only path
        assert!(!path_matches(&include, "   "));
        
        // Test with very long path
        let long_path = "a/".repeat(1000) + "file.ts";
        assert!(!path_matches(&include, &long_path));
        
        // Test with special characters in path
        assert!(path_matches(&include, "src/components/Header@v2.tsx"));
        assert!(path_matches(&include, "src/utils/helper-functions.ts"));
        assert!(path_matches(&include, "src/test file with spaces.ts"));
    }

    #[test]
    fn test_unicode_paths() {
        let include = vec!["src/**/*".to_string()];
        
        // Test with unicode characters
        assert!(path_matches(&include, "src/components/ファイル.tsx"));
        assert!(path_matches(&include, "src/utils/测试.ts"));
        assert!(path_matches(&include, "src/файл.ts"));
    }

    #[test]
    fn test_glob_pattern_edge_cases() {
        // Test with malformed glob patterns that might cause issues
        let patterns = vec![
            "**".to_string(),        // Just double asterisk
            "*".to_string(),         // Just single asterisk
            "?".to_string(),         // Just question mark
            "/".to_string(),         // Just slash
            ".".to_string(),         // Just dot
            "..".to_string(),        // Double dot
        ];
        
        for pattern in patterns {
            let include = vec![pattern.clone()];
            // These shouldn't crash, even if they don't match as expected
            let _ = path_matches(&include, "src/file.ts");
        }
    }

    #[test]
    fn test_extremely_long_input_lines() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Create a very long line
        let long_line = "src/file.ts(".to_string() + &"a".repeat(10000) + "): error";
        
        // Should handle long lines gracefully
        let (state, _should_print) = sm.run(&long_line);
        assert_eq!(*state, State::ParseToPause);
        assert!(_should_print); // Should match src/**/*
    }

    #[test]
    fn test_many_empty_lines() {
        let mut vec = HeaplessVec::new();
        vec.push(96).unwrap();
        let identifier = AnsiSequence::SetGraphicsMode(vec);
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Process many empty lines
        for _ in 0..100 {
            let (state, _) = sm.run(&"".to_string());
            if let State::CheckEnd(_, count) = state {
                if *count > 50 {
                    break; // Avoid infinite processing
                }
            }
        }
        
        // Should eventually reach a stable state
        assert!(matches!(sm.state, State::CheckEnd(_, _)));
    }

    #[test]
    fn test_mixed_line_endings() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Test with different line ending characters (though these are typically
        // handled by the input reader, not the state machine)
        let lines_with_endings = vec![
            "src/file1.ts(1,1): error\r\n",
            "src/file2.ts(2,2): error\n",
            "src/file3.ts(3,3): error\r",
        ];
        
        for line in lines_with_endings {
            let clean_line = line.trim_end_matches(|c| c == '\r' || c == '\n');
            let (state, _should_print) = sm.run(&clean_line.to_string());
            assert_eq!(*state, State::ParseToPause);
            assert!(_should_print);
        }
    }

    #[test]
    fn test_binary_data_in_input() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Test with binary data (should not crash)
        let binary_line = String::from_utf8_lossy(&[0, 1, 2, 3, 255, 254, 253]);
        let (state, _should_print) = sm.run(&binary_line.to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(!_should_print); // Binary data unlikely to match src/**/*
    }

    #[test]
    fn test_nested_parentheses() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Test with nested parentheses
        let line = "src/file.ts(function(x) { return x; }): error";
        let (state, _should_print) = sm.run(&line.to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(_should_print); // Should extract "src/file.ts" as path
    }

    #[test]
    fn test_no_parentheses_in_line() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Test line without parentheses
        let line = "src/file.ts: some error message";
        let (state, _should_print) = sm.run(&line.to_string());
        assert_eq!(*state, State::ParseToPause);
        // Should use entire line as path, which should match src/**/*
        assert!(_should_print);
    }

    #[test]
    fn test_ansi_sequence_at_different_positions() {
        let mut vec = HeaplessVec::new();
        vec.push(96).unwrap();
        let identifier = AnsiSequence::SetGraphicsMode(vec);
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // ANSI sequence not at beginning
        let line = "prefix \u{001b}[96msrc/file.ts\u{001b}[0m: error";
        let (state, _should_print) = sm.run(&line.to_string());
        // Behavior depends on ANSI parser implementation
        // Should handle gracefully without crashing
        assert!(matches!(*state, State::ParseToPause | State::ParseToContinue));
    }

    #[test]
    fn test_multiple_ansi_sequences_same_line() {
        let mut vec = HeaplessVec::new();
        vec.push(96).unwrap();
        let identifier = AnsiSequence::SetGraphicsMode(vec);
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Multiple ANSI sequences in one line
        let line = "\u{001b}[96msrc/file.ts\u{001b}[0m:\u{001b}[93m15\u{001b}[0m:\u{001b}[93m7\u{001b}[0m - error";
        let (state, _should_print) = sm.run(&line.to_string());
        
        // Should process the first matching sequence
        assert!(matches!(*state, State::ParseToPause | State::ParseToContinue));
    }

    #[test]
    fn test_invalid_ansi_sequences() {
        let mut vec = HeaplessVec::new();
        vec.push(96).unwrap();
        let identifier = AnsiSequence::SetGraphicsMode(vec);
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Invalid/incomplete ANSI sequences that won't panic
        let lines = vec![
            "\u{001b}[src/file.ts",      // Missing closing - won't be parsed as ANSI
            "\u{001b}src/file.ts",       // Missing bracket - won't be parsed as ANSI
            "\u{001b}[999msrc/file.ts",  // Invalid code - won't match our identifier
            "normal text line",          // No ANSI at all
        ];
        
        for line in lines {
            let (state, _) = sm.run(&line.to_string());
            // Should handle gracefully without panicking
            assert!(matches!(*state, State::ParseToPause | State::ParseToContinue | State::CheckEnd(_, _)));
        }
    }

    #[test]
    fn test_state_machine_reset_behavior() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include.clone());
        
        // Process some lines
        sm.run(&"src/file1.ts(1,1): error".to_string());
        sm.run(&"some other line".to_string());
        
        // Create a new state machine - should behave the same
        let mut sm2 = SimpleStateMachine::new(include);
        let (state1, print1) = sm.run(&"src/file2.ts(2,2): error".to_string());
        let (state2, print2) = sm2.run(&"src/file2.ts(2,2): error".to_string());
        
        assert_eq!(state1, state2);
        assert_eq!(print1, print2);
    }

    #[test]
    fn test_maximum_check_end_transitions() {
        let mut vec = HeaplessVec::new();
        vec.push(96).unwrap();
        let identifier = AnsiSequence::SetGraphicsMode(vec);
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Force into CheckEnd state with high count
        sm.state = State::CheckEnd(true, 100);
        
        // Should still handle transitions properly
        let (state, _should_print) = sm.run(&"Found 1 error.".to_string());
        assert_eq!(*state, State::End);
        assert!(_should_print);
    }

    #[test] 
    fn test_concurrent_state_machines() {
        // Test that multiple state machines don't interfere with each other
        let include1 = vec!["src/components/**/*".to_string()];
        let include2 = vec!["src/utils/**/*".to_string()];
        
        let mut sm1 = SimpleStateMachine::new(include1);
        let mut sm2 = SimpleStateMachine::new(include2);
        
        let test_line = "src/components/Header.tsx(15,7): error";
        
        let (_, print1) = sm1.run(&test_line.to_string());
        let (_, print2) = sm2.run(&test_line.to_string());
        
        assert!(print1);  // Should match components pattern
        assert!(!print2); // Should not match utils pattern
    }
}