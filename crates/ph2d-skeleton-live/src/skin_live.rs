//! **O ESQUELETO, vivo** (estudo 42 item 5, doc 47) — a forma presa aos ossos é re-cozida a cada
//! quadro a partir da fonte autorada e da pose de agora.
//!
//! Irmão do [`crate::envelope_live`] no padrão (fonte em bytes dentro do componente · `replace_cooked`
//! por quadro · undo e save de graça porque os dois capturam o mundo ECS) e diferente em duas coisas
//! que valem a pena ler antes de mexer:
//!
//! 1. **Não há container.** A gaiola do envelope não é entidade e precisa de um dono; um esqueleto
//!    **já são entidades**, então a forma presa fica onde o artista a pôs.
//! 2. **A cinemática não se escreve.** O mundo de cada osso sai de `parent_world_transform`, que é a
//!    propagação de `Transform` que a casa já corre — logo FK é de borla, e a timeline anima um osso
//!    porque anima um `Transform`.
//!
//! ⚠️ **O ponto onde isto se parte, se alguém o refactorar:** a matriz de um osso é
//! `S_agora⁻¹ ∘ B_agora ∘ rest⁻¹`, e o `rest` guardado **é** `S_bind⁻¹ ∘ B_bind`. Ligar num espaço e
//! cozer noutro devolve uma forma que salta para longe no instante do bind. A composição vive numa
//! porta só ([`ph2d_skeleton::SkinBone::new`]) por causa disso.

use ph2d_ecs::{ChildOf, Entity, SimWorld, StableId, VecPathRef};
use ph2d_skeleton::{Skin, SkinBone, Xform};
use ph2d_skeleton_ecs::{Bone, SkinBind, Tendon};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

use ph2d_vec_entities::entities::VecEntityMap;

/// O que sobra quando se solta uma forma do esqueleto — os dois verbos do envelope, pela mesma razão
/// (o artista pode querer **o que vê** ou **o que desenhou**, e adivinhar é que não).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keep {
    /// A geometria deformada de agora vira o desenho. É o *Expand*.
    Deformed,
    /// A fonte autorada volta. É o *Release*.
    Source,
}

/// O afim local→mundo de uma entidade.
fn world_of(sim: &SimWorld, e: Entity) -> Xform {
    ph2d_vec_entities::transform::xform_of_transform(ph2d_vec_entities::transform::world_transform(
        sim, e,
    ))
}

/// **Os ossos de um esqueleto, em MUNDO** — `(bits, origem, ponta)`, para o overlay desenhar e para
/// o gesto apontar.
///
/// ⚠️ Devolve **todos** os ossos da cena: quem quer um esqueleto só filtra por
/// [`skeleton_of`]. Uma segunda varredura com outra regra divergiria desta na primeira ramificação.
pub fn bone_segments(sim: &SimWorld) -> Vec<(u64, [f64; 2], [f64; 2])> {
    let mut out = Vec::new();
    for (e, length) in ossos_da_cena(sim) {
        let x = world_of(sim, e);
        out.push((e.to_bits(), x.apply([0.0, 0.0]), x.apply([length, 0.0])));
    }
    out.sort_by_key(|(bits, _, _)| *bits);
    out
}

/// Todo osso da cena, com o comprimento dele.
///
/// ⚠️ **Varre entidades em vez de montar uma `QueryState`**, e a razão é a assinatura: uma query
/// pede `&mut World` para nascer, e os dois consumidores disto — o overlay e o gesto — só têm o
/// mundo emprestado. *Uma função que pede `&mut` só para ler obriga o chamador a arranjar um `&mut`
/// que ele não precisa, e é assim que um `clone` do mundo aparece num caminho de quadro.*
fn ossos_da_cena(sim: &SimWorld) -> Vec<(Entity, f64)> {
    sim.world()
        .iter_entities()
        .filter_map(|er| er.get::<Bone>().map(|b| (er.id(), b.length)))
        .collect()
}

