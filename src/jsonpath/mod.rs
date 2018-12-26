pub mod path_reader;
pub mod tokenizer;
pub mod parser;

mod utils {
    use std::result;

    pub fn vec_to_number(vec: &Vec<char>) -> result::Result<isize, String> {
        match vec.iter().map(|c| *c).collect::<String>().as_str().parse::<isize>() {
            Ok(n) => Ok(n),
            _ => Err(format!("vec_to_number: {:?}", vec))
        }
    }

    pub fn vec_to_string(vec: &Vec<char>) -> String {
        vec.iter().map(|c| *c).collect::<String>()
    }
}