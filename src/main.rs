use std::print;

#[derive(PartialEq, Debug)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(i32),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

fn parseInp(input: &str) -> Result<JsonValue, String> {
    match input {
        "null" => Ok(JsonValue::Null),
        "true" => Ok(JsonValue::Bool(true)),
        "false" => Ok(JsonValue::Bool(false)),
        "[]" => Ok(JsonValue::Array(vec![])),
        _ => {
            if input.starts_with('"') && input.ends_with('"') {
                // Ok(JsonValue::String(String::from(&input[1..input.len() - 1])))
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
                                            + char
                                                .to_digit(16)
                                                .ok_or(String::from("Invalid String"))?;
                                    }
                                    None => return Err(String::from("INVALID STRING")),
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
                                _ => return Err(String::from("UNKNOWN ESCAPE CHAR")),
                            }
                            escaped = false;
                        } else {
                            build_str.push(char);
                        }
                    }
                }
                if escaped {
                    return Err(String::from("INVALID String"));
                }
                return Ok(JsonValue::String(build_str));
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
                    .collect::<Result<Vec<JsonValue>, String>>()?;
                if inside_string || escaped {
                    return Err(String::from("Invalid JSON TO PARSE"));
                }
                Ok(JsonValue::Array(v))
            } else if input.starts_with('{') && input.ends_with('}') {
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
                                vEle.push(input[start..i].to_string());
                                start = i + 1
                            }
                        }
                    }
                }
                vEle.push(input[start..input.len() - 1].to_string());

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
                                    _ => return Err(String::from("Incorrect Key")),
                                }
                                break;
                            }
                        }
                    }
                }
                return Ok(JsonValue::Object(kv));
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
    use std::print;

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

    #[test]
    // INCOMPLETE TEST FOR NOW , WILL FIX IT LATER
    fn test_obj_split() {
        let val = parseInp(r#"{"name":"mh","age":25}"#).unwrap();
    }
}

// discarded code - string iteration
// for char in input[1..input.len() - 1].chars().peekable() {
//    if char == '\\'{
//     escaped=true;
//    }
//    else{
//     if char == 'u' && escaped{
//         unicode = true;
//     }
//     else if escaped{
//         match char{
//             'n'=> build_str.push('\n'),
//             '\\'=> build_str.push('\\'),
//             't'=> build_str.push('\t'),
//             'r'=> build_str.push('\r'),
//             'f'=> build_str.push(char::from_u32(0x000c).unwrap()),
//             'b'=> build_str.push(char::from_u32(0x0008).unwrap()),
//             '"'=> build_str.push('\"'),
//             '/'=> build_str.push('/'),
//             _=> return Err(String::from("UNKNOWN ESCAPE CHAR"))
//         }
//         escaped = false;
//     }
//     else{
//         build_str.push(char);
//     }
//    }
// }
