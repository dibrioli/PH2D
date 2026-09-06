//! **O gesto do modo OSSO** (estudo 42 item 5, doc 47 §2.6) — arrastar no vazio faz um osso.
//!
//! ```text
//! press  → se acertou um osso, SELECCIONA-o (é assim que se ramifica)
//!          senão, marca a origem
//! release→ nasce o osso: origem no press, comprimento e ângulo no arrasto,
//!          PAI = o osso seleccionado, e o novo fica seleccionado
//! ```
//!
//! ⇒ arrasto-arrasto-arrasto é uma **cadeia**, sem um clique de cerimónia. É o gesto do Spine, do
//! Moho e do Rive, e a razão de ele funcionar sem estado próprio é a decisão do doc 47 §2.1: um
//! osso é uma **entidade**, então "quem é o pai" já é a selecção que o resto do app usa.
//!
//! ⚠️ **A pose é LOCAL do pai, e é derivada levando os DOIS pontos ao espaço dele.** Compor ângulos
//! e comprimentos à mão (`rot − rot_do_pai`, `len / escala_do_pai`) só está certo com um pai
//! conforme; transformar os pontos está certo com qualquer afim, e é a mesma regra-mãe do pen —
//! *o que se aponta é MUNDO; o que o documento guarda é LOCAL*.

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform, VecBone};
use ph2d_vec_scene::Xform;

/// Raio de acerto de um osso, em píxeis de tela — o mesmo `HANDLE_HIT_PX` que as alças do vetor
/// usam, para o dedo do artista ter sempre a mesma tolerância.
pub(crate) const BONE_HIT_PX: f64 = 12.0;

/// **O osso sob o ponteiro** (o mais próximo dentro do raio), ou `None`.
pub(crate) fn hit(sim: &SimWorld, world: [f64; 2], px_to_world: f64) -> Option<u64> {
    let r = BONE_HIT_PX * px_to_world;
    let mut melhor: Option<(f64, u64)> = None;
    for (bits, a, b) in crate::skin_live::bone_segments(sim) {
        let d2 = ph2d_vec_skin::dist2_to_segment(world, a, b);
        if d2 <= r * r && melhor.is_none_or(|(m, _)| d2 < m) {
            melhor = Some((d2, bits));
        }
    }
    melhor.map(|(_, bits)| bits)
}

/// **A ponta de um osso, em MUNDO** — para o encaixe do próximo nascer colado nela.
pub(crate) fn tip_of(sim: &SimWorld, bits: u64) -> Option<[f64; 2]> {
    crate::skin_live::bone_segments(sim)
        .into_iter()
        .find(|(b, _, _)| *b == bits)
        .map(|(_, _, tip)| tip)
}

/// **Faz um osso** de `origin` a `tip` (mundo), filho de `parent`. Devolve os bits dele.
///
/// `None` se a pose do pai é singular (escala zero) — não há espaço local em que pôr o osso.
pub(crate) fn create(
    sim: &mut SimWorld,
    parent: Option<Entity>,
    origin: [f64; 2],
    tip: [f64; 2],
) -> Option<u64> {
    // O espaço do PAI. Sem pai, o mundo — e aí o inverso é a identidade.
    let pai_mundo = parent.map_or(Xform::IDENTITY, |p| {
        crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(sim, p))
    });
    let inv = pai_mundo.inverse()?;
    let a = inv.apply(origin);
    let b = inv.apply(tip);
    let d = [b[0] - a[0], b[1] - a[1]];
    let length = d[0].hypot(d[1]);
    let rotation = d[1].atan2(d[0]);
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "o `Transform` da casa é f32; a geometria do documento é f64"
                )]
                translation: ph2d_core::Vec2::new(a[0] as f32, a[1] as f32),
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "idem — a rotação do `Transform` é f32"
                )]
                rotation: rotation as f32,
                ..Transform::IDENTITY
            },
            VecBone {
                length,
                ..VecBone::default()
            },
        ))
        .id();
    // ⚠️ **O nome tem de ser ÚNICO**, e não é cosmética: a referência durável entre objectos neste
    // app é o NOME (`stable_name_id`, o hash do `Name`) — dois ossos chamados "Bone" seriam o mesmo
    // sujeito para a timeline. O índice da entidade é único entre as vivas, que é o mesmo critério
    // que o `vec_entities` usa para um caminho novo.
    sim.world_mut()
        .entity_mut(e)
        .insert(Name::new(format!("Bone {}", e.index())));
    match parent {
        Some(p) => {
            sim.world_mut().entity_mut(e).insert(ChildOf(p));
        }
        None => {
            // ⛔ `RootOrder` EXPLÍCITO — sem ele a árvore desempata por bits de alocação, e o undo
            // vira um passo espúrio por quadro (BUGS #15).
            let order = crate::vec_entities::next_root_order(sim);
            sim.world_mut().entity_mut(e).insert(RootOrder(order));
        }
    }
    Some(e.to_bits())
}

