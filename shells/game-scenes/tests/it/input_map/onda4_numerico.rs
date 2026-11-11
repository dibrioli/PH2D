//! Onda 4, itens 6 e 7a: a face da tecla é o token de cromo `key-face` (o mesmo tom aprovado, byte a byte) e o
//! numérico ganha as segundas legendas da foto do dono (End ↓ PgDn ← → Home ↑ PgUp Ins Del), só onde cabem.

use super::lista::onze;
use super::{Mesa, em_cada_modo_de_texto};
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::keys_fit::{LEGEND_MARGIN, LEGEND_PAD, REST_FACE};
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_app_input_map::view_keyboard::sub_legend::{
    SUB_PX, SubLegend, SubLook, nav_of, sub_ink, sub_of,
};
use ph2d_editor_core::zones::Rect;
use ph2d_input::{Binding, Key};
use ph2d_tokens::{ColorToken, Theme, categorical};

/// As áreas reais da aba (1600×1000 e 1280×720), uma estreita (rola, o passo mínimo) e uma larga (1920×1080).
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

/// ⭐ Item 7a, no arranjo da foto do dono: nas áreas reais e nos dois modos de texto, cada segunda legenda à vista
/// está inteira (nunca cortada), na faixa de baixo à direita, por baixo do número, dentro da tecla, menor que ele e sem
/// barra nenhuma na faixa dela; cada uma AUSENTE tem a razão medida (não cabe na largura útil, ou a tecla ligada tem
/// barras na faixa — a `KP8`, o controlo, até 1600). A 1600 e a 1280 aparecem; a 1920 a `KP8` já tem lugar.
#[test]
fn as_segundas_legendas_do_numerico_cabem_inteiras_ou_somem() {
    em_cada_modo_de_texto(|| {
        for a in AREAS {
            let mut m = pinta(a);
            let looks: Vec<SubLook> = m.tab().doc.kb.subs.clone();
            let keys = m.tab().doc.census.keys.clone();
            let bars: Vec<Rect> = m.tab().doc.census.bars.iter().map(|b| b.0).collect();
            let cortadas: Vec<String> = (m.tab().doc.census.elided().into_iter())
                .map(str::to_owned)
                .collect();
            let at = format!("área {}", a.w);
            eprintln!("{at}: {} segundas legendas", looks.len());
            for k in keys.iter().filter(|k| nav_of(k.key).is_some()) {
                let kr = k.rect;
                let Some(s) = looks.iter().find(|s| s.key == k.key) else {
                    // Ausente: tem de ser pela medida.
                    let largura = match sub_of(k.key).expect("tem segunda") {
                        SubLegend::Text(t) => m.ts.prefix_width(&t, SUB_PX),
                        SubLegend::Icon(_) => SUB_PX,
                    };
                    let nao_cabe = largura + LEGEND_PAD * 2.0 > kr.w;
                    let faixa = kr.y + kr.h * 0.5;
                    let barras = bars.iter().any(|b| dentro(*b, kr) && b.y + b.h > faixa);
                    eprintln!(
                        "{at}: {:?} ausente (largura {largura:.1} · tecla {:.1})",
                        k.key, kr.w
                    );
                    assert!(
                        nao_cabe || barras,
                        "{at}: {:?} ausente sem razão medida",
                        k.key
                    );
                    continue;
                };
                if let SubLegend::Text(t) = &s.legend {
                    assert!(!cortadas.contains(t), "{at}: «{t}» cortada");
                    assert_eq!(
                        t,
                        nav_of(s.key).expect("nav").cap_short(),
                        "{at}: da tabela"
                    );
                }
                assert!(dentro(s.sub, kr), "{at}: {:?} dentro da tecla", s.key);
                assert!(
                    s.sub.y >= s.main.y + s.main.h - 0.01,
                    "{at}: por baixo do número"
                );
                assert!(
                    s.sub.x + s.sub.w <= kr.x + kr.w - LEGEND_PAD + 0.01,
                    "{at}: à direita, na folga"
                );
                assert!(
                    s.main.y < s.sub.y,
                    "{at}: o número em cima, a segunda na faixa de baixo"
                );
                assert!(s.sub.h < s.main.h, "{at}: menor que o número");
                let toca =
                    |b: &Rect| b.y < s.sub.y + s.sub.h && b.y + b.h > s.sub.y && dentro(*b, kr);
                assert!(
                    !bars.iter().any(toca),
                    "{at}: nenhuma barra na faixa da segunda"
                );
            }
            if a.w > 600.0 {
                // LITERAL-PX-OK: as duas áreas reais
                assert!(!looks.is_empty(), "{at}: o dono vê as segundas legendas");
            }
            if a.w < 1000.0 {
                // LITERAL-PX-OK: 1600, 1280 e a estreita — o CONTROLO: a KP8 ligada tem barras na faixa e perde a segunda
                assert!(
                    looks.iter().all(|s| s.key != KP8),
                    "{at}: as barras da KP8 vencem"
                );
            }
            // A plaquinha do número ligado (KP8) deixa a barra à vista dos dois lados.
            let k8 = keys.iter().find(|k| k.key == KP8).expect("KP8");
            assert!(
                k8.plate.x - k8.rect.x >= LEGEND_MARGIN - 0.01,
                "{at}: barra à vista à esquerda"
            );
        }
    });
}

/// ⭐ A tinta da segunda (na FACE da tecla) é mais fraca que a do número ligado e igual à do número sem acção (menor),
/// em todo tema, e lê-se (≥ min(3:1, o número)).
#[test]
fn a_segunda_legenda_e_mais_fraca_que_o_numero_e_le_se() {
    let mut falhas = Vec::new();
    for t in Theme::ALL {
        for (bound, main) in [(true, ColorToken::Text1), (false, ColorToken::Text3)] {
            let bg = REST_FACE.resolve(t);
            let n = main.resolve(t).contrast_ratio(&bg);
            let s = sub_ink(bound).resolve(t).contrast_ratio(&bg);
            let fraca = if bound { s < n } else { s <= n };
            if !fraca || s < n.min(3.0) {
                falhas.push(format!("{t:?} {main:?}: número {n:.2} · segunda {s:.2}"));
            }
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// Onda 4, item 3 na vista Keyboard: com a aberta sem ligações, AltGr + F liga o F (a plataforma não ergue a bandeira
/// Alt) e o Alt esquerdo + F recusa (o acorde do editor) — o controlo.
#[test]
fn na_vista_keyboard_altgr_mais_f_liga_e_o_alt_esquerdo_nao() {
    let tecla = |n: &str| Key::named_in_script(n).expect("tecla com nome")[0];
    let f = Key(u32::from('F'));
    for (modificador, alt, liga) in [("right_alt", false, true), ("left_alt", true, false)] {
        let mut m = Mesa::aberta(View::Keyboard);
        m.hero.input_map = onze();
        m.frame();
        let reload = m.hero.input_map.id("reload").expect("reload");
        assert!(
            m.hero
                .input_map
                .get(reload)
                .is_some_and(|a| a.bindings.is_empty())
        );
        m.tab_mut().doc.smoke_open(Some(reload));
        m.frame();
        m.hero.store.set_alt_held(alt);
        assert!(
            !m.physical(tecla(modificador)),
            "{modificador} sozinho não liga"
        );
        m.physical(f);
        let ligou =
            (m.hero.input_map.get(reload)).is_some_and(|a| a.bindings.contains(&Binding::Key(f)));
        assert_eq!(ligou, liga, "{modificador} + F");
    }
}
