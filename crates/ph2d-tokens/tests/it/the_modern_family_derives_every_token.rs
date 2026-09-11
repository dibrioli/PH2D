//! ⭐⭐⭐ **A família moderna deriva TODOS os tokens — e os apelidos da timeline são-no por
//! CONSTRUÇÃO.**
//!
//! Decisão do Enio (2026-09-04): o modelo é o Godot 4.6 «Modern» (MIT), o cinza e o azul são os
//! dele, e os presets são os da tabela dele. Um tema passa a ser cinco entradas; este gate afirma
//! o que isso compra e o que isso obriga.

use ph2d_tokens::derive::Inputs;
use ph2d_tokens::{ColorToken, Theme};

/// **Toda chave de `ColorToken` tem regra de derivação em todo tema moderno.**
///
/// ⚠️ A fábrica dos modernos é um `match` sobre a CHAVE, e um `match` sobre strings não é
/// exaustivo para o compilador — é este laço que o torna exaustivo para o repo: um token novo no
/// `color_tokens!` sem regra aqui reprova antes de estourar no primeiro quadro.
#[test]
fn every_token_derives_in_every_modern_theme() {
    for theme in Theme::MODERN {
        for token in ColorToken::ALL {
            let c = token.factory(theme);
            // Um token opaco no clássico continua opaco no moderno (o achatamento é a decisão
            // do porte); um com alfa continua com alfa. Só se afirma que ALGUMA cor saiu.
            let _ = c;
        }
    }
}

/// ⭐ **Os 16 slots `timeline-*` são apelidos EXACTOS dos slots gerais — por construção, não por
/// coincidência.** No clássico isto era medido contra o `tokens.json` e ficou como decisão do
/// Enio (`spec/02 §2`, degrau B); na derivação a pergunta dissolve-se: não há valor a escolher.
#[test]
fn the_timeline_slots_are_aliases_by_construction() {
    let pairs: &[(ColorToken, ColorToken)] = &[
        (ColorToken::TimelineCurve, ColorToken::Accent),
        (ColorToken::TimelineHandle, ColorToken::Accent),
        (ColorToken::TimelineKeySelected, ColorToken::Accent),
        (ColorToken::TimelineLoopBrace, ColorToken::Accent),
        (ColorToken::TimelinePlayhead, ColorToken::Accent),
        (ColorToken::TimelineSummaryRing, ColorToken::Accent),
        (ColorToken::TimelineHandleLine, ColorToken::AccentSoft),
        (ColorToken::TimelineLoopRegion, ColorToken::AccentSoft),
        (ColorToken::TimelineRowAlt, ColorToken::Bg2),
        (ColorToken::TimelineRulerBg, ColorToken::Bg2),
        (ColorToken::TimelineMarker, ColorToken::Warn),
        (ColorToken::TimelineSummaryKey, ColorToken::Warn),
        (ColorToken::TimelineKeyActive, ColorToken::AccentPress),
        (ColorToken::TimelineMissing, ColorToken::Danger),
        (ColorToken::TimelineKey, ColorToken::Text1),
        (ColorToken::TimelineRulerTick, ColorToken::Text3),
    ];
    assert_eq!(pairs.len(), 16, "os dezasseis, contados");
    for theme in Theme::MODERN {
        for (alias, general) in pairs {
            assert_eq!(
                alias.factory(theme),
                general.factory(theme),
                "{theme:?}: {} devia ser {}",
                alias.key(),
                general.key()
            );
        }
    }
}

/// **As entradas são as do Godot, literalmente** — o cinza e o azul que o dono escolheu.
///
/// `#292929` é `Color(0.161, 0.161, 0.161)` e `#569eff` é `Color(0.337, 0.62, 1.0)` — a tabela
/// `color_preset == "Default"` do `editor_theme_manager.cpp` (MIT).
#[test]
fn the_dark_preset_is_godots_default() {
    let d = Inputs::of(Theme::Dark).expect("moderno");
    let base = d.base.color();
    let accent = d.accent.color();
    assert_eq!(
        (base.r, base.g, base.b),
        (0x29, 0x29, 0x29),
        "o cinza do Godot"
    );
    assert_eq!(
        (accent.r, accent.g, accent.b),
        (0x56, 0x9e, 0xff),
        "o azul do Godot"
    );
    assert!((d.contrast - 0.3).abs() < 1e-6);
    // E o tema chega ao app com esses dois números intactos.
    assert_eq!(ColorToken::Bg2.factory(Theme::Dark), base, "bg-2 e' a base");
    assert_eq!(ColorToken::Accent.factory(Theme::Dark), accent);
}

