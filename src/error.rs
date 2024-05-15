use colored::*;

pub trait ExecutionErrorExt<T> {
    fn idc(self) -> Result<T, ExecutionError>;
}

// Implement the trait for Option
impl<T> ExecutionErrorExt<T> for Option<T> {
    fn idc(self) -> Result<T, ExecutionError> {
        let error = ExecutionError::new(0, String::new());
        match self {
            Some(value) => Ok(value),
            None => Err(error),
        }
    }
}

pub trait ParseErrorExt<T> {
    fn idc(self) -> Result<T, ParseError>;
}

// Implement the trait for Option
impl<T> ParseErrorExt<T> for Option<T> {
    fn idc(self) -> Result<T, ParseError> {
        let error = ParseError::new(0, 0, String::new());
        match self {
            Some(value) => Ok(value),
            None => Err(error),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExecutionError {
    pub at: usize,
    pub cause: String,
}
impl ExecutionError {
    pub fn new(at: usize, cause: String) -> Self {
        Self { at, cause }
    }

    pub fn format_with(&self, input: &str, title: &str) -> String {
        let underscore_width = 5;
        let (line, start_offset) = get_line_at_index(input, self.at);
        let line_number = get_line_number(input, self.at);

        let header = format!(
            "{}:\n  {}",
            title.red().bold(),
            "▍".blue().bold()
        );
        let body = format!(
            "{} {} {}",
            format!("{}", line_number).blue().bold(),
            "▍".blue().bold(),
            line
        );
        let footer = format!(
            "  {} {} {}",
            "▍".blue().bold(),
            start_offset,
            "^".repeat(underscore_width).cyan(),
        );
        let footer = format!("{}\n{}", footer, self.cause.red());

        format!("{}\n{}\n{}", header, body, footer)
    }
}

#[derive(Clone, Debug)]
pub struct ParseError {
    pub at: usize,
    pub depth: u32,
    pub cause: String,
}
impl ParseError {
    pub fn new(at: usize, depth: u32, cause: String) -> Self {
        Self { at, depth, cause }
    }

    pub fn format_with(&self, input: &str, title: &str) -> String {
        let underscore_width = 5;
        let (line, start_offset) = get_line_at_index(input, self.at);
        let line_number = get_line_number(input, self.at);

        let header = format!(
            "{}:\n  {}",
            title.red().bold(),
            "▍".blue().bold()
        );
        let body = format!(
            "{} {} {}",
            format!("{}", line_number).blue().bold(),
            "▍".blue().bold(),
            line
        );
        let footer = format!(
            "  {} {} {}",
            "▍".blue().bold(),
            start_offset,
            "^".repeat(underscore_width).cyan(),
        );
        let footer = format!("{}\n{}", footer, self.cause.red());

        format!("{}\n{}\n{}", header, body, footer)
    }
}

fn get_line_at_index(text: &str, index: usize) -> (String, String) {
    let mut line_start = 0;
    let mut line_end = 0;

    // Find the start and end indices of the line containing the given index
    for (i, c) in text.chars().enumerate() {
        if c == '\n' {
            if i >= index {
                break;
            }
            line_start = i + 1;
        }
        line_end = i;
    }

    // Extract the line from the text
    let line = &text[line_start..=line_end];
    let start_offset = index - line_start;
    let start_offset_string: &String = &line[0..start_offset]
        .chars()
        .map(|c| if c == '\t' { c } else { ' ' })
        .collect();
    (line.to_string(), start_offset_string.clone())
}

fn get_line_number(text: &str, index: usize) -> usize {
    text[0..index].chars().filter(|x| x == &'\n').count() + 1
}
