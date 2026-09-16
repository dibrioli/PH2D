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
    ph2d_skeleton_live::skin_live::bone_polylines(sim)
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
    for (bits, pts) in ph2d_skeleton_live::skin_live::bone_polylines(sim) {
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
    for (bits, pts) in ph2d_skeleton_live::skin_live::bone_polylines(sim) {
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
    for (bits, pts) in ph2d_skeleton_live::skin_live::bone_polylines(sim) {
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
        // ([`ph2d_skeleton_render::joint_radius_px`]): são duas perguntas — *acertei o osso?* e
        // *acertei a junta DELE?* — mas a bolinha que o dedo procura tem de ser exactamente a
        // que o olho vê, senão o realce acende num sítio e o clique pega noutro.
        //
        // ⚠️ **O comprimento entra em píxeis de TELA**, que é onde a lei da bolinha vive: o
        // `px_to_world` é a régua, e dividir por ele é o que leva o osso do mundo para lá.
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
    action: ph2d_tool_vector::BoneAction,
) -> Option<ph2d_skeleton_render::BoneHover> {
    use ph2d_skeleton_render::{BoneHover, BonePart};
    if action == ph2d_tool_vector::BoneAction::Create {
        return tip_at(sim, world, px_to_world).map(|(bone, _)| BoneHover {
            bone,
            part: BonePart::Tip,
        });
    }
    // ⭐⭐⭐ **AS TRÊS ALÇAS DO OSSO EM FOCO COMPETEM POR PROXIMIDADE, e não por ordem.**
    //
    // São a alça da FORÇA (o quadrado na borda da mancha) e as DUAS PAREDES do limite de ângulo (os
    // triângulos nas pontas do arco). Todas se desenham só para o osso em foco, então o dedo faz
    // exactamente a mesma pergunta — ⛔ uma alça agarrável onde nada está desenhado é pior que uma
    // alça ausente.
    //
    // ⚠️⚠️ **A 1.ª redacção testava-as por ORDEM (força, depois paredes) e o gate apanhou-a:** com
    // `strength ≈ 1` e uma parede perto de 90° as duas caem a menos de um dedo uma da outra, e a
    // parede ficava **inalcançável** — o artista via o triângulo e agarrava o quadrado. Reordenar
    // só trocaria quem fica inalcançável.
    //
    // ⇒ **ganha a mais PERTO do ponteiro.** É determinístico, não tem lado arbitrário, e é a única
    // regra que não escolhe uma vítima. As alças vivem FORA do eixo do osso, logo elas continuam a
    // vir antes do corpo: se o corpo ganhasse, um osso vizinho largo engoliria a alça deste.
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
        //
        // ⛔⛔ **Report do dono (2026-09-08): *«não consigo mover os gizmos dos ângulos»*.** Depois
        // de a alça sair para fora do alcance do osso, o único alvo ficou a `17 px` ALÉM da borda
        // do setor — e é a borda do setor que se lê como *«a parede»*. O artista mirava no que via
        // e não havia alvo nenhum ali: um triângulo de `5 px` a `17 px` do sítio para onde a mão
        // vai. *Um alvo que não está onde a coisa PARECE estar é um alvo ausente.*
        //
        // ⇒ o alvo é o **SEGMENTO** do vértice até a alça, que é exactamente o traço desenhado.
        // Ele é grande, está debaixo do que o artista vê, e passa pelo triângulo por construção.
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
        //
        // ⚠️ **Elas só entram onde são DESENHADAS** — o [`ph2d_skeleton_live::skin_live::bend_handles`]
        // devolve `None` num osso sem segmentos, e é a mesma porta que o overlay chama. *Uma alça
        // agarrável onde nada está pintado é pior que uma alça ausente*, e o contrário — pintada e
        // não agarrável — foi o report *«não consigo mover os gizmos dos ângulos»* deste módulo.
        //
        // ⚠️ **O alvo é a BOLINHA e não a haste**, ao contrário das paredes do limite: a haste da
        // alça de curvatura nasce na junta e na ponta, que já são alvos com outros verbos, e um
        // segmento agarrável por cima delas roubaria os dois.
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
                // ponto NEUTRO ela está **em cima do eixo** (é o ponto de controlo no terço), logo
                // `d ≈ d_osso ≈ 0` e a comparação estrita tornava-a **inalcançável no único estado
                // em que todo osso nasce**. *Uma alça que só se agarra depois de já ter sido
                // movida não se agarra nunca.*
                //
                // ⇒ em troca ela paga um raio APERTADO ([`BEND_HIT_PX`]), que é o que deixa o
                // resto do osso a executar o verbo de girar.
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
    // osso (fora de alcance, ou porque o artista a arrastou para lá), e o `hit` só devolve um osso
    // quando o ponteiro está a `BONE_HIT_PX` do **segmento**. Testá-la depois tornaria a alça
    // inalcançável exactamente quando ela mais se distingue da ponta.
    //
    // ⭐⭐⭐ **E ela é um ANEL, não um disco** (report do dono, 2026-09-07: *«o losango do IK e o
    // círculo do outro osso ficam sobrepostos»*). Uma âncora criada no MEIO de uma corrente nasce
    // em cima da junta do osso seguinte — dois alvos concêntricos com verbos diferentes. A regra é
    // a que ele propôs: **por fora da bolinha pega a ÂNCORA, por dentro pega o osso**, e é o furo
    // no meio deste teste que deixa o clique de dentro chegar ao [`hit`] lá abaixo.
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
        && sim
            .world()
            .get::<ph2d_skeleton_ecs::IkGoal>(Entity::from_bits(bone))
            .is_none()
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

/// ⭐⭐⭐ **QUE ALÇAS DE OSSO PEGAM FORA DO MODO OSSO** — a linha é o **VERBO**, não a alça.
///
/// ⛔⛔ **Achado da auditoria de 2026-09-08:** o arco de limite, a alça da força e a ponta da
/// corrente são **pintados e ACENDEM sob o rato nos 14 modos de vector** — o
/// [`crate::app_state::App::refresh_bone_hover`] não se gateia pelo modo, de propósito, porque o
/// `vec_overlay::bones` também não —, e o `Down` só era lido dentro do `DrawMode::Bone`. *Um
/// controlo que acende debaixo do dedo e não responde é a espécie de morto que este repo já pagou
/// três vezes.*
///
/// ⭐ **Entram as quatro que NENHUMA outra ferramenta sabe exprimir:** a força, as duas paredes do
/// limite e a ponta da corrente (a cinemática inversa).
///
/// ⛔ **Ficam de fora [`BonePart::Body`] (girar) e [`BonePart::Joint`] (deslocar):** o gizmo de
/// sprite já faz as duas, e roubar-lhas aqui trocaria a lei do arrasto da seta **em silêncio** —
/// o artista escolhe um osso com a seta e o arrasto passa a fazer outra coisa.
///
/// ⚠️ **É um `match` exaustivo e não uma lista:** uma parte NOVA é erro de compilação aqui, que é
/// o único sítio onde alguém tem de responder *«este verbo existe noutra ferramenta?»*.
pub fn grabbable_outside_bone_mode(part: ph2d_skeleton_render::BonePart) -> bool {
    use ph2d_skeleton_render::BonePart;
    match part {
        // ⭐ As duas alças de CURVATURA entram pela mesma razão que a força: **nenhuma outra
        // ferramenta sabe arquear um osso**, e elas só existem num osso com segmentos — logo não
        // disputam com nada.
        BonePart::Influence
        | BonePart::LimitMin
        | BonePart::LimitMax
        | BonePart::Tip
        | BonePart::BendIn
        | BonePart::BendOut => true,
        BonePart::Body | BonePart::Joint => false,
    }
}

#[cfg(test)]
#[path = "bone_pick_tests.rs"]
mod tests;

/// **A ponta de um osso, em MUNDO** — delegação para [`ph2d_skeleton_live::bone::tip_of`].
pub fn tip_of(sim: &SimWorld, bits: u64) -> Option<[f64; 2]> {
    ph2d_skeleton_live::bone::tip_of(sim, bits)
}
