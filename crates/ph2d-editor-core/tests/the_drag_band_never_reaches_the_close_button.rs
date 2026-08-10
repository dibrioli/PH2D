//! **A faixa de arrasto do título nunca alcança o botão de fechar.**
//!
//! O doc da [`PANEL_HEADER_CLOSE_RESERVE`] enuncia o invariante em prosa — *"passada ao
//! `panel_drag_handle_rect` para que a área de arrasto alargada não SOMBREIE o hit do fechar"* — e
//! o fazia cumprir por uma **CÓPIA do número de que ele depende**. As duas contas discordavam:
//!
//! | | conta | onde |
//! |---|---|---|
//! | o fechar começa em | `w − PANEL_HEAD_PAD(18) − Spacing::Xl2(24)` = `w − 42` | `panel_close_button_rect` |
//! | a faixa acabava em | `w − PANEL_HEADER_CLOSE_RESERVE(40)` | `panel_drag_handle_rect` |
//!
//! ⚠️ **Folga: −2 px, na escala de FÁBRICA.** Não era "errado ao vivo": era errado no produto que
//! shipa. O comentário da constante dizia *"Xl2 close icon size + padding"* e somava `24 + 16`,
//! mas o `PANEL_HEAD_PAD` é **18** (token de chrome, não `Spacing::Xl`) — a derivação nomeou o
//! padding errado e saiu 2 px curta.
//!
//! ⚠️ **E o `HitIndex` resolve por ÚLTIMO-REGISTADO** (`hit.rs`, `.rects.iter().rev()`), então a
//! consequência depende da ordem de registo de cada painel. Varridos os dezasseis que registam as
//! duas coisas: **quinze** registam o fechar DEPOIS (e a sobreposição é inócua) e **um** — o
//! `ph2d-panel-painter-layers`, o painel onde o artista vive — regista a faixa em último
//! (`paint.rs:426` contra `:105`) ⇒ ali os 2 px da esquerda do X **arrastavam o painel em vez de
//! o fechar**.
//!
//! # Porque a cura é a PORTA, e não a constante
//!
//! Tornar a reserva viva obrigaria a renomear uma `pub const` lida por **21 crates de painel**,
//! metade delas em linhas VIVAS nesta janela — a colisão de mesmo-símbolo que a DIRETRIZ §1.5.5
//! nomeia. E não resolveria o mecanismo: o número continuaria a ser uma segunda cópia.
//!
//! O `panel_drag_handle_rect` já recebe o `panel`, e o rect do fechar é **função pura dele**. Então
//! a porta faz a pergunta em vez de confiar no que lhe passaram, e o `min` tem a propriedade que
//! torna isto seguro para os 21 chamadores: **ele só ENCOLHE a faixa, nunca a alarga** ⇒ nenhum
//! painel pode passar a sombrear algo que não sombreava.
//!
//! ⚠️ O gate afirma a **PROPRIEDADE** (*a faixa pára antes do fechar*), varrida sobre escalas
//! AUTORADAS — não a fórmula. Uma expectativa escrita com a fórmula do produto seria o oráculo
//! auto-referente que esta auditoria passou a semana a arrancar.

use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_ADD_RESERVE, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT,
    panel_close_button_rect, panel_drag_handle_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::num_overrides::{NumValue, clear_num_overrides, set_num_override};
use ph2d_tokens::{NumToken, Spacing, Theme, num_runtime};

/// Um painel flutuante de tamanho plausível.
fn panel() -> Rect {
    Rect::new(120.0, 80.0, 320.0, 480.0)
}

/// A borda direita da faixa e a borda esquerda do fechar, na mesma régua.
fn band_right_and_close_left(reserve: f32) -> (f32, f32) {
    let p = panel();
    let band = panel_drag_handle_rect(p, PANEL_HEADER_H_DEFAULT, reserve);
    let close = panel_close_button_rect(p);
    (band.x + band.w, close.x)
}

/// Veste a escala com um `Spacing::Xl2` autorado e publica-a, como o quadro faz.
fn author_xl2(px: f32) {
    set_num_override(
        Theme::Forge,
        NumToken::Spacing(Spacing::Xl2),
        Some(NumValue::Literal(px)),
    )
    .expect("um literal valido entra");
    num_runtime::publish(Theme::Forge);
    assert_eq!(
        Spacing::Xl2.px(),
        px,
        "a escala autorada nao chegou a estar VIVA — o resto deste gate mediria a fabrica"
    );
}

