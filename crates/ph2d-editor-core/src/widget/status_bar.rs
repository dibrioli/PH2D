//! [`StatusBar`] — pill-shaped horizontal HUD with N segments
//! separated by 1 px inset borders.
//!
//! Used in the editor BottomHUD to surface mode + frame stats:
//! `EDIT • 60 fps • 13101/16660 • 21n • 100% • default-scene`.
//! Each [`StatusSegment`] carries its own copy and an optional
//! emphasis token (Accent, Success, etc); a leading status dot is
//! supported for the `EDIT` segment.

use crate::paint::{fill_rounded_rect, paint_text, rect_to_vello, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SegmentTone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warn,
    Danger,
    Muted,
}

impl SegmentTone {
    fn fg(self) -> ColorToken {
        match self {
            Self::Neutral => ColorToken::Text2,
            Self::Accent => ColorToken::Accent,
            Self::Success => ColorToken::Success,
            Self::Warn => ColorToken::Warn,
            Self::Danger => ColorToken::Danger,
            Self::Muted => ColorToken::Text3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StatusSegment {
    pub text: String,
    pub tone: SegmentTone,
    /// Show a leading 7 px circle in `Success` color (used by the
    /// EDIT segment as a "live" indicator).
    pub leading_dot: bool,
}

impl StatusSegment {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: SegmentTone::Neutral,
            leading_dot: false,
        }
    }

    pub fn tone(mut self, tone: SegmentTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn dot(mut self, yes: bool) -> Self {
        self.leading_dot = yes;
        self
    }
}

#[derive(Clone, Debug)]
pub struct StatusBar {
    pub id: NodeId,
    pub label: String,
    pub segments: Vec<StatusSegment>,
}

impl StatusBar {
    pub fn new(id: NodeId, label: impl Into<String>, segments: Vec<StatusSegment>) -> Self {
        Self {
            id,
            label: label.into(),
            segments,
        }
    }

    /// Width estimate based on character count + padding per segment.
    /// Used by the hero composer to center the bar horizontally.
    /// ⭐ A largura em que todo segmento cabe inteiro — MEDIDA, pela mesma conta do pintor
    /// ([`medidas`]). ⛔ Era `0,6 × fonte` por carácter: errava para baixo, e a barra espremia-se
    /// numa janela que a levava (2026-10-02, a escala da interface).
    pub fn preferred_width(&self, text_system: &mut TextSystem) -> f32 {
        let (fixo, texto) = totais(&medidas(self, text_system));
        fixo + texto
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let label = if self.segments.is_empty() {
            self.label.clone()
        } else {
            self.segments
                .iter()
                .map(|s| s.text.as_str())
                .collect::<Vec<_>>()
                .join(" · ")
        };
        NodeBuilder::new(Role::Status)
            .label(label)
            .bounds(x, y, w, h)
            .build()
    }
}

pub fn paint_status_bar(
    bar: &StatusBar,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // ⭐ A pílula é do CLÁSSICO: num tema moderno a barra é um rectângulo de raio 4 sem moldura.
    let radius = crate::paint::frame_radius(theme, Radius::Full.px());
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    crate::paint::stroke_frame(
        scene,
        rect,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );

    if bar.segments.is_empty() {
        return;
    }
    let pad = Spacing::Lg.px();
    let font = TypeToken::Sm.px();
    let widths = larguras(bar, text_system, rect.w);
    let mut x = rect.x;
    for (i, segment) in bar.segments.iter().enumerate() {
        let (w, texto_w) = widths[i];
        let seg_rect = Rect::new(x, rect.y, w, rect.h);
        if i > 0 {
            // 1 px inner divider in Border color, full segment height
            // minus 6 px breathing room top/bottom.
            let div = Rect::new(x, rect.y + Spacing::Sm.px(), 1.0, rect.h - Spacing::Lg.px());
            scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));
        }
        let fg_token = segment.tone.fg();
        let fg = resolve(fg_token, theme);
        let mut text_x = seg_rect.x + pad;
        if segment.leading_dot {
            let r = RAIO_DO_PONTO;
            let cx = text_x + r;
            let cy = seg_rect.y + seg_rect.h * 0.5;
            let dot = Circle::new(Point::new(cx as f64, cy as f64), r as f64);
            scene.inner_mut().fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(resolve(ColorToken::Success, theme)),
                None,
                &dot,
            );
            text_x += recuo_do_ponto();
        }
        let text_y = seg_rect.y + (seg_rect.h - font) * 0.5;
        let text_w = texto_w;
        paint_text(
            text_system,
            scene,
            &segment.text,
            text_x,
            text_y,
            font,
            text_w,
            fg,
        );
        x += w;
    }
}

/// O raio do ponto de estado.
const RAIO_DO_PONTO: f32 = 3.5; // LITERAL-PX-OK: status-bar dot radius (chrome-specific accent)

