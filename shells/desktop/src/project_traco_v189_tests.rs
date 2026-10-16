//! ⭐⭐ **Um projecto gravado ANTES do tracejado em pares abre igual** (F16c, `PROJECT_SCHEMA`
//! `189 → 190`; ordem do dono: *«ficheiros antigos têm de continuar a abrir iguais»*).
//!
//! `tests/fixtures/projecto_v189_traco.ph2dproj` foi gravado pelo código v189 (commit `d138b96401`)
//! — ⛔ **nunca o regenere**. Dentro: a cena de `ph2d-vec-scene/tests/it/fixtures/
//! vec_scene_v22_traco.bin` e um estado Hover com traço `(4, 2)` e `marker_scale 0,6`.

use super::*;
use ph2d_vec_scene::DashPattern;

const V189: &[u8] = include_bytes!("../tests/fixtures/projecto_v189_traco.ph2dproj");

fn le(bytes: &[u8], nome: &str) -> Option<ProjectFile> {
    let path = tmp_path(nome);
    std::fs::write(&path, bytes).expect("grava o fixture");
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    lido.map(|(f, _)| f)
}

#[test]
fn a_v189_project_opens_with_the_same_strokes() {
    let file = le(V189, "traco_v189").expect("um v189 abre");
    let [seta, caixa, lisa] = file.state.vec.paths() else {
        panic!("três formas, lidas {}", file.state.vec.paths().len());
    };
    let s = seta.stroke.as_ref().expect("traço");
    assert_eq!(s.dash, Some(DashPattern::single(3.0, 1.5)));
    assert_eq!((s.marker_scale_start, s.marker_scale_end, s.marker_round), (1.7, 1.7, 0.25));
    assert_eq!(caixa.stroke.as_ref().and_then(|s| s.dash), Some(DashPattern::single(2.0, 1.0)));
    assert_eq!(lisa.stroke.as_ref().map(|s| s.dash), Some(None));

    // O traço de um ESTADO de UI viaja no mesmo ficheiro e lê-se pelo mesmo âmbito.
    let host = caixa.id;
    let [estado] = file.state.ui_states.get(host) else {
        panic!("um estado no hospedeiro");
    };
    let pose = estado.objects.first().expect("uma pose");
    let s = pose.stroke.as_ref().expect("a pose tem traço");
    assert_eq!(s.dash, Some(DashPattern::single(4.0, 2.0)));
    assert_eq!((s.marker_scale_start, s.marker_scale_end), (0.6, 0.6));
}

/// ⛔ CONTROLO: os MESMOS bytes com o cabeçalho do schema de hoje são lidos pela forma nova, e não
/// abrem igual — o que a prova de cima mede é o âmbito do `189`, e não um acaso do formato.
#[test]
fn the_same_bytes_under_todays_header_do_not_open_the_same() {
    let mut hoje = V189.to_vec();
    // `189` e `190` são varints de dois bytes (`0xBD 0x01` / `0xBE 0x01`).
    assert_eq!(&hoje[..2], &[0xBD, 0x01], "o fixture é um v189");
    assert_eq!(PROJECT_SCHEMA, 190);
    hoje[0] = 0xBE;
    let certo = le(V189, "traco_v189_certo").expect("um v189 abre");
    let torto = le(&hoje, "traco_v189_torto");
    assert!(
        torto.is_none_or(|f| f.state.vec != certo.state.vec),
        "lido como v190, o v189 deu a mesma cena — o gate de cima é cego"
    );
}
