//! **Arrastar um filho DENTRO de um fluxo é REORDENAR** (ADR-0153, corolário).
//!
//! # Por que o gesto muda de significado
//!
//! A lei da wave é que a posição de um filho colocado é **derivada por frame**. Um arrasto não tem
//! então onde pousar: escrever `Transform.translation` seria escrever num número que o próximo
//! frame recalcula — a forma voltaria para a fila e o artista veria um gesto **quebrado**, com um
//! passo de undo por cima (o undo deste editor regista por DIFF do mundo ECS, e a escrita
//! invisível conta na mesma).
//!
//! Então o mesmo gesto pede outra coisa: **trocar a POSIÇÃO NA FILA**. É o que o Figma faz, e é a
//! única leitura coerente de arrastar algo cuja posição não é sua.
//!
//! # A régua é PUBLICADA, nunca re-derivada
//!
//! Quem decide o slot lê os centros que o passe de layout **acabou de publicar**
//! ([`crate::layout_live::FlowSlots`]) — nunca uma segunda medição das formas. Uma re-derivação
//! divergiria no primeiro `grow`: o artista veria a forma numa posição e o slot ser escolhido por
//! outra, e nada no ecrã diria porquê.
//!
//! # E ela mede CENTROS, não fronteiras
//!
//! O slot é *quantos irmãos têm o centro antes do cursor*. Medir as BORDAS exigiria decidir o que
//! fazer no vão entre duas caixas (onde nenhuma fronteira contém o ponto), e num fluxo com `gap` o
//! artista passa a maior parte do arrasto exactamente ali.

use ph2d_ecs::{ChildOf, Children, Entity, SimWorld, VecLayout};

use crate::layout_live::{Box2, LayoutLive, Reading};

/// **O slot que este ponto pede**, dados os centros dos irmãos JÁ SEM o arrastado, na ordem do
/// fluxo, e a coordenada do cursor no eixo principal.
///
/// Pura e sem estado: é a metade que um gate pode afirmar sem montar uma cena.
#[must_use]
pub(crate) fn slot_at(centres: &[f64], cursor: f64) -> usize {
    centres.iter().filter(|c| **c < cursor).count()
}

/// **Onde uma fila em LINHAS se parte** — os índices em que cada linha começa.
///
/// Uma linha nova começa quando o topo do filho já não alcança a base da linha corrente. É exacto
/// e **sem tolerância**: as bandas de uma grade ou de um wrap não se sobrepõem, então a fronteira é
/// uma comparação e não um limiar — e um limiar aqui seria um número inventado que a primeira
/// moldura de filhos altos desmentiria.
#[must_use]
fn row_starts(boxes: &[Box2]) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut bottom = f64::INFINITY;
    for (i, (lo, hi)) in boxes.iter().enumerate() {
        if hi[1] <= bottom {
            starts.push(i);
            bottom = lo[1];
        } else {
            bottom = bottom.min(lo[1]);
        }
    }
    starts
}

