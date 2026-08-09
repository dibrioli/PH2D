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

use ph2d_ecs::{Children, Entity, Name, SimWorld, VecPathRef, VecWidget, VecWidgetIcon};
use ph2d_editor::icons::IconId;
use ph2d_editor::widget::{IconGlyph, WidgetKind, icon_glyph};
use ph2d_panel_authored::rows::Row as AuthoredRow;
use ph2d_ui_codegen::{PanelSpec, RowSpec};
use ph2d_vec_scene::{VecPathId, VecScene};
use ph2d_vector::BezPath;

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
fn icon_of(sim: &SimWorld, scene: &VecScene, e: Entity) -> (Option<BezPath>, Option<IconId>) {
    let Some(id) = sim.world().get::<VecPathRef>(e).map(|r| r.0) else {
        return (None, None);
    };
    let chosen = sim
        .world()
        .get::<VecWidgetIcon>(e)
        .and_then(|c| IconId::from_slug(&c.slug));
    let drawn = scene.path(id).and_then(crate::widget_icon::icon_face);
    // ⚠️ **A precedência é perguntada, não repetida** — a MESMA porta que o canvas e o painel
    // compilado percorrem. E é ela que garante que um slug desconhecido nunca chega ao código
    // gerado: ele vira `None` e o desenho assume, dos dois lados.
    match icon_glyph(chosen, drawn.as_ref()) {
        Some(IconGlyph::Builtin(id)) => (None, Some(id)),
        Some(IconGlyph::Path(_)) => (drawn, None),
        None => (None, None),
    }
}

/// **O que a árvore diz sobre UMA row, antes de escolher a REPRESENTAÇÃO.**
///
/// ⚠️ Ela existe porque a mesma leitura serve dois consumidores com necessidades opostas: o painel
/// VIVO quer a curva (`BezPath`) e o CÓDIGO GERADO quer texto (`to_svg`), porque um `const` não
/// constrói uma curva. Derivar as duas de UMA varredura é o que impede o painel do artista e o
/// painel compilado de descreverem árvores diferentes — a divergência que só uma screenshot revela.
pub(crate) struct Authored {
    kind: WidgetKind,
    label: String,
    key: String,
    rgba: Option<[u8; 4]>,
    drawn: Option<BezPath>,
    chosen: Option<IconId>,
}

/// Percorre a sub-árvore de `frame` **na ordem dos filhos**, juntando quem veste.
fn walk(sim: &SimWorld, scene: &VecScene, e: Entity, out: &mut Vec<Authored>) {
    if let Some(w) = sim.world().get::<VecWidget>(e)
        && let Some(kind) = WidgetKind::from_code(w.kind)
    {
        let label = label_of(sim, e).unwrap_or_default();
        let rgba = kind
            .takes_colour()
            .then(|| colour_of(sim, scene, e))
            .flatten();
        let (drawn, chosen) = if kind.takes_icon() {
            icon_of(sim, scene, e)
        } else {
            (None, None)
        };
        out.push(Authored {
            kind,
            key: key_of(&label),
            label,
            rgba,
            drawn,
            chosen,
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
pub(crate) fn authored(sim: &SimWorld, scene: &VecScene, frame: Entity) -> Vec<Authored> {
    let mut rows = Vec::new();
    let kids: Vec<Entity> = sim
        .world()
        .get::<Children>(frame)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    for k in kids {
        walk(sim, scene, k, &mut rows);
    }
    rows
}

/// **O painel que a moldura `frame` descreve, em TEXTO** — a forma que o gerador escreve.
#[must_use]
pub(crate) fn of(sim: &SimWorld, scene: &VecScene, frame: Entity) -> PanelSpec {
    let title = label_of(sim, frame).unwrap_or_default();
    let rows = authored(sim, scene, frame)
        .into_iter()
        .map(|a| RowSpec {
            kind: a.kind.ident().to_string(),
            key: a.key,
            label: a.label,
            rgba: a.rgba,
            // ⚠️ Só AQUI a curva vira texto, e é a viagem do próprio kurbo — o painel vivo nunca a
            // faz, porque ele já tem a curva. Um `to_svg` no laço de pintura seria um formatador
            // de string por quadro para reconstruir o que já estava na mão.
            icon: a.drawn.as_ref().map(ph2d_vector::BezPath::to_svg),
            icon_slug: a.chosen.map(|i| i.slug().to_string()),
        })
        .collect();
    PanelSpec {
        id: key_of(&title),
        title,
        rows,
    }
}

/// **As rows RESOLVIDAS que o painel vivo pinta** — a outra representação da mesma leitura.
#[must_use]
pub(crate) fn live_rows(sim: &SimWorld, scene: &VecScene, frame: Entity) -> Vec<AuthoredRow> {
    authored(sim, scene, frame)
        .into_iter()
        .map(|a| AuthoredRow {
            kind: a.kind,
            id: ph2d_editor::ids::authored_row_id(&a.key),
            label: a.label,
            key: a.key,
            rgba: a.rgba,
            icon: a.drawn,
            icon_id: a.chosen,
        })
        .collect()
}

/// **A moldura que descreve um painel**, se houver alguma no documento.
///
/// ⚠️ **A primeira, e o limite está dito:** com duas molduras autoradas o painel vivo mostra a
/// primeira. Escolher pela SELEÇÃO é o passo seguinte e é decisão de produto — hoje o artista tem
/// uma, e inventar uma regra de desempate que ele não vê seria pior que a limitação.
#[must_use]
pub(crate) fn authored_frame(sim: &mut SimWorld, scene: &VecScene) -> Option<Entity> {
    // O `query` devolve um `QueryState` PRÓPRIO, então o empréstimo mutável acaba aqui e a
    // varredura seguinte corre sobre um `&World` — que é o que o `authored` pede.
    let mut q = sim.world_mut().query::<(Entity, &ph2d_ecs::VecFrame)>();
    let mut frames: Vec<Entity> = q.iter(sim.world()).map(|(e, _)| e).collect();
    frames.sort_unstable();
    frames
        .into_iter()
        .find(|&e| !authored(sim, scene, e).is_empty())
}

#[cfg(test)]
#[path = "ui_panel_spec_tests.rs"]
mod tests;
