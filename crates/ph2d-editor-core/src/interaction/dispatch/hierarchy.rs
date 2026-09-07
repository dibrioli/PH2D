//! Hierarchy panel drag-and-drop resolution.
//!
//! Extracted from [`super`] (Track A9). One responsibility — given
//! the cursor's y-position over the hierarchy panel and the row
//! being dragged, decide where the drop should land. Row geometry
//! is split into three vertical bands (top 30% → sibling above,
//! middle 40% → re-parent inside, bottom 30% → sibling below).
//!
//! [`HierDrop`] is `pub(crate)` because the host's pointer-Up
//! handler in `screens::hero` matches on it to issue the actual
//! ECS mutation.

use super::super::{HitIndex, PainterLayerDrop, WidgetStore};
use ph2d_a11y::NodeId;

/// Drop kind resolved at the end of a hierarchy DnD: a sibling
/// insertion (above or below the given row), or a re-parent inside
/// the given row.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum HierDrop {
    /// Drop dragged just before this row as a sibling.
    Before(NodeId),
    /// Drop dragged just after this row as a sibling. Resolved by
    /// the host to `(target's parent, before = target's next
    /// sibling, or None for "append at end")`. Used for the bottom-
    /// 30% drop band on the LAST visible child of a parent — the
    /// pre-M14.7-polish `End` fallthrough turned that into a root
    /// promotion instead of "append in this parent".
    After(NodeId),
    /// Drop dragged as a child of this row.
    Inside(NodeId),
    /// Drop at the very bottom (root level, end of list).
    End,
}

/// Resolve the drop position for a hierarchy DnD using the cursor's
/// vertical position vs each row rect. Row y is split into three
/// bands:
///   - top 30% → drop above this row (sibling)
///   - middle 40% → drop inside this row (child)
///   - bottom 30% → continue scanning (drop below this row)
///
/// Horizontal position (cursor_x) is intentionally NOT used to gate
/// `Inside` vs `Before` — that previous attempt blocked the common
/// case "drag X onto Y to make X a child of Y when Y is already
/// someone's child". Users naturally hold the cursor near where they
/// pressed down, which may sit left of an indented target row;
/// requiring `cursor_x >= row.x` then forced sibling semantics even
/// when the user clearly aimed at the row vertically. Painter
/// indicator mirrors this same y-only logic, so what the user sees
/// is what they get.
///
/// When the cursor lands below every row, returns `End` — a true
/// root append (now safe thanks to the `RootOrder` component; before
/// that, `End` snapped back to `Entity::to_bits` sort). The
/// in-row bottom-30% band still resolves to `After(id)` so dropping
/// at the foot of a nested last-child still appends inside that
/// child's parent — the user's "drop after t preserves parent"
/// behavior. Skips the dragged row itself.
/// ⭐⭐⭐ **O DIAGNÓSTICO DO ARRASTO DE LINHA** (report do Enio, 2026-09-07: *«reordenei objectos na
/// hierarquia e não funcionou o undo»*).
///
/// ⚠️ A corrida com o log do undo provou que a ORDEM não muda e que o pedido **não chega** ao lado
/// que a escreve — e o gesto tem TRÊS estações (semear no `Down`, activar no `Move`, resolver no
/// `Up`), cada uma com a sua forma de morrer em silêncio. Sem isto, cada suspeita custa uma corrida
/// do dono.
///
/// ⛔ Mesma env do log do undo (`PH2D_UNDO_LOG`), de propósito: o artista já a tem na mão, e uma
/// segunda variável seria uma segunda coisa para ele saber.
pub(crate) fn diag_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_UNDO_LOG").is_some())
}

