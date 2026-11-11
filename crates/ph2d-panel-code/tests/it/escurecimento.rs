//! ⭐ **O escurecimento da letra do código decide pelo FUNDO** (dono 10/11: «a última opção não faz
//! nada»): a tinta MAIS CLARA que o poço em que assenta recebe-o, seja qual for a luminância absoluta
//! (o roxo da palavra-chave no poço escuro tem `Y < 0,5` e ficava de fora). A tabela das tintas, as
//! afirmações da lei na cena e a sonda de placa da quantidade (`CODE_STEM_DARKENING_PX`).

use ph2d_code_buffer::Buffer;
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::code_text::set_mode;
use ph2d_panel_code::{CodeView, Ink};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, TextRendering, Theme};
use ph2d_vector::{Color, VectorScene};
use std::ops::Range;

const W: usize = 800;
const H: usize = 200;
/// Três linhas de Luau com as tintas do realce real (palavra-chave, função, nome, número, texto,
/// comentário, operador).
const CODIGO: &str = "local function hit(t, n)\n  t.hp = t.hp - n * 1.5 -- dano\n  return \"ok\"\n";

/// O realce das três linhas, à mão (o léxico do `ph2d-code-lsp` não é dependência desta crate).
fn realce(buf: &Buffer, r: Range<usize>) -> Vec<(Range<usize>, Ink)> {
    let s = buf.text().slice(r.clone());
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let start = i;
        let ink = if c == b'-' && b.get(i + 1) == Some(&b'-') {
            i = b.len();
            Ink::Comment
        } else if c == b'"' {
            i += 1 + s[i + 1..].find('"').map_or(0, |k| k + 1);
            Ink::String
        } else if c.is_ascii_digit() {
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
                i += 1;
            }
            Ink::Number
        } else if c.is_ascii_alphabetic() {
            while i < b.len() && b[i].is_ascii_alphanumeric() {
                i += 1;
            }
            match &s[start..i] {
                "local" | "function" | "return" => Ink::Keyword,
                _ if b.get(i) == Some(&b'(') => Ink::Function,
                _ if start > 0 && b[start - 1] == b'.' => Ink::Property,
                _ => Ink::Variable,
            }
        } else if c.is_ascii_whitespace() {
            i += 1;
            continue;
        } else {
            i += 1;
            if c.is_ascii_punctuation() && !b"=-*+".contains(&c) {
                Ink::Punctuation
            } else {
                Ink::Operator
            }
        };
        out.push((r.start + start..r.start + i, ink));
    }
    out
}

/// Uma corrida da mono: (alargamento, glifos (id, x, y)).
type Corrida = (f64, Vec<(u32, f32, f32)>);

fn pinta(ts: &mut TextSystem, theme: Theme, mode: TextRendering) -> VectorScene {
    force_cards_on_this_thread(Some(true));
    set_mode(mode);
    let buf = Buffer::new(CODIGO);
    let d = crate::Deco::default();
    let (mut f, mut m) = (|r| realce(&buf, r), |_| Vec::new());
    let mut s = VectorScene::new();
    CodeView::new().paint(
        crate::frame(&buf, &d, &mut f, &mut m),
        Rect::new(0.0, 0.0, W as f32, H as f32),
        &mut s,
        ts,
        &theme,
    );
    set_mode(TextRendering::Default);
    force_cards_on_this_thread(None);
    s
}

/// A fonte da mono do código (o blob).
fn mono_id(ts: &mut TextSystem) -> u64 {
    let mut cache = ph2d_panel_code::LineCache::default();
    let l = cache.get(ts, "a", &[], ph2d_panel_code::code_font_px());
    let run = l.lines().next().and_then(|l| l.runs().next());
    run.expect("moldado").font().data.id()
}

/// As corridas da mono: (alargamento, glifos (id, x, y)).
fn corridas(ts: &mut TextSystem, s: &VectorScene) -> Vec<Corrida> {
    let mono = mono_id(ts);
    let r = &s.inner().encoding().resources;
    r.glyph_runs
        .iter()
        .filter(|g| g.font.data.id() == mono)
        .map(|g| {
            let gl = r.glyphs[g.glyphs.clone()].iter();
            (
                g.font_embolden.amount.xx,
                gl.map(|q| (q.id, q.x, q.y)).collect(),
            )
        })
        .collect()
}

fn corre(ts: &mut TextSystem, theme: Theme, mode: TextRendering) -> Vec<Corrida> {
    let s = pinta(ts, theme, mode);
    corridas(ts, &s)
}