/// **O esqueleto de que este osso faz parte** — sobe até à raiz (o ancestral mais alto que ainda é
/// osso) e desce recolhendo tudo o que é osso.
///
/// `None` ⇒ **todos** os ossos da cena, que é a leitura certa de *"o artista não apontou nenhum"*
/// quando há um esqueleto só — o caso comum, e o que faz o botão Bind funcionar sem cerimónia.
///
/// ⚠️ **A ORDEM não tem sentido, e mesmo assim tem de ser DETERMINÍSTICA.** Os pesos normalizam-se,
/// então permutar os ossos devolve o mesmo desenho — mas a permutação muda a **ordem da soma** em
/// `f64`, logo o último ULP. Ordenar por `to_bits` (id de alocação) resolve isso *dentro de uma
/// sessão*, e depois do bind quem fixa a ordem é a lista **guardada** no componente. ⛔ Isto **não**
/// é o `canonicalize` que o `CLAUDE.md` §5 proíbe: ali os bits decidiam o CONTEÚDO de um snapshot;
/// aqui decidem só em que ordem se somam parcelas que já foram escolhidas.
pub fn skeleton_of(sim: &SimWorld, seed: Option<Entity>) -> Vec<Entity> {
    let todos: Vec<Entity> = ossos_da_cena(sim).into_iter().map(|(e, _)| e).collect();
    let Some(seed) = seed.filter(|e| todos.contains(e)) else {
        let mut t = todos;
        t.sort_by_key(|e| e.to_bits());
        return t;
    };
    // Sobe enquanto o PAI também for osso — parar no primeiro pai não-osso é o que permite pendurar
    // um esqueleto inteiro dentro de um grupo sem ele deixar de ser um esqueleto.
    let mut raiz = seed;
    while let Some(p) = sim.world().get::<ChildOf>(raiz).map(ChildOf::parent) {
        if sim.world().get::<Bone>(p).is_some() {
            raiz = p;
        } else {
            break;
        }
    }
    let mut out = Vec::new();
    let mut pilha = vec![raiz];
    while let Some(e) = pilha.pop() {
        if sim.world().get::<Bone>(e).is_none() {
            continue;
        }
        out.push(e);
        if let Some(f) = sim.world().get::<ph2d_ecs::Children>(e) {
            pilha.extend(f.iter());
        }
    }
    out.sort_by_key(|e| e.to_bits());
    out
}

/// O índice `StableId → entidade` dos ossos da cena — ver [`bone_index`].
pub type BoneIndex = std::collections::BTreeMap<StableId, Entity>;

/// ⭐⭐ **O ÍNDICE `StableId → entidade` dos ossos** — a porta única da resolução de um tendão.
///
/// ⚠️ **Construído uma vez por quadro e passado adiante**, e é o que o doc do
/// [`ph2d_ecs::entity_of_stable_id`] manda fazer: ele é uma varredura linear de propósito (um mapa
/// permanente seria estado derivado a manter coerente com o mundo), e quem resolve muitos ids num
/// quadro constrói o índice.
///
/// ⚠️ **Só ossos**, e é o filtro que faz a resposta ser a certa: um `StableId` de um osso apagado
/// simplesmente não está aqui, e o tendão dele é saltado.
#[must_use]
pub fn bone_index(sim: &SimWorld) -> BoneIndex {
    sim.world()
        .iter_entities()
        .filter(|er| er.get::<Bone>().is_some())
        .filter_map(|er| er.get::<StableId>().map(|s| (*s, er.id())))
        .collect()
}

/// A pele de uma forma, resolvida para ESTE quadro. `None` quando não há osso vivo nenhum (todos
/// apagados) ou quando a pose da forma é singular — nos dois casos a forma fica em paz.
fn resolve(
    sim: &SimWorld,
    skin: &SkinBind,
    shape: Entity,
    index: &std::collections::BTreeMap<StableId, Entity>,
) -> Option<Skin> {
    resolve_with(sim, skin, shape, index, &|e| world_of(sim, e))
}

