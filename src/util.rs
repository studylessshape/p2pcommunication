pub fn compare_string(lhs: &String, rhs: &String) -> bool {
    let mut chars = lhs.chars();
    for c in rhs.chars() {
        if let Some(ch) = chars.next() {
            if ch != c {
                return false;
            }
        } else {
            return false;
        }
    }
    return true;
}