//! ⭐ **O EDITOR GRÁFICO DA CURVA DE EASING** (ciclo 17, doc 126 §6 — report do dono, 09/10:
//! *«o ease deveria ser com gráficos como na timeline e não com números»*).
//!
//! Três coisas, de cima para baixo: a GRELHA das quinze famílias (cada uma em miniatura, com a
//! direção e a forma de agora — escolhe-se a curva pelo DESENHO), a fila das três DIREÇÕES (só nas
//! famílias que a leem), e o GRÁFICO GRANDE da curva escolhida, onde a forma se ARRASTA:
//! - `Bezier` — as duas alças, como no editor de gráfico da timeline;
//! - `Back` · `Spring` — o pico que passa do fim (para cima ou para baixo);
//! - `Elastic` — o lugar do tremor (para os lados);
//! - `Steps` — a largura do primeiro degrau (para os lados).
//!
//! Ele não sabe de que nó a curva vem: recebe `(família, direção, forma)` e devolve as escritas
//! `(nome do param, valor)` — os nomes são os da casa ([`ph2d_ease::nomes`]). Quem as guarda é o
//! hospedeiro (`GraphIntent::SetParam`).

use crate::EditorKey;
use ph2d_a11y::NodeId;
use ph2d_ease::{Shape, nomes};
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_centered, paint_text_elided, resolve,
    stroke_polyline,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const COLUNAS: usize = 5; // LITERAL-PX-OK: CONTAGEM de miniaturas por fila, nao medida
const TILE_ASPECTO: f32 = 0.58; // LITERAL-PX-OK: PROPORCAO altura/largura da miniatura
const HANDLE_R: f32 = 4.5; // LITERAL-PX-OK: raio da alca
const GRAB_R: f32 = 10.0; // LITERAL-PX-OK: meia caixa de agarrar da alca
const RING_W: f32 = 1.5; // LITERAL-PX-OK: anel da alca
const CURVA_W: f32 = 1.5; // LITERAL-PX-OK: traco da curva
const MINI_W: f32 = 1.2; // LITERAL-PX-OK: traco da miniatura
const GRADE_W: f32 = 1.0; // LITERAL-PX-OK: linhas do 0 e do 1
const AMOSTRAS: usize = 96; // LITERAL-PX-OK: CONTAGEM de amostras do grafico grande
const MINI_AMOSTRAS: usize = 32; // LITERAL-PX-OK: CONTAGEM de amostras da miniatura
/// A faixa vertical dos gráficos: a curva sai de `0..1` (`Back`, `Elastic`, a mola).
pub const Y_FAIXA: (f32, f32) = (-0.4, 1.4);

/// As chaves dos nomes das famílias — as da casa.
const NOMES_FAMILIA: [&str; 15] = ph2d_ease::CHAVES_FAMILIA;
const NOMES_DIRECAO: [&str; 3] = ph2d_ease::CHAVES_DIRECAO;

/// A curva que o editor mostra.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Atual<'a> {
    pub curve: i32,
    pub dir: i32,
    pub shape: Shape,
    /// A curva DESENHADA, amostrada pela shell, quando a família é `Custom`.
    pub custom: Option<&'a [f32]>,
}

impl Atual<'_> {
    /// `y(t)` da curva de agora (a `Custom` interpola as amostras da shell).
    #[must_use]
    pub fn em(&self, t: f32) -> f32 {
        valor(self.curve, self.dir, &self.shape, self.custom, t)
    }
}

fn valor(curve: i32, dir: i32, s: &Shape, custom: Option<&[f32]>, t: f32) -> f32 {
    if curve == ph2d_ease::CUSTOM {
        return custom.map_or(t, |c| amostra(c, t));
    }
    ph2d_ease::ease(curve, dir, t, s)
}

fn amostra(c: &[f32], t: f32) -> f32 {
    if c.len() < 2 {
        return t;
    }
    #[expect(clippy::cast_precision_loss, reason = "poucas amostras")]
    let f = t.clamp(0.0, 1.0) * (c.len() - 1) as f32;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "f >= 0"
    )]
    let i = (f.floor() as usize).min(c.len() - 2);
    #[expect(clippy::cast_precision_loss, reason = "poucas amostras")]
    let u = f - i as f32;
    c[i] + (c[i + 1] - c[i]) * u
}

