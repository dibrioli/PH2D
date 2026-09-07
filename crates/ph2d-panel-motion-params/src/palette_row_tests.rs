//! ⭐⭐ **O gate de COSTURA da row de paleta** — ele ficou no PAINEL quando o editor se mudou
//! para a crate-folha, e é a divisão certa: o que ele mede é o `on_click` do painel a virar
//! `MotionParamIntent`, que é uma lei deste lado. *Um gate que segue o código para a folha
//! deixaria a costura do hospedeiro sem régua.*

use crate::snapshot::PaletteRow;
use ph2d_color::parse_palette;

#[cfg(test)]
mod g {
    use super::*;
    /// **O `+` ACRESCENTA uma cor, sem teto — e o `−` para em UMA.**
    ///
    /// O gate do cook (na shell) prova que qualquer comprimento cicla; este prova que o
    /// artista CHEGA lá, que é a metade que falta quando um modelo sem limite fica atrás
    /// de uma UI que não o alcança. O piso de um existe porque uma paleta vazia deixaria o
    /// nó sem nada para ciclar e a faixa sem nada em que clicar de volta.
    #[test]
    fn the_buttons_grow_the_palette_without_a_cap_and_stop_at_one() {
        use crate::snapshot::{ParamsSnapshot, param_pal_add_id, param_pal_remove_id};

        let mut value = String::new(); // não-autorada: as quatro de fábrica
        let click = |value: &str, id| {
            let snap = ParamsSnapshot {
                node: 1,
                title: "motion.color_array".into(),
                modified: Default::default(),
                sections: Vec::new(),
                folded_by_default: std::collections::BTreeSet::new(),
                rows: vec![crate::snapshot::ParamRow::Palette(PaletteRow {
                    name: "palette",
                    label: "Palette".into(),
                    value: value.to_string(),
                })],
            };
            let _ = crate::events::on_click(id, &snap);
            crate::drain_param_intents()
                .into_iter()
                .find_map(|i| match i {
                    crate::MotionParamIntent::SetTextParam { value, .. } => Some(value),
                    _ => None,
                })
        };
        // Nove cliques no `+` a partir de quatro ⇒ TREZE, muito além do cap de quatro.
        for _ in 0..9 {
            value = click(&value, param_pal_add_id(0)).expect("the + emits a palette");
        }
        assert_eq!(
            parse_palette(&value).expect("well formed").len(),
            13,
            "nine clicks on `+` reach thirteen — the old cap was four"
        );
        // E o `−` desce até UMA e para.
        for _ in 0..20 {
            value = click(&value, param_pal_remove_id(0)).expect("the - emits a palette");
        }
        assert_eq!(
            parse_palette(&value).expect("well formed").len(),
            1,
            "the floor is one colour, never zero"
        );
    }
}