/// **O que o press do modo Osso DECIDE** — a porta única, para a decisão ser observável.
///
/// ⚠️ **Ela nasceu de um report** (Enio, 2026-09-06: *"o bind não funciona"*): a decisão vivia
/// dentro do `input_dispatch`, onde nenhum teste a alcança, e faltava-lhe metade — apontar uma
/// forma nunca a SELECCIONAVA, então o botão *Bind* (que age sobre a selecção de formas) só sabia
/// recusar. *Uma decisão que só existe dentro do dispatch é uma decisão que nenhum gate lê.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BonePress {
    /// Acertou um osso: selecciona-o e ARMA a pose (`joint` ⇒ desloca em vez de girar).
    Grab { bone: u64, joint: bool },
    /// Não acertou osso: marca a ORIGEM de um osso novo (já encaixada na ponta do pai, se perto) e
    /// diz que FORMA estava sob o cursor — é essa a metade que o *Bind* precisa.
    Start {
        origin: [f64; 2],
        pick: Option<ph2d_vec_scene::VecPathId>,
    },
}

/// A decisão do press, sem tocar em nada.
pub(crate) fn press(
    sim: &SimWorld,
    scene: &ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    world: [f64; 2],
    px_to_world: f64,
    selected: Option<u64>,
) -> BonePress {
    if let Some(bone) = hit(sim, world, px_to_world) {
        return BonePress::Grab {
            bone,
            joint: grabbed_the_joint(Some(sim), bone, world, px_to_world),
        };
    }
    // ⭐⭐⭐ **O FILHO NASCE NA PONTA DO PAI, SEMPRE** — não "quando o press cai perto dela".
    //
    // ⛔ **O encaixe por PROXIMIDADE era inalcançável, e um gate apanhou-o**: a ponta está SOBRE o
    // segmento do osso, então todo press dentro do raio de encaixe está também dentro do raio de
    // acerto — o `hit` acima ganha sempre, e o ramo do encaixe nunca corria. *Um encaixe que exige
    // pontaria dentro do alvo que ele quer evitar não é um encaixe.*
    //
    // ⇒ a lei passa a ser a do Spine e a do Moho: com um osso aceso, o arrasto seguinte cresce da
    // PONTA dele para onde a mão for. Para começar um osso solto, basta que nenhum osso esteja
    // aceso — e clicar numa forma (o ramo `pick` abaixo) faz exactamente isso.
    let r = BONE_HIT_PX * px_to_world;
    let origin = selected_bone(sim, selected)
        .and_then(|b| tip_of(sim, b))
        .unwrap_or(world);
    BonePress::Start {
        origin,
        pick: pen.path_at(scene, world, r),
    }
}

/// **O press caiu na JUNTA deste osso?** — a bolinha da raiz, dentro do mesmo raio das alças.
///
/// ⚠️ **É a pergunta que escolhe o VERBO** (deslocar × girar), então ela mora ao lado da função que
/// os executa. Escrita no `input_dispatch`, ela e o `pose` divergiriam no dia em que o raio mudasse
/// — e o sintoma seria *"às vezes ele gira, às vezes ele anda"*.
pub(crate) fn grabbed_the_joint(
    sim: Option<&SimWorld>,
    bits: u64,
    world: [f64; 2],
    px_to_world: f64,
) -> bool {
    let Some(sim) = sim else {
        return false;
    };
    crate::skin_live::bone_segments(sim)
        .into_iter()
        .find(|(b, _, _)| *b == bits)
        .is_some_and(|(_, a, _)| {
            // ⚠️ O raio é o da BOLINHA DESENHADA (`BONE_JOINT_R_PX`), e não o do osso: são duas
            // perguntas — *acertei o osso?* e *acertei a junta DELE?* — e usar o mesmo número faria
            // um osso curto ser todo junta, logo impossível de girar.
            (a[0] - world[0]).hypot(a[1] - world[1])
                <= ph2d_vec_render::BONE_JOINT_R_PX * px_to_world
        })
}

