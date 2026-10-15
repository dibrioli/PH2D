//! 📏 **Sonda: as duas barras do `=` no texto pequeno da interface** (CodeEditor, 15/10).
//!
//! Report do dono: *«o = não tem a aparência do símbolo =. O traço superior parece mais grosso»* —
//! no cartão Script, `line 3 · Expected identifier when parsing expression, got '='`.
//!
//! Pinta pela porta do PRODUTO (`paint::paint_text_block`, a do `rows::aviso` do cartão, a
//! `TypeToken::Sm`), dentro do `ui_scale::pintar_no_chrome` (o caminho da escala), rasteriza no
//! `VelloPass` (`AaConfig::Area`, o do chrome) e lê a cobertura por linha de píxel do `=` isolado.
//! As variantes são de EXECUÇÃO, no mesmo processo: o preset (`set_text_rendering`), o estilo do
//! artista (`set_active_text_style`), a escala (`UiScaleMap::no_ecra`), e — sobre a cena já
//! codificada — o `hint` e a fase da linha de base, escritos nos `glyph_runs` da codificação do
//! Vello (nenhum `const` recompilado). A mono não tem pintor no produto ainda: o emissor daqui é
//! o laço do `paint_text_lines` sem o peso/espaçamento da interface.
//!
//! Corre: `PH2D_GPU=1 bash scripts/cargo-test-narrow.sh ph2d-app-components mede_as_duas_barras_do_igual -- --ignored --nocapture`
//! Saída: tabela no stdout + `docs/CodeEditor/ferramentas/medicoes/igual_*.png` (×8, sem filtro).
//!
//! Resultado (15/10, `igual_perfis.txt`): no estilo do dono (Noto · light · small ⇒ 11,08 ppem,
//! `wght` 700, `hint` OFF do `CrispHeavyPlus`) o `=` é `[2.84 3.42 2.35 4.01]` — sem vão, a barra de
//! cima em duas linhas cinzentas, a de baixo numa escura. `hint(true)` dá o MESMO bit (o `prep`);
//! sem o `prep` (autohinter) dá `[5.18 0.84 5.27 0.92]`.
//!
//! ⚠️ Depois da cura (15/10: o registo tira o `prep`, `ph2d_text::without_prep_table`, e o
//! `CrispHeavyPlus` liga o `hint`) a variante `off(ATUAL)` é o preset ⇒ hint ON, e `auto` = `on`.
//! A tabela gravada é a de ANTES. O gate é `as_duas_barras_do_igual_sao_iguais`.

use parley::{FontFamily, StyleProperty};
use ph2d_editor_core::paint::{paint_text_block, set_text_rendering};
use ph2d_editor_core::ui_scale::{UiScaleMap, ao_pixel, pintar_no_chrome};
use ph2d_editor_core::zones::Rect;
use ph2d_text::{PositionedLayoutItem, TextSystem, set_active_text_style};
use ph2d_tokens::{TextRendering, TypeToken, UiFont, UiScale, UiTextSize, UiTextStyle, UiWeight};
use ph2d_vector::{Affine, Color, Fill, Glyph, VectorScene};
use std::fmt::Write as _;

const TEXTO: &str = "got '=' == =";
const LW: f32 = 160.0;
const LH: f32 = 30.0;
const X0: f32 = 4.3;
const Y0: f32 = 6.4;
const ESCALAS: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

#[derive(Copy, Clone, PartialEq)]
enum Hint {
    Preset,
    On,
    Off,
    /// `On` + a fonte SEM a tabela `prep` (contrafactual do autohinter — `ph2d_text::without_prep_table`).
    Auto,
}

#[derive(Clone)]
struct Variante {
    nome: String,
    mono_px: Option<f32>,
    estilo: UiTextStyle,
    preset: TextRendering,
    s: f32,
    hint: Hint,
    fase: f32,
}

fn estilo_do_dono() -> UiTextStyle {
    // `~/.ph2d/prefs.txt` do dono em 15/10: ui_font=noto_sans · ui_font_weight=light · ui_text_size=small.
    UiTextStyle {
        font: UiFont::NotoSans,
        weight: UiWeight::Light,
        size: UiTextSize::Small,
    }
}

fn dev(s: f32) -> (u32, u32) {
    ((LW * s).ceil() as u32, (LH * s).ceil() as u32)
}

