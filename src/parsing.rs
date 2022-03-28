use ergo_headless_dapp_framework::*;
use ergo_lib::*;
use ergotree_ir::mir::constant::Literal;

pub fn string_to_constant(stringy: String) -> Constant {
    stringy.as_bytes()
            .iter()
            .copied()
            .collect::<Vec<u8>>()
            .into()
}

// TODO
// Takes a Coll[Byte] and get
pub fn deserialize_constant_to_base_16_str(c: Constant) -> Option<&'static str> {
    let coll_of_bytes = SType::SColl(Box::new(SType::SByte));

    match c.tpe {
        coll_of_bytes => (),
        _ => return None
    }

    let x = c.try_into().unwrap();
    
    Some("nglj")
}

// TODO
pub fn deserialize_constant_to_base_58_str(c: Constant) -> &'static str {
    todo!()
}

// TODO
pub fn deserialize_constant_to_u64(c: Constant) -> u64 {
    todo!()
}