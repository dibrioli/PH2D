//! **O gesto do modo OSSO** (estudo 42 item 5, doc 47 §2.6) — arrastar faz um osso.
//!
//! ```text
//! press  → caiu na PONTA de um osso?  sim → o novo é FILHO dele, e parte dessa ponta
//!                                     não → o novo é uma RAIZ, e parte de onde a mão pousou
//! release→ nasce o osso: origem no press, comprimento e ângulo no arrasto
//! ```
//!
//! ⭐⭐⭐ **O PARENTESCO É UM SÍTIO NA TELA, e não o estado da selecção** (ordem do dono,
//! 2026-09-09: *«para criar um osso como filho de outro o clique deve acontecer na ponta do osso
//! pai e o usuário arrasta o mouse definindo tamanho e direção do novo osso»*).
//!
//! ⛔⛔ **A lei anterior lia a SELECÇÃO, e isso tornava um gesto legítimo INEXPRIMÍVEL:** com um
//! osso aceso, todo arrasto nascia colado à ponta dele — logo *«não se pode criar ossos fora da
//! cadeia»* (o report). Para fazer uma raiz nova era preciso primeiro desmarcar, que é um gesto de
//! cerimónia que nenhum dos alvos (Spine, Moho, Blender) pede.
//!
//! ⇒ hoje a pergunta é **geométrica e local**: a ponta é a porta, e ela está desenhada. Ramificar
//! do meio de uma corrente — a espinha que dá dois braços — passa a ser o mesmo gesto que
//! continuar uma, o que na lei antiga exigia escolher o pai na Hierarquia.
//!
//! ⚠️ **A pose é LOCAL do pai, e é derivada levando os DOIS pontos ao espaço dele.** Compor ângulos
//! e comprimentos à mão (`rot − rot_do_pai`, `len / escala_do_pai`) só está certo com um pai
//! conforme; transformar os pontos está certo com qualquer afim, e é a mesma regra-mãe do pen —
//! *o que se aponta é MUNDO; o que o documento guarda é LOCAL*.
//!
//! ⚠️ **O que o dedo APONTA mora ao lado, em [`crate::bone_pick`]** — o corte é de 2026-09-09 e é
//! por RESPONSABILIDADE, não por tamanho: aqui *o gesto que faz um osso*, ali *o que o ponteiro
//! significa*. ⛔ Continuam a ser uma lei só: o [`press`] daqui pergunta ao `hover` de lá, e há gate
//! a compará-los ponto a ponto nos dois verbos.

use crate::bone_pick::{BONE_HIT_PX, hover, tip_of};
use ph2d_ecs::{ChildOf, Entity, Name, RootOrder, SimWorld, Transform};
use ph2d_skeleton_ecs::Bone;
use ph2d_vec_scene::Xform;

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
        ph2d_vec_entities::transform::xform_of_transform(ph2d_vec_entities::transform::world_transform(sim, p))
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
            let order = ph2d_vec_entities::entities::next_root_order(sim);
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
    /// **Criar:** arma o osso que vai nascer ([`BoneBirth`] diz de onde e de quem é filho) e diz
    /// que FORMA estava sob o cursor, que é a metade que o *Bind* precisa.
    ///
    /// ⛔ **Não há um `Select` ao lado dele.** Ele existiu enquanto a selecção decidia o
    /// parentesco (*«é assim que se escolhe onde ramificar»*); com a ponta a decidir, um press
    /// sobre o CORPO de um osso não tem nada de próprio a fazer em *Criar* — e trocar a selecção
    /// ali arrancaria o artista do verbo em que ele está (a aresta do foco arma *Transform*).
    Start {
        birth: BoneBirth,
        pick: Option<ph2d_vec_scene::VecPathId>,
    },
    /// **Transformar:** não acertou osso — aponta a forma sob o cursor e mais nada. ⛔ Nenhum osso
    /// nasce neste verbo, nem por arrasto longo.
    Pick {
        path: Option<ph2d_vec_scene::VecPathId>,
    },
}

