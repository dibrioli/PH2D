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
use ph2d_vec_scene::{VecPathId, VecScene};

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

/// **As duas alças de curvatura e as duas extremidades de um osso**, em MUNDO.
///
/// ⚠️⚠️ **Existe um alias GÉMEO na [`ph2d_skeleton_render`], e as duas crates NÃO se conhecem —
/// de propósito.** O desenho do osso nunca importou a lei (ele é `ph2d-vector` + tokens, e é isso
/// que o deixa servir qualquer mídia), e criar a aresta só para partilhar um nome trocaria uma
/// duplicação de **quatro palavras** por uma dependência entre duas famílias. *A forma é a mesma
/// estruturalmente, logo o valor atravessa sem conversão nenhuma.*
pub type BendHandles = ([[f64; 2]; 2], [[f64; 2]; 2]);

/// ⭐⭐⭐⭐ **O OSSO COMO ELE DE FACTO DOBRA — a porta ÚNICA que resolve o `Handles::Auto`.**
///
/// ⛔⛔ **Ninguém nesta árvore lê `Bone::spec()` directamente a partir daqui**, e a razão é a lição
/// que este módulo já pagou três vezes: *um controlo DESENHADO por um mapa e DEFORMADO por outro é
/// um controlo morto sob o dedo*. O corpo que se pinta, a alça que se agarra e a pele que se
/// deforma têm de ler o MESMO osso — e com o `Auto` o osso efectivo já não é o que está guardado.
///
/// # ⭐⭐⭐ As tangentes saem da transformação RELATIVA, nunca do mundo
///
/// O vizinho anterior é o PAI e o seguinte é o FILHO, e a pose de um filho em relação ao pai **é**
/// o `Transform` dele — não é preciso ir ao mundo e voltar. ⚠️ **É isso que torna o ponto neutro
/// EXACTO:** numa corrente recta o pai fica em `(−L_pai, 0)` no local deste osso (o inverso de uma
/// translação pura é exacto) e a ponta do filho em `(L + L_filho, 0)`, logo as duas tangentes são
/// `(1, 0)` **ao bit** e o [`ph2d_skeleton::bend::auto_bend`] devolve `Bend::STRAIGHT`. *Uma volta
/// pelo mundo teria deixado `y ≈ 1e-17`, e ligar o Auto num rig recto arquearia tudo um bocadinho.*
///
/// ⚠️ **Um osso com DOIS filhos-osso não tem «o seguinte»** — ali a corrente ramifica, e escolher um
/// deles seria um sorteio. Nesse caso (e na ponta de uma corrente) a tangente é a do próprio eixo,
/// que deixa aquele lado recto.
#[must_use]
pub fn effective_spec(sim: &SimWorld, e: Entity) -> Option<ph2d_skeleton::bend::BoneSpec> {
    let osso = sim.world().get::<Bone>(e)?;
    let spec = osso.spec();
    if osso.handles != ph2d_skeleton::bend::Handles::Auto {
        return Some(spec);
    }
    let eixo = [spec.length, 0.0];
    // A tangente da RAIZ: da raiz do osso ANTERIOR até à ponta deste. Sem anterior, o próprio eixo.
    let t_raiz = pai_osso(sim, e)
        .and_then(|_| local_de(sim, e).inverse())
        .map_or(eixo, |inv| {
            let pai_raiz = inv.apply([0.0, 0.0]);
            [eixo[0] - pai_raiz[0], eixo[1] - pai_raiz[1]]
        });
    // A tangente da PONTA: da raiz deste osso até à ponta do SEGUINTE. Sem seguinte, o próprio eixo.
    let t_ponta = filho_osso_unico(sim, e).map_or(eixo, |(f, comp)| {
        let x = local_de(sim, f);
        x.apply([comp, 0.0])
    });
    Some(ph2d_skeleton::bend::BoneSpec {
        curve: ph2d_skeleton::bend::auto_bend(versor(t_raiz), versor(t_ponta)),
        ..spec
    })
}

/// O afim LOCAL de uma entidade — a pose dela em relação ao pai.
fn local_de(sim: &SimWorld, e: Entity) -> Xform {
    ph2d_vec_entities::transform::xform_of_transform(
        sim.world()
            .get::<ph2d_ecs::Transform>(e)
            .copied()
            .unwrap_or_default(),
    )
}

