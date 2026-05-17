// A Rust fixture with varying complexity per function.
use std::collections::HashMap;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn classify(n: i32) -> &'static str {
    if n < 0 {
        "negative"
    } else if n == 0 {
        "zero"
    } else if n > 0 && n < 10 {
        "small positive"
    } else {
        "large positive"
    }
}

pub fn sum_evens(items: &[i32]) -> i32 {
    let mut total = 0;
    for &x in items {
        if x % 2 == 0 {
            total += x;
        }
    }
    total
}

// TODO: replace with something less silly
pub fn build_index(words: &[&str]) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for (i, w) in words.iter().enumerate() {
        map.insert((*w).to_string(), i);
    }
    map
}
