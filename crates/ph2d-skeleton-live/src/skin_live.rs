//! **O ESQUELETO, vivo** (estudo 42 item 5, doc 47) — a forma presa aos ossos é re-cozida a cada
//! quadro a partir da fonte autorada e da pose de agora.
//!
//! Irmão do [`crate::envelope_live`] no padrão (fonte em bytes dentro do componente · re-escrita
//! em lugar por quadro · undo e save de graça porque os dois capturam o mundo ECS) e diferente em
//! TRÊS coisas que valem a pena ler antes de mexer:
//!
//! 1. **Não há container.** A gaiola do envelope não é entidade e precisa de um dono; um esqueleto
//!    **já são entidades**, então a forma presa fica onde o artista a pôs.
//! 2. **A cinemática não se escreve.** O mundo de cada osso sai de `parent_world_transform`, que é a
//!    propagação de `Transform` que a casa já corre — logo FK é de borla, e a timeline anima um osso
//!    porque anima um `Transform`.
//! 3. ⛔⛔ **A fonte é uma FOTOGRAFIA, não os parâmetros vivos** — e é por isso que o quadro escreve
//!    por [`ph2d_vec_scene::VecPath::replace_geometry`] e não pela irmã `replace_cooked`, que o
//!    envelope e o texto usam. O envelope deforma o path que o artista está a editar (o estilo vem
//!    de lá, vivo); a pele guardou os bytes no instante do `Bind`, logo o estilo dela está
//!    **congelado** — mandá-lo para o path vivo desfaz toda edição de traço no quadro seguinte
//!    (report do dono, 2026-09-19).
//!
//! ⚠️ **O ponto onde isto se parte, se alguém o refactorar:** a matriz de um osso é
//! `S_agora⁻¹ ∘ B_agora ∘ rest⁻¹`, e o `rest` guardado **é** `S_bind⁻¹ ∘ B_bind`. Ligar num espaço e
//! cozer noutro devolve uma forma que salta para longe no instante do bind. A composição vive numa
//! porta só ([`ph2d_skeleton::SkinBone::new`]) por causa disso.

use ph2d_ecs::{Entity, SimWorld, StableId, VecPathRef};
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

/// ⭐⭐ **A CURVATURA de um osso mudou de ficheiro, NÃO de endereço** (tecto de LOC, 2026-09-16).
///
/// ⚠️ **O `pub use` é deliberado:** os consumidores escrevem `skin_live::bend_handles` e
/// `skin_live::effective_spec`, e reescrevê-los todos trocaria um corte por um mapa de excepções
/// (HOWTO §1.2/§1.4). *O ficheiro é uma unidade de manutenção; o caminho é um contrato.*
pub use crate::bend_live::{BendHandles, bend_handles, effective_spec, set_bend_handle};

/// ⭐⭐ **A MANCHA DE INFLUÊNCIA mudou de FICHEIRO, não de endereço** (tecto de LOC, 2026-09-18) —
/// a mesma lei que o `bend_live` acima já aplica: *o ficheiro é uma unidade de manutenção; o
/// caminho é um contrato*. ⚠️ Ela foi para o [`crate::esqueletos`] porque é o assunto dele — *até
/// onde um osso alcança* — e porque a lei que a governa já lá vive.
pub use crate::esqueletos::{influence_radius, influence_region};

