//! **QUE PAINEL ESTA ÁRVORE DESCREVE** — a porta única do W8b (plano UI/UX §4).
//!
//! Uma moldura autorada com filhos vestidos é a descrição de um painel. Esta função responde
//! *qual*, e o resultado (`PanelSpec`) é **dado simples**: quem o consome escreve código-fonte
//! (`ph2d-ui-codegen`), sem ECS, sem documento e sem janela.
//!
//! # A ordem das rows é a ordem dos FILHOS, e é uma decisão
//!
//! A moldura com auto layout **flui os filhos na ordem em que eles estão na árvore** — é por isso
//! que arrastar um filho dentro de um fluxo **REORDENA** em vez de o mover (ADR-0153: a pose é
//! derivada, então um arrasto não tem onde pousar). Logo, a ordem que o artista vê no painel e na
//! Hierarquia **é** a ordem dos filhos, e ler o z (que é outra pergunta) daria um painel cujas
//! rows saem noutra ordem que a moldura mostra.
//!
//! # Só quem VESTE vira row
//!
//! Um filho sem [`VecWidget`] é desenho — um fundo, uma divisória decorativa, um ícone. Ele
//! continua a ser desenhado pela pele da moldura; o que ele **não** é é um controle. Transformar
//! todo filho em row daria um painel com linhas que não fazem nada, que é o item-de-menu-morto
//! deste repo na sua forma mais cara.
//!
//! ⚠️ E um `kind` que este build **não conhece** também não vira row: o `from_code` devolve
//! `None` de propósito (um documento autorado por um build mais novo), e inventar um tipo aqui
//! seria gerar código para um widget que não existe.

use ph2d_ecs::{Children, Entity, Name, SimWorld, VecWidget};
use ph2d_editor::widget::WidgetKind;
use ph2d_ui_codegen::{PanelSpec, RowSpec};

/// O rótulo de uma entidade — o `Name` que o artista digitou.
fn label_of(sim: &SimWorld, e: Entity) -> Option<String> {
    sim.world().get::<Name>(e).map(|n| n.0.to_string())
}

/// **A chave estável de uma row**, derivada do rótulo.
///
/// ⚠️ Ela é o que vira `NodeId` por hash em runtime, então tem de ser **estável e legível**:
/// minúsculas, e tudo o que não é alfanumérico vira `_`. Duas rows de mesmo rótulo produzem a
/// mesma chave — e isso é **correto e nomeado**: elas são o mesmo controle autorado duas vezes, e
/// o gerador não é quem decide desempatar nomes que o artista repetiu.
pub(crate) fn key_of(label: &str) -> String {
    let mut k: String = label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    if k.is_empty() {
        k.push('_');
    }
    k
}

/// Percorre a sub-árvore de `frame` **na ordem dos filhos**, juntando quem veste.
fn walk(sim: &SimWorld, e: Entity, out: &mut Vec<RowSpec>) {
    if let Some(w) = sim.world().get::<VecWidget>(e)
        && let Some(kind) = WidgetKind::from_code(w.kind)
    {
        let label = label_of(sim, e).unwrap_or_default();
        out.push(RowSpec {
            kind: kind.ident().to_string(),
            key: key_of(&label),
            label,
        });
    }
    // ⚠️ `Children` preserva a ordem de inserção da hierarquia, que é a ordem que o layout flui e
    // a que a Hierarquia mostra. Uma cópia é preciso porque o `walk` empresta o mundo de novo.
    let kids: Vec<Entity> = sim
        .world()
        .get::<Children>(e)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    for k in kids {
        walk(sim, k, out);
    }
}

/// **O painel que a moldura `frame` descreve.**
///
/// ⚠️ A própria moldura **não vira row**, mesmo que alguém a tenha vestido: ela é o painel, e um
/// painel que contivesse a si próprio como primeira linha seria a árvore lida um nível acima do
/// que ela é.
#[must_use]
pub(crate) fn of(sim: &SimWorld, frame: Entity) -> PanelSpec {
    let title = label_of(sim, frame).unwrap_or_default();
    let mut rows = Vec::new();
    let kids: Vec<Entity> = sim
        .world()
        .get::<Children>(frame)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    for k in kids {
        walk(sim, k, &mut rows);
    }
    PanelSpec {
        id: key_of(&title),
        title,
        rows,
    }
}

#[cfg(test)]
#[path = "ui_panel_spec_tests.rs"]
mod tests;
