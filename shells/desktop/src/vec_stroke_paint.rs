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
        StrokePaint::Brush(_) => StrokePaintKind::Brush,
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
        | (StrokePaintKind::Pattern, StrokePaint::Pattern(_))
        | (StrokePaintKind::Brush, StrokePaint::Brush(_)) => return false,
        // ⏳ **O PINCEL ainda não se cria por aqui** (plano 36: o modelo é a W1, a criação é a W4).
        //
        // ⚠️ **Recusar em voz baixa é o certo, e não um `todo!()`:** esta porta é o dreno de um
        // clique, e um panic aqui derrubaria o app se alguém publicasse o chip antes da hora. ⛔ E
        // um `_ => return false` genérico calaria também os casos de cima — *o enum é fechado
        // precisamente para que a próxima variante me traga aqui.*
        (StrokePaintKind::Brush, _) => return false,
        // ⚠️ A cor que fica é a `fallback` do padrão — a que a linha já mostrava.
        (StrokePaintKind::Solid, _) => StrokePaint::Solid(cur.color()),
        (StrokePaintKind::Pattern, _) => {
            let Some((source, size, origin)) = pattern else {
                return false;
            };
            let mut f = PatternFill::new(source, size, cur.color());
            // ⚠️ **A OPACIDADE atravessa a troca de tinta.** Um traço a 50% que vira padrão nasceria
            // com `alpha = 1,0` (o default do construtor) e **saltaria para opaco** no clique; e a
            // primeira mexida no painel puxá-lo-ia de volta a 50%, porque é ali que a opacidade do
            // traço mora (`StrokeStyle::onto`). *Uma opacidade, uma casa — inclusive no nascimento.*
            f.alpha = f32::from(cur.color().a) / 255.0;
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

/// ⭐⭐⭐ **Põe a ARTE de um pincel** (plano 36, W4) — a porta do gesto de duas mãos.
///
/// ⚠️ Resolve por **ID** e não pela selecção, pela mesma razão que o picker do padrão: o alvo é
/// capturado no *arm*, e o clique seguinte cai noutra forma, que passa a ser a selecionada. Ler a
/// selecção aqui apontaria o pincel para a forma errada.
///
/// ⛔ **Uma forma não pode ser o próprio pincel** — a recusa é a primeira linha, e há uma segunda,
/// PURA, no `brush_live`. *Duas metades porque as duas portas existem: esta autora, aquela resolve.*
///
/// `true` se o documento mudou (um passo de undo).
pub(crate) fn set_art(
    scene: &mut VecScene,
    history: &mut History,
    host: ph2d_vec_scene::VecPathId,
    art: ph2d_vec_scene::VecPathId,
) -> bool {
    if art == host {
        return false;
    }
    let Some(cur) = scene
        .path(host)
        .and_then(|p| p.stroke.as_ref())
        .and_then(ph2d_vec_scene::StrokeSpec::brush)
    else {
        return false;
    };
    if cur.art == Some(art) {
        return false;
    }
    let mut next = cur.clone();
    next.art = Some(art);
    let pre = scene.clone();
    let Some(s) = scene.path_mut(host).and_then(|p| p.stroke.as_mut()) else {
        return false;
    };
    s.paint = StrokePaint::Brush(Box::new(next));
    history.push_undo(pre);
    true
}

/// **O que a secção *Brush* pede ao documento** (plano 36, W4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BrushCmd {
    /// Multiplica a altura derivada da largura do traço.
    Scale(f64),
    /// Multiplica a largura do motivo para dar o avanço.
    Spacing(f64),
    /// Desvio ao longo da normal, em unidades de mundo.
    Offset(f64),
    /// Orientação do motivo sobre a curva, em GRAUS.
    Rotation(f64),
    /// A arte do outro lado da curva.
    Flip,
}

/// Aplica `cmd` ao pincel da forma selecionada. No-op silencioso quando não há forma, quando o
/// traço não é um pincel, ou quando o valor já era esse.
///
/// ⚠️ **O `if` de igualdade no fim é o que impede um passo espúrio** quando o slider re-publica o
/// valor que já lá estava — a mesma disciplina da porta do padrão.
pub(crate) fn apply(
    scene: &mut VecScene,
    history: &mut History,
    pen: &PenTool,
    cmd: BrushCmd,
) -> bool {
    let Some(sel) = pen.selected() else {
        return false;
    };
    let Some(cur) = scene
        .path(sel)
        .and_then(|p| p.stroke.as_ref())
        .and_then(ph2d_vec_scene::StrokeSpec::brush)
    else {
        return false;
    };
    let mut next = cur.clone();
    match cmd {
        BrushCmd::Scale(v) => next.scale = v,
        BrushCmd::Spacing(v) => next.spacing = v,
        BrushCmd::Offset(v) => next.offset = v,
        BrushCmd::Rotation(v) => next.rotation_deg = v,
        BrushCmd::Flip => next.flip = !next.flip,
    }
    if &next == cur {
        return false;
    }
    let pre = scene.clone();
    let Some(s) = scene.path_mut(sel).and_then(|p| p.stroke.as_mut()) else {
        return false;
    };
    s.paint = StrokePaint::Brush(Box::new(next));
    history.push_undo(pre);
    true
}

/// O comando que este `NodeId` nomeia (`None` se não é um clique da secção *Brush*).
#[must_use]
pub(crate) fn cmd_for_id(id: ph2d_editor::NodeId) -> Option<BrushCmd> {
    (id == ph2d_editor::ids::VECTOR_BRUSH_FLIP).then_some(BrushCmd::Flip)
}

/// O comando de um SLIDER da secção *Brush* (`None` se não é dela). ⚠️ O `event.rs` do painel já
/// converteu o track para o domínio do documento — aqui `v` é valor.
#[must_use]
pub(crate) fn slider_cmd_for_id(id: ph2d_editor::NodeId, v: f64) -> Option<BrushCmd> {
    use ph2d_editor::ids as i;
    if id == i::VECTOR_BRUSH_SCALE {
        Some(BrushCmd::Scale(v))
    } else if id == i::VECTOR_BRUSH_SPACING {
        Some(BrushCmd::Spacing(v))
    } else if id == i::VECTOR_BRUSH_OFFSET {
        Some(BrushCmd::Offset(v))
    } else if id == i::VECTOR_BRUSH_ROTATION {
        Some(BrushCmd::Rotation(v))
    } else {
        None
    }
}

#[cfg(test)]
#[path = "vec_stroke_paint_tests.rs"]
mod tests;
