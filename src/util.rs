pub fn get_database() -> String {
    let key = "LITTLE_LOOKUP_DATABASE";
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => String::from("postgres://docker:docker@localhost:5432/little-lookup"),
    }
}

pub fn get_pool_size_per_worker() -> u32 {
    let key = "LITTLE_LOOKUP_POOL_SIZE_PER_WORKER";
    match std::env::var(key) {
        Ok(val) => val.parse::<u32>().unwrap(),
        Err(_) => 5
    }
}

pub fn get_psk() -> String {
    let key = "LITTLE_LOOKUP_PSK";
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
