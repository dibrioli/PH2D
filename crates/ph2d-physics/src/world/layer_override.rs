//! **A exceção de uma forma à matriz de camadas** (GEF02 B9, decisão do dono 08/11: o idioma
//! `Collider2D.includeLayers/excludeLayers` do Unity). A matriz continua o DEFAULT por camada; uma
//! forma com `collide = [include, exclude]` diferente de zero colide pela regra
//! [`pair_collides`] em vez da linha da matriz.
//!
//! ## O mecanismo, e porque não chegam os `InteractionGroups`
//!
//! A regra da rapier é `A.mem ∩ B.filter ≠ ∅ AND B.mem ∩ A.filter ≠ ∅` (ou `OR`, só se os dois
//! pedirem). Um `include` que a matriz recusa precisa que o OUTRO lado aceite — e o outro lado não
//! tem exceção nenhuma —, logo nem `And` nem `Or` exprimem a regra. A forma com exceção leva então
//! grupos ABERTOS ([`open_groups`]) e o par decide-se no gancho de filtro de pares da rapier
//! (`ActiveHooks::FILTER_CONTACT_PAIRS | FILTER_INTERSECTION_PAIR`), que só é chamado para pares
//! em que um dos lados o pediu: sem exceção, o mundo é byte-idêntico. A camada e as duas máscaras
//! viajam no `user_data` do colisor ([`OVERRIDE_BIT`]), como o `one_way` — o checkpoint leva-as.
//!
//! ⭐ **A camada de linha VAZIA** (não colide com nada): com `filter = 0` o par nem chegava ao
//! gancho e um `include` dela não valia. O [`super::layers::groups_for`] dá-lhe então o filtro
//! «todas as camadas que NÃO têm linha vazia» — prova de que nada muda sem exceção: a matriz é
//! simétrica, logo nenhuma outra linha tem o bit dela (o teste `And` cai do outro lado), e duas
//! camadas vazias não se aceitam (o bit de cada uma falta no filtro da outra). Só o lado ABERTO
//! de uma forma com exceção passa, e o gancho decide. (Limite: se as 32 linhas forem vazias não
//! há bit livre — a matriz inteira desligada fica como está.)
//!
//! ## As consultas perguntam em nome de quem
//!
//! Um raio/varredura/personagem do PRÓPRIO corpo (player, top-down, projéctil) leva a exceção da
//! forma dele ([`asker_of`]): grupos abertos e a [`pair_collides`] decide com os DOIS lados. Um
//! Ray Sensor tem a sua própria linha («Sees») e pergunta sem exceção.

use rapier2d::geometry::{Collider, ColliderHandle, Group, InteractionGroups, InteractionTestMode};

use super::layers::{LayerMatrix, MAX_LAYERS};

/// O bit do `user_data` que marca um colisor com exceção de camadas (os bits 0 e 1 são do one-way).
pub const OVERRIDE_BIT: u128 = 1 << 2;
const LAYER_SHIFT: u32 = 8;
const INCLUDE_SHIFT: u32 = 32;
const EXCLUDE_SHIFT: u32 = 64;
const LAYER_MASK: u128 = 0x1F;

/// Um lado do par: a camada e as duas máscaras (zero = sem exceção).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Side {
    pub layer: usize,
    pub include: u32,
    pub exclude: u32,
}

/// ⭐ **A regra de contacto com exceções:** o `exclude` de QUALQUER lado ganha; depois vale a
/// matriz OU o `include` de qualquer lado. Simétrica por construção, determinista (só bits).
#[must_use]
pub fn pair_collides(a: Side, b: Side, m: LayerMatrix) -> bool {
    let (la, lb) = (
        1u32 << a.layer.min(MAX_LAYERS - 1),
        1u32 << b.layer.min(MAX_LAYERS - 1),
    );
    if a.exclude & lb != 0 || b.exclude & la != 0 {
        return false;
    }
    m.collides(a.layer, b.layer) || a.include & lb != 0 || b.include & la != 0
}

/// A linha efetiva de uma forma contra quem NÃO tem exceção: `(linha | include) & !exclude`.
#[must_use]
pub fn effective_row(row: u32, collide: [u32; 2]) -> u32 {
    (row | collide[0]) & !collide[1]
}

/// Os bits do `user_data` para `(camada, [include, exclude])`; `0` sem exceção.
#[must_use]
pub(crate) fn override_bits(layer: u8, collide: [u32; 2]) -> u128 {
    if collide == [0; 2] {
        return 0;
    }
    OVERRIDE_BIT
        | (u128::from(layer.min(MAX_LAYERS as u8 - 1)) << LAYER_SHIFT)
        | (u128::from(collide[0]) << INCLUDE_SHIFT)
        | (u128::from(collide[1]) << EXCLUDE_SHIFT)
}

