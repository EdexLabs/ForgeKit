#[cfg(test)]
mod tests {
    use forge_kit::metadata::{FunctionTrie, MetadataCache, MetadataManager};
    use forge_kit::types::{Event, Function};
    use std::collections::HashMap;
    use std::sync::Arc;

    /// Helper for test function creation.
    /// Assumes Function implements Default or has similar field names.
    fn create_test_function(name: &str) -> Function {
        Function {
            name: name.to_string(),
            description: "Test function".to_string(),
            args: Some(vec![]),
            brackets: Some(true),
            version: Some(serde_json::json!("1.0.0")),
            unwrap: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_metadata_manager_initialization() {
        let manager = MetadataManager::new();
        assert_eq!(manager.function_count(), 0);
        assert_eq!(manager.enum_count(), 0);
        assert_eq!(manager.event_count(), 0);
    }

    #[test]
    fn test_import_export_cycle() {
        let manager = MetadataManager::new();

        let func_name = "$test_func";
        let func = create_test_function(func_name);

        let mut enums = HashMap::new();
        enums.insert(
            "Colors".to_string(),
            vec!["Red".to_string(), "Blue".to_string()],
        );

        let event = Event {
            name: "onMessage".to_string(),
            description: "Test event".to_string(),
            fields: None,
        };

        let cache = MetadataCache::new(vec![func.clone()], enums.clone(), vec![event.clone()]);

        manager.import_cache(cache).expect("Import failed");

        // Verify Data Integrity
        assert!(manager.get_exact(func_name).is_some());
        assert_eq!(manager.get_enum("Colors").unwrap(), vec!["Red", "Blue"]);
        assert_eq!(manager.get_event("onMessage").unwrap().name, "onMessage");

        // Verify Round-trip
        let exported = manager.export_cache();
        assert_eq!(exported.functions.len(), 1);
        assert_eq!(exported.enums.get("Colors").unwrap().len(), 2);
    }

    #[test]
    fn test_metadata_source_persistence() {
        let manager = MetadataManager::new();
        let mut func = create_test_function("$test");
        func.extension = Some("test_ext".to_string());
        func.source_url = Some("http://example.com".to_string());
        func.local_path = Some(std::path::PathBuf::from("/tmp/test.js"));
        func.line = Some(42);

        let cache = MetadataCache::new(vec![func.clone()], HashMap::new(), vec![]);
        manager.import_cache(cache).expect("Import failed");

        let saved = manager.get_exact("$test").expect("Function not found");
        assert_eq!(saved.extension, Some("test_ext".to_string()));
        assert_eq!(saved.source_url, Some("http://example.com".to_string()));
        assert_eq!(
            saved.local_path,
            Some(std::path::PathBuf::from("/tmp/test.js"))
        );
        assert_eq!(saved.line, Some(42));

        let exported = manager.export_cache();
        let exported_func = &exported.functions[0];
        assert_eq!(exported_func.extension, Some("test_ext".to_string()));
        assert_eq!(
            exported_func.source_url,
            Some("http://example.com".to_string())
        );
        assert_eq!(
            exported_func.local_path,
            Some(std::path::PathBuf::from("/tmp/test.js"))
        );
        assert_eq!(exported_func.line, Some(42));
    }

    #[test]
    fn test_trie_case_insensitivity() {
        let mut trie = FunctionTrie::new();
        let func = Arc::new(create_test_function("$GetVar"));
        trie.insert("$GetVar", func);

        // Should find it regardless of casing
        assert!(trie.get_exact("$getvar").is_some());
        assert!(trie.get_exact("$GETVAR").is_some());
        assert!(trie.get_exact("$GetVar").is_some());
    }

    #[test]
    fn test_trie_completions_logic() {
        let mut trie = FunctionTrie::new();
        trie.insert("$add", Arc::new(create_test_function("$add")));
        trie.insert("$abs", Arc::new(create_test_function("$abs")));
        trie.insert(
            "$allProfiles",
            Arc::new(create_test_function("$allProfiles")),
        );

        let ab_prefix = trie.get_completions("$a");
        assert_eq!(ab_prefix.len(), 3);

        let abs_prefix = trie.get_completions("$ab");
        assert_eq!(abs_prefix.len(), 1);

        let non_existent = trie.get_completions("$z");
        assert!(non_existent.is_empty());
    }

    #[test]
    fn test_manager_clear() {
        let manager = MetadataManager::new();
        manager
            .import_cache(MetadataCache::new(
                vec![create_test_function("$test")],
                HashMap::new(),
                vec![],
            ))
            .unwrap();

        assert_eq!(manager.function_count(), 1);
        manager.clear();
        assert_eq!(manager.function_count(), 0);
    }
}