/// ⭐⭐⭐ **[`resolve`] com a FONTE DAS POSES DE MUNDO injectada** — a mesma lei, noutro instante.
///
/// ⚠️ **Uma pele é função de POSES, e «agora» é só uma das respostas possíveis.** Os fantasmas do
/// onion precisam da pele em `t ± k` (as poses dos OSSOS naquele instante, compostas pela cadeia), e
/// copiar esta função para lá faria uma forma e a imagem irmã responderem a leis diferentes — o
/// defeito que o [`tendons_for`] já nomeia por escrito, um nível abaixo.
///
/// ⛔ **A crate não conhece a timeline, e não é para conhecer:** quem sabe o que é um instante
/// constrói o fecho e passa-o. Aqui só existe *«onde está esta entidade»*.
///
/// ⚠️ **A fonte responde pela FORMA e pelos OSSOS.** Posar só os ossos deixaria a imagem no sítio de
/// agora com o esqueleto no de `t` — a arte dobrava e escorregava ao mesmo tempo.
fn resolve_with(
    sim: &SimWorld,
    skin: &SkinBind,
    shape: Entity,
    index: &BoneIndex,
    poses: &impl Fn(Entity) -> Xform,
) -> Option<Skin> {
    let shape_inv = poses(shape).inverse()?;
    let mut ossos = Vec::with_capacity(skin.tendons.len());
    for b in &skin.tendons {
        // Um osso apagado — ou um cuja identidade não está no índice — é SALTADO, e os outros
        // renormalizam-se sozinhos: apagar um osso não pode apagar a forma.
        let Some(&e) = index.get(&b.bone) else {
            continue;
        };
        let Some(vb) = sim.world().get::<Bone>(e).copied() else {
            continue;
        };
        if let Some(sb) = SkinBone::new(Xform(b.rest), vb.length, vb.strength, poses(e), shape_inv)
        {
            ossos.push(sb);
        }
    }
    Skin::new(ossos)
}

/// ⭐⭐⭐ **A PELE DE UMA COISA, resolvida agora** — a porta que serve quem não é um `VecPath`.
///
/// ⚠️ **Ela existe porque a 2.ª mídia chegou** (uma imagem que obedece ao esqueleto): o
/// [`resolve`] já respondia a pergunta inteira e estava fechado atrás do laço do [`recook`], que
/// só sabe de formas vectoriais. *Uma lei alcançável só de dentro de um laço é uma lei com um
/// cliente por construção.*
///
/// ⛔ **Ela constrói o índice de ossos a cada chamada**, e é de propósito: quem chama tem UMA
/// coisa na mão (o laço do [`recook`] partilha o índice entre N formas, e continua a partilhá-lo).
/// Com o índice a ser um `BTreeMap` sobre os ossos da cena, o preço é o número de ossos — não o do
/// mundo.
#[must_use]
pub fn skin_of(sim: &SimWorld, e: Entity) -> Option<Skin> {
    let skin = sim.world().get::<SkinBind>(e)?;
    resolve(sim, skin, e, &bone_index(sim))
}

/// ⭐⭐⭐ **A pele de uma coisa NUM INSTANTE** — o [`skin_of`] com as poses de mundo injectadas.
///
/// `poses(e)` devolve **onde `e` está** no instante que interessa; o consumidor de hoje é o onion,
/// que compõe `ph2d_timeline::world_pose_at` pela cadeia. Ver [`resolve_with`] para o porquê de a
/// lei ser UMA.
/// ⚠️ **Quem corre em LAÇO usa o [`skin_of_in`]** e partilha o índice — é a lei que o doc do
/// [`skin_of`] já escreve: *«ela constrói o índice a cada chamada, e é de propósito: quem chama tem
/// UMA coisa na mão»*. O onion tem `N artes × M instantes` na mão.
#[must_use]
pub fn skin_of_with(sim: &SimWorld, e: Entity, poses: &impl Fn(Entity) -> Xform) -> Option<Skin> {
    skin_of_in(sim, e, &bone_index(sim), poses)
}

/// [`skin_of_with`] com o índice de ossos emprestado — para quem resolve MUITAS peles num quadro.
#[must_use]
pub fn skin_of_in(
    sim: &SimWorld,
    e: Entity,
    index: &BoneIndex,
    poses: &impl Fn(Entity) -> Xform,
) -> Option<Skin> {
    let skin = sim.world().get::<SkinBind>(e)?;
    resolve_with(sim, skin, e, index, poses)
}

