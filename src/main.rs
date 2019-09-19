fn main() {
    println!("Hello, world!");
}

#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;

fn str_strip_numbers(s: &str) -> Vec<i64> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\d+").unwrap();
    }
    // iterate over all matches
    RE.find_iter(s)
        .filter_map(|digits| digits.as_str().parse().ok())  // Filter out numbers out of ixx range.
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stripping_numbers() {
        assert_eq!(str_strip_numbers("ab11cdd2k.144"), vec![11, 2, 144]);
        assert!(str_strip_numbers("Ignacio Vazquez-Abrams").is_empty());
    }
}
