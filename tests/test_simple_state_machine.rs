use tsef::{StateMachine, simple_state_machine::SimpleStateMachine, State};

#[cfg(test)]
mod simple_state_machine_tests {
    use super::*;

    #[test]
    fn test_simple_state_machine_creation() {
        let include = vec!["src/**/*".to_string()];
        let sm = SimpleStateMachine::new(include);
        assert_eq!(sm.state, State::ParseToPause);
        assert!(!sm.is_finished());
    }

    #[test]
    fn test_empty_include_list_matches_all() {
        let include = vec![];
        let mut sm = SimpleStateMachine::new(include);
        
        // Should match all lines when include list is empty
        let (state, should_print) = sm.run(&"src/components/Header.tsx(15,7): error TS2322".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print);
        
        let (state, should_print) = sm.run(&"node_modules/@types/react/index.d.ts(1024,9): error TS2717".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print);
    }

    #[test]
    fn test_filter_by_exact_path() {
        let include = vec!["src/components/Header.tsx".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Should match exact path
        let (state, should_print) = sm.run(&"src/components/Header.tsx(15,7): error TS2322".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print);
        
        // Should not match different path
        let (state, should_print) = sm.run(&"src/utils/helpers.ts(23,3): error TS2322".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(!should_print);
    }

    #[test]
    fn test_filter_by_glob_pattern() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Should match paths under src/
        let (state, should_print) = sm.run(&"src/components/Header.tsx(15,7): error TS2322".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print);
        
        let (state, should_print) = sm.run(&"src/utils/helpers.ts(23,3): error TS2322".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print);
        
        // Should not match node_modules paths
        let (state, should_print) = sm.run(&"node_modules/@types/react/index.d.ts(1024,9): error TS2717".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(!should_print);
    }

    #[test]
    fn test_multiple_glob_patterns() {
        let include = vec![
            "src/components/**/*".to_string(),
            "src/utils/**/*".to_string(),
        ];
        let mut sm = SimpleStateMachine::new(include);
        
        // Should match components
        let (state, should_print) = sm.run(&"src/components/Header.tsx(15,7): error TS2322".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print);
        
        // Should match utils
        let (state, should_print) = sm.run(&"src/utils/helpers.ts(23,3): error TS2322".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print);
        
        // Should not match features
        let (state, should_print) = sm.run(&"src/features/orders/index.ts(42,15): error TS2345".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(!should_print);
    }

    #[test]
    fn test_file_extension_filtering() {
        let include = vec!["**/*.tsx".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Should match .tsx files
        let (state, should_print) = sm.run(&"src/components/Header.tsx(15,7): error TS2322".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print);
        
        // Should not match .ts files
        let (state, should_print) = sm.run(&"src/utils/helpers.ts(23,3): error TS2322".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(!should_print);
    }

    #[test]
    fn test_line_without_parentheses() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Line without proper format should not crash
        let (state, should_print) = sm.run(&"src/components/Header.tsx: some error message".to_string());
        assert_eq!(*state, State::ParseToPause);
        // Should still process the path part before the colon
        assert!(should_print);
    }

    #[test]
    fn test_simple_state_machine_never_finishes() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // SimpleStateMachine should never finish
        assert!(!sm.is_finished());
        
        sm.run(&"src/components/Header.tsx(15,7): error TS2322".to_string());
        assert!(!sm.is_finished());
        
        sm.run(&"Found 1 error.".to_string());
        assert!(!sm.is_finished());
    }

    #[test]
    fn test_summary_lines() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Summary lines should be handled gracefully
        let (state, should_print) = sm.run(&"Found 4 errors.".to_string());
        assert_eq!(*state, State::ParseToPause);
        // Summary lines typically don't have parentheses, so they get the full line as path
        // and would not match our src/** pattern
        assert!(!should_print);
    }

    #[test]
    fn test_edge_cases_empty_lines() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Empty line
        let (state, should_print) = sm.run(&"".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(!should_print); // Empty string doesn't match src/**
    }

    #[test]
    fn test_malformed_typescript_output() {
        let include = vec!["src/**/*".to_string()];
        let mut sm = SimpleStateMachine::new(include);
        
        // Malformed line without parentheses
        let (state, should_print) = sm.run(&"some random text".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(!should_print);
        
        // Line with only opening parenthesis
        let (state, should_print) = sm.run(&"src/file.ts(".to_string());
        assert_eq!(*state, State::ParseToPause);
        assert!(should_print); // src/file.ts should match src/**
    }
}