//! **A CANETA da seleção** — o Pen do vetor, dentro do Painter (Enio, 2026-08-07: *"no modo de seleção
//! poderíamos criar um modo pen exatamente como a pen do vector"*).
//!
//! ## Por que isto é pequeno
//!
//! A seleção **já sabe segurar e editar uma Bézier fechada**: `SelectionShape::Freehand` carrega um
//! [`CurveModel`] — o MESMO núcleo de edição que o editor de curva do traço possui — e o Edit mode já
//! desenha o editor por-âncora dele. O que faltava não era a representação, era uma forma de **AUTORAR**
//! uma diretamente, em vez de laçar-e-converter. Então esta wave escreve o GESTO e mais nada: a
//! rasterização (`freehand_spine` → `raster_lasso`), as operações booleanas, o Offset, o Edit mode e o
//! undo chegam prontos, porque a peça que ela produz é a peça que o Convert to Curve já produzia.
//!
//! ## As três leis do gesto
//!
//! 1. **Uma sessão sobrevive ao pen-up.** Todo outro modo de seleção é um gesto Down→Up; a caneta é uma
//!    conversa de vários cliques, e por isso mora num campo próprio ([`PaintState::selection_pen`]) em
//!    vez de no `selection_drag`, que é `take()`-ado no Up.
//! 2. **A base booleana é capturada UMA vez, no início da sessão.** Add/Remove compõem contra a seleção
//!    que existia *antes da caneta*, nunca contra o preview do clique anterior — senão cada âncora nova
//!    somaria em cima da anterior e o `Add` viraria uma acumulação que ninguém pediu.
//! 3. **Uma região de seleção é FECHADA.** Um caminho aberto não é uma região, então a caneta não tem
//!    "commit aberto": fechar no primeiro ponto entrega, `Enter` fecha implicitamente (o mesmo que o
//!    laço faz ao soltar) e `Esc` descarta. Não há terceira saída.
//!
//! ⚠️ **A peça que ela entrega já é uma CURVA** (`handles.len() == points.len()` ⇒
//! [`CurveModel::is_curve`]), e é isso que a separa do laço: um laço nasce polilinha e só vira editável
//! ponto-a-ponto depois do Convert; uma caneta nasce editável, porque o artista acabou de colocar cada
//! tangente com a mão.

use super::PainterTool;
use super::curve_handle::{self, HandleKind};
use super::curve_model::CurveModel;
use super::selection_shapes::SelectionShape;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use std::sync::Arc;

/// O discriminante do modo Pen no seletor da seção Selection.
pub(super) const SELECTION_MODE_PEN: u8 = 4;
/// Mínimo de âncoras para uma região: menos que três não fecha área nenhuma.
const MIN_CLOSE_POINTS: usize = 3;

/// A sessão de caneta em voo — o caminho autorado até agora + o que a mão está fazendo com ele.
///
/// O `model` nasce ABERTO (`closed = false`) e só fecha no commit: enquanto ele está aberto, o desenho
/// que o artista vê é o caminho, e a região que o preview aplica é esse caminho **implicitamente
/// fechado** — exatamente o que ele receberá ao fechar.
pub(crate) struct SelectionPen {
    /// O caminho autorado — âncoras + tangentes `[in, out]` + tipos, sempre paralelos.
    pub(super) model: CurveModel,
    /// `true` entre o Down que pôs a última âncora e o Up dele: o arrasto que puxa as tangentes dela.
    dragging: bool,
    /// Onde a mão está com o botão SOLTO — a linha-fantasma da próxima âncora. `None` até o 1º hover.
    hover: Option<[f32; 2]>,
}

impl PainterTool {
    /// **Há uma caneta em voo?** — a porta única que o roteador de ponteiro, o overlay e as teclas
    /// Enter/Esc perguntam.
    #[must_use]
    pub fn selection_pen_live(&self) -> bool {
        self.paint.selection_pen.is_some()
    }

    /// Quantas âncoras a sessão já tem (`0` sem sessão) — o que o painel usa para dizer se `Enter` já
    /// fecha alguma coisa.
    #[must_use]
    pub fn selection_pen_points(&self) -> usize {
        self.paint
            .selection_pen
            .as_ref()
            .map_or(0, |p| p.model.points.len())
    }

