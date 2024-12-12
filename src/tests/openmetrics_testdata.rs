use nom::combinator::complete;
use nom::error::VerboseError;
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

    let name = path.file_name().unwrap().to_str().unwrap();
    let should_parse = test.should_parse
        || name.starts_with("bad_clashing_names_")
        || name.starts_with("bad_counter_values_")
        || name.starts_with("bad_exemplars_on_unallowed_")
        || name.starts_with("bad_grouping_or_ordering_")
        || name.starts_with("bad_histograms_")
        || name.starts_with("bad_metadata_in_wrong_place")
        || name.starts_with("bad_missing_or_invalid_labels_for_a_type_");

    if should_parse {
        complete::<_, _, VerboseError<_>, _>(crate::exposition)(input.as_str()).unwrap();
    } else {
        complete::<_, _, VerboseError<_>, _>(crate::exposition)(input.as_str()).unwrap_err();
    }
}
