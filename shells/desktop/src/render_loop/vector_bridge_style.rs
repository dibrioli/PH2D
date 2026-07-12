//! **O estilo do traço**, do painel para o documento — módulo irmão do `vector_bridge`
//! (teto de 600 LOC por arquivo da shell).
//!
//! Aqui mora a metade "escrita" da ponte: o registro [`StrokeStyle`] que o painel edita, o
//! DETECTOR de mudança (`differs_from`) e o ESCRITOR (`onto`) — de propósito juntos.
//!
//! # Por que detector e escritor são o mesmo registro
//!
//! Um campo presente num e ausente no outro é um controle que mexe o número e **não muda nada
//! na tela** — e o compilador não diz uma palavra. O slider anda, o valor entra no snapshot, e
//! a linha continua igual; o usuário conclui que o parâmetro não faz nada. Foi o risco vivo do
//! Head Size e do Head Round, e é por isso que os dois lados leem a MESMA struct.

use ph2d_vec_edit::PenTool;
use ph2d_vec_render::GradHandle;
use ph2d_vec_scene::{LineCap, LineJoin, Paint, Rgba8, StrokeSpec, VecScene};
use std::cell::{Cell, RefCell};

pub(crate) fn rgba(c: [u8; 4]) -> Rgba8 {
    Rgba8::new(c[0], c[1], c[2], c[3])
}

/// **A ficha de traço que o painel manda** — tudo o que a tool possui do stroke, MENOS a
/// largura (que só muda enquanto o slider é arrastado, e por isso tem regra própria).
///
/// Uma struct, e não oito valores soltos, porque **dois** lugares precisam falar da mesma
/// coisa: quem DETECTA a mudança ([`Self::differs_from`]) e quem a GRAVA ([`Self::onto`]).
/// Um campo presente num e ausente no outro é um controle que mexe no número e não muda nada
/// na tela — e o compilador não diz nada. Foi exatamente o risco do Head Size / Head Round.
#[derive(Copy, Clone, Debug)]
pub(crate) struct StrokeStyle {
    pub color: Rgba8,
    pub cap: LineCap,
    pub join: LineJoin,
    pub dash: Option<(f64, f64)>,
    pub marker_start: ph2d_vec_scene::Marker,
    pub marker_end: ph2d_vec_scene::Marker,
    /// Tamanho da cabeça da seta (múltiplo) + arredondamento das quinas dela.
    pub marker_scale: f64,
    pub marker_round: f64,
}

impl StrokeStyle {
    /// O traço `s` já é esta ficha, ou reescrevê-lo mudaria alguma coisa? (A largura fica de
    /// fora: ela só acompanha a tool enquanto o slider é arrastado.)
    pub(crate) fn differs_from(&self, s: &StrokeSpec) -> bool {
        s.color != self.color
            || s.cap != self.cap
            || s.join != self.join
            || s.dash != self.dash
            || s.marker_start != self.marker_start
            || s.marker_end != self.marker_end
            || (s.marker_scale - self.marker_scale).abs() > f64::EPSILON
            || (s.marker_round - self.marker_round).abs() > f64::EPSILON
    }

    /// O `StrokeSpec` desta ficha com a largura `width`. **A ficha + a largura determinam o
    /// traço INTEIRO** — nada do spec antigo sobrevive, e é isso que garante que os dois
    /// lados (detectar / gravar) não possam divergir num campo esquecido.
    pub(crate) fn onto(&self, width: f64) -> StrokeSpec {
        StrokeSpec {
            color: self.color,
            width,
            cap: self.cap,
            join: self.join,
            dash: self.dash,
            marker_start: self.marker_start,
            marker_end: self.marker_end,
            marker_scale: self.marker_scale,
            marker_round: self.marker_round,
        }
    }
}

/// **Reestiliza o traço de TODOS os caminhos selecionados.** É o ponto da feature: mudar o
/// Head Size (ou a cor, ou a ponta) com o diagrama inteiro selecionado calibra o diagrama
/// inteiro — não só o primeiro da seleção. Devolve quantos traços foram reescritos.
///
/// `new_width = Some(w)` **só** enquanto o slider de largura é arrastado; senão cada caminho
/// mantém a largura dele (uma escolha de cor nunca pode reengrossar a linha).
pub(crate) fn restyle_selected_strokes(
    scene: &mut VecScene,
    selection: &[ph2d_vec_scene::VecPathId],
    style: &StrokeStyle,
    new_width: Option<f64>,
) -> usize {
    let mut n = 0;
    for &id in selection {
        let Some(path) = scene.path_mut(id) else {
            continue;
        };
        // Um caminho SEM traço (só preenchimento) não tem o que reestilizar — e ganhar um
        // traço do nada seria a UI inventando geometria.
        let Some(old) = path.stroke else {
            continue;
        };
        path.stroke = Some(style.onto(new_width.unwrap_or(old.width)));
        n += 1;
    }
    n
}