/// O afim local→mundo de uma entidade.
pub(crate) fn world_of(sim: &SimWorld, e: Entity) -> Xform {
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
pub(crate) fn ossos_da_cena(sim: &SimWorld) -> Vec<(Entity, ph2d_skeleton::bend::BoneSpec)> {
    sim.world()
        .iter_entities()
        // ⚠️ **A porta que resolve o `Auto`** ([`effective_spec`]), nunca o `Bone::spec()` cru: o
        // corpo que se desenha e a pele que se deforma têm de ler o MESMO osso.
        .filter_map(|er| crate::bend_live::effective_spec(sim, er.id()).map(|spec| (er.id(), spec)))
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
    // ⭐ A subida é a PORTA — ela tem um segundo leitor (o `recusa_do_bind`), e duas cópias
    // divergiriam no dia do primeiro ajuste.
    let raiz = crate::esqueletos::raiz_do_osso(sim, seed);
    // ⭐ **E a DESCIDA também é a porta** desde 2026-09-19: ela ganhou um segundo leitor (a pose de
    // repouso, que desce a partir do osso ESCOLHIDO e não da raiz), e duas cópias divergiriam no
    // dia do primeiro ajuste — foi assim que a subida virou porta uma wave antes.
    crate::esqueletos::ossos_desde(sim, raiz)
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
pub(crate) fn resolve(
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
            crate::bend_live::effective_spec(sim, e).unwrap_or_else(|| vb.spec()),
            poses(e),
            shape_inv,
            j as u32,
            &mut ossos,
        );
    }
    Skin::com_mistura(ossos, crate::mistura_do_angulo::mistura_do_ambiente())
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
///
/// ⭐⭐⭐ **Devolve o DESENHO FIEL de cada forma presa** ([`crate::skin_desenho`]) — o caminho da cena
/// continua a receber a lei nos nós do artista; o que se vê entra pela geometria viva do quadro.
pub fn recook(sim: &SimWorld, scene: &mut VecScene) {
    let _ = recook_desenhando(sim, scene);
}

/// ⭐⭐⭐ **O [`recook`] que DEVOLVE o desenho fiel** — a porta do quadro do produto, que o entrega
/// à geometria viva ([`crate::skin_desenho::funde`]). O [`recook`] fica para quem só quer o
/// caminho da cena (os gates, as sondas) e não tem onde pôr o desenho.
#[must_use]
pub fn recook_desenhando(
    sim: &SimWorld,
    scene: &mut VecScene,
) -> crate::skin_desenho::SkinDesenhado {
    recook_leis(sim, scene, crate::skin_desenho::Leis::do_ambiente())
}

/// ⭐⭐ **[`recook`] com a LEI EXPLÍCITA** — a porta que os gates das duas leis chamam.
///
/// ⛔⛔ **Ela existe porque um estado global posto para o teste é um CANAL ENTRE TESTES.** A 1.ª
/// redacção da F30 pôs um átomo com uma porta `forcar_lei`, e o doc dela dizia *«o nextest corre um
/// processo por teste»* — verdade para o `nextest`, **falsa** para o `cargo test`, que corre os
/// testes em THREADS do mesmo processo. A suíte reprovava em conjunto e passava sozinha, que é a
/// assinatura mais cara que há. ⇒ a lei é **parâmetro**, e quem lê o ambiente é o [`recook`].
pub fn recook_com(sim: &SimWorld, scene: &mut VecScene, curva: bool) {
    // ⚠️ **A porta do CAMPO lê-se AQUI**, no sítio de chamada, e viaja como parâmetro daí para
    // baixo — a mesma lei que a irmã da curva já escreve. Quem precisar de a controlar num gate
    // chama o [`recook_com_mistura`] directamente.
    let _ = recook_leis(
        sim,
        scene,
        crate::skin_desenho::Leis {
            curva,
            ..crate::skin_desenho::Leis::do_ambiente()
        },
    );
}

/// **O recook com as DUAS leis como parâmetro** — a da curva e a da MISTURA.
///
/// `rigido = false` é a mistura LINEAR, o caminho de antes de 2026-09-19: ela dá a CORDA do arco e
/// encolhe a arte, e é o **CONTROLO** dos gates que medem a cura do entalhe do cotovelo.
///
/// ⚠️ **Sem o desenho fiel** (`desenho: false`): os gates que a chamam medem a lei nos NÓS, que é
/// o que ela escreve no caminho da cena — e o bake custaria o relógio deles sem ser lido.
pub fn recook_com_mistura(
    sim: &SimWorld,
    scene: &mut VecScene,
    curva: bool,
    rigido: bool,
    campo: bool,
) {
    let _ = recook_leis(
        sim,
        scene,
        crate::skin_desenho::Leis {
            curva,
            rigido,
            campo,
            c1: ph2d_vec_skin::curva::lei_c1_activa(),
            desenho: false,
            contacto: false,
            efeitos: false,
        },
    );
}

/// ⭐⭐⭐ **O recook com TODAS as leis como parâmetro** — o corpo das três portas acima.
pub fn recook_leis(
    sim: &SimWorld,
    scene: &mut VecScene,
    leis: crate::skin_desenho::Leis,
) -> crate::skin_desenho::SkinDesenhado {
    let mut desenho = crate::skin_desenho::SkinDesenhado::new();
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
        // ⭐⭐⭐ **A GAVETA DESTA FORMA** ([`crate::skin_desenho::quadro`]): o que se deriva do BIND
        // (a fonte lida, o índice da malha do campo) sai de lá já feito, e um quadro em que nada
        // mudou devolve o anterior. ⚠️ A LEI é a mesma de sempre, ao bit — o que ela lê do bind
        // está descrito no [`crate::skin_desenho::calcula`], e as correcções à mão, a escolha do
        // artista (`SkinLaw`) e o campo do domínio passam pelas mesmas portas.
        //
        // Uma fonte corrompida é PULADA (não há o que deformar, e melhor não escrever lixo) — a
        // forma fica com a última geometria boa. Mesma escolha do envelope.
        let estilo = scene.paths().iter().find(|p| p.id == id).map_or(
            crate::skin_desenho::Estilo::NaoServe,
            crate::skin_desenho::estilo_de,
        );
        let eixos = || eixos_do_bind(sim, &skin, &index);
        let Some(q) = crate::skin_desenho::quadro(e.to_bits(), &skin, &pele, leis, &estilo, &eixos)
        else {
            continue;
        };
        if let Some(p) = scene.path_mut(id) {
            // ⭐⭐⭐ **GEOMETRIA, e nunca o estilo** — report do dono de 2026-09-19 (*«num vector
            // linkado aos ossos não consigo mudar a espessura do stroke»*). A fonte é a
            // FOTOGRAFIA do instante do `Bind`, e mandar o estilo dela para cá desfazia toda edição
            // de traço ou preenchimento no quadro seguinte. Ver o cabeçalho da
            // [`ph2d_vec_scene::recook`].
            p.replace_geometry(q.cru);
            // ⭐⭐⭐ **E O DESENHO FIEL leva o ESTILO VIVO** pela mesma porta: a geometria vem do
            // bake, o resto do caminho da cena. ⚠️ Menos a PILHA DE EFEITOS: o desenhado já a
            // leva cozida (em repouso), e um consumidor que o cozesse outra vez aplicá-la-ia duas.
            if let Some(d) = q.desenhado {
                let mut visto = p.clone();
                visto.replace_geometry(d);
                visto.effects.clear();
                desenho.insert(id, visto);
            }
        }
    }
    desenho
}