/// **Um quadro de pele.** Corre depois do `vec_entities::sync` (as entidades existem) e ao lado do
/// `envelope_live::recook`.
pub fn recook(sim: &SimWorld, scene: &mut VecScene) {
    let alvos: Vec<(Entity, SkinBind, VecPathId)> = sim
        .world()
        .iter_entities()
        .filter_map(|er| {
            Some((
                er.id(),
                er.get::<SkinBind>()?.clone(),
                er.get::<VecPathRef>()?.0,
            ))
        })
        .collect();
    // ⚠️ **O diagnóstico da família** (`PH2D_BONE_LOG=1`), irmão do `PH2D_MORPH_LOG`: ele responde
    // as três perguntas que um report de *"não deforma"* não distingue — *há pele? há osso vivo? a
    // matriz é a identidade?* Sem ele, as três produzem o MESMO sintoma na tela.
    let log = std::env::var_os("PH2D_BONE_LOG").is_some();
    if log {
        eprintln!(
            "[bone] peles={} ossos={}",
            alvos.len(),
            ossos_da_cena(sim).len()
        );
    }
    let index = bone_index(sim);
    for (e, skin, id) in alvos {
        let Some(pele) = resolve(sim, &skin, e, &index) else {
            if log {
                eprintln!(
                    "[bone] pele de {id} NAO resolveu (ossos={})",
                    skin.tendons.len()
                );
            }
            continue;
        };
        if log {
            let m: Vec<[f64; 6]> = pele.bones().iter().map(|b| b.pose.0).collect();
            eprintln!("[bone] {id}: {} osso(s), poses={m:?}", pele.len());
        }
        // Uma fonte corrompida é PULADA (não há o que deformar, e melhor não escrever lixo) — a
        // forma fica com a última geometria boa. Mesma escolha do envelope.
        let Ok(mut src) = postcard::from_bytes::<VecPath>(&skin.source) else {
            continue;
        };
        ph2d_vec_skin::apply(&pele, &mut src);
        if let Some(p) = scene.path_mut(id) {
            p.replace_cooked(src);
        }
    }
}

/// **Prende as formas ao esqueleto.** Devolve quantas prendeu.
///
/// A fonte é a geometria que a forma tem **agora** — o que faz um segundo Bind ser um *re-bind na
/// pose actual*, que é o gesto que todo o pacote de rig oferece. E como a pose de repouso é a
/// identidade por construção (§2.5 do doc 47), **prender não move um pixel**.
pub fn bind(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    paths: &[VecPathId],
    seed: Option<Entity>,
) -> usize {
    let ossos = skeleton_of(sim, seed);
    if ossos.is_empty() {
        return 0;
    }
    // ⚠️ **Um osso criado NESTE quadro ainda não tem `StableId`** — a varredura corre uma vez por
    // quadro, e o gesto de prender pode vir antes dela. Semear aqui é o que o
    // `inspector_joint_create` já faz pela mesma razão, e sem isto o tendão nomearia `NONE`.
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let mut feitos = 0;
    for &id in paths {
        let Some(&bits) = map.get(&id) else { continue };
        let shape = Entity::from_bits(bits);
        if sim.world().get_entity(shape).is_err() {
            continue;
        }
        let Some(src) = scene.paths().iter().find(|p| p.id == id) else {
            continue;
        };
        let Ok(bytes) = postcard::to_allocvec(src) else {
            continue;
        };
        let Some(shape_inv) = world_of(sim, shape).inverse() else {
            continue;
        };
        let tendoes = tendons_for(sim, &ossos, shape_inv);
        sim.world_mut()
            .entity_mut(shape)
            .insert(SkinBind::new(bytes, tendoes));
        feitos += 1;
    }
    feitos
}

/// ⭐⭐⭐ **OS TENDÕES DE UMA COISA** — a lei do bind, escrita uma vez para as DUAS mídias.
///
/// `rest = S⁻¹ ∘ B` — aplica o mundo do osso primeiro, depois leva ao espaço da coisa.
///
/// ⚠️ **Um osso sem `StableId` é SALTADO**: `StableId::NONE` não nomeia ninguém, e guardá-lo daria
/// um tendão que resolve para nada — pior que um osso a menos, porque *parece* ligado.
///
/// ⚠️ **Ela saiu do laço do [`bind`] quando a 2.ª mídia chegou** (uma imagem que obedece ao
/// esqueleto). A tentação era copiá-la para o bind novo: a lei é curta e a cópia compilava. ⛔ Mas
/// é exactamente a lei cuja divergência ninguém veria — uma forma e uma imagem presas no mesmo
/// gesto passariam a responder a poses diferentes, e o sintoma seria *«o braço desenhado não
/// acompanha o braço vectorial»*.
#[must_use]
fn tendons_for(sim: &SimWorld, ossos: &[Entity], shape_inv: Xform) -> Vec<Tendon> {
    ossos
        .iter()
        .filter_map(|&e| {
            Some(Tendon {
                bone: ph2d_ecs::stable_id_of(sim.world(), e)?,
                rest: world_of(sim, e).then(&shape_inv).0,
            })
        })
        .collect()
}

