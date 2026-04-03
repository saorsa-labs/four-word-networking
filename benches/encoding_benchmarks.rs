use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use four_word_networking::*;
// use std::net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

fn bench_ipv4_encoding(c: &mut Criterion) {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();
    let test_addresses = vec![
        "192.168.1.1:443",
        "10.0.0.1:22",
        "127.0.0.1:8080",
        "8.8.8.8:53",
        "172.16.1.1:80",
    ];

    c.bench_function("ipv4_encode", |b| {
        b.iter(|| {
            for addr in &test_addresses {
                let _ = encoder.encode(black_box(addr));
            }
        })
    });
}

fn bench_ipv6_encoding(c: &mut Criterion) {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();
    let test_addresses = vec![
        "[::1]:443",
        "[fe80::1]:22",
        "[2001:db8::1]:443",
        "[::]:80",
        "[fe80::abc:def:1234:5678]:8080",
    ];

    c.bench_function("ipv6_encode", |b| {
        b.iter(|| {
            for addr in &test_addresses {
                let _ = encoder.encode(black_box(addr));
            }
        })
    });
}

fn bench_decoding(c: &mut Criterion) {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();
    let test_words = vec![
        "paper.broaden.smith.bully",
        "game.weather.july.general",
        "Bully-Book-Book-Book",
        "Ship-July-Book-Book",
    ];

    c.bench_function("decode_words", |b| {
        b.iter(|| {
            for words in &test_words {
                let _ = encoder.decode(black_box(words));
            }
        })
    });
}

fn bench_round_trip(c: &mut Criterion) {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();
    let test_addresses = vec![
        "192.168.1.1:443",
        "[::1]:443",
        "10.0.0.1:22",
        "[fe80::1]:22",
    ];

    c.bench_function("round_trip", |b| {
        b.iter(|| {
            for addr in &test_addresses {
                if let Ok(words) = encoder.encode(black_box(addr)) {
                    let _ = encoder.decode(black_box(&words));
                }
            }
        })
    });
}

// Additional comprehensive benchmarks

// Dictionary benchmarks disabled - FourWordDictionary no longer exists
// TODO: Update to use Dictionary4K when needed
/*
fn bench_dictionary_performance(c: &mut Criterion) {
    let dict = FourWordDictionary::new().unwrap();

    c.bench_function("dictionary_word_lookup", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let _ = dict.get_word(black_box(i % 16384));
            }
        })
    });

    c.bench_function("dictionary_index_lookup", |b| {
        b.iter(|| {
            let words = ["apple", "orange", "banana", "grape", "cherry"];
            for word in words {
                let _ = dict.find_word(black_box(word));
            }
        })
    });
}
*/

fn bench_compression_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");

    // Test different data sizes
    for size in [32, 64, 128, 256, 512, 1024].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("compress", size), size, |b, &size| {
            let data = vec![42u8; size];
            b.iter(|| {
                let _ = test_compression(black_box(&data));
            });
        });
    }

    group.finish();
}

fn bench_batch_processing(c: &mut Criterion) {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();
    let mut group = c.benchmark_group("batch_processing");

    // Generate test data
    let ipv4_addresses: Vec<String> = (0..1000)
        .map(|i| format!("192.168.{}.{}", i / 256, i % 256))
        .collect();

    group.throughput(Throughput::Elements(ipv4_addresses.len() as u64));
    group.bench_function("batch_encode_ipv4", |b| {
        b.iter(|| {
            for addr in &ipv4_addresses {
                let _ = encoder.encode(black_box(addr));
            }
        })
    });

    // Pre-encode for decoding test
    let encoded_addresses: Vec<String> = ipv4_addresses
        .iter()
        .filter_map(|addr| encoder.encode(addr).ok())
        .collect();

    group.throughput(Throughput::Elements(encoded_addresses.len() as u64));
    group.bench_function("batch_decode_words", |b| {
        b.iter(|| {
            for words in &encoded_addresses {
                let _ = encoder.decode(black_box(words));
            }
        })
    });

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    group.bench_function("encoder_creation", |b| {
        b.iter(|| {
            let _ = FourWordAdaptiveEncoder::new();
        })
    });

    // Dictionary creation benchmark disabled - FourWordDictionary no longer exists
    // TODO: Update to use Dictionary4K when needed
    /*group.bench_function("dictionary_creation", |b| {
        b.iter(|| {
            let _ = FourWordDictionary::new();
        })
    });*/

    group.finish();
}

