//! ⭐⭐⭐ **A secção TWEEN DIZ ONDE MORA O TEMPO** (suplente #22).
//!
//! ⛔⛔ **Report do dono, 2026-09-19, no smoke da `PH2D_TWEEN_SMOKE=1`:** *«onde selecciono o
//! tempo?»*. Um tween é uma função **pura** do relógio, logo ele **não tem duração nenhuma** — quem
//! a tem é o `Timers[i]` do mesmo índice, noutra secção do painel, dezassete secções acima.
//!
//! ⚠️⚠️ *O painel SABIA o número e não o dizia.* O instantâneo já carregava *«há relógio neste
//! índice?»* como um `bool` — ele tinha a informação para responder, e a forma como a guardava
//! deitava fora a metade que interessa. ⇒ o `bool` virou `Option<u64>` e o editor lê-o.
//!
//! # ⛔ As DUAS metades, e porque a terceira não existe aqui
//!
//! 1. **A frase CABE numa linha do Inspector** — medida com o sistema de texto REAL, à largura de
//!    omissão do painel. *Uma resposta cortada a meio é a mesma pergunta outra vez.*
//! 2. **O editor LÊ o campo e passa a chave** — por `include_str!`, que deixa de compilar se o
//!    ficheiro mudar de sítio.
//!
//! ⏳ **A terceira — *ela chega a PIXEL* — NÃO é medível nesta crate, e fica NOMEADA:** a linha é
//! um `paint_text` sem `NodeId`, e o `MockPanelHost` devolve rectângulos de widget; o Vello
//! encaminha texto por `draw_glyphs`, logo **nenhum glifo entra numa contagem de caminhos** (a lei
//! que o arnês do L-System pagou em 2026-09-01). *É a mesma dívida que o `CLAUDE.md` §5 já nomeia
//! sobre as 26 secções opcionais deste painel.*

use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};

/// O editor da secção — a fonte que as duas metades lêem.
const EDITOR: &str = include_str!("../../src/sections/tween_editor.rs");

/// A largura real de uma linha de card do Inspector, à largura de omissão do painel.
fn largura_de_uma_linha() -> f32 {
    let painel = ph2d_tokens::INSPECTOR_W_PX - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX;
    // O `card_frame` recua `Spacing::Sm` de cada lado antes de pintar as rows.
    painel - 2.0 * Spacing::Sm.px()
}

/// ⭐⭐⭐ **A resposta CABE na linha** — com o número e o slot já lá dentro.
///
/// ⚠️ **O texto é o RESOLVIDO e não o molde:** medir `"Duration: {s} s…"` mediria uma frase que
/// ninguém lê — os dois marcadores custam menos do que `0.40` e `1`, logo o molde cabe onde a
/// frase não cabe.
///
/// **Mutações que devem sangrar:** alongar a frase · medir o molde em vez do resolvido.
#[test]
fn a_frase_do_tempo_cabe_numa_linha_do_inspector() {
    let largura = largura_de_uma_linha();
    let fonte = TypeToken::Sm.px();
    let mut ts = TextSystem::new();
    // O pior caso REAL: dois algarismos de segundos e o slot mais alto que o painel oferece.
    let ultimo_slot = ph2d_panel_inspector::ids::INSP_TWEEN_ROW.len();
    for (s, n) in [("0.40", 1_usize), ("12.00", ultimo_slot)] {
        let frase = ph2d_i18n::tr_with(
            "panel.inspector.tween.duration_lives_in_timer",
            &[("s", &s), ("n", &n)],
        );
        let medida = ts.prefix_width(&frase, fonte);
        assert!(
            medida <= largura,
            "a resposta ao dono nao cabe: {frase:?} mede {medida:.1} px numa linha de {largura:.1}"
        );
    }
    // ⛔ O CONTROLO: a régua TEM de conseguir reprovar. Sem ele, um `prefix_width` que devolvesse
    //    zero deixava isto verde para sempre.
    let comprida = "Duration: 0.40 s — set it in the Timers section, timer 1, which lives above.";
    assert!(
        ts.prefix_width(comprida, fonte) > largura,
        "controlo: a regua nao acusa uma frase que nao cabe"
    );
}

