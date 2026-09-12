//! ⭐⭐⭐ **A ÂNCORA, viva** — a restrição de cinemática inversa que corre a cada quadro.
//!
//! O arrasto da ponta (que já existia) **posa**: acaba quando o dedo levanta. Isto **persiste** — o
//! artista anima UM objecto e a corrente inteira o segue, através da timeline e do save. É o degrau
//! que separa *«um editor de esqueletos»* de *«um editor de animação»*, e é o que as quatro
//! referências entregam (ver o doc do [`ph2d_skeleton_ecs::IkGoal`], com a tabela).
//!
//! # ⚠️ A pose que este passe escreve é PRÉ-VISUALIZAÇÃO, não documento
//!
//! É a lei do [`crate::preview_drive`], e ela cabe aqui à letra: *o documento é o valor **AUTORADO**;
//! o que um motor está a escrever agora vê-se, não se guarda nem se desfaz.* O que o artista autora
//! é a pose da **ÂNCORA** (e os três números da restrição); a rotação dos ossos governados é
//! **derivada** dela. Sem o ledger, cada clique enquanto a âncora está fora do sítio empilharia um
//! passo de undo cujo conteúdo é *«o solver mexeu»* — o defeito que a auditoria da §11 mediu.
//!
//! ⚠️ **O motor é o [`ph2d_preview_drive::Driver::SolverPose`], e não um novo**: o doc dele já
//! declara que a pose escrita por um motor é *«o mesmo facto vindo de outro motor»* — é assim que a
//! física e as curvas da timeline já partilham aquela chave. Uma chave nova sobre o **mesmo
//! componente** poria dois memos a repor `Transform`s diferentes na mesma fotografia, e quem
//! ganhasse seria a ordem do `BTreeMap`.
//!
//! # O preço, MEDIDO
//!
//! `measure_the_price_of_one_frame_of_anchors`, sobre uma cena de **2000 objectos** (a ordem de um
//! documento cheio), em release:
//!
//! | cena | por quadro | de um quadro de 16,7 ms |
//! |---|---|---|
//! | sem âncora nenhuma | `16,4 µs` | **`0,098 %`** |
//! | com uma âncora viva | `65,5 µs` | **`0,392 %`** |
//!
//! ⚠️ A linha de cima é o que TODA cena paga, e é por isso que as duas travessias saem cedo antes
//! de construir índice nenhum: sem essa guarda o custo seria função do tamanho da cena em vez do
//! número de restrições.
//!
//! # ⛔ O alvo nunca é descendente da corrente
//!
//! Mover o osso moveria o alvo, que moveria o osso. O passe **recusa** esse caso (e diz-o no
//! `PH2D_BONE_LOG`) em vez de o resolver, porque não há resposta certa para um laço.

use ph2d_ecs::{ChildOf, Entity, SimWorld, StableId, Transform};
use ph2d_skeleton_ecs::IkGoal;

use ph2d_preview_drive::{Driven, PreviewDrive};

/// O nome que uma âncora nova recebe. ⚠️ Em inglês, como toda a UI da casa.

/// **O índice `StableId → entidade` de TUDO** — o alvo de uma âncora é um objecto qualquer, não um
/// osso, então este índice é mais largo que o [`crate::skeleton_live`]'s.
///
/// ⚠️ Construído uma vez por quadro e passado adiante, que é o que o doc do
/// [`ph2d_ecs::entity_of_stable_id`] manda fazer.
fn index(sim: &SimWorld) -> std::collections::BTreeMap<StableId, Entity> {
    sim.world()
        .iter_entities()
        .filter_map(|er| er.get::<StableId>().map(|s| (*s, er.id())))
        .collect()
}

