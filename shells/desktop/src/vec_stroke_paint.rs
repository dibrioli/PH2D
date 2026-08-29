//! ⭐⭐ **COM QUE TINTA O TRAÇO DESENHA** — a porta única da fileira *Type* da secção *Stroke*
//! (plano 35, wave D).
//!
//! # Por que é um módulo próprio, e irmão do [`crate::vec_stroke_present`]
//!
//! Aquele responde *"esta forma TEM traço?"*; este, *"com que tinta ele desenha?"*. São duas
//! perguntas sobre o mesmo objecto, e a resposta de uma **pressupõe** a da outra — sem traço não há
//! tinta de traço. Mantê-las juntas num ficheiro faria a segunda herdar o `Option` da primeira por
//! acidente; separadas, cada uma declara o próprio `None`.
//!
//! # ⛔ Por que a lista tem DUAS variantes e não as cinco do preenchimento
//!
//! O renderer de traço não desenha gradiente. Um chip que produzisse um `StrokePaint::Linear`
//! gravaria estado que **nada pinta** — o documento leria de volta uma tinta inalcançável, e o
//! sintoma seria uma forma que abre diferente de como fechou. A recusa está escrita no
//! [plano 35 §2.1](../../docs/Vector%20Module/35_plano_padrao_no_traco.md); quando um gradiente no
//! traço for pedido, o `StrokePaint` ganha uma variante e esta lista ganha um chip.

use ph2d_panel_vector::StrokePaintKind;
use ph2d_vec_edit::{History, PenTool};
use ph2d_vec_scene::{PatternFill, PatternSource, StrokePaint, VecScene};

/// O chip de tinta de traço que este `NodeId` nomeia (`None` se não é um deles).
///
/// ⚠️ **Uma porta, e não dois `if` no despacho** — a mesma forma do `vec_fill_kind_for_id`: quem
/// acrescentar uma variante ao `StrokePaint` acrescenta-a aqui e o despacho não muda uma linha.
#[must_use]
pub(crate) fn kind_for_id(id: ph2d_editor::NodeId) -> Option<StrokePaintKind> {
    if id == ph2d_editor::ids::VECTOR_STROKE_KIND_SOLID {
        Some(StrokePaintKind::Solid)
    } else if id == ph2d_editor::ids::VECTOR_STROKE_KIND_PATTERN {
        Some(StrokePaintKind::Pattern)
    } else {
        None
    }
}

/// **A tinta do traço da forma selecionada.** `None` quando não há uma resposta — nada selecionado,
/// selecção múltipla, ou **a forma não tem traço**.
///
/// ⚠️ O terceiro caso é o que distingue esta porta da irmã: uma forma sem traço tem uma resposta
/// para *"tem traço?"* (`Some(false)`) e **nenhuma** para *"que tinta?"*. É por isso que a caixa é
/// pintada e a fileira de tipo não.
#[must_use]
pub(crate) fn selected_stroke_paint_kind(
    scene: &VecScene,
    pen: &PenTool,
) -> Option<StrokePaintKind> {
    let [id] = pen.selected_paths() else {
        return None;
    };
    let s = scene.path(*id)?.stroke.as_ref()?;
    Some(match s.paint {
        StrokePaint::Solid(_) => StrokePaintKind::Solid,
        StrokePaint::Pattern(_) => StrokePaintKind::Pattern,
    })
}

/// **Troca a tinta do traço da forma selecionada.** `true` se o documento mudou (um passo de undo).
///
/// - `Solid` -> a cor de recurso do padrão (`StrokeSpec::color()`), que é a que a linha já pintava
///   enquanto o ladrilho não resolvia. ⭐ **Ir e voltar não pisca para uma cor arbitrária.**
/// - `Pattern` **já sendo padrão** -> não mexe: a arte, o reticulado e a colocação sobrevivem a
///   trocar de chip e voltar, exactamente como no preenchimento.
/// - `Pattern` **sem ser** -> precisa de `pattern`, que vem de fora resolvido.
///
/// ⚠️⚠️ **`pattern == None` com `Pattern` é DESISTÊNCIA e não muda nada** — o artista fechou o
/// diálogo da arte, e apagar-lhe a cor do traço por isso seria o pior dos dois mundos. É a mesma lei
/// do `apply_vec_set_fill_kind`, e por isso está escrita do mesmo jeito.
///
/// ⚠️ **A fonte vem RESOLVIDA de fora** porque escolhê-la pode abrir um diálogo de ficheiro, que
/// congela o laço — isso é da shell (`crate::modal`), nunca desta função de documento.
pub(crate) fn set_kind(
    scene: &mut VecScene,
    history: &mut History,
    pen: &PenTool,
    kind: StrokePaintKind,
    pattern: Option<(PatternSource, [f64; 2], [f64; 2])>,
) -> bool {
    let [id] = pen.selected_paths() else {
        return false;
    };
    let id = *id;
    let Some(cur) = scene.path(id).and_then(|p| p.stroke.as_ref()) else {
        return false;
    };
    let novo = match (kind, &cur.paint) {
        // Já é o que se pediu: nada a fazer, e a lei inteira do padrão sobrevive.
        (StrokePaintKind::Solid, StrokePaint::Solid(_))
        | (StrokePaintKind::Pattern, StrokePaint::Pattern(_)) => return false,
        // ⚠️ A cor que fica é a `fallback` do padrão — a que a linha já mostrava.
        (StrokePaintKind::Solid, _) => StrokePaint::Solid(cur.color()),
        (StrokePaintKind::Pattern, _) => {
            let Some((source, size, origin)) = pattern else {
                return false;
            };
            let mut f = PatternFill::new(source, size, cur.color());
            // ⛔ O canto é o da FORMA, não a origem do mundo — a lei que o `Clamp` do preenchimento
            // pagou com um report (`texture_pattern_pick::default_placement`).
            f.origin = origin;
            StrokePaint::Pattern(Box::new(f))
        }
    };
    let pre = scene.clone();
    let Some(path) = scene.path_mut(id) else {
        return false;
    };
    let Some(s) = path.stroke.as_mut() else {
        return false;
    };
    s.paint = novo;
    history.push_undo(pre);
    true
}

#[cfg(test)]
#[path = "vec_stroke_paint_tests.rs"]
mod tests;
