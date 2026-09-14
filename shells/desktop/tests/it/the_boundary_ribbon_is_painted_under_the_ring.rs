//! **A FITA DO PINCEL DE CONTORNO é pintada, é pintada POR BAIXO do anel, e a
//! alfa de cada pedaço é o PESO dele.**
//!
//! ## Porque isto é um arch-gate e não um teste de unidade
//!
//! A mesma razão do irmão `the_pose_bone_is_painted_under_the_ring`: o
//! `boundary_gizmo` precisa de uma `Sculpt3dScene`, que precisa de um
//! `wgpu::Device`. ⚠️ **A figura tem gates de FORMA** (`ph2d-sculpt3d`, no
//! `boundary_previa`) e a lei tem gates de PARIDADE (`ph2d-boundary`, 61
//! fixturas) — os dois ficam **verdes** com o overlay a nunca chamar o gizmo.
//! *Um indicador construído e nunca pintado lê-se exactamente como um que não
//! existe*, que foi o report do dono em 2026-09-14: «não tem gizmo».
//!
//! ## As TRÊS metades
//!
//! **(a) Ela é pintada**, com os quatro estados dela.
//!
//! **(b) Ela vem ANTES do anel do cursor** — o anel é onde a mão está *agora* e
//! tem de ficar por cima. A ordem no Vello é a ordem das chamadas, então **a
//! posição no ficheiro É a lei**.
//!
//! **(c) O PESO chega à tinta.** ⚠️⚠️ Esta é a metade que uma leitura rápida
//! salta, e sem ela o indicador **mente**: pintar todos os pedaços com a mesma
//! força diz que a boca inteira entra, que é precisamente o que o selector
//! `Falloff along the edge` muda. *A figura deixaria de responder à única
//! pergunta para a qual ela existe.*

use std::fs;

const FASE: &str = "src/render_loop/fase_canvas_overlays.rs";

fn source() -> String {
    fs::read_to_string(FASE).unwrap_or_else(|e| panic!("não consegui ler {FASE}: {e}"))
}

/// **(a)** O overlay chama o gizmo e pinta os quatro estados.
#[test]
fn the_boundary_ribbon_reaches_the_canvas() {
    let src = source();
    assert!(
        src.contains(".boundary_gizmo("),
        "a fase de overlays não chama `boundary_gizmo` — o indicador do pincel \
         de contorno está construído, gateado e INVISÍVEL"
    );
    // ⚠️ As quatro tintas, porque são **quatro coisas diferentes**: a fita, a
    // profundidade, o eixo, e o aviso de que ali não há beirada nenhuma.
    for tinta in [
        "BOUNDARY_EDGE_RGBA",
        "BOUNDARY_DEPTH_RGBA",
        "BOUNDARY_PIVOT_RGBA",
        "BOUNDARY_INERT_RGBA",
    ] {
        assert!(
            src.contains(tinta),
            "a fase não usa `{tinta}` — um dos estados do indicador não chega a \
             pixel nenhum"
        );
    }
}

/// **(b)** A fita é traçada **antes** do anel do cursor.
#[test]
fn the_boundary_ribbon_is_painted_under_the_cursor_ring() {
    let src = source();
    let fita = src
        .find(".boundary_gizmo(")
        .expect("o gizmo tem de ser chamado — ver o gate irmão");
    let anel = src
        .find(".cursor_mark(")
        .expect("o anel do cursor tem de continuar a ser desenhado");
    assert!(
        fita < anel,
        "a fita é pintada DEPOIS do anel ({fita} contra {anel}): no Vello a \
         ordem das chamadas é a ordem das camadas, e a mira tem de ficar por cima"
    );
}

/// **(c)** O peso de cada pedaço entra na tinta.
///
/// ⚠️ A régua é a **multiplicação da componente alfa pelo peso**, e não «o
/// `peso` é mencionado»: um `for (caminho, _peso)` que ignorasse o segundo
/// elemento deixaria a palavra no ficheiro e a figura chapada.
#[test]
fn the_weight_of_each_piece_reaches_the_ink() {
    let src = source();
    let bloco = src
        .split_once(".boundary_gizmo(")
        .map(|(_, r)| r)
        .and_then(|r| r.split_once(".cursor_mark("))
        .map(|(l, _)| l)
        .expect("o bloco da fita vive entre a chamada dela e o anel");
    assert!(
        bloco.contains("rgba[3] *= peso"),
        "a alfa de cada pedaço não é multiplicada pelo peso — a fita fica \
         CHAPADA, e o `Falloff along the edge` deixa de ter efeito visível:\n{bloco}"
    );
}