/// O versor de `v`; `(1, 0)` quando ele é degenerado.
///
/// ⚠️ **Sobre um vector já no eixo (`y == 0`, `x > 0`) ele devolve `(1, 0)` AO BIT** — `hypot(x, 0)`
/// é `|x|` exacto e `x / x` é `1.0` exacto. É essa propriedade que faz o ponto neutro do `Auto`
/// não ser uma tolerância.
fn versor(v: [f64; 2]) -> [f64; 2] {
    let n = v[0].hypot(v[1]);
    if n > 0.0 {
        [v[0] / n, v[1] / n]
    } else {
        [1.0, 0.0]
    }
}

/// O pai deste osso, se o pai for um osso.
fn pai_osso(sim: &SimWorld, e: Entity) -> Option<Entity> {
    let p = sim.world().get::<ph2d_ecs::ChildOf>(e)?.parent();
    sim.world().get::<Bone>(p).map(|_| p)
}

/// O ÚNICO filho-osso deste osso, com o comprimento dele; `None` se há zero ou mais de um.
///
/// ⛔ **Mais de um é `None` de propósito:** ali a corrente ramifica, e escolher um filho seria um
/// sorteio que muda com a ordem de varredura.
fn filho_osso_unico(sim: &SimWorld, e: Entity) -> Option<(Entity, f64)> {
    let mut achado: Option<(Entity, f64)> = None;
    for er in sim.world().iter_entities() {
        let filho_de_e = er
            .get::<ph2d_ecs::ChildOf>()
            .is_some_and(|c| c.parent() == e);
        if !filho_de_e {
            continue;
        }
        if let Some(b) = er.get::<Bone>() {
            if achado.is_some() {
                return None;
            }
            achado = Some((er.id(), b.length));
        }
    }
    achado
}

/// **Os ossos de um esqueleto, em MUNDO** — `(bits, origem, ponta)`, para o overlay desenhar e para
/// o gesto apontar.
///
/// ⚠️ Devolve **todos** os ossos da cena: quem quer um esqueleto só filtra por
/// [`skeleton_of`]. Uma segunda varredura com outra regra divergiria desta na primeira ramificação.
pub fn bone_segments(sim: &SimWorld) -> Vec<(u64, [f64; 2], [f64; 2])> {
    // ⭐⭐ **DERIVADA da polilinha, e não uma segunda varredura.** O doc acima diz por escrito que
    // *«uma segunda varredura com outra regra divergiria desta na primeira ramificação»* — quando o
    // osso passou a poder dobrar, essa frase deixou de ser sobre um risco e passou a ser sobre um
    // facto: a raiz e a ponta de um osso curvo são o PRIMEIRO e o ÚLTIMO nó dele.
    //
    // ⚠️ **E é byte-idêntica ao que ela sempre devolveu**, por duas construções que se encontram: um
    // osso rígido tem polilinha de dois nós (`(0,0)` e `(L,0)`, as MESMAS expressões de antes), e num
    // osso curvo o último nó é `(L,0)` exacto porque em `t = 1` os termos da correcção são `0.0`.
    bone_polylines(sim)
        .into_iter()
        .map(|(bits, pts)| {
            let ultimo = *pts.last().expect("a polilinha tem ao menos dois nos");
            (bits, pts[0], ultimo)
        })
        .collect()
}

/// ⭐⭐⭐ **O CORPO de cada osso, em MUNDO** — `(bits, polilinha)`, com `N+1` nós.
///
/// ⚠️⚠️ **Ela e a [`bone_segments`] têm consumidores DIFERENTES e as duas estão certas.** Quem
/// pergunta *«onde nasce e onde acaba este osso»* — a cinemática, o losango da IK, o gesto que
/// arrasta a ponta — quer as duas extremidades, e um osso curvo continua a ter exactamente duas.
/// Quem **DESENHA** o osso e quem o **AGARRA** querem o corpo, e o corpo de um *bendy bone* é esta
/// linha. ⛔ *Desenhar por um mapa e agarrar por outro é um controlo morto sob o dedo* — os dois
/// leem daqui, e há gate a atá-los.
///
/// ⚠️ Um osso **rígido** devolve DOIS nós, ao bit o que sempre devolveu (ver
/// [`ph2d_skeleton::bend::polyline`]).
#[must_use]
pub fn bone_polylines(sim: &SimWorld) -> Vec<(u64, Vec<[f64; 2]>)> {
    let mut out = Vec::new();
    for (e, spec) in ossos_da_cena(sim) {
        let x = world_of(sim, e);
        out.push((
            e.to_bits(),
            ph2d_skeleton::bend::polyline(spec)
                .into_iter()
                .map(|p| x.apply(p))
                .collect(),
        ));
    }
    out.sort_by_key(|(bits, _)| *bits);
    out
}

