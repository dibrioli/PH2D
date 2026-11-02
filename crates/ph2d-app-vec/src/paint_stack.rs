//! ⭐⭐⭐ **OS VERBOS DA PILHA DE APARÊNCIA, do lado da shell** (estudo 42 item 4, v20).
//!
//! O que o painel MOSTRA da selecção, e o que cada clique dele escreve no documento.
//!
//! # A lei da selecção múltipla é a MESMA da aparência do objecto
//!
//! **Mostra o primário, escreve em todos** — a lei que a fileira de verbos booleanos já paga, e que
//! veio de um report: *tocar num filho selecciona o GRUPO*, então o sujeito de um readout é a forma
//! que o artista apontou.
//!
//! ⚠️ **E a ESCRITA é por ÍNDICE**, o que tem uma consequência declarada: acrescentar uma camada
//! acrescenta-a a **todas** as formas seleccionadas, e mexer na camada `2` mexe na `2` de cada uma.
//! Isso é o que o artista espera quando escolhe cinco formas e carrega em *+ Stroke*; ⛔ o que ele
//! **não** espera é que uma forma sem camada `2` ganhe uma — e por isso um índice que não existe é
//! **saltado**, nunca criado.

use ph2d_vec_render::GradHandle;
use ph2d_vec_scene::{
    MAX_PAINT_LAYERS, Paint, PaintEntry, PaintKind, PaintSlot, Place, Rgba8, StackItem,
    StrokeAlign, StrokeSpec, VecPathId, VecScene,
};

/// ⭐⭐⭐ **O QUE UM CLIQUE NA PILHA PEDE** — a metade PURA da ponte, testável sem `App`.
///
/// ⚠️ **A resolução varre o espaço FIXO de ids** ([`MAX_PAINT_LAYERS`]), e não a pilha de hoje: um
/// id de runtime nasce de um índice, e perguntar «que índice é este?» a uma lista que muda de
/// tamanho faria o mesmo clique resolver diferente conforme a forma seleccionada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackVerb {
    AddFill,
    AddStroke,
    /// Liga/desliga o olho da camada `i`.
    Eye(usize),
    /// Sobe a camada `i` (para o topo da pilha).
    Up(usize),
    Down(usize),
    Del(usize),
    /// ⭐ F45: o arrasto pelo ⚙ largou uma linha — o par (linha, lugar) está no estado do painel.
    Move,
}

/// Que verbo este id pede (`None` se ele não é da pilha).
pub fn stack_verb_for_id(id: ph2d_editor_core::NodeId) -> Option<StackVerb> {
    if id == ph2d_panel_vector::ids::VECTOR_PAINT_ADD_FILL {
        return Some(StackVerb::AddFill);
    }
    if id == ph2d_panel_vector::ids::VECTOR_PAINT_ADD_STROKE {
        return Some(StackVerb::AddStroke);
    }
    if id == ph2d_panel_vector::ids::VECTOR_PAINT_MOVE {
        return Some(StackVerb::Move);
    }
    (0..MAX_PAINT_LAYERS).find_map(|i| {
        if id == ph2d_panel_vector::ids::vector_paint_eye_id(i) {
            Some(StackVerb::Eye(i))
        } else if id == ph2d_panel_vector::ids::vector_paint_up_id(i) {
            Some(StackVerb::Up(i))
        } else if id == ph2d_panel_vector::ids::vector_paint_down_id(i) {
            Some(StackVerb::Down(i))
        } else if id == ph2d_panel_vector::ids::vector_paint_del_id(i) {
            Some(StackVerb::Del(i))
        } else {
            None
        }
    })
}

/// **Que QUINA de offset este id pede** (`None` se não é um dos três chips).
///
/// ⚠️ Os códigos são os da casa (`0` Miter · `1` Round · `2` Bevel), resolvidos pela MESMA porta
/// (`vec_expand::join_of_code`) que o Contour e o Expand usam — uma segunda tabela divergiria na
/// primeira quina nova.
pub fn join_code_for_id(id: ph2d_editor_core::NodeId) -> Option<u8> {
    match id {
        _ if id == ph2d_panel_vector::ids::VECTOR_PAINT_JOIN_MITER => Some(0),
        _ if id == ph2d_panel_vector::ids::VECTOR_PAINT_JOIN_ROUND => Some(1),
        _ if id == ph2d_panel_vector::ids::VECTOR_PAINT_JOIN_BEVEL => Some(2),
        // ⭐ F44b: a Position do traço da camada aberta viaja no MESMO canal (códigos `3..=5`,
        // [`ALIGN_CODE`]); quem os separa é o [`set_join_or_align`].
        _ if id == ph2d_panel_vector::ids::VECTOR_PAINT_ALIGN_INNER => Some(ALIGN_CODE),
        _ if id == ph2d_panel_vector::ids::VECTOR_PAINT_ALIGN_CENTRE => Some(ALIGN_CODE + 1),
        _ if id == ph2d_panel_vector::ids::VECTOR_PAINT_ALIGN_OUTER => Some(ALIGN_CODE + 2),
        _ => None,
    }
}

