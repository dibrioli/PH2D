//! ⭐⭐⭐ **As fileiras de propriedade do cartão** — derivadas de DADO, nunca do nome.
//!
//! # ⛔⛔⛔ A gramática de chaves MORREU aqui (Enio, 2026-09-01)
//!
//! Até 31/08 este módulo **parseava** `Casa {Size=Big}`: `parse_combo`, `display_name`,
//! `with_value`, `variant_name`, `chip_label`, `hidden_count`, `row_label` e `declared_axes` liam e
//! escreviam uma gramática dentro do `Name`. Ordem do dono: *«Não vamos mais usar as chaves no
//! nome… Vamos tirar do nome o mecanismo de criação de variações»*.
//!
//! A declaração passou a ser [`ph2d_ecs::VariantValues`] na raiz da receita. ⇒ **renomear é
//! inerte**, que era o pedido — e o nome volta a ser aquilo que o artista quis dizer.
//!
//! ⚠️ **O nome continua a aparecer nos CHIPS do modo plano, e isso não é a lei velha**: ali ele é
//! *rótulo* de uma versão que não declara propriedade nenhuma (o modelo dos *Prefab Variants* do
//! Unity). Ninguém o lê para decidir coisa nenhuma. *A doença era o nome ser MECANISMO, não o nome
//! ser mostrado.*
//!
//! # As duas modalidades, e porque são a mesma função
//!
//! - **PLANO** — nenhuma receita da família declara nada: uma fileira sem nome, um chip por versão,
//!   rotulado pelo nome dela.
//! - **PROPRIEDADE** — uma fileira por chave declarada, um chip por valor distinto.
//!
//! Uma função só, porque a modalidade é um **facto dos dados** e não um modo escolhido: duas
//! funções seriam dois sítios a discordar sobre o que está aceso.

use super::inspector_model_instance::VariantChoice;
use crate::ids;
use std::collections::{BTreeMap, BTreeSet};

/// Uma pergunta que a família faz — `Size`, `State`, ou a fileira sem nome do modo plano.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct VariantAxis {
    /// O rótulo da fileira. **Vazio** no modo plano — quem lhe chama *Variant* é o painel (HR-15).
    pub name: String,
    /// As respostas alcançáveis **daqui**. Ver [`axes_for`].
    pub options: Vec<VariantChoice>,
}

/// ⭐ **Uma versão da família, como o cartão precisa de a ver.**
///
/// A shell extrai isto do mundo (`MasterRoot` + [`ph2d_ecs::VariantValues`]) e passa-o para cá —
/// é o que mantém esta crate sem ECS.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct VariantMember {
    /// O `StableId` da receita.
    pub master: u64,
    /// O `Name` dela — **rótulo** do chip no modo plano, nada mais.
    pub name: String,
    /// O que ela declara. Vazio = modo plano.
    pub values: BTreeMap<String, String>,
}

impl VariantMember {
    /// Concorda com `other` em todas as chaves **menos** `except` — a pergunta da grelha.
    ///
    /// ⚠️ Corre sobre a UNIÃO das chaves: a quem falte uma chave, falta — não é «igual nas outras».
    fn matches_except(&self, other: &Self, except: &str) -> bool {
        self.values
            .keys()
            .chain(other.values.keys())
            .filter(|k| k.as_str() != except)
            .all(|k| self.values.get(k) == other.values.get(k))
    }
}

