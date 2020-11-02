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