/// **A família clássica NÃO se mexeu** — `Inputs::of` é `None` para ela, e a fábrica continua a
/// ler a tabela gerada do `tokens.json`. É a metade que o interruptor `PH2D_UI_NEW=0` promete.
#[test]
fn the_classic_family_has_no_inputs_and_keeps_its_table() {
    for theme in Theme::CLASSIC {
        assert!(Inputs::of(theme).is_none(), "{theme:?} nao e' derivado");
    }
    // O `forge` continua a ser o magenta tingido de sempre — se a derivação lhe tocasse, o acento
    // dele deixaria de ter matiz 340.
    let forge = ColorToken::Accent.factory(Theme::Forge);
    assert!(forge.r > forge.g, "o acento do forge continua magenta");
}

/// ⭐⭐ **O FUNDO DO CANVAS é o que o dono aprovou, byte a byte** — e é o gate que faltava quando
/// a tentativa anterior o moveu.
///
/// O `Bg1` é o fundo do canvas (`hero::canvas_backdrop`, a porta única que o `canvas.rs`
/// documenta) **e** o fundo dos cartões. Em 2026-09-05 eu clareei-o para separar o cartão do
/// painel, e o dono devolveu-o no smoke: *«mudou a cor do canvas»*. ⇒ o `Bg1` e o `Bg0` ficam
/// presos às fórmulas que ele viu, e quem se move para dar contraste é o **painel**.
///
/// ⛔ **A ESCADA de superfícies do Godot foi construída e REVERTIDA** (2026-09-05): pôr o painel
/// na `base` e subir `Bg1`/`Bg2`/`Bg3`/`BgElev` pela família `surface_*`/`button_*` dele resolve
/// o contraste do cartão **e clareia o CANVAS junto** — o `Bg1` responde às duas perguntas neste
/// app (`hero::canvas_backdrop` e os sete cartões do Painter). *«mudou a cor do canvas»* (Enio,
/// no smoke seguinte). O que fica são os dois gates acima: o canvas preso ao que ele aprovou, e
/// o painel a DESCER para dar o degrau.
#[test]
fn the_canvas_ground_is_the_one_the_owner_approved() {
    for theme in Theme::MODERN {
        let r = Inputs::of(theme).expect("moderno").roles();
        assert_eq!(
            ColorToken::Bg1.factory(theme),
            r.dark_3.color(),
            "{theme:?}: o fundo do CANVAS mexeu-se — ver `hero::canvas_backdrop`"
        );
        assert_eq!(
            ColorToken::Bg0.factory(theme),
            r.dark_1.color(),
            "{theme:?}: a moldura do canvas mexeu-se"
        );
    }
}

/// ⭐⭐ **Um CARTÃO destaca-se do PAINEL** — *«o fundo dos cards tem tão pouco contraste com o
/// fundo dos painéis que quase não podem ser diferenciados»* (Enio, 2026-09-05).
///
/// Medido no dia do report: **4/255** no Dark. A barra é **12/255**, e quem se move é o painel
/// (o `Bg1` do cartão é também o canvas — ver o gate acima). ⛔ O OLED fica de fora: base preta e
/// contraste `0` colapsam a família, e quem separa lá é a *Draw Extra Borders*, como no Godot.
#[test]
fn a_card_stands_off_its_panel() {
    for theme in [Theme::Dark, Theme::Gray, Theme::Light] {
        let grey = |t: ColorToken| i32::from(t.factory(theme).g);
        let step = grey(ColorToken::Bg1) - grey(ColorToken::PanelBg);
        let sign = if theme == Theme::Light { -1 } else { 1 };
        assert!(
            step * sign >= 12,
            "{theme:?}: o cartao esta' a {} de 255 do painel (bg1 {:?}, panel {:?})",
            step.abs(),
            ColorToken::Bg1.factory(theme),
            ColorToken::PanelBg.factory(theme)
        );
    }
}

/// **Um tema moderno claro escurece o que o escuro clareia** — o contraste negativo do preset
/// *Light* do Godot é o que faz a «elevação» seguir a ordem natural num fundo claro.
#[test]
fn light_elevates_by_darkening_and_dark_by_lightening() {
    let lum = |c: ph2d_tokens::Color| c.relative_luminance();
    let dark_bg1 = lum(ColorToken::Bg1.factory(Theme::Dark));
    let dark_bg3 = lum(ColorToken::Bg3.factory(Theme::Dark));
    assert!(dark_bg3 > dark_bg1, "no escuro, mais elevado = mais claro");
    let light_bg1 = lum(ColorToken::Bg1.factory(Theme::Light));
    let light_bg3 = lum(ColorToken::Bg3.factory(Theme::Light));
    assert!(
        light_bg3 < light_bg1,
        "no claro, mais elevado = mais escuro"
    );
}
