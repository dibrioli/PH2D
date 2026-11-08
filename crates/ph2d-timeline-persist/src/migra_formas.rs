//! **As trilhas do lado-forma seguem a forma** (GEF02 B2, schema `219 → 220`): a migração
//! `ph2d_physics_ecs::migra_formas_no_retrato` passa a forma de cada corpo para uma filha; a trilha de um
//! canal de física da forma ([`ph2d_ecs::canal_fisica::e_da_forma`]) troca o `wire_id` do corpo pelo da
//! filha. O número do canal não muda.

use ph2d_ecs::canal::familia::FISICA;
use ph2d_timeline::{PropKind, TimelineDoc, WireId};

/// Re-alveja, nos bytes da timeline do ficheiro, cada trilha da forma de `corpo` para `filha` (pares
/// `(StableId do corpo, StableId da filha)`). Devolve quantas mudaram; bytes vazios ficam vazios.
///
/// # Errors
/// Os bytes não são um `TimelineDoc` legível (o load recusa-os logo a seguir, pela mesma porta).
pub fn realveja_trilhas_da_forma(bytes: &mut Vec<u8>, pares: &[(u64, u64)]) -> Result<usize, String> {
    if bytes.is_empty() || pares.is_empty() {
        return Ok(0);
    }
    let mut doc = TimelineDoc::from_bytes(bytes)?;
    let mut n = 0;
    for b in doc.bindings_mut() {
        let PropKind::Field(k) = b.prop else { continue };
        if k.familia != FISICA || !ph2d_ecs::canal_fisica::e_da_forma(k.campo) {
            continue;
        }
        if let Some(&(_, filha)) = pares.iter().find(|&&(c, _)| c == b.wire_id.0) {
            b.wire_id = WireId(filha);
            n += 1;
        }
    }
    if n > 0 {
        *bytes = doc.to_bytes()?;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::FieldKey;
    use ph2d_ecs::canal_fisica as c;

    /// O canal da forma vai para a filha; o do corpo (o tipo) fica; o de outro corpo fica.
    #[test]
    fn a_trilha_da_forma_segue_a_filha_e_a_do_corpo_fica() {
        let mut doc = TimelineDoc::default();
        for (e, campo) in [(1, c::DESLOC_X), (2, c::TIPO), (3, c::ZONA_QUEDA), (4, c::CAMADA)] {
            let t = doc.bind(e, PropKind::Field(FieldKey::new(FISICA, campo, 0, 0)));
            doc.bindings_mut().iter_mut().find(|b| b.target == t).unwrap().wire_id = WireId(e + 100);
        }
        let mut bytes = doc.to_bytes().unwrap();
        let pares = [(101, 201), (102, 202), (103, 203)];
        assert_eq!(realveja_trilhas_da_forma(&mut bytes, &pares), Ok(2));
        let back = TimelineDoc::from_bytes(&bytes).unwrap();
        let ids: Vec<u64> = back.bindings().iter().map(|b| b.wire_id.0).collect();
        assert_eq!(ids, [201, 102, 203, 104]);
        assert_eq!(realveja_trilhas_da_forma(&mut bytes, &pares), Ok(0), "idempotente");
    }
}
