//! ⭐⭐⭐ **O `Follow Curvature` ABRE ONDE FOI MEDIDO — e o botão não guarda uma segunda cópia.**
//!
//! # O report
//!
//! Enio, 2026-09-04, com foto e seta: *«a ponta problemática ainda não tem a densidade de faces
//! adequada como as outras»*. A tabela por ponta
//! ([`ph2d_quadfill::tip_rows`]) responde: no ponto do painel (`Follow Curvature = 0`) **dois
//! dos cinco espinhos saem com `3` a `5` faces dando a volta**, contra `17`–`58` com o knob no
//! máximo — e com `0` voltam a aparecer `2` pontas amputadas e `4` com a grade grossa.
//!
//! # ⛔ Por que este gate existe, e não só o comentário
//!
//! O `0,0` de antes tinha **razão escrita** (*«abre UNIFORME, que é o modo cujo resultado o
//! artista consegue prever»*) e uma **medição** por trás (2026-08-28: *«pede-se 400 % e a saída
//! move-se 7 %»*). As duas envelheceram — a fase zero passou a graduar com renormalização,
//! ganhou a calota por espinho e o acabamento ganhou o remate —, e *uma recusa medida responde
//! UMA pergunta*. Sem gate, a próxima leitura daquele comentário devolve o knob a `0`.
//!
//! # ⚠️ E a metade que já tinha mordido: DUAS cópias do mesmo default
//!
//! O `sculpt3d/birth.rs` tinha a sua própria cópia dos dois números, **com o comentário do
//! vizinho a dizer que um default escrito duas vezes é o que diverge**. Ele divergiu no dia em
//! que o painel mudou. Hoje o botão **deriva** do painel, e este gate proíbe o literal.

/// O valor MEDIDO — ver a tabela em `Sculpt3dUi::default`.
const MEDIDO: f32 = 1.0;

#[test]
fn o_knob_da_curvatura_abre_no_valor_medido() {
    let ui = ph2d_panel_sculpt3d::state::Sculpt3dUi::default();
    assert!(
        (ui.quad_adapt - MEDIDO).abs() < 1.0e-6,
        "⛔ o `Follow Curvature` abre em {} e o valor MEDIDO e' {MEDIDO} -- \
         com `0` a peca do dono sai com 2 de 5 pontas amputadas e 4 com a grade grossa, \
         e dois espinhos ficam com 3 a 5 faces dando a volta (tabela no doc do default)",
        ui.quad_adapt
    );
}

#[test]
fn o_botao_nao_guarda_uma_segunda_copia_dos_defaults() {
    let src = include_str!("../../src/sculpt3d/birth.rs");
    for campo in ["quad_detail", "quad_adapt"] {
        let derivado =
            format!("{campo}: ph2d_panel_sculpt3d::state::Sculpt3dUi::default().{campo}");
        assert!(
            src.contains(&derivado),
            "⛔ o `{campo}` do botao tem de vir do painel: `{derivado}`"
        );
        // ⚠️ **A ausência do LITERAL é a outra metade** — sem ela, alguém acrescenta a cópia ao
        // lado da derivada e as duas voltam a divergir em silêncio.
        for literal in [
            format!("{campo}: 0.0"),
            format!("{campo}: 0.5"),
            format!("{campo}: 1.0"),
        ] {
            assert!(
                !src.contains(&literal),
                "⛔ voltou um literal (`{literal}`) ao lado da porta que deriva do painel"
            );
        }
    }
}
