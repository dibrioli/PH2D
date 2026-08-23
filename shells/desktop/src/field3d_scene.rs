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
    let chosen: Vec<bevy_ecs::entity::Entity> = selected
        .iter()
        .chain(extras.iter())
        .map(|b| bevy_ecs::entity::Entity::from_bits(*b))
        .collect();
    // ⭐ **O arrasto do gizmo entra AQUI**, antes do retrato e do cozimento, pela mesma razão que os
    // intents do painel: o mundo é a verdade e este é o único sítio que a escreve.
    if let Some((bits, motion)) = pending {
        apply_motion(sim, bits, &chosen, motion);
    }
    let (cooked, born) = sync_scene_and_birth(sim, seed.as_ref(), &chosen, ms);
    // ⭐ **O clique é resolvido AQUI**, e não no ponteiro: a pergunta *"de quem é este ponto?"*
    // precisa do mundo, e o ponteiro corre fora do quadro.
    //
    // ⚠️ Ele ganha do pedido de nascimento: uma escolha do artista sobrepõe-se sempre a um default,
    // e os dois só coincidem no primeiro quadro de uma peça — onde a ordem errada faria o primeiro
    // clique não pegar.
    let picked = pick.and_then(|px| resolve_pick(sim, cooked.as_ref(), px));
    let anchor = anchor_for(sim, selected, &chosen);
    with_smoke(|s| {
        s.gizmo = anchor;
        // ⚠️ Só se escreve quando MUDOU: atribuir todo quadro faria o documento parecer novo e
        // re-traçar para sempre, matando o "só se traça o que mudou".
        if s.doc != cooked {
            s.doc = cooked;
        }
        // ⭐⭐ **A peça de um documento novo nasce ENQUADRADA** (W46) — e é AQUI, depois de o
        // documento estar cozido, porque enquadrar precisa do bordo dele. O pedido fica de pé até
        // ser servido: no quadro do load o módulo pode nem estar armado.
        if crate::field3d_smoke::wants_frame() && crate::field3d_input::frame_the_part(s) {
            crate::field3d_smoke::served_frame();
            s.manual = true;
        }
    });
    picked.or(born)
}

/// ⭐ **O gesto vale para a SELEÇÃO INTEIRA** (W27), em torno do pivô que o gizmo mostra.
///
/// ⚠️ **O defeito que isto fecha:** a seleção deste módulo é a do app (clicar na Hierarquia com
/// `Ctrl` escolhe vários, e a fileira de operações já contava com isso desde a W9) — e o arrasto
/// movia **um**. Duas linhas acesas, uma a andar: o artista lê aquilo como o gizmo estar partido.
///
/// ⚠️ **O pivô é o do GIZMO, não o de cada nó**, e é o que faz um giro girar o conjunto em vez de
/// cada peça sobre si mesma. Com um nó só, o pivô **é** a origem dele e as duas leis coincidem
/// byte-a-byte (ver [`ph2d_field_ecs::rotate_world_about`]) — não há caso especial.
fn apply_motion(
    sim: &mut SimWorld,
    primary: u64,
    chosen: &[bevy_ecs::entity::Entity],
    motion: crate::field3d_gizmo::Motion,
) {
    let world = sim.world_mut();
    let primary = bevy_ecs::entity::Entity::from_bits(primary);
    // ⚠️ **Quem está agarrado entra sempre**, mesmo que a seleção do app já não o contenha: o gesto
    // foi começado nele, e o `Grip` congelou-o.
    let mut all: Vec<bevy_ecs::entity::Entity> = vec![primary];
    all.extend(chosen.iter().copied().filter(|e| *e != primary));
    // ⚠️ E um filho de outro escolhido **não anda duas vezes** — ver `top_level`.
    let targets: Vec<bevy_ecs::entity::Entity> = ph2d_field_ecs::top_level(world, &all)
        .into_iter()
        .filter(|e| movable(world, *e))
        .collect();
    let pivot = selection_pivot(world, &targets);
    for e in targets {
        match motion {
            crate::field3d_gizmo::Motion::Translate(d) => {
                ph2d_field_ecs::translate_world(world, e, d);
            }
            crate::field3d_gizmo::Motion::Rotate { axis, angle } => {
                ph2d_field_ecs::rotate_world_about(world, e, axis, angle, pivot);
            }
            crate::field3d_gizmo::Motion::Scale(f) => {
                ph2d_field_ecs::scale_about(world, e, f, pivot);
            }
        }
    }
}