/// **De que CAMADA é a swatch que o picker está a editar** (`None` se ele não está numa).
///
/// ⚠️ Varre o espaço FIXO de ids, como o [`stack_verb_for_id`] — e pela mesma razão.
pub fn layer_of_picker_target(store: &ph2d_editor_core::interaction::WidgetStore) -> Option<usize> {
    let alvo = store.picker_target()?;
    (0..MAX_PAINT_LAYERS).find(|&i| ph2d_panel_vector::ids::vector_paint_swatch_id(i) == alvo)
}

/// **Aplica o verbo.** Devolve se o DOCUMENTO mudou (a vista não conta — ela não entra no undo).
///
/// ⚠️ **Um gesto que muda a PILHA fecha a camada aberta**: o índice guardado é da lista de ANTES, e
/// sobreviver a ela faria o painel mostrar as propriedades de outra camada.
pub fn apply(scene: &mut VecScene, sel: &[VecPathId], verb: StackVerb) -> bool {
    match verb {
        StackVerb::AddFill => {
            ph2d_panel_vector::state::close_open_layer();
            add(scene, sel, true)
        }
        StackVerb::AddStroke => {
            ph2d_panel_vector::state::close_open_layer();
            add(scene, sel, false)
        }
        StackVerb::Eye(i) => mixed::toggle(scene, sel, i),
        StackVerb::Up(i) => {
            ph2d_panel_vector::state::close_open_layer();
            shift(scene, sel, i, true)
        }
        StackVerb::Down(i) => {
            ph2d_panel_vector::state::close_open_layer();
            shift(scene, sel, i, false)
        }
        StackVerb::Del(i) => {
            ph2d_panel_vector::state::close_open_layer();
            remove(scene, sel, i)
        }
        StackVerb::Move => {
            ph2d_panel_vector::state::close_open_layer();
            ph2d_panel_vector::state::take_paint_move()
                .is_some_and(|(it, onde)| move_item(scene, sel, it, onde))
        }
    }
}

/// A cor com que uma camada nova nasce.
///
/// ⚠️ **Não é transparente nem branca**: uma camada nova tem de SER VISÍVEL, senão o artista
/// carrega no botão, nada muda no ecrã, e conclui que ele está morto. Um cinzento médio opaco
/// lê-se sobre qualquer fundo e sobre a maior parte das tintas.
const NOVA: Rgba8 = Rgba8 {
    r: 128,
    g: 128,
    b: 128,
    a: 255,
};

/// A largura com que um contorno novo nasce, em unidades de mundo.
///
/// ⚠️ **É a mesma do `StrokeSpec` de fábrica desta casa** — um contorno novo que nascesse mais fino
/// que o de base leria como um defeito de desenho, e um mais gordo esconderia o desenho.
const LARGURA_NOVA: f64 = 1.0;

