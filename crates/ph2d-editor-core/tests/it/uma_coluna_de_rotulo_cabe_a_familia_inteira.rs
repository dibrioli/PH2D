//! ⭐⭐⭐ **A PORTA QUE DIMENSIONA UMA COLUNA DE RÓTULO PARTILHADA — e o par ida-e-volta dela.**
//!
//! [`ph2d_editor_core::paint::label_column_width`] responde *«que largura reserva uma família de
//! controlos cujos rótulos alinham numa coluna só?»*. A resposta é a do membro mais largo, medida
//! no peso em que o [`ph2d_editor_core::paint::paint_text`] pinta.
//!
//! # ⛔⛔ Porque a porta existe
//!
//! A pergunta é sobre a **LISTA** e era sempre respondida com o item em mãos: um literal escolhido
//! pela palavra mais larga do dia em que alguém a escreveu. Medido em 2026-09-19 na barra de
//! transporte da timeline, o literal `52 px` (escolhido por `AutoKey`) cortava `Ping-Pong` em
//! inglês e **seis dos dez** rótulos da família no idioma de teste.
//!
//! # ⚠️ As duas metades, e porque nenhuma basta
//!
//! **Ida:** com a coluna que a porta devolve, o pintor não elide nenhum membro.
//! **Volta:** com **um pixel a menos**, ele elide — e elide exactamente o mais largo.
//!
//! *Sem a volta, uma porta que devolvesse `f32::INFINITY` passaria a ida com folga infinita*, e a
//! coluna de um painel ficaria larga como o ecrã. A volta é o que a prende ao valor TIGHT.

use ph2d_editor_core::paint::{label_column_width, paint_text, resolve};
use ph2d_editor_core::text_elide::elisao;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Uma família com membros de larguras bem diferentes — e o mais largo **não** é o último,
/// senão um `fold` que devolvesse o último passaria por acaso.
const FAMILIA: &[&str] = &["Loop", "Ping-Pong", "AutoKey", "Snap"];

fn pinta(rotulos: &[&str], orcamento: f32) -> Vec<elisao::Medido> {
    let mut ts = TextSystem::without_system_fonts();
    let mut cena = VectorScene::new();
    let cor = resolve(ColorToken::Text2, Theme::default());
    elisao::medindo(|| {
        for r in rotulos {
            paint_text(
                &mut ts,
                &mut cena,
                r,
                0.0,
                0.0,
                TypeToken::Sm.px(),
                orcamento,
                cor,
            );
        }
    })
    .1
}

fn coluna(rotulos: &[&str]) -> f32 {
    let mut ts = TextSystem::without_system_fonts();
    label_column_width(&mut ts, TypeToken::Sm.px(), rotulos.iter().copied())
}

/// ⭐ **IDA — com a coluna que a porta dá, nenhum membro da família é cortado.**
#[test]
fn com_a_coluna_da_porta_nenhum_rotulo_da_familia_e_cortado() {
    let col = coluna(FAMILIA);
    let medidos = pinta(FAMILIA, col);
    assert_eq!(
        medidos.len(),
        FAMILIA.len(),
        "o censo tem de ver cada rótulo uma vez — {medidos:?}"
    );
    for m in &medidos {
        assert!(
            m.coube(),
            "{:?} foi cortado para {:?} numa coluna de {col} px que se diz medida dele",
            m.texto,
            m.pintado
        );
    }
}

/// ⛔ **VOLTA — um pixel a menos e o mais largo É cortado.**
///
/// É a metade que prende a porta ao valor TIGHT: sem ela, devolver uma folga qualquer (ou
/// `f32::INFINITY`) passaria a ida. E ela nomeia **qual** membro cai — se cair outro, a porta
/// não está a medir o máximo, está a medir outra coisa que por acaso chega.
#[test]
fn um_pixel_a_menos_corta_exactamente_o_membro_mais_largo() {
    let col = coluna(FAMILIA);
    let medidos = pinta(FAMILIA, col - 1.0);
    let cortados: Vec<&str> = medidos
        .iter()
        .filter(|m| !m.coube())
        .map(|m| m.texto.as_str())
        .collect();
    assert_eq!(
        cortados,
        vec!["Ping-Pong"],
        "a {} px (um a menos que a coluna) devia cair SÓ o mais largo",
        col - 1.0
    );
}

/// ⚠️ **Uma família VAZIA reserva ZERO.** Não é um caso de borda decorativo: é a resposta certa
/// (não há rótulo), e é o que impede alguém de lhe pôr um piso escondido lá dentro — quem quiser
/// um mínimo põe-no na chamada, onde ele se vê.
#[test]
fn uma_familia_vazia_nao_reserva_coluna_nenhuma() {
    assert_eq!(coluna(&[]), 0.0);
}

/// ⚠️ **Ela mede no PESO em que o `paint_text` pinta.**
///
/// Medir em `Medium` e pintar em `SemiBold` corta `+0,74 %` a `+1,69 %` além da coluna — *na
/// fronteira exacta em que o corte existe* (a lição que o `prefix_width_weighted` já pagou em
/// 2026-08-30). O controlo é o próprio pintor: a coluna vinda da porta tem de bastar, e é o gate
/// da ida que o afirma. Aqui afirma-se a outra metade — que a porta **não** responde o mesmo que
/// uma medição num peso diferente, senão o par acima ficaria verde por coincidência.
#[test]
fn a_coluna_e_medida_no_peso_em_que_se_pinta() {
    let mut ts = TextSystem::without_system_fonts();
    let font = TypeToken::Sm.px();
    let porta = label_column_width(&mut ts, font, FAMILIA.iter().copied());
    let em_semibold = FAMILIA.iter().fold(0.0_f32, |w, r| {
        w.max(ts.prefix_width_weighted(r, font, ph2d_text::FontWeight::SEMI_BOLD))
    });
    assert!(
        porta < em_semibold,
        "a porta ({porta}) tinha de medir MAIS LEVE que o SemiBold ({em_semibold}) — se os dois \
         pesos dão o mesmo número, este gate deixou de distinguir seja o que for e o par \
         ida-e-volta acima passa a ser a única defesa"
    );
}
