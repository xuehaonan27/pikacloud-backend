pub fn compare(cryp_s1: &String, cryp_s2: &Option<String>) -> bool {
    if cryp_s2.is_none() || cryp_s1 != cryp_s2.as_ref().unwrap() {
        false
    } else {
        true
    }
}

pub fn auth_hash(s: &String, seed: i32) -> String {
    todo!()
}