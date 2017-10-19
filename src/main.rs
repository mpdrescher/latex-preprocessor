use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::Result as IOResult;
use std::env;

const DEFAULT_HEADER: &'static str = r#"\documentclass[12pt, a4paper, twoside, titlepage]{article}
\usepackage{amsmath}
\usepackage{amsfonts}
\usepackage{amssymb}
\usepackage{a4}
\usepackage[ngerman]{babel}
\usepackage[utf8x]{inputenc}
\usepackage{ragged2e}
\begin{document}
\begin{flushleft}
"#;

const DEFAULT_FOOTER: &'static str = r#"\end{flushleft}
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
        let result = document.transpile();
        match write_file(&format!("{}.tex", &filepath), result) {
            Ok(_) => {},
            Err(e) => {
                println!("error while writing {}: {}", filepath, e);
            }
        }
    }
}

fn read_file(path: String) -> IOResult<String> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    let _ = file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn write_file(path: &String, content: String) -> IOResult<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
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

    pub fn transpile(mut self) -> String {
        match self.block_type {
            LineType::Normal => {
                self.content = self.content
                    .into_iter()
                    .map(|x| if x.trim() == "~~" {format!("\\quad\\newline")} else {x})
                    .collect::<Vec<String>>();
                format!("{}\n", fold_strings(self.content, "\n", "").trim())
            },
            LineType::Header(n) => {
                match n {
                    1 => {
                        format!("\\section{{ {} }}\n", fold_strings(self.content, " ", "").trim())
                    },
                    2 => {
                        format!("\\subsection{{ {} }}\n", fold_strings(self.content, " ", "").trim())
                    },
                    3 => {
                        format!("\\subsubsection{{ {} }}\n", fold_strings(self.content, " ", "").trim())
                    },
                    4 => {
                        let mut buffer = String::new();
                        buffer.push_str("\\end{flushleft}\n");
                        buffer.push_str("\\center\n");
                        let cropped_content = fold_strings(self.content, " ", "").trim().to_owned();
                        buffer.push_str("\\large");
                        buffer.push_str(&format!("\\textbf{{ {} }}\n", cropped_content));
                        buffer.push_str("\\normalsize\n");
                        buffer.push_str("\\endcenter\n");
                        buffer.push_str("\\begin{flushleft}\n");
                        buffer
                    },
                    5 => {
                        format!("\\textbf{{ {} }}\\\\\n", fold_strings(self.content, " ", "").trim())
                    },
                    _ => {
                        panic!("header level exceeded 2");
                    }
                }
            },
            LineType::Align => {
                let commented = self.content.iter().any(|x| x.contains("~~"));
                let mut buffer = String::new();
                buffer.push_str("\\begin{align*}\n");
                for elem in self.content {
                    if elem.contains("~~") {
                        let (p1, p2) = elem.split_at(elem.find("~~").unwrap());
                        buffer.push_str(&format!("&{}", p1.trim()));
                        buffer.push_str(&format!(" &&\\text{{ {} }}", p2[2..].to_owned()));
                    }
                    else {
                        if commented {
                            buffer.push_str(&format!("&{} &&\\text{{\\quad}}", elem));
                        }
                        else {
                            buffer.push_str(&format!("&{}", elem));
                        }
                    }
                    buffer.push_str("\\\\\n");
                }    
                buffer.push_str("\\end{align*}\n");
                buffer
            }
        }
    }   
}

pub fn fold_strings(string: Vec<String>, suffix: &'static str, prefix: &'static str) -> String {
    string.into_iter().fold(String::new(), (|mut acc, x| {
        acc.push_str(prefix);
        acc.push_str(&x);
        acc.push_str(suffix); 
        acc
    }))
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
                lines.push(Line::Align(line[1..].to_owned()));
            }
            else if line.starts_with("#") {
                let mut counter = 0;
                for ch in line.chars() {
                    if ch == '#' {
                        counter += 1;
                    }
                    else {
                        break;
                    }
                }
                let cropped_line = line[counter..].to_owned();
                lines.push(Line::Header(cropped_line, counter));
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
        blocks.push(Block::from_block_buffer(block_buffer));
        PreFile {
            blocks: blocks
        }
    }

    pub fn transpile(self) -> String {
        let mut buffer = String::new();
        buffer.push_str(DEFAULT_HEADER);
        for elem in self.blocks {
            buffer.push_str(&elem.transpile());
        }
        buffer.push_str(&DEFAULT_FOOTER);
        buffer
    }
}