/// Quanto o ponto empurra o texto — lido pela medida E pelo pintor. ⛔ A medida reservava
/// `Spacing::Lg` e o pintor empurrava `2r + Sm`: o `EDIT` saía `E…` numa barra com folga.
fn recuo_do_ponto() -> f32 {
    RAIO_DO_PONTO * 2.0 + Spacing::Sm.px()
}

/// `(fixo, texto)` de cada segmento: o recuo (e o ponto) que não encolhe, e o texto MEDIDO no peso em
/// que a elisão do `paint_text` o mede.
fn medidas(bar: &StatusBar, text_system: &mut TextSystem) -> Vec<(f32, f32)> {
    let pad = Spacing::Lg.px();
    let font = TypeToken::Sm.px();
    bar.segments
        .iter()
        .map(|s| {
            let fixo = pad * 2.0 + if s.leading_dot { recuo_do_ponto() } else { 0.0 };
            let texto =
                text_system.prefix_width_weighted(&s.text, font, ph2d_text::FontWeight::MEDIUM);
            (fixo, texto)
        })
        .collect()
}

/// ⭐⭐ **A largura de cada segmento numa barra de `largura`.** Com folga, os segmentos crescem na
/// razão da medida. ⛔ **Apertada, só o TEXTO encolhe** (2026-10-02). Antes encolhia o segmento
/// INTEIRO, recuo incluído, e o pintor descontava depois o recuo cheio. O texto ficava sem
/// largura nenhuma e saía partido em duas linhas (`0` / `ent`) ou por cima do vizinho
/// (`EDIT60 fps`). Gate: `apertada_cada_texto_cabe_no_seu_segmento`.
///
/// Devolve `(segmento, texto)`: a largura do TEXTO viaja calculada, nunca re-derivada pelo pintor
/// como `segmento − recuo`. ⛔ `(t + a) − a` fica um ULP abaixo de `t` numa fracção do domínio, e a
/// elisão compara `<=`: o `EDIT` saía `E…` com folga (foto a 80 %, 2026-10-02).
fn larguras(bar: &StatusBar, text_system: &mut TextSystem, largura: f32) -> Vec<(f32, f32)> {
    let m = medidas(bar, text_system);
    let (fixo, texto) = totais(&m);
    if fixo + texto <= largura || texto <= 0.0 {
        let k = if fixo + texto > 0.0 {
            largura / (fixo + texto)
        } else {
            1.0
        };
        // `t·k + f·(k − 1)`: a `k = 1` o texto é `t` AO BIT.
        return m
            .iter()
            .map(|(f, t)| ((f + t) * k, t * k + f * (k - 1.0)))
            .collect();
    }
    if largura < fixo {
        // Nem o recuo cabe: encolhe ele também, e o texto fica sem nada — a barra nunca vaza.
        let k = largura.max(0.0) / fixo;
        return m.iter().map(|(f, _)| (f * k, 0.0)).collect();
    }
    let teto = teto_comum(m.iter().map(|(_, t)| *t), largura - fixo);
    m.iter()
        .map(|(f, t)| (f + t.min(teto), t.min(teto)))
        .collect()
}

/// `(Σ recuo, Σ texto)` — a MESMA soma para a largura preferida e para a repartição: somar
/// `(recuo + texto)` item a item dá outro arredondamento, e a barra na largura preferida caía, a
/// um ULP, no ramo apertado (o `default-scene` saía `default-sce…`).
fn totais(m: &[(f32, f32)]) -> (f32, f32) {
    (
        m.iter().map(|(f, _)| f).sum(),
        m.iter().map(|(_, t)| t).sum(),
    )
}

