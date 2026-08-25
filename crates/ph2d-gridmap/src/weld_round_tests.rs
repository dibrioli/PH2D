//! Os gates do arredondamento soldado.

use super::round_welded;
use crate::round::RoundOptions;
use crate::weld::{seam_residual, weld};

/// A cadeia até ao corte, com o arredondamento soldado por cima.
fn chain_welded(
    mesh: &mut ph2d_mesh::Mesh,
    rounds: usize,
) -> (crate::solve::GridMap, crate::round::RoundReport, crate::cut::CutMesh, crate::comb::Combed) {
    let (cut, combed, h, singular) = crate::round::tests::chain(mesh);
    let (map, rep) = round_welded(
        mesh,
        &cut,
        &combed,
        h,
        RoundOptions {
            welded_rounds: rounds,
            ..RoundOptions::default()
        },
        &singular,
    );
    (map, rep, cut, combed)
}

/// ⭐⭐⭐ **GATE nº2 DA ESPEC — a translação continua INTEIRA, exacto.**
///
/// ⚠️ **Ele já existia para o caminho penalizado e FICA**, agora sobre outra estrutura:
/// no soldado as inteiras não são «as que fecham ciclo», são as **livres do sistema dos
/// fechos** — e as dependentes têm de sair inteiras da substituição.
///
/// ⛔⛔ **A grandeza que ele lê é a de ANTES do encaixe** ([`RoundReport::shift_frac_max`]),
/// e a primeira redacção deste gate lia o mapa DEPOIS — *ele conferia os inteiros que a
/// própria função acabara de arredondar, que é um gate a fazer-se a si próprio uma
/// pergunta cuja resposta ele escreveu.* A barra é o chão de acumulação de `f32`; um
/// pivô de determinante `2` poria a dependente num **meio-inteiro** e passaria por cima
/// dela, e é isso que este gate existe para não deixar passar.
#[test]
#[ignore = "lento -- corre a cadeia inteira"]
fn every_transition_of_the_welded_map_is_an_integer_translation() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    let (map, rep, _cut, _combed) = chain_welded(&mut mesh, 2_000);
    assert!(rep.pinned > 20, "a fixtura tem de conter o fenómeno: {} pregadas", rep.pinned);
    let biggest = map
        .shift
        .iter()
        .fold(0.0f32, |m, t| m.max(t[0].abs()).max(t[1].abs()));
    // ⭐ O chão: uma dependente é uma soma de ~dezenas de termos `±R^m·t` em `f32`.
    let bar = 64.0 * biggest.max(1.0) * f32::EPSILON;
    assert!(
        rep.shift_frac_max <= bar,
        "a pior distância a inteiro ANTES do encaixe foi {:.3e}, e o chão de `f32` para \
         |t| = {biggest:.1} é {bar:.3e} — um valor muito maior é um pivô de determinante \
         `2` a pôr a dependente num meio-inteiro",
        rep.shift_frac_max
    );
}

/// ⭐⭐⭐ **GATE nº1 DA ESPEC, no fim da cadeia — o resíduo da costura é ZERO, nas duas
/// espécies de ligação.**
///
/// ⛔ **É o gate que não existia**, e é o coração da obra. Prove-se por **mutação**:
/// desligar a eliminação (relaxar as cópias em vez de as derivar) põe este número em
/// `0,23`–`1,41` — que é o que o caminho penalizado mede hoje.
#[test]
#[ignore = "lento -- corre a cadeia inteira"]
fn the_welded_seam_residual_is_zero_on_both_kinds_of_link() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 36, 1.0);
    let (map, rep, cut, combed) = chain_welded(&mut mesh, 2_000);
    let (w, _) = weld(&cut, &combed);
    let sr = seam_residual(&w, &map);
    let biggest = map
        .uv
        .iter()
        .flatten()
        .fold(0.0f32, |m, z| m.max(z[0].abs()).max(z[1].abs()));
    let bar = 8.0 * biggest * f32::EPSILON;
    assert!(
        sr.links > 400 && sr.closures > 20,
        "a fixtura tem de conter as DUAS espécies: {} eliminadas, {} fechos",
        sr.links,
        sr.closures
    );
    assert!(
        sr.max <= bar && sr.turning_max <= bar && sr.flat_max <= bar,
        "resíduo: eliminadas {:.3e}, fechos que rodam {:.3e}, fechos planos {:.3e} — \
         o chão de `f32` para |z| = {biggest:.1} é {bar:.3e}",
        sr.max,
        sr.turning_max,
        sr.flat_max
    );
    assert!(
        rep.seam_after.1 <= bar,
        "a régua independente do solver leu {:.3e}",
        rep.seam_after.1
    );
}

/// ⭐ **A soldadura não inventa nem perde ligações** — a contagem fecha.
#[test]
fn every_seam_link_is_either_eliminated_or_a_closure() {
    for mut mesh in [
        ph2d_mesh::shapes::uv_sphere(24, 36, 1.0),
        ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35),
    ] {
        let (cut, combed, _h, _) = crate::round::tests::chain(&mut mesh);
        let (_, r) = weld(&cut, &combed);
        assert!(r.links > 0, "a fixtura tem de conter costuras");
        assert_eq!(
            r.links,
            r.eliminated + r.closures,
            "ligações: {} = {} eliminadas + {} fechos?",
            r.links,
            r.eliminated,
            r.closures
        );
        assert_eq!(
            r.closures,
            r.turning + r.flat,
            "fechos: {} = {} que rodam + {} planos?",
            r.closures,
            r.turning,
            r.flat
        );
        assert_eq!(
            r.copies - r.classes,
            r.eliminated,
            "cada ligação eliminada apaga exactamente uma variável: {} cópias − {} \
             classes ≠ {} eliminadas",
            r.copies,
            r.classes,
            r.eliminated
        );
    }
}
