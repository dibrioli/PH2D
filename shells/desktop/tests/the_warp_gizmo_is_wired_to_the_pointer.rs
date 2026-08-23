//! **A COSTURA do gizmo dos deformadores de quadrilátero está LIGADA** — o censo de
//! fonte que impede as alças de virarem decoração.
//!
//! ⚠️ **Por que um gate de FONTE e não um de comportamento.** A geometria já tem dez
//! gates puros (`render_loop::warp_gizmo::tests`) e o desenho é tinta; o que nenhum deles
//! vê é a **ligação**: publicar o retrato no prólogo, desenhá-lo, e chamar as três pontas
//! do ponteiro. Cada uma dessas linhas pode ser apagada sem que um único gate de
//! geometria fique vermelho — e o resultado seria um gizmo perfeito que nunca aparece.
//! É a mesma classe que o censo das tomadas de sinal deste mesmo diretório guarda, e a
//! razão de ele existir lá: *a costura é o que se perde num merge, não a matemática.*

use std::fs;

fn src(name: &str) -> String {
    fs::read_to_string(format!("src/{name}")).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// **O RETRATO é publicado, e é publicado com a modalidade da tool.**
#[test]
fn the_view_is_published_once_per_frame_gated_by_the_motion_tool() {
    let s = src("render_loop/mod.rs");
    assert!(
        s.contains("warp_gizmo::publish(warp_gizmo::resolve(motion, motion_tool_active))"),
        "o retrato tem de ser publicado no prólogo, e gateado pela tool Motion — sem a \
         modalidade, as alças de um nó apareceriam sobre o canvas de outra ferramenta"
    );
}

/// **E ELE É DESENHADO.**
#[test]
fn the_published_view_is_actually_drawn() {
    let s = src("render_loop/mod.rs");
    assert!(
        s.contains("warp_overlay::draw_warp_gizmo("),
        "o retrato publicado tem de chegar à tinta"
    );
    assert!(
        s.contains("warp_gizmo::view()"),
        "e o pintor lê o retrato publicado, nunca re-decide a tool"
    );
}

/// **AS TRÊS PONTAS DO PONTEIRO.**
///
/// ⚠️ As três, e não uma: sem o `down` a alça não agarra; sem o `move` ela agarra e não
/// segue; sem o `up` o arrasto nunca larga e o próximo clique continua a escrever no nó
/// anterior. Cada ausência é um defeito diferente, e nenhuma delas é visível num gate de
/// geometria.
#[test]
fn all_three_pointer_ends_are_wired() {
    let s = src("input_dispatch.rs");
    for needle in [
        "self.warp_gizmo_down(",
        "self.warp_gizmo_move(",
        "self.warp_gizmo_up()",
    ] {
        assert!(s.contains(needle), "a costura `{needle}` não está ligada");
    }
}

/// **O `down` do warp vem ANTES do do field e do genérico.**
///
/// ⚠️ Uma alça alcançada pelo caminho genérico escreveria um `Transform` de ENTIDADE em
/// vez do param do nó — o mesmo defeito que fez o gizmo do Flip e o do field virem antes.
/// A ordem no arquivo É a precedência.
#[test]
fn the_warp_grab_is_tried_before_the_generic_gizmo() {
    let s = src("input_dispatch.rs");
    let warp = s.find("self.warp_gizmo_down(").expect("o warp está lá");
    let field = s.find("self.field_gizmo_down(").expect("o field está lá");
    assert!(
        warp < field,
        "o `down` do warp tem de ser tentado antes do do field"
    );
}

/// **A EDIÇÃO SAI PELA PORTA DO PAINEL, e não por uma segunda.**
///
/// ⚠️ O arrasto escreve `set_param` — a mesma função que o slider chama. Um segundo
/// caminho de escrita divergiria do commit, do undo e do que o painel mostra.
#[test]
fn the_drag_writes_through_the_same_port_the_panel_uses() {
    let s = src("warp_gizmo_drag.rs");
    assert!(
        s.contains("graph.set_param("),
        "a edição sai por `set_param`, a porta do painel"
    );
    // ⚠️ E o CONTROLE: nada aqui pode tocar num `Transform` de entidade. O gizmo é de um
    // NÓ; escrever no mundo ECS seria ele a mexer nos sprites que a outra ferramenta
    // manipula — a prova de isolamento que o `field_gizmo` documenta.
    assert!(
        !s.contains("Transform"),
        "o gizmo de um nó não pode escrever num Transform de entidade"
    );
}
