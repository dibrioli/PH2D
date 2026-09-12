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
pub(crate) const BONE_HIT_PX: f64 = 12.0;

/// **O osso sob o ponteiro** (o mais próximo dentro do raio), ou `None`.
pub(crate) fn hit(sim: &SimWorld, world: [f64; 2], px_to_world: f64) -> Option<u64> {
    let r = BONE_HIT_PX * px_to_world;
    let mut melhor: Option<(f64, u64)> = None;
    for (bits, a, b) in crate::skeleton_live::bone_segments(sim) {
        let d2 = ph2d_skeleton::dist2_to_segment(world, a, b);
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
pub(crate) fn tip_at(sim: &SimWorld, world: [f64; 2], px_to_world: f64) -> Option<(u64, [f64; 2])> {
    let mut melhor: Option<(f64, u64, [f64; 2])> = None;
    for (bits, a, b) in crate::skeleton_live::bone_segments(sim) {
        let comp_px = (b[0] - a[0]).hypot(b[1] - a[1]) / px_to_world.max(f64::MIN_POSITIVE);
        let r = ph2d_skeleton_render::joint_radius_px(comp_px) * px_to_world;
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
/// ⚠️ **A régua de «tem pai» é a MESMA do [`crate::skeleton_live::chain_to`]** — o pai existe **e**
/// é osso. Um esqueleto pendurado dentro de um grupo continua a ser uma corrente solta, que é o que
/// o resto do módulo já assume.
///
/// ⚠️ Raio e desempate iguais aos do [`tip_at`]: a bolinha DESENHADA, e ganha a mais perto.
pub(crate) fn free_root_at(
    sim: &SimWorld,
    world: [f64; 2],
    px_to_world: f64,
) -> Option<(u64, [f64; 2])> {
    let mut melhor: Option<(f64, u64, [f64; 2])> = None;
    for (bits, a, b) in crate::skeleton_live::bone_segments(sim) {
        if !is_a_free_chain_root(sim, bits) {
            continue;
        }
        let comp_px = (b[0] - a[0]).hypot(b[1] - a[1]) / px_to_world.max(f64::MIN_POSITIVE);
        let r = ph2d_skeleton_render::joint_radius_px(comp_px) * px_to_world;
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
/// mesma regra que o [`crate::skeleton_live::chain_to`] usa para parar de subir. Escrita duas vezes,
/// ela divergiria no dia em que alguém decidisse o que um osso dentro de um grupo é.
#[must_use]
pub(crate) fn is_a_free_chain_root(sim: &SimWorld, bits: u64) -> bool {
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
pub(crate) fn grabbed_the_joint(
    sim: Option<&SimWorld>,
    bits: u64,
    world: [f64; 2],
    px_to_world: f64,
) -> bool {
    let Some(sim) = sim else {
        return false;
    };
    crate::skeleton_live::bone_segments(sim)
        .into_iter()
        .find(|(b, _, _)| *b == bits)
        .is_some_and(|(_, a, b)| {
            // ⚠️ O raio é o da BOLINHA DESENHADA, pela porta única
            // ([`ph2d_skeleton_render::joint_radius_px`]): são duas perguntas — *acertei o osso?* e
            // *acertei a junta DELE?* — mas a bolinha que o dedo procura tem de ser exactamente a
            // que o olho vê, senão o realce acende num sítio e o clique pega noutro.
            //
            // ⚠️ **O comprimento entra em píxeis de TELA**, que é onde a lei da bolinha vive: o
            // `px_to_world` é a régua, e dividir por ele é o que leva o osso do mundo para lá.
            let comp_px = (b[0] - a[0]).hypot(b[1] - a[1]) / px_to_world.max(f64::MIN_POSITIVE);
            (a[0] - world[0]).hypot(a[1] - world[1])
                <= ph2d_skeleton_render::joint_radius_px(comp_px) * px_to_world
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
pub(crate) fn hover(
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
        if let Some((r, a, b)) = crate::skeleton_live::influence_region(sim, f)
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
        // ⚠️ **E o OSSO entra na mesma competição.** As paredes cruzam o osso sempre que ele está
        // perto de uma delas, então sem isto a cura de cima devolveria o defeito anterior ao
        // contrário: a parede roubaria o gesto de girar em toda a faixa. *Ganha o que está mais
        // perto do dedo* — a regra que não escolhe uma vítima, agora sobre todos os alvos do osso.
        let d_osso = crate::skeleton_live::bone_segments(sim)
            .into_iter()
            .find(|(x, _, _)| *x == f)
            .map_or(f64::INFINITY, |(_, a, b)| {
                ph2d_skeleton::dist2_to_segment(world, a, b).sqrt()
            });
        let melhor = perto
            .into_iter()
            .filter(|&(d, _)| d <= BONE_HIT_PX * px_to_world && d < d_osso)
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
    if let Some((b, ..)) =
        crate::skeleton_goal::anchors(sim)
            .into_iter()
            .find(|&(bits, a, o, p)| {
                let d =
                    (a[0] - world[0]).hypot(a[1] - world[1]) / px_to_world.max(f64::MIN_POSITIVE);
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
    if crate::skeleton_live::chain_ends(sim).contains(&bone)
        && sim
            .world()
            .get::<ph2d_skeleton_ecs::IkGoal>(Entity::from_bits(bone))
            .is_none()
        && let Some((_, a, b)) = crate::skeleton_live::bone_segments(sim)
            .into_iter()
            .find(|(x, _, _)| *x == bone)
    {
        let comp_px = (b[0] - a[0]).hypot(b[1] - a[1]) / px_to_world.max(f64::MIN_POSITIVE);
        if (b[0] - world[0]).hypot(b[1] - world[1])
            <= ph2d_skeleton_render::joint_radius_px(comp_px) * px_to_world
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
pub(crate) fn grabbable_outside_bone_mode(part: ph2d_skeleton_render::BonePart) -> bool {
    use ph2d_skeleton_render::BonePart;
    match part {
        BonePart::Influence | BonePart::LimitMin | BonePart::LimitMax | BonePart::Tip => true,
        BonePart::Body | BonePart::Joint => false,
    }
}

impl crate::App {
    /// **Resolve a metade de osso sob o ponteiro**, uma vez por quadro
    /// ([`crate::App::bone_hover`]).
    ///
    /// ⚠️ **Sem ponteiro no canvas ⇒ LIMPA**, como o realce do Trim e o do Balde: um realce que
    /// sobrevive ao cursor sair da tela é uma alça que finge estar apontada.
    /// ⭐⭐⭐ **A alça de osso sob este ponto, se o VERBO dela não existir noutra ferramenta.**
    ///
    /// ⚠️ **A MESMA porta do realce** ([`hover`]) — o que o artista vê aceso é, por construção, o
    /// que ele vai pegar. Uma segunda varredura seria a segunda resposta à mesma pergunta.
    pub(crate) fn bone_handle_at(
        &self,
        pointer: (f32, f32),
    ) -> Option<ph2d_skeleton_render::BoneHover> {
        let world = self.vec_world_at(pointer)?;
        let px = self.vec_px_to_world();
        let foco = self.selected_bone_bits();
        // ⚠️ **Sempre *Transformar*, e não o verbo armado**: esta porta é a de FORA do modo Osso, e
        // ali o que existe são os verbos que nenhuma outra ferramenta sabe exprimir
        // ([`grabbable_outside_bone_mode`]). O verbo *Criar* não é um deles.
        let h = hover(
            &self.gfx.as_ref()?.sim,
            world,
            px,
            foco,
            ph2d_tool_vector::BoneAction::Transform,
        )?;
        grabbable_outside_bone_mode(h.part).then_some(h)
    }

    pub(crate) fn refresh_bone_hover(&mut self, pointer: (f32, f32)) {
        // ⚠️⚠️ **OS DOIS SLOTS SAEM DA MESMA LEITURA, e a saída antecipada tem de limpar OS DOIS.**
        // A 1.ª redacção desta função limpava só o realce e deixava a pré-visualização congelada:
        // o cursor sai do canvas e um osso fantasma fica desenhado na tela até ao gesto seguinte.
        // *Um par de slots resolvido no mesmo sítio esquece-se meio a meio.*
        let Some(world) = self.vec_world_at(pointer) else {
            self.bone_hover = None;
            self.bone_preview = None;
            return;
        };
        let px_to_world = self.vec_px_to_world();
        // ⚠️ O osso em FOCO entra: a alça da força só existe onde ela se desenha, e o que a desenha
        // é a selecção. Sem ele o dedo procuraria uma alça que não está na tela.
        let foco = self.selected_bone_bits();
        // ⚠️ **O verbo só manda DENTRO do modo Osso.** Fora dele o `bone_action` continua guardado
        // (ele é estado da ferramenta, não do quadro), e lê-lo aqui deixaria o realce das outras 13
        // ferramentas preso no que o artista armou da última vez que passou pelo esqueleto.
        let acao = if self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Bone {
            self.vec_draw_config.bone_action
        } else {
            ph2d_tool_vector::BoneAction::Transform
        };
        // ⭐⭐⭐ **O QUE O ARRASTO SIGNIFICA AGORA** — a porta ÚNICA que o release também lê
        // ([`crate::bone_gesture::drag_now`]). ⛔ É ela que faz o que o artista VÊ ser o que ele
        // RECEBE: a emenda, a ponta encaixada e o limiar saem todos da mesma leitura.
        let agora = self.vec_bone_drag.and_then(|n| {
            let gfx = self.gfx.as_ref()?;
            Some(crate::bone_gesture::drag_now(
                &gfx.sim,
                n,
                world,
                px_to_world,
            ))
        });
        // ⭐⭐⭐ **COM UM ARRASTO VIVO, o realce responde ao RELEASE e não ao press.**
        //
        // ⚠️ São perguntas diferentes — *o que um press aqui faria?* contra *o que este soltar vai
        // fazer?* — e durante um arrasto só a segunda tem sentido: o press já aconteceu. A bolinha
        // que acende é a que o osso novo vai agarrar.
        self.bone_hover = match agora.and_then(|a| a.splice) {
            Some((alvo, _)) => Some(ph2d_skeleton_render::BoneHover {
                bone: alvo,
                part: ph2d_skeleton_render::BonePart::Joint,
            }),
            None => self
                .gfx
                .as_ref()
                .and_then(|gfx| hover(&gfx.sim, world, px_to_world, foco, acao)),
        };
        // ⭐ E o osso que está a NASCER, da MESMA leitura.
        self.bone_preview = self
            .vec_bone_drag
            .zip(agora)
            .map(|(n, a)| (n.origin, a.tip, a.armed));
    }
}

#[cfg(test)]
#[path = "bone_pick_tests.rs"]
mod tests;

/// **A ponta de um osso, em MUNDO** — delegação para [`ph2d_skeleton_live::bone::tip_of`].
pub(crate) fn tip_of(sim: &SimWorld, bits: u64) -> Option<[f64; 2]> {
    ph2d_skeleton_live::bone::tip_of(sim, bits)
}
