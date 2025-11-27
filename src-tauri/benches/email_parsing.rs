// P3 Performance Benchmark: Email Parsing
//
// This benchmark measures the performance of email header decoding and parsing operations
// Target: Decode 1000 email headers in < 5 seconds (< 5ms per header)

use colimail_lib::commands::emails::codec::decode_header;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn generate_test_headers(count: usize) -> Vec<String> {
    let mut headers = Vec::with_capacity(count);

    // Mix of different encoding types and charsets
    let test_cases = [
        // Plain ASCII
        "Simple Subject",
        // UTF-8 Base64
        "=?UTF-8?B?5rWL6K+V6YKu5Lu2?=", // "测试邮件" in Chinese
        // UTF-8 Quoted-Printable
        "=?UTF-8?Q?Test_=E2=9C=85_Email?=", // "Test ✅ Email"
        // GB2312 encoding (common in Chinese emails)
        "=?GB2312?B?1tC5+rTzvfg=?=", // Chinese text
        // Multiple encoded words
        "=?UTF-8?B?5rWL6K+V?= =?UTF-8?B?6YKu5Lu2?=",
        // Mixed plain and encoded
        "Re: =?UTF-8?Q?Important_Meeting?=",
        // Japanese
        "=?ISO-2022-JP?B?GyRCJDMkcyRLJEEkTxsoQg==?=", // "こんにちは"
        // Korean
        "=?EUC-KR?B?vsiz88fPt7E=?=", // Korean text
        // Cyrillic
        "=?KOI8-R?Q?=F0=D2=C9=D7=C5=D4?=", // "привет" in Russian
        // Long subject with multiple encoded words
        "=?UTF-8?B?5rWL6K+V?= =?UTF-8?B?6YKu5Lu2?= =?UTF-8?B?5L2/55So?= =?UTF-8?B?5aSa6YeN?=",
    ];

    for i in 0..count {
        let header = test_cases[i % test_cases.len()];
        headers.push(header.to_string());
    }

    headers
}

fn benchmark_decode_single_header(c: &mut Criterion) {
    let test_cases = [
        ("plain_ascii", "Simple Subject Line"),
        ("utf8_base64", "=?UTF-8?B?5rWL6K+V6YKu5Lu2?="),
        ("utf8_quoted", "=?UTF-8?Q?Test_=E2=9C=85_Email?="),
        ("gb2312", "=?GB2312?B?1tC5+rTzvfg=?="),
        (
            "multiple_words",
            "=?UTF-8?B?5rWL6K+V?= =?UTF-8?B?6YKu5Lu2?=",
        ),
    ];

    let mut group = c.benchmark_group("decode_single_header");

    for (name, header) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &header, |b, &h| {
            b.iter(|| decode_header(black_box(h)))
        });
    }

    group.finish();
}

fn benchmark_decode_batch_headers(c: &mut Criterion) {
    let sizes = [10, 100, 1000, 10000, 100000, 1000000, 10000000];

    let mut group = c.benchmark_group("decode_batch_headers");

    for size in sizes {
        let headers = generate_test_headers(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &headers, |b, h| {
            b.iter(|| {
                for header in h {
                    let _ = decode_header(black_box(header));
                }
            })
        });
    }

    group.finish();
}

fn benchmark_decode_1000_headers(c: &mut Criterion) {
    // P3 Performance Target: 1000 headers in < 5 seconds
    // Testing large volumes: 1M and 10M emails
    let mut group = c.benchmark_group("decode_large_volumes");

    // 1 million emails
    let headers_1m = generate_test_headers(1_000_000);
    group.bench_function("decode_1M_headers", |b| {
        b.iter(|| {
            for header in &headers_1m {
                let _ = decode_header(black_box(header));
            }
        })
    });

    // 10 million emails
    let headers_10m = generate_test_headers(10_000_000);
    group.bench_function("decode_10M_headers", |b| {
        b.iter(|| {
            for header in &headers_10m {
                let _ = decode_header(black_box(header));
            }
        })
    });

    group.finish();
}

fn benchmark_complex_encoding_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_encoding");

    // Very long subject with many encoded words
    let long_subject = "=?UTF-8?B?5rWL6K+V?= ".repeat(50);

    group.bench_function("long_multi_encoded", |b| {
        b.iter(|| decode_header(black_box(&long_subject)))
    });

    // Nested/malformed encoding
    let malformed = "=?UTF-8?B?Invalid Base64 $$$ Here?=";
    group.bench_function("malformed_encoding", |b| {
        b.iter(|| decode_header(black_box(malformed)))
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_decode_single_header,
    benchmark_decode_batch_headers,
    benchmark_decode_1000_headers,
    benchmark_complex_encoding_scenarios
);

criterion_main!(benches);
