//! ⭐⭐ **A LIMPEZA NÃO PODE PIORAR O QUE ELA EXISTE PARA CURAR.**
//!
//! ⛔ **Este ficheiro nasce de uma cura que agravava o defeito** (2026-08-22). O
//! laço de limpeza dissolve paredes para curar patches degenerados; num toro ele
//! corria **dez** rondas e a última levava a decomposição de *"um anel, sinalizado"*
//! para *"uma asa dentro de um patch, não sinalizada"*.
//!
//! ⚠️ **Ela agravava e ESCONDIA ao mesmo tempo**, e é a segunda metade que faz o
//! defeito escapar: o anel é um patch com duas fronteiras, que o `degenerate()`
//! apanha; a asa tem **uma** fronteira, e passa. ⇒ A cadeia devolvia uma malha de
//! género errado, com 100 % de quads, zero arestas de bordo e zero não-manifold.
//!
//! A régua é `|V − E + F do complexo − χ da peça|`. Ela é `0` exactamente quando a
//! decomposição ainda descreve a superfície que entrou.

use ph2d_crossfield::Dual;
use ph2d_mesh::{Mesh, shapes};
use ph2d_trace::patches::decompose;
use ph2d_trace::walk::Walker;

fn tri(mut m: Mesh) -> Mesh {
    m.triangulate();
    m
}

/// A distância topológica de um layout: `0` é uma decomposição honesta.
fn gap(layout: &ph2d_trace::PatchLayout) -> i64 {
    (layout.complex_euler() - layout.mesh_chi).abs()
}

/// ⭐⭐ **A limpeza nunca aumenta a distância topológica.**
///
/// ⚠️ **A fixtura que contém o fenómeno é o toro 48×24**, e sem ela este gate é
/// verde sobre qualquer coisa: nos outros três a limpeza ou não corre, ou fecha na
/// primeira ronda. *Medido antes da guarda: aquele toro ia de `1` para `2`.*
///
/// ⚠️ **A barra é «não piora», e não «fica em zero»** — de propósito. O traçado
/// ainda não sabe **cortar** uma asa, então há peças em que ele começa com a
/// distância a `1` e não tem como a fechar; exigir zero aqui seria escrever nesta
/// asserção um trabalho que mora noutro sítio (e o gate `#[ignore]`
/// `the_genus_survives_on_every_torus`, na `ph2d-quadfill`, é o endereço dele).
#[test]
fn the_cleanup_never_worsens_the_topology() {
    for (name, mesh) in [
        ("esfera 24x36", tri(shapes::uv_sphere(24, 36, 1.0))),
        // ⭐ Cinco rondas com o estado idêntico antes de fechar — é ela que
        // reprova qualquer «teto de rondas paradas».
        ("esfera 48x72", tri(shapes::uv_sphere(48, 72, 1.0))),
        ("toro 32x16", tri(shapes::torus(32, 16, 1.0, 0.35))),
        // ⭐⭐ A fixtura do fenómeno.
        ("toro 48x24", tri(shapes::torus(48, 24, 1.0, 0.35))),
        ("toro 64x32", tri(shapes::torus(64, 32, 1.0, 0.35))),
    ] {
        let mut work = mesh.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);

        // ⚠️ **O ponto de partida é a decomposição CRUA**, antes de a limpeza
        // correr — é contra ela que «não piorar» significa alguma coisa.
        let walker = Walker::new(&work, &dual, &field);
        let (walls, base) = walker.trace_all();
        let raw = decompose(&work, &walls, base);
        let cleaned = ph2d_trace::trace_patches(&work, &dual, &field);

        eprintln!(
            "[f3] {name}: distancia crua {} -> limpa {} ({} rondas, {} paredes)",
            gap(&raw),
            gap(&cleaned),
            cleaned.report.rounds,
            cleaned.report.dissolved,
        );
        assert!(
            gap(&cleaned) <= gap(&raw),
            "{name}: a limpeza levou a distancia topologica de {} para {} -- \
             ela agravou o que existe para curar",
            gap(&raw),
            gap(&cleaned),
        );
    }
}

/// ⭐ **E a limpeza continua a CURAR** — a metade que impede a guarda de virar um
/// «não faças nada».
///
/// ⛔ **Sem este lado, `trace_patches` podia devolver a decomposição crua** e o
/// gate irmão ficaria feliz: *"não piorou"* é verdade sobre uma limpeza que nunca
/// corre. As duas fixturas abaixo **precisam** dela — medido: a esfera 48×72 fecha
/// na ronda 5 e o toro 32×16 na ronda 1.
#[test]
fn the_cleanup_still_closes_what_it_can() {
    for (name, mesh) in [
        ("esfera 48x72", tri(shapes::uv_sphere(48, 72, 1.0))),
        ("toro 32x16", tri(shapes::torus(32, 16, 1.0, 0.35))),
    ] {
        let mut work = mesh.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let walker = Walker::new(&work, &dual, &field);
        let (walls, base) = walker.trace_all();
        let raw = decompose(&work, &walls, base);
        let cleaned = ph2d_trace::trace_patches(&work, &dual, &field);
        assert!(
            gap(&raw) > 0,
            "{name}: a decomposicao crua ja' estava fechada -- esta fixtura nao \
             exercita a limpeza, e o gate e' verde por acidente"
        );
        assert_eq!(
            gap(&cleaned),
            0,
            "{name}: a limpeza nao fechou uma decomposicao que ela FECHAVA antes \
             da guarda -- a guarda esta' a cortar progresso legitimo"
        );
        assert!(
            cleaned.report.rounds > 0,
            "{name}: a limpeza nao correu ronda nenhuma"
        );
    }
}
