//! W19 (`docs/Macros/17_W19_plano.md` §2.7): a família «mundo» sobe o MINOR e exige-se pelo nome.

use ph2d_macro::codec::{self, CodecError, EXIGE_MUNDO, FAMILIA_MUNDO};
use ph2d_macro::{Event, MacroDoc, MacroTime, RefRect, Track};

fn trilha(family: &str) -> Track {
    Track {
        family: family.into(),
        target: 0,
        initial: vec![1],
        events: vec![Event {
            t: MacroTime(10),
            payload: vec![2],
        }],
        edicoes: vec![],
        nasce: None,
        continua: None,
    }
}

fn minor(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[11], bytes[12]])
}

fn exige_mundo(bytes: &[u8]) -> bool {
    let campo = [&[0x03, EXIGE_MUNDO.len() as u8][..], EXIGE_MUNDO.as_bytes()].concat();
    bytes.windows(campo.len()).any(|w| w == campo)
}

#[test]
fn um_leitor_1_3_recusa_o_mundo_em_voz_alta() {
    let doc = MacroDoc {
        reference: RefRect { w: 1.0, h: 1.0 },
        tracks: vec![trilha("painter"), trilha(FAMILIA_MUNDO)],
    };
    let bytes = codec::write(&doc);
    assert_eq!(minor(&bytes), 4);
    assert!(exige_mundo(&bytes), "sem o `exige` um leitor 1.3 saltava a trilha calado");
    assert_eq!(codec::read(&bytes).expect("este leitor sabe"), doc);
    // O leitor 1.3: o mesmo campo com um nome que ele não sabe.
    let antigo: Vec<u8> = {
        let campo = [&[0x03, 5][..], EXIGE_MUNDO.as_bytes()].concat();
        let i = bytes
            .windows(campo.len())
            .position(|w| w == campo)
            .expect("o campo exige")
            + 2;
        let mut b = bytes.clone();
        b[i..i + 5].copy_from_slice(b"mundX");
        b
    };
    assert_eq!(codec::read(&antigo), Err(CodecError::Exige("mundX".into())));
}

#[test]
fn sem_trilha_mundo_nao_sobe_o_minor() {
    let doc = MacroDoc {
        reference: RefRect { w: 1.0, h: 1.0 },
        tracks: vec![trilha("painter")],
    };
    let bytes = codec::write(&doc);
    assert_eq!(minor(&bytes), 0);
    assert!(!exige_mundo(&bytes));
}

#[test]
fn a_trilha_que_continua_outra_vai_e_volta_e_exige_se() {
    let mut volta = trilha("painter");
    volta.target = 9;
    volta.continua = Some(0);
    let doc = MacroDoc {
        reference: RefRect { w: 1.0, h: 1.0 },
        tracks: vec![trilha("painter"), volta],
    };
    let bytes = codec::write(&doc);
    assert_eq!(minor(&bytes), 4);
    let nome = codec::EXIGE_CONTINUA.as_bytes();
    assert!(bytes.windows(nome.len()).any(|w| w == nome));
    assert_eq!(codec::read(&bytes).expect("lê"), doc);
}