/// Pinta a variante e devolve a cena + as colunas (em píxel do ecrã) de cada `=`.
fn pinta(
    ts: &mut TextSystem,
    v: &Variante,
    sem: &mut std::collections::HashMap<u64, ph2d_vector::FontData>,
) -> (VectorScene, Vec<(f32, f32)>) {
    set_text_rendering(v.preset);
    set_active_text_style(v.estilo);
    let mapa = UiScaleMap::no_ecra(UiScale::P100, v.s);
    let (w, h) = dev(v.s);
    let mut cena = VectorScene::new();
    let mut iguais = Vec::new();
    pintar_no_chrome(
        mapa,
        Rect::new(0.0, 0.0, w as f32, h as f32),
        &mut cena,
        |_, c| match v.mono_px {
            None => {
                paint_text_block(ts, c, TEXTO, X0, Y0, TypeToken::Sm.px(), LW, Color::WHITE);
                let layout = ts.layout_for_rendering(
                    TEXTO,
                    TypeToken::Sm.px(),
                    LW,
                    ph2d_text::FontWeight::MEDIUM,
                    v.preset,
                );
                let eq = id_do_igual(&layout);
                for g in glifos(&layout) {
                    if g.0 == eq {
                        iguais.push((X0 + g.1, X0 + g.1 + g.2));
                    }
                }
            }
            Some(px) => {
                let layout = mono(ts, px);
                emite_mono(c, &layout, v.preset);
                let eq = id_do_igual(&layout);
                for g in glifos(&layout) {
                    if g.0 == eq {
                        iguais.push((X0 + g.1, X0 + g.1 + g.2));
                    }
                }
            }
        },
    );
    // ⚠️ As duas variantes de execução escrevem na CODIFICAÇÃO já feita pelo produto.
    for run in &mut cena.inner_mut().encoding_mut().resources.glyph_runs {
        match v.hint {
            Hint::Preset => {}
            Hint::On => run.hint = true,
            Hint::Off => run.hint = false,
            Hint::Auto => {
                run.hint = true;
                let f = run.font.clone();
                run.font = sem
                    .entry(f.data.id())
                    .or_insert_with(|| {
                        let b = ph2d_text::without_prep_table(f.data.data())
                            .unwrap_or_else(|| f.data.data().to_vec());
                        ph2d_vector::FontData::new(b.into(), f.index)
                    })
                    .clone();
            }
        }
        run.transform.translation[1] += v.fase;
    }
    let iguais = iguais.iter().map(|&(a, b)| (a * v.s, b * v.s)).collect();
    (cena, iguais)
}

fn id_do_igual(layout: &ph2d_text::Layout<()>) -> u32 {
    // O `=` é o 5.º carácter de `TEXTO` e o 5.º glifo (latim simples, 1:1).
    glifos(layout)[5].0
}

/// `(id, x, avanço)` de cada glifo, pela ordem.
fn glifos(layout: &ph2d_text::Layout<()>) -> Vec<(u32, f32, f32)> {
    let mut out = Vec::new();
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(gr) = item {
                out.extend(gr.positioned_glyphs().map(|g| (g.id, g.x, g.advance)));
            }
        }
    }
    out
}

fn mono(ts: &mut TextSystem, px: f32) -> ph2d_text::Layout<()> {
    let (fcx, lcx, stack) = ts.mono_contexts();
    let mut b = lcx.ranged_builder(fcx, TEXTO, 1.0, true);
    b.push_default(StyleProperty::FontFamily(FontFamily::Source(stack.into())));
    b.push_default(StyleProperty::FontSize(px));
    let mut layout = b.build(TEXTO);
    layout.break_all_lines(None);
    layout
}

/// O laço do `paint_text_lines` (origem e linha de base no píxel do ecrã, `hint` do preset).
fn emite_mono(cena: &mut VectorScene, layout: &ph2d_text::Layout<()>, preset: TextRendering) {
    let (x, y) = (ao_pixel(X0), ao_pixel(Y0));
    let t = Affine::translate((f64::from(x), f64::from(y)));
    let hint = preset.params().hint;
    let inner = cena.inner_mut();
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(gr) = item else {
                continue;
            };
            let run = gr.run();
            inner
                .draw_glyphs(run.font())
                .font_size(run.font_size())
                .hint(hint)
                .normalized_coords(run.normalized_coords())
                .brush(Color::WHITE)
                .transform(t)
                .draw(
                    Fill::NonZero,
                    gr.positioned_glyphs().map(|g| Glyph {
                        id: g.id,
                        x: g.x,
                        y: ao_pixel(g.y),
                    }),
                );
        }
    }
}

/// Cobertura por linha de píxel (soma das colunas, em px de cobertura total) do `=` em `cols`.
fn perfil(px: &[u8], w: u32, h: u32, cols: (f32, f32)) -> Vec<f32> {
    let c0 = (cols.0.floor() as i64 - 1).max(0) as u32;
    let c1 = ((cols.1.ceil() as i64) + 1).min(i64::from(w)) as u32;
    (0..h)
        .map(|r| {
            (c0..c1)
                .map(|c| f32::from(px[((r * w + c) * 4 + 3) as usize]) / 255.0)
                .sum()
        })
        .collect()
}

