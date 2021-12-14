use super::*;

#[test]
fn test_pad() {
    assert_eq!(format!("{:01$}", 1, 4), "0001");
}

#[test]
fn test_one_for_audiofile_ext() {
    assert_eq!(
        one_for_audiofile_ext(Path::new("/alfa/bra.vo/charlie.ogg")),
        1
    );
    assert_eq!(
        one_for_audiofile_ext(Path::new("/alfa/bra.vo/charlie.MP3")),
        1
    );
    assert_eq!(
        one_for_audiofile_ext(Path::new("/alfa/bra.vo/charlie.pdf")),
        0
    );
    assert_eq!(one_for_audiofile_ext(Path::new("/alfa/bra.vo/charlie")), 0);
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
fn test_make_initials() {
    assert_eq!(make_initials(""), "");
    assert_eq!(make_initials(" "), "");
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
    assert_eq!(make_initials("a.s.,b.s."), "A.S.,B.S.");
    assert_eq!(make_initials("A. Strugatsky, B...Strugatsky."), "A.S.,B.S.");
    assert_eq!(make_initials("Иржи Кропачек, Йозеф Новотный"), "И.К.,Й.Н.");
    assert_eq!(make_initials("Rory O'Connor"), "R.O'C.");
    assert_eq!(make_initials("Öwyn Do'Üwr"), "Ö.Do'Ü.");
    assert_eq!(make_initials("öwyn Do'üwr"), "Ö.D.");
    assert_eq!(make_initials("'"), "'.");
    assert_eq!(make_initials("Jason dinAlt"), "J.dinA.");
    assert_eq!(make_initials("DAMadar"), "DA.");
    assert_eq!(
        make_initials("johannes diderik van der waals"),
        "J.D.v.d.W."
    );
}
