fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_variables() {
        let test_address = std::env::var("TEST_WORKER_ADDRESS").unwrap();
        assert_eq!(test_address, "127.0.0.1:0");
    }

    #[test]
    fn test_main_return_none() {
        assert_eq!(main(), ())
    }
}
