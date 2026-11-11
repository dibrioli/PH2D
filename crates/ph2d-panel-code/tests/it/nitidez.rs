//! ⭐ **A letra do código lê-se tão nítida como a dos Problems** (dono 10/11): a mesma frase como linha
//! do código e como texto de um problema, pintada NA PLACA sobre o fundo real de cada uma. A causa
//! medida era o fundo (o poço quase preto), não o glifo; a sonda imprime a franja (cobertura
//! `0,25–0,75`) e a energia de borda, e afirma o que é estável: o poço nos cartões sai da placa com a
//! cor do cartão das linhas de Problems. Sem placa, não mede.

use ph2d_code_buffer::Buffer;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_editor_core::zones::Rect;
use ph2d_gpu::GpuContext;
use ph2d_panel_code::{
    Band, BandTab, CodeView, ConsoleRow, ConsoleSource, ConsoleView, ProblemRow, Severity,
};
use ph2d_render::VelloPass;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

const TEXTO: &str = "self.sped is never set";
const W: usize = 800;
const H: usize = 200;

struct Vazia;
impl ConsoleSource for Vazia {
    fn len(&self) -> usize {
        0
    }
    fn row(&self, _: usize) -> ConsoleRow<'_> {
        unreachable!("vazia")
    }
}

fn codigo(ts: &mut TextSystem, theme: Theme) -> VectorScene {
    codigo_em(&mut CodeView::new(), ts, theme)
}

fn codigo_em(code: &mut CodeView, ts: &mut TextSystem, theme: Theme) -> VectorScene {
    let aba = Rect::new(0.0, 0.0, W as f32, H as f32);
    let buf = Buffer::new(&format!("\n\n\n{TEXTO}\n"));
    let d = crate::Deco::default();
    let (mut f, mut m) = (|r| crate::spans_de(&buf, r), |_| Vec::new());
    let mut s = VectorScene::new();
    code.paint(
        crate::frame(&buf, &d, &mut f, &mut m),
        aba,
        &mut s,
        ts,
        &theme,
    );
    s
}

fn problema(ts: &mut TextSystem, theme: Theme) -> VectorScene {
    // A faixa mede a letra da mono pelo código já pintado (como na aba).
    let mut code = CodeView::new();
    let _ = codigo_em(&mut code, ts, theme);
    let problemas = [ProblemRow {
        severity: Severity::Error,
        text: TEXTO,
        line: 1,
        on: "",
    }];
    let band = Band {
        open: true,
        tab: BandTab::Problems,
        console: &Vazia,
        problem_count: 1,
        problems: &problemas,
        current: None,
        tab_width: 4,
    };
    let (mut c, mut s) = (ConsoleView::default(), VectorScene::new());
    let faixa = Rect::new(0.0, 0.0, 800.0, ROW_H_PX * 6.0);
    c.paint(band, faixa, &mut code, &mut s, ts, &theme);
    s
}

