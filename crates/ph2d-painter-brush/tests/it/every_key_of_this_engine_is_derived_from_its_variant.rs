//! ⭐⭐ **A CHAVE DE UM RÓTULO DESTE MOTOR DERIVA DO PAR (enum, variante).**
//!
//! ⚠️ **O enum entra na chave e o FICHEIRO não:** o `height_modes.rs` declara DOIS enums com uma
//! `fn name_key` cada, e uma chave derivada do ficheiro poria as duas famílias no mesmo espaço de
//! nomes — a segunda calaria a primeira em silêncio.
//!
//! ⛔ **O que este gate NÃO pode afirmar:** a derivação exacta, porque o nome do enum não existe em
//! runtime (`BrushBlend::Mix` não sabe dizer-se `brush_blend`). O que ele afirma é a FORMA e a
//! UNICIDADE — que é onde uma cópia-e-cola erra: duas variantes a devolver a mesma chave pintam a
//! mesma palavra em dois chips diferentes, e o artista lê dois modos com o mesmo nome.

use ph2d_painter_brush::{BrushBlend, Falloff};

/// Toda chave deste motor tem a forma `paint_brush.<enum>.<variante>` — três segmentos, minúsculas.
fn bem_formada(k: &str) -> bool {
    let p: Vec<&str> = k.split('.').collect();
    p.len() >= 3
        && p[0] == "paint_brush"
        && p[1..].iter().all(|s| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        })
}

#[test]
fn every_blend_mode_has_its_own_well_formed_key() {
    let mut vistas = std::collections::BTreeSet::new();
    let mut n = 0usize;
    for m in 0u8..64 {
        let b = BrushBlend::from_u8(m);
        if b != BrushBlend::from_u8(0) || m == 0 {
            let k = b.name_key();
            assert!(
                bem_formada(k),
                "{k:?} não tem a forma `paint_brush.<enum>.<variante>`"
            );
            if vistas.insert(k) {
                n += 1;
            }
        }
    }
    // ⛔ Piso de população: o `from_u8` satura, logo um enum que encolhesse passaria a devolver
    //    sempre a mesma variante e este gate ficaria verde a medir uma chave só.
    assert!(
        n >= 20,
        "só {n} modos de mistura distintos — a população encolheu?"
    );
}

#[test]
fn every_falloff_has_its_own_well_formed_key() {
    let mut vistas = std::collections::BTreeSet::new();
    for p in 0u8..32 {
        let k = Falloff::from_u8(p).name_key();
        assert!(
            bem_formada(k),
            "{k:?} não tem a forma `paint_brush.<enum>.<variante>`"
        );
        vistas.insert(k);
    }
    assert!(vistas.len() >= 10, "só {} quedas distintas", vistas.len());
}
