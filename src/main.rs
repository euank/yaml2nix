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
            let nix_string = serde_nix::to_string(&docs[0]).unwrap();
            nixpkgs_fmt::reformat_string(&nix_string)
        }
        _ => {
            let nix_string = serde_nix::to_string(&docs).unwrap();
            nixpkgs_fmt::reformat_string(&nix_string)
        }
    }
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
