use super::*;

#[test]
fn test_format() {
    assert_eq!(format!("{:01$}", 1, 4), "0001");
    assert_eq!(format!("{:+}", 4), "+4");
    assert_eq!(format!("{:+}", -4), "-4");
}

#[test]
fn test_std_path() {
    let sep = std::path::MAIN_SEPARATOR;
    let a1 = PathBuf::from(format!("{}{}{}", "alfa", sep, "bravo"));
    let a11 = PathBuf::from("alfa".to_owned() + &sep.to_string() + "bravo");
    let a2: PathBuf = [PathBuf::from("alfa"), PathBuf::from("bravo")]
        .iter()
        .collect();
    let mut a3 = PathBuf::from("alfa");
    a3.push("bravo");

    assert_eq!(a1, a11);
    assert_eq!(a1, a2);
    assert_eq!(a1, a3);
}

#[test]
fn test_string() {
    let simp = "The Good Soldier SvejkThe Good Soldier Svejk";
    let diac = "Osudy dobrého vojáka Švejka za světové války";

    assert_eq!(simp.len(), 44);
    assert_eq!(diac.len(), 50);
    assert_eq!(diac.chars().count(), 44);
    assert_eq!(diac.graphemes(true).count(), 44);

    let (head, _) = diac.split_at(10);
    assert_eq!(head, "Osudy dobr");

    assert_eq!(diac.is_char_boundary(11), false);

    // let (head, _) = diac.split_at(11);
    // assert_eq!(head, "Osudy dobre");

    // There is a way to violate a string:
    let (head, _) = diac.as_bytes().split_at(11);
    assert_eq!(String::from_utf8_lossy(head), "Osudy dobr�");

    let (head, _) = diac.split_at(12);
    assert_eq!(head, "Osudy dobré");

    // Magnificent and irresistible!
    assert_eq!(diac.get(..12).unwrap(), "Osudy dobré");
    assert_eq!(diac.get(33..).unwrap(), " světové války");

    // Now on Vec<char>:
    let vecd = diac.chars().collect::<Vec<char>>();

    let (head, _) = vecd.split_at(10);
    assert_eq!(head.into_iter().collect::<String>(), "Osudy dobr");

    let (head, _) = vecd.split_at(11);
    assert_eq!(head.into_iter().collect::<String>(), "Osudy dobré");
}

#[test]
fn test_str_shrink() {
    let lat1 = "The quick brown fox jumps over the lazy dog!";
    let cyr1 = "Однажды играли в карты у конногвардейца На..";

    assert_eq!(lat1.len(), 44);
    assert_eq!(cyr1.len(), 80);

    assert_eq!(cyr1.chars().collect::<Vec<char>>().len(), 44);

    assert_eq!(str_shrink(cyr1, 10), "Однаж ⚡ На..");
    assert_eq!(str_shrink(lat1, 20), "The quick ⚡ lazy dog!");
    assert_eq!(str_shrink(cyr1, 20), "Однажды иг ⚡ дейца На..");
    assert_eq!(str_shrink(lat1, 30), "The quick brown ⚡ r the lazy dog!");
    assert_eq!(str_shrink(cyr1, 30), "Однажды играли ⚡ огвардейца На..");
    assert_eq!(
        str_shrink(lat1, 43),
        "The quick brown fox j ⚡ ps over the lazy dog!"
    );
    assert_eq!(
        str_shrink(cyr1, 43),
        "Однажды играли в карт ⚡ у конногвардейца На.."
    );
    assert_eq!(str_shrink(lat1, 44), lat1);
    assert_eq!(str_shrink(cyr1, 44), cyr1);
}

#[test]
fn test_is_audiofile_ext() {
    assert!(is_audiofile_ext(Path::new("charlie.ogg")));
    assert!(is_audiofile_ext(Path::new("charlie.MP3")));
    assert!(!is_audiofile_ext(Path::new("charlie.pdf")));
    assert!(!is_audiofile_ext(Path::new("charlie")));
}

#[test]
fn test_has_ext_of() {
    assert_eq!(has_ext_of("charlie.ogg", "OGG"), true);
    assert_eq!(has_ext_of("charlie.ogg", ".ogg"), true);
    assert_eq!(has_ext_of("charlie.ogg", "mp3"), false);
}

#[test]
fn test_str_strip_numbers() {
    assert_eq!(str_strip_numbers("ab11cdd2k.144"), vec![11, 2, 144]);
    assert!(str_strip_numbers("Ignacio Vazquez-Abrams").is_empty());
}

