use {
    anyhow::{anyhow, Context},
    itertools::Itertools,
    serde::Deserialize,
    serde_json::{Map, Value as JsonValue},
};

const SAMPLE: &str = "\
ecl:gry pid:860033327 eyr:2020 hcl:#fffffd
byr:1937 iyr:2017 cid:147 hgt:183cm

iyr:2013 ecl:amb cid:350 eyr:2023 pid:028048884
hcl:#cfa07d byr:1929

hcl:#ae17e1 iyr:2013
eyr:2024
ecl:brn pid:760753108 byr:1931
hgt:179cm

hcl:#cfa07d eyr:2025 pid:166559648
iyr:2011 ecl:brn hgt:59in
";

const INPUT: &str = include_str!("d04.txt");

fn parse_key_value_records(
    s: &str,
) -> impl Iterator<Item = anyhow::Result<Map<String, JsonValue>>> + '_ {
    s.split("\n\n").map(|e| {
        e.split_whitespace()
            .map(|kv| {
                kv.splitn(2, ':')
                    .collect_tuple::<(_, _)>()
                    .map(|(k, v)| (k.to_owned(), v.to_owned().into()))
                    .with_context(|| anyhow!(""))
            })
            .collect::<anyhow::Result<Map<_, _>>>()
    })
}

fn part_1(s: &str) -> anyhow::Result<usize> {
    #[derive(Debug, Deserialize)]
    struct RawCommonIdentityFields {
        #[serde(rename = "byr")]
        birth_year: String,
        #[serde(rename = "iyr")]
        issue_year: String,
        #[serde(rename = "eyr")]
        expiration_year: String,
        #[serde(rename = "hgt")]
        height_cm: String,
        #[serde(rename = "hcl")]
        hair_color: String,
        #[serde(rename = "ecl")]
        eye_color: String,
        #[serde(rename = "pid")]
        passport_id: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum RawIdentity {
        NorthPoleCredentials(RawCommonIdentityFields),
        Passport {
            #[serde(rename = "cid")]
            country_id: String,
            #[serde(flatten)]
            common: RawCommonIdentityFields,
        },
    }

    let parse_identity_record = |map| -> anyhow::Result<RawIdentity> {
        serde_json::from_value(JsonValue::Object(map)).context("failed to parse identity document")
    };

    parse_key_value_records(s).try_fold(0, |count, res| -> anyhow::Result<_> {
        let record = res?;
        Ok(if parse_identity_record(record).is_ok() {
            count + 1
        } else {
            count
        })
    })
}

#[test]
fn d04_p1_sample() {
    assert_eq!(part_1(SAMPLE).unwrap(), 2);
}

#[test]
fn d04_p1_answer() {
    assert_eq!(part_1(INPUT).unwrap(), 239);
}