/// **A escala volta à fábrica.** As tabelas são thread-local, mas os testes de um binário partilham
/// threads: deixar uma escala vestida faz o teste seguinte medir o mundo de outro.
fn back_to_factory() {
    clear_num_overrides();
    num_runtime::publish(Theme::Forge);
}

/// **A propriedade, na escala de FÁBRICA.** Nasceu VERMELHO: −2 px.
#[test]
fn the_drag_band_stops_before_the_close_hit() {
    back_to_factory();
    let (band_right, close_left) = band_right_and_close_left(PANEL_HEADER_CLOSE_RESERVE);
    assert!(
        band_right <= close_left,
        "a faixa de arrasto acaba em {band_right} e o fechar comeca em {close_left} — ela come \
         {} px do botao, e num painel que registre a faixa por ULTIMO esses pixeis arrastam em vez \
         de fechar",
        band_right - close_left
    );
}

/// **CONTROLE.** A faixa continua a ser uma faixa.
///
/// ⚠️ Sem isto, `band.w = 0` satisfaz o gate acima e o painel deixa de se poder arrastar — a
/// correção que passa no teste e destrói a feature.
#[test]
fn the_band_is_still_most_of_the_title_bar() {
    back_to_factory();
    let p = panel();
    let band = panel_drag_handle_rect(p, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
    assert!(
        band.w > p.w * 0.8,
        "a faixa mede {} de um painel de {} — deixou de ser agarravel",
        band.w,
        p.w
    );
    assert!(band.h > 0.0, "a faixa nao tem altura");
}

/// **A propriedade sobrevive a QUALQUER escala autorada.**
///
/// ⚠️ É esta metade que a constante não consegue ter: `Spacing::Xl2` é editável pelo artista
/// (`NumToken::ALL` cobre toda a escala) e `is_a_length` só exige `finito && >= 0` — não há teto.
/// Um número copiado fica onde está enquanto o ícone cresce.
#[test]
fn the_drag_band_stops_before_the_close_hit_under_any_authored_scale() {
    for px in [24.0_f32, 32.0, 48.0, 64.0, 96.0] {
        back_to_factory();
        author_xl2(px);
        let (band_right, close_left) = band_right_and_close_left(PANEL_HEADER_CLOSE_RESERVE);
        assert!(
            band_right <= close_left,
            "com `spacing.2xl` autorado em {px} px a faixa acaba em {band_right} e o fechar comeca \
             em {close_left} — {} px de sobreposicao",
            band_right - close_left
        );
    }
    back_to_factory();
}

/// **A porta só ENCOLHE.** É a propriedade que a torna segura para os 21 chamadores.
///
/// A Hierarquia passa a reserva do par fechar+adicionar (80), muito à esquerda do fechar: o clamp
/// não pode puxar a faixa de volta para a direita e descobrir o botão de adicionar.
#[test]
fn the_clamp_never_widens_a_band_the_caller_made_narrower() {
    back_to_factory();
    let p = panel();
    let band = panel_drag_handle_rect(p, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_ADD_RESERVE);
    assert!(
        (band.x + band.w - (p.x + p.w - PANEL_HEADER_ADD_RESERVE)).abs() < 0.01,
        "a reserva do par fechar+adicionar ({PANEL_HEADER_ADD_RESERVE}) devia mandar, e a faixa \
         acabou em {} em vez de {}",
        band.x + band.w,
        p.x + p.w - PANEL_HEADER_ADD_RESERVE
    );
}

/// **PIN da derivação de fábrica.** A constante é o que a sua própria prosa diz que é.
///
/// ⚠️ Ela continua a ser uma cópia — o que este pin compra é que a cópia **não pode driftar em
/// silêncio**: mudar o `panel-head-pad` no `tokens.json` sem mexer aqui fica vermelho. A defesa
/// contra a AUTORIA é a porta; esta é a defesa contra a EDIÇÃO da fábrica.
#[test]
fn the_reserve_is_the_pad_plus_the_icon_at_factory_scale() {
    back_to_factory();
    assert_eq!(
        PANEL_HEADER_CLOSE_RESERVE,
        PANEL_HEAD_PAD + Spacing::Xl2.factory_px(),
        "a reserva ({PANEL_HEADER_CLOSE_RESERVE}) nao e' o inset do cabecalho ({PANEL_HEAD_PAD}) \
         mais o icone de fechar ({}) — foi assim que ela nasceu 2 px curta",
        Spacing::Xl2.factory_px()
    );
}