/// ⭐⭐⭐ **ESTE OSSO É GOVERNADO POR UMA ÂNCORA?** — ou seja: *o ângulo dele é DERIVADO, e não
/// autorado.*
///
/// ⛔⛔ **Ela nasce de um report do dono** (2026-09-08: *«tudo configurado e a animação não rodou ao
/// rotacionar o bone»*) e a medição foi inequívoca: sobre um osso **livre** o controlo percorre a
/// acção inteira (`rot +21,8° → +95,1°`, a folha de `y +0,573` a `+3,000`); sobre um osso da
/// corrente de uma âncora a rotação fica **PRESA** (`+30,4°` quadro após quadro, com a sonda a somar
/// `+4,58°` a cada um), porque o solver a reescreve **depois** do passe do controlo.
///
/// ⇒ *um osso governado não pode ser um CONTROLO, e não porque alguém o proibiu: o ângulo dele não
/// é uma coisa que o artista escreve.* É a mesma razão pela qual a pose de repouso de um corpo
/// dinâmico não propaga (o dono do `Transform` é o solver, sempre).
///
/// ⚠️ Ela varre as âncoras da cena — que são um punhado — e não a corrente de cada osso: a pergunta
/// é *«alguém manda neste?»*, e quem sabe responder é o lado de quem manda.
#[must_use]
pub(crate) fn is_governed(sim: &SimWorld, bone: Entity) -> bool {
    sim.world()
        .iter_entities()
        .filter_map(|er| {
            er.get::<ph2d_skeleton_ecs::IkGoal>()
                .map(|g| (er.id(), g.chain))
        })
        .any(|(tip, chain)| governed(sim, tip, chain).contains(&bone))
}

/// ⭐⭐⭐ **A AGENDA — quem manda em que osso, e em que ORDEM.** (Report do dono, 2026-09-07:
/// *«múltiplos IKs numa cadeia de bones tem resultado ruim»*.)
///
/// # ⛔ Os três defeitos que ela cura, e eram três
///
/// 1. **A ordem de resolução era a dos ARQUÉTIPOS** (`iter_entities()` sem ordenar). Com correntes
///    que se sobrepõem, *a ordem É a resposta* — e ela não era sequer estável entre sessões.
/// 2. **Ninguém impunha RAIZ PRIMEIRO.** A âncora de cima move os pais e **arrasta** a solução da
///    de baixo: vale sempre a última a correr.
/// 3. **Nada proibia duas âncoras sobre o MESMO osso.** Elas escreviam a mesma rotação em
///    sequência, todo quadro.
///
/// # ⭐⭐ A lei: as correntes são DISJUNTAS, e resolvem-se da raiz para a ponta
///
/// **Passo 1 — a posse, da RAIZ para a ponta.** A âncora mais **rasa** reclama primeiro, e a mais
/// funda fica com o que sobra **abaixo** dela. Uma corrente é cortada no último osso já reclamado:
/// ⛔ uma corrente com um **buraco** no meio não é uma corrente — o osso do meio não obedeceria a
/// ninguém e a cinemática do que está acima dele deixaria de fazer sentido.
///
/// ⚠️⚠️ **A ordem da posse é a RASA primeiro, e a inversa foi construída e MEDIDA como errada.** Com
/// a funda a reclamar primeiro ela leva a corrente inteira e a de cima fica **inerte** — o artista
/// põe duas âncoras e uma delas simplesmente não faz nada. Com a rasa primeiro, cada uma fica com o
/// seu segmento e **as duas alcançam o próprio alvo**, que é o que ele quer: uma IK na coluna e
/// outra na mão trabalham ao mesmo tempo, cada uma no seu pedaço.
///
/// **Passo 2 — a resolução, da RAIZ para a ponta.** Com as correntes disjuntas, mover uma corrente
/// de cima muda a **origem** da de baixo; resolver a de baixo primeiro seria resolvê-la a partir de
/// um sítio que ainda vai mudar. Nesta ordem, um passe basta e o resultado é um **ponto fixo** —
/// que é o que o gate mede.
///
/// ⚠️ **A ordem é DERIVADA da hierarquia, e não autorada.** O Spine deixa o artista reordenar as
/// restrições e o Blender avalia por ordem de osso; entre as duas, a segunda é a que não precisa de
/// UI nenhuma e não tem estado a gravar. ⛔ E o desempate é o `StableId` (a identidade do
/// documento), nunca o `to_bits`: *não se escolhe um desempate melhor, não se tem empate* — e um
/// desempate por id de alocação mudaria a pose entre sessões.
fn schedule(
    sim: &SimWorld,
    cruas: Vec<(Entity, IkGoal)>,
    log: bool,
) -> Vec<(Entity, IkGoal, Vec<Entity>)> {
    // A profundidade de cada ponta na hierarquia de ossos — a régua das duas ordenações.
    let mut com_fundo: Vec<(usize, StableId, Entity, IkGoal)> = cruas
        .into_iter()
        .map(|(tip, g)| {
            let fundo = crate::skeleton_live::chain_to(sim, tip.to_bits()).len();
            let id = ph2d_ecs::stable_id_of(sim.world(), tip).unwrap_or(StableId::NONE);
            (fundo, id, tip, g)
        })
        .collect();
    // PASSO 1: a posse, da mais RASA para a mais funda.
    com_fundo.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut dono: std::collections::BTreeSet<Entity> = std::collections::BTreeSet::new();
    let mut agenda: Vec<(usize, StableId, Entity, IkGoal, Vec<Entity>)> = Vec::new();
    for (fundo, id, tip, g) in com_fundo {
        let toda = governed(sim, tip, g.chain);
        // A corrente é lida da RAIZ para a ponta; cortar num osso já reclamado significa manter só
        // o pedaço que vai dele para baixo.
        let corte = toda
            .iter()
            .rposition(|e| dono.contains(e))
            .map_or(0, |i| i + 1);
        let minha: Vec<Entity> = toda[corte..].to_vec();
        if log && corte > 0 {
            eprintln!(
                "[bone] ancora de {tip:?}: {corte} osso(s) ja' tinham dono - a corrente dela fica                  com {} de {}",
                minha.len(),
                toda.len()
            );
        }
        dono.extend(minha.iter().copied());
        agenda.push((fundo, id, tip, g, minha));
    }
    // ⭐ **PASSO 2 — e ele não existe.** A resolução tem de ser da RAIZ para a ponta, que é
    // **exactamente** a ordem em que a posse já correu ⇒ a `agenda` já sai ordenada.
    //
    // ⚠️ **Havia aqui um segundo `sort_by` idêntico, e uma mutação mostrou-o REDUNDANTE**: apagá-lo
    // não moveu gate nenhum. Ele fazia sentido enquanto a posse era da ponta para a raiz (as duas
    // ordens eram opostas); quando a posse passou a ser rasa-primeiro — a cura que faz as duas
    // âncoras alcançarem os próprios alvos — as duas ordens colapsaram numa. *Uma linha que
    // sobrevive a uma mudança de desenho ao lado dela costuma ter deixado de fazer alguma coisa.*
    agenda
        .into_iter()
        .map(|(_, _, tip, g, corrente)| (tip, g, corrente))
        .collect()
}

