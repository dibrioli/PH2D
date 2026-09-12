//! **O gizmo de canvas da §12 Sockets / Named Anchors** — a metade que o
//! [ADR-0072](../../../../docs/architecture/decisions/0072-named-anchor-unification.md) §2.3
//! declarou em 2026-05 e que só existia como desenho.
//!
//! O Enio pediu-o no primeiro smoke da seção: *«se selecionar a âncora ou slot no painel aparece
//! o gizmo de edição no canvas similar ao gizmo da sprite mas com cores variadas»*. Até aqui o
//! [`super::anchor_overlay`] desenhava a cruz e os retângulos, e **nada disso sabia que existia
//! um ponteiro**: a marca era decoração.
//!
//! # Este módulo é PURO, e é essa a razão de ele existir separado
//!
//! Ele não vê o `World`, a janela, nem o renderer: recebe a pose do sprite e uma âncora, e
//! devolve **onde estão as alças** e **que edição um arrasto produz**. É a mesma escolha do
//! [`super::sim_extract_slice::instances`] e pela mesma razão — a costura de um gizmo é
//! inalcançável por teste quando ela vive dentro do laço de eventos, e é exatamente aí que os
//! erros de gizmo moram (a alça que agarra o vizinho, o arrasto que anda ao contrário, o canto
//! que arrasta o canto oposto).
//!
//! # ⚠️ Só a linha ABERTA ganha alças
//!
//! A cruz continua a desenhar-se para todas as âncoras — ela é o «onde» do sprite. As **alças**
//! são da âncora aberta na lista, e a razão é aritmética: dez alças em oito âncoras seriam
//! oitenta alvos a disputar os mesmos pixels, e o gesto deixaria de ser previsível. Quem diz qual
//! é a linha aberta é `ph2d_panel_inspector::open_anchor_row`.
//!
//! # ⚠️ Um arrasto é UM passo de undo, e isso vem de graça
//!
//! Este módulo emite edições a cada movimento; o `post_frame_undo` **suprime o registo enquanto
//! um botão está premido**, então o gesto inteiro fecha num passo só. É a mesma lei que o arrasto
//! da âncora de joint já usa — e por isso não há aqui nenhuma máquina de «começar/terminar
//! transação».

use ph2d_core::Vec2;
use ph2d_ecs::{NamedAnchor, Transform};
use ph2d_editor::AnchorFieldEdit;

use super::anchor_overlay::anchor_world_point;

/// Quantas alças uma âncora pode oferecer: o centro, a rotação, e os quatro cantos de cada um
/// dos dois retângulos.
pub(crate) const MAX_HANDLES: usize = 10;

/// Quantas edições um único arrasto pode produzir — as quatro componentes de um retângulo.
pub(crate) const MAX_EDITS: usize = 4;

/// **O braço da alça de rotação**, em pixels da FONTE, medido ao longo do `+X` local da âncora.
///
/// ⚠️ É uma distância no espaço do SPRITE, não da tela: assim o braço acompanha a âncora quando o
/// sprite roda ou é escalado, e o gesto continua a ler-se como «rodar isto». Um braço em pixels de
/// tela teria de ser recalculado por nível de zoom e mudaria de significado geométrico.
pub(crate) const ROTATE_ARM_PX: f32 = 28.0;

/// O lado mínimo de um retângulo arrastado, em pixels da FONTE.
///
/// ⚠️ **É um limite de REPRESENTAÇÃO, não de conforto:** abaixo de um pixel da fonte o retângulo
/// endereça menos de um texel, e deixa de existir a coisa que ele diz recortar. Colapsar a zero
/// tornaria a alça inagarrável — o artista perderia o retângulo sem ter como o recuperar a não ser
/// por undo.
pub(crate) const MIN_RECT_PX: f32 = 1.0;

/// Que alça é esta.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum AnchorHandleKind {
    /// O centro — arrastar move a âncora.
    Centre,
    /// O braço — arrastar roda a âncora.
    Rotate,
    /// Um canto do retângulo de ÁREA (`bounds`), `0..4` na ordem do desenho.
    Bounds(u8),
    /// Um canto do retângulo de MIOLO (`center`), `0..4`.
    Center(u8),
}

/// Uma alça, no mundo.
#[derive(Copy, Clone, Debug)]
pub(crate) struct AnchorHandle {
    pub kind: AnchorHandleKind,
    pub world: Vec2,
}

/// Os quatro cantos de um rect `[x, y, w, h]`, na MESMA ordem em que o overlay os liga —
/// trocar a ordem aqui faria a alça de um canto arrastar o vizinho.
fn corner(rect: [f32; 4], i: u8) -> [f32; 2] {
    let [x, y, w, h] = rect;
    match i {
        1 => [x + w, y],
        2 => [x + w, y + h],
        3 => [x, y + h],
        _ => [x, y],
    }
}

