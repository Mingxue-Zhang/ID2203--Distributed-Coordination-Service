/// 参数属性值
pub mod argument;

use crate::argument::Argument;
use getopts::Options;
use serde_json;
use std::fs;

/// 程序版本号
const VERSION: &str = "0.1.0";

/// 定义命令行参数菜单
///
/// `optflag` 定义标志属性，不接收参数值，仅判断是否开启标记布尔值
/// `optopt` 定义接收参数属性，可接收属性后输入的值，一般用在变更属性值
/// `usage` 给定使用说明返回字符串进行命令行输出
fn meun() -> Options {
    let mut opts = Options::new();

    // 基本配置
    opts.optopt(
        "e",
        "env",
        "服务启动环境 根据程序内置配置文件后缀选择",
        "默认值为 dev",
    );
    opts.optopt(
        "p",
        "port",
        "服务端口号 数值范围：3000-9999",
        "默认值为 8080",
    );
    opts.optopt(
        "f",
        "flag",
        "服务数据标识  数值范围：1获取 2清洗 3融合",
        "默认值为 1",
    );
    // 同步参数
    opts.optflag("s", "sync", "同步数据开关默认不开启");
    opts.optopt(
        "S",
        "sync_host",
        "同步数据主机端口，连续多个要以英文逗号,进行分隔",
        "无默认输出主机，使用 \"-s\" 参数时必须填写",
    );
    opts.optopt(
        "T",
        "sync_thread_num",
        "同步线程数量 数值范围：1-12",
        "默认为1，使用 \"-s\" 参数时有效",
    );
    // 可获取参数数据的参数项
    opts.optopt(
        "",
        "file_path",
        "通过文件内容反序列化得到参数值\n运行程序如无指明参数默认后跟配置文件路径",
        "参数配置文件路径",
    );
    // 只进行标记不接收参数值的参数项
    opts.optflag("h", "help", "显示使用帮助信息");
    opts.optflag("v", "version", "输出程序版本");

    opts
}

/// 通过文件内容字符串反序列化
///
/// `fs::read_to_string` 读取文件内容给到 `serde_json::from_str` 进行反序列化
///
/// 需要文件内容格式与结构体保持一致
fn file_to_argument(file_path: &str) -> Option<Argument> {
    if let Ok(contents) = fs::read_to_string(file_path) {
        if let Ok(mut res) = serde_json::from_str::<Argument>(&contents) {
            res.file_path = file_path.to_string();
            Some(res)
        } else {
            eprintln!("参数属性配置文件内容有误： \n{}", contents);
            None
        }
    } else {
        eprintln!("找不到参数属性配置文件路径： \n{}", file_path);
        None
    }
}

/// 将命令行参数进行解析返回可选参数
///
/// `Argument` 为定义可选参数属性，可通过文件进行反序列化得到
///
/// **json文件内容**
/// ```json
/// {
///   "env": "prod",
///   "port": 9910,
///   "flag": 2,
///   "sync": true,
///   "sync_host": ["10.0.0.1:9920"],
///   "sync_thread_num": 3,
///   "file_path": "."
/// }
/// ```
pub fn parse_args(args: Vec<String>) -> Option<Argument> {
    // 先获取参数默认配置
    let mut argument = Argument::default();

    // 第一个参数是程序
    let program = &args[0];

    // 进行解析程序后参数
    let matches = match meun().parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            let brief = format!(
                "Error:\n {} \nUsage:\n{} [Options] file",
                f.to_string(),
                program
            );
            println!("{}", meun().usage(&brief));
            return None;
        }
    };

    // 使用帮助 -h
    if matches.opt_present("h") {
        let brief = format!("Usage:\n{} [Options] file", program);
        println!("{}", meun().usage(&brief));
        return None;
    }

    // 程序版本号 -v
    if matches.opt_present("v") {
        println!("{} v{}", program, VERSION);
        return None;
    }

    // 环境
    if matches.opt_present("env") {
        if let Some(opt) = matches.opt_str("env") {
            argument.env = opt;
        }
    }

    // 端口号
    if matches.opt_present("port") {
        if let Ok(res) = matches.opt_get::<u32>("port") {
            if let Some(opt) = res {
                argument.port = if opt <= 9999 && opt > 3000 {
                    opt as u32
                } else {
                    8080
                };
            }
        }
    }

    // 服务数据标识
    if matches.opt_present("flag") {
        if let Ok(res) = matches.opt_get::<u8>("flag") {
            if let Some(opt) = res {
                argument.flag = if opt <= 3 && opt > 1 { opt } else { 1 };
            }
        }
    }

    // 同步数据开关
    if matches.opt_present("sync") {
        argument.sync = true;
    }

    // 同步数据主机端口
    if matches.opt_present("sync_host") {
        if let Some(opt) = matches.opt_str("sync_host") {
            argument.sync_host = opt.split(',').map(|s| s.to_string()).collect();
        } else {
            eprintln!("host要以英文逗号,进行分隔");
            return None;
        }
    }

    // 同步数据线程数
    if matches.opt_present("sync_thread_num") {
        if let Ok(res) = matches.opt_get::<u8>("sync_thread_num") {
            if let Some(opt) = res {
                argument.sync_thread_num = if opt <= 12 && opt > 1 { 12 } else { 1 };
            }
        }
    }

    // 参数属性配置文件路径
    if matches.opt_present("file_path") {
        if let Some(opt) = matches.opt_str("file_path") {
            if let Some(file_argument) = file_to_argument(&opt) {
                argument = file_argument;
            } else {
                return None;
            }
        }
    }

    // 无指明参数默认后跟配置文件路径
    if !matches.free.is_empty() {
        if let Some(file_argument) = file_to_argument(&matches.free[0]) {
            argument = file_argument;
        } else {
            return None;
        }
    }

    Some(argument)
}