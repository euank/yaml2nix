fn main() {
    let file_name = &std::env::args().collect::<Vec<_>>()[1];
    let file = std::fs::File::open(file_name).expect("could not open file");
    let val: serde_yaml::Value = serde_yaml::from_reader(file).expect("could not deserialize yaml");

    let nix = convert_yaml(val);
    println!("{}", nix);
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
                format!("[ {} ]", str_list.join(" "))
            },

            serde_yaml::Value::Mapping(m) => {
                let mut res = String::new();
                res += "{ ";
                for (k, v) in m {
                    let k = convert_yaml(k.clone());
                    let v = convert_yaml(v.clone());
                    res += &format!("{} = {}; ", k, v);
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
            '$' => "''$".to_string(),
            c => c.to_string(),
        };
    }

    result += "\"";
    result
}

fn convert_yaml(v: serde_yaml::Value) -> NixExpression {
    NixExpression(v)
}
