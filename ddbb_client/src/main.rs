use std::env;

fn main() {
    let env_args = env::args().collect();
    let args = argrments::parse_args(env_args);
    if let Some(opt) = args {
        println!("我的参数 {:#?}", opt)
    }
}