/// ⭐⭐⭐ **PRENDE UMA IMAGEM ao esqueleto** — a 2.ª mídia (ordem do dono, 2026-09-09).
///
/// A malha é traçada da própria tinta ([`crate::skin_image::mesh_from_rgba`]) e guardada
/// nos **bytes opacos** da [`SkinBind`] — ⭐ sem uma variante nova e sem tocar no schema, que é o
/// que o doc daquele campo prometia por escrito desde que ele existe.
///
/// ⚠️ **Os tendões saem da MESMA porta que os de uma forma** ([`tendons_for`]): as duas mídias
/// respondem à mesma pose ou o personagem parte-se ao meio.
///
/// ⚠️ **`pixels_per_meter` é o do PROJECTO** — o mesmo que o extract passa ao
/// `Sprite::resolve_anchor`. A régua da imagem lê a âncora resolvida ao prender e ao desenhar
/// ([`crate::skin_image::pixel_to_local`]); dois valores dariam uma âncora a cada gesto.
///
/// `false` quando não há esqueleto, quando a pose da imagem é singular, ou quando a tinta não dá
/// uma malha — e nos três casos **nada é escrito**, porque uma pele sem malha lá dentro não é uma
/// pele, é uma imagem prestes a sumir.
pub fn bind_image(
    sim: &mut SimWorld,
    e: Entity,
    rgba: &[u8],
    size_px: [u32; 2],
    pixels_per_meter: f32,
    opts: ph2d_poly2d::GridOptions,
    seed: Option<Entity>,
) -> bool {
    let ossos = skeleton_of(sim, seed);
    if ossos.is_empty() || sim.world().get_entity(e).is_err() {
        return false;
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // ⭐⭐⭐ **AS ARTICULAÇÕES GRADUAM A MALHA** (report do dono, 2026-09-10). Elas saem daqui e não
    // do leaf da geometria: só quem PRENDE sabe onde a dobra vai acontecer.
    let focos = crate::skin_image::joints_in_image(sim, e, &ossos, size_px, pixels_per_meter);
    let Some(malha) = crate::skin_image::mesh_from_rgba(rgba, size_px[0], size_px[1], &focos, opts)
    else {
        return false;
    };
    let Ok(bytes) = postcard::to_allocvec(&malha) else {
        return false;
    };
    let Some(shape_inv) = world_of(sim, e).inverse() else {
        return false;
    };
    let tendoes = tendons_for(sim, &ossos, shape_inv);
    sim.world_mut()
        .entity_mut(e)
        .insert(SkinBind::new(bytes, tendoes));
    true
}

/// **Solta as formas seleccionadas do esqueleto.** Devolve quantas soltou.
pub fn release(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    paths: &[VecPathId],
    keep: Keep,
) -> usize {
    let mut feitos = 0;
    for &id in paths {
        let Some(&bits) = map.get(&id) else { continue };
        let e = Entity::from_bits(bits);
        let Some(skin) = sim.world().get::<SkinBind>(e).cloned() else {
            continue;
        };
        if keep == Keep::Source
            && let Ok(src) = postcard::from_bytes::<VecPath>(&skin.source)
            && let Some(p) = scene.path_mut(id)
        {
            p.replace_cooked(src);
        }
        sim.world_mut().entity_mut(e).remove::<SkinBind>();
        feitos += 1;
    }
    feitos
}

/// ⭐⭐⭐ **AS PONTAS DE CORRENTE** — os ossos que não têm osso filho, em bits.
///
/// São eles, e só eles, que ganham a alça do *end effector*: em toda outra junta a ponta de um osso
/// **é** a raiz do seguinte, e ali já há uma bolinha com outro verbo.
pub fn chain_ends(sim: &SimWorld) -> Vec<u64> {
    let ossos: Vec<Entity> = ossos_da_cena(sim).into_iter().map(|(e, _)| e).collect();
    let mut out: Vec<u64> = ossos
        .iter()
        .filter(|&&e| {
            sim.world()
                .get::<ph2d_ecs::Children>(e)
                .is_none_or(|f| f.iter().all(|c| sim.world().get::<Bone>(*c).is_none()))
        })
        .map(|e| e.to_bits())
        .collect();
    out.sort_unstable();
    out
}

/// ⭐⭐⭐ **AS IMAGENS PRESAS A ESTE ESQUELETO** — a arte que se move quando este osso se move.
///
/// ⚠️ **Ela existe porque um OSSO não tem silhueta.** O onion mostra o passado e o futuro do que o
/// animador tem na mão, e o que ele tem na mão quando posa é um osso — a coisa que se vê mover é a
/// ARTE. Sem esta porta o onion de um personagem riggado não mostrava nada: a imagem não está
/// animada (quem tem keys são os ossos) e o osso não tem instância de desenho.
///
/// ⚠️ **O critério é o TENDÃO, e não a hierarquia:** prender é uma decisão autorada, e uma imagem
/// pode estar pendurada em qualquer sítio da cena. Um tendão cujo osso não está no índice (apagado)
/// simplesmente não conta, como em todo o resto deste módulo.
///
/// ⛔ **Só IMAGENS.** Uma forma vectorial presa ao mesmo esqueleto é desenhada pelo Vello e não tem
/// `RenderInstance` — o passe que desenha fantasmas é o de sprites, logo ela não pode ser ghostada
/// por aqui. Limite NOMEADO, não esquecimento.
#[must_use]
pub fn skinned_images_of_skeleton(sim: &SimWorld, seed: Entity, index: &BoneIndex) -> Vec<Entity> {
    let ossos: std::collections::BTreeSet<Entity> =
        skeleton_of(sim, Some(seed)).into_iter().collect();
    if ossos.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<Entity> = sim
        .world()
        .iter_entities()
        .filter(|er| crate::skin_image::is_skinned_image(sim.world(), er.id()))
        .filter(|er| {
            er.get::<SkinBind>().is_some_and(|s| {
                s.tendons
                    .iter()
                    .any(|t| index.get(&t.bone).is_some_and(|b| ossos.contains(b)))
            })
        })
        .map(|er| er.id())
        .collect();
    // A ordem de `iter_entities` é a dos arquétipos; ordenar deixa a lista determinística entre
    // quadros, que é o que um consumidor de desenho precisa.
    out.sort_by_key(|e| e.to_bits());
    out
}

/// **A CORRENTE que acaba neste osso** — da raiz até ele, na ordem em que a cinemática a resolve.
///
/// ⚠️ Ela sobe enquanto o PAI também for osso, que é a mesma regra do [`skeleton_of`] — parar no
/// primeiro pai não-osso é o que permite pendurar um esqueleto dentro de um grupo sem ele deixar de
/// ser um esqueleto.
pub fn chain_to(sim: &SimWorld, bits: u64) -> Vec<Entity> {
    let mut fila = vec![Entity::from_bits(bits)];
    while let Some(&e) = fila.last() {
        let Some(p) = sim.world().get::<ChildOf>(e).map(ChildOf::parent) else {
            break;
        };
        if sim.world().get::<Bone>(p).is_none() {
            break;
        }
        fila.push(p);
    }
    fila.reverse();
    fila
}

/// ⭐⭐ **O RAIO DE INFLUÊNCIA de um osso, em MUNDO** — a porta única do desenho, do dedo e do
/// arrasto.
///
/// A lei é a da pele (`SkinBone::new`): o raio é `força × comprimento do eixo`. ⚠️ **Aqui o eixo é
/// o de MUNDO** e lá é o do espaço da forma — e é isso que faz este número ser o que o artista vê:
/// a mancha cobre, na tela, exactamente o que o osso alcança no desenho que está por baixo dela.
///
/// `None` se o osso não existe. Zero é legal e significa *"não alcança ninguém pelo raio"* — o osso
/// só ganha um ponto pelo desempate do órfão.
pub fn influence_radius(sim: &SimWorld, bits: u64) -> Option<f64> {
    let e = Entity::from_bits(bits);
    let forca = sim.world().get::<Bone>(e)?.strength;
    let (_, a, b) = bone_segments(sim)
        .into_iter()
        .find(|(x, _, _)| *x == bits)?;
    Some((b[0] - a[0]).hypot(b[1] - a[1]) * forca.max(0.0))
}

/// **A região de influência de um osso** — `(raio, origem, ponta)` em MUNDO, para o overlay.
pub fn influence_region(sim: &SimWorld, bits: u64) -> Option<(f64, [f64; 2], [f64; 2])> {
    let r = influence_radius(sim, bits)?;
    let (_, a, b) = bone_segments(sim)
        .into_iter()
        .find(|(x, _, _)| *x == bits)?;
    Some((r, a, b))
}

#[cfg(test)]
#[path = "skin_live_tests.rs"]
mod tests;
