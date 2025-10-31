use quaternion::Quaternion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct QuaternionData<T> {
    pub re: T,
    pub i: T,
    pub j: T,
    pub k: T,
}

impl<T> From<Quaternion<T>> for QuaternionData<T> {
    fn from(value: Quaternion<T>) -> Self {
        let (re, [i, j, k]) = value;
        QuaternionData { re, i, j, k }
    }
}
