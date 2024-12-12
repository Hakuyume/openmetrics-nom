use nom::combinator::complete;
use nom::{Finish, Parser};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::path::PathBuf;

#[rstest::rstest]
fn test(
    #[base_dir = "./OpenMetrics/tests/testdata/parsers"]
    #[files("*")]
    path: PathBuf,
) {
    #[derive(Debug, serde::Deserialize)]
    struct Test {
        #[serde(rename = "type")]
        type_: String,
        file: PathBuf,
        #[serde(rename = "shouldParse")]
        should_parse: bool,
    }

    let test =
        serde_json::from_reader::<_, Test>(File::open(path.join("test.json")).unwrap()).unwrap();
    dbg!(&test);

    assert_eq!(test.type_, "text");
    let input = fs::read_to_string(path.join(&test.file)).unwrap();

    let exposition = complete(crate::exposition)
        .parse(input.as_str())
        .finish()
        .map_err(|e| nom::error::convert_error(input.as_str(), e));

    if test.should_parse {
        let (_, exposition) = exposition.unwrap();
        assert!(validate(&exposition));
    } else {
        if let Ok((_, exposition)) = exposition {
            assert!(!validate(&exposition));
        }
    }
}

fn validate(exposition: &crate::Exposition<&str>) -> bool {
    true
}
