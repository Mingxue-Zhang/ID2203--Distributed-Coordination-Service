use serde::Deserialize;

/// 参数属性值
#[derive(Debug, Deserialize)]
pub struct Argument {
    // 环境
    pub env: String,
    // 端口
    pub port: u32,
    // 服务数据标识  1获取 2清洗 3融合
    pub flag: u8,
    // 同步数据开关 默认关
    pub sync: bool,
    // 同步数据主机端口
    pub sync_host: Vec<String>,
    // 同步线程数量 默认为1
    pub sync_thread_num: u8,
    // 参数属性配置文件路径
    pub file_path: String,
}

/// 声明参数属性默认值
impl Default for Argument {
    fn default() -> Self {
        Self {
            env: String::from("dev"),
            port: 8080,
            flag: 1,
            sync: false,
            sync_host: vec![],
            sync_thread_num: 1,
            file_path: String::new(),
        }
    }
}