//! Os gates de [`super::axes_for`] — a fileira de versões, e o nome que não decide nada.

use super::{VariantMember, axes_for};

fn m(master: u64, name: &str) -> VariantMember {
    VariantMember {
        master,
        name: name.to_string(),
    }
}

/// ⭐⭐ **A família vira UMA fileira com o nome de cada versão**, e a vigente está acesa.
#[test]
fn the_family_becomes_one_row_naming_each_version() {
    let fam = [m(1, "Casa"), m(2, "Casa Variant")];
    let (rows, beyond) = axes_for(&fam, 2);
    assert_eq!(beyond, 0);
    assert_eq!(rows.len(), 1);
    assert!(rows[0].name.is_empty(), "a fileira nao tem nome (HR-15)");
    let labels: Vec<&str> = rows[0].options.iter().map(|o| o.label.as_str()).collect();
    assert_eq!(labels, ["Casa", "Casa Variant"]);
    assert!(rows[0].options[1].current);
    assert_eq!(rows[0].options[0].master, 1, "o chip leva a IDENTIDADE");
}

/// ⛔ **Uma versão sozinha não oferece nada** — um chip único não escolhe coisa nenhuma, e a
/// fileira é derivada: ela aparece e desaparece com a família.
#[test]
fn a_lone_recipe_offers_nothing() {
    assert_eq!(axes_for(&[m(1, "Casa")], 1), (Vec::new(), 0));
}

/// ⛔⛔ **O NOME é RÓTULO, nunca MECANISMO** — chaves lá dentro não declaram coisa alguma.
///
/// Enio revogou as duas encarnações do mecanismo de propriedades (2026-08-31 e 2026-09-01). Este
/// gate é o que impede a gramática de voltar por uma porta esquecida: um nome com `{…}` atravessa
/// **verbatim** até ao chip.
///
/// ⚠️ **A fixtura carrega o fenómeno de propósito** — com nomes limpos ela provaria nada.
#[test]
fn braces_in_a_name_declare_nothing_and_travel_verbatim() {
    let fam = [
        m(1, "Casa {Size=Small}"),
        m(2, "Bob {Size=Big, State=Idle}"),
    ];
    let (rows, _) = axes_for(&fam, 1);
    assert_eq!(rows.len(), 1, "as chaves nao podem virar fileiras");
    let labels: Vec<&str> = rows[0].options.iter().map(|o| o.label.as_str()).collect();
    assert_eq!(labels, ["Casa {Size=Small}", "Bob {Size=Big, State=Idle}"]);
}

/// ⛔ **O que passa do teto da tabela de ids é ESCRITO**, nunca truncado em silêncio.
#[test]
fn the_versions_beyond_the_id_table_are_counted() {
    let fam: Vec<VariantMember> = (0..crate::ids::MAX_INSTANCE_AXIS_VALUES as u64 + 3)
        .map(|i| m(i + 1, &format!("v{i}")))
        .collect();
    let (rows, beyond) = axes_for(&fam, 1);
    assert_eq!(rows[0].options.len(), crate::ids::MAX_INSTANCE_AXIS_VALUES);
    assert_eq!(beyond, 3);
}
