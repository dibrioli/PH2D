//! ⭐ **A ponte com a CENA**: cada cilindro, cada caixa e cada operação é uma **entidade**.
//!
//! # A história curta deste arquivo
//!
//! - **W1** criou o componente e provou por gate que ele atravessa o snapshot — e **ninguém o
//!   produzia**. O gate media a metade errada: que o componente *sobrevive*, nunca que alguma coisa
//!   o *põe* no mundo. Enio, 2026-08-19: *"os objetos não aparecem na hierarchy"*.
//! - **W4** pôs a peça no mundo — como **um** objeto, com a árvore inteira escondida dentro dele.
//!   Enio, no mesmo dia: *"na hierarchy apenas um objeto e não 3 cilindro. Não há gizmo 3d para
//!   mover os objetos."* As duas frases são **um** defeito: um objeto que a cena não enumera não
//!   tem pose que um gizmo agarre.
//! - **W5** (aqui) faz a **hierarquia da cena ser a árvore de modelagem**, e o documento que o
//!   traçador avalia passa a ser **cozido** dela a cada quadro (`ph2d_field_ecs::cook`).
//!
//! ⚠️ **O MUNDO é a verdade.** O `Smoke::doc` é um cache do quadro para a thread do traçado, que
//! precisa de uma cópia própria de qualquer forma.

use ph2d_ecs::SimWorld;
use ph2d_field::{Blend, FieldDoc, NodeShape, Op, Primitive};
use ph2d_field_ecs::{FieldNode, FieldObject};

use crate::field3d_smoke::with_smoke;

/// O nome da peça na Hierarquia. É **conteúdo** (um `Name` que o artista renomeia), não chrome —
/// por isso não passa pelo i18n. Ver `ph2d_field_ecs::shape_name`.
const PART_NAME: &str = "Model";

/// Corre uma vez por quadro, antes do traçado. No-op silencioso quando o módulo não está armado.
/// **O que o shell tem de fazer à seleção** depois de a ponte correr.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectRequest {
    Entity(u64),
    /// O clique caiu no fundo. ⚠️ Limpar é a resposta certa e é o que todo modelador faz — a
    /// alternativa (manter a seleção) deixaria o gizmo aceso em cima de nada.
    Clear,
}

/// Devolve **um pedido de seleção** quando há um: um clique na peça, ou a peça a nascer.
pub(crate) fn ecs_bridge(
    sim: &mut SimWorld,
    selected: Option<u64>,
    extras: &[u64],
) -> Option<SelectRequest> {
    // ⚠️ **A semente é TIRADA**, não copiada: ela vale uma vez. Oferecer o documento cozido no
    // lugar dela — que era o que estava aqui — fazia apagar a peça na Hierarquia **replantá-la** no
    // quadro seguinte, porque a ponte não encontrava raiz e semeava o que tinha acabado de cozer.
    let (seed, ms, pending, pick) = with_smoke(|s| {
        (
            s.seed.take(),
            s.last_trace_ms,
            s.pending_move.take(),
            s.pending_pick.take(),
        )
    })?;
    // ⭐ **O arrasto do gizmo entra AQUI**, antes do retrato e do cozimento, pela mesma razão que os
    // intents do painel: o mundo é a verdade e este é o único sítio que a escreve.
    if let Some((bits, motion)) = pending {
        let entity = bevy_ecs::entity::Entity::from_bits(bits);
        let world = sim.world_mut();
        match motion {
            crate::field3d_gizmo::Motion::Translate(d) => {
                ph2d_field_ecs::translate_world(world, entity, d);
            }
            crate::field3d_gizmo::Motion::Rotate { axis, angle } => {
                ph2d_field_ecs::rotate_world(world, entity, axis, angle);
            }
            crate::field3d_gizmo::Motion::Scale(f) => {
                ph2d_field_ecs::scale_by(world, entity, f);
            }
        }
    }
    let chosen: Vec<bevy_ecs::entity::Entity> = selected
        .iter()
        .chain(extras.iter())
        .map(|b| bevy_ecs::entity::Entity::from_bits(*b))
        .collect();
    let (cooked, born) = sync_scene_and_birth(sim, seed.as_ref(), &chosen, ms);
    // ⭐ **O clique é resolvido AQUI**, e não no ponteiro: a pergunta *"de quem é este ponto?"*
    // precisa do mundo, e o ponteiro corre fora do quadro.
    //
    // ⚠️ Ele ganha do pedido de nascimento: uma escolha do artista sobrepõe-se sempre a um default,
    // e os dois só coincidem no primeiro quadro de uma peça — onde a ordem errada faria o primeiro
    // clique não pegar.
    let picked = pick.and_then(|px| resolve_pick(sim, cooked.as_ref(), px));
    let anchor = anchor_for(sim, selected);
    with_smoke(|s| {
        s.gizmo = anchor;
        // ⚠️ Só se escreve quando MUDOU: atribuir todo quadro faria o documento parecer novo e
        // re-traçar para sempre, matando o "só se traça o que mudou".
        if s.doc != cooked {
            s.doc = cooked;
        }
    });
    picked.or(born)
}