/// Todo osso da cena, com o que o artista autorou nele.
///
/// ⚠️ **Varre entidades em vez de montar uma `QueryState`**, e a razão é a assinatura: uma query
/// pede `&mut World` para nascer, e os dois consumidores disto — o overlay e o gesto — só têm o
/// mundo emprestado. *Uma função que pede `&mut` só para ler obriga o chamador a arranjar um `&mut`
/// que ele não precisa, e é assim que um `clone` do mundo aparece num caminho de quadro.*
fn ossos_da_cena(sim: &SimWorld) -> Vec<(Entity, ph2d_skeleton::bend::BoneSpec)> {
    sim.world()
        .iter_entities()
        // ⚠️ **A porta que resolve o `Auto`** ([`effective_spec`]), nunca o `Bone::spec()` cru: o
        // corpo que se desenha e a pele que se deforma têm de ler o MESMO osso.
        .filter_map(|er| effective_spec(sim, er.id()).map(|spec| (er.id(), spec)))
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
    for (j, b) in skin.tendons.iter().enumerate() {
        // Um osso apagado — ou um cuja identidade não está no índice — é SALTADO, e os outros
        // renormalizam-se sozinhos: apagar um osso não pode apagar a forma.
        let Some(&e) = index.get(&b.bone) else {
            continue;
        };
        let Some(vb) = sim.world().get::<Bone>(e).copied() else {
            continue;
        };
        // ⭐⭐ **UM osso autorado pode dar N sub-ossos** — é aqui, e só aqui, que um *bendy bone* se
        // torna a lista de poses rígidas que a [`Skin`] já sabia misturar. ⚠️ Com o osso recto (o
        // nascimento) ela empurra exactamente o que a `SkinBone::new` empurrava, **ao bit**, então
        // todo rig já autorado atravessa esta linha sem mudar um bit.
        //
        // ⭐⭐⭐ **E o índice do TENDÃO viaja com cada sub-osso** ([`SkinBone::tendon`]): é ele que
        // faz uma tabela de pesos guardada no bind — que é por osso AUTORADO — reencontrar as poses
        // certas depois de a resolução saltar os ossos apagados. ⛔ Usar a posição na pele em vez
        // do índice do tendão faria a arte saltar no instante em que alguém apagasse um osso, e
        // nenhum gate de geometria veria.
        #[expect(
            clippy::cast_possible_truncation,
            reason = "o número de tendões de uma pele é o número de ossos do esqueleto"
        )]
        SkinBone::bent(
            Xform(b.rest),
            // ⚠️ **A porta que resolve o `Auto`** — ver [`effective_spec`].
            effective_spec(sim, e).unwrap_or_else(|| vb.spec()),
            poses(e),
            shape_inv,
            j as u32,
            &mut ossos,
        );
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
        let Ok(guardado) = postcard::from_bytes::<crate::skinned_mesh::SkinnedPath>(&skin.source)
        else {
            continue;
        };
        // ⛔ Uma tabela que não fecha com o caminho cai na lei derivada em vez de ser lida
        // deslocada — pesos plausíveis sobre os pontos errados dão arte errada sem um erro.
        let pesos: &[f64] = if guardado.valida() {
            &guardado.pesos
        } else {
            &[]
        };
        let mut src = guardado.path.clone();
        ph2d_vec_skin::aplica_com(&pele, &mut src, pesos);
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
        let Some(shape_inv) = world_of(sim, shape).inverse() else {
            continue;
        };
        let pares = tendons_and_axes(sim, &ossos, shape_inv);
        // ⭐⭐⭐ **OS PESOS DO PADRÃO-OURO TAMBÉM PARA O VECTOR** (2026-09-15, 2.ª metade). Os eixos
        // já vêm no espaço da forma — que é o espaço em que os vértices dela vivem.
        //
        // ⛔ **Vazio é uma resposta:** um caminho ABERTO não tem interior, logo não tem domínio para
        // a energia, e ele fica na lei derivada. *Inventar um domínio para uma linha seria inventar
        // uma arte que o artista não desenhou.*
        let eixos: Vec<ph2d_skin_weights::Handle> = pares
            .iter()
            .map(|o| ph2d_skin_weights::Handle { a: o.a, b: o.b })
            .collect();
        let pesos = ph2d_vec_skin::pesos::pesos_do_caminho(src, &eixos).unwrap_or_default();
        let guardado = crate::skinned_mesh::SkinnedPath {
            path: src.clone(),
            pesos,
        };
        let Ok(bytes) = postcard::to_allocvec(&guardado) else {
            continue;
        };
        let tendoes = pares.into_iter().map(|o| o.tendon).collect();
        sim.world_mut()
            .entity_mut(shape)
            .insert(SkinBind::new(bytes, tendoes));
        feitos += 1;
    }
    feitos
}

