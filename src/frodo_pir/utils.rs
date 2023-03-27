use base64::{Engine as _, engine::{general_purpose}};
use rand_core::{OsRng, RngCore};

pub fn generate_db_elements(num_elements: usize, ele_byte_len: usize) -> Vec<String> {
    let mut elements = Vec::with_capacity(num_elements);
    for _ in 0..num_elements {
        let mut ele = vec![0u8; ele_byte_len];
        OsRng.fill_bytes(&mut ele);
        let ele_str = general_purpose::STANDARD.encode(ele);
        elements.push(ele_str);
    }
    elements
}
