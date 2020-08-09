use std::io::Read;

fn main() {
    let file_name = &std::env::args().collect::<Vec<_>>()[1];
    let mut file = std::fs::File::open(file_name).expect("could not open file");
    // serde_yaml doesn't support multi-document yaml, so split it up for it ourselves.
    let mut document = String::new();
    file.read_to_string(&mut document).unwrap();

    print!("{}", convert_doc(document))
}

fn convert_doc(document: String) -> String {
    let lines: Vec<_> = document.lines().collect();
    let documents: Vec<_> = lines.split(|line| line.starts_with("---")).map(|lines| lines.join("\n")).collect();

    // Trim empty documents, best effort.
    let documents: Vec<_> = documents.iter().filter(|d| d.trim() != "").collect();

    let docs: serde_yaml::Sequence = documents.iter().filter_map(|document| {
        match serde_yaml::from_str(&document) {
            Ok(doc) => Some(doc),
            Err(e) if format!("{:?}", e) == "EndOfStream" => {
                // An empty document can result in this; let's assume it's non-fatal
                None
            },
            Err(e) => {
                panic!("Unable to deserialize document: {}", e);
            }
        }
    }).collect();

    let len = docs.len();
    match len {
        0 => {
            panic!("no document supplied");
        }
        1 => {
            // TODO: unneeded clone
            let nix = convert_yaml(docs[0].clone());
            let nix_string = format!("{}", nix);
            nixpkgs_fmt::reformat_string(&nix_string)
        }
        _ => {
            let nix = convert_yaml(serde_yaml::Value::Sequence(docs));
            let nix_string = format!("{}", nix);
            nixpkgs_fmt::reformat_string(&nix_string)
        }
    }
}

fn convert_yaml(v: serde_yaml::Value) -> NixExpression {
    NixExpression(v)
}

struct NixExpression(serde_yaml::Value);

impl std::fmt::Display for NixExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match &self.0 {
            serde_yaml::Value::Null => "null".to_string(),
            serde_yaml::Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            serde_yaml::Value::Number(n) => format!("{}", n),
            serde_yaml::Value::String(s) => escape(s),
            serde_yaml::Value::Sequence(s) => {
                let expr_list = s.iter().map(|el| convert_yaml(el.clone()));
                let str_list: Vec<_> = expr_list.map(|expr| format!("{}", expr)).collect();
                format!("[\n {}\n ]", str_list.join("\n"))
            },

            serde_yaml::Value::Mapping(m) => {
                let mut res = String::new();
                res += "{\n";
                for (k, v) in m {
                    let k = convert_yaml(k.clone());
                    let v = convert_yaml(v.clone());
                    res += &format!("{} = {};\n", k, v);
                }
                res += "}";
                res
            },
        };
        write!(f, "{}", s)
    }
}

// Escape a string; valid escapes here, + $ -> ''$
// https://github.com/NixOS/nix/blob/3f264916dbfe346a71fa4182c9037332ac54f9d9/src/libexpr/lexer.l#L54-L76
fn escape(s: &str) -> String {
    let mut result = String::new();
    result += "\"";
    for c in s.chars() {
        result += &match c {
            '\n' => "\n".to_string(),
            '\t' => "\t".to_string(),
            '\r' => "\r".to_string(),
            '"' => "\\\"".to_string(),
            '$' => "''$".to_string(),
            c => c.to_string(),
        };
    }

    result += "\"";
    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_documents() {
        for pairs in vec![
            (
                include_str!("testdata/deployment.yaml"),
                include_str!("testdata/deployment.nix")
            ),
            (
                include_str!("testdata/multi-doc.yaml"),
                include_str!("testdata/multi-doc.nix")
            )
        ] {
            assert_eq!(pairs.1, convert_doc(pairs.0.to_string()));
        }
    }
}
