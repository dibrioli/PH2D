//! ⭐ **As etapas do Motion** (W5b, `docs/Macros/02_plano_W5.md` §1/§3) sobre a macro da cena `=2`: uma
//! etapa por gesto, com o nome do que fez; uma espera antes de um gesto e um valor editado dão, nos dois
//! modos e ao bit, o retrato de GRAVAR a cena já com essa mudança. Controlo: o original difere da
//! verdade.

use super::em_cena::MacroEmCena;
use super::etapas::{self, TipoDaEtapa};
use super::gate_tests::retrato;
use super::roteiro::{CENA, Gesto};
use super::smoke::grava_gestos;
use super::tocar::Modo;
use crate::motion_mundo::MundoDaCena;
use ph2d_macro::{MacroDoc, codec};

const DONO: u64 = 7;

fn gravado(gestos: &[Gesto], antes: &[u32]) -> (MacroDoc, f64) {
    let (b, d) = grava_gestos(DONO, gestos, antes).expect("grava");
    (codec::read(&b).expect("lê-se"), d)
}

fn retrato_no_fim(doc: &MacroDoc, d: f64, modo: Modo) -> Vec<u32> {
    let mut m = MacroEmCena::de(&codec::write(doc), modo)
        .expect("lê-se")
        .expect("tem a trilha do Motion");
    m.segue(d, &MundoDaCena::default(), None).expect("toca até ao fim");
    retrato(m.palco())
}

/// Nos dois modos: a edição dá a verdade ao bit, e o original não.
fn bate(editado: &MacroDoc, verdade: &(MacroDoc, f64), original: &(MacroDoc, f64)) {
    for modo in [Modo::Fiel, Modo::Explicativo] {
        let v = retrato_no_fim(&verdade.0, verdade.1, modo);
        assert_ne!(
            retrato_no_fim(&original.0, original.1, modo),
            v,
            "{modo:?}: controlo — a mudança não muda o retrato"
        );
        assert_eq!(
            retrato_no_fim(editado, verdade.1, modo),
            v,
            "{modo:?}: a macro editada não é a gravada com a mudança"
        );
    }
}

fn tipo_do_slot(s: usize) -> String {
    CENA.iter()
        .filter_map(|g| match g {
            Gesto::Add(t) => Some((*t).to_owned()),
            _ => None,
        })
        .nth(s)
        .expect("o slot existe")
}

#[test]
fn a_cena_tem_uma_etapa_por_gesto_com_o_nome_do_que_fez() {
    let (doc, _) = gravado(CENA, &[]);
    let e = etapas::etapas(&doc.tracks[0]).expect("lê-se");
    assert_eq!(e.len(), CENA.len(), "uma etapa por gesto");
    for (k, (et, g)) in e.iter().zip(CENA).enumerate() {
        let esperado = match *g {
            Gesto::Add(t) => TipoDaEtapa::Acrescenta(vec![t.to_owned()]),
            Gesto::Liga(a, _, b, _) => TipoDaEtapa::Liga {
                de: Some(tipo_do_slot(a)),
                para: Some(tipo_do_slot(b)),
            },
            Gesto::Param(s, p, v) => match &et.tipo {
                TipoDaEtapa::Param { de, .. } => TipoDaEtapa::Param {
                    no: Some(tipo_do_slot(s)),
                    param: p.to_owned(),
                    de: *de,
                    para: f64::from(v),
                },
                outro => panic!("etapa {k}: {outro:?} não é um param"),
            },
        };
        assert_eq!(et.tipo, esperado, "etapa {k}");
    }
    assert!(e.windows(2).all(|w| w[0].t1 <= w[1].t0), "as etapas não se sobrepõem");
}

#[test]
fn uma_espera_antes_de_um_gesto_e_grava_lo_mais_tarde() {
    let original = gravado(CENA, &[]);
    let mut antes = vec![0; CENA.len()];
    antes[15] = 60;
    let verdade = gravado(CENA, &antes);
    let mut editado = original.0.clone();
    etapas::espera(&mut editado.tracks[0], 15, 1_000_000).expect("a etapa existe");
    bate(&editado, &verdade, &original);
}

#[test]
fn encurtar_a_espera_antes_de_um_gesto_e_grava_lo_mais_cedo() {
    let mut antes = vec![0; CENA.len()];
    antes[15] = 40;
    let longo = gravado(CENA, &antes);
    let verdade = gravado(CENA, &[]);
    let mut editado = longo.0.clone();
    etapas::espera(&mut editado.tracks[0], 15, -666_667).expect("a etapa existe");
    bate(&editado, &verdade, &longo);
}

#[test]
fn editar_o_valor_de_um_param_e_grava_lo_com_esse_valor() {
    let original = gravado(CENA, &[]);
    let k = CENA
        .iter()
        .position(|g| matches!(g, Gesto::Param(_, "cols", _)))
        .expect("a cena edita as colunas");
    let mut gestos = CENA.to_vec();
    let Gesto::Param(s, p, _) = gestos[k] else {
        unreachable!()
    };
    gestos[k] = Gesto::Param(s, p, 7.0);
    let verdade = gravado(&gestos, &[]);
    let mut editado = original.0.clone();
    etapas::edita_valor(&mut editado.tracks[0], k, 7.0).expect("é um param");
    bate(&editado, &verdade, &original);
    match &etapas::etapas(&editado.tracks[0]).expect("lê-se")[k].tipo {
        TipoDaEtapa::Param { para, .. } => assert!((para - 7.0).abs() < 1e-9, "o painel mostra 7"),
        outro => panic!("{outro:?}"),
    }
}