/// Resolve um clique guardado: `Some(Entity)` no que estiver sob ele, `Some(Clear)` no fundo.
fn resolve_pick(sim: &mut SimWorld, doc: Option<&FieldDoc>, px: [f32; 2]) -> Option<SelectRequest> {
    let (cam, area) = with_smoke(|s| (s.cam, s.area))?;
    let area = area?;
    let doc = doc?;
    let screen = ph2d_field_render::Screen::new(
        area.w.round().max(1.0) as u32,
        area.h.round().max(1.0) as u32,
        cam.half_extent,
    );
    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e)?;
    Some(
        crate::field3d_pick::node_under(world, root, doc, &cam, screen, px)
            .map_or(SelectRequest::Clear, |e| SelectRequest::Entity(e.to_bits())),
    )
}

/// O que a ponte **faz**, separado de **se** ela corre.
///
/// ⚠️ A separação existe para o gate: `ecs_bridge` pergunta pelo estado do smoke, e um teste não
/// consegue (nem deve) encená-lo. Aqui a peça inicial entra por parâmetro, e o resto é o caminho de
/// produção inteiro — mundo, entidades, intents, retrato, cozimento.
///
/// Devolve `None` quando não há geometria nenhuma: apagar o último filho de uma peça na Hierarquia
/// é um gesto normal, e o resultado normal dele é a tela ficar vazia.
#[cfg(test)]
pub(crate) fn sync_scene(
    sim: &mut SimWorld,
    seed: Option<&FieldDoc>,
    last_trace_ms: f32,
) -> Option<FieldDoc> {
    sync_scene_and_birth(sim, seed, &[], last_trace_ms).0
}

