use thiserror::Error;

/*
* #error : 错误描述，当错误被触发时显示的消息
* #[from] : 错误属性的转换，捕获该错误的源
* #[error("failed with code: {0}")] ：动态错误，根据运行时生成的错误消息
* #[error(transparent)] ： 跨库和模块的错误处理，该错误仅作为其他错误的容器，错误消息从源错误中继承
*/
#[derive(Error, Debug)]
pub enum GrepError {
    #[error("Glob pattern error")]
    GlobPatternError(#[from] glob::PatternError),
    #[error("Regex pattern error")]
    GegexPatternError(#[from] regex::Error),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
}
