//! Onda 4, itens 6 e 7a: a face da tecla é o token de cromo `key-face` (o mesmo tom aprovado, byte a byte) e o
//! numérico ganha as segundas legendas da foto do dono (End ↓ PgDn ← → Home ↑ PgUp Ins Del), só onde cabem.

use super::lista::onze;
use super::{Mesa, em_cada_modo_de_texto};
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::keys_fit::REST_FACE;
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_app_input_map::view_keyboard::sub_legend::{SubLegend, SubLook, nav_of, sub_ink};
use ph2d_editor_core::zones::Rect;
use ph2d_input::{Binding, Key};
use ph2d_tokens::{ColorToken, Theme, categorical};

/// As áreas reais da aba (1600×1000 e 1280×720), uma estreita (rola) e uma larga (1920×1080, o positivo).
const AREAS: [Rect; 4] = [
    Rect::new(308.0, 28.0, 988.0, 890.0), // LITERAL-PX-OK: a área central a 1600×1000
    Rect::new(297.0, 28.0, 690.0, 604.0), // LITERAL-PX-OK: a área central a 1280×720
    Rect::new(297.0, 28.0, 360.0, 604.0), // LITERAL-PX-OK: estreita, abaixo do piso
    Rect::new(308.0, 28.0, 1308.0, 970.0), // LITERAL-PX-OK: a área central a 1920×1080
];

/// Numpad 8, ligado a duas acções (plaquinha e barras à vista).
const KP8: Key = Key(0xF748);

/// ⭐ Item 6: `key-face` é, em TODO tema, o valor resolvido do `KanbanCardRaised` de hoje (o tom aprovado): a
/// imagem antes/depois é a mesma cor a cor, e cada barra das 12 cores mede o mesmo contraste contra ela. Controlo:
/// um neutro de cromo (`Bg3`) difere em algum tema — a comparação vê uma troca.
#[test]
fn a_face_da_tecla_e_o_token_de_cromo_com_o_tom_aprovado() {
    assert_eq!(
        REST_FACE,
        ColorToken::KeyFace,
        "a face de repouso é a de cromo"
    );
    let mut controlo = 0;
    for t in Theme::ALL {
        let (novo, antes) = (
            ColorToken::KeyFace.resolve(t),
            ColorToken::KanbanCardRaised.resolve(t),
        );
        assert_eq!(novo, antes, "{t:?}: key-face byte a byte = o tom aprovado");
        controlo += usize::from(ColorToken::Bg3.resolve(t) != antes);
        for i in 0..12 {
            let bar = categorical(i).resolve(t);
            let (a, b) = (bar.contrast_ratio(&novo), bar.contrast_ratio(&antes));
            assert!((a - b).abs() < 1e-12, "{t:?} cor {i}: {a:.3} ≠ {b:.3}");
        }
    }
    assert!(controlo > 0, "controlo: Bg3 tem de diferir do tom aprovado");
}

fn pinta(a: Rect) -> Mesa {
    let mut m = Mesa::aberta(View::Keyboard);
    let mut map = onze();
    let ids: Vec<_> = map.actions().iter().map(|a| a.id).take(2).collect();
    for id in ids {
        map.get_mut(id)
            .expect("acção")
            .bindings
            .push(Binding::Key(KP8));
    }
    m.hero.input_map = map;
    m.frame();
    m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
    m.paint_in(a);
    m
}

fn dentro(a: Rect, b: Rect) -> bool {
    a.x >= b.x - 0.01
        && a.y >= b.y - 0.01
        && a.x + a.w <= b.x + b.w + 0.01
        && a.y + a.h <= b.y + b.h + 0.01
}

/// ⭐ Item 7a: nas áreas reais e nos dois modos de texto, as segundas legendas aparecem TODAS ou nenhuma; cada uma
/// inteira (nunca cortada), por baixo do número, dentro da tecla e da plaquinha, menor que ele; onde a medida não
/// deixa lugar (1600, 1280, estreita) fica sem nenhuma, e a 1920 tem todas — os dois lados da medida.
#[test]
fn as_segundas_legendas_do_numerico_cabem_inteiras_ou_somem() {
    em_cada_modo_de_texto(|| {
        let mut vistas = Vec::new();
        for a in AREAS {
            let m = pinta(a);
            let doc = &m.tab().doc;
            let subs: &[SubLook] = &doc.kb.subs;
            let at = format!("área {}", a.w);
            eprintln!(
                "{at}: {} segundas legendas, passo {:?}",
                subs.len(),
                doc.kb.fit
            );
            assert!(
                subs.is_empty() || subs.len() == 10,
                "{at}: todas ou nenhuma ({})",
                subs.len()
            );
            let cortadas: Vec<&str> = doc.census.elided();
            for s in subs {
                assert!(nav_of(s.key).is_some(), "{at}: {:?} tem segunda", s.key);
                if let SubLegend::Text(t) = &s.legend {
                    assert!(!cortadas.contains(&t.as_str()), "{at}: «{t}» cortada");
                    assert_eq!(
                        t,
                        nav_of(s.key).expect("nav").cap_short(),
                        "{at}: da tabela"
                    );
                }
                assert!(
                    dentro(s.sub, s.key_rect),
                    "{at}: {:?} dentro da tecla",
                    s.key
                );
                assert!(
                    s.sub.y >= s.main.y + s.main.h - 0.01,
                    "{at}: por baixo do número"
                );
                assert!(s.sub.h < s.main.h, "{at}: menor que o número");
                if s.key == KP8 {
                    assert!(dentro(s.sub, s.plate), "{at}: dentro da plaquinha");
                    assert!(dentro(s.main, s.plate), "{at}: o número na plaquinha");
                }
            }
            vistas.push(!subs.is_empty());
        }
        // MEDIDO: «Home» a 10 px mede 28,8 px + 2 × 4 de barra à vista = 36,8 px de tecla (passo ≥ 41,8); a 1600 o
        // passo é 39,2 e a 1280 é 31,9 — o grupo fica sem nenhuma. A 1920 (passo 55,6–55,9) mostra todas.
        assert_eq!(
            vistas,
            [false, false, false, true],
            "(1600, 1280, estreita, 1920)"
        );
    });
}

/// ⭐ A tinta da segunda é mais fraca que a do número, em todo tema e em cada estado (ligada sobre a face, sem acção,
/// acesa na plaquinha), e continua legível (≥ 3:1, texto pequeno não-essencial: o número é a legenda).
#[test]
fn a_segunda_legenda_e_mais_fraca_que_o_numero_e_le_se() {
    use ph2d_app_input_map::view_keyboard::keys_fit::{LIT_INK, LIT_PLATE};
    let estados = [
        (ColorToken::Text1, REST_FACE),
        (ColorToken::Text3, REST_FACE),
        (LIT_INK, LIT_PLATE),
    ];
    let mut falhas = Vec::new();
    for t in Theme::ALL {
        for (main, fundo) in estados {
            let bg = fundo.resolve(t);
            let n = main.resolve(t).contrast_ratio(&bg);
            let s = sub_ink(main).resolve(t).contrast_ratio(&bg);
            eprintln!("{t:?} {main:?}: número {n:.2} · segunda {s:.2}");
            // Sem acção (`Text3`) a segunda fica no mesmo degrau, menor; nos outros estados, mais fraca.
            let fraca = if main == ColorToken::Text3 {
                s <= n
            } else {
                s < n
            };
            if !fraca || s < n.min(3.0) {
                falhas.push(format!("{t:?} {main:?}: número {n:.2} · segunda {s:.2}"));
            }
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
