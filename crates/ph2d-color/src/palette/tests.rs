//! Palette interchange: round-trips for every format + parsing of hand-written real-world samples.

use super::{PaletteData, PaletteFormat, parse, write};

fn sample() -> PaletteData {
    PaletteData {
        name: "Sunset".to_string(),
        colors: vec![
            [231, 76, 60, 255],
            [241, 196, 15, 255],
            [52, 152, 219, 255],
            [0, 0, 0, 255],
        ],
    }
}

/// Every format round-trips the RGB swatches (names/alpha vary by format, so compare RGB only).
#[test]
fn every_format_round_trips_rgb() {
    for fmt in PaletteFormat::ALL {
        let p = sample();
        let bytes = write(fmt, &p);
        let back = parse(fmt, &bytes).unwrap_or_else(|e| panic!("{fmt:?} parse: {e}"));
        let want: Vec<[u8; 3]> = p.colors.iter().map(|c| [c[0], c[1], c[2]]).collect();
        let got: Vec<[u8; 3]> = back.colors.iter().map(|c| [c[0], c[1], c[2]]).collect();
        assert_eq!(got, want, "{fmt:?} must preserve the RGB swatches");
    }
}

/// `.gpl` and the hex list keep the palette name / alpha where the format allows it.
#[test]
fn gpl_keeps_name_hex_keeps_alpha() {
    let g = parse(PaletteFormat::Gpl, &write(PaletteFormat::Gpl, &sample())).unwrap();
    assert_eq!(g.name, "Sunset", ".gpl carries the palette name");

    let translucent = PaletteData {
        name: String::new(),
        colors: vec![[10, 20, 30, 128], [40, 50, 60, 255]],
    };
    let h = parse(
        PaletteFormat::HexList,
        &write(PaletteFormat::HexList, &translucent),
    )
    .unwrap();
    assert_eq!(
        h.colors, translucent.colors,
        "the hex list round-trips 8-digit alpha"
    );
}

/// A real GIMP `.gpl` (the shape GIMP / Inkscape / Krita / Blender emit) parses.
#[test]
fn parses_a_real_gimp_gpl() {
    let src = "GIMP Palette\nName: My Colors\nColumns: 4\n#\n255   0   0\tRed\n  0 128   0\tGreen\n\
               0   0 255\tBlue\n";
    let p = parse(PaletteFormat::Gpl, src.as_bytes()).expect("gpl");
    assert_eq!(p.name, "My Colors");
    assert_eq!(
        p.colors,
        vec![[255, 0, 0, 255], [0, 128, 0, 255], [0, 0, 255, 255]]
    );
}

/// A coolors.co-style hex dump: bare + `#`-prefixed, 3/6/8-digit, with comments and trailing text.
#[test]
fn parses_a_coolors_hex_dump() {
    let src = "; my palette\n#E74C3C\nf1c40f\n#3498DBFF\n#FFF\n// a note\n#1a1a1a  dark\n";
    let p = parse(PaletteFormat::HexList, src.as_bytes()).expect("hex");
    assert_eq!(
        p.colors,
        vec![
            [231, 76, 60, 255],
            [241, 196, 15, 255],
            [52, 152, 219, 255],
            [255, 255, 255, 255], // #FFF → #FFFFFF
            [26, 26, 26, 255],
        ]
    );
}

/// Malformed input is rejected, not silently empty.
#[test]
fn rejects_garbage() {
    assert!(parse(PaletteFormat::Gpl, b"not a palette").is_err());
    assert!(parse(PaletteFormat::HexList, b"no colours here\njust prose\n").is_err());
    assert!(parse(PaletteFormat::Ase, b"XXXX\x00\x01").is_err());
    assert!(parse(PaletteFormat::Aco, b"\x00\x09\x00\x01").is_err()); // version 9
}

/// The `.ase` byte stream begins with the `ASEF` signature + one block per colour.
#[test]
fn ase_has_signature_and_block_count() {
    let bytes = write(PaletteFormat::Ase, &sample());
    assert_eq!(&bytes[0..4], b"ASEF", "ASE signature");
    assert_eq!(&bytes[8..12], &4u32.to_be_bytes(), "one block per swatch");
}

/// The `.aco` byte stream opens with a v1 header (version 1 + count) and carries a v2 section.
#[test]
fn aco_has_v1_and_v2_sections() {
    let bytes = write(PaletteFormat::Aco, &sample());
    assert_eq!(&bytes[0..2], &1u16.to_be_bytes(), "ACO v1 version");
    assert_eq!(&bytes[2..4], &4u16.to_be_bytes(), "ACO v1 count");
    // v1 body = 4 colours × 10 bytes = 40, so the v2 header (version 2) starts at byte 44.
    assert_eq!(
        &bytes[44..46],
        &2u16.to_be_bytes(),
        "ACO v2 section follows v1"
    );
}

/// Extension → format mapping (import filters / export menu).
#[test]
fn format_from_extension() {
    assert_eq!(
        PaletteFormat::from_extension(".GPL"),
        Some(PaletteFormat::Gpl)
    );
    assert_eq!(
        PaletteFormat::from_extension("aco"),
        Some(PaletteFormat::Aco)
    );
    assert_eq!(
        PaletteFormat::from_extension("css"),
        Some(PaletteFormat::HexList)
    );
    assert_eq!(PaletteFormat::from_extension("png"), None);
}
