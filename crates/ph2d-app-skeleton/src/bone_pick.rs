//! ⭐⭐⭐ **O QUE O PONTEIRO APONTA NUM OSSO** — a família do hit-test e do realce.
//!
//! ⚠️ **O realce e o clique saem daqui, das MESMAS funções.** É a lei que o `pick_hovered_object`
//! já declara para as formas — *«um realce que acendesse outra coisa que a que o clique pega seria
//! pior que não haver realce nenhum»* — e neste módulo ela é mais apertada, porque as alças do osso
//! estão **umas dentro das outras** e executam VERBOS diferentes.
//!
//! ⚠️ **Ele saiu do [`crate::bone_gesture`] em 2026-09-09, por CORTE de responsabilidade** (o tecto
//! de LOC do shell): ali fica *o gesto que FAZ um osso*, aqui *o que o dedo significa*. ⛔ As duas
//! metades continuam a ser uma lei só — o `press` chama o [`hover`] daqui, e há gate a compará-los
//! ponto a ponto nos dois verbos.

use ph2d_ecs::{Entity, SimWorld};

/// Raio de acerto de um osso, em píxeis de tela — o mesmo `HANDLE_HIT_PX` que as alças do vetor
/// usam, para o dedo do artista ter sempre a mesma tolerância.
pub const BONE_HIT_PX: f64 = 12.0;

/// ⭐⭐⭐ **O raio de acerto das duas alças de CURVATURA — apertado, e o recurso tem nome.**
///
/// ⚠️ Elas são as únicas que ganham ao CORPO do osso, porque no ponto neutro estão em cima dele
/// (ver o [`hover`]). ⇒ o que este número gasta é **o comprimento do osso que sobra para o verbo de
/// girar**: as duas bolinhas ficam nos terços, logo elas comem `2 × 2 × este raio` do osso e o
/// ponto do meio fica a `L/6` de cada uma.
///
/// ⇒ o meio do osso continua a girar enquanto `L/6 > 8 px`, isto é **a partir de `48 px` de osso na
/// tela** — que é o dobro do menor osso que a bolinha da junta ainda desenha separada. O gate
/// `o_meio_de_um_osso_curvo_ainda_gira` é quem o afirma.
///
/// ⚠️ **É `2 ×` o raio DESENHADO** ([`ph2d_skeleton_render::BEND_HANDLE_R_PX`]), que é a folga que
/// todas as alças deste app dão ao dedo.
pub const BEND_HIT_PX: f64 = 2.0 * ph2d_skeleton_render::BEND_HANDLE_R_PX;

/// ⭐⭐⭐ **O CORPO de um osso, em MUNDO** — a MESMA polilinha que o overlay pinta
/// ([`ph2d_skeleton_live::skin_live::bone_polylines`]).
///
/// ⛔ **É a porta que impede o osso curvo de ser um controlo morto sob o dedo:** desde a F8 o corpo
/// desenhado é uma polilinha, e um dedo que continuasse a medir a corda raiz→ponta agarraria uma
/// linha que não está pintada em sítio nenhum. Num osso recto ela devolve dois nós e tudo aqui
/// continua a ser o que era, ao bit.
fn corpo(sim: &SimWorld, bits: u64) -> Option<Vec<[f64; 2]>> {
    ph2d_skeleton_live::skin_live::visible_bone_polylines(sim)
        .into_iter()
        .find(|(x, _)| *x == bits)
        .map(|(_, pts)| pts)
}

/// **O comprimento ANDADO deste osso, em píxeis de TELA** — a régua de que a bolinha da junta sai.
///
/// ⚠️ **É o arco e não a corda**, pela mesma razão do desenho: num osso muito arqueado a corda
/// encolhe, e uma bolinha dimensionada por ela ficaria mais pequena exactamente quando o osso é
/// mais difícil de agarrar. Num osso recto os dois números são o mesmo `hypot`.
fn comp_px(pts: &[[f64; 2]], px_to_world: f64) -> f64 {
    ph2d_skeleton::bend::arc_length(pts) / px_to_world.max(f64::MIN_POSITIVE)
}