    /// A linha-fantasma: a mão moveu sobre o canvas com o botão solto. Alimentada pelo
    /// `on_canvas_hover` da shell — o MESMO evento que já mira o anel do pincel.
    pub(super) fn selection_pen_hover(&mut self, pos: [f32; 2]) {
        if let Some(pen) = self.paint.selection_pen.as_mut() {
            pen.hover = Some(pos);
        }
    }

    /// Rota o ponteiro do canvas para a caneta. `true` sempre que a sessão consome (ela é modal
    /// enquanto viva, como a peça colada).
    pub(super) fn selection_pen_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.selection_pen_down(ev.pos),
            PointerPhase::Move => self.selection_pen_drag(ev.pos),
            PointerPhase::Up => {
                if let Some(pen) = self.paint.selection_pen.as_mut() {
                    pen.dragging = false;
                }
                true
            }
            PointerPhase::Hover => {
                self.selection_pen_hover(ev.pos);
                false
            }
        }
    }

    /// Um clique: fecha (se voltou ao primeiro ponto) ou põe a âncora seguinte — e abre a sessão quando
    /// ainda não havia uma.
    fn selection_pen_down(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        // Fechar tem prioridade sobre acrescentar: o primeiro ponto é o alvo mais específico, e sem esta
        // ordem clicar nele empilharia uma âncora em cima da outra em vez de terminar a forma.
        let close = self.paint.selection_pen.as_ref().is_some_and(|pen| {
            pen.model.points.len() >= MIN_CLOSE_POINTS
                && pen.model.points.first().is_some_and(|&f| {
                    let (dx, dy) = (pos[0] - f[0], pos[1] - f[1]);
                    dx * dx + dy * dy <= tol * tol
                })
        });
        if close {
            self.selection_pen_commit();
            return true;
        }
        match self.paint.selection_pen.as_mut() {
            Some(pen) => {
                pen.model.points.push(pos);
                pen.model.handles.push([pos, pos]);
                pen.model.kinds.push(HandleKind::Vector);
                pen.model.selected = Some(pen.model.points.len() - 1);
                pen.dragging = true;
            }
            None => {
                // A base booleana é a seleção que existe AGORA, e ela vale pela sessão inteira (lei 2).
                self.ensure_selection_mask();
                self.paint.selection_base = Arc::clone(&self.paint.selection_crisp);
                self.paint.stroke_undo = Some(self.snapshot_model());
                self.paint.selection_pen = Some(SelectionPen {
                    model: CurveModel::from_curve(
                        vec![pos],
                        vec![[pos, pos]],
                        false,
                        HandleKind::Vector,
                    ),
                    dragging: true,
                    hover: Some(pos),
                });
            }
        }
        self.selection_pen_preview();
        true
    }

    /// O arrasto que sai de um clique: puxa a tangente de saída da última âncora e **espelha** a de
    /// entrada — o clássico "arrasta para curvar" da caneta, e a razão de o tipo virar `Symmetric`
    /// (manual, logo preservado pelo `rebuild` quando as vizinhas se re-derivam).
    fn selection_pen_drag(&mut self, pos: [f32; 2]) -> bool {
        let Some(pen) = self.paint.selection_pen.as_mut() else {
            return false;
        };
        if !pen.dragging {
            pen.hover = Some(pos);
            return true;
        }
        let Some(i) = pen.model.points.len().checked_sub(1) else {
            return true;
        };
        let a = pen.model.points[i];
        pen.model.handles[i] = [[2.0 * a[0] - pos[0], 2.0 * a[1] - pos[1]], pos];
        pen.model.kinds[i] = HandleKind::Symmetric;
        pen.hover = Some(pos);
        self.selection_pen_preview();
        true
    }

    /// O preview vivo: o caminho **implicitamente fechado** vira região e entra na máscara pela mesma
    /// porta dos outros modos. Abaixo de três âncoras não há área, e o que o artista vê é o overlay da
    /// caneta — a região simplesmente ainda não existe.
    fn selection_pen_preview(&mut self) {
        let Some(pen) = self.paint.selection_pen.as_ref() else {
            return;
        };
        if pen.model.points.len() < MIN_CLOSE_POINTS {
            return;
        }
        let spine = self.freehand_spine_closed(&pen.model);
        let region = self.raster_lasso(&spine);
        self.apply_selection_region(&region);
        self.invalidate_composite();
    }

    /// O contorno FECHADO do caminho autorado — a mesma achatadura Bézier que a rasterização usa, com
    /// `closed` forçado. Uma porta, porque o preview e o commit têm de concordar sobre que forma é essa.
    fn freehand_spine_closed(&self, model: &CurveModel) -> Vec<[f32; 2]> {
        let mut closed = model.clone();
        closed.closed = true;
        curve_handle::rebuild(
            &closed.points,
            &closed.kinds,
            &mut closed.handles,
            true, // as tangentes derivadas (Vector) olham o vizinho pelo anel, não pela ponta
        );
        self.freehand_spine(&closed.points, &closed.handles)
    }

    /// **Fecha e entrega**: o caminho vira uma `Freehand` fechada na lista de formas, com a operação
    /// booleana escolhida, em UM passo de undo. `true` quando havia o que entregar.
    ///
    /// ⚠️ Uma sessão com menos de três âncoras é **descartada**, não entregue: ela não delimita área
    /// nenhuma, e commitá-la gravaria um passo de undo cujo efeito é zero.
    pub fn selection_pen_commit(&mut self) -> bool {
        let Some(pen) = self.paint.selection_pen.take() else {
            return false;
        };
        self.paint.hover_pos = None;
        if pen.model.points.len() < MIN_CLOSE_POINTS {
            self.selection_pen_abandon();
            return true;
        }
        let mut model = pen.model;
        model.closed = true;
        model.selected = None;
        curve_handle::rebuild(&model.points, &model.kinds, &mut model.handles, true);
        let spine = self.freehand_spine(&model.points, &model.handles);
        let region = self.raster_lasso(&spine);
        self.apply_selection_region(&region);
        self.push_selection_entry(
            SelectionShape::Freehand {
                model,
                u: [1.0, 0.0],
            },
            self.paint.selection_bool_op,
        );
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
        self.paint.selection_base = Arc::new(Vec::new());
        self.invalidate_composite();
        true
    }

    /// **Esc**: joga a sessão fora e devolve a seleção ao que era antes dela. `true` quando havia uma.
    ///
    /// Nada foi commitado, então não há passo de undo a gastar — o `stroke_undo` guardado no início da
    /// sessão é o estado a reinstalar, não um passo a gravar (a mesma economia da peça colada).
    pub fn selection_pen_cancel(&mut self) -> bool {
        if self.paint.selection_pen.take().is_none() {
            return false;
        }
        self.selection_pen_abandon();
        true
    }

    /// Devolve a máscara ao estado do início da sessão e solta o que ela segurava. Chamada pelas DUAS
    /// mortes (cancelar, e o commit degenerado) — escrever a limpeza em cada uma é como a terceira nasce
    /// sem ela.
    fn selection_pen_abandon(&mut self) {
        if let Some(before) = self.paint.stroke_undo.take() {
            self.restore_model(before);
        }
        self.paint.selection_base = Arc::new(Vec::new());
        self.invalidate_composite();
    }

    /// O que a shell DESENHA da caneta — o mesmo snapshot que o editor de curva do traço publica, para
    /// que exista uma resposta só a *"como este app desenha uma Bézier sendo autorada"*.
    ///
    /// A linha-fantasma até o cursor entra no `spine` (e não como campo novo): ela é parte do caminho que
    /// o artista está vendo, e o desenhador não precisa saber que a última perna é provisória.
    #[must_use]
    pub(super) fn selection_pen_overlay(&self) -> Option<super::curve::CurveOverlay> {
        let pen = self.paint.selection_pen.as_ref()?;
        let mut spine = Vec::new();
        super::curve_geom::flatten_spine(&pen.model.points, &pen.model.handles, false, &mut spine);
        if let (Some(&h), Some(&last)) = (pen.hover.as_ref(), pen.model.points.last())
            && !pen.dragging
        {
            spine.push(last);
            spine.push(h);
        }
        let sel = pen.model.selected;
        let tangents = sel.and_then(|i| {
            let a = *pen.model.points.get(i)?;
            let h = *pen.model.handles.get(i)?;
            let live = |p: [f32; 2]| (p != a).then_some(p);
            Some(super::curve_tangent::TangentHandles {
                anchor: a,
                in_handle: live(h[0]),
                out_handle: live(h[1]),
                grabbed_out: pen.dragging.then_some(true),
            })
        });
        Some(super::curve::CurveOverlay {
            points: pen.model.points.clone(),
            selected: sel,
            spine,
            tangents,
            selected_kind: None,
            transform_gizmo: None,
        })
    }
}
