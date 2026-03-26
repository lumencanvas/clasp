//! Benchmarks for DefraStateStore in-memory cache operations.
//!
//! These benchmarks test the hot path (DashMap cache) WITHOUT DefraDB.
//! The cache must stay under 100us for SET/GET to meet CLASP latency targets.

use clasp_core::state::ParamState;
use clasp_core::Value;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use dashmap::DashMap;
use std::sync::Arc;

fn bench_cache_get(c: &mut Criterion) {
    let cache: Arc<DashMap<String, ParamState>> = Arc::new(DashMap::new());

    // Pre-populate with 1000 params
    for i in 0..1000 {
        let addr = format!("/bench/param/{}", i);
        cache.insert(
            addr,
            ParamState::new(Value::Float(i as f64), "bench".into()),
        );
    }

    c.bench_function("cache_get_hit", |b| {
        b.iter(|| {
            let val = cache.get("/bench/param/500");
            criterion::black_box(val);
        })
    });

    c.bench_function("cache_get_miss", |b| {
        b.iter(|| {
            let val = cache.get("/bench/param/nonexistent");
            criterion::black_box(val);
        })
    });
}

fn bench_cache_set(c: &mut Criterion) {
    let cache: Arc<DashMap<String, ParamState>> = Arc::new(DashMap::new());

    c.bench_function("cache_set_new", |b| {
        let mut i = 0u64;
        b.iter(|| {
            let addr = format!("/bench/set/{}", i);
            cache.insert(
                addr,
                ParamState::new(Value::Float(i as f64), "bench".into()),
            );
            i += 1;
        })
    });

    // Pre-populate for update benchmark
    let cache2: Arc<DashMap<String, ParamState>> = Arc::new(DashMap::new());
    for i in 0..1000 {
        let addr = format!("/bench/update/{}", i);
        cache2.insert(addr, ParamState::new(Value::Float(0.0), "bench".into()));
    }

    c.bench_function("cache_set_update", |b| {
        let mut rev = 1u64;
        b.iter(|| {
            if let Some(mut entry) = cache2.get_mut("/bench/update/500") {
                entry.value = Value::Float(rev as f64);
                entry.revision = rev;
            }
            rev += 1;
        })
    });
}

fn bench_cache_pattern_match(c: &mut Criterion) {
    let cache: Arc<DashMap<String, ParamState>> = Arc::new(DashMap::new());

    // 1000 params across 10 namespaces
    for ns in 0..10 {
        for i in 0..100 {
            let addr = format!("/bench/ns{}/param/{}", ns, i);
            cache.insert(
                addr,
                ParamState::new(Value::Float(i as f64), "bench".into()),
            );
        }
    }

    let mut group = c.benchmark_group("cache_pattern_match");

    group.bench_function("single_namespace", |b| {
        b.iter(|| {
            let matches: Vec<_> = cache
                .iter()
                .filter(|entry| entry.key().starts_with("/bench/ns5/"))
                .map(|entry| entry.key().clone())
                .collect();
            criterion::black_box(matches);
        })
    });

    group.bench_function("all_params", |b| {
        b.iter(|| {
            let matches: Vec<_> = cache.iter().map(|entry| entry.key().clone()).collect();
            criterion::black_box(matches);
        })
    });

    group.finish();
}

fn bench_cache_concurrent(c: &mut Criterion) {
    let cache: Arc<DashMap<String, ParamState>> = Arc::new(DashMap::new());

    // Pre-populate
    for i in 0..1000 {
        let addr = format!("/bench/concurrent/{}", i);
        cache.insert(
            addr,
            ParamState::new(Value::Float(i as f64), "bench".into()),
        );
    }

    c.bench_function("concurrent_read_write", |b| {
        let mut i = 0u64;
        b.iter(|| {
            // Interleave reads and writes
            let read_addr = format!("/bench/concurrent/{}", i % 1000);
            let write_addr = format!("/bench/concurrent/{}", (i + 500) % 1000);

            let _ = cache.get(&read_addr);
            if let Some(mut entry) = cache.get_mut(&write_addr) {
                entry.value = Value::Float(i as f64);
            }
            i += 1;
        })
    });
}

fn bench_cache_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_scaling");

    for size in [100, 1000, 10000, 100000].iter() {
        let cache: DashMap<String, ParamState> = DashMap::new();
        for i in 0..*size {
            let addr = format!("/bench/scale/{}", i);
            cache.insert(
                addr,
                ParamState::new(Value::Float(i as f64), "bench".into()),
            );
        }

        group.bench_with_input(BenchmarkId::new("get", size), size, |b, &size| {
            let addr = format!("/bench/scale/{}", size / 2);
            b.iter(|| {
                let val = cache.get(&addr);
                criterion::black_box(val);
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_get,
    bench_cache_set,
    bench_cache_pattern_match,
    bench_cache_concurrent,
    bench_cache_scaling,
);
criterion_main!(benches);
