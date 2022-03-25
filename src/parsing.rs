use ergo_headless_dapp_framework::*;

pub fn string_to_constant(stringy: String) -> Constant {
    stringy.as_bytes()
            .iter()
            .copied()
            .collect::<Vec<u8>>()
            .into()
}

// TODO
pub fn deserialize_constant_to_base_16_str(c: Constant) -> &'static str {
    todo!()
}

// TODO
pub fn deserialize_constant_to_base_58_str(c: Constant) -> &'static str {
    todo!()
}

// TODO
pub fn deserialize_constant_to_u64(c: Constant) -> u64 {
    todo!()
}