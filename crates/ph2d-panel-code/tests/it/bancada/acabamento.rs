//! Rodada do dono 10/11: a faixa da frase do pé mais estreita, com o texto no meio dela (lido na
//! tinta), e as entradas do «In this script» mais juntas que os títulos das secções.

use super::quem;
use crate::{linhas_de_base, meio_das_maiusculas};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::bancada::{Aim, BancadaView, outline_entry_h, phrase_height};
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

/// A faixa é uma linha sem o vão de controlo, e o meio das maiúsculas da frase cai no meio dela.
#[test]
fn a_faixa_da_frase_e_uma_linha_sem_o_vao_e_o_texto_fica_no_meio() {
    assert_eq!(phrase_height(), ROW_H_PX, "a faixa = uma linha, sem o vão");
    let (who, mut ts) = (quem(), TextSystem::without_system_fonts());
    let mut v = BancadaView::default();
    let area = Rect::new(0.0, 800.0, 1000.0, phrase_height());
    let mut scene = VectorScene::new();
    v.paint_phrase(
        &who,
        "hero.luau",
        area,
        &mut scene,
        &mut ts,
        &Theme::default(),
    );
    let (px, _, base) = *linhas_de_base(&scene)
        .first()
        .expect("a frase pintou letras");
    let meio = meio_das_maiusculas(&mut ts, v.said(), px, base);
    let alvo = area.y + area.h * 0.5;
    assert!(
        (meio - alvo).abs() < 0.75,
        "as maiúsculas em {meio}, o meio da faixa em {alvo}"
    );
}

/// As entradas (`speed`, `lives`) avançam o passo apertado; um título de secção custa uma linha
/// inteira; os alvos do clique são os rects pintados.
#[test]
fn as_entradas_do_in_this_script_ficam_mais_juntas_que_os_titulos() {
    let (who, mut ts) = (quem(), TextSystem::without_system_fonts());
    let mut v = BancadaView::default();
    let area = Rect::new(0.0, 0.0, 300.0, 600.0);
    v.paint_outline(
        &who,
        area,
        &mut VectorScene::new(),
        &mut ts,
        &Theme::default(),
    );
    let eh = outline_entry_h();
    assert!(eh < ROW_H_PX, "o passo {eh} não aperta ({ROW_H_PX})");
    let at = |aim| v.target(aim).expect("a entrada tem alvo");
    let (speed, lives) = (at(Aim::Entry(0)), at(Aim::Entry(1)));
    assert!((lives.y - speed.y - eh).abs() < 1e-3, "{speed:?} {lives:?}");
    assert!((speed.h - eh).abs() < 1e-3, "o alvo é a linha pintada");
    // Da última propriedade (`fresh`, linha 5) à primeira função: a entrada mais o título.
    let (fresh, on_signal) = (at(Aim::Entry(5)), at(Aim::Entry(14)));
    assert!((on_signal.y - fresh.y - eh - ROW_H_PX).abs() < 1e-3);
}

/// O primeiro texto pintado dentro de `r`: (tamanho desenhado, meio das maiúsculas).
fn texto_em(scene: &VectorScene, ts: &mut TextSystem, r: Rect, nominal: f32) -> (f32, f32) {
    let (px, _, base) = linhas_de_base(scene)
        .into_iter()
        .find(|&(_, x, b)| x >= r.x && x < r.x + r.w && b > r.y && b < r.y + r.h)
        .unwrap_or_else(|| panic!("nada pintado em {r:?}"));
    (px, meio_das_maiusculas(ts, "Hg", nominal, base))
}

