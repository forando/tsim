use regex::Regex;
use std::collections::{HashMap, HashSet};
use indicatif::ProgressBar;
use crate::threadpool::ThreadPool;
use std::sync::{mpsc, Arc};
use std::thread;
use std::sync::mpsc::Sender;
use crate::utils::{FloatWrapper, Similarities};

const PRECISION: usize = 3;

pub fn run(records: HashMap<String, String>) -> Similarities {
    let records = map_to_vec(records);
    vec_to_similarities(records)
}

fn vec_to_similarities(records: Vec<HashSet<String>>) -> Similarities {
    let records_length = records.len();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move|| run_similarities(records, tx));

    let total_combinations = calculate_combinations(records_length);
    let pb = ProgressBar::new(total_combinations as u64);
    let mut all_similarities: Vec<f32> = Vec::with_capacity(total_combinations);
    let mut total = 0.0;
    let mut min = 0.0;
    let mut max = 0.0;
    for similarities in rx {
        let len = similarities.len() as u64;
        for similarity in similarities {
            total += similarity;
            if similarity < min {
                min = similarity;
            } else if similarity > max {
                max = similarity;
            }
            all_similarities.push(similarity);
        }
        pb.inc(len);
    }
    let avg = total / all_similarities.len() as f32;
    let avg = FloatWrapper::new(avg).rnd_decimals(PRECISION as u32);
    Similarities::new(all_similarities, avg, min, max, PRECISION)
}

fn run_similarities(records: Vec<HashSet<String>>, tx: Sender<Vec<f32>>) {
    let records_length = records.len();

    if records_length == 0 {
        return;
    }

    let records = Arc::new(records);
    let logical_cpus = num_cpus::get();
    let pool = ThreadPool::new(logical_cpus);

    for i in 0..(records_length - 1) {
        let tx = mpsc::Sender::clone(&tx);

        let records = Arc::clone(&records);
        pool.execute(move||{
            let next = i + 1;
            let mut similarities: Vec<f32> = Vec::with_capacity(records_length - i);
            for j in next..records_length {
                let i_text = &records[i];
                let j_text = &records[j];
                let sim = get_similarity(i_text, j_text);
                similarities.push(sim);
            }
            tx.send(similarities).unwrap();
        });
    }
}

fn map_to_vec(records: HashMap<String, String>) -> Vec<HashSet<String>> {
    let mut lines: Vec<HashSet<String>> = Vec::with_capacity(records.len());
    let (tx, rx) = mpsc::channel();
    thread::spawn(move||text_to_set(records, tx));
    for received in rx {
        lines.push(received);
    }
    lines
}

fn text_to_set(records: HashMap<String, String>, tx: Sender<HashSet<String>>) {
    let logical_cpus = num_cpus::get();
    let pool = ThreadPool::new(logical_cpus);

    for (_, content) in records {
        let tx = mpsc::Sender::clone(&tx);
        pool.execute(move|| {
            let line = get_stripped_string(&content);
            tx.send(line).unwrap();
        });
    }
}

fn get_stripped_string(s1: &str) -> HashSet<String> {
    let re = Regex::new(r"['|`|’|.|,|?|!|:|;]").unwrap();
    let formatted_str = re.replace_all(s1, " ");

    let split = formatted_str.split_whitespace();
    let mut set: HashSet<String> = HashSet::new();

    for line in split {
        set.insert(line.parse().unwrap());
    }
    return  set;
}

fn get_similarity(arr1: &HashSet<String>, arr2: &HashSet<String>) -> f32 {
    let intersect = arr1.intersection(arr2).count() as f32;
    let a = arr1.len() as f32;
    let b = arr2.len() as f32;
    let similarity = intersect / (a + b - intersect);
    FloatWrapper::new(similarity).rnd_decimals(PRECISION as u32)
}

fn calculate_combinations(num: usize) -> usize {
    num * (num - 1) / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_split_string() {
        let res = get_stripped_string("Hello. I’m Andrew; and, who're you?you");
        assert_eq!(8, res.len(), "get_stripped_string('Hello. I’m Andrew; and, who're you?you')");
    }

    #[test]
    fn should_calculate_similarity() {
        let vec1: Vec<String> = vec![String::from("a"), String::from("b"), String::from("c")];
        let vec2: Vec<String> = vec![String::from("b"), String::from("c"), String::from("d")];

        let arr1: HashSet<String> = vec1.into_iter().collect();
        let arr2: HashSet<String> = vec2.into_iter().collect();

        let res = get_similarity(&arr1, &arr2);

        assert_eq!(0.5, res, "get_similarity([a,b,c], [b,c,d])");
    }

    #[test]
    fn should_calculate_total_combinations() {
        let combinations = calculate_combinations(5);
        assert_eq!(10, combinations, "calculate_combinations(3)");
    }

    #[test]
    fn should_run_similarities() {
        let (tx, rx) = mpsc::channel();

        let record1: Vec<String> = vec![String::from("a"), String::from("b"), String::from("c")];
        let record2: Vec<String> = vec![String::from("b"), String::from("c"), String::from("d")];

        let record1: HashSet<String> = record1.into_iter().collect();
        let record2: HashSet<String> = record2.into_iter().collect();

        let records: Vec<HashSet<String>> = vec![record1, record2];

        thread::spawn(move|| run_similarities(records, tx));

        let mut i = 0;
        for _ in rx {
            i += 1;
        }

        assert_eq!(1, i);
    }

    #[test]
    fn should_not_run_similarities_if_exactly_one_record() {
        let (tx, rx) = mpsc::channel();

        let record1: Vec<String> = vec![String::from("a"), String::from("b"), String::from("c")];

        let record1: HashSet<String> = record1.into_iter().collect();

        let records: Vec<HashSet<String>> = vec![record1];

        thread::spawn(move|| run_similarities(records, tx));

        let mut i = 0;
        for _ in rx {
            i += 1;
        }

        assert_eq!(0, i);
    }

    #[test]
    fn should_not_run_similarities_if_zero_records() {
        let (tx, rx) = mpsc::channel();

        let records: Vec<HashSet<String>> = vec![];

        thread::spawn(move|| run_similarities(records, tx));

        let mut i = 0;
        for _ in rx {
            i += 1;
        }

        assert_eq!(0, i);
    }
}