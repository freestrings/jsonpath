use serde_json::Number;

pub fn to_f64(n: &Number) -> f64 {
    if n.is_i64() {
        n.as_i64().unwrap() as f64
    } else if n.is_f64() {
        n.as_f64().unwrap()
    } else {
        n.as_u64().unwrap() as f64
    }
}

pub fn abs_index(n: isize, len: usize) -> usize {
    if n < 0_isize {
        (n + len as isize).max(0) as usize
    } else {
        n.min(len as isize) as usize
    }
}

pub fn to_path_str<'a>(key: &'a str) -> (&'a str, Option<String>) {
    let key = if key.starts_with('\'') || key.starts_with('"') {
        let s = &key[1..key.len() - 1];
        if key.contains('\\') {
            (s, Some(s.chars().filter(|ch| ch != &'\\').collect()))
        } else {
            (s, None)
        }
    } else {
        (key, None)
    };
    key
}