/// ⭐⭐⭐ **O OSSO QUE ESTÁ A NASCER** — de onde ele parte, e de quem é filho.
///
/// ⚠️ **Os dois viajam JUNTOS porque são a mesma decisão**, tomada no press e lida no release. A
/// 1.ª redacção desta wave guardava só a origem e ia buscar o pai à selecção no `Up` — que é
/// exactamente a lei que o dono mandou tirar, sobrevivendo no outro extremo do gesto. *Um facto
/// decidido no press e re-derivado no release é duas respostas para a mesma pergunta.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BoneBirth {
    /// A origem, em MUNDO — a ponta do pai, ou o ponto onde a mão pousou.
    pub(crate) origin: [f64; 2],
    /// O pai, em bits — `None` faz uma RAIZ.
    pub(crate) parent: Option<u64>,
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
    let alvo = hover(sim, world, px_to_world, foco, action);
    if action == BoneAction::Transform {
        return match alvo {
            Some(h) => BonePress::Grab {
                bone: h.bone,
                part: h.part,
            },
            // ⛔ Em *Transformar* um press no vazio **não arma osso nenhum** — ele só aponta a
            // forma, que é o que o *Bind* precisa. É esta linha que faz o verbo ser um só.
            None => BonePress::Pick {
                path: pen.path_at(scene, world, BONE_HIT_PX * px_to_world),
            },
        };
    }
    // ⭐⭐⭐ **EM *CRIAR* TODO PRESS ARMA UM OSSO — o que a ponta decide é de QUEM ele é filho.**
    //
    // ⛔⛔ **A lei anterior era o oposto e o dono reportou-a:** a origem saía da PONTA DO OSSO
    // ACESO, sempre, logo com um esqueleto na cena não havia press nenhum que fizesse uma raiz
    // nova. Aqui o encaixe é **local** — ele acontece onde a mão está, e a ponta que o oferece está
    // desenhada —, e é isso que o torna alcançável nos dois sentidos.
    //
    // ⚠️ **O `alvo` já é a resposta**: em *Criar* o [`hover`] devolve `Tip`, e só isso. Ir buscar a
    // ponta uma segunda vez aqui seria a segunda resposta à mesma pergunta — e o realce e o clique
    // divergiriam no primeiro ajuste do raio.
    let parent = alvo.map(|h| h.bone);
    let origin = parent
        .and_then(|b| tip_of(sim, b))
        // ⛔ Sem pai, a origem é o ponto CRU. ⚠️ Um press sobre o corpo de outro osso cai aqui de
        // propósito: começar um osso em cima de outro é um gesto legítimo, e era inexprimível
        // enquanto o corpo consumia o press para trocar a selecção.
        .unwrap_or(world);
    BonePress::Start {
        birth: BoneBirth { origin, parent },
        pick: pen.path_at(scene, world, BONE_HIT_PX * px_to_world),
    }
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
        ph2d_vec_entities::transform::xform_of_transform(ph2d_vec_entities::transform::world_transform(sim, p))
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

/// ⭐⭐⭐ **O QUE ESTE ARRASTO SIGNIFICA AGORA** — o que se DESENHA e o que o release vai FAZER.
///
/// ⚠️⚠️ **Os dois consumidores lêem esta estrutura, e é isso que os mantém de acordo**: a
/// pré-visualização ([`crate::App::refresh_bone_hover`]) e o release ([`crate::input_dispatch`]).
/// Enquanto cada um resolvesse as três coisas por si, os dois responderiam certo à própria pergunta
/// e errado um ao outro — e o sintoma seria o pior desta família: *o osso encaixa na tela e nasce
/// noutro sítio*.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BoneDragNow {
    /// A corrente solta que o release vai ADOPTAR, e a base dela em mundo. `None` = sem emenda.
    pub(crate) splice: Option<(u64, [f64; 2])>,
    /// A ponta do osso: a **base do alvo** quando há emenda, senão o ponto onde a mão está.
    pub(crate) tip: [f64; 2],
    /// Este arrasto chega a fazer um osso? — medido sobre a ponta **já encaixada**.
    pub(crate) armed: bool,
}

/// [`BoneDragNow`] para este instante do arrasto — a porta única dos dois consumidores.
#[must_use]
pub(crate) fn drag_now(
    sim: &SimWorld,
    birth: BoneBirth,
    pointer: [f64; 2],
    px_to_world: f64,
) -> BoneDragNow {
    let splice = splice_target(sim, pointer, px_to_world, birth);
    // ⭐ **O ENCAIXE vê-se antes de acontecer** — é a mesma lei do press na ponta do pai.
    let tip = splice.map_or(pointer, |(_, base)| base);
    BoneDragNow {
        splice,
        tip,
        // ⚠️ **Sobre a ponta ENCAIXADA, nunca sobre o ponteiro**: o osso que vai nascer é o
        // encaixado, e medir o outro armaria a pré-visualização num comprimento que ninguém faz.
        armed: drag_makes_a_bone(birth.origin, tip, px_to_world),
    }
}

