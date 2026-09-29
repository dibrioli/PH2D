//! Os gates da decisão TUDO-OU-NADA da placa (doc 121 §2.1 do Motion) — puros, sem adaptador.

use super::*;
use ph2d_vec_scene::{Rgba8, StrokeSpec};

/// Uma cópia daquela geometria, na posição `x`, com `basis` e `size` dados.
fn vi(geometry_id: u32, x: f32, basis: [f32; 4], size: [f32; 2]) -> VectorInstance {
    VectorInstance {
        geometry_id,
        texture_id: 0,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        premultiplied: 0.0,
        world_pos: [x, 0.0],
        size,
        basis,
        tint: [1.0, 0.5, 0.25, 1.0],
        anchor: [0.0, 0.0],
        sampling: 0,
        blend_linha: 0,
        mistura: Default::default(),
    }
}

const ID: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

/// Uma estrela sem traço e uma com traço, no mesmo store.
fn store() -> (VecPathStore, u32, u32) {
    let mut s = VecPathStore::default();
    let lisa = s.push(ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 5, 0.4));
    let mut p = ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5);
    p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 0.05));
    let tracada = s.push(p);
    (s, lisa, tracada)
}

/// ⭐ Formas simples e conformes vão à placa, com as cópias na ORDEM das linhas.
#[test]
fn formas_conformes_vao_a_placa_na_ordem() {
    let (s, lisa, tracada) = store();
    let rot = [0.6, 0.8, -0.8, 0.6];
    let insts = [
        vi(lisa, 1.0, ID, [2.0, 2.0]),
        vi(tracada, 2.0, rot, [3.0, 3.0]),
        vi(lisa, 3.0, rot, [0.5, 0.5]),
    ];
    let mut p = PlacaDeFormas::default();
    assert!(p.decide(true, &insts, &s, Affine::IDENTITY));
    assert!(p.ativa());
    let xs: Vec<f32> = p.copias().iter().map(|c| c.pos[0]).collect();
    assert_eq!(xs, vec![1.0, 2.0, 3.0], "a ordem das linhas é o desenho");
    assert_eq!(p.copias()[1].geometry, tracada);
    assert_eq!(p.copias()[0].tint, [1.0, 0.5, 0.25, 1.0]);
}

/// ⛔ Um quad de IMAGEM no meio devolve o quadro INTEIRO ao Vello — partir a lista trocaria a
/// ordem entre as duas metades.
#[test]
fn uma_imagem_no_meio_devolve_tudo_ao_vello() {
    let (s, lisa, _) = store();
    let mut img = vi(0, 2.0, ID, [1.0, 1.0]);
    img.texture_id = 9;
    let insts = [
        vi(lisa, 1.0, ID, [1.0, 1.0]),
        img,
        vi(lisa, 3.0, ID, [1.0, 1.0]),
    ];
    let mut p = PlacaDeFormas::default();
    assert!(!p.decide(true, &insts, &s, Affine::IDENTITY));
    assert!(p.copias().is_empty(), "nenhuma cópia fica a meio caminho");
}

/// ⛔ Uma linha com mistura própria pede uma camada — o quadro fica no Vello.
#[test]
fn uma_mistura_devolve_tudo_ao_vello() {
    let (s, lisa, _) = store();
    let mut m = vi(lisa, 2.0, ID, [1.0, 1.0]);
    m.blend_linha = 2; // o modo 1 (`Add`) — tem camada
    let mut p = PlacaDeFormas::default();
    assert!(!p.decide(
        true,
        &[vi(lisa, 1.0, ID, [1.0, 1.0]), m],
        &s,
        Affine::IDENTITY
    ));
}

/// ⛔ Um TRAÇO sob escala não uniforme vai ao Vello (a caneta é do MUNDO, bug #27); o MESMO
/// afim sem traço vai à placa — ⚠️ é o controlo que prova que a recusa é do traço.
#[test]
fn o_traco_sob_escala_nao_uniforme_fica_no_vello() {
    let (s, lisa, tracada) = store();
    let mut p = PlacaDeFormas::default();
    assert!(!p.decide(
        true,
        &[vi(tracada, 1.0, ID, [3.0, 1.0])],
        &s,
        Affine::IDENTITY
    ));
    assert!(p.decide(true, &[vi(lisa, 1.0, ID, [3.0, 1.0])], &s, Affine::IDENTITY));
}

/// Sem a porta ligada, ou sem cópias, o quadro é o de sempre.
#[test]
fn desligada_ou_vazia_nao_vai_a_placa() {
    let (s, lisa, _) = store();
    let mut p = PlacaDeFormas::default();
    assert!(!p.decide(
        false,
        &[vi(lisa, 1.0, ID, [1.0, 1.0])],
        &s,
        Affine::IDENTITY
    ));
    assert!(!p.decide(true, &[], &s, Affine::IDENTITY));
}

/// ⛔ Uma forma com TINTA própria é do Vello (o passe pinta com a cor da cópia).
#[test]
fn uma_tinta_propria_fica_no_vello() {
    let (mut s, _, _) = store();
    let mut pintada = ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5);
    pintada.fill = Some(ph2d_vec_scene::Paint::Solid(Rgba8::new(1, 2, 3, 255)));
    let h = s.push(pintada);
    let mut p = PlacaDeFormas::default();
    assert!(!p.decide(true, &[vi(h, 1.0, ID, [1.0, 1.0])], &s, Affine::IDENTITY));
}

/// A conformidade: rotação e escala uniforme sim, espelho sim, esticão e cisalha não.
#[test]
fn conforme_separa_as_poses() {
    let v = |b, s| vi(1, 0.0, b, s);
    assert!(conforme(&v([0.6, 0.8, -0.8, 0.6], [2.0, 2.0])));
    assert!(
        conforme(&v(ID, [-2.0, 2.0])),
        "um espelho mantém a caneta redonda"
    );
    assert!(!conforme(&v(ID, [2.0, 1.0])));
    assert!(!conforme(&v([1.0, 0.0, 0.5, 1.0], [1.0, 1.0])), "cisalha");
    // ⚠️ A folga é RELATIVA: a mesma rotação a 1e-6 e a 1e6 é conforme nos dois.
    assert!(conforme(&v([0.6, 0.8, -0.8, 0.6], [1e-6, 1e-6])));
    assert!(conforme(&v([0.6, 0.8, -0.8, 0.6], [1e6, 1e6])));
}
