//! **O que a shell VÊ da curva** — o snapshot read-only do editor (`CurveOverlay`) e a porta que o
//! produz. Módulo FILHO de [`super`] (`curve`), cortado por ASSUNTO: o pai é o que o editor FAZ (rota
//! de ponteiro, fases, verbos, undo), este é o que ele MOSTRA.
//!
//! O corte veio do teto de LOC quando o overlay ganhou a fase de DESENHO (Enio, 2026-08-07: *"o gizmo
//! está invisível ao ser criado"*) — e é o mesmo corte que `curve_tangent` / `curve_gizmo` já fizeram
//! para as suas metades do chrome.

use super::super::curve_geom;
use super::*;

/// A read-only snapshot of the Curve editor for the shell's overlay (control dots + auto-smoothed spine).
pub struct CurveOverlay {
    /// Control points (image-space px) — drawn as draggable dots.
    pub points: Vec<[f32; 2]>,
    /// The selected point index (drawn highlighted), if any.
    pub selected: Option<usize>,
    /// The flattened spine (px) — the curve guide; matches the painted dabs exactly (same `flatten_spine`).
    pub spine: Vec<[f32; 2]>,
    /// The selected anchor's draggable Bézier **tangent handles**, or `None` when none selected / collapsed.
    pub tangents: Option<TangentHandles>,
    /// The selected anchor's **handle kind** wire `u8` (`0=Free 1=Aligned 2=Vector 3=Auto`), or `None` —
    /// lets the shell mark the active menu item.
    pub selected_kind: Option<u8>,
    /// The whole-curve **transform gizmo** (bbox move / scale / rotate), or `None` when too small to frame.
    pub transform_gizmo: Option<TransformGizmo>,
}

impl PainterTool {
    /// Snapshot the Curve editor for the shell overlay.
    ///
    /// ⚠️ Na fase de DESENHO (o arrasto Down→Up da reta inicial) ele publica **só o spine** — a linha que
    /// o artista está puxando —, com âncoras, tangentes e gizmo VAZIOS. As duas metades da regra:
    /// enquanto a tinta aparecia sob a mão o `None` bastava (o desenho mostrava a figura), e com o gesto
    /// rascunhado (`super::shape_draft`) ele deixaria a tela em branco; e as ALÇAS ficam de fora porque
    /// o `curve_down` só as pega com `editing` — alça desenhada que não responde é chrome morto.
    #[must_use]
    pub fn curve_overlay(&self) -> Option<CurveOverlay> {
        let ed = self.paint.curve.as_ref()?;
        if !ed.editing {
            // Free Hand ACUMULA os pontos capturados; o Curve/Arc guarda um só e pinta `[anchor, cursor]`
            // — daí o `draft_to`. Uma porta, dois desenhos, o MESMO que a tinta faria.
            let mut spine = Vec::new();
            match ed.draft_to {
                Some(p) if !ed.freehand => spine = vec![ed.anchor, p],
                _ => curve_geom::flatten_spine(
                    &ed.model.points,
                    &ed.model.handles,
                    ed.model.closed,
                    &mut spine,
                ),
            }
            return Some(CurveOverlay {
                points: Vec::new(),
                selected: None,
                spine,
                tangents: None,
                selected_kind: None,
                transform_gizmo: None,
            });
        }
        // DRAWING-ONLY offset (Enio 2026-07-05, the Selection model): the whole EDITOR — control anchors,
        // handles, gizmo AND the guide **line** (spine) — stays on the PRISTINE curve; ONLY the painted
        // drawing (the black dabs, filled in `curve_fill`) is offset. So the artist edits the real curve and
        // sees the offset result, with nothing in the editor moving or bunching (Enio 2026-07-05: "ponto e
        // linha ficassem parados e apenas o desenho sofresse o offset").
        let points = ed.model.points.clone();
        let handles = ed.model.handles.clone();
        let osel = ed.model.selected;
        let mut spine = Vec::new();
        curve_geom::flatten_spine(&points, &handles, ed.model.closed, &mut spine);
        // Which tangent handle (if any) is being dragged — for the overlay accent colour.
        let grabbed_handle = match ed.grab {
            Some(CurveGrab::Tangent(i, is_out)) => Some((i, is_out)),
            _ => None,
        };
        let tangents = osel.and_then(|s| {
            curve_tangent::build_tangents(
                &points,
                &handles,
                s,
                grabbed_handle,
                self.paint.shape_grab_tol_px,
                ed.model.closed,
            )
        });
        let selected_kind = ed.model.selected_kind_wire();
        let (grabbed, rotating) = match &ed.gizmo {
            Some(g) => (Some(g.handle), curve_gizmo::is_rotate(g)),
            None => (None, false),
        };
        let transform_gizmo =
            curve_gizmo::overlay(&points, self.paint.shape_grab_tol_px, grabbed, rotating);
        Some(CurveOverlay {
            points,
            selected: osel,
            spine,
            tangents,
            selected_kind,
            transform_gizmo,
        })
    }
}
