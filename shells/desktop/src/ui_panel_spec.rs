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

use ph2d_ecs::{Children, Entity, Name, SimWorld, VecPathRef, VecWidget};
use ph2d_editor::widget::WidgetKind;
use ph2d_ui_codegen::{PanelSpec, RowSpec};
use ph2d_vec_scene::{VecPathId, VecScene};

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

/// **A cor que uma row mostra**, quando o tipo dela É uma cor.
///
/// ⚠️ Ela sai do **preenchimento da forma que veste o widget** — a única resposta que não está no
/// retângulo nem nos tokens. Uma forma sem preenchimento devolve `None`, e é isso que o xadrez de
/// transparência da swatch significa: *nenhuma cor escolhida*.
///
/// ⚠️ E ela é perguntada **só para os tipos que a consomem** ([`WidgetKind::takes_colour`]): ler o
/// fill de um `Slider` seria carregar um número que nada mostra, e o dia em que alguém o mostrasse
/// seria o dia em que o slider mudaria de cor por o artista ter pintado a forma.
fn colour_of(sim: &SimWorld, scene: &VecScene, e: Entity) -> Option<[u8; 4]> {
    let id: VecPathId = sim.world().get::<VecPathRef>(e)?.0;
    let c = scene.path(id)?.fill.as_ref()?.primary_color();
    Some([c.r, c.g, c.b, c.a])
}

/// **O glifo que uma row desenha**, quando o tipo dela É um botão de ícone — a forma que veste o
/// widget, pela porta única [`crate::widget_icon::icon_face`].
///
/// ⚠️ Ele sai daqui como **texto SVG** porque é isso que um `const` de código gerado consegue
/// carregar: um `BezPath` não é construível em `const`. Quem o reconstitui é o `rows()` do painel,
/// uma vez, e a viagem `to_svg`/`from_svg` é a do próprio kurbo — não um formato inventado aqui.
///
/// ⚠️ E, como o `label` e a cor ao lado, ele é um **SNAPSHOT**: o painel gerado desenha o ícone
/// que a forma tinha quando alguém apertou o botão. Quem quer o glifo VIVO olha a moldura no
/// canvas, que relê o documento a cada quadro.
fn icon_of(sim: &SimWorld, scene: &VecScene, e: Entity) -> Option<String> {
    let id: VecPathId = sim.world().get::<VecPathRef>(e)?.0;
    crate::widget_icon::icon_face(scene.path(id)?).map(|p| p.to_svg())
}

/// Percorre a sub-árvore de `frame` **na ordem dos filhos**, juntando quem veste.
fn walk(sim: &SimWorld, scene: &VecScene, e: Entity, out: &mut Vec<RowSpec>) {
    if let Some(w) = sim.world().get::<VecWidget>(e)
        && let Some(kind) = WidgetKind::from_code(w.kind)
    {
        let label = label_of(sim, e).unwrap_or_default();
        let rgba = kind
            .takes_colour()
            .then(|| colour_of(sim, scene, e))
            .flatten();
        let icon = kind.takes_icon().then(|| icon_of(sim, scene, e)).flatten();
        out.push(RowSpec {
            kind: kind.ident().to_string(),
            key: key_of(&label),
            label,
            rgba,
            icon,
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
        walk(sim, scene, k, out);
    }
}

/// **O painel que a moldura `frame` descreve.**
///
/// ⚠️ A própria moldura **não vira row**, mesmo que alguém a tenha vestido: ela é o painel, e um
/// painel que contivesse a si próprio como primeira linha seria a árvore lida um nível acima do
/// que ela é.
#[must_use]
pub(crate) fn of(sim: &SimWorld, scene: &VecScene, frame: Entity) -> PanelSpec {
    let title = label_of(sim, frame).unwrap_or_default();
    let mut rows = Vec::new();
    let kids: Vec<Entity> = sim
        .world()
        .get::<Children>(frame)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    for k in kids {
        walk(sim, scene, k, &mut rows);
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