/// ⛔ **O alvo está DENTRO da corrente?** — o laço que este passe recusa em vez de resolver.
///
/// Sobe do alvo pelos pais: se encontrar qualquer osso governado, mover a corrente moveria o alvo.
fn feeds_back(sim: &SimWorld, alvo: Entity, corrente: &[Entity]) -> bool {
    let mut e = alvo;
    loop {
        if corrente.contains(&e) {
            return true;
        }
        let Some(p) = sim.world().get::<ChildOf>(e).map(ChildOf::parent) else {
            return false;
        };
        e = p;
    }
}

/// ⭐⭐⭐ **AS ÂNCORAS DA CENA** — `(osso, âncora, origem do osso, ponta do osso)` em MUNDO, ordenado.
///
/// ⚠️ **Uma varredura, três consumidores**: o desenho (o losango e o tracejado), o dedo (o realce) e
/// o arrasto. Uma segunda varredura com outra regra divergiria desta na primeira ramificação — é a
/// mesma lei que o [`crate::skeleton_live::bone_segments`] já declara.
///
/// ⚠️ O **segmento do osso** vem junto porque o tamanho do losango sai da mesma porta da bolinha
/// (`joint_radius_px`, sobre o comprimento do osso **na tela**) — sem ele o desenho teria de
/// re-encontrar o osso, que é a segunda resposta à mesma pergunta.
///
/// ⚠️ Um osso cuja âncora perdeu o alvo **não entra**: não há onde desenhar nem o que agarrar.
pub(crate) fn anchors(sim: &SimWorld) -> Vec<ph2d_skeleton_render::Goal> {
    // ⚠️ **A saída cedo vem ANTES do índice, e não é micro-optimização:** esta função corre no
    // caminho de DESENHO de todo quadro, e o índice é uma travessia do mundo inteiro. A cena comum
    // não tem âncora nenhuma — fazê-la pagar uma varredura por quadro para descobrir isso seria o
    // custo a ser função do tamanho da cena em vez do número de restrições.
    if !sim
        .world()
        .iter_entities()
        .any(|er| er.contains::<IkGoal>())
    {
        return Vec::new();
    }
    let idx = index(sim);
    let segs = crate::skeleton_live::bone_segments(sim);
    let mut out: Vec<ph2d_skeleton_render::Goal> = sim
        .world()
        .iter_entities()
        .filter_map(|er| {
            let g = er.get::<IkGoal>()?;
            let alvo = idx.get(&g.target).copied()?;
            let (_, o, ponta) = segs
                .iter()
                .copied()
                .find(|(x, _, _)| *x == er.id().to_bits())?;
            let a = ph2d_vec_entities::transform::xform_of_transform(
                ph2d_vec_entities::transform::world_transform(sim, alvo),
            )
            .apply([0.0, 0.0]);
            Some((er.id().to_bits(), a, o, ponta))
        })
        .collect();
    out.sort_by_key(|g| g.0);
    out
}