/// **O que o painel mostra** — `None` sem forma na selecção. `sel` vem de [`ordered`]: o primeiro
/// é o primário.
///
/// ⭐ `vivo` = a camada viva do quadro ([`crate::driven_style::live_layers`]): a Opacity é a que a
/// TELA desenha (`object_alpha`), não a gravada — numa forma que a linha do tempo conduz as duas
/// diferem (plano 34 D1).
pub fn published(
    scene: &VecScene,
    sel: &[VecPathId],
    vivo: &ph2d_vec_scene::VecViewState,
    doc: ph2d_editor_core::DocPx,
) -> Option<ph2d_panel_vector::state::Appearance> {
    let p = scene.path(*sel.first()?)?;
    let layers_differ = layers_differ(scene, sel);
    let outras = || sel.iter().filter_map(|id| scene.path(*id));
    let alfa =
        |q: &ph2d_vec_scene::VecPath| ph2d_vec_scene::object_alpha(q, vivo.bound_style(q.id));
    let opacity = alfa(p);
    Some(ph2d_panel_vector::state::Appearance {
        opacity,
        blend: p.blend,
        mixed_opacity: outras().any(|o| alfa(o) != opacity),
        mixed_blend: outras().any(|o| o.blend != p.blend),
        layers_differ,
        order: visible_order(p),
        layers: p
            .paints
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let c = e.swatch_color();
                ph2d_panel_vector::state::PaintRow {
                    is_fill: matches!(e.kind, PaintKind::Fill(_)),
                    color: [c.r, c.g, c.b, c.a],
                    // Em px do documento, como o Weight do traço de base (F44c: a linha mostrava
                    // metros — `0` num traço de 5 px).
                    width: doc.of_world(e.width().unwrap_or(0.0)),
                    align: match &e.kind {
                        PaintKind::Stroke(sp) => sp.align,
                        PaintKind::Fill(_) => StrokeAlign::default(),
                    },
                    ramp: ramp_of(&e.kind),
                    enabled: e.enabled,
                    opacity: e.opacity.get(),
                    blend: e.blend,
                    offset: e.offset,
                    dilate: e.dilate,
                    dilate_join: e.dilate_join,
                    mixed: if layers_differ {
                        ph2d_panel_vector::state::RowMixed::default()
                    } else {
                        row_mixed(scene, sel, i)
                    },
                }
            })
            .collect(),
    })
}

/// As pontas do degradê de uma camada de preenchimento (`None` = não é um degradê de paragens).
fn ramp_of(k: &PaintKind) -> Option<[[u8; 4]; 2]> {
    let PaintKind::Fill(Paint::Linear { stops, .. } | Paint::Radial { stops, .. }) = k else {
        return None;
    };
    let c = |s: &ph2d_vec_scene::GradientStop| [s.color.r, s.color.g, s.color.b, s.color.a];
    Some([c(stops.first()?), c(stops.last()?)])
}

/// **Acrescenta uma camada no TOPO** de cada forma seleccionada. Devolve se alguma mudou.
///
/// ⛔ Uma forma no tecto ([`MAX_PAINT_LAYERS`]) é saltada — e o painel já esconde os botões nesse
/// caso, então isto é a segunda metade da mesma recusa, do lado que de facto escreve.
pub fn add(scene: &mut VecScene, sel: &[VecPathId], fill: bool) -> bool {
    let mut mudou = false;
    for id in sel {
        if let Some(p) = scene.path_mut(*id)
            && p.paints.len() < MAX_PAINT_LAYERS
        {
            p.paints.push(if fill {
                PaintEntry::fill(Paint::Solid(NOVA))
            } else {
                PaintEntry::stroke(StrokeSpec::new(NOVA, LARGURA_NOVA))
            });
            mudou = true;
        }
    }
    mudou
}

/// **Apaga a camada `i`** de cada forma seleccionada que a tenha; a base fica entre as mesmas vizinhas.
pub fn remove(scene: &mut VecScene, sel: &[VecPathId], i: usize) -> bool {
    edit(scene, sel, |p| p.remove_paint(i))
}

/// **Move a camada `i` uma linha** na pilha INTEIRA à vista (`up` = para o topo) — a vizinha pode
/// ser a tinta de base (F45).
pub fn shift(scene: &mut VecScene, sel: &[VecPathId], i: usize, up: bool) -> bool {
    edit(scene, sel, |p| {
        let vistas = visible_order(p);
        let eu = StackItem::Entry(i);
        let Some(k) = vistas.iter().position(|x| *x == eu) else {
            return false;
        };
        let viz = if up {
            k.checked_add(1)
        } else {
            k.checked_sub(1)
        };
        match viz.and_then(|v| vistas.get(v)) {
            Some(&v) if up => p.move_paint(eu, Place::Above(v)),
            Some(&v) => p.move_paint(eu, Place::Below(v)),
            None => false,
        }
    })
}

/// ⭐ **Move a linha `it` para logo acima/abaixo de outra** em cada forma seleccionada — o arrasto
/// pelo ⚙ (F45). Uma forma sem uma das duas linhas é saltada.
pub fn move_item(scene: &mut VecScene, sel: &[VecPathId], it: StackItem, onde: Place) -> bool {
    edit(scene, sel, |p| p.move_paint(it, onde))
}

/// As linhas que o painel mostra, de baixo para cima: a pilha inteira sem a base ausente.
#[must_use]
pub fn visible_order(p: &ph2d_vec_scene::VecPath) -> Vec<StackItem> {
    p.draw_order().filter(|it| p.has_item(*it)).collect()
}

