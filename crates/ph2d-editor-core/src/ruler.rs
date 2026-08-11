//! As **RÉGUAS** do canvas — as faixas graduadas nas bordas de cima e da esquerda, e o gesto
//! de onde as guias nascem (plano 25 §9, a W6.2).
//!
//! # Por que aqui, ao lado da grade
//!
//! A régua e a grade respondem à **MESMA pergunta** — *onde a coordenada `x` do mundo pousa na
//! tela?* — e a resposta tem de ser uma só: um traço de régua marcado em 100 e uma linha de
//! grade desenhada em 100 que não coincidam na tela é o tipo de discordância que ninguém
//! atribui a um bug de projeção, só a "o app está torto". Por isso este módulo **não tem
//! projeção própria**: ele chama a do [`crate::grid`], que é o dono do [`GridView`].
//!
//! # O que a régua mede, e em que UNIDADE ela o imprime
//!
//! Ela mede o **MUNDO** — o espaço contínuo, por oposição à rede da grade. Isso é o que ela
//! mede; não é a unidade em que ela escreve.
//!
//! O número impresso sai da porta única [`crate::length::LengthDisplay`], na unidade que o
//! artista escolheu no menu Settings. ⚠️ **O doc antigo aqui dizia *"world-units, a mesma
//! régua que o Inspector mostra nos campos X/Y"* — e isso era FALSO**: o Inspector converte
//! desde que a fronteira de display existe (`panel-inspector/src/sync.rs`, e o rótulo dele
//! diz `Position (px)`), o painel de Grid Snap converte, e a régua era a única superfície do
//! app que não convertia — não por decisão, mas porque `paint_rulers` nem sequer RECEBIA as
//! settings. Com os defaults (100 px/m, Pixels) um objeto que o Inspector punha em `150`
//! ficava sob um traço de régua rotulado `1,5`.
//!
//! A frase estava certa na INTENÇÃO (*um ponto, um número*) e o código a contradizia; agora
//! é o código que a cumpre.
//!
//! O **zero** é a origem da grade ([`crate::grid_snap::GridSnapState::active_origin`]) — de novo
//! um número, dois consumidores. Uma origem só da régua faria o "0" da régua e a âncora da rede
//! caírem em lugares diferentes, e as duas seriam "a origem".
//!
//! ⚠️ **A cadência dos traços NÃO é a da grade**, e isso é decisão, não descuido: a grade pode
//! estar desligada, e uma régua tem de continuar legível em qualquer zoom. Ela escolhe o passo
//! `1/2/5 × 10^k` que mantém os rótulos separados na tela — o comportamento de toda régua de
//! DCC. A grade marca a REDE; a régua mede o MUNDO.

use crate::grid::{GridView, world_bounds};
use crate::length::LengthDisplay;
use crate::paint::{fill_rounded_rect, paint_text_centered, resolve, stroke_polyline};
use crate::zones::Rect;
use ph2d_guides::GuideAxis;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

/// Largura da faixa de régua, em pixels de tela. Ela **ocupa** a borda do canvas (o modelo de
/// sobreposição do Figma), então o número é o menor que ainda comporta um rótulo de 4 dígitos
/// no corpo de texto pequeno mais os traços.
pub const RULER_PX: f32 = 20.0;

/// Distância mínima entre dois rótulos, em pixels. Abaixo disto os números se tocam e a régua
/// vira uma faixa cinza — é este número que escolhe a década do passo.
const MIN_LABEL_PX: f32 = 56.0;

/// Altura do traço maior (o do rótulo), como fração da faixa.
const MAJOR_TICK_FRAC: f32 = 0.55;
/// Altura do traço menor (as subdivisões sem rótulo).
const MINOR_TICK_FRAC: f32 = 0.28;
/// Quantas subdivisões entre dois rótulos. Cinco é o que 1/2/5 admite sem virar mancha.
const MINOR_PER_MAJOR: i32 = 5;
/// Corpo do rótulo, em pixels.
const LABEL_PX: f32 = 9.0;

/// Teto de traços por régua. É de RECURSO: uma tela 4K com o passo mínimo legível
/// (`MIN_LABEL_PX`) comporta ~73 rótulos, ou ~366 traços menores; 4096 é ordens acima e
/// existe só para que um zoom degenerado (câmera com `height_world` ~0) não peça um vetor
/// do tamanho do mundo.
const MAX_TICKS: i64 = 4096;