/// **POSAR um osso** — as duas metades do gesto, e por que são duas.
///
/// ⚠️ **O gizmo de sprite NÃO serve aqui, e isso foi medido:** ele dimensiona-se pela caixa da
/// geometria (`vec_gizmo_view::anchor_half` pede um `VecPathRef`), e um osso não tem geometria
/// nenhuma — a caixa sai `0×0` e as alças colapsam num ponto. ⇒ o osso posa-se **agarrando o osso**,
/// que é o gesto do Spine, do Moho e de todo pacote de rig.
///
/// - **Pelo CORPO** ⇒ gira (a origem fica, a ponta segue o ponteiro).
/// - **Pela JUNTA** (a bolinha da raiz) ⇒ desloca.
///
/// *Duas coisas diferentes precisam de dois gestos*: sem o segundo, um esqueleto inteiro não se
/// move do sítio onde nasceu, e a única saída seria o painel de Transform.
pub(crate) fn pose(sim: &mut SimWorld, bone: Entity, world: [f64; 2], desloca: bool) -> bool {
    // O espaço do PAI — a pose local vive nele. Sem pai, o mundo.
    let pai = sim.world().get::<ChildOf>(bone).map(ChildOf::parent);
    let pai_mundo = pai.map_or(Xform::IDENTITY, |p| {
        crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(sim, p))
    });
    let Some(inv) = pai_mundo.inverse() else {
        return false;
    };
    let p = inv.apply(world);
    let Some(mut t) = sim.world_mut().get_mut::<Transform>(bone) else {
        return false;
    };
    if desloca {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "o `Transform` da casa é f32; a geometria do documento é f64"
        )]
        {
            t.translation = ph2d_core::Vec2::new(p[0] as f32, p[1] as f32);
        }
        return true;
    }
    let o = [f64::from(t.translation.x), f64::from(t.translation.y)];
    let d = [p[0] - o[0], p[1] - o[1]];
    // ⛔ Sobre a própria origem não há DIRECÇÃO — apontar para lá daria um ângulo arbitrário, e o
    // osso saltaria. Ficar quieto é a resposta certa.
    if d[0].hypot(d[1]) < f64::EPSILON {
        return false;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "idem — a rotação do `Transform` é f32"
    )]
    {
        t.rotation = d[1].atan2(d[0]) as f32;
    }
    true
}

/// **A selecção do gizmo é um OSSO?** — a porta única da pergunta.
///
/// ⚠️ **Não há um "osso activo" à parte**, e é essa ausência que faz o gizmo de sprite POSAR o osso
/// no modo Select sem uma linha de código própria, e o pai do próximo osso ser exactamente o que
/// está aceso na Hierarquia. *Um segundo estado de selecção divergiria do primeiro no primeiro
/// clique.*
pub(crate) fn selected_bone(
    sim: &SimWorld,
    selection: impl IntoIterator<Item = u64>,
) -> Option<u64> {
    // ⚠️ **A selecção INTEIRA, e não só o primário**: prender uma forma a UM esqueleto entre vários
    // faz-se escolhendo os dois (a forma e um osso dele) na Hierarquia, e o primário é a forma. Ler
    // só o primário tornaria essa desambiguação inexprimível.
    selection
        .into_iter()
        .find(|&b| sim.world().get::<VecBone>(Entity::from_bits(b)).is_some())
}

impl crate::App {
    /// [`selected_bone`] pela selecção do gizmo deste quadro.
    ///
    /// ⚠️ **Ela existe SÓ para o caminho do gesto**, onde `self` está inteiro na mão. No laço de
    /// desenho o `gfx` já está emprestado mutável de ponta a ponta, e ali chama-se a função livre
    /// acima — a lei é a mesma, e é por isso que ela vive numa função só.
    pub(crate) fn selected_bone_bits(&self) -> Option<u64> {
        let gfx = self.gfx.as_ref()?;
        selected_bone(&gfx.sim, gfx.hero_screen.as_ref()?.gizmo.iter_selected())
    }
}

#[cfg(test)]
#[path = "bone_gesture_tests.rs"]
mod tests;
