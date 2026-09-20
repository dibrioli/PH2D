//! ⭐⭐⭐ **UM PASSO QUE NOMEIA UMA CAIXA AFIRMA QUE ELA ESTÁ NA TELA** — e o dono
//! aprova o smoke com o passo impossível dentro.
//!
//! A lei é a que o §31 desta linha pagou e o `CLAUDE.md` §5 já publica; aqui ela
//! cobre o passo **(13)** da `=14`, que a ordem do dono de 2026-09-19 acrescentou
//! (*«pincéis com Snake Hook e Grab ainda não têm a opção de usar a normal da
//! superfície para dar a direção da puxada»*).

use ph2d_sculpt3d::{Brush, Verb};

/// ⭐⭐⭐⭐ **O passo (13): o rótulo existe, e a caixa é oferecida EXACTAMENTE aos
/// dois pincéis que ele nomeia.**
///
/// ⛔⛔ **As três metades, e cada uma sozinha mente:**
///
/// 1. o **RÓTULO** que o roteiro imprime é o que o painel pinta (se ele for
///    renomeado, o dono procura uma palavra que não está na tela);
/// 2. a caixa é **OFERECIDA** com aqueles dois verbos na mão — senão o passo
///    manda marcar o que não aparece;
/// 3. e ela **NÃO** é oferecida a um verbo que o passo não nomeia — sem esta,
///    uma porta que devolvesse `true` a toda a gente passaria, e o roteiro
///    estaria a descrever mal o produto.
#[test]
fn o_passo_do_puxao_pela_normal_nomeia_uma_caixa_que_existe() {
    let roteiro = include_str!("scripts_topology.rs");
    for chave in [
        "panel.sculpt3d.puxa_pela_normal",
        "panel.sculpt3d.surface_only",
    ] {
        let rotulo = ph2d_i18n::tr(chave);
        assert!(
            roteiro.contains(&format!("`{rotulo}`")),
            "o roteiro da =14 manda o dono procurar `{rotulo}` e nao o nomeia"
        );
    }

    let mut ui = ph2d_panel_sculpt3d::Sculpt3dUi::default();
    for verbo in [Verb::Move, Verb::SnakeHook] {
        ph2d_panel_sculpt3d::state::switch_verb(&mut ui, verbo);
        assert!(
            ph2d_panel_sculpt3d::interruptor_oferecido(
                &ui,
                ph2d_panel_sculpt3d::ids::SCULPT3D_PUXA_PELA_NORMAL
            ),
            "{verbo:?}: o roteiro manda marcar uma caixa que o painel nao desenha"
        );
        assert!(
            Brush {
                verb: verbo,
                ..Brush::default()
            }
            .oferece_puxar_pela_normal(),
            "{verbo:?}: o motor e o painel discordam sobre a caixa"
        );
    }
    // A metade NEGATIVA — o carimbo não tem puxão para redireccionar.
    ph2d_panel_sculpt3d::state::switch_verb(&mut ui, Verb::Draw);
    assert!(
        !ph2d_panel_sculpt3d::interruptor_oferecido(
            &ui,
            ph2d_panel_sculpt3d::ids::SCULPT3D_PUXA_PELA_NORMAL
        ),
        "a caixa e' oferecida ao carimbo, que nao tem puxao nenhum"
    );
}