/// A mesma porta, aberta para os gates da W27 — o caminho real (`ecs_bridge`) pergunta pelo estado
/// do smoke, que um teste não encena.
#[cfg(test)]
pub(crate) fn apply_motion_for_test(
    sim: &mut SimWorld,
    primary: u64,
    chosen: &[bevy_ecs::entity::Entity],
    motion: crate::field3d_gizmo::Motion,
) {
    apply_motion(sim, primary, chosen, motion);
}

#[cfg(test)]
#[path = "field3d_selection_tests.rs"]
mod selection_tests;

#[cfg(test)]
#[path = "field3d_group_tests.rs"]
mod group_tests;

#[cfg(test)]
#[path = "field3d_reach_tests.rs"]
mod reach_tests;

#[cfg(test)]
#[path = "field3d_isolate_tests.rs"]
mod isolate_tests;

/// ⭐ **DE ONDE se coze** — a peça inteira, ou só o nó isolado (W38).
///
/// # Isolar não precisou de lei nenhuma
///
/// O [`ph2d_field_ecs::cook`] já dizia, desde a W5, *"coze a **subárvore** de `root`"*. Isolar **é**
/// cozer a partir daquele nó: os irmãos ficam de fora porque a travessia nunca chega a eles, e a
/// operação que os juntava fica de fora porque ela está acima. *A feature era uma generalização de
/// uma função que já existia, não maquinaria nova.*
///
/// ⚠️ **A pose da cadeia acima entra no nó de topo** (a linha que a W38 acrescentou ao `cook`) —
/// senão isolar um nó dentro de um grupo deslocado atirava a peça para a origem, e da cadeira isso
/// lê como *"isolar mexeu no meu modelo"*.
///
/// ⚠️ **O isolamento é CONFERIDO, não obedecido.** Ele guarda bits de entidade, e os bits morrem num
/// undo (o restore respawna tudo com ids novos). Um isolamento pendurado numa entidade morta
/// apagaria a peça da tela sem nada a explicar — então um alvo que já não é um nó de campo é
/// **largado**, e a peça inteira volta.
pub(crate) fn cook_root(
    world: &bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
    isolated: Option<u64>,
) -> bevy_ecs::entity::Entity {
    let Some(bits) = isolated else {
        return root;
    };
    let node = bevy_ecs::entity::Entity::from_bits(bits);
    if world.get::<FieldNode>(node).is_some() {
        node
    } else {
        crate::field3d_smoke::forget_isolation();
        crate::field3d_notice::say("Isolation dropped: that object is gone".into());
        root
    }
}

/// ⭐ **Este nó pode ser mexido por um gesto?** — a pergunta única, e ela junta duas leis da CASA.
///
/// | lei | de onde vem | o que significa aqui |
/// |---|---|---|
/// | **escondido** (W28) | o olho da Hierarquia ([`ph2d_ecs::Visibility`]) | mover o que a peça não mostra é um gesto sem resposta na tela |
/// | **trancado** (W29) | o cadeado ([`ph2d_ecs::is_locked_for_edit`]) | *"Cadeado trava apenas o objeto"* — Enio, 2026-05-26, escrito no doc do componente |
///
/// ⚠️ **O predicado do cadeado é o da casa, não um novo**: ele já é consultado pelo gizmo 2D, pelo
/// Flip, pelas juntas e pelo vetorial — e ele **sobe a cadeia** à procura de um antepassado com
/// `GroupedChildren`, que é o que trancar um grupo significa. Escrever aqui um `get::<Locked>` seria
/// a segunda resposta a *"isto pode mexer?"*, e ela nasceria já sem a metade do grupo.
fn movable(world: &bevy_ecs::world::World, e: bevy_ecs::entity::Entity) -> bool {
    !ph2d_field_ecs::is_hidden(world, e) && !ph2d_ecs::is_locked_for_edit(world, e)
}

