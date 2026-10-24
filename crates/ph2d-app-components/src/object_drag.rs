//! ⭐ **Um objecto da Hierarquia como carga entre painéis** (item 8 do CodeEditor,
//! `docs/CodeEditor/07_arrastar_para_o_codigo.md`): a carga e o nome que o fantasma diz, e — na queda —
//! os nomes que a aba de script recebe. A shell só chama; a decisão vive aqui.

use crate::asset_drop::DropAction;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, Name, SimWorld, StableId, World};
use ph2d_editor_core::doc_surface::{DropAt, Dropping};
use ph2d_editor_core::interaction::drag_payload::DragPayload;
use ph2d_editor_core::screens::hero::{HeroScreen, surface_input};
use ph2d_host::Modifiers;
use std::collections::BTreeMap;

/// A carga de arrastar o objecto `bits` e o nome dele (o fantasma diz-o); `None` = sem `StableId`.
#[must_use]
pub fn payload_of(world: &World, bits: u64) -> Option<(DragPayload, String)> {
    let e = Entity::try_from_bits(bits)?;
    let stable_id = world.get::<StableId>(e)?.0;
    let name = world.get::<Name>(e).map(|n| n.as_str().to_owned())?;
    Some((DragPayload::Object { stable_id }, name))
}

/// ⭐ **Os nomes que a queda escreve**: os da SELECÇÃO se o objecto arrastado está nela (o Godot
/// arrasta a selecção inteira), senão o dele — pela ordem da lista de objectos. Sem nome, fica de fora.
#[must_use]
pub fn dropped_names(world: &mut World, stable_id: u64, selection: &[u64]) -> Vec<String> {
    let order = ph2d_ecs::list_order(world);
    let Some(dragged) = order
        .iter()
        .copied()
        .find(|&e| world.get::<StableId>(e).is_some_and(|s| s.0 == stable_id))
    else {
        return Vec::new();
    };
    let take: Vec<Entity> = if selection.contains(&dragged.to_bits()) {
        order
            .into_iter()
            .filter(|e| selection.contains(&e.to_bits()))
            .collect()
    } else {
        vec![dragged]
    };
    take.into_iter()
        .filter_map(|e| world.get::<Name>(e).map(|n| n.as_str().to_owned()))
        .collect()
}

/// `Down` numa linha da Hierarquia (ponto FÍSICO): semeia a carga entre painéis (ainda não armada —
/// dentro do painel é o reparentar dele, e o fantasma só aparece fora).
pub fn seed(hero: &mut HeroScreen, world: &World, row_bits: Option<u64>, at: (f32, f32)) {
    if let Some((payload, name)) = row_bits.and_then(|b| payload_of(world, b)) {
        let (x, y) = hero.escala().to_logical(at);
        hero.store.begin_named_drag(payload, x, y, name);
    }
}

/// ⭐ **Um objecto largado no campo de uma propriedade de objecto** (8c): o nome dele passa a ser o
/// valor da propriedade `prop` do script de `entity_bits` — a mesma edição de escrever o nome.
pub fn into_script_slot(sim: &mut SimWorld, entity_bits: u64, prop: &str, stable_id: u64) -> bool {
    let w = sim.world_mut();
    let Some(nome) = ph2d_ecs::entity_of_stable_id(w, StableId(stable_id))
        .and_then(|e| w.get::<Name>(e))
        .map(|n| n.as_str().to_owned())
    else {
        return false;
    };
    let edit = ph2d_editor_core::script_edits::ScriptFieldEdit::SetObject(prop.to_owned(), nome);
    crate::script_inspector::apply(sim, entity_bits, &edit)
}

/// O mesmo, pela LINHA do cartão Script que o Inspector pintou (o largar na shell).
pub fn into_script_row(sim: &mut SimWorld, entity_bits: u64, row: u16, stable_id: u64) -> bool {
    ph2d_panel_inspector::script_prop_name(entity_bits, row)
        .is_some_and(|prop| into_script_slot(sim, entity_bits, &prop, stable_id))
}

fn selection(hero: &HeroScreen) -> Vec<u64> {
    hero.gizmo
        .selection
        .into_iter()
        .chain(hero.gizmo.extra_selection.iter().copied())
        .collect()
}

/// Onde a queda acha os NOMES de uma carga da biblioteca: o mundo (o mestre, quem usa a imagem) e a
/// biblioteca de imagens ([`crate::library_names`]).
pub struct Libraries<'a> {
    pub sim: &'a mut SimWorld,
    pub db: &'a AssetDb,
    pub atlas: &'a BTreeMap<u32, AssetId>,
}

