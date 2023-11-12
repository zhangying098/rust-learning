use clap::Parser;
use colored::*;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use regex::Regex;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Stdout, Write},
    ops::Range,
    path::Path,
};

mod error;
pub use error::GrepError;

// 定义类型，在使用的时候可以简化复杂类型的书写
pub type StrategyFn<W, R> = fn(&Path, BufReader<R>, &Regex, &mut W) -> Result<(), GrepError>;

// 简化版的 grep, 支持正则表达式和文件通配符
#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Ying zhang")]
pub struct GrepConfig {
    // 用于查找正则表达式
    pattern: String,
    // 文件通配符
    glob: String,
}

impl GrepConfig {
    /// 使用缺省的策略查找匹配
    pub fn match_with_default_strategy(&self) -> Result<(), GrepError> {
        self.match_with(default_strategy)
    }

    /// 使用某个策略函数查找匹配
    pub fn match_with(&self, strategy: StrategyFn<Stdout, File>) -> Result<(), GrepError> {
        // 创建正则对象
        let regex = Regex::new(&self.pattern)?;
        /*
            文件或目录的路径模式匹配
                支持相对路径和绝对路径：
                    /home/\*rust
                    ../\*rust
                匹配目标所在同级目录下的所有目录和文件,输出路径
                    目标：/home/Git/rust-learning/rgrep/src/\*rs
                    输出：
                        "/home/Git/rust-learning/rgrep/src/main.rs"
                        "/home/Git/rust-learning/rgrep/src/rs"
        */
        let files: Vec<_> = glob::glob(&self.glob)?.collect();
        // 并行处理所有文件
        files.into_par_iter().for_each(|v| {
            /*
            if let 表达式复习：
                代替条件操作数的是关键字 let + 模式  = 检验对象，如果检验对象的值和模式匹配，则执行相应的块
            */
            if let Ok(filename) = v {
                if let Ok(file) = File::open(&filename) {
                    // 文件进行缓冲读取
                    let reader = BufReader::new(file);
                    // 标准输出
                    let mut stdout = io::stdout();
                    // 主要 grep 实现函数
                    if let Err(e) = strategy(filename.as_path(), reader, &regex, &mut stdout) {
                        println!("Internal error: {:?}", e);
                    }
                }
            }
        });
        Ok(())
    }
}

/// 缺省策略，重头到尾串行查找，最后输出 writer
pub fn default_strategy<W: Write, R: Read>(
    path: &Path,
    reader: BufReader<R>,
    pattern: &Regex,
    writer: &mut W,
) -> Result<(), GrepError> {
    // .filter_map ：对迭代器产生的元素进行过滤（filter）和映射（map），返回Option类型的值
    let matches: String = reader
        .lines()
        .enumerate()
        .map(|(lineno, line)| {
            // line.ok 返回 Ok 的结果
            // .flatten() 将嵌套的 Option 扁平化，去除一层包裹的 Some
            line.ok()
                .map(|line| {
                    // 对行内容进行匹配，并将匹配的结果交给 format_line 进行格式输出
                    // m.range() 记录匹配的内容在 line 中的起止位置
                    pattern
                        .find(&line)
                        .map(|m| format_line(&line, lineno + 1, m.range()))
                })
                .flatten()
        })
        .filter_map(|v| v.ok_or(()).ok())
        .join("\n");

    if !matches.is_empty() {
        writer.write(path.display().to_string().green().as_bytes())?;
        writer.write(b"\n")?;
        writer.write(matches.as_bytes())?;
        writer.write(b"\n")?;
    }
    Ok(())
}

///格式化输出匹配的行，包含行号，列号和带有高亮的第一个匹配项
pub fn format_line(line: &str, lineno: usize, range: Range<usize>) -> String {
    let Range { start, end } = range;
    let prefix = &line[..start];
    format!(
        "{0: >6}:{1: <3} {2}{3}{4}",
        lineno.to_string().blue(),
        // 找到匹配项的起始位置，注意对汉字等非 ascii 字符，我们不能使用 prefix.len()
        // 这是一个 O(n) 的操作，会拖累效率，这里只是为了演示的效果
        (prefix.chars().count() + 1).to_string().cyan(),
        prefix,
        &line[start..end].red(),
        &line[end..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn format_line_should_work() {
        let result = format_line("Hello, Tyr~", 1000, 7..10);
        let expected = format!(
            "{0: >6}:{1: <3} Hello, {2}~",
            "1000".blue(),
            "7".cyan(),
            "Tyr".red()
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn default_strategy_should_work() {
        let path = Path::new("src/main.rs");
        let input = b"hello world!\nhey Tyr";
        let reader = BufReader::new(&input[..]);
        let pattern = Regex::new("he\\w+").unwrap();
        let mut writer = Vec::new();
        default_strategy(path, reader, &pattern, &mut writer).unwrap();
        let result = String::from_utf8(writer).unwrap();
        let expected = [
            String::from("src/main.rs"),
            format_line("hello world!", 1, 0..5),
            format_line("hey Tyr!\n", 2, 0..3),
        ];
        assert_eq!(result, expected.join("\n"));
    }
}
