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

use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform};
use ph2d_skeleton_ecs::Bone;
use ph2d_vec_scene::Xform;

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

/// **A ponta de um osso, em MUNDO** — para o encaixe do próximo nascer colado nela.
pub(crate) fn tip_of(sim: &SimWorld, bits: u64) -> Option<[f64; 2]> {
    crate::skeleton_live::bone_segments(sim)
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
            Bone {
                length,
                ..Bone::default()
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
    /// **Transformar:** acertou um osso — selecciona-o e ARMA o verbo que a `part` diz.
    Grab {
        bone: u64,
        part: ph2d_skeleton_render::BonePart,
    },
    /// **Criar:** acertou um osso — apenas o SELECCIONA. É assim que se escolhe onde ramificar, e
    /// ⛔ sem armar pose nenhuma: neste verbo o arrasto é para fazer um osso, não para posar.
    Select { bone: u64 },
    /// **Criar:** não acertou osso — marca a ORIGEM de um osso novo (já encaixada na ponta do osso
    /// aceso) e diz que FORMA estava sob o cursor, que é a metade que o *Bind* precisa.
    Start {
        origin: [f64; 2],
        pick: Option<ph2d_vec_scene::VecPathId>,
    },
    /// **Transformar:** não acertou osso — aponta a forma sob o cursor e mais nada. ⛔ Nenhum osso
    /// nasce neste verbo, nem por arrasto longo.
    Pick {
        path: Option<ph2d_vec_scene::VecPathId>,
    },
}

/// A decisão do press, sem tocar em nada.
///
/// ⭐⭐⭐ **O `action` é o que resolve a ambiguidade do PONTEIRO** (Enio, 2026-09-07: *«do modo como
/// está fica confuso para o usuário»*). Antes, o mesmo arrasto criava OU posava consoante o que
/// estava por baixo do cursor — e isso torna inalcançáveis dois gestos legítimos: começar um osso
/// **em cima** de outro, e posar um osso **sem medo** de criar um por engano.
pub(crate) fn press(
    sim: &SimWorld,
    scene: &ph2d_vec_scene::VecScene,
    pen: &ph2d_vec_edit::PenTool,
    world: [f64; 2],
    px_to_world: f64,
    selected: Option<u64>,
    action: ph2d_tool_vector::BoneAction,
) -> BonePress {
    use ph2d_tool_vector::BoneAction;
    // ⭐⭐⭐ **O clique PERGUNTA AO REALCE** — não é uma segunda varredura que um gate compara com a
    // primeira: é a MESMA função. O que o artista vê aceso é, por construção, o que ele vai pegar.
    let foco = selected_bone(sim, selected);
    if let Some(h) = hover(sim, world, px_to_world, foco) {
        return match action {
            BoneAction::Transform => BonePress::Grab {
                bone: h.bone,
                part: h.part,
            },
            BoneAction::Create => BonePress::Select { bone: h.bone },
        };
    }
    if action == BoneAction::Transform {
        // ⛔ Em *Transformar* um press no vazio **não arma osso nenhum** — ele só aponta a forma,
        // que é o que o *Bind* precisa. É esta linha que faz o verbo ser um só.
        return BonePress::Pick {
            path: pen.path_at(scene, world, BONE_HIT_PX * px_to_world),
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
    let origin = foco.and_then(|b| tip_of(sim, b)).unwrap_or(world);
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
pub(crate) fn hover(
    sim: &SimWorld,
    world: [f64; 2],
    px_to_world: f64,
    focused: Option<u64>,
) -> Option<ph2d_skeleton_render::BoneHover> {
    use ph2d_skeleton_render::{BoneHover, BonePart};
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

/// ⭐⭐⭐ **A MIRA** — que rotação LOCAL este osso precisa de ter para apontar a `world`?
///
/// ⚠️ **Porta única de dois consumidores**: o gesto que gira um osso à mão ([`pose`]) e a restrição
/// que o gira a cada quadro ([`crate::skeleton_goal`]). Escrita duas vezes, ela divergiria na
/// primeira vez que alguém corrigisse a composição do espaço do pai — e o sintoma seria o osso a
/// saltar entre o que o dedo faz e o que a âncora faz, que é indistinguível de um defeito da
/// própria cinemática.
///
/// ⚠️ **A pose é LOCAL do pai, e deriva-se levando o PONTO ao espaço dele** — nunca subtraindo
/// ângulos. Compor `rot − rot_do_pai` só está certo com um pai conforme; transformar o ponto está
/// certo com qualquer afim.
///
/// `None` quando o pai é singular, ou quando `world` cai **sobre a própria origem** do osso: ali não
/// há direcção nenhuma, e apontar para lá daria um ângulo arbitrário — o osso saltaria.
#[must_use]
pub(crate) fn aim_rotation(sim: &SimWorld, bone: Entity, world: [f64; 2]) -> Option<f64> {
    let pai = sim.world().get::<ChildOf>(bone).map(ChildOf::parent);
    let pai_mundo = pai.map_or(Xform::IDENTITY, |p| {
        crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(sim, p))
    });
    let inv = pai_mundo.inverse()?;
    let p = inv.apply(world);
    let t = sim.world().get::<Transform>(bone)?;
    let o = [f64::from(t.translation.x), f64::from(t.translation.y)];
    let d = [p[0] - o[0], p[1] - o[1]];
    (d[0].hypot(d[1]) >= f64::EPSILON).then(|| d[1].atan2(d[0]))
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
        .find(|&b| sim.world().get::<Bone>(Entity::from_bits(b)).is_some())
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

/// **O segmento de MUNDO de um osso** — a fixtura que os dois módulos de teste partilham.
///
/// ⚠️ Ela vive aqui, e não num deles, porque os dois a usam: uma cópia por ficheiro divergiria no
/// primeiro ajuste, e é o mesmo defeito que este módulo já curou no raio da junta.
#[cfg(test)]
pub(crate) fn test_segment(sim: &SimWorld, bits: u64) -> ([f64; 2], [f64; 2]) {
    crate::skeleton_live::bone_segments(sim)
        .into_iter()
        .find(|(b, _, _)| *b == bits)
        .map(|(_, a, b)| (a, b))
        .expect("o osso")
}

/// Uma corrente de `n` ossos de 10 unidades, deitada sobre o eixo X. Devolve `[raiz, .., ponta]`.
#[cfg(test)]
pub(crate) fn test_chain(sim: &mut SimWorld, n: usize) -> Vec<u64> {
    let mut out = Vec::new();
    let mut pai = None;
    for i in 0..n {
        let x = f64::from(u16::try_from(i).unwrap_or(0)) * 10.0;
        let b = create(sim, pai, [x, 0.0], [x + 10.0, 0.0]).expect("osso");
        pai = Some(Entity::from_bits(b));
        out.push(b);
    }
    out
}

#[cfg(test)]
#[path = "bone_gesture_tests.rs"]
mod tests;


impl crate::App {
    /// **Resolve a metade de osso sob o ponteiro**, uma vez por quadro
    /// ([`crate::App::bone_hover`]).
    ///
    /// ⚠️ **Sem ponteiro no canvas ⇒ LIMPA**, como o realce do Trim e o do Balde: um realce que
    /// sobrevive ao cursor sair da tela é uma alça que finge estar apontada.
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
        self.bone_hover = self
            .gfx
            .as_ref()
            .and_then(|gfx| hover(&gfx.sim, world, px_to_world, foco));
        // ⭐ E o osso que está a NASCER, pela mesma leitura do ponteiro.
        self.bone_preview = self
            .vec_bone_drag
            .map(|o| (o, world, drag_makes_a_bone(o, world, px_to_world)));
    }
}

/// ⭐⭐⭐ **A CORRENTE ALCANÇA `goal`** — cinemática inversa sobre a cadeia que acaba em `tip`.
///
/// ⚠️⚠️ **A escrita de volta passa pela MESMA porta que a rotação à mão** (`pose(.., Body)`), osso
/// a osso e **do pai para o filho**. É isso que garante três coisas de graça: os comprimentos ficam
/// (uma rotação não estica), a pose guardada continua a ser LOCAL do pai, e o que a IK escreve é
/// indistinguível do que o artista escreveria a rodar cada osso — logo o undo, o save e a timeline
/// não precisam de saber que a IK existe.
///
/// ⛔ **A ordem pai→filho é load-bearing**: cada `pose` lê o mundo do pai para converter o alvo para
/// local, e um filho resolvido antes do pai leria um mundo que ainda vai mudar.
pub(crate) fn reach_chain(sim: &mut SimWorld, tip: Entity, goal: [f64; 2]) -> bool {
    let cadeia = crate::skeleton_live::chain_to(sim, tip.to_bits());
    let segs = crate::skeleton_live::bone_segments(sim);
    let mut juntas: Vec<[f64; 2]> = Vec::with_capacity(cadeia.len() + 1);
    let mut comps: Vec<f64> = Vec::with_capacity(cadeia.len());
    for &e in &cadeia {
        let Some((_, a, b)) = segs.iter().copied().find(|(x, _, _)| *x == e.to_bits()) else {
            return false;
        };
        juntas.push(a);
        comps.push((b[0] - a[0]).hypot(b[1] - a[1]));
        if e == tip {
            juntas.push(b);
        }
    }
    if comps.is_empty() || juntas.len() != comps.len() + 1 {
        return false;
    }
    ph2d_skeleton::reach(&mut juntas, &comps, goal, ph2d_skeleton::Reach::default());
    let mut mexeu = false;
    for (i, &e) in cadeia.iter().enumerate() {
        mexeu |=
            crate::bone_pose::pose(sim, e, juntas[i + 1], ph2d_skeleton_render::BonePart::Body);
    }
    mexeu
}

/// ⭐⭐⭐ **ESTE ARRASTO CHEGA A FAZER UM OSSO?** — a porta única do `Up` e da pré-visualização.
///
/// ⛔ Um arrasto mais curto que o raio das alças **não** faz osso: um osso de comprimento zero não
/// tem eixo, logo não pesa ponto nenhum e é invisível — seria lixo que só o `Delete` da Hierarquia
/// acha. O limiar é em píxeis de TELA, então basta aproximar o zoom para fazer um menor.
///
/// ⚠️⚠️ **Ela existe porque a PRÉ-VISUALIZAÇÃO nasceu** (Enio, 2026-09-07). Enquanto o osso só
/// aparecia no `Up`, esta decisão podia viver lá dentro; com o desenho vivo, a mesma pergunta passa
/// a ter **dois** leitores — e o `CLAUDE.md` §5.0 diz o que acontece quando eles divergem: *uma
/// cena que ensina o contrário do que acontece é pior que uma cena ausente*. O artista veria um
/// osso a crescer e o `Up` não faria nada.
#[must_use]
pub(crate) fn drag_makes_a_bone(origin: [f64; 2], tip: [f64; 2], px_to_world: f64) -> bool {
    (tip[0] - origin[0]).hypot(tip[1] - origin[1]) >= BONE_HIT_PX * px_to_world
}