/// **As pontas de corrente que ainda mostram o ANEL** — as que não têm âncora.
///
/// ⭐ É esta subtracção que faz o losango **substituir** o anel em vez de se somar a ele: num osso
/// ancorado a ponta deixa de ser agarrável (o que se arrasta é o alvo), e desenhar as duas coisas
/// por cima uma da outra prometeria dois verbos onde há um.
pub(crate) fn unanchored_ends(sim: &SimWorld) -> Vec<u64> {
    crate::skeleton_live::chain_ends(sim)
        .into_iter()
        .filter(|&b| sim.world().get::<IkGoal>(Entity::from_bits(b)).is_none())
        .collect()
}

/// **Move a âncora deste osso para `world`.** Devolve `false` se ele não tem âncora viva.
///
/// ⚠️ É por aqui que o arrasto da ponta passa quando há restrição: ele deixa de posar a corrente
/// (que o passe reescreveria no quadro seguinte, e o artista veria o osso voltar) e passa a mover o
/// **objecto autorado**, que é o que o documento guarda.
pub(crate) fn drag_anchor(sim: &mut SimWorld, bone: Entity, world: [f64; 2]) -> bool {
    let Some(g) = sim.world().get::<IkGoal>(bone).copied() else {
        return false;
    };
    let Some(alvo) = index(sim).get(&g.target).copied() else {
        return false;
    };
    // O espaço do PAI do ALVO — a pose local vive nele. Uma âncora nasce raiz, então isto costuma
    // ser a identidade; mas o artista pode tê-la pendurado num objecto, e aí o que ele arrasta é a
    // posição de mundo e o que se grava é a local.
    let pai = sim.world().get::<ChildOf>(alvo).map(ChildOf::parent);
    let pai_mundo = pai.map_or(ph2d_vec_scene::Xform::IDENTITY, |p| {
        ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(sim, p),
        )
    });
    let Some(inv) = pai_mundo.inverse() else {
        return false;
    };
    let p = inv.apply(world);
    let Some(mut t) = sim.world_mut().get_mut::<Transform>(alvo) else {
        return false;
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o `Transform` da casa é f32; a geometria do documento é f64"
    )]
    {
        t.translation = ph2d_core::Vec2::new(p[0] as f32, p[1] as f32);
    }
    true
}

