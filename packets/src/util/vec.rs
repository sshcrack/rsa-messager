use std::collections::VecDeque;


pub fn vec_to_decque<T>(v: Vec<T>) -> VecDeque<T> {
    let mut r: VecDeque<T> = VecDeque::new();
    for i in v {
        r.push_back(i);
    }

    return r;
}

pub fn decque_to_vec<T>(v: VecDeque<T>) -> Vec<T> {
    let mut r: Vec<T> = Vec::new();
    for i in v {
        r.push(i);
    }

    return r;
}