/// As duas barras: corridas contíguas de linhas com tinta (> 2 % do pico).
fn barras(p: &[f32]) -> Vec<(usize, Vec<f32>)> {
    let pico = p.iter().copied().fold(0.0_f32, f32::max);
    let mut out: Vec<(usize, Vec<f32>)> = Vec::new();
    let mut cur: Option<(usize, Vec<f32>)> = None;
    for (i, &v) in p.iter().enumerate() {
        if v > 0.02 * pico {
            cur.get_or_insert_with(|| (i, Vec::new())).1.push(v);
        } else if let Some(c) = cur.take() {
            out.push(c);
        }
    }
    out.extend(cur);
    out
}

fn linha(v: &Variante, p: &[f32]) -> String {
    let b = barras(p);
    let mut s = format!("{:<36}", v.nome);
    let fmt = |(r0, vals): &(usize, Vec<f32>)| {
        let v: Vec<String> = vals.iter().map(|x| format!("{x:.2}")).collect();
        format!("r{r0:>2}[{}]", v.join(" "))
    };
    if b.len() != 2 {
        let todas: Vec<String> = b.iter().map(fmt).collect();
        let _ = write!(s, "  SEM VÃO ({} corrida): {}", b.len(), todas.join(" "));
        return s;
    }
    let pk = |x: &(usize, Vec<f32>)| x.1.iter().copied().fold(0.0_f32, f32::max);
    let tinta = |x: &(usize, Vec<f32>)| x.1.iter().sum::<f32>();
    let (pt, pb) = (pk(&b[0]), pk(&b[1]));
    let _ = write!(
        s,
        "  sup {:<30} inf {:<30} pico {:.2}/{:.2} tinta {:.2}/{:.2} Δpico {:>3.0}%",
        fmt(&b[0]),
        fmt(&b[1]),
        pt,
        pb,
        tinta(&b[0]),
        tinta(&b[1]),
        100.0 * (pt - pb).abs() / pt.max(pb)
    );
    s
}

fn variantes() -> Vec<Variante> {
    let mut vs = Vec::new();
    let fab = UiTextStyle::default();
    let chp = TextRendering::CrispHeavyPlus;
    let def = TextRendering::Default;
    let dono = estilo_do_dono();
    let ui = |vs: &mut Vec<Variante>, nome: String, estilo, preset, s, hint, fase| {
        vs.push(Variante {
            nome,
            mono_px: None,
            estilo,
            preset,
            s,
            hint,
            fase,
        });
    };
    for &s in &ESCALAS {
        // O dono: Noto · light · small, preset de fábrica do app (hint OFF).
        for (hn, h) in [
            ("off(ATUAL)", Hint::Preset),
            ("on", Hint::On),
            ("auto", Hint::Auto),
        ] {
            ui(
                &mut vs,
                format!("s{s:.2} dono CHP hint={hn}"),
                dono,
                chp,
                s,
                h,
                0.0,
            );
        }
        for (hn, h) in [("on(preset)", Hint::Preset), ("auto", Hint::Auto)] {
            ui(
                &mut vs,
                format!("s{s:.2} dono Default hint={hn}"),
                dono,
                def,
                s,
                h,
                0.0,
            );
        }
        // A fábrica: Inter (sem `prep` ⇒ o `on` já é o autohinter).
        for (hn, h) in [("off(preset)", Hint::Preset), ("on", Hint::On)] {
            ui(
                &mut vs,
                format!("s{s:.2} inter CHP hint={hn}"),
                fab,
                chp,
                s,
                h,
                0.0,
            );
        }
        for (hn, h) in [("on(preset)", Hint::Preset), ("off", Hint::Off)] {
            ui(
                &mut vs,
                format!("s{s:.2} inter Default hint={hn}"),
                fab,
                def,
                s,
                h,
                0.0,
            );
        }
        for px in [12.0_f32, 13.0] {
            for (hn, hint) in [("off", Hint::Off), ("on", Hint::On), ("auto", Hint::Auto)] {
                vs.push(Variante {
                    nome: format!("s{s:.2} mono {px}px hint={hn}"),
                    mono_px: Some(px),
                    estilo: fab,
                    preset: chp,
                    s,
                    hint,
                    fase: 0.0,
                });
            }
        }
    }
    // A linha de base FORA do píxel (o contrafactual do `ao_pixel`): fase em píxel do ecrã.
    for fase in [0.25_f32, 0.5, 0.75] {
        for (hn, h) in [("off", Hint::Off), ("auto", Hint::Auto)] {
            ui(
                &mut vs,
                format!("s1.00 dono CHP fase+{fase:.2} hint={hn}"),
                dono,
                chp,
                1.0,
                h,
                fase,
            );
        }
    }
    vs
}

