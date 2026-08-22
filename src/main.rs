#[derive(PartialEq, Debug)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(i32),
    String(String),
    Array(Vec<JsonValue>),
}

fn parseInp(input: &str) -> Result<JsonValue, String> {
    match input {
        "null" => Ok(JsonValue::Null),
        "true" => Ok(JsonValue::Bool(true)),
        "false" => Ok(JsonValue::Bool(false)),
        "[]" => Ok(JsonValue::Array(vec![])),
        _ => {
            if input.starts_with('"') && input.ends_with('"') {
                Ok(JsonValue::String(String::from(&input[1..input.len() - 1])))
            } else if input.starts_with('[') && input.ends_with(']') {
                let mut depth = 0;
                let mut start = 1;
                let mut v = vec![];
                let mut inside_string: bool = false;
                let mut escaped: bool = false;
                for (i, char) in input.char_indices() {
                    if escaped {
                        escaped = false
                    } else {
                        if char == '\\' && inside_string{
                            escaped = true
                        } 
                        else if char == '"' {
                            if inside_string {
                                inside_string = false
                            } else {
                                inside_string = true
                            }
                        } 
                        else {
                            if char == '[' {
                                depth += 1
                            } else if char == ']' {
                                depth -= 1;
                                if depth == 0 {
                                    v.push(&input[start..i])
                                }
                            } else if depth == 1 && char == ',' {
                                v.push(&input[start..i]);
                                start = i + 1
                            }
                        }
                    }
                }
                let v = v
                    .iter()
                    .map(|x| parseInp(x.trim()))
                    .collect::<Result<Vec<JsonValue>, String>>()?;
                if inside_string || escaped{
                    return Err(String::from("Invalid JSON TO PARSE"))
                }
                Ok(JsonValue::Array(v))
            } else if let Ok(x) = input.parse::<i32>() {
                return Ok(JsonValue::Number(x));
            } else {
                return Err(String::from("wWrong type"));
            }
        }
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_array_split() {
        assert_eq!(
            parseInp("[1,2,3]"),
            Ok(JsonValue::Array(vec![
                JsonValue::Number(1),
                JsonValue::Number(2),
                JsonValue::Number(3)
            ]))
        );
    }
}