/// ⭐⭐ **UM OSSO PRESO: o tendão que se guarda MAIS o eixo dele**, no espaço da coisa deformada.
///
/// ⚠️ **Os dois viajam juntos porque a coluna `j` da tabela de pesos é o tendão `j`** — separá-los
/// em duas listas é a forma como um osso sem `StableId` numa delas passa a descrever o osso
/// seguinte na outra, com a soma dos pesos a `1` e nenhum gate de geometria a acusar.
// ⚠️ Sem `Copy`: o [`Tendon`] carrega o `rest` e não o é.
#[derive(Clone, Debug, PartialEq)]
pub struct OssoPreso {
    /// O que a [`SkinBind`] guarda.
    pub tendon: Tendon,
    /// A raiz do eixo, no espaço da coisa deformada.
    pub a: [f64; 2],
    /// A ponta do eixo, no mesmo espaço.
    pub b: [f64; 2],
}

/// ⭐⭐⭐ **OS TENDÕES E O EIXO DE CADA UM, no espaço da coisa** — a porta que o padrão-ouro precisa.
///
/// ⚠️⚠️ **Ela existe para os dois NÃO poderem desalinhar-se.** Os pesos guardados são indexados
/// pela posição na lista de tendões, e o solver precisa do **eixo** de cada osso para saber que
/// pedaço da arte é de quem. Se as duas listas fossem produzidas por duas varreduras, bastaria um
/// osso sem `StableId` numa delas para a coluna `j` da tabela passar a descrever o osso `j+1` — e
/// a arte sairia deformada pelo osso errado, com a soma dos pesos a `1` e nenhum gate de geometria
/// a acusar. ⇒ **um percurso só, um `filter_map` só, dois valores por elemento.**
///
/// O eixo sai do próprio `rest` (`S⁻¹ ∘ B`), que é a única coisa que o tendão guarda — logo ele é,
/// por construção, o eixo que a pele vai usar no quadro.
///
/// ⚠️⚠️ **Ela é a ÚNICA porta do bind das DUAS mídias, e essa lei vem do `tendons_for` que ela
/// substituiu** (morto em 2026-09-15, quando os pesos passaram a precisar dos eixos): *a tentação
/// era copiar a lei para o bind novo — ela é curta e a cópia compilava. ⛔ Mas é exactamente a lei
/// cuja divergência ninguém veria: uma forma e uma imagem presas no mesmo gesto passariam a
/// responder a poses diferentes, e o sintoma seria «o braço desenhado não acompanha o braço
/// vectorial».*
///
/// ⚠️ **Um osso sem `StableId` é SALTADO**: `StableId::NONE` não nomeia ninguém, e guardá-lo daria
/// um tendão que resolve para nada — pior que um osso a menos, porque *parece* ligado.
#[must_use]
fn tendons_and_axes(sim: &SimWorld, ossos: &[Entity], shape_inv: Xform) -> Vec<OssoPreso> {
    ossos
        .iter()
        .filter_map(|&e| {
            let rest = world_of(sim, e).then(&shape_inv);
            let comprimento = sim.world().get::<Bone>(e)?.length;
            Some(OssoPreso {
                tendon: Tendon {
                    bone: ph2d_ecs::stable_id_of(sim.world(), e)?,
                    rest: rest.0,
                },
                a: rest.apply([0.0, 0.0]),
                b: rest.apply([comprimento, 0.0]),
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
    let Some(shape_inv) = world_of(sim, e).inverse() else {
        return false;
    };
    let pares = tendons_and_axes(sim, &ossos, shape_inv);
    // ⭐⭐⭐ **OS PESOS DO PADRÃO-OURO, RESOLVIDOS AQUI** — uma vez, sobre a arte, ao prender.
    //
    // ⚠️ **O espaço é o da MALHA (pixels da imagem)**, e é por isso que os eixos atravessam a
    // régua `local → pixel`: o solver mede distâncias sobre a própria arte, e no espaço da forma
    // uma sprite escalada daria um osso que prende mais ou menos vértices conforme o zoom do
    // artista.
    let pesos = crate::skin_image::weights_for_mesh(sim, e, &malha, &pares, pixels_per_meter);
    let guardada = crate::skinned_mesh::SkinnedMesh { mesh: malha, pesos };
    let Ok(bytes) = postcard::to_allocvec(&guardada) else {
        return false;
    };
    let tendoes = pares.into_iter().map(|o| o.tendon).collect();
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
            && let Ok(g) = postcard::from_bytes::<crate::skinned_mesh::SkinnedPath>(&skin.source)
            && let Some(p) = scene.path_mut(id)
        {
            p.replace_cooked(g.path);
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
/// ⭐⭐⭐ **AS DUAS ALÇAS DE CURVATURA de um osso, em MUNDO**, mais as duas extremidades dele — a
/// porta ÚNICA do desenho e do dedo ([`ph2d_skeleton_render::draw_bend`] e o `bone_pick`).
///
/// ⛔⛔ **`None` quando a curvatura é INERTE**, isto é, quando o osso produz um sub-osso só
/// (`segments <= 1`). Pintar a alça ali prometeria um verbo que o arrasto não executa — a espécie
/// de controlo morto que este módulo acabou de pagar no botão `Smooth`. *E a pergunta é feita pela
/// porta do produto ([`ph2d_skeleton::bend::segments_of`]), nunca por um `> 1` escrito aqui.*
///
/// ⚠️ **As extremidades saem da POLILINHA e não do `spec`**: num osso já arqueado a ponta desenhada
/// é o último nó da curva, e é dela que a haste da alça tem de sair para o desenho fechar.
#[must_use]
pub fn bend_handles(sim: &SimWorld, bits: u64) -> Option<BendHandles> {
    let e = Entity::try_from_bits(bits)?;
    let spec = effective_spec(sim, e)?;
    if ph2d_skeleton::bend::segments_of(spec.segments) <= 1 {
        return None;
    }
    // ⛔⛔ **EM `Auto` NÃO HÁ ALÇA PARA AGARRAR, e é isso que a torna honesta:** as duas são
    // DERIVADAS da corrente, então arrastar uma seria escrever num valor que o quadro seguinte
    // recalcula — *o artista veria o gizmo voltar debaixo do dedo*, que é o defeito que a âncora
    // de IK deste módulo já pagou. ⇒ ali elas não se pintam nem se pegam, e o painel esconde os
    // quatro números pela mesma razão.
    if sim.world().get::<Bone>(e)?.handles == ph2d_skeleton::bend::Handles::Auto {
        return None;
    }
    let x = world_of(sim, e);
    let [inn, out] = ph2d_skeleton::bend::handles(spec.length, spec.curve);
    let pts = ph2d_skeleton::bend::polyline(spec);
    let (a, b) = (
        *pts.first().expect("a polilinha tem ao menos dois nos"),
        *pts.last().expect("a polilinha tem ao menos dois nos"),
    );
    Some(([x.apply(inn), x.apply(out)], [x.apply(a), x.apply(b)]))
}

/// ⭐⭐⭐ **ESCREVE A CURVATURA a partir de um ponto de MUNDO** — o inverso do [`bend_handles`], e o
/// verbo do arrasto.
///
/// `ponta = false` move a alça da raiz, `true` a da ponta. Devolve `false` quando não há onde
/// escrever (não é osso, é rígido, ou a pose é singular).
///
/// ⚠️ **A conversão mundo → local passa pelo INVERSO da mesma pose que o desenho usa**, e não por
/// uma cadeia escrita à mão: um osso filho herda a pose do pai, e reconstruí-la aqui poria a alça
/// onde ela não está desenhada — o defeito que o `sprite_image_to_screen_affine` deste repo já
/// pagou por escrito.
pub fn set_bend_handle(sim: &mut SimWorld, bits: u64, world: [f64; 2], ponta: bool) -> bool {
    let Some(e) = Entity::try_from_bits(bits) else {
        return false;
    };
    let Some(osso) = sim.world().get::<Bone>(e).copied() else {
        return false;
    };
    let spec = osso.spec();
    // ⚠️ As MESMAS duas recusas do [`bend_handles`], e pela mesma razão — escrever onde não há alça
    // pintada seria o gesto a mandar num valor que ninguém vê.
    if ph2d_skeleton::bend::segments_of(spec.segments) <= 1
        || osso.handles == ph2d_skeleton::bend::Handles::Auto
    {
        return false;
    }
    let Some(inv) = world_of(sim, e).inverse() else {
        return false;
    };
    let Some(v) = ph2d_skeleton::bend::bend_from_handle(spec.length, inv.apply(world), ponta)
    else {
        return false;
    };
    let Some(mut osso) = sim.world_mut().get_mut::<Bone>(e) else {
        return false;
    };
    // ⛔ **Sem tecto, e é §0.0:** não há recurso nenhum a limitar quanto um osso arqueia — o que
    // limita é o olho do artista. O ponto NEUTRO, esse, é alcançável (largar a alça no terço).
    if ponta {
        osso.curve.out = v;
    } else {
        osso.curve.inn = v;
    }
    true
}

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