/// A mesma coisa, mais **quem selecionar quando a peça acaba de nascer**.
///
/// ⭐ *Feature nova = auto-play*: um gizmo que só aparece depois de o artista adivinhar que tem de
/// clicar numa linha da Hierarquia é um gizmo que a maioria nunca vê. Ao nascer, a peça seleciona o
/// **primeiro filho** — um objeto de verdade, com setas em cima dele —, e não a raiz, que é o grupo
/// inteiro. Uma vez, e só nessa: re-selecionar todo quadro tiraria da mão do artista o direito de
/// escolher outro.
pub(crate) fn sync_scene_and_birth(
    sim: &mut SimWorld,
    seed: Option<&FieldDoc>,
    selection: &[bevy_ecs::entity::Entity],
    last_trace_ms: f32,
) -> (Option<FieldDoc>, Option<SelectRequest>) {
    let mut born = None;
    let world = sim.world_mut();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &FieldObject)>();
    let root = match q.iter(world).next().map(|(e, _)| e) {
        Some(e) => e,
        // A primeira vez: a semente explode em objetos. Depois disto a **cena** é a fonte, e a
        // semente já não existe — ver `Smoke::seed`. Sem raiz e sem semente não há peça, e é isso
        // que faz apagar a peça na Hierarquia apagá-la de verdade.
        None => {
            let Some(doc) = seed else {
                return (None, None);
            };
            let root = ph2d_field_ecs::spawn_doc(world, doc, PART_NAME);
            born = Some(
                world
                    .get::<bevy_ecs::hierarchy::Children>(root)
                    .and_then(|c| c.iter().copied().next())
                    .unwrap_or(root)
                    .to_bits(),
            );
            root
        }
    };

    // ⭐ O que um gesto de criar/combinar acabou de fazer nascer — e que passa a estar selecionado.
    let mut created: Option<u64> = None;
    // Uma seleção que tem de ser LIMPA — o que ela apontava deixou de existir.
    let mut cleared = false;
    // A câmera é o «onde estou a olhar»: uma forma nova nasce no centro do quadro e no tamanho dele.
    let cam = with_smoke(|s| s.cam).unwrap_or_default();
    // As edições do painel escrevem no COMPONENTE do nó, que é a peça de verdade.
    for intent in ph2d_panel_model3d::drain_intents() {
        match intent {
            // ⭐ O verbo do gizmo é estado de VISTA: ele não entra no mundo, entra no smoke.
            ph2d_panel_model3d::ModelIntent::SetGizmoMode { slot } => {
                if let Some(mode) = crate::field3d_gizmo::Mode::ALL.get(slot).copied() {
                    with_smoke(|s| {
                        s.gizmo_mode = mode;
                        // Trocar de verbo com uma alça agarrada deixaria um arrasto órfão.
                        s.drag = None;
                        s.gizmo_hot = None;
                    });
                }
            }
            // ⭐ **Criar** — perto do que está selecionado, no tamanho do enquadramento.
            ph2d_panel_model3d::ModelIntent::AddShape { slot } => {
                if let Some(prim) = shape_at(slot, new_shape_size(cam.half_extent)) {
                    let parent = where_to_add(world, root, selection.first().map(|e| e.to_bits()));
                    if let Ok(e) = ph2d_field_ecs::add_leaf(world, parent, prim, cam.target) {
                        // ⭐ A forma nova fica SELECIONADA: é o que põe o gizmo em cima dela sem
                        // ninguém ter de a procurar na Hierarquia.
                        created = Some(e.to_bits());
                    }
                }
            }
            // ⭐ **Duplicar e apagar** — as duas ações sobre o objeto escolhido.
            ph2d_panel_model3d::ModelIntent::Act { slot } => {
                if let Some(&one) = selection.first() {
                    match slot {
                        0 => created = duplicate_node(world, one),
                        // ⚠️ O que foi apagado não pode continuar selecionado: o gizmo ficaria
                        // aceso sobre uma entidade que já não existe.
                        1 if ph2d_field_ecs::remove(world, one) => cleared = true,
                        _ => {}
                    }
                }
            }
            // ⭐ **Combinar** — trocar a operação de uma, ou embrulhar as escolhidas numa nova.
            ph2d_panel_model3d::ModelIntent::ApplyOp { slot } => {
                if let Some(op) = op_at(slot) {
                    match selection {
                        [one] => {
                            let _ = ph2d_field_ecs::set_op(world, *one, op);
                        }
                        many => {
                            if let Some(group) = ph2d_field_ecs::wrap_in_op(world, many, op) {
                                created = Some(group.to_bits());
                            }
                        }
                    }
                }
            }
            // O referencial dos eixos é estado de VISTA, como o verbo.
            ph2d_panel_model3d::ModelIntent::SetGizmoFrame { slot } => {
                if let Some(frame) = crate::field3d_gizmo::Frame::ALL.get(slot).copied() {
                    with_smoke(|s| s.gizmo_frame = frame);
                }
            }
            ph2d_panel_model3d::ModelIntent::SetParam {
                entity,
                index,
                value,
            } => {
                // Uma recusa é informação, não erro: o nó diz que aquele número não cabe, e o
                // retrato publicado logo abaixo devolve o controle ao valor que ficou.
                let _ = ph2d_field_ecs::set_dim(
                    world,
                    bevy_ecs::entity::Entity::from_bits(entity),
                    index,
                    value,
                );
            }
        }
    }

    // ⭐ O alcance do gesto é **o que cabe no quadro**: uma dimensão maior do que ele é uma cujo
    // efeito não se vê. O campo numérico continua sem teto, porque digitar 1000 é uma afirmação
    // sobre a peça e não sobre a janela.
    publish_snapshot(world, root, selection, cam.half_extent * 2.0, last_trace_ms);
    // ⚠️ Uma peça inválida (um raio que deixou de caber porque a escala do pai mudou) devolve
    // `None` aqui, e a tela mostra o que o cozimento **de facto** produziu. Guardar o último
    // documento válido faria a tela mentir sobre a cena — que é exatamente o defeito que este
    // módulo acabou de pagar no cache do traçado.
    (
        ph2d_field_ecs::cook(world, root).and_then(Result::ok),
        // ⚠️ **A ordem é a das intenções mais recentes.** O que acabou de nascer ganha do
        // nascimento da peça (os dois só coincidem no primeiro quadro, e ali a ordem errada faria a
        // primeira forma criada não ficar selecionada); e um apagar sem nada novo pede a limpeza,
        // senão o gizmo ficaria aceso sobre uma entidade que já não existe.
        created
            .map(SelectRequest::Entity)
            .or_else(|| cleared.then_some(SelectRequest::Clear))
            .or_else(|| born.map(SelectRequest::Entity)),
    )
}