/// O que o pintor registou — o hospedeiro torna as miniaturas botões e as alças pontos.
#[derive(Default)]
pub struct EaseWidgets {
    pub buttons: Vec<NodeId>,
    /// `(id da alça, editor, índice, tela)`.
    pub handles: Vec<(NodeId, NodeId, u8, Rect)>,
}

/// A fração da largura que a grelha das famílias ocupa (o resto é das direções e do gráfico).
const ESQUERDA: f32 = 0.6; // LITERAL-PX-OK: FRACAO da largura, nao medida

/// ⚠️ **Duas colunas, e é MEDIDO:** o painel do grafo tem `205,5 px` de altura no app (a timeline
/// aberta por baixo, `paint_fit_tests`), e a 1.ª redacção empilhava grelha, direções e gráfico em
/// `~400 px` — o gráfico grande ficava fora do painel. À esquerda a grelha, à direita as direções
/// por cima do gráfico; a altura é a da grelha.
struct Geo {
    familias: [Rect; 15],
    direcoes: [Rect; 3],
    canvas: Rect,
    altura: f32,
}

fn geometria(x: f32, y: f32, w: f32, scale: f32) -> Geo {
    let gap = Spacing::Xs.px() * scale;
    let row_h = ROW_H_PX * scale;
    let esq = w * ESQUERDA;
    #[expect(clippy::cast_precision_loss, reason = "cinco colunas")]
    let c = COLUNAS as f32;
    let tw = ((esq - gap * (c - 1.0)) / c).max(1.0);
    let th = tw * TILE_ASPECTO;
    let topo = y + row_h + gap;
    let filas = ph2d_ease::LAST as usize / COLUNAS + 1;
    #[expect(clippy::cast_precision_loss, reason = "tres filas")]
    let grelha_h = filas as f32 * th + (filas as f32 - 1.0) * gap;
    let familias = std::array::from_fn(|f| {
        let (col, fila) = (f % COLUNAS, f / COLUNAS);
        #[expect(clippy::cast_precision_loss, reason = "poucas colunas")]
        Rect::new(
            x + col as f32 * (tw + gap),
            topo + fila as f32 * (th + gap),
            tw,
            th,
        )
    });
    let dx = x + esq + 2.0 * gap;
    let dw = (w - esq - 2.0 * gap).max(1.0);
    let dir_w = (dw - 2.0 * gap) / 3.0;
    let dir_h = th;
    #[expect(clippy::cast_precision_loss, reason = "tres")]
    let direcoes =
        std::array::from_fn(|d| Rect::new(dx + d as f32 * (dir_w + gap), topo, dir_w, dir_h));
    let canvas = Rect::new(
        dx,
        topo + dir_h + gap,
        dw,
        (grelha_h - dir_h - gap).max(1.0),
    );
    Geo {
        familias,
        direcoes,
        canvas,
        altura: row_h + gap + grelha_h,
    }
}

/// A altura do conteúdo, para a largura `w`.
#[must_use]
pub fn height(w: f32, scale: f32) -> f32 {
    geometria(0.0, 0.0, w, scale).altura
}

/// A família lê a direção?
fn usa_direcao(curve: i32) -> bool {
    ph2d_ease::USES_DIRECTION.contains(&curve)
}

/// Desenha uma curva numa caixa, na faixa [`Y_FAIXA`].
fn desenha_curva(
    scene: &mut VectorScene,
    caixa: Rect,
    n: usize,
    f: impl Fn(f32) -> f32,
    w: f32,
    cor: ph2d_vector::Color,
) {
    let (lo, hi) = Y_FAIXA;
    #[expect(clippy::cast_precision_loss, reason = "poucas amostras")]
    let linha: Vec<(f32, f32)> = (0..=n)
        .map(|k| {
            let t = k as f32 / n as f32;
            let v = ((f(t) - lo) / (hi - lo)).clamp(0.0, 1.0);
            (caixa.x + t * caixa.w, caixa.y + (1.0 - v) * caixa.h)
        })
        .collect();
    stroke_polyline(scene, &linha, w, cor);
}

