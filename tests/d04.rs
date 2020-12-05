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

#[derive(Debug, Deserialize)]
struct RawCommonIdentityFields {
    #[serde(rename = "byr")]
    birth_year: String,
    #[serde(rename = "iyr")]
    issue_year: String,
    #[serde(rename = "eyr")]
    expiration_year: String,
    #[serde(rename = "hgt")]
    height: String,
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

fn parse_identity_record(map: Map<String, JsonValue>) -> anyhow::Result<RawIdentity> {
    serde_json::from_value(JsonValue::Object(map)).context("failed to parse identity document")
}

fn count_records<F>(s: &str, mut f: F) -> anyhow::Result<usize>
where
    F: FnMut(Map<String, JsonValue>) -> bool,
{
    parse_key_value_records(s).try_fold(0, |count, res| -> anyhow::Result<_> {
        let record = res?;
        Ok(if f(record) { count + 1 } else { count })
    })
}

fn part_1(s: &str) -> anyhow::Result<usize> {
    count_records(s, |record| parse_identity_record(record).is_ok())
}

fn validate_birth_year(birth_year: &str) -> bool {
    birth_year
        .parse::<u16>()
        .ok()
        .filter(|&by| by >= 1920 && by <= 2002)
        .is_some()
}

fn validate_height(height: &str) -> bool {
    height
        .strip_suffix("cm")
        .and_then(|cm| cm.parse::<u8>().ok())
        .filter(|&cm| cm >= 150 && cm <= 193)
        .is_some()
        || height
            .strip_suffix("in")
            .and_then(|ins| ins.parse::<u8>().ok())
            .filter(|&ins| ins >= 59 && ins <= 76)
            .is_some()
}

fn validate_hair_color(hair_color: &str) -> bool {
    hair_color
        .strip_prefix('#')
        .filter(|hc| hc.len() == 6 && hc.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')))
        .is_some()
}

fn validate_eye_color(eye_color: &str) -> bool {
    matches!(
        &*eye_color,
        "amb" | "blu" | "brn" | "gry" | "grn" | "hzl" | "oth"
    )
}

fn validate_passport_id(passport_id: &str) -> bool {
    passport_id.len() == 9 && passport_id.chars().all(|c| c.is_ascii_digit())
}

fn validate_common_identity_fields(common: &RawCommonIdentityFields) -> bool {
    let RawCommonIdentityFields {
        birth_year,
        issue_year,
        expiration_year,
        height,
        hair_color,
        eye_color,
        passport_id,
    } = common;

    validate_birth_year(&birth_year)
        && issue_year
            .parse::<u16>()
            .ok()
            .filter(|&iy| iy >= 2010 && iy <= 2020)
            .is_some()
        && expiration_year
            .parse::<u16>()
            .ok()
            .filter(|&ey| ey >= 2020 && ey <= 2030)
            .is_some()
        && validate_height(&height)
        && validate_hair_color(&hair_color)
        && validate_eye_color(&eye_color)
        && validate_passport_id(&passport_id)
}

fn part_2(s: &str) -> anyhow::Result<usize> {
    count_records(s, |record| {
        parse_identity_record(record).map_or(false, |identity| match identity {
            RawIdentity::NorthPoleCredentials(common)
            | RawIdentity::Passport {
                country_id: _,
                common,
            } => validate_common_identity_fields(&common),
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

#[test]
fn d04_p2_sample() {
    assert!(validate_birth_year("2002"));
    assert!(!validate_birth_year("2003"));

    assert!(validate_height("60in"));
    assert!(validate_height("190cm"));
    assert!(!validate_height("190in"));
    assert!(!validate_height("190"));

    assert!(validate_hair_color("#123abc"));
    assert!(!validate_hair_color("#123abz"));
    assert!(!validate_hair_color("123abc"));

    assert!(validate_eye_color("brn"));
    assert!(!validate_eye_color("wat"));

    assert!(validate_passport_id("000000001"));
    assert!(!validate_passport_id("0123456789"));

    assert!(parse_key_value_records(
        "\
eyr:1972 cid:100
hcl:#18171d ecl:amb hgt:170 pid:186cm iyr:2018 byr:1926

iyr:2019
hcl:#602927 eyr:1967 hgt:170cm
ecl:grn pid:012533040 byr:1946

hcl:dab227 iyr:2012
ecl:brn hgt:182cm pid:021572410 eyr:2020 byr:1992 cid:277

hgt:59cm ecl:zzz
eyr:2038 hcl:74454a iyr:2023
pid:3556412378 byr:2007
"
    )
    .all(|res| {
        let record = res.unwrap();
        match parse_identity_record(record).unwrap() {
            RawIdentity::NorthPoleCredentials(common)
            | RawIdentity::Passport {
                common,
                country_id: _,
            } => !validate_common_identity_fields(&common),
        }
    }));

    assert!(parse_key_value_records(
        "\
pid:087499704 hgt:74in ecl:grn iyr:2012 eyr:2030 byr:1980
hcl:#623a2f

eyr:2029 ecl:blu cid:129 byr:1989
iyr:2014 pid:896056539 hcl:#a97842 hgt:165cm

hcl:#888785
hgt:164cm byr:2001 iyr:2015 cid:88
pid:545766238 ecl:hzl
eyr:2022

iyr:2010 hgt:158cm hcl:#b6652a ecl:blu byr:1944 eyr:2021 pid:093154719
"
    )
    .all(|res| {
        let record = res.unwrap();
        match parse_identity_record(record).unwrap() {
            RawIdentity::NorthPoleCredentials(common)
            | RawIdentity::Passport {
                common,
                country_id: _,
            } => validate_common_identity_fields(&common),
        }
    }));
}

#[test]
fn d04_p2_answer() {
    assert_eq!(part_2(INPUT).unwrap(), 188);
}
