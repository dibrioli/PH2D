//! Editor-layer enforcement of `Name` uniqueness.
//!
//! `ph2d_ecs::Name` deliberately permits duplicates — see `Name`'s
//! own docs: "Author tools that care about uniqueness enforce it in
//! the editor layer." This module is that enforcement for the desktop
//! shell.
//!
//! Why it matters:
//! - The Hierarchy panel used to fall back on `name == primary_label`
//!   to decide which row to paint as "selected" when the bridge
//!   couldn't pre-mark it. Two sprites with the same name → both rows
//!   lit up on a single-entity selection (user-reported bug 2026-05-27,
//!   "selecionar uma cópia seleciona a outra"). The fallback is gone,
//!   but unique names are the data-side guarantee that defends against
//!   any future code that keys off labels.
//! - Audit logs (HR-11) use Name as a stable friendly id. Collisions
//!   make logs ambiguous.
//!
//! Strategy: a base label gets bumped with ` (1)`, ` (2)`, ... until a
//! free slot is found. Suffix is **stripped** before bumping so
//! `Sprite (1)` duplicated becomes `Sprite (2)` (not `Sprite (1) (1)`).

use ph2d_ecs::{Entity, Name, Resource, SimWorld};

/// **Os nomes que a timeline ainda ESPERA** — reservados contra reciclagem (Enio, 2026-07-22).
///
/// Uma track cujo objeto foi deletado fica **dormente** e reencontra o dono pelo NOME quando
/// ele volta (`timeline_persist::upkeep`) — é o que faz delete + Ctrl+Z devolver a animação
/// junto com o objeto, porque o undo global não carrega o `TimelineDoc`.
///
/// Mas o nome de um objeto morto voltava ao pote: o objeto SEGUINTE nascia com ele, e a track
/// órfã o adotava — o objeto novo nascia dirigido pela animação do antigo, com a pose reescrita
/// todo frame. É o report *"a timeline não funciona para o segundo objeto"*.
///
/// A cura é tirar a colisão de cena em vez de administrá-la: enquanto uma track espera por
/// "Sprite", **nada mais se chama "Sprite"** — o próximo objeto nasce "Sprite (1)". O undo
/// continua curando porque ele **restaura o `Name` literal** do snapshot e nunca passa por
/// aqui; e quem pergunta *"este nome está livre?"* passa por [`name_in_use`], uma porta só.
///
/// Guarda o **hash** (`WireId`), não o texto: é o que a binding carrega, e é o que a torna
/// comparável sem manter uma segunda cópia do nome que poderia divergir.
///
/// Vive como `Resource` no `World` — onde [`unique_name`] já olha — e por isso **não** entra
/// no `WorldSnapshot` (que serializa `entities`, não resources): reserva é estado vivo de
/// sessão, não conteúdo do documento, e um undo não deve rebobiná-la.
#[derive(Resource, Default)]
pub struct ReservedNames {
    wires: Vec<u64>,
}

impl ReservedNames {
    /// Republica a reserva (chamado pelo upkeep do frame). Zero-alloc no estado estável:
    /// sem órfãs, é um `clear` sobre um vec que já está vazio.
    pub fn publish(&mut self, wires: impl Iterator<Item = u64>) {
        self.wires.clear();
        self.wires.extend(wires);
    }

    fn holds(&self, candidate: &str) -> bool {
        !self.wires.is_empty() && self.wires.contains(&ph2d_ecs::stable_name_id(candidate))
    }
}

/// Returns a `String` that is guaranteed not to collide with any
/// existing `Name` component in `world`. `base` is the desired label;
/// if it's already free, it's returned unchanged.
pub fn unique_name(world: &mut SimWorld, base: &str) -> String {
    let stem = strip_numeric_suffix(base);
    if !name_in_use(world, base) {
        return base.to_owned();
    }
    let mut n: u32 = 1;
    loop {
        let candidate = format!("{stem} ({n})");
        if !name_in_use(world, &candidate) {
            return candidate;
        }
        n = n.checked_add(1).expect("name suffix overflow (u32)");
    }
}

/// Like [`unique_name`] but ignores `exclude`'s own `Name`. Use for
/// **rename** flows where the entity already owns the candidate label
/// (would otherwise self-collide and bump). Returns `base` unchanged
/// when the only collision is the entity itself.
pub fn unique_name_excluding(world: &mut SimWorld, base: &str, exclude: Entity) -> String {
    if !name_in_use_excluding(world, base, exclude) {
        return base.to_owned();
    }
    let stem = strip_numeric_suffix(base);
    let mut n: u32 = 1;
    loop {
        let candidate = format!("{stem} ({n})");
        if !name_in_use_excluding(world, &candidate, exclude) {
            return candidate;
        }
        n = n.checked_add(1).expect("name suffix overflow (u32)");
    }
}

/// **A porta única de *"este nome está livre?"*** — e "em uso" inclui o que a timeline espera.
///
/// As duas metades têm de ser perguntadas juntas: um objeto vivo com o nome, e uma track
/// dormente que o reserva ([`ReservedNames`]). Perguntar só a primeira é o bug que fazia o
/// objeto novo herdar a animação do deletado.
fn name_in_use(world: &mut SimWorld, candidate: &str) -> bool {
    if reserved_holds(world, candidate) {
        return true;
    }
    let w = world.world_mut();
    let mut q = w.query::<&Name>();
    q.iter(w).any(|n: &Name| n.as_str() == candidate)
}

