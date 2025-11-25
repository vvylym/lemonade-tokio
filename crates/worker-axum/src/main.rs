fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_return_none() {
        let res = main();
        assert_eq!(res, ())
    }
}