/// **APAGA a âncora** deste osso, o alvo com ela, **e devolve a pose que o artista autorou**.
/// Devolve `true` se havia uma.
///
/// ⚠️ **O alvo morre junto**, e a razão é que ele existe *para* a restrição: deixá-lo para trás
/// encheria a Hierarquia de objectos vazios que não fazem nada, e o artista não teria como saber
/// quais podia apagar. ⛔ Quem o quiser guardar tem o Ctrl+Z, que é a porta da casa para isso.
///
/// # ⭐⭐⭐ E a POSE volta — a metade que faltava
///
/// ⛔ **Sem ela, apagar a restrição ASSAVA a pose dela no documento, em silêncio** (report do dono,
/// 2026-09-07: *«Remove IK … não funciona plenamente»*): a âncora saía e a corrente ficava dobrada
/// onde ela a tinha posto, sem caminho de volta.
///
/// ⚠️ **E isso contradizia a lei que este próprio módulo escreveu:** o que a restrição escreve é
/// **pré-visualização** — *vê-se, não se guarda*. Deixá-la ficar promovia-a a documento no
/// `settle()` do quadro seguinte. O Blender faz o que se faz aqui: remover a *constraint* devolve
/// o osso à pose de FK.
///
/// ⚠️ **A `settle` NÃO servia**: ela é para um motor que **largou** (e aí o vivo *é* o documento);
/// aqui o motor foi **desligado**, e o vivo é dele. São dois factos diferentes com a mesma forma —
/// ver [`PreviewDrive::release_to_authored`].
pub(crate) fn remove(sim: &mut SimWorld, bone: Entity, preview: &mut PreviewDrive) -> bool {
    let Some(g) = sim.world().get::<IkGoal>(bone).copied() else {
        return false;
    };
    // ⚠️ **A corrente lê-se ANTES de o componente sair** — depois dele o `governed` não tem por onde
    // saber que ossos esta âncora governava.
    let corrente = governed(sim, bone, g.chain);
    if let Some(alvo) = index(sim).get(&g.target).copied() {
        let _ = sim.world_mut().despawn(alvo);
    }
    sim.world_mut().entity_mut(bone).remove::<IkGoal>();
    for e in corrente {
        preview.release_to_authored(sim, e, ph2d_preview_drive::Driver::SolverPose);
    }
    true
}

/// ⭐⭐⭐ **UM QUADRO DE RESTRIÇÕES.** Devolve quantas correntes moveu.
///
/// Corre **antes** do [`crate::skeleton_live::recook`] — ele lê a pose de agora, e a pose de agora é
/// o que este passe acaba de escrever.
pub(crate) fn solve(sim: &mut SimWorld, preview: &mut PreviewDrive) -> usize {
    let cruas: Vec<(Entity, IkGoal)> = sim
        .world()
        .iter_entities()
        .filter_map(|er| Some((er.id(), *er.get::<IkGoal>()?)))
        .collect();
    if cruas.is_empty() {
        return 0;
    }
    let log = std::env::var_os("PH2D_BONE_LOG").is_some();
    let idx = index(sim);
    let ancoras = schedule(sim, cruas, log);
    let mut feitas = 0;
    for (tip, g, corrente) in ancoras {
        if corrente.is_empty() {
            continue;
        }
        let Some(alvo) = idx.get(&g.target).copied() else {
            if log {
                eprintln!("[bone] ancora de {tip:?} sem alvo vivo - corrente em paz");
            }
            continue;
        };
        if feeds_back(sim, alvo, &corrente) {
            if log {
                eprintln!(
                    "[bone] ancora de {tip:?} RECUSADA: o alvo esta' dentro da propria corrente"
                );
            }
            continue;
        }
        let goal = ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(sim, alvo),
        )
        .apply([0.0, 0.0]);
        if solve_one(sim, preview, &corrente, goal, g) {
            feitas += 1;
        }
    }
    if log && feitas > 0 {
        eprintln!("[bone] {feitas} corrente(s) sob ancora");
    }
    feitas
}

