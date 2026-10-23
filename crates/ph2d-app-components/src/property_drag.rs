//! ⭐ **Uma linha do Inspector arrastada para o código** (8b do CodeEditor, doc 07 §3.10): o nome com
//! que um script a lê — `"health.max"`, a pose `"x", "y"`; uma propriedade declarada pelo script do
//! objecto, `self.<nome>` no texto desse script.
//!
//! O Inspector grava o texto de cada linha e o cartão dela ([`arrasto`]); aqui o texto volta a ser o
//! nome pelo MESMO catálogo que pinta o rótulo (`FieldDesc.label_key`) e que o `ph2d.get` lê
//! (`FieldDesc.name`). Duas linhas com o mesmo texto no mesmo objecto decidem-se pelo título do cartão;
//! se nem assim, a linha não arrasta (nunca um nome à sorte).

use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::scene::property::{self, full_name};
use ph2d_ecs::scene::property_ref::RefNames;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::interaction::drag_payload::DragPayload;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::widget::arrasto::{self, Linha};
use ph2d_i18n::tr;

/// Os nomes que o rótulo de um arrasto de linha leva (o fantasma mostra-os assim).
#[must_use]
pub fn names_of_label(label: &str) -> Vec<String> {
    label.split(", ").map(str::to_owned).collect()
}

/// A pose não está no catálogo: o cartão Transform pinta pares, o script lê cada eixo.
fn pose(texto: &str) -> Option<&'static [&'static str]> {
    let t = |k| tr(k) == texto;
    Some(
        if t("panel.inspector.transform.position_m") || t("panel.inspector.transform.position_px") {
            &["x", "y"]
        } else if t("panel.inspector.transform.rotation") {
            &["rotation"]
        } else if t("panel.inspector.transform.scale") {
            &["scale_x", "scale_y"]
        } else if t("panel.inspector.transform.skew") {
            &["skew_x", "skew_y"]
        } else {
            return None;
        },
    )
}

/// ⭐ **A LEI:** a carga e o rótulo do arrasto de `linha`, pintada no Inspector de `entity`.
#[must_use]
pub fn carga(
    reg: &ComponentRegistry,
    sim: &SimWorld,
    entity: Entity,
    linha: &Linha,
) -> Option<(DragPayload, String)> {
    if let Some(script) = &linha.script {
        let lib = sim.world().get_resource::<ph2d_script::ProjectScripts>()?;
        let id = lib.0.by_name(script)?;
        return Some((DragPayload::Property { script: id.0 }, linha.texto.clone()));
    }
    if linha.cartao == tr("panel.inspector.transform.transform") {
        let names = pose(&linha.texto)?;
        return Some((DragPayload::Property { script: 0 }, names.join(", ")));
    }
    let world = sim.world();
    let mut achados: Vec<(&str, String)> = ph2d_component_desc::all()
        .filter(|d| !d.code.is_empty())
        .flat_map(|d| {
            d.fields
                .iter()
                .filter(|f| !f.name.is_empty() && tr(f.label_key) == linha.texto)
                .map(move |f| (tr(d.display_key), full_name(d.code, f.name)))
        })
        .filter(|(_, n)| property::get(reg, world, entity, n, RefNames::default()).is_ok())
        .collect();
    if achados.len() > 1 {
        achados.retain(|(cartao, _)| *cartao == linha.cartao);
    }
    match achados.as_slice() {
        [(_, nome)] => Some((DragPayload::Property { script: 0 }, nome.clone())),
        _ => None,
    }
}

/// O `Down` sobre o nome de uma linha do Inspector semeia o arrasto (ainda clique até ao limiar).
pub fn seed(
    hero: &mut HeroScreen,
    sim: &SimWorld,
    reg: &ComponentRegistry,
    at: (f32, f32),
) -> bool {
    if hero.chrome_panel_at(at.0, at.1) != Some(ph2d_editor_core::ids::INSP_PANEL) {
        return false;
    }
    let (x, y) = hero.escala().to_logical(at);
    let carga = arrasto::sob(x, y).and_then(|linha| {
        let e = Entity::try_from_bits(hero.gizmo.selection?)?;
        carga(reg, sim, e, &linha)
    });
    let Some((payload, label)) = carga else {
        return false;
    };
    hero.store.begin_named_drag(payload, x, y, label);
    true
}

#[cfg(test)]
#[path = "property_drag_tests.rs"]
mod tests;
