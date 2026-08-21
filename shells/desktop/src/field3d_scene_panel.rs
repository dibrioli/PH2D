//! ⭐ **A ponte com o PAINEL** — o retrato que a peça publica, e o que cada controle oferece.
//!
//! ⚠️ É um **módulo-filho** de [`super`] e não um irmão de topo: ele lê o mundo pelas mesmas portas
//! do pai e os caminhos antigos continuam a valer pelo re-export. O corte é por **assunto** — o
//! irmão possui a cena (nascer, apagar, cozinhar, apontar) e este responde *"o que o painel mostra
//! e o que ele oferece"*.
//!
//! ⭐ **É aqui, num sítio só, que uma faixa aberta se fecha.** O documento diz a *forma* do que cada
//! grandeza admite ([`ph2d_field::Span`]); esta é a única metade que sabe o enquadramento, e é ela
//! que escreve as duas pontas de cada linha. Espalhar isto era o que fazia toda linha começar em
//! zero — e uma posição negativa ser indigitável, em silêncio.

use super::*;

/// **A ponte com o painel**: publica o retrato da peça.
///
/// ⭐ **A ordem é load-bearing.** Drenar ANTES de publicar é o que faz a edição aparecer no mesmo
/// quadro: se o retrato saísse primeiro, o painel pintaria o valor antigo por um quadro e o
/// controle daria um salto para trás debaixo do dedo — o sintoma clássico de um espelho publicado
/// cedo demais.
pub(crate) fn publish_snapshot(
    world: &bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
    selection: &[bevy_ecs::entity::Entity],
    view_span: f32,
    ms: f32,
) {
    let all = ph2d_field_ecs::walk(world, root);
    let rows = param_rows(world, selection.first().copied(), view_span);
    // ⚠️ A lista de verbos é **derivada de `Mode::ALL`**, que é a fonte da contagem. O painel não
    // conhece o enum — acrescentar um verbo lá faz o seletor seguir sem uma linha de mudança.
    let (active, frame) = with_smoke(|s| (s.gizmo_mode, s.gizmo_frame)).unwrap_or_default();
    let modes = crate::field3d_gizmo::Mode::ALL
        .iter()
        .map(|m| ph2d_panel_model3d::ModeChip {
            key: m.key(),
            active: *m == active,
        })
        .collect();
    let frames = crate::field3d_gizmo::Frame::ALL
        .iter()
        .map(|f| ph2d_panel_model3d::ModeChip {
            key: f.key(),
            active: *f == frame,
        })
        .collect();
    let adds = SHAPES
        .iter()
        .map(|key| ph2d_panel_model3d::ModeChip { key, active: false })
        .collect();
    let ops = ops_for(world, selection);
    let mods = mods_for(world, selection);
    // ⚠️ Vazio sem seleção, pela mesma razão da fileira de operações: um controle que aparece e não
    // faz nada é pior do que um que não aparece.
    let acts = if selection.is_empty() {
        Vec::new()
    } else {
        ACTS.iter()
            .map(|key| ph2d_panel_model3d::ModeChip { key, active: false })
            .collect()
    };
    ph2d_panel_model3d::publish(ph2d_panel_model3d::ModelSnapshot {
        modes,
        frames,
        adds,
        ops,
        mods,
        acts,
        rows,
        node_count: all.len(),
        last_trace_ms: ms,
    });
}

