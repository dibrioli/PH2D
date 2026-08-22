//! **§12 Sockets / Named Anchors** ([ADR-0072]) — o snapshot que a seção lê e o commit que ela
//! escreve. Irmão do [`super::inspector_ordering`], pela mesma razão dele.
//!
//! # A conversão de unidades mora AQUI, num sítio só
//!
//! ⚠️ O componente guarda a pose num [`ph2d_ecs::Transform`], e **um `Transform` é em metros e
//! radianos** — é a lei da casa, e não há porta de escala. O artista, porém, mede a boca de uma
//! arma **na imagem**, em pixels, e escreve a rotação em graus. Por isso o snapshot converte na
//! saída e o commit converte na entrada, **nas duas pontas do mesmo ficheiro**: duplicar a
//! conversão faria o número que ele lê e o que o motor aplica divergirem no dia em que uma
//! metade fosse corrigida.
//!
//! ⚠️ `bounds`/`center` NÃO se convertem: eles já são pixels da fonte, como o
//! `Sprite::region_rect`, e pela mesma razão — são uma medida tirada da imagem.
//!
//! [ADR-0072]: ../../../../docs/architecture/decisions/0072-named-anchor-unification.md

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, NamedAnchor, NamedAnchorList, SimWorld, World};
use ph2d_editor::{AnchorFieldEdit, InspectorAnchorInfo, InspectorAnchorRow, Toast};

use super::inspector_ordering::queue_set;

const NAMED_ANCHOR_LIST: &str = "ph2d::ecs::NamedAnchorList";

/// Constrói o snapshot da §12, ou `None` quando a entidade não é digna de Inspector.
pub(super) fn build_anchor_info(
    world: &World,
    entity_bits: u64,
    selected: &[u64],
    selected_count: usize,
    pixels_per_meter: f32,
) -> Option<InspectorAnchorInfo> {
    let entity = Entity::from_bits(entity_bits);
    world.get::<ph2d_ecs::Transform>(entity)?;
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let list = world.get::<NamedAnchorList>(entity);
    let rows: Vec<InspectorAnchorRow> = list
        .map(|l| {
            l.iter()
                .map(|a| InspectorAnchorRow {
                    name: a.name.clone(),
                    pos: [
                        a.transform.translation.x * ppm,
                        a.transform.translation.y * ppm,
                    ],
                    rot_deg: a.transform.rotation.to_degrees(),
                    bounds: a.bounds,
                    center: a.center,
                })
                .collect()
        })
        .unwrap_or_default();
    // Divergência: com seleção múltipla, basta uma lista diferente para o «Mixed».
    let mut mixed = false;
    if selected.len() > 1 {
        let mine = list.cloned().unwrap_or_default();
        for &bits in selected {
            let other = world
                .get::<NamedAnchorList>(Entity::from_bits(bits))
                .cloned()
                .unwrap_or_default();
            mixed |= other != mine;
        }
    }
    Some(InspectorAnchorInfo {
        entity_bits,
        rows,
        present: list.is_some(),
        selected_count,
        mixed,
    })
}