/// O que a aba escreve para uma carga: objectos (pela selecção) ou nomes (8b).
enum Cargo {
    Objects(Vec<String>),
    Names(Vec<String>),
    Property(Vec<String>, Option<u64>),
}

/// ⭐ **Os nomes de uma carga**: um objecto e a selecção dele; um mestre pelo `Name` dele (o que o
/// `ph2d.spawn` recebe); uma imagem pelo nome da biblioteca (o que o `sprite.image` recebe); uma
/// linha do Inspector pelo rótulo do arrasto.
fn cargo_of(
    payload: DragPayload,
    label: Option<&str>,
    libs: &mut Libraries<'_>,
    hero: &HeroScreen,
) -> Option<Cargo> {
    match payload {
        DragPayload::Property { script } => Some(Cargo::Property(
            crate::property_drag::names_of_label(label?),
            (script != 0).then_some(script),
        )),
        DragPayload::Object { stable_id } => Some(Cargo::Objects(dropped_names(
            libs.sim.world_mut(),
            stable_id,
            &selection(hero),
        ))),
        _ => library_name(payload, libs).map(|n| Cargo::Names(vec![n])),
    }
}

/// O nome que o script recebe para uma carga da biblioteca: o `Name` do mestre, o nome da imagem.
fn library_name(payload: DragPayload, libs: &mut Libraries<'_>) -> Option<String> {
    match payload {
        DragPayload::Prefab { stable_id } => {
            let w = libs.sim.world_mut();
            let e = ph2d_ecs::entity_of_stable_id(w, StableId(stable_id))?;
            Some(w.get::<Name>(e)?.as_str().to_owned())
        }
        DragPayload::Image { asset } => {
            let names = crate::library_names::session_library(libs.sim, libs.db, libs.atlas);
            names.get(&AssetId::from_digest(asset)).cloned()
        }
        DragPayload::Object { .. } | DragPayload::Property { .. } => None,
    }
}

/// ⭐ **A aba de script debaixo do cursor**, para a lei da queda: `None` = não há aba de família
/// activa ou o cursor não está sobre ela (`over_doc`); `Some(on_text)` = está, e a aba diz se
/// escreveria ali (e mostra onde). Cada chamada sem carga sobre a aba apaga a marca dela.
pub fn hover_code(
    hero: &mut HeroScreen,
    mut libs: Libraries<'_>,
    at: (f32, f32),
    over_doc: bool,
    mods: Modifiers,
) -> Option<bool> {
    let payload = hero
        .store
        .asset_drag()
        .and_then(|d| d.armed.then_some(d.payload));
    let doc = over_doc && hero.documents.surface_active().is_some();
    let label = hero.store.asset_drag_label().map(str::to_owned);
    let cargo = payload
        .filter(|_| doc)
        .and_then(|p| cargo_of(p, label.as_deref(), &mut libs, hero));
    let Some(cargo) = cargo else {
        surface_input::drop_hover(hero, None);
        return doc.then_some(false);
    };
    Some(surface_input::drop_hover(
        hero,
        Some(drop_at(at, &cargo, mods)),
    ))
}

/// A queda [`DropAction::WriteCode`] da lei: a aba de script escreve (um passo de undo). `true` = a
/// queda era esta (a shell não a passa ao `perform_drop`). O ponto é o mesmo que a aba recebe do rato.
pub fn write_code(
    action: DropAction,
    (payload, label): (DragPayload, Option<&str>),
    hero: &mut HeroScreen,
    mut libs: Libraries<'_>,
    at: (f32, f32),
    mods: Modifiers,
) -> bool {
    // Um objecto largado no campo de objecto do cartão Script (8c) também se resolve aqui.
    if let DropAction::SetScriptObject {
        entity_bits,
        row,
        stable_id,
    } = action
    {
        surface_input::drop_hover(hero, None);
        into_script_row(libs.sim, entity_bits, row, stable_id);
        return true;
    }
    let cargo = (action == DropAction::WriteCode)
        .then(|| cargo_of(payload, label, &mut libs, hero))
        .flatten();
    let Some(cargo) = cargo else {
        surface_input::drop_hover(hero, None);
        return false;
    };
    surface_input::drop(hero, drop_at(at, &cargo, mods));
    surface_input::drop_hover(hero, None);
    true
}

