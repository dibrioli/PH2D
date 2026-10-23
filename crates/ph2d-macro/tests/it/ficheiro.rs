//! W18 (`docs/Macros/16_W18_plano.md` §2.4–2.5): a ficha de nascimento e o ficheiro `.ph2dmacro`.

use ph2d_macro::codec::{self, CodecError};
use ph2d_macro::ficheiro::{self, Ficheiro};
use ph2d_macro::{Event, MacroDoc, MacroTime, Nascimento, RefRect, Track};

fn trilha(nasce: Option<Nascimento>) -> Track {
    Track {
        family: "p".into(),
        target: 7,
        initial: vec![9],
        events: vec![Event {
            t: MacroTime(1000),
            payload: vec![1, 2],
        }],
        edicoes: vec![],
        nasce,
    }
}

fn ficha() -> Nascimento {
    Nascimento {
        inicio: MacroTime(250),
        nome: "Tela".into(),
        local: [1.5, -2.0, 0.25, 2.0, 0.5, 0.0, 0.125],
        pai: Some(0),
    }
}

fn minor(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[11], bytes[12]])
}

#[test]
fn a_ficha_de_nascimento_tem_os_bytes_congelados() {
    let doc = MacroDoc {
        reference: RefRect { w: 2.0, h: 4.0 },
        tracks: vec![trilha(Some(Nascimento {
            inicio: MacroTime(300),
            nome: "a".into(),
            local: [0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
            pai: None,
        }))],
    };
    let bytes = codec::write(&doc);
    let mut nasce = vec![0x01, 0x02, 0xAC, 0x02, 0x02, 0x01, b'a', 0x03, 28];
    for v in [0.0f32, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0] {
        nasce.extend_from_slice(&v.to_le_bytes());
    }
    nasce.push(0x00);
    let mut esperado_na_trilha = vec![0x08, nasce.len() as u8];
    esperado_na_trilha.extend_from_slice(&nasce);
    assert!(
        bytes
            .windows(esperado_na_trilha.len())
            .any(|w| w == esperado_na_trilha),
        "a etiqueta 8 da trilha mudou de forma"
    );
    assert!(bytes.windows(6).any(|w| w == b"objeto"));
    assert_eq!(minor(&bytes), 3);
    assert_eq!(codec::read(&bytes).expect("lê"), doc);
}

#[test]
fn uma_gravacao_sem_ficha_nao_sobe_o_minor_nem_exige_objeto() {
    let doc = MacroDoc {
        reference: RefRect { w: 2.0, h: 4.0 },
        tracks: vec![trilha(None)],
    };
    let bytes = codec::write(&doc);
    assert_eq!(minor(&bytes), 0);
    assert!(!bytes.windows(6).any(|w| w == b"objeto"));
}

#[test]
fn o_ficheiro_vai_e_volta_ao_bit() {
    let f = Ficheiro {
        doc: MacroDoc {
            reference: RefRect { w: 96.0, h: 64.0 },
            tracks: vec![trilha(None), trilha(Some(ficha()))],
        },
        modo: 2,
        nome: "Céu ao pôr do sol".into(),
    };
    let bytes = ficheiro::escreve(&f);
    let back = ficheiro::le(&bytes).expect("lê");
    assert_eq!(back, f);
    assert_eq!(ficheiro::escreve(&back), bytes);
}

#[test]
fn o_componente_le_um_ficheiro_e_o_ficheiro_le_um_componente() {
    let doc = MacroDoc {
        reference: RefRect { w: 96.0, h: 64.0 },
        tracks: vec![trilha(Some(ficha()))],
    };
    let f = Ficheiro {
        doc: doc.clone(),
        modo: 1,
        nome: "x".into(),
    };
    assert_eq!(
        codec::read(&ficheiro::escreve(&f)).expect("o componente lê o ficheiro"),
        doc
    );
    let lido = ficheiro::le(&codec::write(&doc)).expect("o ficheiro lê o componente");
    assert_eq!((lido.doc, lido.modo, lido.nome.as_str()), (doc, 0, ""));
}

#[test]
fn uma_ficha_com_o_local_cortado_recusa_se() {
    let doc = MacroDoc {
        reference: RefRect { w: 2.0, h: 4.0 },
        tracks: vec![trilha(Some(ficha()))],
    };
    let mut bytes = codec::write(&doc);
    // O campo 3 da ficha (28 bytes) passa a dizer 27: o resto do registo desalinha-se.
    let i = bytes
        .windows(2)
        .position(|w| w == [0x03, 28])
        .expect("o local está lá");
    bytes[i + 1] = 27;
    assert!(matches!(
        codec::read(&bytes),
        Err(CodecError::BadFieldLength { .. } | CodecError::Truncated | CodecError::BadVarint | CodecError::BadUtf8)
    ));
}
