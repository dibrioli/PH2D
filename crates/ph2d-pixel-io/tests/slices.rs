//! **As slices no `.ase` e no JSON** (P11): o Trial lê os nossos ficheiros e escreve `meta.slices`, e o
//! nosso escritor tem de dar os mesmos bytes (`docs/PixelArt/P11_oraculos_arquivo.md` F3, F5).

use ph2d_pixel_core::{Slice, Sprite};
use ph2d_pixel_io::ase::{Aviso, escrever, ler};
use ph2d_pixel_io::sheet::{OpcoesFolha, OpcoesJson, json_da_folha, montar};
use serde_json::Value;

const ORACULO: &str = include_str!("fixtures/oracle_slices_meta.json");

fn ase(nome: &str) -> Vec<u8> {
    let p = format!(
        "{}/tests/fixtures/slices/{nome}.ase",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(&p).expect("a fixtura")
}

fn meta_slices(s: &Sprite) -> String {
    let f = montar(s, &OpcoesFolha::default(), "x", "ase");
    let o = OpcoesJson {
        slices: true,
        ..OpcoesJson::default()
    };
    let t = json_da_folha(s, &f, "s.png", "RGBA8888", o);
    t[t.find("\"slices\"").expect("a chave")..].to_owned()
}

fn casos() -> Vec<(String, String)> {
    let v: Value = serde_json::from_str(ORACULO).unwrap();
    let c = v["casos"].as_object().unwrap();
    assert_eq!(c.len(), 11, "os casos da fixtura");
    c.iter()
        .map(|(k, t)| (k.clone(), t.as_str().unwrap().to_owned()))
        .collect()
}

/// ⭐ O mesmo `.ase`, o Trial e nós: os mesmos bytes em `meta.slices`, com keys, escondidas, fora do
/// canvas, gémeas, sem cor, com texto e UTF-8. *Mutação: tirar o `.rev()`, ou trocar a ordem de `data`.*
#[test]
fn our_meta_slices_are_the_bytes_the_trial_writes_for_the_same_file() {
    for (nome, dele) in casos() {
        let (s, _) = ler(&ase(&nome)).unwrap_or_else(|e| panic!("{nome}: {e:?}"));
        assert_eq!(meta_slices(&s), dele, "{nome}");
    }
}

fn sem_ids(v: &[Slice]) -> Vec<Slice> {
    let mut v = v.to_vec();
    for c in &mut v {
        c.id = ph2d_pixel_core::SliceId(0);
        // A cor em falta grava-se: sem ela o Trial não desenha a slice (F3).
        c.color.get_or_insert(ph2d_pixel_core::DEFAULT_SLICE_COLOR);
    }
    v
}

/// Gravar e reabrir não perde nada, salvo o que o formato não diz: pivot por key numa slice com flag
/// geral, que se escreve `0,0` (limitação declarada no plano).
#[test]
fn saving_and_reopening_keeps_every_slice() {
    for (nome, _) in casos() {
        let (s, _) = ler(&ase(&nome)).unwrap();
        let (bytes, _) = escrever(&s).unwrap();
        let (volta, _) = ler(&bytes).unwrap();
        assert_eq!(sem_ids(volta.slices()), sem_ids(s.slices()), "{nome}");
        let (de_novo, _) = escrever(&volta).unwrap();
        assert_eq!(de_novo, bytes, "{nome}: gravar é estável");
    }
}

/// F3: a key fora de ordem ordena-se, e o chunk conta onde quer que esteja.
#[test]
fn unsorted_keys_sort_and_a_chunk_in_a_later_frame_is_read() {
    let (s, _) = ler(&ase("weird")).unwrap();
    let c = s.slices().iter().find(|c| c.name == "unsorted").unwrap();
    assert_eq!(c.keys.iter().map(|k| k.frame).collect::<Vec<_>>(), [1, 3]);
    assert_eq!(
        s.slices().iter().filter(|c| c.name == "twin").count(),
        2,
        "gémeas ficam"
    );
    let far = s.slices().iter().find(|c| c.name == "farframe").unwrap();
    assert_eq!(far.keys[0].frame, 99, "a key além do fim fica");

    let (t, _) = ler(&ase("tarde")).unwrap();
    assert_eq!(t.slices()[0].keys.len(), 2);
}

/// O user data pertence só à slice que o precede: o de outra coisa não a pinta.
#[test]
fn user_data_belongs_only_to_the_chunk_right_before_it() {
    let (s, avisos) = ler(&ase("ud")).unwrap();
    assert!(!s.slices().is_empty());
    assert!(avisos.is_empty(), "{avisos:?}");
    let (r, _) = ler(&ase("realorder")).unwrap();
    assert!(
        r.slices().iter().all(|c| c.color.is_some()),
        "camadas → slice+user → cels"
    );
}

/// Propriedades tipadas ficam para a P12: a slice fica, e o artista é avisado.
#[test]
fn typed_properties_warn_and_the_slice_stays() {
    let (limpo, _) = ler(&ase("full")).unwrap();
    let (bytes, _) = escrever(&limpo).unwrap();
    assert_eq!(ler(&bytes).unwrap().1, Vec::new());

    // O user data de `full` traz a cor (bit 2): acende-se o bit das propriedades (4).
    let mut marcado = bytes.clone();
    let cabecalho = [14, 0, 0, 0, 0x20, 0x20, 2, 0, 0, 0];
    let i = marcado.windows(10).position(|w| w == cabecalho).unwrap();
    marcado[i + 6..i + 10].copy_from_slice(&6_u32.to_le_bytes());
    let (s, avisos) = ler(&marcado).unwrap();
    assert_eq!(avisos, vec![Aviso::PropriedadesDeSlices(1)]);
    assert_eq!(s.slices().len(), 3);
}
