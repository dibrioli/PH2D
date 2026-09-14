//! **O OSSO DO PINCEL DE POSE é pintado, e é pintado POR BAIXO do anel.**
//!
//! ## Por que isto é um arch-gate e não um teste de unidade
//!
//! É a mesma razão do irmão `the_brush_ring_marks_the_hit_the_dab_will_use`: o
//! `pose_gizmo` precisa de uma `Sculpt3dScene`, que precisa de um
//! `wgpu::Device`, e a decisão que importa vive num bloco que só roda com
//! janela. ⚠️ **A figura tem gates de FORMA** (`ph2d-app-sculpt3d`) e a cadeia
//! tem gates de LEI (`ph2d-sculpt3d`, `ph2d-pose`) — os dois ficam **verdes**
//! com o overlay a nunca chamar o gizmo. *Um indicador construído e nunca
//! pintado lê-se exactamente como um que não existe* (a `line/Vector` pagou
//! esta lição nos chips da booleana, e o Motion na queixa que nunca chegava a
//! pixel).
//!
//! ## As duas metades, e porque são duas
//!
//! **(a) Ele é pintado.** Sem isto o trabalho inteiro é inerte.
//!
//! **(b) Ele vem ANTES do anel do cursor.** O anel é onde a mão está *agora* e
//! tem de ficar por cima; com a ordem trocada o osso — que é maior — passa a
//! tapar a mira, que é a única superfície que diz ao artista se o `pick` está
//! certo. ⚠️ A ordem no Vello é a ordem das chamadas, então **a posição no
//! ficheiro É a lei** aqui.
//!
//! Mutações que sangram: apagar a chamada a `pose_gizmo`; trocar os dois blocos
//! de sítio.

use std::fs;

const FASE: &str = "src/render_loop/fase_canvas_overlays.rs";

fn source() -> String {
    fs::read_to_string(FASE).unwrap_or_else(|e| panic!("não consegui ler {FASE}: {e}"))
}

/// **(a)** O overlay chama o gizmo e pinta o que ele devolve.
#[test]
fn the_pose_bone_reaches_the_canvas() {
    let src = source();
    assert!(
        src.contains(".pose_gizmo("),
        "a fase de overlays não chama `pose_gizmo` — o indicador do pincel de \
         pose está construído, gateado e INVISÍVEL"
    );
    // ⚠️ As três tintas, porque elas são **três estados** e não decoração: o
    // osso, a dobradiça, e o aviso de que ali não há dobradiça nenhuma (§11.1).
    for tinta in ["POSE_BONE_RGBA", "POSE_PIVOT_RGBA", "POSE_INERT_RGBA"] {
        assert!(
            src.contains(tinta),
            "a fase não usa `{tinta}` — um dos três estados do indicador não \
             chega a pixel nenhum"
        );
    }
}

/// **(b)** O osso é traçado **antes** do anel do cursor.
#[test]
fn the_pose_bone_is_painted_under_the_cursor_ring() {
    let src = source();
    let osso = src
        .find(".pose_gizmo(")
        .expect("o gizmo tem de ser chamado — ver o gate irmão");
    let anel = src
        .find(".cursor_mark(")
        .expect("o anel do cursor tem de continuar a ser desenhado");
    assert!(
        osso < anel,
        "o osso é pintado DEPOIS do anel ({osso} contra {anel}): no Vello a \
         ordem das chamadas é a ordem das camadas, e o osso é a figura maior — \
         ele passaria a tapar a mira, que é a superfície que revela um `pick` \
         errado"
    );
}