/// ⭐⭐ **Estação 1 do arrasto de linha — e ela fala em TODA pressão, não só nas que acertam.**
///
/// ⚠️ A redacção anterior vivia **dentro** do braço `if let Some(hit)` e ainda por baixo do
/// `is_focusable`: uma pressão no canvas — que é onde o dono de facto arrastou no smoke de
/// 2026-09-07 — não deixava rasto nenhum, e **uma pressão sem log e um gesto que falha são a
/// mesma linha em branco**. O diagnóstico passa a correr antes de qualquer triagem.
///
/// A moldura das linhas registadas é a metade que decide: ela separa *«pressionei o painel
/// errado»* de *«o painel não me ouviu»*, que é a pergunta em aberto deste report e que nenhuma
/// contagem de linhas responde (três linhas registadas dizem que a lista existe, nunca ONDE).
pub(super) fn diag_down(store: &WidgetStore, hit_index: &HitIndex, x: f32, y: f32) {
    if !diag_on() {
        return;
    }
    let mut bbox: Option<[f32; 4]> = None;
    for (id, r) in hit_index.iter_registrations() {
        if !store.is_hierarchy_row(id) {
            continue;
        }
        let b = bbox.get_or_insert([r.x, r.y, r.x + r.w, r.y + r.h]);
        b[0] = b[0].min(r.x);
        b[1] = b[1].min(r.y);
        b[2] = b[2].max(r.x + r.w);
        b[3] = b[3].max(r.y + r.h);
    }
    let alvo = match hit_index.hit_with_rect(x, y) {
        Some((id, _)) if store.is_hierarchy_row(id) => "LINHA DA HIERARQUIA".to_string(),
        Some((id, _)) => format!("outro widget {id:?}"),
        None => "nada (canvas ou fundo de painel)".to_string(),
    };
    let n = store.hierarchy_row_count();
    match bbox {
        Some(b) => eprintln!(
            "[hier] down em ({x:.0},{y:.0}) -> {alvo} | {n} linha(s) da hierarquia ocupam x {:.0}..{:.0}, y {:.0}..{:.0}",
            b[0], b[2], b[1], b[3],
        ),
        None => eprintln!(
            "[hier] down em ({x:.0},{y:.0}) -> {alvo} | {n} linha(s) no store e NENHUMA registada no ecra'"
        ),
    }
}

pub(super) fn find_hierarchy_drop(
    hit_index: &HitIndex,
    store: &WidgetStore,
    cursor_y: f32,
    dragged: NodeId,
) -> HierDrop {
    for (id, rect) in hit_index.iter_registrations() {
        // Live mode: the static fixture range (400..=411) misses
        // every ECS-bridge row, so consult the store's per-frame
        // row set instead. The fixture range stays valid for the
        // demo's prepopulated ids — both pass `is_hierarchy_row`
        // because `populate_live` and `populate` both call
        // `set_hierarchy_row_ids`.
        if !store.is_hierarchy_row(id) {
            continue;
        }
        if id == dragged {
            continue;
        }
        let top = rect.y;
        let bot = rect.y + rect.h;
        let inside_top = top + rect.h * 0.3;
        let inside_bot = top + rect.h * 0.7;
        if cursor_y < top || cursor_y >= bot {
            continue;
        }
        if cursor_y < inside_top {
            return HierDrop::Before(id);
        } else if cursor_y < inside_bot {
            return HierDrop::Inside(id);
        } else {
            // Bottom band: drop AS THE NEXT SIBLING of this row.
            // Host resolves "after t" to t's parent + slot just past
            // t in the Children list. Last-child-of-parent + drop
            // here = append to parent's Children.
            return HierDrop::After(id);
        }
    }
    // Cursor is below every visible row → root append. Host clears
    // `ChildOf` on the dragged entity and writes a fresh `RootOrder`
    // index past the last existing root, so the panel will paint it
    // at the very bottom on the next frame.
    HierDrop::End
}

/// Resolve where a dragged Painter layer row was dropped, by hit-testing
/// `cursor_y` against the registered row rects. Mirror of
/// [`find_hierarchy_drop`] — same 30/40/30 band split (Before / Inside /
/// After) — but keyed on [`WidgetStore::is_painter_layer_row`] and
/// returning a [`PainterLayerDrop`] for the painter tool to resolve against
/// its `LayerStack`. The dispatch does NOT mutate any structure here.
pub(super) fn find_painter_layer_drop(
    hit_index: &HitIndex,
    store: &WidgetStore,
    cursor_y: f32,
    dragged: NodeId,
) -> PainterLayerDrop {
    for (id, rect) in hit_index.iter_registrations() {
        if !store.is_painter_layer_row(id) || id == dragged {
            continue;
        }
        let top = rect.y;
        let bot = rect.y + rect.h;
        if cursor_y < top || cursor_y >= bot {
            continue;
        }
        let inside_top = top + rect.h * 0.3;
        let inside_bot = top + rect.h * 0.7;
        if cursor_y < inside_top {
            return PainterLayerDrop::Before(id);
        } else if cursor_y < inside_bot {
            return PainterLayerDrop::Inside(id);
        } else {
            return PainterLayerDrop::After(id);
        }
    }
    // Below every visible row → root level, bottom of the stack.
    PainterLayerDrop::End
}