/// ⭐⭐⭐ **As fileiras que esta versão pode oferecer.**
///
/// `members` é a família inteira (a base e as variantes dela), **ordenada por `master`** — a ordem
/// é a que o artista vê, e ordenar por id é o que a torna estável entre quadros. `me` é a versão
/// vigente.
///
/// Devolve as fileiras e quantas ficaram **de fora** do teto da tabela de ids — ⛔ escrito, nunca
/// truncado em silêncio.
///
/// # ⚠️ Uma fileira com um valor só NÃO é oferecida
///
/// É derivação, não uma regra à parte: a fileira existe quando a família declara **dois ou mais**
/// valores distintos naquela chave. Um chip único é um controlo que não escolhe nada — a espécie
/// que a caça aos knobs mortos nomeia —, e como a fileira é derivada ela **desaparece sozinha**
/// quando os valores voltam a concordar. *Uma fileira derivada não pode ficar morta.*
///
/// # A GRELHA, e a combinação que não existe
///
/// Com duas propriedades a família é uma grelha `n × m` que o artista quase nunca enche. O chip de
/// um valor cujo alvo não existe vem com `master == 0` e [`VariantChoice::missing`] — o painel
/// pinta-o esmaecido com `+`, e o clique **cria** a combinação a partir da versão vigente.
/// ⛔ As outras três saídas foram pesadas e recusadas no
/// [plano](../../../../../docs/Components/06_plano_variacoes_sem_chaves.md) §2.3-bis: não fazer
/// nada é o chip morto sob o dedo; aproximar faz o app acender um valor e mostrar outro; recusar
/// manda o artista fazer à mão o que a app sabe fazer.
#[must_use]
pub fn axes_for(members: &[VariantMember], me: u64) -> (Vec<VariantAxis>, usize) {
    if members.len() < 2 {
        // Uma versão só: não há nada a escolher. ⚠️ E isto é o caso comum de uma receita simples.
        return (Vec::new(), 0);
    }
    let keys: BTreeSet<&str> = members
        .iter()
        .flat_map(|m| m.values.keys().map(String::as_str))
        .collect();
    if keys.is_empty() {
        return (flat(members, me), 0);
    }
    let Some(mine) = members.iter().find(|m| m.master == me) else {
        // A versão vigente não é da família — não há combinação a partir da qual perguntar.
        return (Vec::new(), 0);
    };
    let mut axes = Vec::new();
    let mut beyond = 0usize;
    for key in keys {
        let Some(axis) = row_for(members, mine, key) else {
            continue;
        };
        if axes.len() >= ids::MAX_INSTANCE_AXES {
            beyond += 1;
            continue;
        }
        axes.push(axis);
    }
    (axes, beyond)
}

/// Modo PLANO: um chip por versão, rotulado pelo nome dela.
fn flat(members: &[VariantMember], me: u64) -> Vec<VariantAxis> {
    let options: Vec<VariantChoice> = members
        .iter()
        .take(ids::MAX_INSTANCE_AXIS_VALUES)
        .map(|m| VariantChoice {
            master: m.master,
            label: m.name.clone(),
            current: m.master == me,
            missing: false,
        })
        .collect();
    vec![VariantAxis {
        name: String::new(),
        options,
    }]
}

/// A fileira de UMA chave, vista a partir de `mine`. `None` quando não há o que escolher.
fn row_for(members: &[VariantMember], mine: &VariantMember, key: &str) -> Option<VariantAxis> {
    // Valores distintos, na ordem em que a família os apresenta (que é a ordem por `master`).
    let mut values: Vec<&str> = Vec::new();
    for m in members {
        if let Some(v) = m.values.get(key)
            && !values.contains(&v.as_str())
        {
            values.push(v.as_str());
        }
    }
    if values.len() < 2 {
        return None;
    }
    let current = mine.values.get(key).map(String::as_str);
    let options: Vec<VariantChoice> = values
        .into_iter()
        .take(ids::MAX_INSTANCE_AXIS_VALUES)
        .map(|v| {
            // ⚠️ **O alvo é quem concorda em TUDO menos nesta chave** — não é «quem tem este
            // valor». Numa grelha, `Color=Red` sozinho é ambíguo: há um por cada `Size`.
            let target = members
                .iter()
                .filter(|m| m.values.get(key).map(String::as_str) == Some(v))
                .find(|m| m.matches_except(mine, key));
            VariantChoice {
                master: target.map_or(0, |m| m.master),
                label: v.to_string(),
                current: current == Some(v),
                missing: target.is_none(),
            }
        })
        .collect();
    Some(VariantAxis {
        name: key.to_string(),
        options,
    })
}

#[cfg(test)]
#[path = "variant_axes_tests.rs"]
mod tests;