/// **A ponte com o painel**: publica o retrato da peça.
///
/// ⭐ **A ordem é load-bearing.** Drenar ANTES de publicar é o que faz a edição aparecer no mesmo
/// quadro: se o retrato saísse primeiro, o painel pintaria o valor antigo por um quadro e o
/// controle daria um salto para trás debaixo do dedo — o sintoma clássico de um espelho publicado
/// cedo demais.
fn publish_snapshot(
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
        acts,
        rows,
        node_count: all.len(),
        last_trace_ms: ms,
    });
}

/// ⭐ **Onde o gizmo tem de aparecer** — a pose de MUNDO do nó selecionado.
///
/// ⚠️ A seleção é a do **app** (`hero.gizmo.selection`), e não uma deste módulo: clicar numa linha
/// da Hierarquia é o gesto que faz as setas aparecerem. Uma seleção própria seria uma segunda ideia
/// de *"o que está selecionado"* dentro do mesmo aplicativo, e as duas divergiriam no primeiro
/// clique.
///
/// Devolve `None` quando o selecionado não é um nó de modelagem — um sprite selecionado não pode
/// fazer aparecer um gizmo 3D em cima dele.
fn anchor_for(sim: &mut SimWorld, selected: Option<u64>) -> Option<crate::field3d_gizmo::Anchor> {
    let bits = selected?;
    let frame = with_smoke(|s| s.gizmo_frame).unwrap_or_default();
    let entity = bevy_ecs::entity::Entity::from_bits(bits);
    let world = sim.world_mut();
    world.get::<FieldNode>(entity)?;
    let pose = ph2d_field_ecs::world_xform(world, entity);
    Some(crate::field3d_gizmo::Anchor {
        entity: bits,
        origin: pose.translation,
        // ⚠️ Os eixos viajam **já resolvidos**: a lei do gizmo não sabe que existe uma escolha de
        // referencial, e quem a faz é quem tem a pose. Ver `Anchor::axes`.
        axes: frame.axes(pose.rotation),
    })
}