/// ⭐ **Os números do objeto selecionado** — o painel é o inspetor da seleção.
///
/// ⚠️ **Mudou de forma na W10.** Antes era uma linha por nó com o raio dele — uma segunda vista da
/// estrutura, a competir com a Hierarquia e sem onde pôr largura, altura e profundidade. A divisão
/// passou a ser a da casa: a Hierarquia mostra **o que existe**, o painel mostra **os números do
/// escolhido**.
///
/// `view_span` é o alcance do **gesto** (ver [`ph2d_field::Span`]): uma posição e uma largura não
/// têm teto físico, e quem escolhe até onde o slider vai é a vista — o que cabe no enquadramento. O
/// documento só contribui as **paredes** (um filete que não cabe).
///
/// ⭐ **É AQUI, num sítio só, que uma faixa aberta se fecha.** O documento diz a *forma* do que cada
/// grandeza admite; esta função é a única que sabe o enquadramento, e é ela que escreve as duas
/// pontas. Espalhar isto por linha era o que fazia toda linha começar em zero.
pub(crate) fn param_rows(
    world: &bevy_ecs::world::World,
    selected: Option<bevy_ecs::entity::Entity>,
    view_span: f32,
) -> Vec<ph2d_panel_model3d::ParamRow> {
    let Some(e) = selected else {
        return Vec::new();
    };
    // ⚠️ O valor E as pontas vêm os DOIS do nó (`params_of`). Um painel que guardasse o seu próprio
    // valor teria duas verdades sobre o mesmo número, e a que aparece na tela seria a errada sempre
    // que algo o mudasse de outro lado — um desfazer, um arquivo aberto, o gizmo.
    ph2d_field_ecs::params_of(world, e)
        .into_iter()
        .map(|(param, d)| {
            use ph2d_field::{Bound, Span};
            let (lo, bound) = match d.span {
                // Positiva: o documento recusa `≤ 0`, e o teto é o que cabe no quadro.
                Span::Positive => (0.0, Bound::Soft(view_span)),
                // A única ponta que o documento **impõe**.
                Span::Wall(w) => (0.0, Bound::Hard(w)),
                // Simétrica em torno da origem: as duas pontas são da vista, e a de baixo é
                // negativa — sem isto uma posição negativa não se digita.
                Span::Free => (-view_span, Bound::Soft(view_span)),
                // Periódica: as pontas são da representação, e a vista não tem voto.
                Span::Turn(half) => (-half, Bound::Wrap(half)),
                // ⭐ **Sem faixa nenhuma**: a grandeza tem valor e não é editável neste estado. As
                // duas pontas colapsam no próprio valor — não há para onde arrastar — e a linha
                // segue marcada para o painel a pintar como facto.
                Span::Locked => (d.value, Bound::Wrap(d.value)),
                // ⭐ **Contagem**: as duas pontas são do DOCUMENTO — o piso é 1 porque zero cópias
                // é a peça a desaparecer (e apagar já tem botão), e o teto é o da matriz.
                Span::Count { max } => (1.0, Bound::Hard(max as f32)),
            };
            ph2d_panel_model3d::ParamRow {
                entity: e.to_bits(),
                param,
                key: d.key,
                value: d.value,
                lo,
                bound,
                live: d.span != Span::Locked,
                integral: matches!(d.span, Span::Count { .. }),
            }
        })
        .collect()
}

/// ⭐ **As formas que se podem acrescentar**, na ordem do seletor.
///
/// ⚠️ **É a fonte da contagem**, como o `Mode::ALL`: acrescentar uma primitiva aqui faz o painel
/// seguir sem uma linha de mudança.
pub(crate) const SHAPES: [&str; 4] = [
    "panel.model3d.add.box",
    "panel.model3d.add.sphere",
    "panel.model3d.add.cylinder",
    "panel.model3d.add.torus",
];

/// As ações sobre o objeto escolhido, na ordem do seletor.
/// ⭐ **Os modificadores oferecidos, e quais o nó já tem** — interruptores, não ações.
///
/// ⚠️ **A lista é derivada de [`ph2d_field::UnaryKind::ALL`]**, que é a fonte da contagem: um
/// modificador novo entra lá e o painel segue sem uma linha de mudança. É a mesma lei do `Mode::ALL`
/// e do `SHAPES`.
///
/// ⚠️ Vazio sem seleção — um interruptor sem nó para ligar não tem o que dizer.
pub(crate) fn mods_for(
    world: &bevy_ecs::world::World,
    selection: &[bevy_ecs::entity::Entity],
) -> Vec<ph2d_panel_model3d::ModeChip> {
    let Some(&one) = selection.first() else {
        return Vec::new();
    };
    if world.get::<FieldNode>(one).is_none() {
        return Vec::new();
    }
    let have = ph2d_field_ecs::mods_of(world, one);
    ph2d_field::UnaryKind::ALL
        .iter()
        .map(|k| ph2d_panel_model3d::ModeChip {
            key: k.key(),
            // ⭐ **Aceso = o nó JÁ TEM um daquela natureza.** É o que faz o botão dizer o estado em
            // vez de só disparar — e é a diferença entre um interruptor e um botão que empilha
            // cascas sem o artista perceber.
            active: have.iter().any(|u| u.kind() == *k),
        })
        .collect()
}

