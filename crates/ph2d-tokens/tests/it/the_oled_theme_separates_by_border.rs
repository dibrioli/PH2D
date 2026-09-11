//! ⭐⭐⭐ **No OLED o que separa duas superfícies é a MOLDURA, porque o contraste não pode.**
//!
//! O preset *Black (OLED)* do Godot tem `contrast = 0` — num fundo preto não há para onde
//! escurecer —, e é por isso que ele liga o `interface/theme/draw_extra_borders`. Medido nesta
//! casa (2026-09-07): no OLED o `bg-0`, o `bg-1`, o `bg-2` e o `panel-bg` são **os quatro
//! `#000000`**. Um cartão sobre um painel é, por preenchimento, invisível — e tem de ser.
//!
//! ⚠️⚠️ **A lista de pendências desta linha dizia «no OLED nenhum cartão é visível» e isso já era
//! FALSO.** A derivação da wave 1 liga o `extra_borders` no preset OLED, e ele chega às três
//! tabelas — widgets, painéis e campos. *A quinta pendência desta jornada que a medição achou já
//! feita: o placar de uma linha envelhece à velocidade das waves dela.*
//!
//! Este gate existe para que a cura não se perca outra vez: ela não está num pintor, está numa
//! **entrada de tema**, e nada no caminho de um cartão a menciona.

use ph2d_tokens::Theme;
use ph2d_tokens::visuals::{Chrome, Feel, Frame, Widgets, frame};

/// ⛔ **As superfícies do OLED são indistinguíveis por preenchimento — é a premissa.**
///
/// Se um dia elas deixarem de o ser, este gate deixa de descrever o problema que resolve, e a
/// moldura extra passa a ser decoração em vez de necessidade.
#[test]
fn the_oled_surfaces_are_indistinguishable_by_fill() {
    use ph2d_tokens::ColorToken as C;
    let preto = |t: C| {
        let c = t.resolve(Theme::Oled);
        (c.r, c.g, c.b) == (0, 0, 0)
    };
    for t in [C::Bg0, C::Bg1, C::Bg2, C::PanelBg] {
        assert!(
            preto(t),
            "`{}` deixou de ser preto puro no OLED: a premissa deste tema mudou e a moldura extra \\
             precisa de ser re-justificada",
            t.key()
        );
    }
}

/// ⭐⭐ **E por isso o OLED traça onde as outras famílias modernas não traçam.**
///
/// ⚠️ A régua é a DIFERENÇA entre o OLED e o `Dark`, não um número: as duas são modernas, a pele
/// plana é a mesma, e o que as separa é exactamente esta entrada.
#[test]
fn oled_draws_the_border_the_flat_skin_removes_elsewhere() {
    for (t, esperado) in [
        (Theme::Oled, true),
        (Theme::Dark, false),
        (Theme::Gray, false),
    ] {
        let w = Widgets::of(t);
        let c = Chrome::of(t);
        assert_eq!(
            w.inactive.bg_stroke.is_visible(),
            esperado,
            "{t:?}: a moldura de repouso de um WIDGET (o cartao le^-a por `Feel::Rest`)"
        );
        assert_eq!(
            c.panel_border.is_visible(),
            esperado,
            "{t:?}: a moldura de um PAINEL"
        );
        assert_eq!(
            c.field_border.is_visible(),
            esperado,
            "{t:?}: a moldura de um CAMPO"
        );
    }
}

/// ⛔ **E o cartão alcança-a**: a porta que ele usa (`Feel::Rest`) devolve uma moldura no OLED.
///
/// ⚠️ Sem este, as duas asserções acima ficariam verdes com a tabela certa e o **caminho** partido
/// — é a diferença entre *o tema declara uma moldura* e *o cartão recebe uma*.
#[test]
fn the_door_a_card_uses_returns_that_border_in_oled() {
    match frame(Theme::Oled, Feel::Rest) {
        Frame::Modern(s) => assert!(
            s.is_visible(),
            "a porta `frame(Oled, Rest)` — a que o cartao chama — devolveu moldura invisivel: no \\
             OLED isso deixa o cartao sem nada que o separe do painel"
        ),
        Frame::Classic => panic!("o OLED e' um tema MODERNO; a porta devolveu o ramo classico"),
    }
    match frame(Theme::Dark, Feel::Rest) {
        Frame::Modern(s) => assert!(
            !s.is_visible(),
            "o `Dark` voltou a tracar moldura de repouso: a pele plana perdeu-se"
        ),
        Frame::Classic => panic!("o `Dark` e' um tema MODERNO"),
    }
}
