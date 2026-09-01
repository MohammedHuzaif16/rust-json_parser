use std::print;

#[derive(PartialEq, Debug)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

#[derive(Debug)]
enum JsonError {
    StringError,
    ArrayError,
    ObjectError,
    NumberError,
}

fn parseInp(input: &str) -> Result<JsonValue, JsonError> {
    match input {
        "null" => Ok(JsonValue::Null),
        "true" => Ok(JsonValue::Bool(true)),
        "false" => Ok(JsonValue::Bool(false)),
        "[]" => Ok(JsonValue::Array(vec![])),
        _ => {
            if input.starts_with('"') && input.ends_with('"') {
                parseString(input)
            } else if input.starts_with('[') && input.ends_with(']') {
                parseArray(input)
            } else if input.starts_with('{') && input.ends_with('}') {
                parseObject(input)
            } else {
                parseNumber(input)
            }
        }
    }
}

fn parseNumber(input: &str) -> Result<JsonValue, JsonError> {
    let mut chars = input.chars().peekable();
    let mut is_decimal = false;
    let mut is_exp = false;
    let mut exp_sign = false;
    let mut exp_digit = false;
    let mut integer_digit_count = 0;

    // Optional leading '-'
    if let Some(&x) = chars.peek() {
        if x == '-' {
            chars.next();
        }
    }

    while let Some(char) = chars.next() {
        // Integer / fraction / exponent digits
        if char.is_ascii_digit() {
            if !is_decimal && !is_exp {
                integer_digit_count += 1;

                // Leading zero: 01, 00, 012 etc. are invalid
                if integer_digit_count == 1 && char == '0' {
                    if let Some(&x) = chars.peek() {
                        if x.is_ascii_digit() {
                            return Err(JsonError::NumberError);
                        }
                    }
                }
            }

            if is_exp {
                exp_digit = true;
            }

            continue;
        }

        // Decimal point
        if char == '.' {
            if is_decimal || is_exp {
                return Err(JsonError::NumberError);
            }

            match chars.peek() {
                Some(&x) if x.is_ascii_digit() => {}
                _ => return Err(JsonError::NumberError),
            }

            is_decimal = true;
            continue;
        }

        // Exponent
        if char == 'e' || char == 'E' {
            if is_exp {
                return Err(JsonError::NumberError);
            }

            match chars.peek() {
                Some(&x) if x.is_ascii_digit() => {}
                Some(&x) if x == '+' || x == '-' => {}
                _ => return Err(JsonError::NumberError),
            }

            is_exp = true;
            continue;
        }

        // Exponent sign
        if char == '+' || char == '-' {
            if !is_exp || exp_sign {
                return Err(JsonError::NumberError);
            }

            match chars.peek() {
                Some(&x) if x.is_ascii_digit() => {}
                _ => return Err(JsonError::NumberError),
            }

            exp_sign = true;
            continue;
        }

        return Err(JsonError::NumberError);
    }

    // An exponent must contain at least one digit.
    if is_exp && !exp_digit {
        return Err(JsonError::NumberError);
    }

    match input.parse::<f64>() {
        Ok(x) => Ok(JsonValue::Number(x)),
        Err(_) => return Err(JsonError::NumberError),
    }
}
fn parseString(input: &str) -> Result<JsonValue, JsonError> {
    let mut build_str = String::new();
    let mut escaped: bool = false;
    let mut chars = input[1..input.len() - 1].chars().peekable();

    while let Some(char) = chars.next() {
        if char == '\\' {
            escaped = true;
        } else {
            if char == 'u' && escaped {
                let mut hSum = 0;
                let mut i = 0;
                while i < 4 {
                    match chars.next() {
                        Some(char) => {
                            hSum = hSum * 16
                                + char.to_digit(16).ok_or(String::from("Invalid String"))?;
                        }
                        None => return Err(JsonError::StringError),
                    };
                    i += 1
                }
                let val = char::from_u32(hSum).ok_or(String::from("Invalid String"))?;
                build_str.push(val);
                escaped = false;
            } else if escaped {
                match char {
                    'n' => build_str.push('\n'),
                    '\\' => build_str.push('\\'),
                    't' => build_str.push('\t'),
                    'r' => build_str.push('\r'),
                    'f' => build_str.push(char::from_u32(0x000c).unwrap()),
                    'b' => build_str.push(char::from_u32(0x0008).unwrap()),
                    '"' => build_str.push('\"'),
                    '/' => build_str.push('/'),
                    _ => return Err(JsonError::StringError),
                }
                escaped = false;
            } else {
                build_str.push(char);
            }
        }
    }
    if escaped {
        return Err(JsonError::StringError);
    }
    return Ok(JsonValue::String(build_str));
}
fn parseArray(input: &str) -> Result<JsonValue, JsonError> {
    let mut depth = 0;
    let mut start = 1;
    let mut v = vec![];
    let mut inside_string: bool = false;
    let mut escaped: bool = false;
    for (i, char) in input.char_indices() {
        if escaped {
            escaped = false
        } else {
            if char == '\\' && inside_string {
                escaped = true
            } else if char == '"' {
                if inside_string {
                    inside_string = false
                } else {
                    inside_string = true
                }
            } else {
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
        .collect::<Result<Vec<JsonValue>, JsonError>>()?;
    if inside_string || escaped {
        return Err(JsonError::ArrayError);
    }
    Ok(JsonValue::Array(v))
}
fn parseObject(input: &str) -> Result<JsonValue, JsonError> {
    if input == "{}" {
        return Ok(JsonValue::Object(vec![]));
    }
    let mut isString = false;
    let mut escaped = false;
    let mut vEle: Vec<String> = vec![];
    let mut start = 1;
    let mut arrayDepth = 0;
    let mut objDepth = 0;
    for (i, char) in input.char_indices() {
        if char == '{' && i == 0 || char == '}' && i == input.len() - 1 {
            continue;
        } else if escaped {
            escaped = false
        } else {
            if char == '\\' && isString {
                escaped = true
            } else if char == '"' {
                if !escaped {
                    if !isString {
                        isString = true
                    } else {
                        isString = false
                    }
                }
                escaped = false
            } else if char == '[' && !isString {
                arrayDepth += 1
            } else if char == ']' && !isString {
                arrayDepth -= 1;
            } else if char == '{' && !isString {
                objDepth += 1
            } else if char == '}' && !isString {
                objDepth -= 1;
            } else {
                if !isString && arrayDepth == 0 && objDepth == 0 && char == ',' {
                    vEle.push(input[start..i].trim().to_string());
                    start = i + 1
                }
            }
        }
    }
    vEle.push(input[start..input.len() - 1].trim().to_string());

    let mut kv: Vec<(String, JsonValue)> = vec![];
    for ele in vEle {
        isString = false;
        escaped = false;
        for (i, char) in ele.char_indices() {
            if escaped {
                escaped = false;
            } else {
                if char == '\\' {
                    escaped = true
                } else if char == '"' {
                    if isString {
                        isString = false
                    } else {
                        isString = true
                    }
                } else if char == ':' && !isString {
                    match parseInp(&ele[..i])? {
                        JsonValue::String(x) => {
                            kv.push((x, parseInp(ele[i + 1..ele.len()].trim())?));
                        }
                        _ => return Err(JsonError::ObjectError),
                    }
                    break;
                }
            }
        }
    }
    return Ok(JsonValue::Object(kv));
}
fn main() {}

#[cfg(test)]
mod tests {
    use std::{assert_eq, print};

    use super::*;
    #[test]
    fn test_basic_array_split() {
        assert_eq!(
            parseInp("[1,2,3]"),
            Ok(JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0)
            ]))
        );
    }

    #[test]
    fn test_obj() {
        assert_eq!(
            parseInp(r#"{"name":"mh","age":25}"#),
            Ok(JsonValue::Object(vec![
                ("name".to_string(), JsonValue::String("mh".to_string())),
                ("age".to_string(), JsonValue::Number(25.0))
            ]))
        )
    }

    #[test]
    fn test_null_obj() {
        assert_eq!(parseInp(r#"{}"#), Ok(JsonValue::Object(vec![])))
    }

    #[test]
    fn test_obj_all() {
        assert_eq!(
            parseInp(
                r#"{"active":true,"value":null, "url":"http://example.com","user":{"items":[1,2,3]}}"#
            ),
            Ok(JsonValue::Object(vec![
                ("active".to_string(), JsonValue::Bool(true)),
                ("value".to_string(), JsonValue::Null),
                (
                    "url".to_string(),
                    JsonValue::String("http://example.com".to_string())
                ),
                (
                    "user".to_string(),
                    JsonValue::Object(vec![(
                        "items".to_string(),
                        JsonValue::Array(vec![
                            JsonValue::Number(1.0),
                            JsonValue::Number(2.0),
                            JsonValue::Number(3.0)
                        ])
                    )])
                )
            ]))
        )
    }
}