fn drop_at(at: (f32, f32), cargo: &Cargo, mods: Modifiers) -> DropAt<'_> {
    DropAt {
        x: at.0,
        y: at.1,
        what: match cargo {
            Cargo::Objects(n) => Dropping::Objects(n),
            Cargo::Names(n) => Dropping::Names(n),
            Cargo::Property(names, script) => Dropping::Property {
                names,
                script: *script,
            },
        },
        mods,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{ChildOf, RootOrder, Transform};

    fn obj(w: &mut World, sid: u64, name: &str, order: u32) -> Entity {
        w.spawn((
            Transform::IDENTITY,
            StableId(sid),
            Name::new(name),
            RootOrder(order),
        ))
        .id()
    }

    #[test]
    fn a_selecao_inteira_se_o_arrastado_esta_nela_pela_ordem_da_lista() {
        let mut w = World::new();
        let b = obj(&mut w, 2, "Placar", 2);
        let a = obj(&mut w, 1, "Porta", 1);
        let c = obj(&mut w, 3, "Fora", 3);
        let filho = w
            .spawn((
                Transform::IDENTITY,
                StableId(4),
                Name::new("Arma"),
                ChildOf(a),
            ))
            .id();
        let sel = [b.to_bits(), filho.to_bits(), a.to_bits()];
        assert_eq!(dropped_names(&mut w, 2, &sel), ["Porta", "Arma", "Placar"]);
        assert_eq!(
            dropped_names(&mut w, 3, &sel),
            ["Fora"],
            "fora da selecção, só ele (o Godot)"
        );
        assert!(dropped_names(&mut w, 99, &sel).is_empty());
        let (p, nome) = payload_of(&w, c.to_bits()).expect("carga");
        assert_eq!(p, DragPayload::Object { stable_id: 3 });
        assert_eq!(nome, "Fora");
    }

    /// ⭐ 8b: o que a aba recebe de uma carga da biblioteca — o nome do mestre (o do `ph2d.spawn`) e o
    /// da imagem (o do `sprite.image`, pelo ficheiro).
    /// ⭐ 8c: um objecto largado no campo de uma propriedade de objecto passa a ser o valor dela, pelo
    /// nome; um asset nesse campo recusa.
    #[test]
    fn um_objecto_largado_no_campo_de_objecto_e_o_valor_da_propriedade() {
        use crate::asset_drop::{DropTarget, resolve};
        use ph2d_script::{LuauScript, ScriptId, ScriptValue};
        let alvo = DropTarget::ScriptObject {
            entity_bits: 3,
            row: 1,
        };
        assert_eq!(
            resolve(DragPayload::Object { stable_id: 9 }, alvo),
            DropAction::SetScriptObject {
                entity_bits: 3,
                row: 1,
                stable_id: 9
            }
        );
        assert_eq!(
            resolve(DragPayload::Image { asset: [0; 32] }, alvo),
            DropAction::Refuse
        );
        let mut sim = SimWorld::new();
        let heroi = sim
            .world_mut()
            .spawn((StableId(1), LuauScript::library(ScriptId(1))))
            .id();
        obj(sim.world_mut(), 9, "Porta", 1);
        let bits = heroi.to_bits();
        assert!(into_script_slot(&mut sim, bits, "porta", 9));
        let s = sim.world().get::<LuauScript>(heroi).expect("script");
        assert_eq!(s.own["porta"], ScriptValue::Object("Porta".into()));
        assert!(
            !into_script_slot(&mut sim, bits, "porta", 9),
            "o mesmo: nada muda"
        );
        assert!(
            !into_script_slot(&mut sim, bits, "porta", 77),
            "nenhum objecto"
        );
    }

    #[test]
    fn um_mestre_e_uma_imagem_chegam_ao_script_pelo_nome() {
        let mut sim = SimWorld::new();
        let mestre = sim
            .world_mut()
            .spawn((Name::new("Inimigo"), StableId(41), ph2d_ecs::MasterRoot))
            .id();
        let db = AssetDb::new();
        let porta = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/CodeEditor/ferramentas/smoke/cena8/door_open.png");
        let img = db.load_png_path(&porta).expect("png");
        let atlas: BTreeMap<u32, AssetId> = [(5, img)].into_iter().collect();
        let mut libs = Libraries {
            sim: &mut sim,
            db: &db,
            atlas: &atlas,
        };
        assert_eq!(
            library_name(DragPayload::Prefab { stable_id: 41 }, &mut libs).as_deref(),
            Some("Inimigo")
        );
        assert_eq!(
            library_name(
                DragPayload::Image {
                    asset: *img.as_bytes()
                },
                &mut libs
            )
            .as_deref(),
            Some("door_open")
        );
        assert_eq!(
            library_name(DragPayload::Prefab { stable_id: 9 }, &mut libs),
            None
        );
        let _ = mestre;
    }
}
