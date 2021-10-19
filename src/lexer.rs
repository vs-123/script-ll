use crate::errors::Error;

#[derive(Debug, Clone)]
pub struct Line(pub Vec<String>);

pub fn lex(code: String) -> (Vec<Line>, Error) {
    let mut lexed_code: Vec<Line> = Vec::new();
    let mut lexed_code_line: Vec<String> = Vec::new();
    let mut temp: String = String::new(); // Will be used to add to lexed_code_line
    let mut is_string: bool = false;
    let mut temp_string: String = String::new(); // Will be used for strings in ll
    let mut error: Error = Error::None;

    for (line_number, line) in code.replace('\r', "").trim().split('\n').enumerate() {
        let line_characters = &(*line).chars().collect::<Vec<char>>();
        for (character_index, &c) in line_characters.clone().iter().enumerate() {
            if c == '"' {
                if !is_string {
                    is_string = true
                } else {
                    is_string = false;
                    temp_string.push(c);
                }
            }

            if is_string && character_index == line_characters.len() - 1 {
                error = Error::LexingError(format!(
                    "\nCode:\n{} | {}\nProblem: String was never ended",
                    line_number,
                    &(*line)
                ));
                break;
            } else if is_string {
                temp_string.push(c);
            } else {
                if !temp_string.is_empty() {
                    lexed_code_line.push(temp_string.clone());
                    temp_string = String::new();
                }

                if c == ' ' {
                    lexed_code_line.push(temp.clone());
                    temp = String::new();
                } else if character_index == line_characters.len() - 1 && c != '"' {
                    temp.push(c);
                    lexed_code_line.push(temp.clone());
                    temp = String::new();
                } else if !vec!['\"'].contains(&c) {
                    temp.push(c);
                }
            }
        }

        lexed_code_line.retain(|x| !(*x).is_empty());

        lexed_code.push(Line(lexed_code_line));
        lexed_code_line = Vec::new();
    }

    if lexed_code.is_empty() {
        error = Error::LexingError("No code found".to_string());
    }

    (lexed_code, error)
}