/// Uma corrente, um alvo. Separada por responsabilidade e pelo teto de LOC por função (HR-18).
fn solve_one(
    sim: &mut SimWorld,
    preview: &mut PreviewDrive,
    corrente: &[Entity],
    goal: [f64; 2],
    g: IkGoal,
) -> bool {
    let Some((mut juntas, comps)) = joints_of(sim, corrente) else {
        return false;
    };
    let alcance: f64 = comps.iter().sum();
    if alcance <= f64::EPSILON {
        return false;
    }
    // ⚠️ A suavidade é uma FRACÇÃO do alcance (adimensional, como a força do osso) e a lei do
    // `reach` quer unidades de MUNDO — a conversão vive aqui, que é a porta onde as duas se
    // encontram.
    ph2d_skeleton::reach(
        &mut juntas,
        &comps,
        goal,
        ph2d_skeleton::Reach {
            softness: g.softness.max(0.0) * alcance,
            // ⭐ O lado AUTORADO da restrição. ⚠️ O gesto de arrastar a ponta continua a passar
            // `Keep` (o default), e é a diferença que importa: um gesto preserva o que se vê, uma
            // restrição defende o que se autorou.
            bend: g.bend,
            ..ph2d_skeleton::Reach::default()
        },
    );
    let mix = g.mix.clamp(0.0, 1.0);
    let mut mexeu = false;
    for (i, &e) in corrente.iter().enumerate() {
        // ⭐⭐ A MIRA é a mesma porta do gesto, e a MISTURA é sobre o ÂNGULO: interpolar as posições
        // das juntas encurtaria os ossos.
        let Some(alvo_rot) = crate::bone_gesture::aim_rotation(sim, e, juntas[i + 1]) else {
            continue;
        };
        let Some(antes) = sim.world().get::<Transform>(e).copied() else {
            continue;
        };
        // ⚠️ **O limite apara DEPOIS da mistura**, e a ordem importa: com uma faixa maior que meia
        // volta o caminho curto entre dois ângulos que estão ambos dentro dela pode passar por
        // FORA. Aparar o alvo antes de misturar deixaria a pose final a violar o limite em
        // silêncio, que é a única coisa que este componente existe para impedir.
        let nova = crate::bone_limit::limited(
            sim,
            e,
            ph2d_skeleton::blend_angle(f64::from(antes.rotation), alvo_rot, mix),
        );
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a rotação do `Transform` da casa é f32; a lei do módulo é f64"
        )]
        let nova = nova as f32;
        if nova == antes.rotation {
            // ⭐⭐⭐ **A ÂNCORA AINDA CONDUZ, mesmo com a corrente parada** — a mesma lei do osso
            // inteligente (report do dono, 2026-09-09). Uma restrição é um condutor
            // **PERSISTENTE**: ela escreve todo quadro, e o output dela é constante na maior parte
            // do tempo. Sem esta linha a `settle` lê a constância como *«o motor largou»* e promove
            // a pose da restrição a documento — e o *Remove IK* já não tem o autorado para devolver.
            preview.still_driving(e, ph2d_preview_drive::Driver::SolverPose);
            continue;
        }
        let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) else {
            continue;
        };
        t.rotation = nova;
        let depois = *t;
        // ⚠️ O ledger: a rotação que esta restrição escreve é PRÉ-VISUALIZAÇÃO. O que o artista
        // autora é a pose da ÂNCORA.
        preview.driven(e, Driven::SolverPose(antes), Driven::SolverPose(depois));
        mexeu = true;
    }
    mexeu
}

#[cfg(test)]
#[path = "skeleton_goal_tests.rs"]
mod tests;

/// ⭐ **O que a âncora escreve é PRÉ-VISUALIZAÇÃO** — irmão pelo teto de 600 LOC, e o corte é por
/// RESPONSABILIDADE: o `tests` mede a LEI que ela resolve por quadro; este mede o que acontece ao
/// DOCUMENTO (quem larga, quem escreve por cima, e o que sobra quando o motor é desligado).
#[cfg(test)]
#[path = "skeleton_goal_ledger_tests.rs"]
mod ledger_tests;

/// ⭐⭐⭐ **CRIA a âncora** deste osso — delegação para [`ph2d_skeleton_live::goal::add`].
pub(crate) fn add(sim: &mut SimWorld, bone: Entity) -> Option<Entity> {
    ph2d_skeleton_live::goal::add(sim, bone)
}

/// **A corrente que esta âncora governa** — delegação para [`ph2d_skeleton_live::goal::governed`].
///
/// ⚠️ `chain` é `u32` e não `u16`: a 1.ª redacção desta delegação escreveu a assinatura de
/// memória e o compilador apanhou-a em três sítios. *Uma assinatura lê-se do ficheiro.*
pub(crate) fn governed(sim: &SimWorld, tip: Entity, chain: u32) -> Vec<Entity> {
    ph2d_skeleton_live::goal::governed(sim, tip, chain)
}

/// **As juntas de uma corrente, em MUNDO** — delegação para
/// [`ph2d_skeleton_live::goal::joints_of`].
fn joints_of(sim: &SimWorld, corrente: &[Entity]) -> Option<(Vec<[f64; 2]>, Vec<f64>)> {
    ph2d_skeleton_live::goal::joints_of(sim, corrente)
}
