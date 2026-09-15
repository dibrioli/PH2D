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

use ph2d_ecs::{Entity, Name, SimWorld};

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

/// ⭐⭐⭐ **`n` nomes livres de uma vez** — a porta que a FÁBRICA usa (TOP-20 #11).
///
/// # ⚠️ Porque ela existe, e porque não é um `for` à volta da [`unique_name`]
///
/// A [`unique_name`] varre **todos os nomes do mundo** por candidato tentado, e tenta ` (1)`,
/// ` (2)`… até achar um livre. Chamá-la `n` vezes seguidas é `O(n² × nomes)`: uma rajada de 1 024
/// cópias numa cena de 10 000 objectos faz **dez milhões** de comparações de string por tique — e
/// o defeito é invisível numa cena de demonstração, onde `n` é 3.
///
/// Aqui os nomes usados entram num conjunto **uma vez**, e cada candidato é uma consulta a ele:
/// `O(nomes + n)`. ⚠️ **Os nomes gerados entram no conjunto à medida que saem**, senão `n` cópias
/// recebiam todas o mesmo nome — que é exactamente o defeito que esta crate existe para impedir.
///
/// ⚠️ **`BTreeSet` e não `HashSet`**, e o lint estrutural desta casa impõe-o: a ordem de um `Hash*`
/// não é determinista, e o dia em que alguém ITERAR este conjunto — para listar os nomes, para
/// escolher um — a resposta muda entre corridas. *A cerca não é sobre esta função; é sobre a
/// próxima pessoa a lê-la.*
///
/// ⛔ **A lei é a mesma da [`unique_name`], e não uma segunda**: o mesmo `strip_numeric_suffix`, o
/// mesmo formato ` (n)`, a mesma resposta para `n = 1`. Há gate a atar as duas.
#[must_use]
pub fn unique_names(world: &mut SimWorld, base: &str, n: usize) -> Vec<String> {
    let mut usados: std::collections::BTreeSet<String> = {
        let w = world.world_mut();
        let mut q = w.query::<&Name>();
        q.iter(w).map(|x: &Name| x.as_str().to_owned()).collect()
    };
    let stem = strip_numeric_suffix(base);
    let mut out = Vec::with_capacity(n);
    let mut proximo: u32 = 1;
    for _ in 0..n {
        let nome = if usados.contains(base) {
            loop {
                let c = format!("{stem} ({proximo})");
                proximo = proximo.checked_add(1).expect("name suffix overflow (u32)");
                if !usados.contains(&c) {
                    break c;
                }
            }
        } else {
            base.to_owned()
        };
        usados.insert(nome.clone());
        out.push(nome);
    }
    out
}

fn name_in_use(world: &mut SimWorld, candidate: &str) -> bool {
    let w = world.world_mut();
    let mut q = w.query::<&Name>();
    q.iter(w).any(|n: &Name| n.as_str() == candidate)
}

fn name_in_use_excluding(world: &mut SimWorld, candidate: &str, exclude: Entity) -> bool {
    let w = world.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .any(|(e, n): (Entity, &Name)| e != exclude && n.as_str() == candidate)
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

    /// ⭐⭐ **A porta em lote responde o MESMO que a de um** — a lei é uma só.
    ///
    /// (Mutação: não inserir o nome gerado no conjunto ⇒ as três cópias saem com o mesmo nome.)
    #[test]
    fn the_batch_door_answers_the_same_as_the_single_one() {
        let mut sim = world_with(&["Sprite", "Sprite (1)"]);
        assert_eq!(unique_names(&mut sim, "Sprite", 1), vec!["Sprite (2)"]);
        let mut sim = world_with(&["Sprite", "Sprite (1)"]);
        assert_eq!(
            unique_names(&mut sim, "Sprite", 3),
            vec!["Sprite (2)", "Sprite (3)", "Sprite (4)"],
            "cada nome gerado tem de ocupar o lugar dele"
        );
        // E com o nome livre, a primeira fica com o base — como a porta de um.
        let mut sim = world_with(&[]);
        assert_eq!(
            unique_names(&mut sim, "Mob", 2),
            vec!["Mob".to_string(), "Mob (1)".to_string()]
        );
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
