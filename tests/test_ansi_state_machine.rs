use ansi_parser::AnsiSequence;
use tsef::{StateMachine, ansi_state_machine::AnsiStateMachine, State};
use heapless::Vec as HeaplessVec;

#[cfg(test)]
mod ansi_state_machine_tests {
    use super::*;

    fn create_test_ansi_sequence() -> AnsiSequence {
        // Create a typical ANSI sequence used in TypeScript compiler output
        let mut vec = HeaplessVec::new();
        vec.push(96).unwrap();
        AnsiSequence::SetGraphicsMode(vec) // Cyan color
    }

    fn create_ansi_line_with_path(path: &str) -> String {
        format!("\u{001b}[96m{}\u{001b}[0m:15:7 - error TS2322", path)
    }
    


    #[test]
    fn test_ansi_state_machine_creation() {
        let identifier = create_test_ansi_sequence();
        let include = vec!["src/**/*".to_string()];
        let sm = AnsiStateMachine::new(identifier.clone(), include);
        
        assert_eq!(sm.identifier, identifier);
        assert_eq!(sm.state, State::ParseToPause);
        assert!(!sm.is_finished());
    }

    #[test]
    fn test_empty_include_list_matches_all() {
        let identifier = create_test_ansi_sequence();
        let include = vec![];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // With empty include list, everything matches (should_block = false)
        // Start in ParseToPause, stay in ParseToPause, print everything
        let ansi_line = create_ansi_line_with_path("src/components/Header.tsx");
        let (state, should_print) = sm.run(&ansi_line);
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print); // parsing_to_pause=true means print
        