pub(crate) const ACTS: [&str; 2] = ["panel.model3d.act.duplicate", "panel.model3d.act.delete"];

/// As três booleanas, na ordem do seletor.
pub(crate) const OPS: [&str; 3] = [
    "panel.model3d.op.union",
    "panel.model3d.op.subtract",
    "panel.model3d.op.intersect",
];

/// ⭐ **O tamanho de uma forma nova, DERIVADO do enquadramento.**
///
/// ⚠️ A condição que o fixa é a única que importa: uma forma nova tem de ser **vista**. Um tamanho
/// fixo em unidades de mundo nasce invisível numa peça grande e tapa a janela numa pequena — e nos
/// dois casos o artista conclui que o botão não funcionou. Um quarto da meia-altura do quadro põe-na
/// a metade da altura da tela, que é onde se vê o que ela é.
pub(crate) fn new_shape_size(half_extent: f32) -> f32 {
    (half_extent * 0.25).max(f32::MIN_POSITIVE)
}

/// A primitiva que cada posição do seletor cria, no tamanho do enquadramento.
///
/// ⚠️ **Todas nascem com o `round` que têm direito**, e não a zero: este é o módulo cujo argumento é
/// o arredondamento, e uma caixa de aresta viva ao nascer esconderia exatamente aquilo que ele faz
/// melhor do que o Blender. O valor é uma fração do tamanho, então ele cabe sempre.
pub(crate) fn shape_at(slot: usize, r: f32) -> Option<Primitive> {
    let round = r * 0.1;
    Some(match slot {
        0 => Primitive::Box {
            half: [r; 3],
            round,
        },
        1 => Primitive::Sphere { radius: r },
        2 => Primitive::Cylinder {
            radius: r,
            half_height: r * 1.2,
            round,
        },
        3 => Primitive::Torus {
            major: r,
            minor: r * 0.35,
        },
        _ => return None,
    })
}

pub(crate) fn op_at(slot: usize) -> Option<Op> {
    Some(match slot {
        0 => Op::Union(Blend::Sharp),
        1 => Op::Difference(Blend::Sharp),
        2 => Op::Intersection(Blend::Sharp),
        _ => return None,
    })
}

/// ⭐ **Quais operações fazem sentido AGORA** — e vazio quando nenhuma faz.
///
/// ⚠️ Publicar a fileira sempre daria três botões que às vezes não fazem nada, que é a affordance
/// que mente. Ela aparece em dois casos, e em cada um quer dizer uma coisa precisa:
///
/// | Selecionado | O que os botões fazem | O «ativo» |
/// |---|---|---|
/// | uma **operação** | trocam-na (união vira subtração) | a operação que ela é |
/// | **dois ou mais irmãos** | embrulham-nos numa operação nova | nenhum |
pub(crate) fn ops_for(
    world: &bevy_ecs::world::World,
    selected: &[bevy_ecs::entity::Entity],
) -> Vec<ph2d_panel_model3d::ModeChip> {
    let chips = |active: Option<usize>| -> Vec<ph2d_panel_model3d::ModeChip> {
        OPS.iter()
            .enumerate()
            .map(|(i, key)| ph2d_panel_model3d::ModeChip {
                key,
                active: active == Some(i),
            })
            .collect()
    };
    if let [one] = selected
        && let Some(FieldNode {
            shape: NodeShape::Combine(op),
        }) = world.get::<FieldNode>(*one)
    {
        return chips(Some(match op {
            Op::Union(_) => 0,
            Op::Difference(_) => 1,
            Op::Intersection(_) => 2,
        }));
    }
    let siblings = selected.len() >= 2
        && selected
            .iter()
            .all(|e| world.get::<FieldNode>(*e).is_some())
        && selected
            .iter()
            .map(|e| world.get::<bevy_ecs::hierarchy::ChildOf>(*e).map(|c| c.0))
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == 1;
    if siblings { chips(None) } else { Vec::new() }
}
