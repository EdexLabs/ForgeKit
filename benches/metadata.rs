use criterion::{Criterion, black_box, criterion_group, criterion_main};
use forge_kit::metadata::{FunctionTrie, MetadataManager}; // Adjust crate name
use forge_kit::types::Function; // Adjust crate name
use std::sync::Arc;

/// Helper to generate a mock function
fn create_mock_function(name: &str) -> Function {
    Function {
        name: name.to_string(),
        aliases: Some(vec![format!("$alias_{}", name)]),
        extension: Some("core".to_string()),
        source_url: None,
        version: Some(serde_json::json!("1.0.0")),
        description: "Benchmark function".to_string(),
        brackets: Some(true),
        unwrap: true,
        args: Some(vec![]),
        output: None,
        category: Some("Utility".to_string()),
        experimental: Some(false),
        examples: Some(vec![]),
        deprecated: Some(false),
        local_path: None,
        line: None,
        extra: todo!(),
    }
}

fn bench_metadata_lookup(c: &mut Criterion) {
    let manager = MetadataManager::new();

    // Seed the manager with 500 functions to simulate a real environment
    let mut functions = Vec::new();
    for i in 0..500 {
        functions.push(create_mock_function(&format!("$function_{}", i)));
    }
    // Manually trigger the private add_functions logic via public-facing methods
    // or by importing a mock cache for speed.
    let cache = forge_kit::metadata::MetadataCache::new(functions, Default::default(), vec![]);
    manager.import_cache(cache).unwrap();

    // 1. Exact Match Lookup (Case-insensitive)
    c.bench_function("metadata_get_exact_hit", |b| {
        b.iter(|| manager.get_exact(black_box("$FUNCTION_250")))
    });

    c.bench_function("metadata_get_exact_miss", |b| {
        b.iter(|| manager.get_exact(black_box("$nonExistentFunction")))
    });

    // 2. Prefix Match (Used by the parser to find functions in strings)
    c.bench_function("metadata_get_prefix_match", |b| {
        // Simulates finding a function name at the start of a block of code
        b.iter(|| manager.get_prefix(black_box("$function_123[arg1;arg2]")))
    });

    // 3. Completions (IDE/Intellisense style)
    c.bench_function("metadata_get_completions_small", |b| {
        b.iter(|| manager.get_completions(black_box("$function_1")))
    });

    // 4. Heavy Concurrency Test
    // Measures DashMap and RwLock contention
    c.bench_function("metadata_concurrent_reads", |b| {
        b.iter(|| {
            std::thread::scope(|s| {
                for _ in 0..4 {
                    s.spawn(|| {
                        for i in 0..50 {
                            manager.get(&format!("$function_{}", i));
                        }
                    });
                }
            });
        })
    });
}

fn bench_trie_internals(c: &mut Criterion) {
    let func = Arc::new(create_mock_function("test"));

    // 1. Insertion performance
    c.bench_function("trie_insert", |b| {
        b.iter(|| {
            let mut t = FunctionTrie::new();
            t.insert(black_box("veryLongFunctionNameWithPrefix"), func.clone());
        })
    });

    // 2. Cold lookup (Large trie)
    let mut large_trie = FunctionTrie::new();
    for i in 0..1000 {
        large_trie.insert(&format!("$func_{}", i), func.clone());
    }

    c.bench_function("trie_lookup_1k_size", |b| {
        b.iter(|| large_trie.get_exact(black_box("$func_999")))
    });
}

fn bench_serialization(c: &mut Criterion) {
    let manager = MetadataManager::new();
    let mut functions = Vec::new();
    for i in 0..200 {
        functions.push(create_mock_function(&format!("$f_{}", i)));
    }
    manager
        .import_cache(forge_kit::metadata::MetadataCache::new(
            functions,
            Default::default(),
            vec![],
        ))
        .unwrap();

    c.bench_function("metadata_export_to_json", |b| {
        b.iter(|| manager.cache_to_json().unwrap())
    });
}

criterion_group!(
    benches,
    bench_metadata_lookup,
    bench_trie_internals,
    bench_serialization
);
criterion_main!(benches);
