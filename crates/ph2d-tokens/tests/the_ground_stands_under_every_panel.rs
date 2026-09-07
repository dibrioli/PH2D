//! ⭐⭐⭐ **O CHÃO da janela fica ABAIXO do painel — é isso que faz cada área ler-se como cartão.**
//!
//! Enio, 2026-09-07, com o Godot ao lado: *«os painéis, o canvas, a timeline, na Godot parecem
//! cards, e assim os espaços entre cards ficam legais… a aparência de cards me parece mais pro»* —
//! depois de a divisória de 4 px da wave 30 não ter ficado boa.
//!
//! ⛔⛔ **A razão de não ter ficado boa era o FUNDO, não o vão.** Medido: o painel desta casa é
//! `#131313` no Dark — mais fundo que o `bg-0` (`#1B1B1B`) e que o `bg-1` (`#1F1F1F`), que eram o
//! que o `paint_canvas_bg` punha na janela. ⇒ o vão de 4 px mostrava uma cor **mais clara** que as
//! superfícies que separava, que é o oposto de uma divisória. *Um espaço só se lê se o que aparece
//! nele estiver ATRÁS das duas coisas que ele separa.*
//!
//! # ⚠️ Um degrau ABSOLUTO, e a razão está medida
//!
//! A 1.ª derivação foi *«mais um passo na mesma escada»* (`base.lerp(BLACK, c · 2.4)`) e **satura
//! no tema claro**: ali o painel já está a `1/255` do branco. A 2.ª foi uma **fracção do painel**
//! (`panel.lerp(BLACK, 0.45)`) e deu `9/255` no Dark contra **`114`** no Light — *uma escada
//! relativa mede-se em passos diferentes conforme onde se está nela*. ⇒ o degrau é absoluto, e dá
//! `10/255` nos três temas com base não-preta.

use ph2d_tokens::{ColorToken as C, Theme};

/// A distância de canal entre dois tokens, no maior canal.
fn distance(a: C, b: C, t: Theme) -> i16 {
    let (x, y) = (a.resolve(t), b.resolve(t));
    let d = |p: u8, q: u8| i16::from(p) - i16::from(q);
    d(x.r, y.r)
        .abs()
        .max(d(x.g, y.g).abs())
        .max(d(x.b, y.b).abs())
}

/// ⭐⭐⭐ **O chão está abaixo do painel, e a distância lê-se — nos três temas que a têm.**
///
/// ⛔ O **OLED** fica de fora com mecanismo: a base dele é preta, a família multiplicativa colapsa
/// lá, e quem separa duas superfícies é a *Draw Extra Borders* — o mesmo que o `panel` já declara,
/// e que o gate `the_oled_theme_separates_by_border` mede.
#[test]
fn the_ground_is_a_readable_step_below_the_panel() {
    for t in [Theme::Dark, Theme::Gray, Theme::Light] {
        let d = distance(C::WindowGround, C::PanelBg, t);
        assert!(
            (8..=14).contains(&d),
            "{t:?}: o chao esta' a {d}/255 do painel — abaixo de 8 ele nao se le^ numa divisoria de \
             4 px, acima de 14 ele deixa de ser um degrau e passa a ser outra superficie"
        );
    }
}

/// ⚠️ **E ele é o mais FUNDO de todos** — se um tom passar por baixo dele, o chão deixa de ser chão.
#[test]
fn nothing_sits_below_the_ground() {
    for t in [Theme::Dark, Theme::Gray] {
        let ground = C::WindowGround.resolve(t);
        for other in [C::PanelBg, C::Bg0, C::Bg1, C::Bg2, C::BgElev] {
            let c = other.resolve(t);
            let soma = |x: ph2d_tokens::Color| u32::from(x.r) + u32::from(x.g) + u32::from(x.b);
            assert!(
                soma(ground) <= soma(c),
                "{t:?}: `{}` ({}) esta' mais fundo que o chao ({}) — o chao deixou de estar atras \
                 de tudo, e a divisoria passa a mostrar uma cor mais clara que os vizinhos",
                other.key(),
                soma(c),
                soma(ground)
            );
        }
    }
}

/// ⛔ **O clássico não se mexe**: lá o chão É o painel, logo a divisória não mostra nada — e é
/// exactamente isso que mantém o `PH2D_UI_NEW=0` byte a byte no que shipou.
#[test]
fn the_classic_look_is_untouched_because_its_ground_is_its_panel() {
    for t in [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
    ] {
        assert_eq!(
            distance(C::WindowGround, C::PanelBg, t),
            0,
            "{t:?}: o chao do classico deixou de ser o painel — a aparencia antiga mudou"
        );
    }
}
