use std::env;

fn _load_env<T, F>(key: &str, transform: F, default_value: Option<T>) -> T
where
    F: FnOnce(String) -> T,
{
    match env::var(key) {
        Ok(value) => transform(value),
        Err(_) => match default_value {
            Some(value) => value,
            None => panic!("Missing environment variable: {}", key),
        },
    }
}

pub fn load_env_panic(key: &str) -> String {
    env::var(key).expect(&format!("{} must be set", key))
}

pub fn load_env_optional(key: &str) -> Option<String> {
    env::var(key).ok()
}