/// ⭐ **O teto comum que reparte `disponivel` pelos textos** — os curtos ficam INTEIROS e só os mais
/// longos que o teto encolhem (`EDIT`, `0 ent`, `100%` não viram `…` para sobrar um píxel ao nome
/// da cena). Encher por água: o menor leva o que pede se a parte justa o cobre, e o resto divide-se.
fn teto_comum(textos: impl Iterator<Item = f32>, disponivel: f32) -> f32 {
    let mut t: Vec<f32> = textos.collect();
    t.sort_by(f32::total_cmp);
    let mut resto = disponivel.max(0.0);
    let n = t.len();
    for (i, w) in t.iter().enumerate() {
        let parte = resto / (n - i) as f32;
        if *w > parte {
            return parte;
        }
        resto -= w;
    }
    f32::INFINITY
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> StatusBar {
        StatusBar::new(
            NodeId(1),
            "Editor HUD",
            vec![
                StatusSegment::new("EDIT").dot(true),
                StatusSegment::new("60 fps"),
                StatusSegment::new("13101 / 16660").tone(SegmentTone::Accent),
                StatusSegment::new("21n"),
                StatusSegment::new("100%"),
                StatusSegment::new("default-scene").tone(SegmentTone::Muted),
            ],
        )
    }

    #[test]
    fn defaults_match_spec() {
        let s = fixture();
        assert_eq!(s.segments.len(), 6);
        assert!(s.segments[0].leading_dot);
    }

    #[test]
    fn preferred_width_grows_with_segments() {
        let one = StatusBar::new(NodeId(1), "x", vec![StatusSegment::new("EDIT")]);
        let mut ts = TextSystem::without_system_fonts();
        assert!(fixture().preferred_width(&mut ts) > one.preferred_width(&mut ts));
    }

    #[test]
    fn a11y_role_is_status() {
        let node = fixture().build_a11y(0.0, 0.0, 400.0, 32.0);
        assert_eq!(node.role(), Role::Status);
    }

    fn smoke(bar: StatusBar, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_status_bar(
            &bar,
            Rect::new(0.0, 0.0, bar.preferred_width(&mut text), 34.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    /// ⭐⭐ **Apertada, cada texto cabe no SEU segmento** (2026-10-02, a barra de estatísticas a 200 %
    /// numa janela de portátil): a qualquer largura, do confortável ao absurdo, o que o pintor
    /// pousa mede no máximo a largura que deu ao texto — e com folga nada é cortado. Régua: o que
    /// foi PINTADO. *Mutação: encolher o segmento inteiro (o recuo incluído) ⇒ a largura do texto
    /// cai a zero e esta régua reprova.*
    #[test]
    fn apertada_cada_texto_cabe_no_seu_segmento() {
        let mut ts = TextSystem::without_system_fonts();
        let bar = fixture();
        let cheia = bar.preferred_width(&mut ts);
        let mut w = cheia;
        while w > 120.0 {
            let mut scene = VectorScene::new();
            let (_, medidos) = crate::text_elide::elisao::medindo(|| {
                paint_status_bar(
                    &bar,
                    Rect::new(0.0, 0.0, w, 34.0),
                    &mut scene,
                    &mut ts,
                    Theme::Forge,
                );
            });
            assert_eq!(
                medidos.len(),
                bar.segments.len(),
                "a {w}: um texto por segmento"
            );
            let fixo: f32 = medidas(&bar, &mut ts).iter().map(|(f, _)| f).sum();
            let orcamento: f32 = medidos.iter().map(|m| m.largura).sum();
            assert!(
                orcamento + fixo.min(w) <= w + 0.5,
                "a {w:.0}: os textos receberam {orcamento:.1} + {fixo:.1} de recuo, mais que a barra"
            );
            if w >= cheia {
                assert!(
                    medidos.iter().all(|m| m.coube()),
                    "com folga nada se corta: {medidos:?}"
                );
            } else {
                // Apertada: nenhum texto fica sem largura enquanto o recuo deixar sobra (o defeito
                // da foto), e o MAIS CURTO fica inteiro enquanto a parte justa o cobrir.
                assert!(
                    w - fixo < 1.0 || medidos.iter().all(|m| m.largura > 0.0),
                    "a {w:.0}: um texto ficou sem largura nenhuma: {medidos:?}"
                );
                let textos: Vec<f32> = medidas(&bar, &mut ts).iter().map(|(_, t)| *t).collect();
                let (i, menor) = textos
                    .iter()
                    .enumerate()
                    .min_by(|a, b| a.1.total_cmp(b.1))
                    .expect("há segmentos");
                if w - fixo >= menor * textos.len() as f32 {
                    assert!(
                        medidos[i].coube(),
                        "a {w:.0}: o mais curto foi cortado: {medidos:?}"
                    );
                }
            }
            w -= 37.0;
        }
    }

    /// ⭐ **Na largura preferida nada se corta, em estilo nenhum** — a régua do ULP: a 80 % o `EDIT`
    /// saía `E…` porque o pintor re-derivava o texto como `segmento − recuo`. A largura exacta é o
    /// caso de fronteira, e só a varredura de estilos o encontra.
    #[test]
    fn na_largura_preferida_nada_se_corta_em_estilo_nenhum() {
        let acusados = crate::text_elide::em_todo_estilo(|ts| {
            let bar = fixture();
            let w = bar.preferred_width(ts);
            let mut scene = VectorScene::new();
            let (_, medidos) = crate::text_elide::elisao::medindo(|| {
                paint_status_bar(
                    &bar,
                    Rect::new(0.0, 0.0, w, 34.0),
                    &mut scene,
                    ts,
                    Theme::Forge,
                );
            });
            medidos
                .iter()
                .filter(|m| !m.coube())
                .map(|m| format!("«{}» -> «{}»", m.texto, m.pintado))
                .collect()
        });
        assert!(
            acusados.is_empty(),
            "cortes na largura preferida:\n  {}",
            acusados.join("\n  ")
        );
    }

    #[test]
    fn paint_smoke_full_hud() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_single_segment() {
        smoke(
            StatusBar::new(NodeId(1), "x", vec![StatusSegment::new("EDIT").dot(true)]),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_empty_segments_just_pill() {
        smoke(StatusBar::new(NodeId(1), "x", vec![]), Theme::Blueprint);
    }
}