fn linhas_0_e_1(scene: &mut VectorScene, caixa: Rect, theme: Theme) {
    let (lo, hi) = Y_FAIXA;
    let grade = resolve(ColorToken::GridLine, theme);
    for nivel in [0.0_f32, 1.0] {
        let y = caixa.y + caixa.h * (1.0 - (nivel - lo) / (hi - lo));
        stroke_polyline(
            scene,
            &[(caixa.x, y), (caixa.x + caixa.w, y)],
            GRADE_W,
            grade,
        );
    }
}

/// Uma miniatura clicável.
#[expect(clippy::too_many_arguments, reason = "o pintor de uma miniatura")]
fn miniatura(
    caixa: Rect,
    rotulo: &str,
    id: NodeId,
    escolhida: bool,
    f: impl Fn(f32) -> f32,
    hit: &mut HitIndex,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    fonte: f32,
    out: &mut EaseWidgets,
) {
    let raio = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    let fundo = if escolhida {
        ColorToken::AccentSoft
    } else {
        ColorToken::Bg0
    };
    fill_rounded_rect(scene, caixa, raio, resolve(fundo, theme));
    let rot_h = fonte * 1.3; // LITERAL-PX-OK: FACTOR da altura de uma linha de texto
    let grafico = Rect::new(
        caixa.x + 3.0,
        caixa.y + 3.0,
        caixa.w - 6.0,
        caixa.h - rot_h - 3.0,
    ); // LITERAL-PX-OK: respiro de 3 px dentro da miniatura
    linhas_0_e_1(scene, grafico, theme);
    let cor = if escolhida {
        ColorToken::Accent
    } else {
        ColorToken::Text1
    };
    desenha_curva(
        scene,
        grafico,
        MINI_AMOSTRAS,
        f,
        MINI_W,
        resolve(cor, theme),
    );
    paint_text_centered(
        text,
        scene,
        rotulo,
        Rect::new(caixa.x, caixa.y + caixa.h - rot_h, caixa.w, rot_h),
        fonte,
        resolve(ColorToken::Text2, theme),
    );
    hit.register(id, caixa);
    out.buttons.push(id);
}