/// ⭐⭐⭐ **PRENDER** — o `bind`, a porta dele e o que ele guarda, num irmão pelo tecto de LOC.
#[path = "skin_live_prender.rs"]
mod prender;
pub use prender::{OssoPreso, bind, bind_com};
pub(crate) use prender::{eixos_do_bind, tendons_and_axes};

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
            && let Some(g) = crate::skinned_mesh::le(&skin.source)
            && let Some(p) = scene.path_mut(id)
        {
            // ⚠️ **GEOMETRIA**, pela mesma razão do quadro: o *Release* devolve **o que o artista
            // DESENHOU**, e a cor com que ele o pintou depois do `Bind` é dele.
            p.replace_geometry(g.path);
        }
        sim.world_mut().entity_mut(e).remove::<SkinBind>();
        feitos += 1;
    }
    feitos
}

/// ⭐ **Navegar o esqueleto** — as três perguntas que não são prender nem recozer, num irmão.
///
/// ⚠️ O `pub use` mantém `skin_live::chain_to` a ser `skin_live::chain_to` — ver o cabeçalho dele.
#[path = "skin_live_navega.rs"]
mod navega;
pub use navega::{chain_ends, chain_to, skinned_images_of_skeleton};

#[cfg(test)]
#[path = "skin_live_tests.rs"]
mod tests;

/// ⭐ **O gate do FIO do campo**, num irmão — ver o cabeçalho dele.
#[cfg(test)]
#[path = "skin_live_campo_tests.rs"]
mod campo_tests;

/// ⭐ **O gate do TRAÇO que sobrevive ao quadro**, num irmão — ver o cabeçalho dele.
#[cfg(test)]
#[path = "skin_live_traco_tests.rs"]
mod traco_tests;

/// ⭐⭐⭐ **Prender COZE os efeitos**, num irmão — ver o cabeçalho dele.
#[cfg(test)]
#[path = "skin_live_efeitos_tests.rs"]
mod efeitos_tests;

#[cfg(test)]
#[path = "skin_live_seleccao_tests.rs"]
mod seleccao_tests;
