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

use ph2d_vec_scene::{
    MAX_PAINT_LAYERS, Paint, PaintEntry, PaintKind, Rgba8, StrokeSpec, VecPathId, VecScene,
};

/// ⭐⭐⭐ **O QUE UM CLIQUE NA PILHA PEDE** — a metade PURA da ponte, testável sem `App`.
///
/// ⚠️ **A resolução varre o espaço FIXO de ids** ([`MAX_PAINT_LAYERS`]), e não a pilha de hoje: um
/// id de runtime nasce de um índice, e perguntar «que índice é este?» a uma lista que muda de
/// tamanho faria o mesmo clique resolver diferente conforme a forma seleccionada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StackVerb {
    AddFill,
    AddStroke,
    /// Liga/desliga o olho da camada `i`.
    Eye(usize),
    /// Sobe a camada `i` (para o topo da pilha).
    Up(usize),
    Down(usize),
    Del(usize),
    /// Abre/fecha as propriedades da camada `i` — **estado de VISTA**, nunca documento.
    Open(usize),
}

/// Que verbo este id pede (`None` se ele não é da pilha).
pub(crate) fn stack_verb_for_id(id: ph2d_editor::NodeId) -> Option<StackVerb> {
    use ph2d_editor::ids;
    if id == ids::VECTOR_PAINT_ADD_FILL {
        return Some(StackVerb::AddFill);
    }
    if id == ids::VECTOR_PAINT_ADD_STROKE {
        return Some(StackVerb::AddStroke);
    }
    (0..MAX_PAINT_LAYERS).find_map(|i| {
        if id == ids::vector_paint_eye_id(i) {
            Some(StackVerb::Eye(i))
        } else if id == ids::vector_paint_up_id(i) {
            Some(StackVerb::Up(i))
        } else if id == ids::vector_paint_down_id(i) {
            Some(StackVerb::Down(i))
        } else if id == ids::vector_paint_del_id(i) {
            Some(StackVerb::Del(i))
        } else if id == ids::vector_paint_row_id(i) {
            Some(StackVerb::Open(i))
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
pub(crate) fn join_code_for_id(id: ph2d_editor::NodeId) -> Option<u8> {
    use ph2d_editor::ids;
    match id {
        _ if id == ids::VECTOR_PAINT_JOIN_MITER => Some(0),
        _ if id == ids::VECTOR_PAINT_JOIN_ROUND => Some(1),
        _ if id == ids::VECTOR_PAINT_JOIN_BEVEL => Some(2),
        _ => None,
    }
}

/// **De que CAMADA é a swatch que o picker está a editar** (`None` se ele não está numa).
///
/// ⚠️ Varre o espaço FIXO de ids, como o [`stack_verb_for_id`] — e pela mesma razão.
pub(crate) fn layer_of_picker_target(
    store: &ph2d_editor::interaction::WidgetStore,
) -> Option<usize> {
    let alvo = store.picker_target()?;
    (0..MAX_PAINT_LAYERS).find(|&i| ph2d_editor::ids::vector_paint_swatch_id(i) == alvo)
}

/// **Aplica o verbo.** Devolve se o DOCUMENTO mudou (a vista não conta — ela não entra no undo).
///
/// ⚠️ **Um gesto que muda a PILHA fecha a camada aberta**: o índice guardado é da lista de ANTES, e
/// sobreviver a ela faria o painel mostrar as propriedades de outra camada.
pub(crate) fn apply(scene: &mut VecScene, sel: &[VecPathId], verb: StackVerb) -> bool {
    match verb {
        StackVerb::AddFill => {
            ph2d_panel_vector::state::close_open_layer();
            add(scene, sel, true)
        }
        StackVerb::AddStroke => {
            ph2d_panel_vector::state::close_open_layer();
            add(scene, sel, false)
        }
        StackVerb::Eye(i) => toggle(scene, sel, i),
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
        // ⛔ Este é de VISTA: abrir uma linha não muda o documento, e devolver `true` aqui poria um
        // passo de undo por clique de UI — o defeito que o `post_frame_undo` desta casa mede por
        // DIFF justamente para não ter.
        //
        // ⛔⛔ **E a SWATCH não é um verbo desta lista, o que é uma decisão e não um esquecimento:**
        // ela é uma *picker swatch* (`register_picker_swatch`), e o `pointer_down` do editor-core
        // curto-circuita o Down dessas para abrir o picker partilhado — elas **nunca** emitem
        // `Click`. Existiu aqui um `StackVerb::Swatch` que abria a camada, e ele era **inalcançável
        // por construção**: nenhum produtor o podia emitir, e um teste que o chamava à mão deixava-o
        // com cara de vivo. Quem lê a escolha de cor é a shell, pelo `layer_of_picker_target`.
        StackVerb::Open(i) => {
            ph2d_panel_vector::state::toggle_open_layer(i);
            false
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

/// **O que o painel mostra** — `None` sem forma na selecção.
pub(crate) fn published(
    scene: &VecScene,
    sel: &[VecPathId],
) -> Option<ph2d_panel_vector::state::Appearance> {
    let p = scene.path(*sel.first()?)?;
    Some(ph2d_panel_vector::state::Appearance {
        opacity: p.opacity.get(),
        blend: p.blend,
        layers: p
            .paints
            .iter()
            .map(|e| {
                let c = e.swatch_color();
                ph2d_panel_vector::state::PaintRow {
                    is_fill: matches!(e.kind, PaintKind::Fill(_)),
                    color: [c.r, c.g, c.b, c.a],
                    width: e.width().unwrap_or(0.0),
                    enabled: e.enabled,
                    opacity: e.opacity.get(),
                    blend: e.blend,
                    offset: e.offset,
                    dilate: e.dilate,
                    dilate_join: e.dilate_join,
                }
            })
            .collect(),
    })
}

/// **Acrescenta uma camada no TOPO** de cada forma seleccionada. Devolve se alguma mudou.
///
/// ⛔ Uma forma no tecto ([`MAX_PAINT_LAYERS`]) é saltada — e o painel já esconde os botões nesse
/// caso, então isto é a segunda metade da mesma recusa, do lado que de facto escreve.
pub(crate) fn add(scene: &mut VecScene, sel: &[VecPathId], fill: bool) -> bool {
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

/// **Apaga a camada `i`** de cada forma seleccionada que a tenha.
pub(crate) fn remove(scene: &mut VecScene, sel: &[VecPathId], i: usize) -> bool {
    edit(scene, sel, |p| {
        if i < p.paints.len() {
            p.paints.remove(i);
            true
        } else {
            false
        }
    })
}

/// **Move a camada `i` uma posição** (`up` = para o topo da pilha, que é o fim do vector).
pub(crate) fn shift(scene: &mut VecScene, sel: &[VecPathId], i: usize, up: bool) -> bool {
    edit(scene, sel, |p| {
        let n = p.paints.len();
        let j = if up {
            i.checked_add(1)
        } else {
            i.checked_sub(1)
        };
        match j {
            Some(j) if i < n && j < n => {
                p.paints.swap(i, j);
                true
            }
            _ => false,
        }
    })
}

/// **Liga/desliga o olho** da camada `i`.
pub(crate) fn toggle(scene: &mut VecScene, sel: &[VecPathId], i: usize) -> bool {
    edit(scene, sel, |p| match p.paints.get_mut(i) {
        Some(e) => {
            e.enabled = !e.enabled;
            true
        }
        None => false,
    })
}

/// **A opacidade** da camada `i`.
pub(crate) fn set_opacity(scene: &mut VecScene, sel: &[VecPathId], i: usize, v: f32) -> bool {
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
pub(crate) fn set_blend(scene: &mut VecScene, sel: &[VecPathId], i: usize, code: u8) -> bool {
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
pub(crate) fn set_width(scene: &mut VecScene, sel: &[VecPathId], i: usize, w: f64) -> bool {
    let w = w.max(0.0);
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
pub(crate) fn set_offset(scene: &mut VecScene, sel: &[VecPathId], i: usize, o: [f64; 2]) -> bool {
    edit(scene, sel, |p| match p.paints.get_mut(i) {
        Some(e) if e.offset != o => {
            e.offset = o;
            true
        }
        _ => false,
    })
}

/// **O OFFSET DE CAD da camada `i`** (v22) — a silhueta cresce (`>0`) ou encolhe (`<0`).
pub(crate) fn set_dilate(scene: &mut VecScene, sel: &[VecPathId], i: usize, d: f64) -> bool {
    edit(scene, sel, |p| match p.paints.get_mut(i) {
        Some(e) if e.dilate != d => {
            e.dilate = d;
            true
        }
        _ => false,
    })
}

/// **A QUINA desse offset** (`0` Miter · `1` Round · `2` Bevel).
pub(crate) fn set_dilate_join(scene: &mut VecScene, sel: &[VecPathId], i: usize, j: u8) -> bool {
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
pub(crate) fn set_color(scene: &mut VecScene, sel: &[VecPathId], i: usize, c: Rgba8) -> bool {
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

#[cfg(test)]
#[path = "vec_paint_stack_tests.rs"]
mod tests;