/// **A opacidade** da camada `i`.
pub fn set_opacity(scene: &mut VecScene, sel: &[VecPathId], i: usize, v: f32) -> bool {
    let novo = ph2d_vec_scene::Opacity::new(v);
    edit(scene, sel, |p| match p.paints.get_mut(i) {
        Some(e) if e.opacity != novo => {
            e.opacity = novo;
            true
        }
        _ => false,
    })
}

/// **O modo de mistura** da camada `i`. O que chega do painel é o CÓDIGO do modo.
pub fn set_blend(scene: &mut VecScene, sel: &[VecPathId], i: usize, code: u8) -> bool {
    let novo = ph2d_vec_scene::BlendMode::from_u8(code);
    edit(scene, sel, |p| match p.paints.get_mut(i) {
        Some(e) if e.blend != novo => {
            e.blend = novo;
            true
        }
        _ => false,
    })
}

/// **A largura** do contorno da camada `i`. No-op numa camada de preenchimento.
/// `px` em px do documento (o Weight da linha), gravado em metros de mundo.
pub fn set_width(
    scene: &mut VecScene,
    sel: &[VecPathId],
    i: usize,
    px: f64,
    doc: ph2d_editor_core::DocPx,
) -> bool {
    let w = doc.to_world(px.max(0.0));
    edit(scene, sel, |p| {
        match p.paints.get_mut(i).map(|e| &mut e.kind) {
            Some(PaintKind::Stroke(s)) if (s.width - w).abs() > f64::EPSILON => {
                s.width = w;
                true
            }
            _ => false,
        }
    })
}

/// **ONDE a camada `i` desenha** (v21) — em unidades de mundo, relativo à forma.
///
/// ⚠️ Vale nas DUAS espécies: a sombra dura de um PREENCHIMENTO é o caso que motivou isto.
pub fn set_offset(scene: &mut VecScene, sel: &[VecPathId], i: usize, o: [f64; 2]) -> bool {
    edit(scene, sel, |p| match p.paints.get_mut(i) {
        Some(e) if e.offset != o => {
            e.offset = o;
            true
        }
        _ => false,
    })
}

/// **O OFFSET DE CAD da camada `i`** (v22) — a silhueta cresce (`>0`) ou encolhe (`<0`).
pub fn set_dilate(scene: &mut VecScene, sel: &[VecPathId], i: usize, d: f64) -> bool {
    edit(scene, sel, |p| match p.paints.get_mut(i) {
        Some(e) if e.dilate != d => {
            e.dilate = d;
            true
        }
        _ => false,
    })
}

/// **A QUINA desse offset** (`0` Miter · `1` Round · `2` Bevel).
pub fn set_dilate_join(scene: &mut VecScene, sel: &[VecPathId], i: usize, j: u8) -> bool {
    edit(scene, sel, |p| match p.paints.get_mut(i) {
        Some(e) if e.dilate_join != j => {
            e.dilate_join = j;
            true
        }
        _ => false,
    })
}

/// **A cor** da camada `i` — a sólida de um preenchimento, ou a do contorno.
///
/// ⛔ Numa camada cuja tinta é um gradiente ou um padrão, isto **substitui-a por uma cor sólida**:
/// a swatch da linha mostra UMA cor, e escrever uma cor onde a swatch mostra uma cor é o que ela
/// promete. Um gradiente numa camada edita-se onde os gradientes se editam.
pub fn set_color(scene: &mut VecScene, sel: &[VecPathId], i: usize, c: Rgba8) -> bool {
    edit(scene, sel, |p| {
        match p.paints.get_mut(i).map(|e| &mut e.kind) {
            Some(PaintKind::Fill(f)) => {
                *f = Paint::Solid(c);
                true
            }
            Some(PaintKind::Stroke(s)) => {
                s.paint = ph2d_vec_scene::StrokePaint::Solid(c);
                true
            }
            None => false,
        }
    })
}

/// O 1.º código da Position no canal do [`join_code_for_id`] (`3` Inside · `4` Center · `5` Outside).
pub const ALIGN_CODE: u8 = 3;

/// **A linha que o painel tem ABERTA** — a base, ou a camada cujo ⚙ está aberto. É o endereço que
/// o corpo de ajustes (tipo, degradê, padrão, pontas, junções, tracejado, pincel) edita e mostra.
#[must_use]
pub fn open_slot() -> PaintSlot {
    ph2d_panel_vector::state::open_layer_index().map_or(PaintSlot::Base, PaintSlot::Layer)
}

