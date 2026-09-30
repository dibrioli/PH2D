//! Os gates da VOZ do relevo sem plano (`docs/3D/29`, D1) — irmão (`#[path]`)
//! do `painter_na_malha.rs`.

use super::voz_do_relevo;

/// ⭐⭐ **As QUATRO células:** só fala quando há relevo a moldar E não há
/// plano onde ele more. Calar nas outras três é metade do valor — uma voz que
/// fala sempre é ruído que o artista aprende a ignorar.
#[test]
fn a_voz_do_relevo_fala_so_sem_plano() {
    assert!(
        voz_do_relevo(true, false).is_some(),
        "relevo sem plano tem de falar"
    );
    assert!(
        voz_do_relevo(true, true).is_none(),
        "com o plano armado o relevo mora nele"
    );
    assert!(
        voz_do_relevo(false, false).is_none(),
        "uma pincelada de cor não perde nada"
    );
    assert!(voz_do_relevo(false, true).is_none());
}

/// ⭐ **A voz está LIGADA no pen-down** — ela é uma função pura e a prova de
/// comportamento da costura é `#[ignore]` + placa, logo o elo mede-se pelo
/// TEXTO (a lição das M19–M22 desta linha: sem isto, apagar a chamada passava
/// em tudo o que corre sem adaptador).
#[test]
fn a_voz_do_relevo_e_chamada_no_pen_down() {
    let src = include_str!("painter_na_malha.rs");
    // ⚠️ Sem prosa e SEM espaço nenhum: o `cargo fmt` parte a chamada em
    //   linhas, e uma agulha com espaços casaria zero depois dele.
    let codigo: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .flat_map(|l| l.chars().filter(|c| !c.is_whitespace()))
        .collect();
    let abre = codigo
        .find("scene.painter_abre(x,y)")
        .expect("o pen-down abre o traço");
    let voz = codigo
        .find("voz_do_relevo(painter.stroke_shapes_relief(),scene.stroke.tinta_fina.is_some(),)")
        .expect("o pen-down não pergunta pela voz do relevo");
    assert!(
        voz > abre,
        "a voz tem de vir DEPOIS do empréstimo do plano, que é o que ela lê"
    );
}
