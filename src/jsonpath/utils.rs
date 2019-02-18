use std::result;

pub fn vec_to_int<F>(vec: &Vec<char>, msg_handler: F) -> result::Result<isize, String>
    where F: Fn() -> String {
    match vec.iter().map(|c| *c).collect::<String>().as_str().parse::<isize>() {
        Ok(n) => Ok(n),
        _ => Err(msg_handler())
    }
}

pub fn vec_to_float<F>(vec: &Vec<char>, msg_handler: F) -> result::Result<f64, String>
    where F: Fn() -> String {
    match vec.iter().map(|c| *c).collect::<String>().as_str().parse::<f64>() {
        Ok(n) => Ok(n),
        _ => Err(msg_handler())
    }
}

pub fn vec_to_string(vec: &Vec<char>) -> String {
    vec.iter().map(|c| *c).collect::<String>()
}