/// O preenchimento da linha aberta ([`open_slot`]) de `p`.
#[must_use]
pub fn open_fill(p: &ph2d_vec_scene::VecPath) -> Option<&Paint> {
    p.slot_fill(open_slot())
}

pub fn open_fill_mut(p: &mut ph2d_vec_scene::VecPath) -> Option<&mut Paint> {
    p.slot_fill_mut(open_slot())
}

/// O gizmo de degradê no canvas sobre a linha ABERTA ([`open_slot`]) — o mesmo endereço que o
/// corpo de ajustes edita (smoke do dono 01/11: o ⚙ de uma camada aberto e a alça editava a base).
#[must_use]
pub fn grad_hit(p: &ph2d_vec_scene::VecPath, l: [f64; 2], t: f64) -> Option<GradHandle> {
    ph2d_vec_render::hit_gradient_handle(p, open_slot(), l[0], l[1], t)
}

/// O centro de `sel` no espaço do degradê da linha aberta: o da caixa, menos o deslocamento da
/// camada (ela desenha deslocada — um ponto novo no centro nasce sobre a forma).
#[must_use]
pub fn open_centre(scene: &ph2d_vec_scene::VecScene, sel: VecPathId) -> Option<[f64; 2]> {
    let (lo, hi) = scene.path_bbox(sel)?;
    let o = match open_slot() {
        PaintSlot::Layer(i) => scene.path(sel)?.paints.get(i)?.offset,
        PaintSlot::Base => [0.0, 0.0],
    };
    Some([(lo[0] + hi[0]) * 0.5 - o[0], (lo[1] + hi[1]) * 0.5 - o[1]])
}

pub fn grad_drag(p: &mut ph2d_vec_scene::VecPath, h: GradHandle, x: f64, y: f64) -> bool {
    ph2d_vec_render::drag_gradient_handle(p, open_slot(), h, x, y)
}

/// **Põe `paint` no preenchimento da linha aberta** ([`open_slot`]): a base (vista ou escondida;
/// sem nenhuma, nasce à vista) ou a camada aberta, se ela é um preenchimento — nunca a cria.
pub fn put_slot_fill(p: &mut ph2d_vec_scene::VecPath, paint: Paint) -> bool {
    let slot = open_slot();
    match p.slot_fill_mut(slot) {
        Some(f) => *f = paint,
        None if slot == PaintSlot::Base => p.fill = Some(paint),
        None => return false,
    }
    true
}

/// **A Position (`StrokeSpec::align`) da camada `i`.** No-op numa camada de preenchimento.
pub fn set_align(scene: &mut VecScene, sel: &[VecPathId], i: usize, a: StrokeAlign) -> bool {
    edit(scene, sel, |p| {
        match p.paints.get_mut(i).map(|e| &mut e.kind) {
            Some(PaintKind::Stroke(s)) if s.align != a => {
                s.align = a;
                true
            }
            _ => false,
        }
    })
}

/// O canal dos chips da camada aberta: `0..=2` = a QUINA do offset, [`ALIGN_CODE`]`..` = a Position.
pub fn set_join_or_align(scene: &mut VecScene, sel: &[VecPathId], i: usize, code: u8) -> bool {
    match code.checked_sub(ALIGN_CODE) {
        None => set_dilate_join(scene, sel, i, code),
        Some(0) => set_align(scene, sel, i, StrokeAlign::Inner),
        Some(1) => set_align(scene, sel, i, StrokeAlign::Centre),
        Some(_) => set_align(scene, sel, i, StrokeAlign::Outer),
    }
}

/// O molde das seis acima: aplica `f` a cada forma da selecção e devolve se alguma mudou.
fn edit(
    scene: &mut VecScene,
    sel: &[VecPathId],
    mut f: impl FnMut(&mut ph2d_vec_scene::VecPath) -> bool,
) -> bool {
    let mut mudou = false;
    for id in sel {
        if let Some(p) = scene.path_mut(*id) {
            mudou |= f(p);
        }
    }
    mudou
}

#[path = "paint_stack_mixed.rs"]
mod mixed;
pub use mixed::{apply_picker, layers_differ, ordered, row_mixed, set_offset_axes, toggle};

#[cfg(test)]
#[path = "paint_stack_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "paint_stack_slot_tests.rs"]
mod slot_tests;

#[cfg(test)]
#[path = "paint_order_tests.rs"]
mod order_tests;
