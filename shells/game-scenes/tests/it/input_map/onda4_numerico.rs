//! Onda 4, itens 6 e 7a: a face da tecla é o token de cromo `key-face` (o mesmo tom aprovado, byte a byte) e o
//! numérico ganha as segundas legendas da foto do dono (End ↓ PgDn ← → Home ↑ PgUp Ins Del), só onde cabem.

use super::lista::onze;
use super::{Mesa, em_cada_modo_de_texto};
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::keys_fit::{LEGEND_MARGIN, REST_FACE};
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_app_input_map::view_keyboard::sub_legend::{
    SUB_PAD, SUB_PX, SubLegend, SubLook, cap_height_px, clear_of_corners, nav_of, sub_ink, sub_of,
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
/// está inteira (nunca cortada), na faixa de baixo à direita, por baixo do número, dentro da tecla e fora da curva do
/// canto, menor que ele e sem barra na faixa dela. As de TEXTO são um conjunto e as SETAS outro: nunca um subconjunto,
/// salvo a tecla ligada com barra na faixa (a `KP8`). Controlo da coerência: a 1280 o «End» caberia SOZINHO (≤ 18 +
/// 2 × `SUB_PAD` < 28,1) e mesmo assim falta, porque o «Home» não cabe.
#[test]
fn as_segundas_legendas_do_numerico_cabem_inteiras_ou_somem() {
    // MEDIDO (Default e CrispHeavyPlus), número a 15 px e a segunda Regular a 10 px, alinhada à direita com `SUB_PAD`
    // dos dois lados (pode começar à esquerda do número: decisão do coordenador). «Home» Regular + 2 × 2 cabe nos 34,5
    // da tecla a 1600 e não nos 28,1 a 1280. 1600 = 9 (a ↑ da KP8 ligada perde a faixa); 1280 e estreita = 3 (só as
    // setas); 1920 = 10.
    const CONTAGEM: [usize; 4] = [9, 3, 3, 10];
    em_cada_modo_de_texto(|| {
        for (a, conta) in AREAS.into_iter().zip(CONTAGEM) {
            let mut m = pinta(a);
            let looks: Vec<SubLook> = m.tab().doc.kb.subs.clone();
            let keys = m.tab().doc.census.keys.clone();
            let bars: Vec<Rect> = m.tab().doc.census.bars.iter().map(|b| b.0).collect();
            let cortadas: Vec<String> = (m.tab().doc.census.elided().into_iter())
                .map(str::to_owned)
                .collect();
            let regulares = m.tab().doc.census.regular.clone();
            let at = format!("área {}", a.w);
            eprintln!("{at}: {} segundas legendas", looks.len());
            for l in looks
                .iter()
                .filter(|l| matches!(l.legend, SubLegend::Text(_)))
            {
                eprintln!(
                    "{at}: {:?} tinta {:.2} × {:.2} · tecla {:.2}",
                    l.legend, l.sub.w, l.sub.h, l.key_rect.w
                );
            }
            assert_eq!(looks.len(), conta, "{at}: a tabela medida");
            if a.w > 600.0 && a.w < 700.0 {
                // LITERAL-PX-OK: 1280 — o CONTROLO da coerência: o «End» sozinho caberia na largura útil.
                let end = keys
                    .iter()
                    .find(|k| k.key == Key(0xF741))
                    .expect("Numpad 1");
                let w = m.ts.prefix_width("End", SUB_PX);
                // A largura no peso do número é um tecto da do Regular.
                assert!(w + SUB_PAD * 2.0 <= end.rect.w, "{at}: o End sozinho cabe");
                assert!(
                    looks.iter().all(|l| l.key != end.key),
                    "{at}: e falta, pelo conjunto"
                );
            }
            for k in keys.iter().filter(|k| nav_of(k.key).is_some()) {
                let kr = k.rect;
                let Some(s) = looks.iter().find(|s| s.key == k.key) else {
                    // Ausente: o conjunto dela está fora (nenhuma do mesmo tipo à vista), ou as barras na faixa.
                    let texto = matches!(sub_of(k.key), Some(SubLegend::Text(_)));
                    let mesmo = |l: &&SubLook| matches!(l.legend, SubLegend::Text(_)) == texto;
                    let conjunto_fora = !looks.iter().any(|l| mesmo(&l));
                    let faixa = kr.y + kr.h * 0.5;
                    let barras = bars.iter().any(|b| dentro(*b, kr) && b.y + b.h > faixa);
                    assert!(
                        conjunto_fora || barras,
                        "{at}: {:?} ausente com o conjunto à vista e sem barras — um subconjunto",
                        k.key
                    );
                    continue;
                };
                if let SubLegend::Text(t) = &s.legend {
                    assert!(!cortadas.contains(t), "{at}: «{t}» cortada");
                    assert!(regulares.contains(t), "{at}: «{t}» no peso Regular");
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
                    s.sub.x + s.sub.w <= kr.x + kr.w - SUB_PAD + 0.01,
                    "{at}: à direita, na folga"
                );
                assert!(
                    clear_of_corners(s.sub, kr),
                    "{at}: {:?} fora da curva do canto",
                    s.key
                );
                assert!(
                    s.main.y < s.sub.y,
                    "{at}: o número em cima, a segunda na faixa de baixo"
                );
                assert!(
                    s.sub.h <= s.main.h * 0.75 + 0.01,
                    "{at}: tinta da segunda {:.2} > 75 % da do número {:.2}",
                    s.sub.h,
                    s.main.h
                );
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

/// ⭐ As setas da segunda legenda pesam como a palavra ao lado (foto o4_numerico_1600: «quase não se veem»): a caixa de
/// TINTA de cada seta mede ≥ a altura de maiúscula do texto secundário a `SUB_PX`, nas áreas reais e nos dois modos.
/// Controlo: a maiúscula medida é de uma fonte de verdade (entre metade e o corpo inteiro).
#[test]
fn as_setas_da_segunda_legenda_tem_a_altura_da_maiuscula() {
    em_cada_modo_de_texto(|| {
        let cap = cap_height_px(SUB_PX);
        assert!(
            cap > SUB_PX * 0.5 && cap < SUB_PX,
            "maiúscula {cap:.2} a {SUB_PX}"
        );
        for a in AREAS {
            let m = pinta(a);
            let setas: Vec<&SubLook> = (m.tab().doc.kb.subs.iter())
                .filter(|l| matches!(l.legend, SubLegend::Icon(_)))
                .collect();
            assert!(!setas.is_empty(), "área {}: há setas", a.w);
            for l in setas {
                let tinta = l.sub.w.max(l.sub.h);
                eprintln!(
                    "área {}: {:?} tinta {tinta:.2} · maiúscula {cap:.2}",
                    a.w, l.key
                );
                assert!(
                    tinta >= cap - 0.01,
                    "área {}: {:?} tinta {tinta:.2} < {cap:.2}",
                    a.w,
                    l.key
                );
            }
        }
    });
}