/// Qual das duas réguas. **Um vocabulário, com a ponte explícita** para o eixo da guia que
/// ela cria — a régua de CIMA mede o X e o que nasce dela é uma linha HORIZONTAL, e essa
/// inversão é exatamente o par que se troca sem o compilador reclamar.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RulerAxis {
    /// A faixa de cima: mede o **X** do mundo.
    Top,
    /// A faixa da esquerda: mede o **Y**.
    Left,
}

impl RulerAxis {
    /// O eixo da guia que um arrasto desta régua cria.
    #[must_use]
    pub fn spawns(self) -> GuideAxis {
        match self {
            RulerAxis::Top => GuideAxis::Horizontal,
            RulerAxis::Left => GuideAxis::Vertical,
        }
    }
}

/// A faixa horizontal (a régua de CIMA), dentro do canvas.
#[must_use]
pub fn top_band(canvas: Rect) -> Rect {
    Rect::new(canvas.x, canvas.y, canvas.w, RULER_PX)
}

/// A faixa vertical (a régua da ESQUERDA), dentro do canvas.
#[must_use]
pub fn left_band(canvas: Rect) -> Rect {
    Rect::new(canvas.x, canvas.y, RULER_PX, canvas.h)
}

/// Sobre qual régua o ponteiro está, se alguma.
///
/// ⚠️ O **canto** (onde as duas faixas se cruzam) pertence à de CIMA, e a escolha é arbitrária
/// mas tem de ser feita UMA vez: sem regra, um press exatamente no canto criaria uma guia
/// horizontal ou vertical conforme a ordem em que os dois `if` foram escritos.
#[must_use]
pub fn hit(canvas: Rect, p: (f32, f32)) -> Option<RulerAxis> {
    if !contains(canvas, p) {
        return None;
    }
    if contains(top_band(canvas), p) {
        return Some(RulerAxis::Top);
    }
    contains(left_band(canvas), p).then_some(RulerAxis::Left)
}

fn contains(r: Rect, p: (f32, f32)) -> bool {
    p.0 >= r.x && p.0 < r.x + r.w && p.1 >= r.y && p.1 < r.y + r.h
}

/// O passo de rótulo: o menor `1/2/5 × 10^k` que ocupe ao menos `MIN_LABEL_PX` na tela.
///
/// `px_per_world` é quantos pixels vale uma unidade de mundo. Sem transcendental (nem `log10`
/// nem `powf`): as duas escadas percorrem décadas por multiplicação, o que também mantém os
/// passos em potências exatas de 10 quando o `f64` as representa.
#[must_use]
pub fn label_step(px_per_world: f64) -> f64 {
    if !px_per_world.is_finite() || px_per_world <= 0.0 {
        return 1.0;
    }
    let want = f64::from(MIN_LABEL_PX) / px_per_world;
    if !want.is_finite() || want <= 0.0 {
        return 1.0;
    }
    // A década que contém `want`.
    let mut decade = 1.0f64;
    while decade < want {
        decade *= 10.0;
    }
    while decade / 10.0 >= want {
        decade /= 10.0;
    }
    // Dentro dela, a primeira das subdivisões 1/2/5 que serve.
    for cand in [decade / 10.0, decade / 5.0, decade / 2.0, decade] {
        if cand >= want {
            return cand;
        }
    }
    decade
}

/// Formata um rótulo de régua **na unidade que o artista lê**: inteiro quando o passo é
/// inteiro, senão com as casas que o passo exige. Sem isto um passo de 0,2 imprime `0`, `0`,
/// `1` — três traços, dois rótulos iguais e um salto.
///
/// ⚠️ **Delega à porta única** ([`crate::length::LengthDisplay::text`]): a régua NÃO tem
/// política de formatação própria. Antes desta wave ela tinha, e o preço estava na tela —
/// a mesma linha de grade lia `100` no painel de Grid Snap e `1` aqui.
#[must_use]
pub fn label_text(value: f64, step: f64, display: LengthDisplay) -> String {
    display.text(value, step)
}

