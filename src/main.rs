use ergo_headless_dapp_framework::Constant;

fn main() {
    let x = Constant::from(12 as i64);
    println!("{}", &x.try_into());
}