fn lin(v: u8) -> f32 {
    let c = f32::from(v) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// A linha mais clara/escura da imagem (o texto da frase) e o fundo à esquerda dela.
struct Medida {
    bg: [u8; 3],
    franja_srgb: f32,
    franja_luz: f32,
    borda: f32,
    /// A soma do contraste de cada píxel da caixa contra o fundo (a «massa» da tinta).
    tinta: f32,
}

/// A caixa da frase: a linha de base com mais glifos (corridas de 2+, sem os números de linha).
fn caixa(s: &VectorScene) -> (usize, usize, usize, usize) {
    let r = &s.inner().encoding().resources;
    let mut linhas: Vec<(i32, f32, f32, f32, usize)> = Vec::new();
    // Uma corrida da tabela vem com `font_size` 0 (o tamanho vai no glifo): vale o maior da cena.
    let fs_cena = r.glyph_runs.iter().map(|g| g.font_size).fold(0.0, f32::max);
    for g in r.glyph_runs.iter().filter(|g| g.glyphs.len() >= 2) {
        let [a, b, c, d] = g.transform.matrix;
        let [tx, ty] = g.transform.translation;
        let fs = if g.font_size > 0.0 {
            g.font_size
        } else {
            fs_cena
        } * d.abs().max(a.abs());
        for q in &r.glyphs[g.glyphs.clone()] {
            let (x, y) = (a * q.x + c * q.y + tx, b * q.x + d * q.y + ty);
            let k = y.round() as i32;
            match linhas.iter_mut().find(|l| l.0 == k) {
                Some(l) => {
                    l.1 = l.1.min(x);
                    l.2 = l.2.max(x + fs);
                    l.4 += 1;
                }
                None => linhas.push((k, x, x + fs, fs, 1)),
            }
        }
    }
    let (y, x0, x1, fs, _) = linhas
        .into_iter()
        .max_by_key(|l| l.4)
        .expect("a frase pintada");
    let y = y as f32;
    let topo = (y - fs).max(0.0);
    (
        x0 as usize,
        x1 as usize,
        topo as usize,
        (y + fs * 0.3) as usize,
    )
}

fn mede(rgba: &[u8], dark: bool, (x0, x1, y0, y1): (usize, usize, usize, usize)) -> Medida {
    let y = |i: usize| {
        0.2126 * lin(rgba[i * 4]) + 0.7152 * lin(rgba[i * 4 + 1]) + 0.0722 * lin(rgba[i * 4 + 2])
    };
    let luma = |i: usize| {
        (0.2126 * f32::from(rgba[i * 4])
            + 0.7152 * f32::from(rgba[i * 4 + 1])
            + 0.0722 * f32::from(rgba[i * 4 + 2]))
            / 255.0
    };
    // O fundo = o pixel logo à direita da frase, no topo da caixa.
    let ib = (y1 - 1) * W + (x1 + 2).min(W - 1);
    let bg = [rgba[ib * 4], rgba[ib * 4 + 1], rgba[ib * 4 + 2]];
    let (yb, lb) = (y(ib), luma(ib));
    let px: Vec<usize> = (y0..y1)
        .flat_map(|yy| (x0..x1).map(move |x| yy * W + x))
        .collect();
    let cw = x1 - x0;
    let n = px.len();
    let ink = |f: &dyn Fn(usize) -> f32, b: f32| {
        let v = px.iter().map(|&i| f(i));
        let extremo = if dark {
            v.fold(0.0, f32::max)
        } else {
            v.fold(1.0, f32::min)
        };
        extremo - b
    };
    let (dy, dl) = (ink(&y, yb), ink(&luma, lb));
    let cov = |f: &dyn Fn(usize) -> f32, b: f32, d: f32| -> Vec<f32> {
        px.iter()
            .map(|&i| ((f(i) - b) / d).clamp(0.0, 1.0))
            .collect() // CLAMP-OK: cobertura
    };
    let (cl, cs) = (cov(&y, yb, dy), cov(&luma, lb, dl));
    let franja = |c: &[f32]| {
        let t: Vec<f32> = c.iter().copied().filter(|v| *v > 0.05).collect();
        t.iter().filter(|v| **v > 0.25 && **v < 0.75).count() as f32 / t.len().max(1) as f32
    };
    let mut e = 0.0;
    for i in 0..n {
        if i % cw + 1 < cw {
            e += (cs[i + 1] - cs[i]).abs();
        }
        if i + cw < n {
            e += (cs[i + cw] - cs[i]).abs();
        }
    }
    Medida {
        bg,
        franja_srgb: franja(&cs),
        franja_luz: franja(&cl),
        borda: e / cs.iter().sum::<f32>().max(1.0),
        tinta: px.iter().map(|&i| (luma(i) - lb).abs()).sum(),
    }
}

#[test]
fn a_letra_do_codigo_tem_o_fundo_dos_problemas() {
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("sem placa — não mede");
        return;
    };
    let mut pass = VelloPass::new(
        &gpu,
        ph2d_gpu::TextureFormat::Rgba8Unorm,
        (W as u32, H as u32),
    )
    .expect("vello");
    let mut ts = TextSystem::without_system_fonts();
    for theme in [Theme::Dark, Theme::Forge, Theme::Light] {
        let dark = ColorToken::Text1.resolve(theme).relative_luminance() > 0.5;
        let mut corre = |nome: &str, cartoes: bool, prob: bool| {
            force_cards_on_this_thread(Some(cartoes));
            let s = if prob {
                problema(&mut ts, theme)
            } else {
                codigo(&mut ts, theme)
            };
            force_cards_on_this_thread(None);
            let rgba = pass
                .render_and_readback(&gpu, s.inner(), (W as u32, H as u32))
                .expect("render");
            let cx = caixa(&s);
            let m = mede(&rgba, dark, cx);
            eprintln!(
                "{theme:?} {nome}: fundo {:?} · franja sRGB {:.3} · franja luz {:.3} · borda {:.3}",
                m.bg, m.franja_srgb, m.franja_luz, m.borda
            );
            m
        };
        corre("código, poço antigo", false, false);
        let novo = corre("código, poço novo", true, false);
        corre("problema", true, true);
        let k = ColorToken::KanbanCard.resolve(theme);
        assert_eq!(
            novo.bg,
            [k.r, k.g, k.b],
            "{theme:?}: o poço não saiu da placa no tom do cartão"
        );
    }
}