/// Desenha as duas réguas sobre o canvas de `view`.
///
/// `origin` é o zero da régua, em mundo — a origem da grade. Nada aqui deriva projeção: as
/// coordenadas saem das mesmas funções que a grade usa.
///
/// ⚠️ **A GEOMETRIA não depende do `display`.** Os traços caem onde sempre caíram: o passo é
/// escolhido em MUNDO a partir do zoom, e só o NÚMERO impresso cruza a fronteira de unidade.
/// É isto que faz de um projeto em metros um caso **byte-idêntico** ao que já shipava
/// (`from_meters` é a identidade ali) — a conversão não pode mover um pixel de traço.
pub fn paint_rulers(
    scene: &mut VectorScene,
    view: &GridView,
    origin: [f32; 2],
    text_system: &mut TextSystem,
    theme: Theme,
    display: LengthDisplay,
) {
    let canvas = view.canvas;
    if canvas.w <= RULER_PX || canvas.h <= RULER_PX {
        return; // canvas menor que a própria régua: nada legível cabe
    }
    let bg = resolve(ColorToken::Bg1, theme);
    let line = resolve(ColorToken::Border, theme);
    let text = resolve(ColorToken::Text2, theme);

    fill_rounded_rect(scene, top_band(canvas), 0.0, bg);
    fill_rounded_rect(scene, left_band(canvas), 0.0, bg);

    paint_axis(
        scene,
        view,
        RulerAxis::Top,
        origin,
        (line, text),
        text_system,
        display,
    );
    paint_axis(
        scene,
        view,
        RulerAxis::Left,
        origin,
        (line, text),
        text_system,
        display,
    );
    // A moldura das duas faixas, por último: ela separa a régua da arte.
    stroke_polyline(
        scene,
        &[
            (canvas.x, canvas.y + RULER_PX),
            (canvas.x + canvas.w, canvas.y + RULER_PX),
        ],
        1.0,
        line,
    );
    stroke_polyline(
        scene,
        &[
            (canvas.x + RULER_PX, canvas.y),
            (canvas.x + RULER_PX, canvas.y + canvas.h),
        ],
        1.0,
        line,
    );
}

/// A coordenada de MUNDO sob um ponto de tela, ao longo de uma régua — a **porta única** do
/// gesto: é ela que diz onde a guia pousa quando o dedo solta.
///
/// ⚠️ É a inversa EXATA da projeção que [`ticks`] usa para desenhar, e as duas moram no
/// [`crate::grid`] lado a lado justamente para não poderem divergir. Um pouso derivado por
/// outra conta cairia a meio pixel do traço que o artista mirou — a doença de *seed que não
/// casa com sample*, aqui com o dedo como testemunha.
#[must_use]
pub fn world_at(view: &GridView, screen: f32, axis: RulerAxis) -> f64 {
    let (bounds, _) = world_bounds(view);
    f64::from(match axis {
        RulerAxis::Top => crate::grid::screen_to_world_x(screen, &bounds, view),
        RulerAxis::Left => crate::grid::screen_to_world_y(screen, &bounds, view),
    })
}

/// Quantas unidades de mundo vale UM pixel, ao longo de uma régua. É o que converte a
/// tolerância de agarrar uma guia (um número de pixels, porque é o dedo que a define) para a
/// régua em que as guias vivem.
#[must_use]
pub fn world_per_px(view: &GridView, axis: RulerAxis) -> f64 {
    let (bounds, ppm_y) = world_bounds(view);
    let ppm = match axis {
        RulerAxis::Top => view.window_w / (bounds.right - bounds.left),
        RulerAxis::Left => ppm_y,
    };
    if ppm.abs() < f32::EPSILON {
        return 0.0;
    }
    f64::from(1.0 / ppm)
}

/// Um traço da régua: **onde** na tela, **que** valor de mundo, e se leva rótulo.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tick {
    /// A coordenada de tela ao longo da régua (x na de cima, y na da esquerda).
    pub screen: f32,
    /// O valor de MUNDO que este traço marca.
    pub world: f64,
    /// Leva rótulo? (um a cada [`MINOR_PER_MAJOR`]).
    pub major: bool,
}