/// **O slot que este ponto pede numa fila em LINHAS.**
///
/// ⚠️ A régua 1-D é ERRADA aqui, e não meramente imprecisa: numa grade 3×3 as três células da
/// coluna 0 partilham o mesmo `x`, então *"quantos centros estão antes do cursor"* conta as três
/// mesmo quando o cursor está na PRIMEIRA — soltar na célula (0,0) devolvia o slot 3, o começo da
/// segunda linha. Medido antes de uma linha ser escrita.
///
/// A ordem de leitura resolve-a: um irmão vem antes do cursor se está numa linha ANTERIOR, ou na
/// mesma linha e mais à esquerda.
#[must_use]
pub(crate) fn slot_at_rows(boxes: &[Box2], cursor: [f64; 2]) -> usize {
    let starts = row_starts(boxes);
    if starts.is_empty() {
        return 0;
    }
    // A banda `y` de cada linha, como a UNIÃO das caixas dela — um filho baixo alinhado ao topo
    // não encolhe a linha que um irmão alto define.
    let band = |r: usize| {
        let from = starts[r];
        let to = starts.get(r + 1).copied().unwrap_or(boxes.len());
        boxes[from..to]
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), (b, t)| {
                (lo.min(b[1]), hi.max(t[1]))
            })
    };
    // ⚠️ **A linha do cursor é a de banda mais PRÓXIMA**, e não *"a que o contém"*: no vão entre
    // duas linhas nenhuma banda contém o ponto, e é ali que o artista passa a maior parte do
    // arrasto — exactamente o argumento que fez a régua 1-D medir centros e não fronteiras.
    let row = (0..starts.len())
        .min_by(|&a, &b| {
            dist_to_band(band(a), cursor[1]).total_cmp(&dist_to_band(band(b), cursor[1]))
        })
        .unwrap_or(0);
    let from = starts[row];
    let to = starts.get(row + 1).copied().unwrap_or(boxes.len());
    let in_row = boxes[from..to]
        .iter()
        .filter(|(lo, hi)| (lo[0] + hi[0]) * 0.5 < cursor[0])
        .count();
    from + in_row
}

/// A distância de `y` a uma banda `[lo, hi]` — zero DENTRO dela.
fn dist_to_band((lo, hi): (f64, f64), y: f64) -> f64 {
    if y < lo {
        lo - y
    } else if y > hi {
        y - hi
    } else {
        0.0
    }
}

/// A moldura que COLOCA esta entidade — `None` quando ela não está dentro de um fluxo.
#[must_use]
pub(crate) fn flow_parent(sim: &SimWorld, e: Entity) -> Option<Entity> {
    let parent = sim.world().get::<ChildOf>(e)?.parent();
    sim.world()
        .get::<VecLayout>(parent)
        .is_some()
        .then_some(parent)
}

/// **O arrasto pousa como uma troca de posição na fila.** Devolve `true` se a ordem mudou.
///
/// `cursor` é o ponto do ponteiro em MUNDO. O eixo lido é o principal do fluxo — no transversal um
/// filho não tem para onde ir, então mexer nele não é informação e ignorá-lo é o que mantém o
/// gesto previsível quando a mão treme.
pub(crate) fn drop_at(
    sim: &mut SimWorld,
    layout: &LayoutLive,
    dragged: Entity,
    cursor: [f32; 2],
) -> bool {
    let Some(parent) = flow_parent(sim, dragged) else {
        return false;
    };
    let Some(slots) = layout.slots_of(parent) else {
        return false;
    };
    // As caixas dos OUTROS: o arrastado sai da régua, senão o slot que ele já ocupa competiria
    // consigo mesmo e o gesto teria um ponto morto do tamanho da própria forma.
    let boxes: Vec<Box2> = slots
        .kids
        .iter()
        .filter(|(e, _)| *e != dragged)
        .map(|(_, b)| *b)
        .collect();
    let cursor = [f64::from(cursor[0]), f64::from(cursor[1])];
    let slot = match slots.reading {
        Reading::Rows => slot_at_rows(&boxes, cursor),
        r => {
            let axis = usize::from(r == Reading::ColumnY);
            let centres: Vec<f64> = boxes
                .iter()
                .map(|(lo, hi)| (lo[axis] + hi[axis]) * 0.5)
                .collect();
            slot_at(&centres, cursor[axis])
        }
    };

    // A ordem VIVA da hierarquia é a autoridade sobre quem são os irmãos — a régua do layout pode
    // ser de um frame em que a lista era outra (uma forma acabada de apagar, por exemplo).
    let mut desired: Vec<Entity> = sim
        .world()
        .get::<Children>(parent)
        .map(|c| c.iter().copied().filter(|e| *e != dragged).collect())
        .unwrap_or_default();
    desired.insert(slot.min(desired.len()), dragged);
    ph2d_ecs::reinsert_children_in_order(sim.world_mut(), parent, &desired)
}

#[cfg(test)]
#[path = "layout_reorder_tests.rs"]
mod tests;