/// As alças desta âncora, em coordenadas de MUNDO. Devolve o array e quantas valem — zero
/// alocações (HR-3).
pub(crate) fn handles(
    sprite_world: Transform,
    a: &NamedAnchor,
    ppm: f32,
) -> ([Option<AnchorHandle>; MAX_HANDLES], usize) {
    let mut out: [Option<AnchorHandle>; MAX_HANDLES] = [None; MAX_HANDLES];
    let mut n = 0;
    let mut push = |kind, local: [f32; 2]| {
        out[n] = Some(AnchorHandle {
            kind,
            world: anchor_world_point(sprite_world, a, local, ppm),
        });
        n += 1;
    };
    push(AnchorHandleKind::Centre, [0.0, 0.0]);
    push(AnchorHandleKind::Rotate, [ROTATE_ARM_PX, 0.0]);
    if let Some(b) = a.bounds {
        for i in 0..4u8 {
            push(AnchorHandleKind::Bounds(i), corner(b, i));
        }
    }
    if let Some(c) = a.center {
        for i in 0..4u8 {
            push(AnchorHandleKind::Center(i), corner(c, i));
        }
    }
    (out, n)
}

/// A alça sob o ponto, ou `None`.
///
/// ⚠️ **O empate resolve-se pela MAIS ESPECÍFICA, não pela primeira.** O centro e um canto podem
/// cair no mesmo pixel (um retângulo de origem `[0,0]`), e ganhar o centro deixaria aquele canto
/// inagarrável para sempre. As alças saem de [`handles`] com o centro à cabeça, então preferir a
/// de índice MAIOR em caso de empate é preferir o canto.
pub(crate) fn hit(
    hs: &[Option<AnchorHandle>; MAX_HANDLES],
    n: usize,
    world: Vec2,
    tol: f32,
) -> Option<AnchorHandleKind> {
    let mut best: Option<(f32, AnchorHandleKind)> = None;
    for h in hs.iter().take(n).flatten() {
        let d = (h.world - world).length();
        if d > tol {
            continue;
        }
        // `<=` e não `<`: em empate exato fica o ÚLTIMO visto, que é o mais específico.
        if best.is_none_or(|(bd, _)| d <= bd) {
            best = Some((d, h.kind));
        }
    }
    best.map(|(_, k)| k)
}

/// A base local do SPRITE em coordenadas de mundo: a origem e os vetores de um pixel da fonte em
/// `x` e em `y`.
///
/// ⚠️ **A base é do SPRITE, não da âncora** — mover uma âncora rodada tem de a mover na direção em
/// que o rato andou, não na direção para onde ela aponta. Já os CANTOS de um retângulo vivem no
/// espaço da âncora, e por isso usam a base dela.
fn sprite_axes(sprite_world: Transform, ppm: f32) -> (Vec2, Vec2, Vec2) {
    let idle = NamedAnchor::socket("");
    let o = anchor_world_point(sprite_world, &idle, [0.0, 0.0], ppm);
    (
        o,
        anchor_world_point(sprite_world, &idle, [1.0, 0.0], ppm) - o,
        anchor_world_point(sprite_world, &idle, [0.0, 1.0], ppm) - o,
    )
}

/// A base local da ÂNCORA: origem e os vetores de um pixel da fonte nos eixos dela.
fn anchor_axes(sprite_world: Transform, a: &NamedAnchor, ppm: f32) -> (Vec2, Vec2, Vec2) {
    let o = anchor_world_point(sprite_world, a, [0.0, 0.0], ppm);
    (
        o,
        anchor_world_point(sprite_world, a, [1.0, 0.0], ppm) - o,
        anchor_world_point(sprite_world, a, [0.0, 1.0], ppm) - o,
    )
}

/// Resolve `target - o = u·ex + v·ey` — a inversa 2×2 da base.
///
/// ⚠️ **É assim e não com uma `Transform::inverse` porque essa não existe**, e escrevê-la para
/// isto obrigaria a inverter também a distorção (`skew`) num tipo que a linha da física possui.
/// Duas sondas dão a base exata da transformação afim, seja ela qual for — incluindo distorcida.
/// Devolve `None` quando a base é degenerada (escala zero num eixo), que é onde a divisão morreria.
fn solve(o: Vec2, ex: Vec2, ey: Vec2, target: Vec2) -> Option<[f32; 2]> {
    let det = ex.x * ey.y - ex.y * ey.x;
    if !det.is_finite() || det.abs() < f32::EPSILON {
        return None;
    }
    let d = target - o;
    Some([
        (d.x * ey.y - d.y * ey.x) / det,
        (ex.x * d.y - ex.y * d.x) / det,
    ])
}