#[test]
fn test_human_fine() {
    assert_eq!(human_fine(0), "0");
    assert_eq!(human_fine(1), "1");
    assert_eq!(human_fine(42), "42");
    assert_eq!(human_fine(1800), "2kB");
    assert_eq!(human_fine(123456789), "117.7MB");
    assert_eq!(human_fine(123456789123), "114.98GB");
    assert_eq!(human_fine(1024), "1kB");
    assert_eq!(human_fine(1024.0_f64.powi(2) as u64), "1.0MB");
    assert_eq!(human_fine(1024.0_f64.powi(3) as u64), "1.00GB");
    assert_eq!(human_fine(1024.0_f64.powi(4) as u64), "1.00TB");
}

#[test]
fn test_initials() {
    assert_eq!(initials(""), "");
    assert_eq!(initials(" "), "");
    assert_eq!(initials(".. , .. "), "");
    assert_eq!(initials(" ,, .,"), "");
    assert_eq!(initials(", a. g, "), "A.G.");
    assert_eq!(initials("- , -I.V.-A,E.C.N-, ."), "I.V-A.,E.C.N.");
    assert_eq!(initials("John ronald reuel Tolkien"), "J.R.R.T.");
    assert_eq!(initials("  e.B.Sledge "), "E.B.S.");
    assert_eq!(initials("Apsley Cherry-Garrard"), "A.C-G.");
    assert_eq!(initials("Windsor Saxe-\tCoburg - Gotha"), "W.S-C-G.");
    assert_eq!(initials("Elisabeth Kubler-- - Ross"), "E.K-R.");
    assert_eq!(initials("  Fitz-Simmons Ashton-Burke Leigh"), "F-S.A-B.L.");
    assert_eq!(initials("Arleigh \"31-knot\"Burke "), "A.B.");
    assert_eq!(
        initials("Harry \"Bing\" Crosby, Kris \"Tanto\" Paronto"),
        "H.C.,K.P."
    );
    assert_eq!(
        initials("William J. \"Wild Bill\" Donovan, Marta \"Cinta Gonzalez"),
        "W.J.D.,M.C.G."
    );
    assert_eq!(initials("язон динАльт, шарль д'Артаньян"), "Я.динА.,Ш.д'А.");
    assert_eq!(initials("шарль д'артаньян"), "Ш.Д.");
    assert_eq!(
        initials("Charles de Batz de Castelmore d'Artagnan"),
        "C.d.B.d.C.d'A."
    );
    assert_eq!(
        initials("Mario Del Monaco, Hutchinson of London"),
        "M.D.M.,H.o.L."
    );
    assert_eq!(initials("Anselm haut Rodric"), "A.h.R.");
    assert_eq!(initials("Ансельм от Родрик"), "А.о.Р.");
    assert_eq!(initials("Leonardo Wilhelm DiCaprio"), "L.W.DiC.");
    assert_eq!(initials("леонардо вильгельм ди каприо"), "Л.В.д.К.");
    assert_eq!(initials("kapitän zur see"), "K.z.S.");
    assert_eq!(initials("De Beers, Guido van Rossum"), "D.B.,G.v.R.");
    assert_eq!(initials("Манфред фон Рихтгофен"), "М.ф.Р.");
    assert_eq!(initials("Armand Jean du Plessis"), "A.J.d.P.");
    assert_eq!(initials("a.s.,b.s."), "A.S.,B.S.");
    assert_eq!(initials("A. Strugatsky, B...Strugatsky."), "A.S.,B.S.");
    assert_eq!(initials("Иржи Кропачек,, йозеф Новотный"), "И.К.,Й.Н.");
    assert_eq!(initials("Rory O'Connor"), "R.O'C.");
    assert_eq!(initials("Öwyn Do'Üwr"), "Ö.Do'Ü.");
    assert_eq!(initials("öwyn Do'üwr"), "Ö.D.");
    assert_eq!(initials("'"), "'.");
    assert_eq!(initials("Jason dinAlt"), "J.dinA.");
    assert_eq!(initials("Jackie McGee"), "J.McG.");
    assert_eq!(initials("Ross Macdonald"), "R.M.");
    assert_eq!(initials("DAMadar"), "DA.");
    assert_eq!(initials("johannes diderik van der waals"), "J.D.v.d.W.");
}
