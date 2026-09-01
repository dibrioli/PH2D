//! ⭐⭐⭐ **A porta de ESCRITA das variações** — gravar uma versão, e renomear o valor de uma.
//!
//! # ⛔⛔⛔ Porque este módulo existe (Enio, 2026-09-01)
//!
//! *«nós realmente não conseguimos nos entender e precisamos mudar o modo de criar Variações. Não
//! vamos mais usar as chaves no nome. Vamos usar o Card com botões específicos para cada função…
//! com o momento de colocar o nome que vai gerar o botão seletor da variação.»*
//!
//! Até 31/08 a declaração vivia dentro do `Name` e **todo gesto tinha de a reescrever**. Isso pôs
//! renomear no caminho de uma operação estrutural, e custou seis reports com foto. A declaração é
//! agora [`ph2d_ecs::VariantValues`], e o `Name` voltou a ser um nome.
//!
//! # A porta ÚNICA de cada pergunta
//!
//! | Pergunta | Aqui |
//! |---|---|
//! | que valores esta receita declara? | [`values_of`] |
//! | gravar a modificação de uma cópia como versão nova | [`save_variation`] |
//! | renomear o valor que uma receita declara | [`rename_value`] |
//! | já existe uma irmã que declara esta combinação? | [`sibling_declaring`] |
//!
//! ⚠️ O **elo de família** continua a ser o `InstanceOf` de sempre — não há segundo ponteiro.

use ph2d_ecs::{Entity, SimWorld, VariantValues};
use std::collections::BTreeMap;

/// O que a receita de `StableId == id` declara. Vazio quando ela não declara nada (modo plano).
pub(crate) fn values_of(sim: &mut SimWorld, id: u64) -> BTreeMap<String, String> {
    crate::instance_verbs_walk::entity_for_stable_id(sim, id)
        .map(Entity::from_bits)
        .and_then(|e| {
            sim.world()
                .get::<VariantValues>(e)
                .map(|v| v.values.clone())
        })
        .unwrap_or_default()
}

/// Escreve a declaração de uma receita. ⚠️ Vazio **remove** o componente — um mapa vazio no arquivo
/// e a ausência dele têm de ser a mesma coisa, senão duas gravações do mesmo estado dão bytes
/// diferentes e o undo regista um passo que ninguém deu.
pub(crate) fn set_values(sim: &mut SimWorld, entity: Entity, values: BTreeMap<String, String>) {
    if values.is_empty() {
        sim.world_mut().entity_mut(entity).remove::<VariantValues>();
    } else {
        sim.world_mut()
            .entity_mut(entity)
            .insert(VariantValues { values });
    }
}

/// ⭐ **A irmã que declararia a MESMA combinação** que `master` declararia com `key = value`.
///
/// É a pergunta que impede duas receitas de dizerem o mesmo — o estado em que a fileira colapsa
/// para um valor só e o cartão desce ao modo plano.
pub(crate) fn sibling_declaring(
    sim: &mut SimWorld,
    master: u64,
    key: &str,
    value: &str,
) -> Option<u64> {
    let mut wanted = values_of(sim, master);
    wanted.insert(key.to_string(), value.to_string());
    let family = crate::render_loop::inspector_instance::family_members(sim, master);
    family
        .into_iter()
        .find(|m| m.master != master && m.values == wanted)
        .map(|m| m.master)
}

/// Porque um *Salvar Variação* pode não acontecer — e ⚠️ **todo caminho fala**.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SaveRefusal {
    /// A seleção não é (nem pertence a) uma cópia de receita nenhuma.
    NotAnInstance,
    /// A propriedade ou o valor vieram vazios.
    Empty,
    /// Outra versão da família já declara esta combinação.
    Duplicate,
    /// O verbo de promover a receita recusou — a razão dele.
    Verb(crate::instance_verbs::VerbRefusal),
}

/// O que a gravação produziu — a voz do artista sai daqui.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Saved {
    /// A receita nova.
    pub recipe: u64,
    /// Quantas excepções da cópia foram ABSORVIDAS pela versão nova.
    pub absorbed: usize,
    pub property: String,
    pub value: String,
}