/// **O osso sob o ponteiro** (o mais próximo dentro do raio), ou `None`.
pub fn hit(sim: &SimWorld, world: [f64; 2], px_to_world: f64) -> Option<u64> {
    let r = BONE_HIT_PX * px_to_world;
    let mut melhor: Option<(f64, u64)> = None;
    for (bits, pts) in ph2d_skeleton_live::skin_live::visible_bone_polylines(sim) {
        let d2 = ph2d_skeleton::dist2_to_polyline(world, &pts);
        if d2 <= r * r && melhor.is_none_or(|(m, _)| d2 < m) {
            melhor = Some((d2, bits));
        }
    }
    melhor.map(|(_, bits)| bits)
}

/// ⭐⭐⭐ **A PONTA DE OSSO SOB O PONTEIRO** — a porta ÚNICA do parentesco (ordem do dono,
/// 2026-09-09). Devolve o osso que a oferece e o ponto dela, em mundo.
///
/// ⚠️ **O raio é o que está DESENHADO** ([`ph2d_skeleton_render::joint_radius_px`]), pela mesma lei
/// que o corpo e a junta já seguem: *o dedo pega exactamente o que o olho vê*. Ele encolhe num osso
/// curto porque a bolinha encolhe — e é por isso que a régua sai da porta do desenho e não de uma
/// constante local, que divergiria dela no primeiro ajuste.
///
/// ⚠️ **Ganha a mais PERTO, e não a primeira da lista.** Numa corrente contínua a ponta de um osso
/// é a raiz do seguinte, e a ponta DELE está a um comprimento de distância — sem a competição por
/// proximidade, a ordem de varredura escolheria o pai errado sempre que dois alvos se tocassem.
///
/// ⛔ **É a ponta de TODO osso, não só das que fecham a corrente.** Ramificar do meio de uma
/// espinha é o gesto que faz dois braços, e uma lista só de pontas-de-corrente torná-lo-ia
/// inexprimível — que é exactamente o defeito que esta wave veio curar, um nível acima.
pub fn tip_at(sim: &SimWorld, world: [f64; 2], px_to_world: f64) -> Option<(u64, [f64; 2])> {
    let mut melhor: Option<(f64, u64, [f64; 2])> = None;
    for (bits, pts) in ph2d_skeleton_live::skin_live::visible_bone_polylines(sim) {
        let b = *pts.last().expect("a polilinha tem ao menos dois nos");
        let r = ph2d_skeleton_render::joint_radius_px(comp_px(&pts, px_to_world)) * px_to_world;
        let d = (b[0] - world[0]).hypot(b[1] - world[1]);
        if d <= r && melhor.is_none_or(|(m, _, _)| d < m) {
            melhor = Some((d, bits, b));
        }
    }
    melhor.map(|(_, bits, p)| (bits, p))
}

/// ⭐⭐⭐ **A BASE DE UMA CORRENTE SOLTA SOB O PONTEIRO** — o espelho do [`tip_at`], e a porta do
/// gesto que **junta dois esqueletos separados** (ordem do dono, 2026-09-09).
///
/// ⚠️ **Só ossos SEM PAI-OSSO**, e a razão não é de conveniência, é de ambiguidade: a raiz de um
/// osso do meio de uma corrente **É** a ponta do pai dele, no mesmo pixel — e ali a lei da ponta já
/// fala (*«daqui nasce um filho»*). *Uma base que já tem dono não está livre para ser adoptada, e
/// dois verbos no mesmo pixel é exactamente o defeito que a lei da ponta veio curar.*
///
/// ⚠️ **A régua de «tem pai» é a MESMA do [`ph2d_skeleton_live::skin_live::chain_to`]** — o pai existe **e**
/// é osso. Um esqueleto pendurado dentro de um grupo continua a ser uma corrente solta, que é o que
/// o resto do módulo já assume.
///
/// ⚠️ Raio e desempate iguais aos do [`tip_at`]: a bolinha DESENHADA, e ganha a mais perto.
pub fn free_root_at(sim: &SimWorld, world: [f64; 2], px_to_world: f64) -> Option<(u64, [f64; 2])> {
    let mut melhor: Option<(f64, u64, [f64; 2])> = None;
    for (bits, pts) in ph2d_skeleton_live::skin_live::visible_bone_polylines(sim) {
        if !is_a_free_chain_root(sim, bits) {
            continue;
        }
        let a = pts[0];
        let r = ph2d_skeleton_render::joint_radius_px(comp_px(&pts, px_to_world)) * px_to_world;
        let d = (a[0] - world[0]).hypot(a[1] - world[1]);
        if d <= r && melhor.is_none_or(|(m, _, _)| d < m) {
            melhor = Some((d, bits, a));
        }
    }
    melhor.map(|(_, bits, p)| (bits, p))
}