fn bench_edge_cases(c: &mut Criterion) {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();
    let mut group = c.benchmark_group("edge_cases");

    let edge_cases = vec![
        "0.0.0.0:0",
        "255.255.255.255:65535",
        "127.0.0.1:8080",
        "[::]:0",
        "[::1]:443",
        "[ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff]:65535",
    ];

    group.bench_function("edge_case_encoding", |b| {
        b.iter(|| {
            for addr in &edge_cases {
                let _ = encoder.encode(black_box(addr));
            }
        })
    });

    group.finish();
}

fn bench_concurrent_access(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let encoder = Arc::new(FourWordAdaptiveEncoder::new().unwrap());
    let mut group = c.benchmark_group("concurrent_access");

    group.bench_function("concurrent_encoding", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..4)
                .map(|i| {
                    let encoder = Arc::clone(&encoder);
                    thread::spawn(move || {
                        let addr = format!("192.168.1.{i}");
                        let _ = encoder.encode(black_box(&addr));
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });

    group.finish();
}

fn bench_scalability(c: &mut Criterion) {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();
    let mut group = c.benchmark_group("scalability");

    // Test with different input sizes
    for count in [10, 100, 1000, 10000].iter() {
        let addresses: Vec<String> = (0..*count)
            .map(|i| format!("10.0.{}.{}", i / 256, i % 256))
            .collect();

        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("encode_scale", count),
            &addresses,
            |b, addresses| {
                b.iter(|| {
                    for addr in addresses {
                        let _ = encoder.encode(black_box(addr));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_real_world_patterns(c: &mut Criterion) {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();
    let mut group = c.benchmark_group("real_world");

    // Common real-world addresses
    let real_world_addresses = vec![
        "8.8.8.8:53",                // Google DNS
        "1.1.1.1:53",                // Cloudflare DNS
        "208.67.222.222:53",         // OpenDNS
        "192.168.1.1:80",            // Common router
        "10.0.0.1:22",               // Private network SSH
        "172.16.0.1:443",            // Private network HTTPS
        "[2001:4860:4860::8888]:53", // Google IPv6 DNS
        "[2606:4700:4700::1111]:53", // Cloudflare IPv6 DNS
        "127.0.0.1:8080",            // Local development
        "[::1]:3000",                // Local development IPv6
    ];

    group.bench_function("real_world_encode", |b| {
        b.iter(|| {
            for addr in &real_world_addresses {
                let _ = encoder.encode(black_box(addr));
            }
        })
    });

    group.finish();
}

fn bench_error_handling(c: &mut Criterion) {
    let encoder = FourWordAdaptiveEncoder::new().unwrap();
    let mut group = c.benchmark_group("error_handling");

    let invalid_inputs = vec![
        "invalid.address",
        "999.999.999.999:80",
        "192.168.1.1:99999",
        "not-an-ip",
        "",
        "::gg:1",
        "one.two.three",
        "invalid.word.combination.here",
    ];

    group.bench_function("error_handling", |b| {
        b.iter(|| {
            for input in &invalid_inputs {
                let _ = encoder.encode(black_box(input));
                let _ = encoder.decode(black_box(input));
            }
        })
    });

    group.finish();
}

// Helper function for compression benchmark
fn test_compression(data: &[u8]) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Simple compression test - replace with actual compression logic
    Ok(data.to_vec())
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100)
        .confidence_level(0.95)
        .noise_threshold(0.05);
    targets =
        bench_ipv4_encoding,
        bench_ipv6_encoding,
        bench_decoding,
        bench_round_trip,
        // bench_dictionary_performance, // Disabled - FourWordDictionary no longer exists
        bench_compression_performance,
        bench_batch_processing,
        bench_memory_usage,
        bench_edge_cases,
        bench_concurrent_access,
        bench_scalability,
        bench_real_world_patterns,
        bench_error_handling
);
criterion_main!(benches);
