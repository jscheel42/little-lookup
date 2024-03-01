use std::collections::HashMap;

pub fn get_database() -> String {
    let key = "LITTLE_LOOKUP_DATABASE";
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => String::from("postgres://docker:docker@localhost:5432/little-lookup"),
    }
}


pub fn get_namespace(query_options_map: &HashMap<String, String>) -> &str {
    match query_options_map.get("ns") {
        Some(namespace) => return namespace.as_str(),
        None => (),
    };

    match query_options_map.get("namespace") {
        Some(namespace) => namespace.as_str(),
        None => "default",
    }
}

pub fn get_pool_size_per_worker() -> u32 {
    let key = "LITTLE_LOOKUP_POOL_SIZE_PER_WORKER";
    match std::env::var(key) {
        Ok(val) => val.parse::<u32>().unwrap(),
        Err(_) => 5
    }
}

pub enum PSKType {
    READ,
    WRITE
} 

pub fn get_psk(psk_type: PSKType) -> String {
    let key = match psk_type {
        PSKType::READ  => "LITTLE_LOOKUP_PSK_READ",
        PSKType::WRITE => "LITTLE_LOOKUP_PSK_WRITE"
    };

    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => String::from("")
    }
}

pub fn get_worker_num() -> usize {
    let key = "LITTLE_LOOKUP_WORKER_NUM";
    match std::env::var(key) {
        Ok(val) => val.parse::<usize>().unwrap(),
        Err(_) => 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_database() {
        // Test when the environment variable is set
        std::env::set_var("LITTLE_LOOKUP_DATABASE", "test_database");
        assert_eq!(get_database(), "test_database");

        // Test when the environment variable is not set
        std::env::remove_var("LITTLE_LOOKUP_DATABASE");
        assert_eq!(
            get_database(),
            "postgres://docker:docker@localhost:5432/little-lookup"
        );
    }

    #[test]
    fn test_get_namespace() {
        let mut query_options_map = HashMap::new();

        // Test when "ns" key is present
        query_options_map.insert(String::from("ns"), String::from("test_namespace"));
        assert_eq!(get_namespace(&query_options_map), "test_namespace");

        // Test when "namespace" key is present
        query_options_map.remove("ns");
        query_options_map.insert(String::from("namespace"), String::from("test_namespace"));
        assert_eq!(get_namespace(&query_options_map), "test_namespace");

        // Test when neither "ns" nor "namespace" key is present
        query_options_map.remove("namespace");
        assert_eq!(get_namespace(&query_options_map), "default");
    }

    #[test]
    fn test_get_pool_size_per_worker() {
        // Test when the environment variable is set
        std::env::set_var("LITTLE_LOOKUP_POOL_SIZE_PER_WORKER", "10");
        assert_eq!(get_pool_size_per_worker(), 10);

        // Test when the environment variable is not set
        std::env::remove_var("LITTLE_LOOKUP_POOL_SIZE_PER_WORKER");
        assert_eq!(get_pool_size_per_worker(), 5);
    }

    #[test]
    fn test_get_psk() {
        // Test when the environment variable is set
        std::env::set_var("LITTLE_LOOKUP_PSK_READ", "test_read_psk");
        assert_eq!(get_psk(PSKType::READ), "test_read_psk");

        std::env::set_var("LITTLE_LOOKUP_PSK_WRITE", "test_write_psk");
        assert_eq!(get_psk(PSKType::WRITE), "test_write_psk");

        // Test when the environment variable is not set
        std::env::remove_var("LITTLE_LOOKUP_PSK_READ");
        assert_eq!(get_psk(PSKType::READ), "");

        std::env::remove_var("LITTLE_LOOKUP_PSK_WRITE");
        assert_eq!(get_psk(PSKType::WRITE), "");
    }

    #[test]
    fn test_get_worker_num() {
        // Test when the environment variable is set
        std::env::set_var("LITTLE_LOOKUP_WORKER_NUM", "4");
        assert_eq!(get_worker_num(), 4);

        // Test when the environment variable is not set
        std::env::remove_var("LITTLE_LOOKUP_WORKER_NUM");
        assert_eq!(get_worker_num(), 2);
    }
}
