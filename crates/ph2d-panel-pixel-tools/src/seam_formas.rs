//! A costura das FORMAS do Vector no painel PIXEL (P6b, ADR-0192 §12): os campos de cada forma são
//! os do catálogo do Vector ([`ph2d_tool_vector::shapes::SHAPES`]) — o mesmo rótulo, a mesma faixa,
//! a mesma unidade (os píxeis do Vector são células aqui) — e cada um escreve o seu valor nas
//! opções (filho de [`crate::seam`]).

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::pixel_panel::{PixelOptions, PixelShape};
use ph2d_editor_core::widget::{ButtonState, CheckboxState, CheckboxValue, TextInputState};
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};
use ph2d_tool_vector::shapes::{FieldDesc, FieldUnit, SHAPES, ShapeDesc};

/// A tradução dos rótulos do catálogo do Vector: a PONTE do painel do Vector
/// (`ph2d_panel_vector::nomes_do_motor::campo`), injectada pelo registo dos painéis — um painel não
/// depende de outro, e uma 2.ª tabela de nomes divergiria da dele.
static NOMES: std::sync::OnceLock<fn(&'static str) -> &'static str> = std::sync::OnceLock::new();

/// Injecta a ponte dos nomes (o `ph2d-panel-registry-init`, que vê os dois painéis).
pub fn set_field_names(f: fn(&'static str) -> &'static str) {
    let _ = NOMES.set(f);
    ph2d_editor_core::pixel_panel::set_field_names(nome);
}

/// A tradução dos NOMES das formas do catálogo (`ph2d_panel_vector::nomes_do_motor::forma`), pela
/// mesma razão que [`NOMES`].
static FORMAS: std::sync::OnceLock<fn(&'static str) -> &'static str> = std::sync::OnceLock::new();

/// Injecta a ponte dos nomes das formas: o núcleo do Pixel passa a nomear cada forma pelo Vector.
pub fn set_shape_names(f: fn(&'static str) -> &'static str) {
    let _ = FORMAS.set(f);
    ph2d_editor_core::pixel_panel::set_shape_names(shape_name);
}

/// O nome da forma `k` como o flyout do Vector o pinta (sem ponte, o rótulo do motor).
#[must_use]
pub fn shape_name(k: ph2d_vec_scene::ShapeKind) -> &'static str {
    let rotulo = SHAPES.iter().find(|d| d.kind == k).map_or("", |d| d.label);
    FORMAS.get().map_or(rotulo, |f| f(rotulo))
}

/// O nome a pintar de um rótulo do catálogo (sem ponte, o rótulo do motor).
#[must_use]
pub fn nome(rotulo: &'static str) -> &'static str {
    NOMES.get().map_or(rotulo, |f| f(rotulo))
}

/// A caixa «Fill» das formas fechadas.
pub const SHAPE_FILL: NodeId = hash_node_id("pixel.opt.shape_fill");

/// A linha do catálogo do Vector da forma.
#[must_use]
pub fn desc(s: PixelShape) -> &'static ShapeDesc {
    let k = s.kind();
    SHAPES
        .iter()
        .find(|d| d.kind == k)
        .expect("toda forma do Vector está no catálogo dele")
}

/// O campo `i` da forma `s` (número), ou a opção `j` dele (uma escolha).
#[must_use]
pub fn field_id(s: PixelShape, i: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.shape.{}.{i}", s.index()))
}

#[must_use]
pub fn option_id(s: PixelShape, i: usize, j: usize) -> NodeId {
    hash_node_id_runtime(&format!("pixel.shape.{}.{i}.{j}", s.index()))
}

/// Todo campo de toda forma: `(forma, índice, descritor)`.
pub fn fields() -> impl Iterator<Item = (PixelShape, usize, &'static FieldDesc)> {
    PixelShape::ALL.into_iter().flat_map(|s| {
        desc(s)
            .fields
            .iter()
            .enumerate()
            .map(move |(i, f)| (s, i, f))
    })
}

pub(crate) fn populate(store: &mut WidgetStore) {
    let (state, value) = (CheckboxState::Normal, CheckboxValue::Unchecked);
    store.register(SHAPE_FILL, InteractiveState::Checkbox { state, value });
    let omissao = PixelOptions::default();
    for (s, i, f) in fields() {
        match f.unit {
            FieldUnit::Choice(nomes) => {
                for j in 0..nomes.len() {
                    let state = ButtonState::Normal;
                    store.register(option_id(s, i, j), InteractiveState::Button { state });
                }
            }
            _ => {
                let id = field_id(s, i);
                let value = omissao.shapes.get(s).get(i);
                store.register(
                    id,
                    InteractiveState::NumberInput {
                        state: TextInputState::Normal,
                        value,
                        buffer: value.to_string(),
                        caret: 0,
                        last_committed: value,
                        selection_anchor: None,
                    },
                );
                store.set_number_range(id, f.min, f.max, f.step);
            }
        }
    }
}

/// Os campos numéricos das formas e o valor de cada um nas opções (o painel sincroniza-os).
pub(crate) fn numeros(o: &PixelOptions) -> impl Iterator<Item = (NodeId, f64)> + '_ {
    fields()
        .filter(|(.., f)| !matches!(f.unit, FieldUnit::Choice(_)))
        .map(|(s, i, _)| (field_id(s, i), o.shapes.get(s).get(i)))
}

/// As opções com o campo numérico `id` em `v` (preso à faixa; as contagens inteiras), se é um.
pub(crate) fn com_numero(mut o: PixelOptions, id: NodeId, v: f64) -> Option<PixelOptions> {
    let (s, i, f) = fields().find(|(s, i, _)| field_id(*s, *i) == id)?;
    if !v.is_finite() {
        return Some(o);
    }
    let v = ph2d_editor_core::math::safe_clamp_f64(v, f.min, f.max);
    let v = if f.unit == FieldUnit::Count {
        v.round()
    } else {
        v
    };
    o.shapes.set(s, o.shapes.get(s).with(i, v));
    Some(o)
}

/// As opções com a opção `id` escolhida, se é uma.
pub(crate) fn com_opcao(mut o: PixelOptions, id: NodeId) -> Option<PixelOptions> {
    for (s, i, f) in fields() {
        if let FieldUnit::Choice(nomes) = f.unit
            && let Some(j) = (0..nomes.len()).find(|&j| option_id(s, i, j) == id)
        {
            #[expect(clippy::cast_precision_loss, reason = "índice de uma escolha (≤ 4)")]
            let j = j as f64;
            o.shapes.set(s, o.shapes.get(s).with(i, j));
            return Some(o);
        }
    }
    None
}

/// É um campo numérico de forma.
#[must_use]
pub fn is_field(id: NodeId) -> bool {
    fields().any(|(s, i, f)| !matches!(f.unit, FieldUnit::Choice(_)) && field_id(s, i) == id)
}