/// ⭐⭐⭐ **O «above» da frase é uma AFIRMAÇÃO, e ela é DERIVADA da tabela das secções.**
///
/// ⚠️ *Uma palavra de POSIÇÃO envelhece sozinha no dia em que alguém reordenar o painel* — e o
/// artista que role para baixo à procura do relógio nunca o encontra. A tabela
/// [`ph2d_editor_core::ids::LIVE_SECTIONS`] é a ordem em que o painel as pinta, logo a frase pode
/// dizer *«above»* exactamente enquanto isto for verdade.
///
/// **Mutação que deve sangrar:** trocar as duas entradas de lugar na tabela.
#[test]
fn a_seccao_dos_relogios_esta_mesmo_acima_da_do_tween() {
    let pos = |id| {
        ph2d_editor_core::ids::LIVE_SECTIONS
            .iter()
            .position(|(s, _)| *s == id)
            .unwrap_or_else(|| panic!("{id:?} nao esta' na tabela das seccoes vivas"))
    };
    let timers = pos(ph2d_editor_core::ids::INSP_LIVE_TIMER_SECTION);
    let tween = pos(ph2d_editor_core::ids::INSP_LIVE_TWEEN_SECTION);
    assert!(
        timers < tween,
        "a seccao TIMERS esta' na posicao {timers} e a do TWEEN na {tween} — a frase promete          «above» e manda o artista rolar para o lado errado"
    );
}

/// ⭐⭐ **A queixa do relógio a ZERO NOMEIA o timer** — as duas curas ficam em sítios diferentes.
///
/// ⚠️ *Dizer «não há relógio» a quem tem um relógio a zero manda-o anexar um segundo*, e aí ele fica
/// com dois tweens e um deles mudo.
#[test]
fn a_queixa_do_relogio_parado_nomeia_o_timer() {
    let frase = ph2d_i18n::tr_with(
        "panel.inspector.tween.the_timer_here_has_no_duration",
        &[("n", &2_usize)],
    );
    assert!(
        frase.contains('2') && !frase.contains('{'),
        "a queixa nao resolveu o numero do timer: {frase:?}"
    );
    let mut ts = TextSystem::new();
    let medida = ts.prefix_width(&frase, TypeToken::Sm.px());
    assert!(
        medida <= largura_de_uma_linha(),
        "a queixa nao cabe: {medida:.1} px"
    );
}

/// ⭐⭐⭐ **O EDITOR lê o campo e pinta a frase** — a metade que prova que a linha não é só uma chave
/// na tabela.
///
/// ⚠️ **Ela é textual de propósito**, e é a única forma disponível aqui: a alternativa (contar
/// geometria) é **cega a glifos** — ver o cabeçalho.
///
/// **Mutação que deve sangrar:** apagar o bloco do readout do editor.
#[test]
fn o_editor_le_a_duracao_e_pinta_a_frase() {
    assert!(
        EDITOR.contains("row.duracao_us"),
        "o editor deixou de ler a duracao do relogio"
    );
    assert!(
        EDITOR.contains("panel.inspector.tween.duration_lives_in_timer"),
        "o editor deixou de pintar a frase que responde «onde selecciono o tempo?»"
    );
    // ⚠️ **A ORDEM é a lei**: ela vem DEPOIS dos presets, porque eles reescrevem este número e vê-lo
    //    mudar debaixo do botão é o que prova ao artista que UM clique fez as duas coisas.
    let presets = EDITOR
        .find("panel.inspector.tween.preset")
        .expect("o editor deixou de pintar os presets");
    let tempo = EDITOR
        .find("panel.inspector.tween.duration_lives_in_timer")
        .expect("ja' acusado acima");
    assert!(
        presets < tempo,
        "a linha do tempo subiu para cima dos presets — ela tem de os SEGUIR"
    );
}