/// Aplica uma [`AnchorFieldEdit`].
///
/// ⚠️ **Ler-modificar-escrever a lista inteira**: mexer numa âncora não pode repor as outras.
/// E toda edição **anexa** o componente se ele faltar — mexer num controlo da seção é autorar.
///
/// Devolve um aviso quando a edição foi **recusada** (nome inválido, repetido, ou o cap de 64).
/// ⚠️ *Recusar em silêncio seria pior que aceitar*: o artista escreveria um nome, veria a lista
/// não mudar, e não saberia porquê.
pub(super) fn apply_anchor_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: &AnchorFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
    pixels_per_meter: f32,
) -> Option<Toast> {
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let entity = Entity::from_bits(entity_bits);
    let mut list = sim
        .world()
        .get::<NamedAnchorList>(entity)
        .cloned()
        .unwrap_or_default();

    match edit {
        AnchorFieldEdit::Add => {
            let name = list.next_free_name();
            if let Err(e) = list.insert(NamedAnchor::socket(name)) {
                return Some(Toast::error(format!("Anchor not added: {}", describe(e))));
            }
        }
        AnchorFieldEdit::Remove(i) => {
            let name = list.iter().nth(usize::from(*i)).map(|a| a.name.clone())?;
            list.remove(&name);
            // ⚠️ A lista pode ficar VAZIA e o componente fica — anexado-e-vazio é um estado
            // que o artista criou, e apagá-lo por ele seria decidir no lugar dele. O «Remove»
            // que retira o componente inteiro não existe de propósito: ele não tem gesto.
        }
        AnchorFieldEdit::Rename(i, new_name) => {
            let idx = usize::from(*i);
            list.iter().nth(idx)?;
            if let Err(e) = ph2d_ecs::validate_anchor_name(new_name) {
                return Some(Toast::error(format!(
                    "Anchor name rejected: {}",
                    describe(e)
                )));
            }
            if list
                .iter()
                .enumerate()
                .any(|(j, a)| j != idx && a.name == *new_name)
            {
                return Some(Toast::error(format!(
                    "Anchor name '{new_name}' is already used on this sprite"
                )));
            }
            if let Some(a) = list.0.get_mut(idx) {
                a.name = new_name.clone();
            }
        }
        other => {
            let idx = match other {
                AnchorFieldEdit::Pos(i, ..)
                | AnchorFieldEdit::Rot(i, _)
                | AnchorFieldEdit::BoundsOn(i, _)
                | AnchorFieldEdit::Bounds(i, ..)
                | AnchorFieldEdit::CenterOn(i, _)
                | AnchorFieldEdit::Center(i, ..) => usize::from(*i),
                // Tratados acima; o braço existe para o `match` não precisar de curinga, que
                // engoliria em silêncio a próxima variante.
                AnchorFieldEdit::Add | AnchorFieldEdit::Remove(_) | AnchorFieldEdit::Rename(..) => {
                    return None;
                }
            };
            apply_field(list.0.get_mut(idx)?, other, ppm);
        }
    }
    queue_set(queue, registry, entity_bits, NAMED_ANCHOR_LIST, &list);
    // Chegou aqui = a edição foi aceite. O `None` é «sem aviso», não «sem efeito».
    None
}

/// Escreve um campo de UMA âncora. Separado para manter o `match` de cima legível.
fn apply_field(a: &mut NamedAnchor, edit: &AnchorFieldEdit, ppm: f32) {
    match edit {
        AnchorFieldEdit::Pos(_, axis, v) => {
            // px -> m, na única porta que o faz.
            let m = v / ppm;
            if *axis == 0 {
                a.transform.translation.x = m;
            } else {
                a.transform.translation.y = m;
            }
        }
        AnchorFieldEdit::Rot(_, deg) => a.transform.rotation = deg.to_radians(),
        AnchorFieldEdit::BoundsOn(_, on) => {
            // Ligar semeia um retângulo VISÍVEL: um de área zero seria indistinguível de não
            // ter ligado nada.
            a.set_bounds(on.then_some(a.bounds.unwrap_or([0.0, 0.0, 16.0, 16.0])));
        }
        AnchorFieldEdit::Bounds(_, f, v) => {
            let mut b = a.bounds.unwrap_or([0.0; 4]);
            if let Some(slot) = b.get_mut(usize::from(*f)) {
                *slot = *v;
            }
            a.set_bounds(Some(b));
        }
        AnchorFieldEdit::CenterOn(_, on) => {
            a.set_center(on.then_some(a.center.unwrap_or([0.0, 0.0, 8.0, 8.0])));
        }
        AnchorFieldEdit::Center(_, f, v) => {
            let mut c = a.center.unwrap_or([0.0; 4]);
            if let Some(slot) = c.get_mut(usize::from(*f)) {
                *slot = *v;
            }
            a.set_center(Some(c));
        }
        AnchorFieldEdit::Add | AnchorFieldEdit::Remove(_) | AnchorFieldEdit::Rename(..) => {}
    }
}

