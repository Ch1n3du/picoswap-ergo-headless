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