/// **Este osso abre uma corrente?** — não tem pai, ou o pai que tem não é osso.
///
/// ⚠️ **Porta única das duas perguntas que a precisam** ([`free_root_at`] e o gate que a mede), e a
/// mesma regra que o [`ph2d_skeleton_live::skin_live::chain_to`] usa para parar de subir. Escrita duas vezes,
/// ela divergiria no dia em que alguém decidisse o que um osso dentro de um grupo é.
#[must_use]
pub fn is_a_free_chain_root(sim: &SimWorld, bits: u64) -> bool {
    let Some(e) = Entity::try_from_bits(bits) else {
        return false;
    };
    sim.world()
        .get::<ph2d_ecs::ChildOf>(e)
        .map(ph2d_ecs::ChildOf::parent)
        .is_none_or(|p| sim.world().get::<ph2d_skeleton_ecs::Bone>(p).is_none())
}

/// **O press caiu na JUNTA deste osso?** — a bolinha da raiz, dentro do mesmo raio das alças.
///
/// ⚠️ **É a pergunta que escolhe o VERBO** (deslocar × girar), então ela mora ao lado da função que
/// os executa. Escrita no `input_dispatch`, ela e o `pose` divergiriam no dia em que o raio mudasse
/// — e o sintoma seria *"às vezes ele gira, às vezes ele anda"*.
pub fn grabbed_the_joint(
    sim: Option<&SimWorld>,
    bits: u64,
    world: [f64; 2],
    px_to_world: f64,
) -> bool {
    let Some(sim) = sim else {
        return false;
    };
    corpo(sim, bits).is_some_and(|pts| {
        let a = pts[0];
        // ⚠️ O raio é o da BOLINHA DESENHADA, pela porta única
        (a[0] - world[0]).hypot(a[1] - world[1])
            <= ph2d_skeleton_render::joint_radius_px(comp_px(&pts, px_to_world)) * px_to_world
    })
}