/// ⭐ **Pinta o editor** e regista as miniaturas e as alças. Devolve a altura do conteúdo.
#[expect(
    clippy::too_many_arguments,
    reason = "a assinatura dos editores irmãos"
)]
pub fn paint(
    label: &str,
    atual: &Atual<'_>,
    key: EditorKey<'_>,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    scale: f32,
    hit: &mut HitIndex,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    out: &mut EaseWidgets,
) -> f32 {
    let row_h = ROW_H_PX * scale;
    let geo = geometria(x, y, w, scale);
    // O cabeçalho diz a curva escolhida pelo NOME — as miniaturas mostram-na pelo desenho.
    #[expect(clippy::cast_sign_loss, reason = "indice de familia")]
    let nome = NOMES_FAMILIA
        .get(atual.curve.max(0) as usize)
        .map_or("", |k| tr(k));
    paint_text_elided(
        text,
        scene,
        &format!("{label} · {nome}"),
        x,
        y + (row_h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let fonte = TypeToken::Sm.px() * scale;
    // ── As quinze famílias.
    for f in 0..=ph2d_ease::LAST {
        #[expect(clippy::cast_sign_loss, reason = "0..=14")]
        let (caixa, nome) = (geo.familias[f as usize], tr(NOMES_FAMILIA[f as usize]));
        let dir = if usa_direcao(f) { atual.dir } else { 0 };
        let custom = (f == ph2d_ease::CUSTOM).then_some(atual.custom).flatten();
        miniatura(
            caixa,
            nome,
            key.sub(&format!("fam/{f}")),
            f == atual.curve,
            |t| valor(f, dir, &atual.shape, custom, t),
            hit,
            scene,
            text,
            theme,
            fonte,
            out,
        );
    }
    // ── As três direções (só nas famílias que a leem).
    if usa_direcao(atual.curve) {
        for d in 0..3_i32 {
            #[expect(clippy::cast_sign_loss, reason = "0..3")]
            let (caixa, nome) = (geo.direcoes[d as usize], tr(NOMES_DIRECAO[d as usize]));
            miniatura(
                caixa,
                nome,
                key.sub(&format!("dir/{d}")),
                d == atual.dir,
                |t| ph2d_ease::ease(atual.curve, d, t, &atual.shape),
                hit,
                scene,
                text,
                theme,
                fonte,
                out,
            );
        }
    }
    // ── O gráfico da curva escolhida, com as alças da forma.
    let canvas = geo.canvas;
    let raio = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, canvas, raio, resolve(ColorToken::Bg0, theme));
    linhas_0_e_1(scene, canvas, theme);
    desenha_curva(
        scene,
        canvas,
        AMOSTRAS,
        |t| atual.em(t),
        CURVA_W,
        resolve(ColorToken::Accent, theme),
    );
    let ponto = |p: (f32, f32)| {
        let (lo, hi) = Y_FAIXA;
        (
            canvas.x + p.0 * canvas.w,
            canvas.y + canvas.h * (1.0 - ((p.1 - lo) / (hi - lo)).clamp(0.0, 1.0)),
        )
    };
    let alcas = alcas(atual);
    if atual.curve == ph2d_ease::BEZIER && alcas.len() == 2 {
        // As linhas das alças, como no gráfico da timeline: de cada ponta à alça dela.
        let fraca = resolve(ColorToken::TextDisabled, theme);
        stroke_polyline(scene, &[ponto((0.0, 0.0)), ponto(alcas[0])], GRADE_W, fraca);
        stroke_polyline(scene, &[ponto((1.0, 1.0)), ponto(alcas[1])], GRADE_W, fraca);
    }
    for (i, &p) in alcas.iter().enumerate() {
        let (px, py) = ponto(p);
        let id = key.sub(&format!("alca/{i}"));
        let gr = GRAB_R * scale;
        hit.register(id, Rect::new(px - gr, py - gr, gr * 2.0, gr * 2.0));
        #[expect(clippy::cast_possible_truncation, reason = "no maximo duas alcas")]
        out.handles.push((id, key.root(), i as u8, canvas));
        fill_circle(
            scene,
            px,
            py,
            HANDLE_R * scale + RING_W,
            resolve(ColorToken::Accent, theme),
        );
        fill_circle(
            scene,
            px,
            py,
            HANDLE_R * scale,
            resolve(ColorToken::Text1, theme),
        );
    }
    height(w, scale)
}

/// ⭐ **As alças da forma, em coordenadas do gráfico** `(t, y)` — a porta única, lida pelo pintor
/// e pelo arrasto.
#[must_use]
pub fn alcas(a: &Atual<'_>) -> Vec<(f32, f32)> {
    match a.curve {
        ph2d_ease::BEZIER => {
            let b = a.shape.bezier;
            vec![(b[0].clamp(0.0, 1.0), b[1]), (b[2].clamp(0.0, 1.0), b[3])]
        }
        ph2d_ease::BACK | ph2d_ease::SPRING | ph2d_ease::ELASTIC => {
            let (t, y, _) = excesso(a);
            vec![(t, y)]
        }
        ph2d_ease::STEPS => {
            let n = a.shape.steps.round().max(1.0);
            let t = 1.0 / n;
            vec![(t, a.em(t))]
        }
        _ => Vec::new(),
    }
}

/// O ponto da curva que MAIS passa da faixa `0..1` — `(t, y, quanto)`. Sem excesso (uma mola
/// lenta), o meio da curva.
fn excesso(a: &Atual<'_>) -> (f32, f32, f32) {
    let mut melhor = (0.5, a.em(0.5), 0.0_f32);
    for k in 1..AMOSTRAS * 2 {
        #[expect(clippy::cast_precision_loss, reason = "poucas amostras")]
        let t = k as f32 / (AMOSTRAS * 2) as f32;
        let y = a.em(t);
        let e = (-y).max(y - 1.0).max(0.0);
        if e > melhor.2 + 1e-6 {
            melhor = (t, y, e);
        }
    }
    melhor
}

/// Procura `p ∈ [lo, hi]` com `g(p) = alvo`, para `g` monótona (em qualquer sentido).
fn bissecta(lo: f32, hi: f32, alvo: f32, g: impl Fn(f32) -> f32) -> f32 {
    let (glo, ghi) = (g(lo), g(hi));
    let sobe = ghi >= glo;
    let (mut a, mut b) = (lo, hi);
    for _ in 0..40 {
        let m = 0.5 * (a + b);
        if (g(m) < alvo) == sobe {
            a = m;
        } else {
            b = m;
        }
    }
    0.5 * (a + b)
}

/// ⭐ **Uma alça arrastada para `(t, y)`** — as escritas que a põem lá. A porta única, também
/// lida pelos testes de ida e volta.
#[must_use]
pub fn arrasta(a: &Atual<'_>, alca: usize, t: f32, y: f32) -> Vec<(&'static str, f32)> {
    let com = |s: Shape| Atual { shape: s, ..*a };
    let alvo_excesso = (-y).max(y - 1.0).max(0.0);
    match a.curve {
        ph2d_ease::BEZIER => {
            let i = if alca == 0 { 0 } else { 2 };
            vec![
                (nomes::BEZIER[i], t.clamp(0.0, 1.0)),
                (nomes::BEZIER[i + 1], y),
            ]
        }
        ph2d_ease::BACK => {
            let c = bissecta(0.0, 5.0, alvo_excesso, |c| {
                excesso(&com(Shape {
                    overshoot: c,
                    ..a.shape
                }))
                .2
            });
            vec![(nomes::OVERSHOOT, c)]
        }
        ph2d_ease::SPRING => {
            let z = if alvo_excesso <= 1e-4 {
                1.0
            } else {
                bissecta(0.05, 1.0, alvo_excesso, |z| {
                    excesso(&com(Shape {
                        damping: z,
                        ..a.shape
                    }))
                    .2
                })
            };
            vec![(nomes::DAMPING, z)]
        }
        ph2d_ease::ELASTIC => {
            let p = bissecta(0.05, 1.0, t, |p| {
                excesso(&com(Shape {
                    period: p,
                    ..a.shape
                }))
                .0
            });
            vec![(nomes::PERIOD, p)]
        }
        ph2d_ease::STEPS => {
            let n = (1.0 / t.max(1.0 / 64.0)).round().clamp(1.0, 64.0);
            vec![(nomes::STEPS, n)]
        }
        _ => Vec::new(),
    }
}

/// O arrasto que o despacho guardou, se for deste editor: as escritas da alça.
pub fn drain_drag(
    store: &mut WidgetStore,
    key: EditorKey<'_>,
    a: &Atual<'_>,
) -> Option<Vec<(&'static str, f32)>> {
    let (_parent, _channel, index, nx, ny) = store.take_curve_point_drag_if(|p| p == key.root())?;
    let (lo, hi) = Y_FAIXA;
    let w = arrasta(a, index as usize, nx.clamp(0.0, 1.0), lo + ny * (hi - lo));
    (!w.is_empty()).then_some(w)
}

/// Uma miniatura clicada: a família ou a direção que ela escolhe.
#[must_use]
pub fn click(key: EditorKey<'_>, id: NodeId) -> Option<Vec<(&'static str, f32)>> {
    for f in 0..=ph2d_ease::LAST {
        if id == key.sub(&format!("fam/{f}")) {
            #[expect(clippy::cast_precision_loss, reason = "0..=14")]
            return Some(vec![(nomes::CURVE, f as f32)]);
        }
    }
    for d in 0..3_i32 {
        if id == key.sub(&format!("dir/{d}")) {
            #[expect(clippy::cast_precision_loss, reason = "0..3")]
            return Some(vec![(nomes::DIR, d as f32)]);
        }
    }
    None
}

#[cfg(test)]
#[path = "ease_tests.rs"]
mod tests;
