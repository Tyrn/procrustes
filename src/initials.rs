use itertools::join;
use regex::Regex;
use std::sync::LazyLock;
use unicode_segmentation::UnicodeSegmentation;

/// Returns a comma-separated list of initials,
/// `authors` being a comma-separated list of full names.
///
/// ```
/// use procrustes::initials::initials;
///
/// assert_eq!(initials("Jason dinAlt"), "J.dinA.");
/// assert_eq!(initials("Jackie McGee"), "J.McG.");
/// assert_eq!(initials("Ross Macdonald"), "R.M.");
/// assert_eq!(initials("Madonna"), "M.");
/// assert_eq!(initials(""), "");
/// ```
pub fn initials(authors: &str) -> String {
    static SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s.]+").unwrap());
    static NICKNAME: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#""(?:\\.|[^"\\])*""#).unwrap());
    static NOBILIARY_PARTICLES: [&str; 47] = [
        "von", "фон", "van", "ван", "der", "дер", "til", "тиль", "zu", "цу", "zum", "цум", "zur",
        "цур", "af", "аф", "of", "из", "da", "да", "de", "де", "des", "дез", "del", "дель", "den",
        "ден", "di", "ди", "dos", "душ", "дос", "du", "дю", "la", "ла", "ля", "le", "ле", "las",
        "лас", "los", "лос", "haut", "от", "the",
    ];

    fn gv(s: &str) -> Vec<&str> {
        UnicodeSegmentation::graphemes(s, true).collect()
    }

    /// Converts `name` to its initial. Mostly by keeping the first character
    /// and dropping the rest; deals with special cases, too. See the unit test.
    ///
    fn initial(name: &str) -> String {
        let cut: Vec<&str> = name.split('\'').collect();

        if cut.len() > 1 && !cut[1].is_empty() {
            // Deal with '.
            if cut[1].chars().next().unwrap().is_lowercase() && !cut[0].is_empty() {
                return gv(cut[0])[0].to_uppercase();
            }
            return cut[0].to_owned() + "'" + gv(cut[1])[0];
        }

        let v = gv(name);
        let mut v_iter = v.iter();

        if v.len() > 1 {
            // Deal with prefixes.
            match name {
                "Старший" => return "Ст".to_string(),
                "Младший" => return "Мл".to_string(),
                "Ст" | "ст" | "Sr" | "Мл" | "мл" | "Jr" => return name.to_string(),
                _ => (),
            };
            let mut prefix: Vec<&str> = vec![*v_iter.next().unwrap()];
            for vch in v_iter {
                prefix.push(vch);
                if vch.chars().next().unwrap().is_uppercase() {
                    return prefix.concat();
                }
            }
        }

        if v[0].chars().next().unwrap().is_lowercase() && NOBILIARY_PARTICLES.contains(&name) {
            return v[0].to_string();
        }

        v[0].to_uppercase()
    }

    join(
        NICKNAME
            .replace_all(authors, " ")
            .replace('"', " ")
            .split(',')
            .filter(|author| author.replace(['.', '-'], "").trim() != "")
            .map(|author| {
                [
                    join(
                        author
                            .split('-')
                            .filter(|barrel| barrel.replace('.', "").trim() != "")
                            .map(|barrel| {
                                join(
                                    SPACE
                                        .split(barrel)
                                        .filter(|name| !name.is_empty())
                                        .map(initial),
                                    ".",
                                )
                            }),
                        "-",
                    ),
                    ".".to_string(),
                ]
                .concat()
            }),
        ",",
    )
}