/// ⭐⭐ **O QUE ESTÁ SOB O PONTEIRO** — o osso e a METADE dele, para o realce dizer qual verbo o
/// clique vai executar (Enio, 2026-09-06).
///
/// ⚠️ **Ele sai das MESMAS duas funções que o clique usa** ([`hit`] e [`grabbed_the_joint`]) e não
/// de uma varredura própria — é a lei que o `pick_hovered_object` já declara para as formas: *«um
/// realce que acendesse outra coisa que a que o clique pega seria pior que não haver realce
/// nenhum»*. Aqui isso é mais forte ainda, porque as duas alças estão **uma dentro da outra** e
/// executam verbos diferentes.
///
/// ⭐⭐⭐ **E ele é MODAL, porque o clique é** (ordem do dono, 2026-09-09): em *Criar* existe **um**
/// alvo — a ponta de onde um filho nasce — e mais nada. Acender o corpo ali prometeria o verbo de
/// girar, que naquele modo o press não executa; e a **junta** de um filho está desenhada em cima da
/// ponta do pai, logo sem esta separação o realce acenderia o osso ERRADO exactamente no ponto que
/// decide o parentesco.
pub fn hover(
    sim: &SimWorld,
    world: [f64; 2],
    px_to_world: f64,
    focused: Option<u64>,
    action: ph2d_tool_bone::BoneAction,
) -> Option<ph2d_skeleton_render::BoneHover> {
    use ph2d_skeleton_render::{BoneHover, BonePart};
    // ⭐⭐⭐ **O PINCEL DE PESO não agarra osso nenhum, e por isso não REALÇA nenhum.**
    if action == ph2d_tool_bone::BoneAction::Weight {
        return None;
    }
    if action == ph2d_tool_bone::BoneAction::Create {
        return tip_at(sim, world, px_to_world).map(|(bone, _)| BoneHover {
            bone,
            part: BonePart::Tip,
        });
    }
    // ⭐⭐⭐ **AS TRÊS ALÇAS DO OSSO EM FOCO COMPETEM POR PROXIMIDADE, e não por ordem.**
    if let Some(f) = focused {
        let mut alcas: Vec<([f64; 2], BonePart)> = Vec::new();
        if let Some((r, a, b)) = ph2d_skeleton_live::skin_live::influence_region(sim, f)
            && let Some(h) = ph2d_skeleton_render::influence_handle(a, b, r)
        {
            alcas.push((h, BonePart::Influence));
        }
        let mut perto = alcas
            .into_iter()
            .map(|(p, q)| ((p[0] - world[0]).hypot(p[1] - world[1]), q))
            .collect::<Vec<_>>();
        // ⭐⭐⭐ **A PAREDE INTEIRA é agarrável, e não só o triângulo na ponta dela.**
        if let Some(arc) = crate::bone_limit::arc(sim, Entity::from_bits(f), px_to_world) {
            for (e, q) in [
                (arc.handle_min, BonePart::LimitMin),
                (arc.handle_max, BonePart::LimitMax),
            ] {
                perto.push((
                    ph2d_skeleton::dist2_to_segment(world, arc.apex, e).sqrt(),
                    q,
                ));
            }
        }
        // ⭐⭐⭐ **AS DUAS ALÇAS DE CURVATURA, na mesma competição por proximidade.**
        if let Some(([inn, out], _)) = ph2d_skeleton_live::skin_live::bend_handles(sim, f) {
            for (p, q) in [(inn, BonePart::BendIn), (out, BonePart::BendOut)] {
                perto.push(((p[0] - world[0]).hypot(p[1] - world[1]), q));
            }
        }
        // ⚠️ **E o OSSO entra na mesma competição.** As paredes cruzam o osso sempre que ele está
        // perto de uma delas, então sem isto a cura de cima devolveria o defeito anterior ao
        // contrário: a parede roubaria o gesto de girar em toda a faixa. *Ganha o que está mais
        // perto do dedo* — a regra que não escolhe uma vítima, agora sobre todos os alvos do osso.
        let d_osso = corpo(sim, f).map_or(f64::INFINITY, |pts| {
            ph2d_skeleton::dist2_to_polyline(world, &pts).sqrt()
        });
        let melhor = perto
            .into_iter()
            .filter(|&(d, q)| {
                // ⛔⛔ **A alça de CURVATURA é a única que ignora o `d_osso`, e é obrigatório:** no
                let curvatura = matches!(q, BonePart::BendIn | BonePart::BendOut);
                let raio = if curvatura { BEND_HIT_PX } else { BONE_HIT_PX };
                d <= raio * px_to_world && (curvatura || d < d_osso)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((_, part)) = melhor {
            return Some(BoneHover { bone: f, part });
        }
    }
    // ⭐⭐⭐ **A ÂNCORA vem ANTES do [`hit`], e a razão é geométrica:** ela pode estar longe de todo
    if let Some((b, ..)) = crate::goal::anchors(sim)
        .into_iter()
        .find(|&(bits, a, o, p)| {
            let d = (a[0] - world[0]).hypot(a[1] - world[1]) / px_to_world.max(f64::MIN_POSITIVE);
            let comp = (p[0] - o[0]).hypot(p[1] - o[1]) / px_to_world.max(f64::MIN_POSITIVE);
            // ⚠️ O miolo só se descarta se houver mesmo um osso lá dentro: com a âncora longe de
            // tudo, o disco inteiro é dela — senão o centro do losango seria um buraco morto.
            let miolo = ph2d_skeleton_render::joint_radius_px(comp);
            let tapado = hit(sim, world, px_to_world).is_some_and(|h| h != bits)
                || grabbed_the_joint(Some(sim), bits, world, px_to_world);
            d <= ph2d_skeleton_render::goal_radius_px(comp) && !(tapado && d <= miolo)
        })
    {
        return Some(BoneHover {
            bone: b,
            part: BonePart::Tip,
        });
    }
    let bone = hit(sim, world, px_to_world)?;
    // ⭐ **A PONTA antes da junta e do corpo**: ela vive DENTRO do raio de acerto do osso, então
    // sem esta ordem o corpo ganhava-a sempre e o *end effector* seria inalcançável.
    // ⚠️ E só existe onde ela é desenhada — em quem fecha a corrente **e não tem âncora**: num osso
    // ancorado o desenho põe o losango na âncora e a alça foi já apanhada acima, então repetir aqui
    // daria duas respostas para o mesmo dedo.
    if ph2d_skeleton_live::skin_live::chain_ends(sim).contains(&bone)
        && ph2d_constraints_ecs::ancora_de(sim.world(), Entity::from_bits(bone)).is_none()
        && let Some(pts) = corpo(sim, bone)
    {
        let b = *pts.last().expect("a polilinha tem ao menos dois nos");
        if (b[0] - world[0]).hypot(b[1] - world[1])
            <= ph2d_skeleton_render::joint_radius_px(comp_px(&pts, px_to_world)) * px_to_world
        {
            return Some(BoneHover {
                bone,
                part: BonePart::Tip,
            });
        }
    }
    Some(BoneHover {
        bone,
        part: if grabbed_the_joint(Some(sim), bone, world, px_to_world) {
            BonePart::Joint
        } else {
            BonePart::Body
        },
    })
}

/// ⭐⭐ **Os candidatos sob o dedo, em Object** (A14): o OSSO à vista sob o ponteiro e, a seguir, o
/// esqueleto dono dele — o 1.º clique escolhe o osso, o seguinte no mesmo sítio passa ao esqueleto
/// (o ciclo do pick). ⛔ Só o esqueleto (a Armature do Blender) deixava o osso inalcançável fora do
/// Pose, e no Pose o cadeado recusa a forma do Bind (report do dono, 06/10).
#[must_use]
pub fn objects_at(sim: &SimWorld, world: [f64; 2], px_to_world: f64) -> Vec<u64> {
    let Some(osso) = hit(sim, world, px_to_world) else {
        return Vec::new();
    };
    let dono = ph2d_skeleton_ecs::skeleton_of(sim.world(), Entity::from_bits(osso));
    std::iter::once(osso)
        .chain(dono.map(Entity::to_bits))
        .collect()
}

/// ⭐ **A semente do Bind** — o osso escolhido, senão o 1.º osso de um esqueleto escolhido (em Object
/// a selecção é o OBJECTO esqueleto, não um osso dele).
#[must_use]
pub fn bind_seed(sim: &SimWorld, selection: &[u64]) -> Option<u64> {
    crate::bone_gesture::selected_bone(sim, selection.iter().copied()).or_else(|| {
        selection.iter().find_map(|&b| {
            let e = Entity::try_from_bits(b)?;
            sim.world().get::<ph2d_skeleton_ecs::Skeleton>(e)?;
            ph2d_skeleton_ecs::bones_of(sim.world(), e)
                .first()
                .map(|o| o.to_bits())
        })
    })
}

/// ⭐ **As formas do Bind** — as da caneta E as formas-objecto da selecção (A14: em Object a forma
/// escolhida é um OBJECTO do gizmo, e a caneta pode estar vazia). Sem repetidos, pela ordem da caneta.
#[must_use]
pub fn bind_paths(
    pen: &[ph2d_vec_scene::VecPathId],
    entities: &ph2d_vec_entities::entities::VecEntityMap,
    selection: &[u64],
) -> Vec<ph2d_vec_scene::VecPathId> {
    let mut ids = pen.to_vec();
    for b in selection {
        if let Some((pid, _)) = entities.iter().find(|(_, x)| **x == *b)
            && !ids.contains(pid)
        {
            ids.push(*pid);
        }
    }
    ids
}

#[cfg(test)]
#[path = "bone_pick_tests.rs"]
mod tests;

/// **A ponta de um osso, em MUNDO** — delegação para [`ph2d_skeleton_live::bone::tip_of`].
pub fn tip_of(sim: &SimWorld, bits: u64) -> Option<[f64; 2]> {
    ph2d_skeleton_live::bone::tip_of(sim, bits)
}
