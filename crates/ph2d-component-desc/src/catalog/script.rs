//! **O script do utilizador** — um tipo só, e desde o TOP-20 #16 um tipo que o artista ANEXA.
//!
//! # ⭐⭐⭐ A armadilha que esta linha descrevia, e como ela fechou (2026-09-16)
//!
//! Até aqui este ficheiro declarava o `LuauScript` como [`crate::Attach::Machinery`], e o motivo
//! estava escrito como uma cerca: *o `register_script_components` NÃO era chamado no boot*, logo
//! um `LuauScript` posto num objecto era **descartado em silêncio** pelo `WorldSnapshot` —
//! oferecê-lo na paleta daria ao artista um componente que se anexa, se vê e **evapora**.
//!
//! A nota nomeava também a cura e o preço dela, e os dois foram pagos na mesma wave
//! (`docs/Components/13_plano_script_properties.md`):
//!
//! - o registador **entra no boot** (`init.rs`) e o `PROJECT_SCHEMA` sobe um degrau;
//! - o `ScriptHost` deixa de correr só um *placeholder*: ele carrega cada ficheiro como um módulo,
//!   dá a cada objecto uma tabela `self` e corre os ganchos no passo fixo;
//! - e o componente deixa de guardar um `AssetId` de bytecode (que nada produzia) e bits de
//!   entidade (que envenenariam o undo) — guarda o CAMINHO e os números que o artista PÔS.
//!
//! ⇒ `Authored`, `O::ANY`: um script serve a um objecto vazio (*«o cérebro da cena»*) tanto quanto
//! a uma sprite.
//!
//! ⚠️ **Os campos descritos são os DOIS que o componente tem** — o ficheiro e o mapa de valores
//! próprios. As linhas que o Inspector pinta por baixo (uma por `ph2d.property`) **não** são campos
//! deste descritor: elas nascem da DECLARAÇÃO do script, que só a VM conhece, e um override
//! por-campo da F4 sobre elas seria um override sobre um nome que o ficheiro pode mudar amanhã.

use crate::{
    ComponentCategory as C, ComponentDesc as D, FieldDesc, FieldKind as K, ObjectKinds as O,
    Propagation,
};

const fn f(field_id: u16, name: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        name,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
    }
}

/// Os dois campos do `LuauScript`.
const SCRIPT_FIELDS: &[FieldDesc] = &[
    f(1, "Script File", K::Text),
    // ⚠️ Um MAPA nome → valor. `Text` é o controlo mais honesto que esta enumeração tem para ele:
    // o painel não o edita como um campo só, edita-o linha a linha a partir das declarações.
    f(2, "Properties", K::Text),
];

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[D::authored(
    "ph2d::script::LuauScript",
    "Script",
    C::Scripting,
    O::ANY,
    SCRIPT_FIELDS,
)];