/// ⭐ **As dimensões do objeto selecionado** — o painel é o inspetor da seleção.
///
/// ⚠️ **Mudou de forma na W10.** Antes era uma linha por nó com o raio dele — uma segunda vista da
/// estrutura, a competir com a Hierarquia e sem onde pôr as outras dimensões. A divisão passou a
/// ser a da casa: a Hierarquia mostra **o que existe**, o painel mostra **os números do escolhido**.
///
/// `view_span` é o alcance do **gesto** (ver [`Bound`]): uma largura de caixa não tem teto físico, e
/// quem escolhe até onde o slider vai é a vista — o que cabe no enquadramento. O documento só
/// contribui as **paredes** (um filete que não cabe).
fn param_rows(
    world: &bevy_ecs::world::World,
    selected: Option<bevy_ecs::entity::Entity>,
    view_span: f32,
) -> Vec<ph2d_panel_model3d::ParamRow> {
    let Some(e) = selected else {
        return Vec::new();
    };
    let Some(node) = world.get::<FieldNode>(e) else {
        return Vec::new();
    };
    // ⚠️ O valor E o teto vêm os DOIS do nó. Um painel que guardasse o seu próprio valor teria duas
    // verdades sobre o mesmo número, e a que aparece na tela seria a errada sempre que algo o
    // mudasse de outro lado — um desfazer, um arquivo aberto, o gizmo.
    let row = |index: usize, key: &'static str, value: f32, limit: Option<f32>| {
        ph2d_panel_model3d::ParamRow {
            entity: e.to_bits(),
            index,
            key,
            value,
            bound: limit.map_or(ph2d_field::Bound::Soft(view_span), ph2d_field::Bound::Hard),
        }
    };
    match &node.shape {
        // ⚠️ Uma operação tem **uma** dimensão: o raio da mistura. Ela entra pela mesma porta das
        // outras (`set_dim`, índice 0), senão haveria dois caminhos de escrita a divergir.
        NodeShape::Combine(_) => ph2d_field_ecs::radius_of(world, e)
            .map(|v| {
                vec![row(
                    0,
                    "field.dim.round",
                    v,
                    match ph2d_field_ecs::radius_bound(world, e) {
                        Some(ph2d_field::Bound::Hard(h)) => Some(h),
                        _ => None,
                    },
                )]
            })
            .unwrap_or_default(),
        NodeShape::Leaf(_) => ph2d_field_ecs::dims_of(world, e)
            .into_iter()
            .enumerate()
            .map(|(i, d)| row(i, d.key, d.value, d.limit))
            .collect(),
    }
}

#[cfg(test)]
#[path = "field3d_scene_tests.rs"]
mod tests;

/// ⭐ **As formas que se podem acrescentar**, na ordem do seletor.
///
/// ⚠️ **É a fonte da contagem**, como o `Mode::ALL`: acrescentar uma primitiva aqui faz o painel
/// seguir sem uma linha de mudança.
const SHAPES: [&str; 4] = [
    "panel.model3d.add.box",
    "panel.model3d.add.sphere",
    "panel.model3d.add.cylinder",
    "panel.model3d.add.torus",
];

/// As ações sobre o objeto escolhido, na ordem do seletor.
const ACTS: [&str; 2] = ["panel.model3d.act.duplicate", "panel.model3d.act.delete"];