fn lum(c: Color) -> f32 {
    c.discard_alpha().relative_luminance()
}

fn poco(theme: Theme) -> Color {
    resolve(ColorToken::KanbanCard, theme)
}

/// A tabela: cada tinta do realce sobre o poço (o tom do cartão), a luminância e quem a recebe
/// pela lei velha (`Y > 0,5`) e pela nova (`Y > Y_poço`). Afirma a lei nova nos três temas.
#[test]
fn a_tabela_das_tintas_sobre_o_poco() {
    for theme in [Theme::Dark, Theme::Forge, Theme::Light] {
        let bg = poco(theme);
        eprintln!("{theme:?}: poço Y {:.3}", lum(bg));
        for ink in Ink::ALL {
            let c = resolve(ink.token(), theme);
            let (velha, nova) = (lum(c) > 0.5, lum(c) > lum(bg));
            eprintln!(
                "  {ink:?} ({:?}): Y {:.3} · antes {velha} · agora {nova}",
                ink.token(),
                lum(c)
            );
            set_mode(TextRendering::CrispPlusDarkened);
            let a = ph2d_panel_code::code_text::embolden(12.0, c, bg).amount.xx;
            set_mode(TextRendering::Default);
            assert_eq!(a > 0.0, nova, "{theme:?} {ink:?}: a lei do fundo");
        }
    }
}

/// (a) No poço escuro, sob a opção, TODAS as corridas da mono alargam — a palavra-chave roxa e o
/// nome incluídos (pela lei velha o roxo e o comentário ficavam de fora); (d) os avanços não mudam.
#[test]
fn no_poco_escuro_toda_a_tinta_do_realce_alarga() {
    let mut ts = TextSystem::without_system_fonts();
    for theme in [Theme::Dark, Theme::Forge] {
        let kw = resolve(ColorToken::CodeKeyword, theme);
        assert!(
            lum(kw) < 0.5,
            "{theme:?}: o roxo é o caso que a lei velha perdia"
        );
        let com = corre(&mut ts, theme, TextRendering::CrispPlusDarkened);
        let sem = corre(&mut ts, theme, TextRendering::CrispHeavyPlus);
        assert!(
            com.len() >= 5,
            "{theme:?}: as tintas do realce {}",
            com.len()
        );
        assert!(
            com.iter().all(|r| r.0 > 0.0),
            "{theme:?}: {:?}",
            com.iter().map(|r| r.0).collect::<Vec<_>>()
        );
        let pos = |c: &[Corrida]| c.iter().map(|r| r.1.clone()).collect::<Vec<_>>();
        assert_eq!(
            pos(&com),
            pos(&sem),
            "{theme:?}: o alargamento mexeu nos avanços"
        );
    }
}

/// (b) No Light (tinta escura sobre o poço claro) nenhuma corrida alarga.
#[test]
fn no_light_a_tinta_escura_nao_alarga() {
    let mut ts = TextSystem::without_system_fonts();
    let c = corre(&mut ts, Theme::Light, TextRendering::CrispPlusDarkened);
    assert!(!c.is_empty() && c.iter().all(|r| r.0 == 0.0), "{c:?}");
}

/// (c) O modo de fábrica: a cena é a de sempre — zero alargamento e os mesmos glifos que o Crisp.
#[test]
fn o_modo_de_fabrica_nao_alarga() {
    let mut ts = TextSystem::without_system_fonts();
    for theme in [Theme::Dark, Theme::Forge, Theme::Light] {
        let c = corre(&mut ts, theme, TextRendering::Default);
        assert!(!c.is_empty() && c.iter().all(|r| r.0 == 0.0), "{theme:?}");
    }
}

/// O fundo de cada píxel e a caixa das corridas da mono (sem a calha: x ≥ o 1.º glifo do texto).
fn caixa(ts: &mut TextSystem, s: &VectorScene) -> (usize, usize, usize, usize) {
    let fs = f64::from(ph2d_panel_code::code_font_px()) as f32;
    let c = corridas(ts, s);
    let gl: Vec<_> = c
        .iter()
        .filter(|r| r.1.len() >= 2)
        .flat_map(|r| r.1.clone())
        .collect();
    // A calha: os dígitos estão à esquerda do texto; o texto começa no maior «primeiro x» por linha.
    let x0 = gl.iter().map(|g| g.1).fold(f32::MAX, f32::min);
    let x_texto = gl
        .iter()
        .map(|g| g.1)
        .filter(|x| *x > x0 + fs * 2.0)
        .fold(f32::MAX, f32::min);
    let (y0, y1) = gl
        .iter()
        .fold((f32::MAX, 0.0_f32), |a, g| (a.0.min(g.2), a.1.max(g.2)));
    let x1 = gl.iter().map(|g| g.1).fold(0.0, f32::max) + fs;
    (
        (x_texto - 1.0) as usize,
        (x1 as usize).min(W - 1),
        (y0 - fs) as usize,
        ((y1 + fs * 0.35) as usize).min(H - 1),
    )
}

