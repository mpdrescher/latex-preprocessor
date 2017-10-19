use std::fs::File;
use std::io::Read;
use std::io::Result as IOResult;
use std::env;

const DEFAULT_HEADER: &'static str = r#"
\documentclass[12pt, a4paper]{article}
\usepackage{amsmath}
\usepackage{amsfonts}
\usepackage{amssymb}
\begin{document}
"#;

const DEFAULT_FOOTER: &'static str = r#"
\end{document}
"#;

fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();
    for filepath in args {
        let filecontent = match read_file(filepath.clone()) {
            Ok(v) => v,
            Err(e) => {
                println!("error while reading {}: {}", filepath, e);
                return;
            }
        };
        let document = PreFile::from_string(filecontent);
        println!("{}", document.transpile());
    }
}

fn read_file(path: String) -> IOResult<String> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    let _ = file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

pub enum Line {
    Normal(String),
    Header(String, usize),
    Align(String)
}

impl Line {
    pub fn type_equals(&self, other: &Line) -> bool {
        self.get_type() == other.get_type()
    }

    pub fn get_type(&self) -> LineType {
        match self {
            &Line::Normal(_) => LineType::Normal,
            &Line::Header(_, x) => LineType::Header(x),
            &Line::Align(_) => LineType::Align
        }
    }

    pub fn get_content(self) -> String {
        match self {
            Line::Normal(s) => s,
            Line::Header(s, _) => s,
            Line::Align(s) => s
        }
    }
}

#[derive(PartialEq)]
pub enum LineType {
    Normal,
    Header(usize),
    Align
}

struct Block {
    block_type: LineType,
    content: Vec<String>
}

impl Block {
    pub fn from_block_buffer(buffer: Vec<Line>) -> Block {
        if buffer.len() == 0 {
            panic!("block buffer is empty");
        }
        let block_type = buffer.get(0).unwrap().get_type();
        let mut content_buffer = Vec::new();
        for elem in buffer {
            content_buffer.push(elem.get_content());
        }
        Block {
            block_type: block_type,
            content: content_buffer
        }
    }

    pub fn transpile(self) -> String {

    }
}

pub struct PreFile {
    blocks: Vec<Block>
}

impl PreFile {
    pub fn from_string(string: String) -> PreFile {
        let mut lines = Vec::new();
        for line_str in string.lines() {
            let line = line_str.to_owned();
            if line.starts_with(">") {
                lines.push(Line::Align(line));
            }
            else if line.starts_with("#") {
                lines.push(Line::Header(line, 1));
            }
            else { 
                lines.push(Line::Normal(line));
            }
        }
        let mut blocks = Vec::new();
        let mut block_buffer = Vec::new();
        let mut current_type = None;
        for line in lines {
            if Some(line.get_type()) == current_type {
                block_buffer.push(line);
            }
            else if current_type == None {
                current_type = Some(line.get_type());
                block_buffer.push(line);
            }
            else {
                blocks.push(Block::from_block_buffer(block_buffer));
                block_buffer = Vec::new();
                current_type = Some(line.get_type());
                block_buffer.push(line);
            }
        }
        PreFile {
            blocks: blocks
        }
    }

    pub fn transpile(self) -> String {
        let mut buffer = String::new();
        buffer.append(&mut DEFAULT_HEADER.to_owned());
        for elem in self.blocks {
            buffer.append(&mut elem.transpile());
        }
        buffer.append(&mut DEFAULT_FOOTER.to_owned());
        buffer
    }
}