/// ⭐⭐⭐ **Gravar a cópia modificada como uma VERSÃO nova.**
///
/// `existing` é *«como se chama o que já existe»* — obrigatório quando a propriedade **nasce
/// agora**, e ignorado quando ela já existe na família.
///
/// # ⚠️ Porque a propriedade nova escreve em TODA a família
///
/// Nascer uma propriedade significa que toda receita passa a declarar um valor nela. Sem escrever
/// as irmãs, a fileira nova nasceria com **um botão em branco** — e uma fileira de um valor só nem
/// sequer é oferecida, então o artista veria o gesto não fazer nada.
///
/// # ⚠️ E uma propriedade NUNCA nasce sozinha
///
/// Não há gesto de *«criar propriedade vazia»*: uma fileira com um valor é um controlo que não
/// escolhe nada. É por isso que a propriedade nasce **com duas** — a que já existia e a que o
/// artista acabou de fazer.
pub(crate) fn save_variation(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
    entity_bits: u64,
    property: &str,
    value: &str,
    existing: Option<&str>,
) -> Result<Saved, SaveRefusal> {
    let property = property.trim();
    let value = value.trim();
    if property.is_empty() || value.is_empty() {
        return Err(SaveRefusal::Empty);
    }
    let entity = Entity::from_bits(entity_bits);
    let root =
        crate::instance_verbs::instance_root_of(sim, entity).ok_or(SaveRefusal::NotAnInstance)?;
    let base = sim
        .world()
        .get::<ph2d_ecs::InstanceOf>(root)
        .map(|l| l.master)
        .filter(|m| *m != 0)
        .ok_or(SaveRefusal::NotAnInstance)?;
    if sibling_declaring(sim, base, property, value).is_some() {
        return Err(SaveRefusal::Duplicate);
    }
    let absorbed = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(root)
        .map_or(0, |o| o.overrides.len());

    // ⚠️ **A família é lida ANTES** — depois do verbo há uma receita a mais, e ela é a que estamos
    // a criar. *Uma lista lida depois incluiria o próprio sujeito.*
    let before: Vec<u64> = crate::render_loop::inspector_instance::family_members(sim, base)
        .into_iter()
        .map(|m| m.master)
        .collect();

    let (recipe, _instance) =
        crate::instance_verbs::make_master(sim, registry, root, docs).map_err(SaveRefusal::Verb)?;

    // A versão nova declara a combinação da BASE mais o valor que o artista escreveu.
    let mut mine = values_of(sim, base);
    mine.insert(property.to_string(), value.to_string());
    set_values(sim, recipe, mine);

    // ⭐ A propriedade que NASCE agora dá nome ao que já existia, em toda a família.
    if let Some(existing) = existing.map(str::trim).filter(|s| !s.is_empty()) {
        for id in before {
            let Some(e) = crate::instance_verbs_walk::entity_for_stable_id(sim, id) else {
                continue;
            };
            let mut v = values_of(sim, id);
            if v.contains_key(property) {
                continue;
            }
            v.insert(property.to_string(), existing.to_string());
            set_values(sim, Entity::from_bits(e), v);
        }
    }
    let recipe_id = sim
        .world()
        .get::<ph2d_ecs::StableId>(recipe)
        .map_or(0, |s| s.0);
    Ok(Saved {
        recipe: recipe_id,
        absorbed,
        property: property.to_string(),
        value: value.to_string(),
    })
}

/// O que a renomeação de um valor produziu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Renamed {
    /// A receita passou a declarar o valor novo.
    Written,
    /// ⭐ O valor pedido **já existe** na família: quem pediu deve TROCAR para essa irmã, não
    /// escrever por cima — duas receitas com a mesma combinação colapsam a fileira.
    Switch(u64),
    /// Nada a fazer (o valor já era esse), ou a receita não declara aquela chave.
    Nothing,
}

/// ⭐⭐ **Renomear o valor que uma receita declara** — o campo que o clique no chip aceso abre.
pub(crate) fn rename_value(sim: &mut SimWorld, master: u64, key: &str, value: &str) -> Renamed {
    let value = value.trim();
    if value.is_empty() {
        return Renamed::Nothing;
    }
    let mut mine = values_of(sim, master);
    if mine.get(key).map(String::as_str) == Some(value) {
        return Renamed::Nothing;
    }
    if let Some(sibling) = sibling_declaring(sim, master, key, value) {
        return Renamed::Switch(sibling);
    }
    let Some(e) = crate::instance_verbs_walk::entity_for_stable_id(sim, master) else {
        return Renamed::Nothing;
    };
    mine.insert(key.to_string(), value.to_string());
    set_values(sim, Entity::from_bits(e), mine);
    Renamed::Written
}

#[cfg(test)]
#[path = "variant_save_tests.rs"]
mod tests;
