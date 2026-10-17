use super::*;
use crate::visuals::{Chrome, Widgets};

const THEMES: [Theme; 12] = Theme::ALL;

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}

/// O azul e o laranja do dono (cena 25), os dois extremos, e o cinza onde o melhor par
/// preto/branco é o PIOR possível (`L ≈ 0,179`, `≈ 4,58:1`) — posto, nunca amostrado.
fn bodies() -> Vec<Color> {
    let mut v = vec![
        rgb(40, 120, 240),
        rgb(240, 140, 40),
        rgb(250, 250, 250),
        rgb(8, 8, 8),
        rgb(118, 118, 118),
        rgb(117, 117, 117),
        rgb(119, 119, 119),
    ];
    for r in (0..=255).step_by(51) {
        for g in (0..=255).step_by(51) {
            for b in (0..=255).step_by(51) {
                v.push(rgb(r as u8, g as u8, b as u8));
            }
        }
    }
    v
}

#[test]
fn a_tinta_escolhida_le_se_sobre_qualquer_corpo() {
    for theme in THEMES {
        for under in bodies() {
            let ink = readable_on(under, theme);
            assert!(
                ink.contrast_ratio(&under) >= INK_MIN,
                "{theme:?} {under:?} → {ink:?} = {:.2}:1",
                ink.contrast_ratio(&under)
            );
        }
    }
}

#[test]
fn a_tinta_e_do_tema_quando_o_tema_chega() {
    for theme in THEMES {
        let t1 = ColorToken::Text1.resolve(theme);
        let bg0 = ColorToken::Bg0.resolve(theme);
        for under in [t1, bg0] {
            let ink = readable_on(under, theme);
            let pole = if under == t1 { bg0 } else { t1 };
            if pole.contrast_ratio(&under) >= INK_MIN {
                assert_eq!(ink, Color { a: 255, ..pole }, "{theme:?}");
            }
        }
    }
}

#[test]
fn fora_do_escopo_a_resposta_e_a_de_sempre() {
    for theme in THEMES {
        for &t in ColorToken::ALL {
            assert_eq!(
                t.resolve(theme),
                t.resolve_unscoped(theme),
                "{t:?} {theme:?}"
            );
        }
    }
}

#[test]
fn dentro_do_escopo_cada_papel_responde_pelo_desenho() {
    let under = rgb(40, 120, 240);
    for theme in THEMES {
        let _g = dress(Body::Drawing { under: Some(under) });
        let ink = readable_on(under, theme);
        for &t in ColorToken::ALL {
            let c = t.resolve(theme);
            match t.role() {
                Role::Surface(Rank::Rest) | Role::Line => assert_eq!(c.a, 0, "{t:?}"),
                Role::Surface(_) => assert!(c.a > 0 && c.a < 64, "{t:?} {c:?}"),
                Role::Ink(InkLevel::Primary) => assert_eq!(c, ink, "{t:?}"),
                Role::Ink(InkLevel::OnAccent) => {
                    let accent = ColorToken::Accent.resolve(theme);
                    assert!(
                        c.contrast_ratio(&accent)
                            >= INK_MIN.min(
                                t.resolve_unscoped(theme)
                                    .contrast_ratio(&ColorToken::Accent.resolve_unscoped(theme))
                            ),
                        "{t:?} {theme:?}"
                    );
                }
                Role::Ink(_) => assert!(c.a < 255 && Color { a: 255, ..c } == ink, "{t:?}"),
                Role::Graphic => {
                    let raw = t.resolve_unscoped(theme);
                    if raw.a > 0 {
                        assert!(c.contrast_ratio(&under) >= GRAPHIC_MIN, "{t:?} {theme:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn o_corpo_chapado_pinta_a_cor_do_artista_e_o_estado_por_cima() {
    let colour = rgb(40, 120, 240);
    for theme in THEMES {
        let _g = dress(Body::Flat { colour });
        assert_eq!(ColorToken::Bg2.resolve(theme), colour);
        let w = Widgets::of(theme);
        assert_eq!(w.inactive.bg_fill, colour);
        assert_ne!(w.hovered.bg_fill, colour, "o hover tem de se ver no painel");
        assert_ne!(w.active.bg_fill, w.hovered.bg_fill);
        assert!(!w.inactive.bg_stroke.is_visible());
        assert_eq!(Chrome::of(theme).field_fill, colour);
    }
}

#[test]
fn as_duas_portas_do_tema_obedecem_ao_escopo() {
    for theme in THEMES {
        let _g = dress(Body::Drawing {
            under: Some(rgb(240, 140, 40)),
        });
        let w = Widgets::of(theme);
        for v in [w.noninteractive, w.inactive] {
            assert_eq!(v.bg_fill.a, 0, "{theme:?}");
            assert!(!v.bg_stroke.is_visible());
        }
        let c = Chrome::of(theme);
        assert_eq!(c.field_fill.a, 0, "{theme:?}");
        assert!(!c.field_border.is_visible() && !c.panel_border.is_visible());
    }
}

#[test]
fn sem_preenchimento_o_conteudo_fica_o_do_tema() {
    for theme in THEMES {
        let _g = dress(Body::Drawing { under: None });
        assert_eq!(ColorToken::Bg2.resolve(theme).a, 0);
        assert_eq!(
            ColorToken::Text1.resolve(theme),
            ColorToken::Text1.resolve_unscoped(theme)
        );
        assert_eq!(
            ColorToken::Accent.resolve(theme),
            ColorToken::Accent.resolve_unscoped(theme)
        );
    }
}

#[test]
fn o_escopo_nao_deixa_residuo_nem_num_panico() {
    let theme = Theme::Dark;
    let before = (
        ColorToken::Bg2.resolve(theme),
        Widgets::of(theme),
        Chrome::of(theme),
    );
    {
        let _a = dress(Body::Flat {
            colour: rgb(40, 120, 240),
        });
        {
            let _b = dress(Body::Drawing { under: None });
            assert_eq!(ColorToken::Bg2.resolve(theme).a, 0, "o de dentro manda");
        }
        assert_eq!(
            ColorToken::Bg2.resolve(theme),
            rgb(40, 120, 240),
            "volta ao de fora"
        );
    }
    let r = std::panic::catch_unwind(|| {
        let _g = dress(Body::Drawing { under: None });
        panic!("pintor partido");
    });
    assert!(r.is_err());
    assert_eq!(
        (
            ColorToken::Bg2.resolve(theme),
            Widgets::of(theme),
            Chrome::of(theme)
        ),
        before
    );
}
