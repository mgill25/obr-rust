# obr-rust

This is the [One Billion Row challenge](https://github.com/gunnarmorling/1brc) in Rust

## Hardware Spec
- 2021 M1 Pro, 10 CPU/32 GB RAM

Using 100 million rows for quick measurements.

```bash
~/D/P/o/obr-rust ❯❯❯ wc -l measurements.100_mil.txt                                     
100000000 measurements.100_mil.txt
```

## Approach
Simply divide up the input file into multiple chunks and launch 1 thread-per-chunk. Each thread splits up the rows by newline, parses the rows and then computes aggregates in-memory. The partial aggregates are kept in a hashmap.

## Performance
The program is multi-threaded and performance varies according to the amount of threads launched, the amount of work each thread has to do, and the after work of combining together the partially aggregated results.

### Measurements (with various chunk sizes)
There is not a lot of variability between chunk sizes from 8MB to 128MB, but 16MB chunk size was the best performing.

#### 8MB
```bash
~/D/P/o/obr-rust ❯❯❯ hyperfine --warmup 2 --shell=none target/release/obr-rust                                             master ◼
Benchmark 1: target/release/obr-rust
  Time (mean ± σ):      2.426 s ±  0.027 s    [User: 12.931 s, System: 0.588 s]
  Range (min … max):    2.386 s …  2.475 s    10 runs
```
#### 16MB
```bash
~/D/P/o/obr-rust ❯❯❯ hyperfine --warmup 2 --shell=none target/release/obr-rust                                             master ◼
Benchmark 1: target/release/obr-rust
  Time (mean ± σ):      2.374 s ±  0.019 s    [User: 12.905 s, System: 0.561 s]
  Range (min … max):    2.343 s …  2.406 s    10 runs
```
#### 32MB
```bash
/D/P/o/obr-rust ❯❯❯ hyperfine --warmup 2 --shell=none target/release/obr-rust                                             master ◼
Benchmark 1: target/release/obr-rust
  Time (mean ± σ):      2.509 s ±  0.043 s    [User: 12.757 s, System: 0.624 s]
  Range (min … max):    2.441 s …  2.571 s    10 runs
```
#### 64MB 
```bash
~/D/P/o/obr-rust ❯❯❯ hyperfine --warmup 2 --shell=none target/release/obr-rust                                             master ◼
Benchmark 1: target/release/obr-rust
  Time (mean ± σ):      2.499 s ±  0.044 s    [User: 12.712 s, System: 0.614 s]
  Range (min … max):    2.464 s …  2.606 s    10 runs
```
#### 128MB
```bash
~/D/P/o/obr-rust ❯❯❯ hyperfine --warmup 2 --shell=none target/release/obr-rust                                             master ◼
Benchmark 1: target/release/obr-rust
  Time (mean ± σ):      2.527 s ±  0.040 s    [User: 12.607 s, System: 0.594 s]
  Range (min … max):    2.487 s …  2.588 s    10 runs
```

#### 256MB
```bash
~/D/P/o/obr-rust ❯❯❯ hyperfine --warmup 2 --shell=none target/release/obr-rust                                             master ◼
Benchmark 1: target/release/obr-rust
  Time (mean ± σ):      3.328 s ±  0.021 s    [User: 11.384 s, System: 0.541 s]
  Range (min … max):    3.303 s …  3.381 s    10 runs
```

#### 512MB
```bash
~/D/P/o/obr-rust ❯❯❯ hyperfine --warmup 2 --shell=none target/release/obr-rust                                             master ◼
Benchmark 1: target/release/obr-rust
  Time (mean ± σ):      5.514 s ±  0.033 s    [User: 11.384 s, System: 0.595 s]
  Range (min … max):    5.460 s …  5.560 s    10 runs
```

We see a reduction in performance as we increase chunk size. A very dramatic
slowdown as we go as high as 1GB chunks

#### 1024 MB (1 GB)
```bash
~/D/P/o/obr-rust ❯❯❯ hyperfine --warmup 2 --shell=none target/release/obr-rust                                             master ◼
Benchmark 1: target/release/obr-rust
  Time (mean ± σ):      9.645 s ±  0.057 s    [User: 11.199 s, System: 0.564 s]
  Range (min … max):    9.550 s …  9.733 s    10 runs
```
