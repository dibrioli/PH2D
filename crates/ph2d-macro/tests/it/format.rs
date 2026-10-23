//! O formato versionado (plano §5, gate 5). Os bytes destas fixturas estão CONGELADOS à mão:
//! nenhum é produzido pelo escritor, para que uma mudança no escritor não as arraste consigo.

use ph2d_macro::codec::{self, CodecError};
use ph2d_macro::{Event, MacroDoc, MacroTime, RefRect, Track};

fn cat(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

/// O documento que as duas fixturas descrevem.
fn expected() -> MacroDoc {
    MacroDoc {
        reference: RefRect { w: 2.0, h: 4.0 },
        tracks: vec![Track {
            family: "p".into(),
            target: 7,
            initial: vec![9],
            events: vec![Event {
                t: MacroTime(1000),
                payload: vec![1, 2],
            }],
            edicoes: vec![],
        }],
    }
}

/// v1.0, escrito à mão: reference (2.0, 4.0) · uma trilha `p`, alvo 7, inicial [9] ·
/// um passo em t = 1000 µs com carga [1, 2].
fn v1_0() -> Vec<u8> {
    let event = [0x01, 0x02, 0xE8, 0x07, 0x02, 0x02, 0x01, 0x02, 0x00];
    let track = cat(&[
        &[
            0x01, 0x01, b'p', 0x02, 0x01, 0x07, 0x03, 0x01, 0x09, 0x04, 0x09,
        ],
        &event,
        &[0x00],
    ]);
    assert_eq!(track.len(), 0x15);
    cat(&[
        b"PH2DMACRO",
        &[0x01, 0x00, 0x00, 0x00],
        &[0x01, 0x08, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x80, 0x40],
        &[0x02, 0x15],
        &track,
        &[0x00],
    ])
}

/// v1.9 (o futuro dentro da mesma major): o mesmo documento com um campo DESCONHECIDO em
/// cada nível — documento (tag 9), trilha (tag 7) e passo (tag 5).
fn v1_9_com_campos_desconhecidos() -> Vec<u8> {
    let event = [
        0x01, 0x02, 0xE8, 0x07, 0x02, 0x02, 0x01, 0x02, 0x05, 0x01, 0xFF, 0x00,
    ];
    let track = cat(&[
        &[0x01, 0x01, b'p', 0x02, 0x01, 0x07, 0x03, 0x01, 0x09],
        &[0x07, 0x02, 0xAB, 0xCD],
        &[0x04, 0x0C],
        &event,
        &[0x00],
    ]);
    assert_eq!(track.len(), 0x1C);
    cat(&[
        b"PH2DMACRO",
        &[0x01, 0x00, 0x09, 0x00],
        &[0x01, 0x08, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x80, 0x40],
        &[0x09, 0x03, 0xAA, 0xBB, 0xCC],
        &[0x02, 0x1C],
        &track,
        &[0x00],
    ])
}

#[test]
fn a_frozen_v1_0_file_reads_and_the_writer_reproduces_it_to_the_byte() {
    assert_eq!(codec::read(&v1_0()), Ok(expected()));
    assert_eq!(codec::write(&expected()), v1_0());
}

#[test]
fn a_newer_minor_with_unknown_fields_at_every_level_still_reads() {
    assert_eq!(
        codec::read(&v1_9_com_campos_desconhecidos()),
        Ok(expected())
    );
}

#[test]
fn another_major_is_refused_never_guessed() {
    let mut b = v1_0();
    b[9] = 0x02;
    assert_eq!(
        codec::read(&b),
        Err(CodecError::MajorMismatch {
            found: 2,
            supported: 1
        })
    );
}

#[test]
fn a_file_that_is_not_a_macro_is_refused() {
    assert_eq!(codec::read(b"PH2DPROJ...."), Err(CodecError::NotAMacro));
}

#[test]
fn every_truncation_is_an_error_never_a_shorter_document() {
    let b = v1_0();
    for n in 0..b.len() {
        assert!(codec::read(&b[..n]).is_err(), "cortado em {n} leu-se");
    }
}

#[test]
fn a_track_without_family_is_refused() {
    let track = [0x02, 0x01, 0x07, 0x00];
    let b = cat(&[
        b"PH2DMACRO",
        &[0x01, 0x00, 0x00, 0x00],
        &[0x01, 0x08, 0, 0, 0, 0x40, 0, 0, 0x80, 0x40],
        &[0x02, 0x04],
        &track,
        &[0x00],
    ]);
    assert_eq!(
        codec::read(&b),
        Err(CodecError::MissingField {
            record: "Track",
            tag: 1
        })
    );
}

#[test]
fn time_going_backwards_inside_a_track_is_refused() {
    let mut doc = expected();
    doc.tracks[0].events.push(Event {
        t: MacroTime(999),
        payload: vec![],
    });
    assert_eq!(
        codec::read(&codec::write(&doc)),
        Err(CodecError::NonMonotonicTime { track: 0 })
    );
}

#[test]
fn a_rich_document_round_trips_to_the_byte() {
    let doc = MacroDoc {
        reference: RefRect {
            w: 1024.0,
            h: 768.5,
        },
        tracks: vec![
            Track {
                family: "painter".into(),
                target: u64::MAX,
                initial: (0..=255).collect(),
                events: (0..300)
                    .map(|i| Event {
                        t: MacroTime(i * 16_667),
                        payload: vec![i as u8; (i % 7) as usize],
                    })
                    .collect(),
                edicoes: vec![],
            },
            Track {
                family: "motion".into(),
                target: 0,
                initial: vec![],
                events: vec![],
                edicoes: vec![],
            },
        ],
    };
    // As edições de uma etapa, todas as etiquetas — a 8 (o texto do balão do tutorial) e a 9 (W16) inclusive.
    let mut doc = doc;
    doc.tracks[0].edicoes = vec![ph2d_macro::EdicaoDeEtapa {
        ancora: 3,
        nome: Some("Céu".into()),
        dica: Some("Ligue a caixa ao vento".into()),
        espera_us: -250,
        trocas: vec![(4, vec![1, 2])],
        insercoes: vec![(5, vec![3])],
        sai_da_gravacao: true,
        apagada: true,
        desloca_us: -1_500,
    }];
    let bytes = codec::write(&doc);
    let back = codec::read(&bytes).expect("lê");
    assert_eq!(back, doc);
    assert_eq!(codec::write(&back), bytes);
}