/// A recusa, em palavras que o artista entende — nunca o nome da variante.
/// ⚠️ **Cada braço nomeia o SEU teto — a função não recebe nenhum.**
///
/// Ela recebia um `cap: usize`, e cada chamador passava o número que achava certo: o `Add` passava
/// `ANCHORS_MAX`, o `Rename` passava `ANCHOR_NAME_MAX_BYTES`. As duas mensagens liam igual
/// («over the limit of 64») e só estavam certas **por coincidência** de os dois tetos serem 64 —
/// mexer num deles tornava uma delas falsa, em silêncio (auditoria 2026-08-22). *Quem sabe de que
/// teto uma falha é, é a falha.*
fn describe(e: ph2d_ecs::AnchorNameError) -> String {
    match e {
        ph2d_ecs::AnchorNameError::Empty => "the name is empty".to_string(),
        ph2d_ecs::AnchorNameError::TooLong => {
            format!("the name is over {} bytes", ph2d_ecs::ANCHOR_NAME_MAX_BYTES)
        }
        ph2d_ecs::AnchorNameError::ControlChar => "the name has a control character".to_string(),
        ph2d_ecs::AnchorNameError::Duplicate => "that name is already used".to_string(),
        ph2d_ecs::AnchorNameError::ListFull => {
            format!("this sprite already has {} anchors", ph2d_ecs::ANCHORS_MAX)
        }
    }
}

// ⛔ Houve aqui um `detach_anchor_list`, atrás de um `#[allow(dead_code)]` e **sem chamador
// nenhum** — retirado na auditoria de 2026-08-22. Ele contradizia, a seis linhas de distância, o
// comentário do braço `Remove`: *«O "Remove" que retira o componente inteiro não existe de
// propósito: ele não tem gesto»*. Uma das duas coisas mentia, e o `allow` era o que a deixava
// mentir em silêncio. *Código morto e comentário velho mentem — e um `allow(dead_code)` é o
// carimbo que autoriza a mentira a ficar.*

#[cfg(test)]
mod tests {
    use super::*;

    /// px ↔ m fecha o círculo: o que o artista escreve é o que ele volta a ler.
    #[test]
    fn the_position_round_trips_through_pixels() {
        let mut a = NamedAnchor::socket("m");
        apply_field(&mut a, &AnchorFieldEdit::Pos(0, 0, 28.0), 100.0);
        assert!((a.transform.translation.x - 0.28).abs() < 1e-6);
        // O caminho de volta é o do snapshot.
        assert!((a.transform.translation.x * 100.0 - 28.0).abs() < 1e-4);
    }

    /// A rotação viaja em radianos e é autorada em graus.
    #[test]
    fn the_rotation_is_authored_in_degrees_and_stored_in_radians() {
        let mut a = NamedAnchor::socket("m");
        apply_field(&mut a, &AnchorFieldEdit::Rot(0, 90.0), 100.0);
        assert!((a.transform.rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }

    /// ⚠️ Ligar a área semeia um retângulo **visível**. Um de área zero seria indistinguível de
    /// não ter ligado nada, e o artista ligaria a caixa e não veria mudança nenhuma.
    #[test]
    fn switching_bounds_on_seeds_a_visible_rect() {
        let mut a = NamedAnchor::socket("m");
        apply_field(&mut a, &AnchorFieldEdit::BoundsOn(0, true), 100.0);
        let b = a.bounds.expect("a area tinha de nascer");
        assert!(b[2] > 0.0 && b[3] > 0.0, "area nula: invisivel");
        assert_eq!(a.kind(), ph2d_ecs::AnchorKind::Slice);
    }

    /// Editar um campo da área preserva os outros três.
    #[test]
    fn editing_one_bounds_field_preserves_its_siblings() {
        let mut a = NamedAnchor::socket("m");
        a.set_bounds(Some([1.0, 2.0, 3.0, 4.0]));
        apply_field(&mut a, &AnchorFieldEdit::Bounds(0, 2, 9.0), 100.0);
        assert_eq!(a.bounds, Some([1.0, 2.0, 9.0, 4.0]));
    }

    /// Desligar a área leva o miolo — a mesma lei do componente.
    #[test]
    fn switching_bounds_off_takes_the_centre_with_it() {
        let mut a = NamedAnchor::socket("m");
        apply_field(&mut a, &AnchorFieldEdit::BoundsOn(0, true), 100.0);
        apply_field(&mut a, &AnchorFieldEdit::CenterOn(0, true), 100.0);
        assert_eq!(a.kind(), ph2d_ecs::AnchorKind::NineSliceRegion);
        apply_field(&mut a, &AnchorFieldEdit::BoundsOn(0, false), 100.0);
        assert_eq!(a.center, None);
        assert_eq!(a.kind(), ph2d_ecs::AnchorKind::Socket);
    }
}