/// Tem este colisor uma exceção?
#[must_use]
pub(crate) fn has_override(c: &Collider) -> bool {
    c.user_data & OVERRIDE_BIT != 0
}

/// O lado de um colisor: das máscaras no `user_data`, ou da camada nos `memberships`.
#[must_use]
pub(crate) fn side_of(c: &Collider) -> Side {
    if has_override(c) {
        let u = c.user_data;
        return Side {
            layer: ((u >> LAYER_SHIFT) & LAYER_MASK) as usize,
            include: (u >> INCLUDE_SHIFT) as u32,
            exclude: (u >> EXCLUDE_SHIFT) as u32,
        };
    }
    let bits = c.collision_groups().memberships.bits();
    Side {
        layer: (bits.trailing_zeros() as usize).min(MAX_LAYERS - 1),
        include: 0,
        exclude: 0,
    }
}

/// Os grupos de um colisor com exceção: abertos, o gancho decide.
#[must_use]
pub(crate) fn open_groups() -> InteractionGroups {
    InteractionGroups::new(Group::ALL, Group::ALL, InteractionTestMode::And)
}

/// O par `(c1, c2)` colide? Só é perguntado quando um dos dois tem exceção (o gancho).
#[must_use]
pub(crate) fn pair_of(c1: &Collider, c2: &Collider, m: LayerMatrix) -> bool {
    pair_collides(side_of(c1), side_of(c2), m)
}

/// O lado de quem pergunta: a `layer` pedida e a exceção do colisor dele (se tiver).
#[must_use]
pub(crate) fn asker_of(layer: u8, c: Option<&Collider>) -> Side {
    let (include, exclude) = match c {
        Some(c) if has_override(c) => {
            let s = side_of(c);
            (s.include, s.exclude)
        }
        _ => (0, 0),
    };
    Side {
        layer: usize::from(layer).min(MAX_LAYERS - 1),
        include,
        exclude,
    }
}

/// Os grupos de uma consulta: os da camada sem exceção (byte-idêntico), abertos com ela.
#[must_use]
pub(crate) fn query_groups(asker: Side, m: LayerMatrix) -> InteractionGroups {
    if asker.include == 0 && asker.exclude == 0 {
        super::layers::groups_for(asker.layer, m)
    } else {
        open_groups()
    }
}

/// O predicado das consultas feitas por `asker`. Sem exceção dele, um colisor sem exceção já foi
/// decidido pelos grupos; com ela, os grupos são abertos e TODO colisor responde pela
/// [`pair_collides`].
pub(crate) fn query_sees(
    asker: Side,
    m: LayerMatrix,
) -> impl Fn(ColliderHandle, &Collider) -> bool {
    let open = asker.include != 0 || asker.exclude != 0;
    move |_, c| (!open && !has_override(c)) || pair_collides(asker, side_of(c), m)
}

#[cfg(test)]
#[path = "layer_override_query_tests.rs"]
mod query_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_regra_e_simetrica_e_o_exclude_ganha() {
        let mut m = LayerMatrix::all();
        m.set(0, 3, false);
        let a = Side {
            layer: 0,
            include: 1 << 3,
            exclude: 1 << 1,
        };
        let b = Side {
            layer: 3,
            include: 0,
            exclude: 0,
        };
        let g = Side {
            layer: 1,
            include: 0,
            exclude: 0,
        };
        assert!(
            pair_collides(a, b, m) && pair_collides(b, a, m),
            "o include vence o «não» da matriz"
        );
        assert!(
            !pair_collides(a, g, m) && !pair_collides(g, a, m),
            "o exclude vale dos dois lados"
        );
        let x = Side {
            layer: 3,
            include: 0,
            exclude: 1,
        };
        assert!(
            !pair_collides(a, x, m),
            "o exclude de um lado vence o include do outro"
        );
    }

    #[test]
    fn os_bits_do_user_data_ida_e_volta() {
        assert_eq!(override_bits(4, [0; 2]), 0);
        let u = override_bits(31, [0xF0F0_0001, 0x8000_0002]);
        assert_ne!(u & OVERRIDE_BIT, 0);
        assert_eq!(((u >> LAYER_SHIFT) & LAYER_MASK) as usize, 31);
        assert_eq!((u >> INCLUDE_SHIFT) as u32, 0xF0F0_0001);
        assert_eq!((u >> EXCLUDE_SHIFT) as u32, 0x8000_0002);
    }
}