/// The colour the selected gradient handle addresses on `fill` — a multi-point
/// point's colour, or the ramp stop at the START/END end the linear/radial handle
/// sits on. Drives the Fill swatch, the picker seed, and the recolour. `None` if the
/// handle doesn't match the fill kind (e.g. a stale selection after a kind switch).
pub(crate) fn selected_grad_color(fill: &Paint, handle: GradHandle) -> Option<Rgba8> {
    match (fill, handle) {
        (Paint::MultiPoint { points }, GradHandle::Point(i)) => points.get(i).map(|gp| gp.color),
        (Paint::Linear { stops, .. }, GradHandle::LinearStart)
        | (Paint::Radial { stops, .. }, GradHandle::RadialCenter) => stops.first().map(|s| s.color),
        (Paint::Linear { stops, .. }, GradHandle::LinearEnd)
        | (Paint::Radial { stops, .. }, GradHandle::RadialEdge) => stops.last().map(|s| s.color),
        (Paint::Linear { stops, .. }, GradHandle::Stop(i))
        | (Paint::Radial { stops, .. }, GradHandle::Stop(i)) => stops.get(i).map(|s| s.color),
        _ => None,
    }
}

/// Recolour the slot the selected gradient handle addresses to `c`; returns whether
/// it changed. Mirror of [`selected_grad_color`] on the mutable fill.
pub(crate) fn set_selected_grad_color(fill: &mut Paint, handle: GradHandle, c: Rgba8) -> bool {
    let slot = match (fill, handle) {
        (Paint::MultiPoint { points }, GradHandle::Point(i)) => {
            points.get_mut(i).map(|gp| &mut gp.color)
        }
        (Paint::Linear { stops, .. }, GradHandle::LinearStart)
        | (Paint::Radial { stops, .. }, GradHandle::RadialCenter) => {
            stops.first_mut().map(|s| &mut s.color)
        }
        (Paint::Linear { stops, .. }, GradHandle::LinearEnd)
        | (Paint::Radial { stops, .. }, GradHandle::RadialEdge) => {
            stops.last_mut().map(|s| &mut s.color)
        }
        (Paint::Linear { stops, .. }, GradHandle::Stop(i))
        | (Paint::Radial { stops, .. }, GradHandle::Stop(i)) => {
            stops.get_mut(i).map(|s| &mut s.color)
        }
        _ => None,
    };
    match slot {
        Some(slot) if *slot != c => {
            *slot = c;
            true
        }
        _ => false,
    }
}

/// Push `alpha` (0..255) onto an Opacity slider's stored value — unless the user
/// is dragging it — so a colour-picker alpha change reflects on the panel and the
/// drag baseline stays correct. The linked chip's display is driven from the
/// slider track in `paint`, so it follows without a separate push.
pub(crate) fn sync_opacity_slider(
    store: &mut ph2d_editor::interaction::WidgetStore,
    id: ph2d_editor::NodeId,
    alpha: u8,
) {
    use ph2d_editor::InteractiveState;
    use ph2d_editor::widget::SliderState;
    if let Some(InteractiveState::Slider { state, value, .. }) = store.get_mut(id)
        && !matches!(*state, SliderState::Dragging)
    {
        *value = f32::from(alpha) / 255.0;
    }
}

thread_local! {
    /// Pre-image of the scene captured at the START of a recolour gesture (the
    /// first frame the colour actually changes the selected path). Committed to
    /// `History` as ONE undo step when the gesture ends (the picker closes /
    /// the discrete pick's frame finishes). `None` between gestures.
    pub(crate) static RECOLOR_PRE: RefCell<Option<VecScene>> = const { RefCell::new(None) };
    /// O caminho cujas PONTAS a tool adotou por último — o "alvo" dos dois seletores de
    /// marker, no mesmo modelo do alvo dos campos de forma (`vec_shape_params`). Só a
    /// MUDANÇA de alvo semeia; semear todo frame brigaria com a escolha que o usuário
    /// acabou de fazer (o `SetValue` do popover chega DEPOIS deste passe, no frame
    /// seguinte, e seria imediatamente desfeito).
    static MARKER_TARGET: Cell<Option<ph2d_vec_scene::VecPathId>> = const { Cell::new(None) };
}

/// **Semeia os seletores de ponta a partir do caminho SELECIONADO** (quando a seleção
/// muda). O painel pinta a partir da tool, então sem isto os dois chips mostrariam a
/// última ponta autorada — não a do traço que está na tela. Espelho exato do
/// `seed_shape_fields` + `adopt_shape_values` dos campos de forma.
///
/// Um caminho SEM traço (só preenchimento) não tem ponta nenhuma a doar: o alvo passa a
/// ser ele, mas o Style da tool fica onde estava (o default do próximo traço).
pub(crate) fn seed_markers_from_selection(
    tool: &mut ph2d_tool_vector::VectorTool,
    pen: &PenTool,
    scene: &VecScene,
) {
    let target = pen.selected();
    if MARKER_TARGET.with(Cell::get) == target {
        return;
    }
    MARKER_TARGET.with(|c| c.set(target));
    let Some(stroke) = target
        .and_then(|id| scene.paths().iter().find(|p| p.id == id))
        .and_then(|p| p.stroke)
    else {
        return;
    };
    tool.adopt_markers(
        stroke.marker_start,
        stroke.marker_end,
        stroke.marker_scale,
        stroke.marker_round,
    );
}