        // All paths match empty include list, so same behavior
        let ansi_line = create_ansi_line_with_path("node_modules/@types/react/index.d.ts");
        let (state, should_print) = sm.run(&ansi_line);
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print);
    }

    #[test] 
    fn test_filter_by_exact_path() {
        let identifier = create_test_ansi_sequence();
        let include = vec!["src/components/Header.tsx".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // When we hit a matching path:
        // - ANSI sequence matches, path matches include → should_block=false
        // - In ParseToPause: should_flip = false → stay in ParseToPause
        // - Return: (ParseToPause, parsing_to_pause=true)
        let ansi_line = create_ansi_line_with_path("src/components/Header.tsx");
        let (state, should_print) = sm.run(&ansi_line);
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print); // parsing_to_pause=true means print
        
        // Following lines should continue printing (stay in same state)
        let (state, should_print) = sm.run(&"  some error details".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print); // parsing_to_pause=true means print
    }

    #[test]
    fn test_filter_by_glob_pattern() {
        let identifier = create_test_ansi_sequence();
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Should match paths under src/ → should_block=false, stay in ParseToPause
        let ansi_line = create_ansi_line_with_path("src/components/Header.tsx");
        let (state, should_print) = sm.run(&ansi_line);
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print);
        
        // Should not match node_modules → should_block=true, flip to ParseToContinue
        let ansi_line = create_ansi_line_with_path("node_modules/@types/react/index.d.ts");
        let (state, should_print) = sm.run(&ansi_line);
        assert_eq!(*state, State::ParseToContinue);
        assert!(!should_print); // parsing_to_pause=false means don't print
    }

    #[test]
    fn test_state_transitions() {
        let identifier = create_test_ansi_sequence();
        let include = vec!["src/components/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Start in ParseToPause
        assert_eq!(sm.state, State::ParseToPause);
        
        // Hit a matching path → stay in ParseToPause (should_block=false, no flip)
        let ansi_line = create_ansi_line_with_path("src/components/Header.tsx");
        let (state, should_print) = sm.run(&ansi_line);
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print); // parsing_to_pause=true means print
        
        // Continue with non-ANSI lines (stay in same state)
        let (state, should_print) = sm.run(&"  error details".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print);
        
        // Hit a non-matching path → flip to ParseToContinue (should_block=true, flip)
        let ansi_line = create_ansi_line_with_path("src/utils/helpers.ts");
        let (state, should_print) = sm.run(&ansi_line);
        assert_eq!(*state, State::ParseToContinue);
        assert!(!should_print); // parsing_to_pause=false means don't print
    }

    #[test]
    fn test_empty_lines_trigger_check_end() {
        let identifier = create_test_ansi_sequence();
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Process an empty line
        let (state, should_print) = sm.run(&"".to_string());
        assert!(matches!(*state, State::CheckEnd(true, 1)));
        assert!(should_print); // In CheckEnd, we use the go_back_state boolean
        
        // Another empty line
        let (state, should_print) = sm.run(&"".to_string());
        assert!(matches!(*state, State::CheckEnd(true, 2)));
        assert!(should_print);
    }

    #[test]
    fn test_end_state_detection() {
        let identifier = create_test_ansi_sequence();
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Simulate the end sequence: multiple empty lines followed by summary text
        sm.run(&"".to_string());
        sm.run(&"".to_string());
        
        // A line that starts with text (not ANSI) after empty lines should trigger End state
        let (state, should_print) = sm.run(&"Found 4 errors in 4 files.".to_string());
        assert_eq!(*state, State::End);
        assert!(should_print); // End state prints everything
        
        assert!(sm.is_finished());
    }

    #[test]
    fn test_non_ansi_lines_in_ansi_mode() {
        let identifier = create_test_ansi_sequence();
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Non-ANSI lines should maintain current state
        let (state, should_print) = sm.run(&"some regular text".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print); // In ParseToPause, parsing_to_pause=true means print
        
        // Transition to ParseToContinue by hitting non-matching ANSI line
        let ansi_line = create_ansi_line_with_path("other/file.ts");
        sm.run(&ansi_line); // This should flip to ParseToContinue
        
        // Now non-ANSI lines should not print
        let (state, should_print) = sm.run(&"error details".to_string());
        assert_eq!(*state, State::ParseToContinue);
        assert!(!should_print); // parsing_to_pause=false means don't print
    }

    #[test]
    fn test_wrong_ansi_sequence_ignored() {
        let identifier = create_test_ansi_sequence();
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Create a line with different ANSI sequence (91 instead of 96)
        let wrong_ansi_line = "\u{001b}[91msrc/components/Header.tsx\u{001b}[0m:15:7 - error";
        let (state, should_print) = sm.run(&wrong_ansi_line.to_string());
        
        // Should maintain current state since ANSI sequence doesn't match
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print); // Non-ANSI line behavior: print based on current state
    }

    #[test]
    fn test_malformed_ansi_line() {
        let identifier = create_test_ansi_sequence();
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Line with non-matching ANSI sequence (won't trigger path extraction)
        // This is treated as a non-ANSI line, so prints based on current state
        let malformed_line = "\u{001b}[91msome red text\u{001b}[0m";
        let (state, should_print) = sm.run(&malformed_line.to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print); // ParseToPause state = print
    }

    #[test]
    fn test_end_state_prints_everything() {
        let identifier = create_test_ansi_sequence();
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Transition to end state
        sm.state = State::End;
        
        // Everything should print in end state
        let (state, should_print) = sm.run(&"any line".to_string());
        assert_eq!(*state, State::End);
        assert!(should_print);
        
        let (state, should_print) = sm.run(&"another line".to_string());
        assert_eq!(*state, State::End);
        assert!(should_print);
    }

    #[test]
    fn test_check_end_state_transitions() {
        let identifier = create_test_ansi_sequence();
        let include = vec!["src/**/*".to_string()];
        let mut sm = AnsiStateMachine::new(identifier, include);
        
        // Set to CheckEnd state with go_back_state=true (meaning we go back to ParseToPause)
        sm.state = State::CheckEnd(true, 1);
        
        // Non-empty line should transition back to original state and process normally
        let ansi_line = create_ansi_line_with_path("src/components/Header.tsx");
        let (state, should_print) = sm.run(&ansi_line);
        assert_eq!(*state, State::ParseToPause); // Matching path, no flip
        assert!(should_print);
    }
}