fn name_in_use_excluding(world: &mut SimWorld, candidate: &str, exclude: Entity) -> bool {
    if reserved_holds(world, candidate) {
        return true;
    }
    let w = world.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .any(|(e, n): (Entity, &Name)| e != exclude && n.as_str() == candidate)
}

/// A timeline está segurando este nome? (`false` quando ninguém publicou reserva — o caso de
/// todo harness que não monta uma timeline.)
fn reserved_holds(world: &mut SimWorld, candidate: &str) -> bool {
    world
        .world_mut()
        .get_resource::<ReservedNames>()
        .is_some_and(|r| r.holds(candidate))
}

/// `"Sprite (3)"` → `"Sprite"`. `"Sprite"` → `"Sprite"`. Preserves
/// embedded parens that aren't a trailing ` (n)` group.
fn strip_numeric_suffix(s: &str) -> &str {
    let Some(open) = s.rfind(" (") else {
        return s;
    };
    if !s.ends_with(')') {
        return s;
    }
    let inner = &s[open + 2..s.len() - 1];
    if !inner.is_empty() && inner.chars().all(|c| c.is_ascii_digit()) {
        &s[..open]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{Name, SimWorld};

    fn world_with(names: &[&str]) -> SimWorld {
        let mut sim = SimWorld::new();
        for n in names {
            sim.world_mut().spawn(Name::new(*n));
        }
        sim
    }

    #[test]
    fn returns_base_when_free() {
        let mut sim = world_with(&[]);
        assert_eq!(unique_name(&mut sim, "Sprite"), "Sprite");
    }

    #[test]
    fn bumps_to_1_on_first_collision() {
        let mut sim = world_with(&["Sprite"]);
        assert_eq!(unique_name(&mut sim, "Sprite"), "Sprite (1)");
    }

    #[test]
    fn walks_past_existing_numbered_siblings() {
        let mut sim = world_with(&["Sprite", "Sprite (1)", "Sprite (2)"]);
        assert_eq!(unique_name(&mut sim, "Sprite"), "Sprite (3)");
    }

    #[test]
    fn does_not_compound_suffix_when_duplicating_a_numbered_name() {
        // Duplicating "Sprite (1)" must yield "Sprite (2)", not
        // "Sprite (1) (1)". Strip the numeric suffix before bumping.
        let mut sim = world_with(&["Sprite", "Sprite (1)"]);
        assert_eq!(unique_name(&mut sim, "Sprite (1)"), "Sprite (2)");
    }

    #[test]
    fn fills_gap_in_numbered_sequence() {
        // (1) and (3) exist but (2) is free — pick (2), not (4).
        let mut sim = world_with(&["Sprite", "Sprite (1)", "Sprite (3)"]);
        assert_eq!(unique_name(&mut sim, "Sprite"), "Sprite (2)");
    }

    #[test]
    fn preserves_non_numeric_paren_groups() {
        // "Player (DPS)" is its own base, not a numbered slot.
        let mut sim = world_with(&["Player (DPS)"]);
        assert_eq!(unique_name(&mut sim, "Player (DPS)"), "Player (DPS) (1)");
    }

    #[test]
    fn empty_base_is_uniquified() {
        let mut sim = world_with(&[""]);
        assert_eq!(unique_name(&mut sim, ""), " (1)");
    }

    #[test]
    fn excluding_self_treats_current_name_as_free() {
        // Renaming "Sprite" → "Sprite" is a no-op; the entity already
        // owns the label, so the function must NOT bump it to "(1)".
        let mut sim = SimWorld::new();
        let me = sim.world_mut().spawn(Name::new("Sprite")).id();
        sim.world_mut().spawn(Name::new("Other"));
        assert_eq!(unique_name_excluding(&mut sim, "Sprite", me), "Sprite");
    }

    #[test]
    fn excluding_self_still_collides_with_others() {
        let mut sim = SimWorld::new();
        let me = sim.world_mut().spawn(Name::new("A")).id();
        sim.world_mut().spawn(Name::new("B"));
        assert_eq!(unique_name_excluding(&mut sim, "B", me), "B (1)");
    }

    #[test]
    fn strip_helper_edge_cases() {
        assert_eq!(strip_numeric_suffix("Sprite"), "Sprite");
        assert_eq!(strip_numeric_suffix("Sprite (1)"), "Sprite");
        assert_eq!(strip_numeric_suffix("Sprite (42)"), "Sprite");
        assert_eq!(strip_numeric_suffix("Sprite ()"), "Sprite ()");
        assert_eq!(strip_numeric_suffix("Sprite (DPS)"), "Sprite (DPS)");
        // No space before the opening paren — not our suffix convention
        // (we only strip ` (N)` with a leading space). Leave intact so
        // a user who literally named something "(1)" doesn't lose it.
        assert_eq!(strip_numeric_suffix("(1)"), "(1)");
    }
}