/// Os traços de uma régua — a função **pura** que o desenho consome.
///
/// ⚠️ Ela existe para que a projeção seja feita **UMA vez** e possa ser confrontada com a da
/// grade num teste. Sem ela o `paint` seria o único dono das coordenadas, e a única forma de
/// conferir que um traço marcado em 100 cai onde a linha de grade de 100 cai seria olhar a tela.
#[must_use]
pub fn ticks(view: &GridView, origin: [f32; 2], axis: RulerAxis) -> Vec<Tick> {
    let (bounds, ppm_y) = world_bounds(view);
    let (px_per_world, o, from, to) = match axis {
        RulerAxis::Top => (
            f64::from(view.window_w / (bounds.right - bounds.left)),
            f64::from(origin[0]),
            f64::from(bounds.left),
            f64::from(bounds.right),
        ),
        RulerAxis::Left => (
            f64::from(ppm_y),
            f64::from(origin[1]),
            f64::from(bounds.bottom),
            f64::from(bounds.top),
        ),
    };
    let step = label_step(px_per_world);
    let minor = step / f64::from(MINOR_PER_MAJOR);
    if !minor.is_finite() || minor <= 0.0 || !from.is_finite() || !to.is_finite() {
        return Vec::new();
    }
    // Do primeiro traço menor visível até o último, em passos exatos a partir da ORIGEM.
    let first = ((from - o) / minor).floor() as i64;
    let last = ((to - o) / minor).ceil() as i64;
    // Uma janela absurda (zoom degenerado) devolve nada em vez de percorrer o mundo. O teto é
    // de RECURSO — é quantos traços cabem numa tela 4K com o passo mínimo legível, com folga —
    // e não um palpite: acima dele os traços já se sobrepõem e nada mais é legível.
    if last.saturating_sub(first) > MAX_TICKS {
        return Vec::new();
    }
    (first..=last)
        .map(|k| {
            let world = o + minor * k as f64;
            Tick {
                screen: match axis {
                    RulerAxis::Top => crate::grid::world_to_screen_x(world as f32, &bounds, view),
                    RulerAxis::Left => crate::grid::world_to_screen_y(world as f32, &bounds, view),
                },
                world,
                major: k % i64::from(MINOR_PER_MAJOR) == 0,
            }
        })
        .collect()
}

fn paint_axis(
    scene: &mut VectorScene,
    view: &GridView,
    axis: RulerAxis,
    origin: [f32; 2],
    (line, text): (ph2d_vector::Color, ph2d_vector::Color),
    text_system: &mut TextSystem,
    display: LengthDisplay,
) {
    let canvas = view.canvas;
    let horizontal = axis == RulerAxis::Top;
    let band = if horizontal {
        top_band(canvas)
    } else {
        left_band(canvas)
    };
    let o = if horizontal {
        f64::from(origin[0])
    } else {
        f64::from(origin[1])
    };
    let step = label_step(match axis {
        RulerAxis::Top => {
            let (b, _) = world_bounds(view);
            f64::from(view.window_w / (b.right - b.left))
        }
        RulerAxis::Left => {
            let (_, ppm_y) = world_bounds(view);
            f64::from(ppm_y)
        }
    });
    for t in ticks(view, origin, axis) {
        if !in_band(band, t.screen, horizontal) {
            continue;
        }
        let len = band_len(band, horizontal)
            * if t.major {
                MAJOR_TICK_FRAC
            } else {
                MINOR_TICK_FRAC
            };
        let pts = if horizontal {
            [
                (t.screen, band.y + band.h - len),
                (t.screen, band.y + band.h),
            ]
        } else {
            [
                (band.x + band.w - len, t.screen),
                (band.x + band.w, t.screen),
            ]
        };
        stroke_polyline(scene, &pts, 1.0, line);
        if !t.major {
            continue;
        }
        let label = label_text(t.world - o, step, display);
        // O rótulo cabe na parte da faixa que os traços não usam.
        let cell = if horizontal {
            Rect::new(
                t.screen - MIN_LABEL_PX * 0.5,
                band.y,
                MIN_LABEL_PX,
                band.h - len,
            )
        } else {
            Rect::new(band.x, t.screen - band.w * 0.5, band.w - len, band.w)
        };
        paint_text_centered(text_system, scene, &label, cell, LABEL_PX, text);
    }
}

fn band_len(band: Rect, horizontal: bool) -> f32 {
    if horizontal { band.h } else { band.w }
}

fn in_band(band: Rect, s: f32, horizontal: bool) -> bool {
    if horizontal {
        s >= band.x && s <= band.x + band.w
    } else {
        s >= band.y && s <= band.y + band.h
    }
}

#[cfg(test)]
#[path = "ruler_tests.rs"]
mod tests;