/// As três booleanas, na ordem do seletor.
const OPS: [&str; 3] = [
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
fn new_shape_size(half_extent: f32) -> f32 {
    (half_extent * 0.25).max(f32::MIN_POSITIVE)
}

/// A primitiva que cada posição do seletor cria, no tamanho do enquadramento.
///
/// ⚠️ **Todas nascem com o `round` que têm direito**, e não a zero: este é o módulo cujo argumento é
/// o arredondamento, e uma caixa de aresta viva ao nascer esconderia exatamente aquilo que ele faz
/// melhor do que o Blender. O valor é uma fração do tamanho, então ele cabe sempre.
fn shape_at(slot: usize, r: f32) -> Option<Primitive> {
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

fn op_at(slot: usize) -> Option<Op> {
    Some(match slot {
        0 => Op::Union(Blend::Sharp),
        1 => Op::Difference(Blend::Sharp),
        2 => Op::Intersection(Blend::Sharp),
        _ => return None,
    })
}

/// ⭐ **Duplicar um nó** — a porta ÚNICA, e os dois lugares que duplicam chamam-na.
///
/// ⚠️ **Uma lei, dois chamadores**: o botão do painel e a linha *Duplicate* da Hierarquia. Cada um
/// com a sua conta seria a segunda resposta a *"onde vai a cópia?"*, e elas divergiriam no primeiro
/// ajuste — com o artista a ver o mesmo gesto fazer duas coisas conforme por onde o pediu. É a mesma
/// lição que o bloco vetorial da Hierarquia já tem escrita ao lado.
///
/// # A cópia sai UM DEGRAU da grelha, para a direita da TELA
///
/// ⚠️ Não é decoração, e a alternativa foi considerada: **duplicar em cima do original** é o que o
/// Blender faz — e ele resolve o resto entrando logo em modo de mover. Aqui não há esse modo, então
/// uma cópia exatamente por baixo seria **um botão que parece não fazer nada**: a única prova seria
/// uma linha nova na Hierarquia.
///
/// O **quanto** é o degrau da grelha (derivado do enquadramento: o menor número redondo que ainda se
/// consegue mirar); o **para onde** é a direita da câmera, que é para onde «o próximo» vai em
/// qualquer arrumação.
///
/// Devolve os bits da cópia, para quem chamar a poder selecionar. `None` quando não há o que
/// duplicar (ver `ph2d_field_ecs::duplicate`: a raiz **é** a peça).
pub(crate) fn duplicate_node(
    world: &mut bevy_ecs::world::World,
    node: bevy_ecs::entity::Entity,
) -> Option<u64> {
    let (cam, screen) = view()?;
    duplicate_with_view(world, node, &cam, screen)
}

/// A mesma lei, **com a vista em mãos** — e é a separação que o resto do módulo já usa.
///
/// ⚠️ Ela existe para o gate: [`duplicate_node`] lê a câmera do estado do módulo, e um teste não
/// consegue (nem deve) encená-lo. Aqui a vista entra por parâmetro e o resto é o caminho de
/// produção inteiro.
pub(crate) fn duplicate_with_view(
    world: &mut bevy_ecs::world::World,
    node: bevy_ecs::entity::Entity,
    cam: &ph2d_field_render::Orbit,
    screen: ph2d_field_render::Screen,
) -> Option<u64> {
    let (right, _, _) = cam.basis();
    let step = crate::field3d_gizmo::snap_step(screen);
    let off = [right[0] * step, right[1] * step, right[2] * step];
    ph2d_field_ecs::duplicate(world, node, off).map(|e| e.to_bits())
}

/// A câmera e o enquadramento deste quadro — `None` quando o módulo não está armado.
fn view() -> Option<(ph2d_field_render::Orbit, ph2d_field_render::Screen)> {
    with_smoke(|s| {
        let a = s
            .area
            .unwrap_or(ph2d_editor::zones::Rect::new(0.0, 0.0, 1.0, 1.0));
        (
            s.cam,
            ph2d_field_render::Screen::new(
                a.w.round().max(1.0) as u32,
                a.h.round().max(1.0) as u32,
                s.cam.half_extent,
            ),
        )
    })
}

/// **Onde uma forma nova entra** — perto do que está selecionado.
///
/// ⚠️ Uma operação selecionada adota-a; uma folha selecionada ganha-a como **irmã**, e não como
/// filha (uma folha não tem filhos, e pendurar uma forma numa esfera não quer dizer nada). Sem
/// seleção, ela vai para a raiz.
fn where_to_add(
    world: &bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
    selected: Option<u64>,
) -> bevy_ecs::entity::Entity {
    let Some(e) = selected.map(bevy_ecs::entity::Entity::from_bits) else {
        return root;
    };
    match world.get::<FieldNode>(e).map(|n| &n.shape) {
        Some(NodeShape::Combine(_)) => e,
        Some(NodeShape::Leaf(_)) => world
            .get::<bevy_ecs::hierarchy::ChildOf>(e)
            .map_or(root, |c| c.0),
        None => root,
    }
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
fn ops_for(
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
