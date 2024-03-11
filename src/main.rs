use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug)]
struct Statistics {
    min: f64,
    max: f64,
    mean: f64,
    count: u64,
}

impl Statistics {
    fn new() -> Self {
        Self {
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            count: 0,
        }
    }
}

// parse a chunk, sum up all the entries in it. Return sum and count
// This function is called by multiple threads running in parallel.
fn parse_and_compute(chunk: Vec<u8>) -> FxHashMap<String, Statistics> {
    // Initialize intermediate data structures
    let mut station_temps: FxHashMap<&str, Vec<f64>> = FxHashMap::default();
    let content = String::from_utf8_lossy(&chunk);
    // Iterate over the rows in the file buffer and populate data structures
    content.split('\n').for_each(|row| {
        let split = row.split(';').collect::<Vec<_>>();
        if split.len() == 2 {
            let name = split[0];
            if !name.is_empty() {
                let measurement = split[1];
                let temperature = measurement.parse::<f64>().unwrap();
                if station_temps.contains_key(name) {
                    let temps: &mut Vec<f64> = station_temps.get_mut(name).unwrap();
                    temps.push(temperature);
                } else {
                    let new_vec: Vec<f64> = vec![temperature];
                    station_temps.insert(name, new_vec);
                }
            } else {
                println!("Empty...");
            }
        }
    });
    // Compute aggregations
    let mut results: FxHashMap<String, Statistics> = FxHashMap::default();
    station_temps.iter().for_each(|(name, temps)| {
        let mut sum_temp = 0.0;
        let mut min_temp = (*temps)[0];
        let mut max_temp = (*temps)[0];
        for temp in (*temps).iter() {
            sum_temp += temp;
            if temp < &min_temp {
                min_temp = *temp;
            }
            if temp > &max_temp {
                max_temp = *temp;
            }
        }
        let mean = sum_temp / temps.len() as f64;
        let mut statistics = Statistics::new();
        statistics.count = temps.len() as u64;
        statistics.min = min_temp;
        statistics.max = max_temp;
        statistics.mean = mean;
        results.insert((*name).to_owned(), statistics);
    });
    results
}

fn create_newline_separated_chunks(content: &str, chunk_size: usize) -> Vec<Vec<u8>> {
    let mut chunks = Vec::new();
    let mut leftover = String::new();
    let all_chunks = content.as_bytes().chunks(chunk_size);
    let num_chunks = all_chunks.len();
    for (idx, chunk) in all_chunks.enumerate() {
        // At every chunk's start or end, there can be a partial row.
        let chunk_string;
        if !leftover.is_empty() {
            // Leftover might have a newline at the beginning. Get rid of it.
            // Add leftover from previous iteration head of the current chunk
            chunk_string = leftover.trim_start().to_owned() + &String::from_utf8_lossy(chunk);
        } else {
            // No leftover from previous iteration. Just consider the chunk.
            chunk_string = String::from_utf8_lossy(chunk).to_string();
        }
        // If the chunk does not end in the newline, go back until we find one.
        match chunk_string.rfind('\n') {
            Some(newline_index) => {
                let (valid_chunk, _leftover) = chunk_string.split_at(newline_index);
                leftover = _leftover.to_string();
                if idx == num_chunks - 1 {
                    // last leftover. Just shove it into the chunk.
                    let last_chunk = valid_chunk.to_owned() + &leftover;
                    chunks.push(last_chunk.as_bytes().to_owned());
                } else {
                    chunks.push(valid_chunk.as_bytes().to_owned())
                }
            }
            None => {
                // No newline at all.
                chunks.push(chunk_string.as_bytes().to_owned())
            }
        }
    }
    chunks
}

#[cfg(test)]
mod tests {
    use crate::create_newline_separated_chunks;

    #[test]
    fn test_chunk_creation() {
        let content = "row_one\nrow_two\nrow_three\nrow_four\nrow_five";
        let mut expected = vec!["row_one", "row_two", "row_three", "row_four", "row_five"];
        expected.reverse();
        let chunks = create_newline_separated_chunks(content, 10);
        for chunk in chunks {
            let content = String::from_utf8(chunk).unwrap();
            assert_eq!(content, expected.pop().unwrap());
        }
        let mut bigger_expected = vec!["row_one", "row_two\nrow_three", "row_four", "row_five"];
        bigger_expected.reverse();
        let bigger_chunks = create_newline_separated_chunks(content, 13);
        for c in bigger_chunks {
            let content = String::from_utf8(c).unwrap();
            assert_eq!(content, bigger_expected.pop().unwrap());
        }
    }
}

/// this is the main function.
fn main() -> std::io::Result<()> {
    // Open file and read data into a buffer
    let mut file = File::open("measurements.100_mil.txt")?;
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents)?;
    let mut handles = vec![];

    // Divide up the input file into chunks
    let chunk_size = 32 * 1024 * 1024;
    let chunks = create_newline_separated_chunks(contents.as_str(), chunk_size);

    // Spawn threads
    let results = Arc::new(Mutex::new(FxHashMap::<String, Statistics>::default()));
    for chunk in chunks {
        let result_clone = Arc::clone(&results);
        let handle = thread::spawn(move || {
            let partial_agg = parse_and_compute(chunk);
            let mut updated_result = result_clone.lock().unwrap();
            // instead of naively updating results, we need to combine them per-key
            for (station, stats) in partial_agg {
                let mut updated_stats = Statistics::new();
                if let Some(old_stats) = updated_result.get(&station) {
                    updated_stats.min = f64::min(stats.min, old_stats.min);
                    updated_stats.max = f64::max(stats.max, old_stats.max);
                    updated_stats.count = old_stats.count + stats.count;
                    let old_sum = old_stats.mean * old_stats.count as f64;
                    let new_sum = stats.mean * stats.count as f64;
                    updated_stats.mean =
                        (old_sum + new_sum) / (old_stats.count + stats.count) as f64;
                    updated_result.insert(station, updated_stats);
                } else {
                    updated_result.insert(station, stats);
                }
            }
        });
        handles.push(handle);
    }
    println!("launched {} threads", handles.len());
    // Wait for all threads to finish processing
    for handle in handles {
        handle.join().unwrap();
    }
    let final_result = Arc::try_unwrap(results).unwrap().into_inner().unwrap();

    // Sort the results
    let mut stations: Vec<String> = Vec::new();
    for (_name, _) in final_result.iter() {
        stations.push(_name.to_owned());
    }
    stations.sort();

    // Print results
    print!("{{");
    let result_len = stations.len();
    for (_idx, name) in stations.iter().enumerate() {
        let stats = final_result.get(name).unwrap();
        print!(
            "{}={:.1}/{:.1}/{:.1}",
            name, stats.min, stats.mean, stats.max
        );
        if _idx < result_len - 1 {
            print!(", ");
        }
    }
    println!("}}");
    Ok(())
}