/// A(s) edição(ões) que este arrasto produz. Array + contagem, zero alocações.
///
/// `row` é o índice da âncora na lista — o mesmo que os campos do painel usam.
pub(crate) fn drag(
    kind: AnchorHandleKind,
    row: u8,
    a: &NamedAnchor,
    sprite_world: Transform,
    world: Vec2,
    ppm: f32,
) -> ([Option<AnchorFieldEdit>; MAX_EDITS], usize) {
    let mut out: [Option<AnchorFieldEdit>; MAX_EDITS] = [None, None, None, None];
    match kind {
        AnchorHandleKind::Centre => {
            let (o, ex, ey) = sprite_axes(sprite_world, ppm);
            let Some([px, py]) = solve(o, ex, ey, world) else {
                return (out, 0);
            };
            out[0] = Some(AnchorFieldEdit::Pos(row, 0, px));
            out[1] = Some(AnchorFieldEdit::Pos(row, 1, py));
            (out, 2)
        }
        AnchorHandleKind::Rotate => {
            // O ângulo mede-se no espaço do SPRITE: a rotação da âncora é relativa a ele.
            let (o, ex, ey) = sprite_axes(sprite_world, ppm);
            let centre = anchor_world_point(sprite_world, a, [0.0, 0.0], ppm);
            let (Some(p), Some(c)) = (solve(o, ex, ey, world), solve(o, ex, ey, centre)) else {
                return (out, 0);
            };
            let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
            if dx.hypot(dy) < f32::EPSILON {
                // Em cima do centro não há direção — manter o ângulo é melhor que saltar para 0.
                return (out, 0);
            }
            out[0] = Some(AnchorFieldEdit::Rot(row, dy.atan2(dx).to_degrees()));
            (out, 1)
        }
        AnchorHandleKind::Bounds(i) | AnchorHandleKind::Center(i) => {
            let is_bounds = matches!(kind, AnchorHandleKind::Bounds(_));
            let Some(rect) = (if is_bounds { a.bounds } else { a.center }) else {
                return (out, 0);
            };
            let (o, ex, ey) = anchor_axes(sprite_world, a, ppm);
            let Some(p) = solve(o, ex, ey, world) else {
                return (out, 0);
            };
            // ⚠️ **O canto OPOSTO fica quieto** — é o que faz o gesto ser «redimensionar» e não
            // «mover». Ele é o `(i + 2) % 4`, e essa aritmética depende da ordem de `corner`.
            let q = corner(rect, (i + 2) % 4);
            let rebuilt = [
                p[0].min(q[0]),
                p[1].min(q[1]),
                (p[0] - q[0]).abs().max(MIN_RECT_PX),
                (p[1] - q[1]).abs().max(MIN_RECT_PX),
            ];
            for (f, v) in rebuilt.into_iter().enumerate() {
                let f = f as u8;
                out[usize::from(f)] = Some(if is_bounds {
                    AnchorFieldEdit::Bounds(row, f, v)
                } else {
                    AnchorFieldEdit::Center(row, f, v)
                });
            }
            (out, MAX_EDITS)
        }
    }
}

/// **O raio de agarre de uma alça, em px de TELA.**
///
/// ⚠️ Em px de tela e não de mundo: o dedo do artista tem o mesmo tamanho em qualquer nível de
/// zoom. Um raio em metros faria a alça ficar impossível de agarrar quando se afasta a câmara —
/// exatamente quando ela é mais precisa de agarrar.
pub(crate) const GRAB_PX: f32 = 9.0;

/// Um arrasto de gizmo de âncora em curso.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnchorDrag {
    /// O sprite dono da âncora — guardado para o arrasto não migrar se a seleção mudar a meio.
    pub entity: u64,
    /// A linha na lista. É o índice que as `AnchorFieldEdit` carregam.
    pub row: u8,
    pub kind: AnchorHandleKind,
}

/// Abre um arrasto se o ponteiro caiu numa alça da âncora ABERTA. Função pura: recebe a pose e a
/// lista, devolve a decisão.
///
/// ⚠️ **Só a âncora aberta é testada** — a mesma lei do desenho, e é isso que garante que *a alça
/// pintada é a alça que agarra*. Consultar todas aqui faria o canvas agarrar uma alça que ninguém
/// desenhou.
pub(crate) fn open_drag(
    sprite_world: Transform,
    list: &ph2d_ecs::NamedAnchorList,
    open_row: Option<usize>,
    entity: u64,
    world: Vec2,
    ppm: f32,
    tol_world: f32,
) -> Option<AnchorDrag> {
    let row = open_row?;
    let a = list.iter().nth(row)?;
    let (hs, n) = handles(sprite_world, a, ppm);
    let kind = hit(&hs, n, world, tol_world)?;
    Some(AnchorDrag {
        entity,
        row: u8::try_from(row).ok()?,
        kind,
    })
}

#[cfg(test)]
#[path = "anchor_gizmo_tests.rs"]
mod tests;