/// ⭐⭐⭐ **ESTE ARRASTO ACABA A JUNTAR DUAS CORRENTES?** — a porta ÚNICA da emenda (ordem do dono,
/// 2026-09-09: *«tornar possível criar uma cadeia de ossos a partir de dois ossos separados ligando
/// a ponta de um com o fundo de outro ao criar um osso intermediário»*).
///
/// Devolve o osso que vai ser **ADOPTADO** e a base dele, em mundo — que é para onde a ponta do osso
/// novo encaixa.
///
/// ⚠️⚠️ **A pré-visualização e o `Up` perguntam ESTA função, e é isso que os mantém de acordo.** O
/// artista vê o osso novo saltar para a base do outro *porque é exactamente ali que ele vai nascer*;
/// uma segunda leitura do lado do release prometeria uma emenda que o gesto não faz.
///
/// ⛔ **A recusa do CICLO vive aqui e não no [`connect`]**, e a diferença é observável: se ela
/// vivesse lá, a pré-visualização prometeria a emenda e o release faria o osso **sem** ela — *um
/// gesto que promete duas coisas e entrega uma*. Aqui o alvo simplesmente não existe, e o desenho
/// diz isso.
///
/// ⚠️ O `alvo` seria filho do osso NOVO, cujo pai é o `birth.parent` — logo o ciclo é *«o alvo já
/// está acima de mim»*, e testa-se subindo a árvore INTEIRA (não só a corrente de ossos): um osso
/// pendurado num grupo que descende do alvo fecharia o laço na mesma.
#[must_use]
pub(crate) fn splice_target(
    sim: &SimWorld,
    world: [f64; 2],
    px_to_world: f64,
    birth: BoneBirth,
) -> Option<(u64, [f64; 2])> {
    let (alvo, base) = crate::bone_pick::free_root_at(sim, world, px_to_world)?;
    (!adopting_would_cycle(sim, alvo, birth.parent)).then_some((alvo, base))
}

/// **Adoptar `alvo` sob um osso novo filho de `novo_pai` fecharia um laço?**
///
/// ⚠️ É `alvo == novo_pai` **ou** `alvo` acima dele: o osso novo herda a linhagem do pai, então pôr
/// ali dentro alguém que já a contém torna a árvore cíclica — e uma travessia de `Transform` sobre
/// uma árvore cíclica não devolve, ela **pendura o app**.
#[must_use]
fn adopting_would_cycle(sim: &SimWorld, alvo: u64, novo_pai: Option<u64>) -> bool {
    let Some(alvo) = Entity::try_from_bits(alvo) else {
        return true;
    };
    let mut actual = novo_pai.and_then(Entity::try_from_bits);
    while let Some(e) = actual {
        if e == alvo {
            return true;
        }
        actual = sim.world().get::<ChildOf>(e).map(ChildOf::parent);
    }
    false
}

/// ⭐⭐⭐ **PENDURA `child` em `parent` SEM O MOVER** — o acto que junta as duas correntes.
///
/// ⚠️ **A pose de mundo do adoptado NÃO pode mudar.** Ele é uma corrente inteira que o artista já
/// posicionou; um `ChildOf` cru somaria a pose do pai novo e o esqueleto todo saltaria. A porta que
/// devolve a pose local certa já existe ([`ph2d_vec_entities::transform::reparent_keeping_world`]) e é a
/// mesma que a Hierarquia usa — ⛔ escrever a conta aqui seria a segunda resposta à mesma pergunta.
///
/// ⚠️ **O `RootOrder` SAI**, e não é cosmética: ele é o desempate entre RAÍZES, e o adoptado deixou
/// de ser uma. Deixá-lo faria a árvore carregar uma ordem que descreve o que ele já não é — e há
/// gate a dizer que *«um filho não leva `RootOrder`»* desde que o osso existe.
///
/// ⛔ **Ele não pergunta pelo ciclo** — quem o faz é o [`splice_target`], para que a
/// pré-visualização e o release recusem o MESMO gesto. Ver a nota lá.
pub(crate) fn connect(sim: &mut SimWorld, child: u64, parent: u64) -> bool {
    let (Some(c), Some(p)) = (Entity::try_from_bits(child), Entity::try_from_bits(parent)) else {
        return false;
    };
    if !ph2d_vec_entities::transform::reparent_keeping_world(sim, c, p) {
        return false;
    }
    if let Ok(mut e) = sim.world_mut().get_entity_mut(c) {
        e.remove::<RootOrder>();
    }
    true
}