/// Alarga cada corrida da cena em `a` px por lado (o que o preset faz, sem recompilar).
fn alarga(s: &mut VectorScene, a: f64) {
    for g in &mut s.inner_mut().encoding_mut().resources.glyph_runs {
        g.font_embolden = ph2d_vector::FontEmbolden::new(ph2d_vector::Diagonal2::new(a, a));
    }
}

/// ⭐ **A tabela do `STEM_DARKENING_PX`** (dono 10/11): a linha do código a 12 e 13 px (Small e
/// Normal; a mono não pinta a 11) com o contorno alargado 0–0,4 px, contra o texto do problema,
/// no poço escuro e no Light. Afirma o tecto: no valor escolhido a tinta cresce ≤ 10 %.
#[test]
fn o_escurecimento_aproxima_a_franja() {
    use ph2d_editor_core::paint::set_text_rendering;
    use ph2d_tokens::{TextRendering, UiTextSize};
    let Ok(gpu) = GpuContext::new(GpuContext::default_instance(), None) else {
        eprintln!("sem placa — não mede");
        return;
    };
    let mut pass = VelloPass::new(
        &gpu,
        ph2d_gpu::TextureFormat::Rgba8Unorm,
        (W as u32, H as u32),
    )
    .expect("vello");
    let mut ts = TextSystem::without_system_fonts();
    set_text_rendering(TextRendering::CrispHeavyPlus);
    force_cards_on_this_thread(Some(true));
    let antes = ph2d_text::active_text_style();
    let escolhido = f64::from(ph2d_tokens::STEM_DARKENING_PX);
    for size in [UiTextSize::Small, UiTextSize::Normal] {
        ph2d_text::set_active_text_style(ph2d_tokens::UiTextStyle { size, ..antes });
        let px = ph2d_panel_code::code_font_px();
        for theme in [Theme::Dark, Theme::Light] {
            let dark = ColorToken::Text1.resolve(theme).relative_luminance() > 0.5;
            let mut mede_em = |mut s: VectorScene, a: f64| {
                alarga(&mut s, a);
                let rgba = pass
                    .render_and_readback(&gpu, s.inner(), (W as u32, H as u32))
                    .expect("render");
                mede(&rgba, dark, caixa(&s))
            };
            let p = mede_em(problema(&mut ts, theme), 0.0);
            eprintln!(
                "{theme:?} {px} px problema: franja sRGB {:.3} · luz {:.3} · borda {:.3}",
                p.franja_srgb, p.franja_luz, p.borda
            );
            let base = mede_em(codigo(&mut ts, theme), 0.0).tinta;
            for a in [0.0, 0.025, 0.05, 0.075, 0.1, 0.2, 0.3, 0.4] {
                let m = mede_em(codigo(&mut ts, theme), a);
                let cresce = m.tinta / base - 1.0;
                eprintln!(
                    "{theme:?} {px} px código +{a:.3}: franja sRGB {:.3} · luz {:.3} · borda {:.3} · tinta {:+.3}",
                    m.franja_srgb, m.franja_luz, m.borda, cresce
                );
                if (a - escolhido).abs() < 1e-6 {
                    assert!(
                        cresce <= 0.10,
                        "{theme:?} {px} px: +{a} lê-se negrito ({cresce:+.3})"
                    );
                }
            }
        }
    }
    ph2d_text::set_active_text_style(antes);
    force_cards_on_this_thread(None);
    set_text_rendering(TextRendering::default());
}

