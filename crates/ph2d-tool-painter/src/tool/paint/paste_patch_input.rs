//! O gesto da peça colada: o gizmo, o hit-test das alças e a transformação. Partido de [`super`]
//! (B52.R16).

use super::super::PainterTool;
use super::PastePatch;
use crate::tool::paint::selection_gizmo::SelectionGizmoView;

impl PainterTool {
    /// O gizmo da peça, na MESMA forma que o shell já desenha para as formas de seleção.
    ///
    /// ⚠️ **`op = 4`** — fora da faixa `0..=3` das operações booleanas, e é o que faz o shell desenhar
    /// um glifo próprio no quadrado central em vez de herdar o `n` do New: a peça não é uma operação
    /// de conjunto, e um `n` ali diria que ela é.
    #[must_use]
    pub fn paste_patch_gizmo(&self) -> Option<SelectionGizmoView> {
        let patch = self.paint.paste_patch.as_ref()?;
        let tol = self.paint.shape_grab_tol_px;
        let c = patch.corners();
        let (u, v) = (patch.u, [-patch.u[1], patch.u[0]]);
        let mid = |a: [f32; 2], b: [f32; 2]| [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
        let handles = [
            c[0],
            c[1],
            c[2],
            c[3],
            mid(c[1], c[2]),
            mid(c[0], c[1]),
            mid(c[3], c[0]),
            mid(c[2], c[3]),
        ];
        let _ = (u, v);
        Some(SelectionGizmoView {
            outline: vec![c[0], c[1], c[2], c[3], c[0]],
            box_corners: c,
            scale_handles: handles,
            center: patch.center,
            diamond: None,
            scale_tol: tol,
            rotate_tol: tol,
            accent: 0,
            op: 4,
            edit_curve: None,
        })
    }
}

// ── O gesto ─────────────────────────────────────────────────────────────────────────────────────────
// Alças, no MESMO vocabulário do gizmo de seleção: `0..=3` quinas, `4..=7` meios de aresta (R,T,L,B),
// `8` rotacionar, `9` mover.
const H_ROTATE: u8 = 8;
const H_MOVE: u8 = 9;
/// O anel de rotação alcança este tanto de raios-de-alça além de um quadrado de escala — o MESMO
/// número do gizmo de seleção, para o gesto ler igual nos dois.
const ROTATE_BAND: f32 = 2.6;

impl PainterTool {
    /// Ponteiro sobre a peça flutuante. `true` quando ela consumiu o evento — enquanto ela vive, ela
    /// tem a primeira palavra sobre o canvas.
    ///
    /// ⚠️ **Um Down FORA de toda alça e fora da peça NÃO a aplica nem a cancela** — ele é consumido e
    /// não faz nada. Aplicar num clique perdido transformaria um toque acidental em tinta, e cancelar
    /// jogaria fora o trabalho de posicionar; as duas saídas são teclas, e é o que as torna deliberadas.
    pub(in crate::tool::paint) fn paste_patch_pointer(
        &mut self,
        ev: ph2d_editor_core::tool::CanvasPointer,
    ) -> bool {
        use ph2d_editor_core::tool::PointerPhase;
        let Some(patch) = self.paint.paste_patch.clone() else {
            return false;
        };
        match ev.phase {
            PointerPhase::Down => {
                let tol = self.paint.shape_grab_tol_px;
                if let Some(h) = patch_handle_at(&patch, ev.pos, tol) {
                    self.paint.paste_grab = Some((h, ev.pos, patch));
                }
                true
            }
            PointerPhase::Move => {
                if let Some((h, start, initial)) = self.paint.paste_grab.clone() {
                    let next = transformed(&initial, h, start, ev.pos);
                    self.paint.paste_patch = Some(next);
                    self.redraw_paste_patch();
                }
                true
            }
            PointerPhase::Up => {
                self.paint.paste_grab = None;
                true
            }
            // Hover: a peça não age, mas CONSOME — enquanto ela flutua o canvas é dela, e deixar o
            // hover passar acenderia a mira de outra ferramenta por baixo dela.
            PointerPhase::Hover => true,
        }
    }
}

/// Qual alça está sob `p` — quinas/arestas primeiro, depois o anel de rotação, depois o centro, e por
/// fim o CORPO da peça (que move igual ao centro: arrastar a coisa é o gesto que ninguém procura numa
/// alça).
fn patch_handle_at(patch: &PastePatch, p: [f32; 2], tol: f32) -> Option<u8> {
    let c = patch.corners();
    let mid = |a: [f32; 2], b: [f32; 2]| [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let handles = [
        c[0],
        c[1],
        c[2],
        c[3],
        mid(c[1], c[2]),
        mid(c[0], c[1]),
        mid(c[3], c[0]),
        mid(c[2], c[3]),
    ];
    let near = |a: [f32; 2], r: f32| {
        let d = [p[0] - a[0], p[1] - a[1]];
        d[0] * d[0] + d[1] * d[1] <= r * r
    };
    // ⚠️ **As alças ESPECÍFICAS primeiro, o centro depois** — a mesma precedência da cadeia do Delete
    // (o alvo mais específico vence). Perguntar pelo centro antes tem um modo de falha medido: numa
    // peça menor que o dobro da tolerância o quadrado central COBRE as quinas, e escalar vira
    // impossível — o gate mediu a quina oposta andando os 16 px inteiros do arrasto, isto é, a peça
    // toda se movendo. Mover continua alcançável pelo CORPO, que é o último degrau.
    for (i, hp) in handles.iter().enumerate() {
        if near(*hp, tol) {
            return Some(i as u8);
        }
    }
    if near(patch.center, tol) {
        return Some(H_MOVE);
    }
    for hp in &handles {
        if near(*hp, tol * ROTATE_BAND) {
            return Some(H_ROTATE);
        }
    }
    patch.sample(p).map(|_| H_MOVE)
}

/// A peça transformada por um arrasto da alça `h`, computada SEMPRE da `initial` pristina — nunca do
/// quadro anterior. É o que mantém um arrasto longo livre de deriva.
fn transformed(initial: &PastePatch, h: u8, start: [f32; 2], now: [f32; 2]) -> PastePatch {
    let mut out = initial.clone();
    let d = [now[0] - start[0], now[1] - start[1]];
    if h == H_MOVE {
        out.center = [initial.center[0] + d[0], initial.center[1] + d[1]];
        return out;
    }
    if h == H_ROTATE {
        // Rotação pelo ÂNGULO VARRIDO em torno do centro, transcendental-free: o seno e o cosseno
        // saem do produto escalar e do produto vetorial dos dois raios normalizados.
        let a = [start[0] - initial.center[0], start[1] - initial.center[1]];
        let b = [now[0] - initial.center[0], now[1] - initial.center[1]];
        let (la, lb) = (
            (a[0] * a[0] + a[1] * a[1]).sqrt(),
            (b[0] * b[0] + b[1] * b[1]).sqrt(),
        );
        if la <= f32::EPSILON || lb <= f32::EPSILON {
            return out;
        }
        let (ax, ay) = (a[0] / la, a[1] / la);
        let (bx, by) = (b[0] / lb, b[1] / lb);
        let (cos, sin) = (ax * bx + ay * by, ax * by - ay * bx);
        let u = initial.u;
        out.u = [u[0] * cos - u[1] * sin, u[0] * sin + u[1] * cos];
        return out;
    }
    // Escala: a alça arrastada anda, a OPOSTA fica parada — que é o que faz uma quina puxar a caixa
    // em vez de a inflar em torno do centro.
    let (u, v) = (initial.u, [-initial.u[1], initial.u[0]]);
    let lx = d[0] * u[0] + d[1] * u[1];
    let ly = d[0] * v[0] + d[1] * v[1];
    // Sinais por alça: quinas mexem nos dois eixos, meios de aresta só no seu.
    let (sx, sy): (f32, f32) = match h {
        0 => (-1.0, -1.0),
        1 => (1.0, -1.0),
        2 => (1.0, 1.0),
        3 => (-1.0, 1.0),
        4 => (1.0, 0.0),  // meio da aresta direita
        5 => (0.0, -1.0), // topo
        6 => (-1.0, 0.0), // esquerda
        _ => (0.0, 1.0),  // base
    };
    const MIN_HALF: f32 = 0.5;
    let hx = (initial.hx + sx * lx * 0.5).max(MIN_HALF);
    let hy = (initial.hy + sy * ly * 0.5).max(MIN_HALF);
    // O centro acompanha metade do crescimento, na direção da alça: é isso que prega o lado oposto.
    let gx = (hx - initial.hx) * sx;
    let gy = (hy - initial.hy) * sy;
    out.hx = hx;
    out.hy = hy;
    out.center = [
        initial.center[0] + u[0] * gx + v[0] * gy,
        initial.center[1] + u[1] * gx + v[1] * gy,
    ];
    out
}
