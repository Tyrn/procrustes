use super::*;

#[test]
fn test_pad() {
    assert_eq!(format!("{:01$}", 1, 4), "0001");
}

#[test]
fn test_is_audiofile_ext() {
    assert!(is_audiofile_ext(Path::new("/alfa/bra.vo/charlie.ogg")));
    assert!(is_audiofile_ext(Path::new("/alfa/bra.vo/charlie.MP3")));
    assert!(!is_audiofile_ext(Path::new("/alfa/bra.vo/charlie.pdf")));
    assert!(!is_audiofile_ext(Path::new("/alfa/bra.vo/charlie")));
}

#[test]
fn test_has_ext_of() {
    assert_eq!(has_ext_of("/alfa/bra.vo/charlie.ogg", "OGG"), true);
    assert_eq!(has_ext_of("/alfa/bra.vo/charlie.ogg", ".ogg"), true);
    assert_eq!(has_ext_of("/alfa/bra.vo/charlie.ogg", "mp3"), false);
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
fn test_make_initials() {
    assert_eq!(make_initials(""), "");
    assert_eq!(make_initials(" "), "");
    assert_eq!(make_initials(".. , .. "), "");
    assert_eq!(make_initials(" ,, .,"), "");
    assert_eq!(make_initials(", a. g, "), "A.G.");
    assert_eq!(make_initials("- , -I.V.-A,E.C.N-, ."), "I.V-A.,E.C.N.");
    assert_eq!(make_initials("John ronald reuel Tolkien"), "J.R.R.T.");
    assert_eq!(make_initials("  e.B.Sledge "), "E.B.S.");
    assert_eq!(make_initials("Apsley Cherry-Garrard"), "A.C-G.");
    assert_eq!(make_initials("Windsor Saxe-\tCoburg - Gotha"), "W.S-C-G.");
    assert_eq!(make_initials("Elisabeth Kubler-- - Ross"), "E.K-R.");
    assert_eq!(
        make_initials("  Fitz-Simmons Ashton-Burke Leigh"),
        "F-S.A-B.L."
    );
    assert_eq!(make_initials("Arleigh \"31-knot\"Burke "), "A.B.");
    assert_eq!(
        make_initials("Harry \"Bing\" Crosby, Kris \"Tanto\" Paronto"),
        "H.C.,K.P."
    );
    assert_eq!(
        make_initials("William J. \"Wild Bill\" Donovan, Marta \"Cinta Gonzalez"),
        "W.J.D.,M.C.G."
    );
    assert_eq!(
        make_initials("язон динАльт, шарль д'Артаньян"),
        "Я.динА.,Ш.д'А."
    );
    assert_eq!(make_initials("шарль д'артаньян"), "Ш.Д.");
    assert_eq!(
        make_initials("Charles de Batz de Castelmore d'Artagnan"),
        "C.d.B.d.C.d'A."
    );
    assert_eq!(
        make_initials("Mario Del Monaco, Hutchinson of London"),
        "M.D.M.,H.o.L."
    );
    assert_eq!(make_initials("Anselm haut Rodric"), "A.h.R.");
    assert_eq!(make_initials("Ансельм от Родрик"), "А.о.Р.");
    assert_eq!(make_initials("Leonardo Wilhelm DiCaprio"), "L.W.DiC.");
    assert_eq!(make_initials("De Beers, Guido van Rossum"), "D.B.,G.v.R.");
    assert_eq!(make_initials("Манфред фон Рихтгофен"), "М.ф.Р.");
    assert_eq!(make_initials("Armand Jean du Plessis"), "A.J.d.P.");
    assert_eq!(make_initials("a.s.,b.s."), "A.S.,B.S.");
    assert_eq!(make_initials("A. Strugatsky, B...Strugatsky."), "A.S.,B.S.");
    assert_eq!(make_initials("Иржи Кропачек,, йозеф Новотный"), "И.К.,Й.Н.");
    assert_eq!(make_initials("Rory O'Connor"), "R.O'C.");
    assert_eq!(make_initials("Öwyn Do'Üwr"), "Ö.Do'Ü.");
    assert_eq!(make_initials("öwyn Do'üwr"), "Ö.D.");
    assert_eq!(make_initials("'"), "'.");
    assert_eq!(make_initials("Jason dinAlt"), "J.dinA.");
    assert_eq!(make_initials("Jackie McGee"), "J.McG.");
    assert_eq!(make_initials("Ross Macdonald"), "R.M.");
    assert_eq!(make_initials("DAMadar"), "DA.");
    assert_eq!(
        make_initials("johannes diderik van der waals"),
        "J.D.v.d.W."
    );
}