/// A fonte da mono do código (o blob), para separar as corridas do código das da interface.
fn mono_id(ts: &mut TextSystem) -> u64 {
    let mut cache = ph2d_panel_code::LineCache::default();
    let l = cache.get(ts, "a", &[], ph2d_panel_code::code_font_px());
    let run = l
        .lines()
        .next()
        .and_then(|l| l.runs().next())
        .expect("moldado");
    run.font().data.id()
}

/// As corridas do código pintado: (dica, alarga, x de cada glifo).
fn corridas_do_codigo(ts: &mut TextSystem) -> Vec<(bool, bool, Vec<f32>)> {
    let mono = mono_id(ts);
    let s = codigo(ts, Theme::Dark);
    let r = &s.inner().encoding().resources;
    r.glyph_runs
        .iter()
        .filter(|g| g.font.data.id() == mono)
        .map(|g| {
            let xs = r.glyphs[g.glyphs.clone()].iter().map(|x| x.x).collect();
            (g.hint, g.font_embolden.amount.xx > 0.0, xs)
        })
        .collect()
}

/// ⭐ Dono 10/11: a letra do código segue SÓ o View ▸ Text rendering da aba
/// ([`ph2d_panel_code::code_text`]): cada modo dá a dica, o encaixe e o escurecimento dele; o
/// encaixe move um glifo menos de um píxel (os avanços da mono não mudam); o modo do app já não
/// chega ao código.
#[test]
fn a_letra_do_codigo_segue_o_modo_do_script_e_nao_o_do_app() {
    use ph2d_editor_core::paint::set_text_rendering;
    use ph2d_panel_code::code_text::set_mode;
    use ph2d_tokens::{SnapX, TextRendering};
    let mut ts = TextSystem::without_system_fonts();
    let base = corridas_do_codigo(&mut ts);
    assert!(!base.is_empty());
    for mode in TextRendering::ALL {
        set_mode(mode);
        let c = corridas_do_codigo(&mut ts);
        set_mode(TextRendering::Default);
        let p = mode.params();
        assert_eq!(c.len(), base.len(), "{mode:?}");
        assert!(c.iter().all(|r| r.0 == p.hint), "{mode:?}: a dica");
        assert_eq!(
            c.iter().any(|r| r.1),
            p.stem_darkening,
            "{mode:?}: o escurecimento"
        );
        for (r, b) in c.iter().zip(&base) {
            for (x, x0) in r.2.iter().zip(&b.2) {
                let passo = match p.snap_x {
                    SnapX::None => 0.0,
                    SnapX::Half => 2.0,
                    SnapX::Full => 1.0,
                };
                assert!(
                    passo == 0.0 || (x * passo).fract() == 0.0,
                    "{mode:?}: {x} encaixado"
                );
                assert!((x - x0).abs() <= 0.5, "{mode:?}: {x} × {x0}");
            }
        }
    }
    for app in TextRendering::ALL {
        set_text_rendering(app);
        let c = corridas_do_codigo(&mut ts);
        set_text_rendering(TextRendering::default());
        assert_eq!(c, base, "o modo do app {app:?} chegou ao código");
    }
}

/// O modo do script não chega à letra da interface (um rótulo do `paint_text` do app).
#[test]
fn o_modo_do_script_nao_muda_a_letra_da_interface() {
    use ph2d_panel_code::code_text::set_mode;
    use ph2d_tokens::TextRendering;
    let mut ts = TextSystem::without_system_fonts();
    let mut ui = |modo| {
        set_mode(modo);
        let mut s = VectorScene::new();
        let ink = ph2d_editor_core::paint::resolve(ColorToken::Text1, Theme::Dark);
        let px = ph2d_tokens::TypeToken::Sm.px();
        ph2d_editor_core::paint::paint_text(&mut ts, &mut s, "bob.luau", 3.3, 4.0, px, 200.0, ink);
        set_mode(TextRendering::Default);
        let r = &s.inner().encoding().resources;
        let runs: Vec<_> = r
            .glyph_runs
            .iter()
            .map(|g| (g.hint, g.font_embolden.amount.xx.to_bits()))
            .collect();
        (runs, r.glyphs.iter().map(|g| g.x).collect::<Vec<f32>>())
    };
    let base = ui(TextRendering::Default);
    assert!(!base.0.is_empty());
    for modo in TextRendering::ALL {
        assert_eq!(ui(modo), base, "{modo:?}");
    }
}
