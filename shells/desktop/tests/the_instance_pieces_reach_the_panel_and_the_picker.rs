//! **Arch-gate: a lista de PEÇAS chega ao painel, e a cor escolhida volta** (plano UI/UX W5b).
//!
//! # Porque é um arch-gate, e não um gate de unidade
//!
//! Os gates de `vec_component_pieces::tests` provam o MOTOR — que esconder uma peça muda o
//! desenho, que a cor pousa como override, que o *Update Main* absorve. Todos passariam com a
//! fiação da shell **arrancada**: a lista nunca publicada (a seção não pinta linha nenhuma) ou o
//! `picker_target` nunca lido (a swatch abre o OKLCH e a escolha morre ali). Essa metade vive
//! dentro do laço de frame, que exige janela — nenhum teste de unidade a alcança.
//!
//! É a mesma classe do `a_placed_instance_lands_a_screen_step_from_its_main`, e a mesma lição:
//! *um gate de unidade é cego à fiação do shell*.

use std::fs;

fn src(name: &str) -> String {
    fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// **A shell PUBLICA as peças** — sem isto a seção pinta zero linhas e o override é inalcançável.
#[test]
fn the_shell_publishes_the_piece_rows_from_the_single_door() {
    let s = src("render_loop/mod.rs");
    assert!(
        s.contains("ph2d_panel_vector::state::set_instance_pieces("),
        "a shell deixou de publicar as peças: a lista fica VAZIA, a seção não pinta linha \
         nenhuma, e o override volta a não ter porta — com todos os gates do motor VERDES"
    );
    assert!(
        s.contains("crate::vec_component_pieces::piece_rows("),
        "as linhas deixaram de vir da porta única. Uma segunda travessia daria uma ORDEM que pode \
         diferir da que resolve o clique — e o sintoma é o clique na linha 2 a recolorir a peça 3, \
         sem erro nenhum"
    );
}

/// **A cor escolhida no picker VOLTA para a peça.**
///
/// ⚠️ A swatch é alvo de picker: o clique dela só ABRE o OKLCH. Se ninguém ler o alvo de volta, a
/// cor é escolhida, o picker fecha e **nada muda** — e o seam do painel (que só prova que a swatch
/// é um alvo de picker) fica verde.
#[test]
fn the_picked_colour_is_read_back_onto_the_piece() {
    let s = src("render_loop/mod.rs");
    let at = s
        .find("crate::vec_component_pieces::colour_target(")
        .expect(
            "ninguém pergunta se o alvo do picker é a swatch de uma peça — a cor escolhida morre \
             no picker",
        );
    let block = &s[at..];
    let end = block
        .find("crate::vec_component_pieces::set_piece_colour(")
        .expect(
            "o alvo é resolvido e a cor nunca é escrita: a swatch abre o OKLCH e a escolha não \
             chega ao override",
        );
    assert!(
        block[..end].contains("blender_picker("),
        "a cor não vem do picker partilhado — um segundo lugar para *qual cor foi escolhida* \
         divergiria do que o OKLCH mostra"
    );
}

/// **O Swap ARMA o pick modal, e o clique seguinte é dele.**
///
/// ⚠️ Sem o arm o botão é um clique que não faz nada; e sem a variante entrar no `PathPick` o
/// clique cairia no picking/gizmo — o artista selecionaria o mestre em vez de trocar por ele.
#[test]
fn the_swap_arms_the_modal_pick_and_the_click_resolves_it() {
    let s = src("render_loop/mod.rs");
    assert!(
        s.contains("crate::vec_pick::PathPick::InstanceMain("),
        "o Swap deixou de armar o pick modal: o botão acende e não leva a lado nenhum"
    );
    let d = src("input_dispatch.rs");
    let at = d
        .find("crate::vec_pick::PathPick::InstanceMain(")
        .expect("o clique do pick não resolve o Swap — o pick fica armado para sempre");
    assert!(
        d[at..].contains("crate::vec_component_pieces::swap_main("),
        "o braço do Swap não chama a porta que troca o mestre"
    );
}