/// Recorte ×8 (vizinho mais próximo), branco sobre preto, das linhas com tinta.
fn grava_png(px: &[u8], w: u32, h: u32, dir: &std::path::Path, nome: &str) {
    let rows: Vec<u32> = (0..h)
        .filter(|&r| (0..w).any(|c| px[((r * w + c) * 4 + 3) as usize] > 0))
        .collect();
    let cols: Vec<u32> = (0..w)
        .filter(|&c| (0..h).any(|r| px[((r * w + c) * 4 + 3) as usize] > 0))
        .collect();
    let (Some(&r0), Some(&r1), Some(&c0), Some(&c1)) =
        (rows.first(), rows.last(), cols.first(), cols.last())
    else {
        return;
    };
    let (r0, c0) = (r0.saturating_sub(1), c0.saturating_sub(1));
    let (r1, c1) = ((r1 + 1).min(h - 1), (c1 + 1).min(w - 1));
    const K: u32 = 8;
    let (cw, ch) = ((c1 - c0 + 1) * K, (r1 - r0 + 1) * K);
    let mut out = vec![0u8; (cw * ch) as usize];
    for y in 0..ch {
        for x in 0..cw {
            let (r, c) = (r0 + y / K, c0 + x / K);
            out[(y * cw + x) as usize] = px[((r * w + c) * 4 + 3) as usize];
        }
    }
    image::GrayImage::from_raw(cw, ch, out)
        .unwrap()
        .save(dir.join(format!("igual_{nome}.png")))
        .unwrap();
}

#[test]
#[ignore = "sonda de placa (PH2D_GPU=1): mede, não vigia"]
fn mede_as_duas_barras_do_igual() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("sem placa — a sonda não mede nada");
        return;
    };
    let mut pass =
        ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, dev(2.0)).unwrap();
    let mut ts = TextSystem::without_system_fonts();
    let mut sem = std::collections::HashMap::new();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/CodeEditor/ferramentas/medicoes");
    std::fs::create_dir_all(&dir).unwrap();
    let pngs = [
        ("s1.00 dono CHP hint=off(ATUAL)", "dono_s100_hintoff_ATUAL"),
        ("s1.00 dono CHP hint=on", "dono_s100_hinton_noop"),
        ("s1.00 dono CHP hint=auto", "dono_s100_autohint"),
        ("s1.00 inter CHP hint=off(preset)", "inter_s100_hintoff"),
        ("s1.00 inter CHP hint=on", "inter_s100_autohint"),
        ("s1.25 dono CHP hint=off(ATUAL)", "dono_s125_hintoff"),
        ("s1.25 dono CHP hint=auto", "dono_s125_autohint"),
        ("s1.00 mono 13px hint=off", "mono13_s100_hintoff"),
        ("s1.00 mono 13px hint=auto", "mono13_s100_autohint"),
    ];
    let vs = variantes();
    let mut primeira: Vec<Vec<f32>> = Vec::new();
    let mut tabela = String::new();
    // Duas voltas, a 2.ª pela ordem inversa: o mesmo processo, intercalado; a 2.ª confere a 1.ª.
    for volta in 0..2 {
        let ordem: Vec<usize> = if volta == 0 {
            (0..vs.len()).collect()
        } else {
            (0..vs.len()).rev().collect()
        };
        let mut perfis = vec![Vec::new(); vs.len()];
        for i in ordem {
            let v = &vs[i];
            let (cena, iguais) = pinta(&mut ts, v, &mut sem);
            let (w, h) = dev(v.s);
            let px = pass
                .render_and_readback(&gpu, cena.inner(), (w, h))
                .unwrap();
            // O último `=` está entre espaços: nenhum vizinho entra nas colunas dele.
            let p = perfil(&px, w, h, *iguais.last().unwrap());
            if volta == 0 {
                let _ = writeln!(tabela, "{}", linha(v, &p));
                if let Some((_, n)) = pngs.iter().find(|(k, _)| *k == v.nome) {
                    grava_png(&px, w, h, &dir, n);
                }
            }
            perfis[i] = p;
        }
        if volta == 0 {
            primeira = perfis;
        } else {
            assert_eq!(
                primeira, perfis,
                "a rasterização não é determinística entre voltas"
            );
        }
    }
    set_text_rendering(TextRendering::Default);
    set_active_text_style(UiTextStyle::default());
    print!("{tabela}");
    std::fs::write(dir.join("igual_perfis.txt"), &tabela).unwrap();
}
