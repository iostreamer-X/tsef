use tsef::path_matches;

#[cfg(test)]
mod path_matching_tests {
    use super::*;

    #[test]
    fn test_empty_include_list_matches_all() {
        let include = vec![];
        assert!(path_matches(&include, "src/components/Header.tsx"));
        assert!(path_matches(&include, "node_modules/@types/react/index.d.ts"));
        assert!(path_matches(&include, "any/path/file.ts"));
    }

    #[test]
    fn test_exact_file_path_matching() {
        let include = vec!["src/components/Header.tsx".to_string()];
        assert!(path_matches(&include, "src/components/Header.tsx"));
        assert!(!path_matches(&include, "src/components/Footer.tsx"));
        assert!(!path_matches(&include, "src/utils/helpers.ts"));
    }

    #[test]
    fn test_multiple_exact_paths() {
        let include = vec![
            "src/components/Header.tsx".to_string(),
            "src/utils/helpers.ts".to_string(),
        ];
        assert!(path_matches(&include, "src/components/Header.tsx"));
        assert!(path_matches(&include, "src/utils/helpers.ts"));
        assert!(!path_matches(&include, "src/features/orders/index.ts"));
        assert!(!path_matches(&include, "node_modules/@types/react/index.d.ts"));
    }

    #[test]
    fn test_glob_patterns_directory() {
        let include = vec!["src/components/**/*".to_string()];
        assert!(path_matches(&include, "src/components/Header.tsx"));
        assert!(path_matches(&include, "src/components/Footer.tsx"));
        assert!(path_matches(&include, "src/components/ui/Button.tsx"));
        assert!(path_matches(&include, "src/components/forms/LoginForm.tsx"));
        assert!(!path_matches(&include, "src/utils/helpers.ts"));
        assert!(!path_matches(&include, "src/features/orders/index.ts"));
    }

    #[test]
    fn test_glob_patterns_file_extension() {
        let include = vec!["**/*.tsx".to_string()];
        assert!(path_matches(&include, "src/components/Header.tsx"));
        assert!(path_matches(&include, "src/pages/HomePage.tsx"));
        assert!(path_matches(&include, "components/Button.tsx"));
        assert!(!path_matches(&include, "src/utils/helpers.ts"));
        assert!(!path_matches(&include, "src/features/orders/index.ts"));
    }

    #[test]
    fn test_multiple_glob_patterns() {
        let include = vec![
            "src/components/**/*".to_string(),
            "src/features/**/*".to_string(),
        ];
        assert!(path_matches(&include, "src/components/Header.tsx"));
        assert!(path_matches(&include, "src/features/orders/index.ts"));
        assert!(path_matches(&include, "src/features/users/UserService.ts"));
        assert!(!path_matches(&include, "src/utils/helpers.ts"));
        assert!(!path_matches(&include, "node_modules/@types/react/index.d.ts"));
    }

    #[test]
    fn test_root_level_glob() {
        let include = vec!["src/**/*".to_string()];
        assert!(path_matches(&include, "src/components/Header.tsx"));
        assert!(path_matches(&include, "src/utils/helpers.ts"));
        assert!(path_matches(&include, "src/features/orders/index.ts"));
        assert!(path_matches(&include, "src/index.ts"));
        assert!(!path_matches(&include, "node_modules/@types/react/index.d.ts"));
        assert!(!path_matches(&include, "tests/unit/test.ts"));
    }

    #[test]
    fn test_single_wildcard() {
        let include = vec!["src/components/*".to_string()];
        assert!(path_matches(&include, "src/components/Header.tsx"));
        assert!(path_matches(&include, "src/components/Footer.tsx"));
        // Note: The glob library's * wildcard DOES match nested paths
        assert!(path_matches(&include, "src/components/ui/Button.tsx"));
        assert!(!path_matches(&include, "src/utils/helpers.ts"));
    }

    #[test]
    fn test_question_mark_wildcard() {
        let include = vec!["src/components/Header.??x".to_string()];
        assert!(path_matches(&include, "src/components/Header.tsx"));
        assert!(path_matches(&include, "src/components/Header.jsx"));
        assert!(!path_matches(&include, "src/components/Header.ts"));
        assert!(!path_matches(&include, "src/components/Footer.tsx"));
    }

    #[test]
    fn test_complex_nested_patterns() {
        let include = vec![
            "src/**/components/**/*.tsx".to_string(),
            "src/**/utils/**/*.ts".to_string(),
        ];
        assert!(path_matches(&include, "src/components/Header.tsx"));
        assert!(path_matches(&include, "src/features/user/components/UserCard.tsx"));
        assert!(path_matches(&include, "src/utils/helpers.ts"));
        assert!(path_matches(&include, "src/shared/utils/formatting.ts"));
        assert!(!path_matches(&include, "src/components/Header.ts"));
        assert!(!path_matches(&include, "src/services/api.ts"));
    }

    #[test]
    fn test_case_sensitivity() {
        let include = vec!["SRC/components/**/*".to_string()];
        // Test case sensitivity - behavior depends on the underlying filesystem and glob implementation
        // On case-sensitive systems, this should not match
        assert!(!path_matches(&include, "src/components/Header.tsx"));
    }
}