/// ⭐ A sonda da quantidade: as três linhas no poço escuro a 12/13/15 px (Small/Normal/Large), o
/// alargamento posto à mão em cada corrida da mono. Imprime: píxeis da caixa que mudam > 4/255 contra
/// o `+0`, a massa da tinta (Σ|luma − poço|) relativa ao `+0` e a energia de borda. Sem placa, não mede.
#[test]
fn a_quantidade_do_escurecimento_do_codigo() {
    use ph2d_gpu::GpuContext;
    use ph2d_render::VelloPass;
    use ph2d_tokens::UiTextSize;
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
    let antes = ph2d_text::active_text_style();
    let escolhido = f64::from(ph2d_tokens::CODE_STEM_DARKENING_PX);
    for size in UiTextSize::ALL {
        ph2d_text::set_active_text_style(ph2d_tokens::UiTextStyle { size, ..antes });
        let px = ph2d_panel_code::code_font_px();
        for theme in [Theme::Dark, Theme::Forge] {
            let mono = mono_id(&mut ts);
            let bg = poco(theme);
            let bg8 = bg.to_rgba8();
            let lb =
                (0.2126 * f32::from(bg8.r) + 0.7152 * f32::from(bg8.g) + 0.0722 * f32::from(bg8.b))
                    / 255.0;
            let mut render = |a: f64| {
                let mut s = pinta(&mut ts, theme, TextRendering::CrispHeavyPlus);
                for g in &mut s.inner_mut().encoding_mut().resources.glyph_runs {
                    if g.font.data.id() == mono {
                        g.font_embolden =
                            ph2d_vector::FontEmbolden::new(ph2d_vector::Diagonal2::new(a, a));
                    }
                }
                let cx = caixa(&mut ts, &s);
                (
                    pass.render_and_readback(&gpu, s.inner(), (W as u32, H as u32))
                        .expect("render"),
                    cx,
                )
            };
            let (base, (x0, x1, y0, y1)) = render(0.0);
            let idx: Vec<usize> = (y0..y1)
                .flat_map(|y| (x0..x1).map(move |x| y * W + x))
                .collect();
            let luma = |p: &[u8], i: usize| {
                (0.2126 * f32::from(p[i * 4])
                    + 0.7152 * f32::from(p[i * 4 + 1])
                    + 0.0722 * f32::from(p[i * 4 + 2]))
                    / 255.0
            };
            let massa = |p: &[u8]| idx.iter().map(|&i| (luma(p, i) - lb).abs()).sum::<f32>();
            let borda = |p: &[u8]| {
                let cw = x1 - x0;
                let mut e = 0.0;
                for (k, &i) in idx.iter().enumerate() {
                    if k % cw + 1 < cw {
                        e += (luma(p, i + 1) - luma(p, i)).abs();
                    }
                    if k + cw < idx.len() {
                        e += (luma(p, i + W) - luma(p, i)).abs();
                    }
                }
                e / massa(p).max(1.0)
            };
            let m0 = massa(&base);
            for a in [0.05, 0.075, 0.1, 0.15] {
                let (img, _) = render(a);
                let muda = idx
                    .iter()
                    .filter(|&&i| (0..3).any(|c| img[i * 4 + c].abs_diff(base[i * 4 + c]) > 4))
                    .count() as f32
                    / idx.len() as f32;
                let cresce = massa(&img) / m0 - 1.0;
                eprintln!(
                    "{theme:?} {px} px +{a:.3}: mudam {:.1} % da caixa · tinta {:+.1} % · borda {:.3} (+0: {:.3})",
                    muda * 100.0,
                    cresce * 100.0,
                    borda(&img),
                    borda(&base)
                );
                if (a - escolhido).abs() < 1e-6 && px <= 16.0 {
                    assert!(
                        muda >= 0.02,
                        "{theme:?} {px} px: +{a} não se vê ({muda:.3})"
                    );
                    assert!(
                        cresce <= 0.15,
                        "{theme:?} {px} px: +{a} lê-se negrito ({cresce:+.3})"
                    );
                }
            }
        }
    }
    ph2d_text::set_active_text_style(antes);
}