/// Fecho 10/11: em qualquer tamanho de letra da interface, o texto das peças sai desenhado no
/// tamanho escolhido UMA vez (não ao quadrado) e no meio da linha que a mesma conta mediu.
#[test]
fn o_texto_das_pecas_aplica_o_tamanho_da_letra_uma_vez() {
    use ph2d_code_buffer::Buffer;
    use ph2d_panel_code::{
        Band, BandTab, ConsoleRow, ConsoleSource, ConsoleView, ListHit, ListRow, ScriptList,
        ScriptListView, Tone,
    };
    use ph2d_tokens::{TypeToken, UiTextSize, UiTextStyle};
    struct Uma;
    impl ConsoleSource for Uma {
        fn len(&self) -> usize {
            1
        }
        fn row(&self, _: usize) -> ConsoleRow<'_> {
            ConsoleRow {
                text: "Hello",
                tone: Tone::Plain,
                line: None,
                tag: None,
            }
        }
    }
    let antes = ph2d_text::active_text_style();
    let nominal = TypeToken::Sm.px();
    let mut falhas = Vec::new();
    for size in [UiTextSize::Small, UiTextSize::Normal, UiTextSize::Large] {
        ph2d_text::set_active_text_style(UiTextStyle { size, ..antes });
        let quer = ph2d_text::displayed_font_px(nominal);
        let (who, mut ts) = (quem(), TextSystem::without_system_fonts());
        let theme = Theme::default();
        let mut casos: Vec<(&str, VectorScene, Rect)> = Vec::new();
        // A frase do pé.
        let mut v = BancadaView::default();
        let area = Rect::new(0.0, 800.0, 1000.0, phrase_height());
        let mut s = VectorScene::new();
        v.paint_phrase(&who, "hero.luau", area, &mut s, &mut ts, &theme);
        casos.push(("frase", s, area));
        // Uma entrada do «In this script».
        let mut s = VectorScene::new();
        v.paint_outline(
            &who,
            Rect::new(0.0, 0.0, 300.0, 600.0),
            &mut s,
            &mut ts,
            &theme,
        );
        casos.push(("entrada", s, v.target(Aim::Entry(0)).expect("a entrada")));
        // Uma linha do console.
        let mut c = ConsoleView::default();
        let band = Band {
            open: true,
            tab: BandTab::Console,
            console: &Uma,
            problem_count: 0,
            problems: &[],
            current: None,
            tab_width: 4,
        };
        let mut s = VectorScene::new();
        let faixa = Rect::new(0.0, 0.0, 800.0, ROW_H_PX * 6.0);
        c.paint(
            band,
            faixa,
            &mut ph2d_panel_code::CodeView::new(),
            &mut s,
            &mut ts,
            &theme,
        );
        casos.push(("console", s, c.row_rect(0).expect("a linha")));
        // O nome na lista de scripts.
        let (filtro, linhas) = (
            Buffer::new(""),
            [ListRow {
                id: 1,
                name: "hero.luau",
                dirty: false,
                current: false,
                users: 1,
            }],
        );
        let lista = ScriptList {
            open: true,
            filter: &filtro,
            focus: None,
            rows: &linhas,
            renaming: None,
            notice: None,
            hover_count: None,
        };
        let mut l = ScriptListView::default();
        let mut s = VectorScene::new();
        let la = Rect::new(0.0, 0.0, 240.0, ScriptListView::height_for(1));
        l.paint(&lista, la, &mut s, &mut ts, &theme);
        casos.push(("lista", s, l.target_rect(ListHit::Row(1)).expect("a linha")));
        for (onde, s, r) in &casos {
            let (px, meio) = texto_em(s, &mut ts, *r, nominal);
            let alvo = r.y + r.h * 0.5;
            if (px - quer).abs() > 1e-3 || (meio - alvo).abs() > 1.0 {
                falhas.push(format!(
                    "{size:?} {onde}: {px} px (quer {quer}), meio {meio} × {alvo}"
                ));
            }
        }
    }
    ph2d_text::set_active_text_style(antes);
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// Auditoria do fecho 10/11: numa mesma linha, todo texto se centra pela mesma lei (o meio das
/// maiúsculas). O título «Who has what» e a contagem ao lado, e o título do balão com a linha e a dica,
/// leem-se na mesma altura (antes: um pela conta `(h − f) / 2`, o outro pela `folga_do_texto`).
#[test]
fn os_textos_da_mesma_linha_centram_se_pela_mesma_lei() {
    use super::{GRADE, LIMITES, MARCA, meio};
    use ph2d_panel_code::CodeView;
    let (who, mut ts) = (quem(), TextSystem::without_system_fonts());
    let mut v = BancadaView::default();
    let theme = Theme::default();
    let meios = |scene: &VectorScene, n: usize, ts: &mut TextSystem| -> Vec<f32> {
        linhas_de_base(scene)
            .into_iter()
            .take(n)
            .map(|(px, _, base)| meio_das_maiusculas(ts, "Hg", px, base))
            .collect()
    };
    let mut grade = VectorScene::new();
    v.paint_bench(
        &who,
        GRADE,
        None,
        &mut CodeView::new(),
        &mut grade,
        &mut ts,
        &theme,
    );
    let cabeca = meios(&grade, 2, &mut ts);
    v.set_marks(&[(0, MARCA)]);
    let (x, y) = meio(MARCA);
    assert!(v.hover(x, y, 0), "controlo: a marca abre o balão");
    let mut balao = VectorScene::new();
    v.paint_popover(
        &who,
        LIMITES,
        &mut CodeView::new(),
        &mut balao,
        &mut ts,
        &theme,
    );
    let faixa = meios(&balao, 3, &mut ts);
    for (onde, m) in [("cabeça da grelha", cabeca), ("faixa do balão", faixa)] {
        assert!(m.len() >= 2, "{onde}: controlo, pintou {} textos", m.len());
        let (lo, hi) = m
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
        // Medido: 0,64 px de arredondamento entre dois corpos; a lei antiga dava 3,64.
        assert!(hi - lo < 1.0, "{onde}: os meios das maiúsculas {m:?}");
    }
}
