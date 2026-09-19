//! ⭐⭐⭐ **O RELÓGIO DE UM TWEEN MORA NA SECÇÃO DELE** (suplente #22, W7 → W9).
//!
//! ⛔⛔ **Report do dono, 2026-09-19, em duas perguntas seguidas:** *«onde selecciono o tempo?»* e,
//! depois da primeira cura, *«porque usar timer para isso? por que não embutir na própria
//! secção?»*.
//!
//! # ⚠️⚠️ A PREMISSA DESTE FICHEIRO MORREU, e a morte está no diff
//!
//! Ele nasceu (W7) a medir uma **linha de leitura** — *«Duration 0,40 s — set in Timer 1, above.»* —
//! que dizia ao artista onde ir. A segunda pergunta do dono mostrou que isso era um **penso**: a
//! resposta certa não é apontar para outra secção, é **trazer os controlos para esta**. ⇒ a chave
//! `duration_lives_in_timer` foi **apagada**, e com ela os dois gates que a mediam.
//!
//! *Um gate cuja premissa morre reescreve-se com a morte visível, nunca se apaga em silêncio* — é
//! a mesma disciplina do `as_duas_colunas_coincidem_hoje_e_isso_nao_e_uma_lei` da escultura, que
//! morreu e renasceu em doze horas.
//!
//! # ⛔⛔ E porque isto NÃO é «duas superfícies sobre um valor»
//!
//! Os três controlos escrevem no **mesmo** `Timers[i]`, pela **mesma**
//! [`TimerFieldEdit`](ph2d_editor_core::screens::hero::TimerFieldEdit) e no **mesmo** índice que a
//! secção TIMERS. Não são duas leis: são **dois chamadores de uma porta**, que é o que o teclado e
//! o menu do `project_io` já fazem para gravar. A armadilha que os três chips de `Detail` da
//! escultura pagaram era outra — lá havia duas *implementações* a escrever o mesmo número.
//!
//! # ⏳ O que NÃO é medível nesta crate, e fica NOMEADO
//!
//! *Que a legenda chegue a PIXEL.* Ela é um `paint_text` sem `NodeId`, o `MockPanelHost` devolve
//! rectângulos de widget, e o Vello encaminha texto por `draw_glyphs` — **nenhum glifo entra numa
//! contagem de caminhos** (a lei que o arnês do L-System pagou em 2026-09-01). Os três CONTROLOS,
//! esses, têm costura a sério no [`super::seam_tween`].

use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};

/// O editor da secção — a fonte que as metades textuais lêem.
const EDITOR: &str = include_str!("../../src/sections/tween_editor.rs");

/// A largura real de uma linha de card do Inspector, à largura de omissão do painel.
fn largura_de_uma_linha() -> f32 {
    let painel = ph2d_tokens::INSPECTOR_W_PX - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX;
    // O `card_frame` recua `Spacing::Sm` de cada lado antes de pintar as rows.
    painel - 2.0 * Spacing::Sm.px()
}

/// ⭐⭐⭐ **A LEGENDA do relógio CABE na linha** — com o número do timer já lá dentro.
///
/// ⚠️ **O texto é o RESOLVIDO e não o molde:** medir `"… Timer {n} …"` mediria uma frase que
/// ninguém lê. E isto não é teoria — a **primeira** redacção desta secção (W7) media `309,9 px`
/// numa linha de `256,0` e teria shipado **cortada**; foi este gate que a apanhou.
///
/// **Mutações que devem sangrar:** alongar a legenda · medir o molde em vez do resolvido.
#[test]
fn a_legenda_do_relogio_cabe_numa_linha_do_inspector() {
    let largura = largura_de_uma_linha();
    let fonte = TypeToken::Sm.px();
    let mut ts = TextSystem::new();
    // O pior caso REAL: o slot mais alto que o painel endereça.
    let ultimo = ph2d_panel_inspector::ids::INSP_TWEEN_ROW.len();
    for n in [1_usize, ultimo] {
        let frase = ph2d_i18n::tr_with("panel.inspector.tween.clock_is_timer", &[("n", &n)]);
        assert!(
            !frase.contains('{'),
            "a legenda nao resolveu o numero do timer: {frase:?}"
        );
        let medida = ts.prefix_width(&frase, fonte);
        assert!(
            medida <= largura,
            "a legenda nao cabe: {frase:?} mede {medida:.1} px numa linha de {largura:.1}"
        );
    }
    // ⛔ O CONTROLO: a régua TEM de conseguir reprovar. Sem ele, um `prefix_width` que devolvesse
    //    zero deixava isto verde para sempre.
    let comprida = "Clock — Timer 1, exactly the same one that the Timers section above shows you.";
    assert!(
        ts.prefix_width(comprida, fonte) > largura,
        "controlo: a regua nao acusa uma frase que nao cabe"
    );
}

/// ⭐⭐ **A secção que a legenda NOMEIA existe** — ela promete *«same as in Timers»*.
///
/// ⚠️ *Uma frase que nomeia outra superfície do painel é uma AFIRMAÇÃO sobre ela*, e envelhece
/// sozinha no dia em que alguém a retirar.
#[test]
fn a_seccao_que_a_legenda_nomeia_existe() {
    assert!(
        ph2d_editor_core::ids::LIVE_SECTIONS
            .iter()
            .any(|(s, _)| *s == ph2d_editor_core::ids::INSP_LIVE_TIMER_SECTION),
        "a legenda do relogio manda o artista a` seccao TIMERS, e ela nao esta' na tabela"
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

/// ⭐⭐⭐ **O EDITOR pinta o relógio, lê-o do INSTANTÂNEO, e só depois dos presets.**
///
/// ⚠️ **A ordem é a lei:** os presets reescrevem a duração, e vê-la mudar **debaixo** do botão é o
/// que prova ao artista que UM clique fez as duas coisas.
///
/// ⛔ **E as três colunas vêm do instantâneo**, nunca do store: ler dali faria a caixa sobreviver à
/// troca de objecto — a lei que a §11 pagou com um report.
///
/// **Mutação que deve sangrar:** apagar o bloco do relógio do editor.
#[test]
fn o_editor_pinta_o_relogio_depois_dos_presets() {
    for campo in ["row.duracao_us", "row.repeat", "row.autostart"] {
        assert!(
            EDITOR.contains(campo),
            "o editor deixou de ler `{campo}` do instantaneo"
        );
    }
    for chave in [
        "panel.inspector.tween.clock_is_timer",
        "panel.inspector.tween.duration_seconds",
        "panel.inspector.tween.repeat",
        "panel.inspector.tween.autostart",
    ] {
        assert!(
            EDITOR.contains(chave),
            "o editor deixou de pintar `{chave}` — o relogio saiu da seccao do tween"
        );
    }
    let presets = EDITOR
        .find("panel.inspector.tween.preset")
        .expect("o editor deixou de pintar os presets");
    let relogio = EDITOR
        .find("if row.duracao_us.is_some()")
        .expect("o editor deixou de decidir se pinta o relogio");
    assert!(
        presets < relogio,
        "o bloco do relogio subiu para cima dos presets — ele tem de os SEGUIR, senao o artista \
         nao ve^ a duracao mudar debaixo do botao"
    );
}
