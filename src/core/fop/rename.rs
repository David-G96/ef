use color_eyre::Result as Res;
use heck::ToPascalCase;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RenameRule {
    /// simply replace string `old` to `new`
    Replace {
        old: String,
        new: String,
    },
    /// change the case
    SetCase(CaseType), // 大小写转换
    AddPrefix(String),
    AddSuffix(String),
    Numbering {
        start: u32,
        pad: usize,
    },
    RegexReplace {
        pattern: String,
        replacement: String,
    },
    Pipe(Vec<Self>),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum CaseType {
    /// caseType
    Pascal,
    /// CaseType
    Camel,
    /// case_type
    Snake,
    /// case-type
    Kebab,
    /// CASE_TYPE
    Upper,
    /// case_type
    Lower,
}

pub fn rename(input: Vec<String>, rule: RenameRule) -> Res<Vec<String>> {
    let result: Vec<String> = match rule {
        RenameRule::Replace { old, new } => input
            .into_iter()
            .map(move |s| s.replace(&old, &new))
            .collect(),
        RenameRule::SetCase(case_type) => input
            .into_iter()
            .map(move |s| apply_case(s, case_type))
            .collect(),
        RenameRule::AddPrefix(prefix) => input
            .into_iter()
            .map(move |s| format!("{}{}", prefix, s))
            .collect(),
        RenameRule::AddSuffix(suffix) => input
            .into_iter()
            .map(move |s| format!("{}{}", s, suffix))
            .collect(),
        RenameRule::Numbering { start, pad } => {
            let mut counter = start;
            input
                .into_iter()
                .map(move |s| {
                    let num_str = format!("{:0pad$}", counter, pad = pad);
                    counter += 1;
                    format!("{}_{}", num_str, s)
                })
                .collect()
        }
        RenameRule::RegexReplace {
            pattern,
            replacement,
        } => {
            let regex = regex::Regex::new(&pattern)?;
            input
                .into_iter()
                .map(move |s| regex.replace_all(&s, &replacement).to_string())
                .collect()
        }
        RenameRule::Pipe(rules) => rules
            .iter()
            .try_fold(input, |acc, rule| rename(acc, rule.clone()))?,
    };

    Ok(result)
}

fn apply_case(input: String, case_type: CaseType) -> String {
    match case_type {
        CaseType::Pascal => to_pascal_case(&input),
        CaseType::Camel => to_camel_case(&input),
        CaseType::Snake => to_snake_case(&input),
        CaseType::Kebab => to_kebab_case(&input),
        CaseType::Upper => input.to_uppercase(),
        CaseType::Lower => input.to_lowercase(),
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c.is_alphanumeric() {
            if capitalize_next {
                result.push(c.to_uppercase().next().unwrap_or(c));
                capitalize_next = false;
            } else {
                result.push(c.to_lowercase().next().unwrap_or(c));
            }
        } else {
            capitalize_next = true;
        }
    }
    result
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c.is_alphanumeric() {
            if capitalize_next {
                result.push(c.to_uppercase().next().unwrap_or(c));
                capitalize_next = false;
            } else {
                result.push(c.to_lowercase().next().unwrap_or(c));
            }
        } else {
            capitalize_next = true;
        }
    }

    result
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_space = true; // Start with true to not add underscore at beginning

    for c in s.chars() {
        if c.is_alphanumeric() {
            if !prev_was_space {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
            prev_was_space = false;
        } else if c.is_whitespace() || c == '_' || c == '-' {
            prev_was_space = true;
        } else {
            result.push('_');
            prev_was_space = false;
        }
    }

    result
}

fn to_kebab_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_whitespace() || c == '_' {
                '-'
            } else {
                c
            }
        })
        .collect::<String>()
        .chars()
        .map(|c| c.to_lowercase().to_string())
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_replace_rule() {
        let input = vec!["file1.txt".to_string(), "file2.txt".to_string()];
        let rule = RenameRule::Replace {
            old: ".txt".to_string(),
            new: ".bak".to_string(),
        };
        let result = rename(input, rule).unwrap();
        assert_eq!(
            result,
            vec!["file1.bak".to_string(), "file2.bak".to_string()]
        );
    }

    #[test]
    fn test_set_case_rule() {
        let input = vec!["file_name".to_string(), "another_file".to_string()];
        let rule = RenameRule::SetCase(CaseType::Pascal);
        let result = rename(input, rule).unwrap();
        assert_eq!(
            result,
            vec!["FileName".to_string(), "AnotherFile".to_string()]
        );
    }

    #[test]
    fn test_add_prefix_rule() {
        let input = vec!["file.txt".to_string(), "doc.pdf".to_string()];
        let rule = RenameRule::AddPrefix("prefix_".to_string());
        let result = rename(input, rule).unwrap();
        assert_eq!(
            result,
            vec!["prefix_file.txt".to_string(), "prefix_doc.pdf".to_string()]
        );
    }

    #[test]
    fn test_add_suffix_rule() {
        let input = vec!["file".to_string(), "document".to_string()];
        let rule = RenameRule::AddSuffix("_suffix".to_string());
        let result = rename(input, rule).unwrap();
        assert_eq!(
            result,
            vec!["file_suffix".to_string(), "document_suffix".to_string()]
        );
    }

    #[test]
    fn test_numbering_rule() {
        let input = vec![
            "file".to_string(),
            "document".to_string(),
            "image".to_string(),
        ];
        let rule = RenameRule::Numbering { start: 1, pad: 3 };
        let result = rename(input, rule).unwrap();
        assert_eq!(
            result,
            vec![
                "001_file".to_string(),
                "002_document".to_string(),
                "003_image".to_string()
            ]
        );
    }

    #[test]
    fn test_regex_replace_rule() {
        let input = vec!["file123.txt".to_string(), "doc456.pdf".to_string()];
        let rule = RenameRule::RegexReplace {
            pattern: r"\d+".to_string(),
            replacement: "###".to_string(),
        };
        let result = rename(input, rule).unwrap();
        assert_eq!(
            result,
            vec!["file###.txt".to_string(), "doc###.pdf".to_string()]
        );
    }

    #[test]
    fn test_pipe_rule() {
        let input = vec!["file.txt".to_string()];
        let rule = RenameRule::Pipe(vec![
            RenameRule::AddPrefix("prefix_".to_string()),
            RenameRule::Replace {
                old: ".txt".to_string(),
                new: ".bak".to_string(),
            },
        ]);
        let result = rename(input, rule).unwrap();
        assert_eq!(result, vec!["prefix_file.bak".to_string()]);
    }

    #[test]
    fn test_edge_case_empty_string() {
        let input = vec!["".to_string()];
        let rule = RenameRule::AddPrefix("test_".to_string());
        let result = rename(input, rule).unwrap();
        assert_eq!(result, vec!["test_".to_string()]);
    }

    #[test]
    fn test_edge_case_empty_vec() {
        let input: Vec<String> = vec![];
        let rule = RenameRule::AddPrefix("test_".to_string());
        let result = rename(input, rule).unwrap();
        let expected: Vec<String> = vec![];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_rename_pascal_case() {
        assert_eq!("CaseType", to_pascal_case("case type"));
        assert_eq!("CaseType", to_pascal_case("case_type"));
        assert_eq!("CaseType", to_pascal_case("case-type"));
        assert_eq!("CaseType", to_pascal_case("CASE_TYPE"));
        assert_eq!("你好世界", to_pascal_case("你好世界"));
    }
}