/// **Onde o gizmo pousa numa seleção**: a média das origens de mundo dos escolhidos.
///
/// ⚠️ A média das ORIGENS, e não o centro das caixas: a caixa de um campo implícito custa uma
/// varredura, e o que o artista agarra é o que ele vê — as setas estão sobre as origens.
fn selection_pivot(
    world: &bevy_ecs::world::World,
    targets: &[bevy_ecs::entity::Entity],
) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    let mut n = 0.0f32;
    for e in targets {
        let t = ph2d_field_ecs::world_xform(world, *e).translation;
        for k in 0..3 {
            sum[k] += t[k];
        }
        n += 1.0;
    }
    if n == 0.0 {
        return [0.0; 3];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
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
/// ⭐ **O mundo traz uma peça de modelagem — há alguma coisa PARA VER?** (W45)
///
/// ⚠️ Ela não abre nada: devolve um **fato**. Quem a consome é o quadro, porque o mundo vive no
/// `gfx` e o load corre sem janela (ver `field3d_smoke::ask_open_panel_if_part`).
///
/// # ⚠️ A pergunta não é «há uma raiz», e também não é «há um nó»
///
/// As duas primeiras versões desta função perguntaram isso, e uma **prova de mutação** mostrou que
/// eram a mesma pergunta: o [`ph2d_field_ecs::spawn_doc`] dá `FieldNode` à raiz **sempre** (ela nasce
/// nó e recebe o `FieldObject` depois), então *"há nó"* nunca difere de *"há raiz"* — e o comentário
/// que eu tinha escrito ao lado afirmava que difere. *Uma condição cujo caso distintivo é
/// inalcançável é peso morto com uma nota a mentir.*
///
/// ⭐ O que separa de facto os dois casos é o **cozimento**: apagar os filhos deixa a raiz de pé, e
/// ela coze para `None` — geometria nenhuma. Abrir um painel para isso é ocupar o encaixe da direita
/// para não mostrar coisa alguma.
///
/// ⚠️ **`Some(Err)` conta como peça**, de propósito: uma peça que não cozinha é exactamente quando o
/// artista mais precisa do painel — é lá que o módulo diz **porquê** (W25).
///
/// ⚠️ `&mut World` para uma leitura porque `World::query` o exige — não porque ela escreva.
pub(crate) fn world_has_a_part(world: &mut bevy_ecs::world::World) -> bool {
    let mut q = world.query::<(bevy_ecs::entity::Entity, &FieldObject)>();
    let roots: Vec<bevy_ecs::entity::Entity> = q.iter(world).map(|(e, _)| e).collect();
    roots
        .into_iter()
        .any(|r| ph2d_field_ecs::cook(world, r).is_some())
}

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
    // ⚠️ Ele entra no drenar das intenções e sai de lá atualizado: a importação de escultura (logo
    // abaixo) e um botão do painel podem criar no MESMO quadro, e o mais recente ganha.
    let mut created: Option<u64> = None;
    // A câmera é o «onde estou a olhar»: uma forma nova nasce no centro do quadro e no tamanho dele.
    let cam = with_smoke(|s| s.cam).unwrap_or_default();

    // ⭐ **A escultura que o diálogo carregou vira NÓ aqui** — o terceiro salto do pedido de
    // importar (ADR-0161 W22). Ela chega pelo nome; o campo já está no registo.
    //
    // ⚠️ **A escala vem do ENQUADRAMENTO e não do arquivo**, e vai para a POSE: o campo foi
    // construído nas unidades do autor de propósito — é isso que faz a célula da grade ser a
    // resolução real dele —, e um clique desfaz a pose sem tocar na geometria.
    // ⭐⭐ **A forma de PERFIL, cozida pelo shell, vira nó** (W53) — o terceiro salto, ao lado do
    // da escultura e pela mesma razão: quem tem a cena vetorial não tem o mundo.
    //
    // ⚠️ **A escala vem do ENQUADRAMENTO e vai para a POSE**, como na escultura: o perfil é
    // construído nas unidades em que foi **desenhado**, e um clique desfaz a pose sem tocar na
    // geometria.
    if let Some((prim, extent)) = crate::field3d_smoke::take_pending_profile() {
        let parent = where_to_add(world, root, selection.first().map(|e| e.to_bits()));
        if let Ok(e) = ph2d_field_ecs::add_leaf(world, parent, prim, cam.target) {
            let s = crate::field3d_import::framing_scale(extent, cam.half_extent);
            let _ = ph2d_field_ecs::set_param(world, e, ph2d_field::Param::Scale, s);
            created = Some(e.to_bits());
        }
    }
    if let Some(key) = crate::field3d_smoke::take_pending_sculpt() {
        let parent = where_to_add(world, root, selection.first().map(|e| e.to_bits()));
        if let Ok(e) = ph2d_field_ecs::add_sampled(world, parent, &key, cam.target) {
            if let Some(extent) = crate::field3d_smoke::take_sculpt_extent() {
                let s = crate::field3d_import::framing_scale(extent, cam.half_extent);
                let _ = ph2d_field_ecs::set_param(world, e, ph2d_field::Param::Scale, s);
            }
            created = Some(e.to_bits());
        }
    }
    // ⭐ **A TECLA do isolamento** (W44). ⚠️ Drenada aqui, e não no `intents`, porque **não é um
    // pedido do painel**: a lei dela é global (`key_isolation`) e o que ela precisa é da seleção,
    // que esta função tem. A voz é a mesma do chip — um só canal, como sempre.
    if crate::field3d_smoke::take_isolate_key_request() {
        let on =
            crate::field3d_smoke::toggle_isolate_by_key(selection.first().map(|e| e.to_bits()));
        crate::field3d_notice::say(if on.is_some() {
            "Isolated: showing only this object (Shift+I brings the part back)".into()
        } else {
            "Isolation off: the whole part is back".into()
        });
    }
    // ⭐ **O que o painel PEDE** vive no irmão — ver [`field3d_scene_intents`](self::intents).
    let (created, cleared) = intents::apply(world, root, selection, cam, created);
    // ⭐ **SÓ UMA OPERAÇÃO PODE TER FILHOS** (W31), e a lei impõe-se aqui — na derivação, não em
    // cada gesto.
    //
    // ⚠️ Enio, 2026-08-22: *"Se coloco um objeto como filho do outro ele some."* E somia mesmo: o
    // cozimento emite uma **forma** e nunca olha para os filhos dela, então o nó largado ali ficava
    // no mundo, aparecia na Hierarquia, e não entrava em documento nenhum. Reparar **onde a árvore
    // é lida** apanha o arrasto da Hierarquia, o do futuro, e qualquer outro caminho que produza a
    // mesma forma inválida — um remendo no drenar do arrasto deixaria os outros de fora.
    ph2d_field_ecs::promote_leaf_hosts(world, root);

    // ⭐ O alcance do gesto é **o que cabe no quadro**: uma dimensão maior do que ele é uma cujo
    // efeito não se vê. O campo numérico continua sem teto, porque digitar 1000 é uma afirmação
    // sobre a peça e não sobre a janela.
    publish_snapshot(world, root, selection, cam.half_extent * 2.0, last_trace_ms);
    // ⚠️ Uma peça inválida (um raio que deixou de caber porque a escala do pai mudou) devolve
    // `None` aqui, e a tela mostra o que o cozimento **de facto** produziu. Guardar o último
    // documento válido faria a tela mentir sobre a cena — que é exatamente o defeito que este
    // módulo acabou de pagar no cache do traçado.
    // ⭐ **UMA PEÇA QUE NÃO COZINHA DIZ PORQUÊ** (W25). O `Err` estava a ser deitado fora por um
    // `.and_then(Result::ok)`, e o resultado era a peça **inteira** a desaparecer da tela com a
    // Hierarquia intacta e nada a explicar — indistinguível de um problema de câmera.
    //
    // ⚠️ **`None` e `Err` são coisas diferentes e só uma é um problema**: apagar o último filho de
    // uma peça devolve `None` e é um gesto normal, cujo resultado normal é a tela ficar vazia.
    let cooked = match ph2d_field_ecs::cook(
        world,
        cook_root(world, root, crate::field3d_smoke::isolated()),
    ) {
        Some(Ok(doc)) => {
            // Voltou a estar bem: o próximo problema — mesmo que seja o mesmo — volta a ser dito.
            crate::field3d_notice::clear();
            Some(doc)
        }
        Some(Err(e)) => {
            crate::field3d_notice::say(crate::field3d_notice::explain(&e));
            None
        }
        None => None,
    };
    // ⭐ **AS ESCULTURAS QUE FALTAM VOLTAM AQUI** (W23) — colado ao cozimento, e não no load do
    // projeto, por uma razão de alcance: o documento é quem **nomeia** as esculturas, e ele nasce
    // deste `cook`. Um gancho no `project_load` cobriria o Ctrl+O e deixaria de fora tudo o resto
    // que traz um nó de volta (o undo de um apagar, um duplicate, um documento semeado) — e cada um
    // desses buracos teria o mesmo sintoma mudo.
    //
    // ⚠️ **Custa uma varredura dos nós por quadro e nada mais**: o caso normal é o registo já
    // conhecer todos os nomes, e aí `missing_keys` devolve vazio sem tocar no disco. Quem lê arquivo
    // é a **primeira** vez de cada nome (ver `field3d_reload`, e a tabela de custo no doc 06 §24).
    if let Some(doc) = cooked.as_ref() {
        crate::field3d_notice::say_all(crate::field3d_reload::resolve_missing(doc));
    }
    (
        cooked,
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
/// ⭐ **O que o painel mostra** vive no irmão — ver [`field3d_scene_panel`](self::panel).
///
/// ⚠️ **O re-export lista só o que sai do módulo.** Um `use` largo compilaria e deixaria avisos de
/// import morto a acumular, que é como uma fronteira deixa de dizer alguma coisa. Os gates alcançam
/// o resto pelo caminho completo (`panel::param_rows`) — privado é visível para quem está dentro.
#[path = "field3d_scene_intents.rs"]
mod intents;

#[path = "field3d_scene_panel.rs"]
mod panel;
pub(crate) use panel::{new_shape_size, op_at, publish_snapshot, shape_at};

/// ⭐ **Onde o gizmo tem de aparecer** — a pose de MUNDO do nó selecionado.
///
/// ⚠️ A seleção é a do **app** (`hero.gizmo.selection`), e não uma deste módulo: clicar numa linha
/// da Hierarquia é o gesto que faz as setas aparecerem. Uma seleção própria seria uma segunda ideia
/// de *"o que está selecionado"* dentro do mesmo aplicativo, e as duas divergiriam no primeiro
/// clique.
///
/// Devolve `None` quando o selecionado não é um nó de modelagem — um sprite selecionado não pode
/// fazer aparecer um gizmo 3D em cima dele.
fn anchor_for(
    sim: &mut SimWorld,
    selected: Option<u64>,
    chosen: &[bevy_ecs::entity::Entity],
) -> Option<crate::field3d_gizmo::Anchor> {
    let bits = selected?;
    let frame = with_smoke(|s| s.gizmo_frame).unwrap_or_default();
    let entity = bevy_ecs::entity::Entity::from_bits(bits);
    let world = sim.world_mut();
    world.get::<FieldNode>(entity)?;
    // ⭐ **Um nó ESCONDIDO ou TRANCADO não tem gizmo** (W28/W29): setas que não mexem em nada são um
    // gesto sem resposta na tela. A linha da Hierarquia continua lá, com o olho e o cadeado a
    // dizerem porquê — e é por eles que se desfaz.
    //
    // ⚠️ **Aqui o módulo DIVERGE do gizmo 2D da casa, e é de propósito.** Lá o desenho fica e o
    // *Down* é recusado; aqui as alças são o único sinal de que o gesto existe, e alças que não
    // agarram seriam a mesma coisa que um botão pintado e morto. O que se ganha lá — *«ele está
    // ali»* — este módulo ganha na Hierarquia, que é onde o cadeado se vê.
    if !movable(world, entity) {
        return None;
    }
    let pose = ph2d_field_ecs::world_xform(world, entity);
    // ⭐ **Com vários escolhidos, o gizmo pousa no MEIO deles** (W27) — e é esse ponto que o giro e
    // o tamanho usam como pivô. Com um só, a média é a própria origem: a lei antiga é o caso
    // particular desta, e não um ramo à parte.
    //
    // ⚠️ Os **eixos** continuam a ser os do principal: é o que mantém o seletor Global/Local a
    // significar alguma coisa numa seleção (o «Local» de um conjunto é o do objeto ativo, que é o
    // que todo modelador faz).
    let mut all: Vec<bevy_ecs::entity::Entity> = vec![entity];
    all.extend(chosen.iter().copied().filter(|e| *e != entity));
    let targets = ph2d_field_ecs::top_level(world, &all);
    Some(crate::field3d_gizmo::Anchor {
        entity: bits,
        origin: selection_pivot(world, &targets),
        // ⚠️ Os eixos viajam **já resolvidos**: a lei do gizmo não sabe que existe uma escolha de
        // referencial, e quem a faz é quem tem a pose. Ver `Anchor::axes`.
        axes: frame.axes(pose.rotation),
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
        // Uma escultura é uma FOLHA para efeitos de onde uma peça nova entra.
        Some(NodeShape::Leaf(_) | NodeShape::Sampled { .. }) => world
            .get::<bevy_ecs::hierarchy::ChildOf>(e)
            .map_or(root, |c| c.0),
        None => root,
    }
}

/// ⚠️ **Esta declaração já caiu uma vez, e a suíte ficou VERDE.** Um corte de bloco levou-a, 30
/// gates deixaram de correr, e o que apareceu no terminal foi `✓ 52 passaram` — porque um teste que
/// não é compilado não reprova, ele desaparece. *Contar é a única defesa: uma suíte que encolhe é
/// uma suíte que perdeu alguma coisa.*
#[cfg(test)]
#[path = "field3d_scene_tests.rs"]
mod tests;

/// ⚠️ **Um arquivo de gates que ninguém declara não existe** — e a suíte fica VERDE.
///
/// Aconteceu neste módulo em 20/08: um corte de bloco levou a declaração do módulo de gates da
/// cena, **30 gates deixaram de correr**, e o terminal disse `✓ 52 passaram`. Um teste que não é
/// compilado não reprova — ele desaparece.
///
/// Este gate lê o diretório e exige que cada `field3d_*_tests.rs` seja **nomeado** por algum
/// arquivo de código do módulo. Ele não prova que os gates lá dentro são bons; prova que eles
/// **correm**, que era a metade que faltava.
#[cfg(test)]
mod no_orphan_test_files {
    #[test]
    fn every_field3d_test_file_is_declared_by_a_module() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        // ⚠️ **Sem os comentários**, e esta linha é a diferença entre um gate e um enfeite. A
        // primeira versão lia o arquivo inteiro — e passava porque encontrava a declaração **no
        // doc-comment deste próprio teste**, que a cita como exemplo. Uma prova de mutação apanhou-o:
        // apagar a declaração de verdade deixava-o verde.
        //
        // *Um gate que procura texto tem de procurar CÓDIGO.*
        let sources: String = std::fs::read_dir(&dir)
            .expect("o diretório do shell existe")
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "rs"))
            .filter_map(|e| std::fs::read_to_string(e.path()).ok())
            .flat_map(|src| {
                src.lines()
                    .filter(|l| !l.trim_start().starts_with("//"))
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut orphans: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(&dir).expect("lê o diretório").flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("field3d_") || !name.ends_with("_tests.rs") {
                continue;
            }
            // Declarado por `#[path = "..."]`, ou por um `mod` do mesmo nome (sem `.rs`).
            let by_path = sources.contains(&format!("#[path = \"{name}\"]"));
            let by_mod = sources.contains(&format!("mod {};", name.trim_end_matches(".rs")));
            if !by_path && !by_mod {
                orphans.push(name);
            }
        }
        assert!(
            orphans.is_empty(),
            "estes arquivos de gates existem e NÃO correm: {orphans:?} — a suíte está verde por \
             não os compilar"
        );
    }
}
