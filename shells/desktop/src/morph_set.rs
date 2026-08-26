//! ⭐⭐ **O CONJUNTO DE ESTADOS — um botão, e as formas viram uma máquina** (plano 32 W8).
//!
//! Enio, 2026-08-25:
//!
//! > *"o usuário seleciona todas as peças que estarão envolvidas na máquina de estados do morph.
//! > Com o clique de um único botão um objeto novo surge na hierarquia tendo como filhos as shapes
//! > escolhidas. Todas as setas são atribuídas automaticamente cobrindo todas as morphs possíveis
//! > entre todas as formas (tanto de ida como de volta). As setas são virtuais e ninguém jamais vê.
//! > No canvas uma única shape aparece (a shape do estado atual) e as demais ficam ocultas."*
//!
//! # As quatro coisas que o clique faz, e por que são um só passo
//!
//! 1. nasce o objecto (um `VecPath` vazio + [`VecMorph`], que é o que a cena **desenha**);
//! 2. as formas escolhidas viram **filhos** dele (`ChildOf`), na ordem de z;
//! 3. cada uma ganha `Visibility::hidden()` — *no canvas aparece uma forma só*;
//! 4. o [`VecMorphMachine`] recebe o **grafo completo dirigido** sobre elas.
//!
//! ⚠️ **Um passo de undo, não quatro.** As quatro escritas acontecem no mesmo quadro e o
//! `post_frame_undo` regista por DIFF — um Ctrl+Z desfaz o conjunto inteiro, que é o que o gesto
//! promete. Reparentar num quadro e esconder no seguinte daria dois passos, e o primeiro deixaria
//! o artista com nove formas empilhadas.
//!
//! # ⛔ O que esta wave APAGOU, e não se reconstrói sem ler isto
//!
//! A W3a desenhava as setas no canvas (âmbar, entre as formas) e a W3b tinha um **modo** de arrasto
//! forma→forma que criava uma aresta. As duas morreram aqui, e não por gosto:
//!
//! - *"as setas são virtuais e ninguém jamais vê"* mata o desenho por decisão directa;
//! - o grafo passou a ser **completo por construção**, então o arrasto criaria uma aresta **que já
//!   existe**. Um gesto cujo produto já está lá é um gesto que não faz nada — e o modo dele seria
//!   um pill na fileira a competir com treze irmãos por uma resposta que ninguém pode ver.
//!
//! ⚠️ *Duas portas para a mesma pergunta divergem em silêncio*: com o botão a gerar `n(n-1)` e o
//! arrasto a acrescentar à mão, a lista deixaria de ser derivável e a próxima derivação apagaria
//! o trabalho do arrasto.

use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, VecMorph, VecMorphMachine};
use ph2d_morph_machine::{MorphGraph, MorphState};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// **O conjunto à espera da entidade dele nascer** — o `sync` do quadro seguinte é que cria a
/// entidade do path novo, e só aí há onde pendurar os componentes.
///
/// ⚠️ **Espelho do `vec_morph_pending`**, e um slot PRÓPRIO porque o payload é outro: aquele leva
/// um componente, este leva a máquina **e** a lista de quem vai ser reparentado.
#[derive(Clone, Debug)]
pub(crate) struct MorphSetPending {
    /// O path do objecto novo (a forma morfada, ainda vazia).
    pub(crate) path: VecPathId,
    /// O nome que a Hierarquia mostra.
    pub(crate) name: String,
    /// As formas-membro, na ordem de z — **a primeira é o estado inicial**.
    pub(crate) members: Vec<VecPathId>,
}

/// ⭐ **O MÍNIMO de formas que fazem um conjunto.**
///
/// ⚠️ **Não é um teto medido, é a DEFINIÇÃO**: um morph é a forma entre **duas** outras, então com
/// uma só não há entre nenhum. O número tem **dois** leitores — o [`create`], que recusa abaixo
/// dele, e o [`disconnect_row`], que **dissolve** ao chegar a ele. ⛔ Escrito à mão nos dois, a
/// fronteira dissolveria num sítio e recusaria noutro no dia em que alguém mudasse um.
pub(crate) const MIN_STATES: usize = 2;

/// **AS FORMAS DA SELEÇÃO QUE PODEM VIRAR ESTADOS.**
///
/// ⚠️ **Um estado é uma forma DESENHADA**, então o que entra é o que a cena sabe desenhar e o mapa
/// conhece.
///
/// ⛔ **Duas exclusões, e cada uma fecha uma porta diferente para o mesmo defeito** — *um conjunto
/// por cima de outro*:
///
/// 1. **uma entidade que já É um Morph** — a máquina de fora teria estados cuja geometria a de
///    dentro re-escreve a cada quadro, por baixo dela;
/// 2. **uma forma que já é MEMBRO de um conjunto** (o pai tem `VecMorphMachine`) — ela está oculta
///    e pertence a outra máquina; um segundo conjunto sobre ela dava dois donos à mesma forma, e o
///    artista alcança-a facilmente pela Hierarquia.
///
/// ⚠️ **A ordem é a da SELEÇÃO**, que é a de z — e ela é load-bearing: o primeiro membro é o
/// `start` do grafo, e é a forma que o artista vê quando o conjunto nasce.
#[must_use]
pub(crate) fn eligible(sim: &SimWorld, map: &VecEntityMap, sel: &[VecPathId]) -> Vec<VecPathId> {
    let mut out: Vec<VecPathId> = Vec::new();
    for id in sel {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let w = sim.world();
        if w.get_entity(e).is_err() || w.get::<VecMorph>(e).is_some() {
            continue;
        }
        let owned = w
            .get::<ChildOf>(e)
            .is_some_and(|c| w.get::<VecMorphMachine>(c.parent()).is_some());
        if owned {
            continue;
        }
        if !out.contains(id) {
            out.push(*id);
        }
    }
    out
}

/// ⭐⭐⭐ **O GRAFO, DERIVADO DOS FILHOS** — a lei da W11.
///
/// Enio, 2026-08-26: *"sendo uma forma que previamente não participava do Morph states, se for
/// arrastada na hierarquia e se tornar filha de um objeto Morph State, automaticamente passa a
/// fazer parte do sistema."*
///
/// ⇒ **arrastar para dentro É entrar, e nenhum código reage ao gesto** — não há gesto a que
/// reagir. A lista de formas é `Children(host)` filtrado ao que a cena sabe desenhar, e a tecla de
/// cada uma sai da tabela do componente (uma forma sem entrada usa os valores de partida).
///
/// É a lei que o módulo 3D Modeling já paga (`CLAUDE.md` §5.1): *a hierarquia da cena É o
/// documento, e o resto é cozido dela a cada quadro.*
///
/// ⚠️ **A ORDEM é a dos `Children`**, que é a ordem de inserção (= a de z), e o **primeiro é onde
/// a máquina nasce**. Reordenar irmãos na Hierarquia muda o estado inicial — e é a resposta certa:
/// o artista vê a ordem e pode mexer nela.
///
/// ⛔ **Um filho sem `VecPathRef` não entra** (um sprite, um grupo vazio): a máquina interpola
/// FORMAS, e um estado que não tem geometria não é interpolável. Ele fica na árvore, visível, e
/// simplesmente não participa.
///
/// ⛔ **Um filho que é ele próprio um Morph não entra** — a geometria dele é reescrita a cada
/// quadro por baixo da máquina de fora.
#[must_use]
pub(crate) fn graph_of(sim: &SimWorld, map: &VecEntityMap, host: Entity) -> MorphGraph {
    let w = sim.world();
    let Some(machine) = w.get::<VecMorphMachine>(host) else {
        return MorphGraph::default();
    };
    let Some(kids) = w.get::<ph2d_ecs::Children>(host) else {
        return MorphGraph::default();
    };
    let states = kids
        .iter()
        .filter_map(|&child| {
            if w.get::<VecMorph>(child).is_some() {
                return None;
            }
            // A forma é o `VecPathId` que o mapa conhece — a mesma porta que todo o resto usa.
            let id = path_of(map, child)?;
            Some(MorphState::with_key(id, &machine.key_of(id)))
        })
        .collect();
    MorphGraph { states }
}

/// **Cria o conjunto**: põe o path novo na cena e devolve o pendente. `None` se a seleção não dá
/// para um conjunto.
///
/// ⚠️ **O path nasce VAZIO de propósito** — a geometria é DERIVADA pelo `morph_live::recook` de
/// todo quadro, e inventá-la aqui seria uma 2ª porta para a mesma pergunta.
pub(crate) fn create(
    sim: &SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    sel: &[VecPathId],
    max_states: usize,
) -> Option<MorphSetPending> {
    let members = eligible(sim, map, sel);
    if members.len() < MIN_STATES || members.len() > max_states {
        return None;
    }
    let path = scene.push_path(VecPath::default());
    Some(MorphSetPending {
        // O nome diz **quantos estados**, que é a coisa que o artista quer reconhecer na árvore.
        name: format!("Morph States {}", members.len()),
        path,
        members,
    })
}

/// **Drena o pendente** — pendura os componentes, reparenta os membros e esconde-os.
///
/// Roda entre o `vec_entities::sync` (a entidade do path já existe) e o `morph_live::recook` — o
/// mesmo lugar do `morph_live::upkeep`, e pela mesma razão.
///
/// ⚠️ **Devolve `true` quando consumiu**, e o chamador limpa o slot. Se a forma sumiu entretanto
/// (o artista apagou), consome à mesma: um pendente que nunca resolve ficaria a tentar para sempre.
pub(crate) fn upkeep(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    pending: &mut Option<MorphSetPending>,
) {
    let Some(p) = pending.as_ref() else { return };
    if !scene.paths().iter().any(|q| q.id == p.path) {
        *pending = None;
        return;
    }
    let Some(&bits) = map.get(&p.path) else {
        return;
    };
    let host = Entity::from_bits(bits);
    if sim.world().get_entity(host).is_err() {
        return;
    }
    let start = p.members[0];
    let members: Vec<Entity> = p
        .members
        .iter()
        .filter_map(|id| map.get(id).map(|&b| Entity::from_bits(b)))
        .collect();
    let name = p.name.clone();

    // ⭐⭐ **ONDE O CONJUNTO FICA, e onde cada estado passa a ficar** (plano 32 W9).
    //
    // Enio, 2026-08-25: *"aqui, diferente da tool morph, todas as peças participantes são
    // alinhadas numa mesma posição e o morph states faz o morph numa mesma posição, não desloca a
    // peça de lugar."*
    //
    // ⚠️ **É a diferença de PRODUTO entre os dois objectos, e ela tem uma razão.** Um Morph
    // autorado entre duas formas em sítios diferentes *quer* que a forma viaje — é um efeito de
    // transição. Um conjunto de ESTADOS é um objecto só que muda de aparência: a personagem que
    // agacha não salta dois metros para a esquerda por isso.
    let centre = align(sim, scene, map, &p.members);
    if let Ok(mut e) = sim.world_mut().get_entity_mut(host) {
        // ⚠️ **`sources = [start, start]`, e `t` no zero.** O `VecMorph::new` nasce a meio caminho
        // de propósito (um morph a `t=0` sobre a forma A não se anuncia), e aqui é o contrário: o
        // conjunto tem de mostrar **exactamente** o estado inicial, senão a primeira coisa que o
        // artista vê é uma forma que ele nunca desenhou.
        e.insert((
            VecMorph {
                sources: [start, start],
                t: 0.0,
            },
            // ⭐ **A máquina nasce VAZIA de teclas** — e sem lista nenhuma: as formas são os
            // filhos que este mesmo `upkeep` pendura logo abaixo (W11).
            VecMorphMachine::new(),
            Name::new(name),
            // ⭐⭐ **O conjunto tem POSE, e é isso que o torna arrastável** (plano 32 W9).
            //
            // Enio, 2026-08-25: *"o objeto criado como pai tem que ser arrastável no canvas como um
            // objeto qualquer (…) e deve arrastar os filhos junto."*
            //
            // ⚠️ **Um Morph comum vive na IDENTIDADE de propósito** (`morph_live`: a geometria dele
            // é MUNDO, reescrita a cada quadro, e uma pose por cima a deslocaria duas vezes).
            // O conjunto quebra essa regra pelo outro lado: o `recook` coze-o no referencial DELE,
            // a partir das poses **locais** dos filhos, e é por isso que a pose pode existir.
            Transform::from_translation(centre),
        ));
    }
    for m in members {
        if let Ok(mut e) = sim.world_mut().get_entity_mut(m) {
            // ⚠️ **Só reparentar.** A ocultação é **DERIVADA** de ser filho de um conjunto
            // ([`is_set_member`]) — guardá-la aqui era a metade que não acompanhava o arrasto na
            // Hierarquia, nos DOIS sentidos (W11f).
            e.insert(ChildOf(host));
            // ⛔ O `RootOrder` sai: ele só vale para raízes, e um membro deixou de o ser. É o
            // mesmo par de operações do `vec_entities::group_entities`, e a razão é a mesma.
            e.remove::<ph2d_ecs::RootOrder>();
        }
    }
    *pending = None;
}

/// ⭐⭐ **ALINHA os membros num ponto só, e devolve esse ponto** (plano 32 W9).
///
/// Cada membro passa a ter o **centro do que se vê** exactamente sobre a origem do conjunto, e o
/// conjunto nasce no centro da caixa que os continha a todos — que é onde o artista estava a olhar.
///
/// # A conta, e por que ela preserva rotação, escala e cisalhamento
///
/// Para o membro `i`, com pose de mundo `translate(t) . M` (onde `M` é a parte linear) e centro de
/// mundo `c_i`, a pose LOCAL nova é `translate(t - c_i) . M`. Compondo com o pai (`translate(C)`),
/// o centro vai parar em `C + c_i - c_i = C` — **o mesmo ponto para todos**.
///
/// ⚠️ **Só a translação muda.** Girar ou re-escalar um estado ao alinhá-lo seria destruir o
/// desenho para o pôr no sítio; a diferença entre as formas é o que a máquina existe para mostrar.
///
/// ⚠️ **A caixa é a da CURVA** (`path_curve_bbox`), não a dos pontos de controlo: o artista alinha
/// o que vê, e uma tangente longe da tinta puxaria o centro para fora da forma.
///
/// ⚠️ **Os QUATRO cantos são transformados**, e não só o min/max local: sob rotação a caixa
/// alinhada aos eixos do mundo não é a imagem da caixa local, e usar dois cantos poria uma forma
/// girada fora do centro.
///
/// ⛔ **Um membro sem caixa fica onde está.** Uma forma degenerada não tem centro, e inventar um
/// arrastaria o resto do conjunto por causa dela.
fn align(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    members: &[VecPathId],
) -> ph2d_core::Vec2 {
    // 1) MEDE tudo antes de escrever o que quer que seja: a pose de mundo de cada membro e o
    //    centro de mundo do que ele desenha. Escrever a meio da medição faria o segundo membro ser
    //    medido contra um mundo que o primeiro já mexeu.
    let mut rows: Vec<(Entity, Transform, [f64; 2])> = Vec::new();
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for &id in members {
        let Some(&bits) = map.get(&id) else { continue };
        let e = Entity::from_bits(bits);
        let Some((bl, tr)) = scene.path_curve_bbox(id) else {
            continue;
        };
        let world = crate::vec_transform::world_transform(sim, e);
        let x = crate::vec_transform::xform_of_transform(world);
        let (mut cl, mut ch) = ([f64::MAX; 2], [f64::MIN; 2]);
        for corner in [
            [bl[0], bl[1]],
            [tr[0], bl[1]],
            [tr[0], tr[1]],
            [bl[0], tr[1]],
        ] {
            let w = x.apply(corner);
            for k in 0..2 {
                cl[k] = cl[k].min(w[k]);
                ch[k] = ch[k].max(w[k]);
            }
        }
        for k in 0..2 {
            lo[k] = lo[k].min(cl[k]);
            hi[k] = hi[k].max(ch[k]);
        }
        rows.push((e, world, [(cl[0] + ch[0]) * 0.5, (cl[1] + ch[1]) * 0.5]));
    }
    if rows.is_empty() {
        return ph2d_core::Vec2::ZERO;
    }
    let centre = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];

    // 2) ESCREVE: a pose local de cada membro, com só a translação mexida.
    for (e, world, own) in rows {
        #[allow(clippy::cast_possible_truncation)]
        let t = Transform {
            translation: ph2d_core::Vec2::new(
                world.translation.x - own[0] as f32,
                world.translation.y - own[1] as f32,
            ),
            ..world
        };
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.insert(t);
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ph2d_core::Vec2::new(centre[0] as f32, centre[1] as f32)
}

/// ⭐ **DESCONECTAR uma forma** — ela sai do conjunto e volta a ser solta e **visível**.
///
/// Enio, 2026-08-26: *"no lugar de clear melhor um botão de desconectar."* ⚠️ E o nome é o certo:
/// **não se apaga nada**. A forma continua no documento com o desenho dela; ela só deixa de ser um
/// estado.
///
/// # ⚠️ As DUAS coisas que ele desfaz, e nenhuma a mais
///
/// 1. o `ChildOf` — e é só isso que a tira da lista (a lista são os filhos, W11);
/// 2. a pose de MUNDO, para ela sair onde estava.
///
/// ⛔ **Ele NÃO mexe no `Visibility`** desde a W11f: a ocultação de um membro é **derivada**
/// ([`is_set_member`]), então sair do conjunto já a torna visível. Removê-la aqui destruiria o olho
/// da Hierarquia de quem tivesse escondido a forma **antes** de ela entrar.
///
/// ⛔ **A tecla dela FICA na tabela**, de propósito: voltar a arrastá-la para dentro devolve-lha.
/// Perder autoria por um gesto reversível seria pior do que não ter o gesto.
///
/// ⚠️ **A pose de MUNDO é preservada** (`reparent_keeping_world` ao contrário): ela sai onde
/// estava, e não onde a aritmética do pai a deixaria.
pub(crate) fn disconnect(sim: &mut SimWorld, map: &VecEntityMap, shape: VecPathId) -> bool {
    let Some(&bits) = map.get(&shape) else {
        return false;
    };
    let e = Entity::from_bits(bits);
    let world = crate::vec_transform::world_transform(sim, e);
    let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
        return false;
    };
    em.remove::<ChildOf>();
    em.insert(world);
    true
}

/// ⭐⭐ **DISSOLVER o conjunto** — o objecto some, as formas voltam soltas e visíveis, onde estavam.
///
/// Enio, 2026-08-26: *"precisamos de um botão para desfazer tudo em morph states."*
///
/// ⚠️ **É o inverso EXACTO do [`create`]/[`upkeep`]**, e desde a W11 não precisa de código próprio
/// de desmontagem: a lista são os filhos, então dissolver é **desconectar todos** e apagar o pai.
/// *Um botão que desfaz tem de chegar ao mesmo mundo de onde se partiu, e não a um parecido.*
///
/// ⛔ **O `ungroup_entities` NÃO serve** — medido: ele recusa um pai **com geometria** (`is_plain_group`),
/// e o conjunto tem `VecPathRef`. Reutilizá-lo teria sido silenciosamente inerte.
///
/// Devolve o path do objecto que deve ser removido da cena (o chamador tem-na à mão), ou `None`.
pub(crate) fn dissolve(sim: &mut SimWorld, map: &VecEntityMap, host: Entity) -> Option<VecPathId> {
    let shapes = graph_of(sim, map, host).shapes();
    for id in shapes {
        disconnect(sim, map, id);
    }
    // O path do próprio conjunto — o chamador apaga-o da cena, e o `sync` leva a entidade junto.
    path_of(map, host)
}

/// ⭐⭐ **O ⊘ DE UMA LINHA, inteiro** — a porta que o despacho chama (plano 32 W11d).
///
/// Ela compõe as **duas** metades que o gesto significa, e existe por isso: escritas como duas
/// linhas no braço do `match` do laço de render, elas **não são alcançáveis de um teste** — e a
/// segunda foi esquecida durante uma wave inteira.
///
/// 1. a forma sai do conjunto ([`disconnect`]) e recupera a pose de mundo;
/// 2. ⛔ **e leva as poses dela** ([`crate::vec_ui_state_edit::forget_object_in_all_states`]): um
///    estado grava a sub-árvore com a pose **LOCAL** de cada filho, então a pose antiga faria o
///    `install` do próximo Show atirar a forma solta para a origem do conjunto.
///
/// ⛔⛔ **E a TERCEIRA metade: tirar a penúltima DISSOLVE o conjunto** (W11f).
///
/// A pergunta estava aberta no §8.3 deste plano (*"um conjunto com uma forma ainda é um
/// conjunto?"*) e a medição respondeu-a: **não, e deixá-lo é pior do que um objecto inerte.** Um
/// conjunto esvaziado pelo ⊘ mantém o `VecMorph` que o [`upkeep`] lhe deu, e o `sources` continua a
/// nomear a **primeira forma** — que já saiu. ⇒ o artista desconecta as três, e fica com um
/// **fantasma** com o desenho da primeira, que ele não sabe o que é nem como apagar.
///
/// ⇒ **a fronteira é a do [`create`]**, que recusa abaixo de **2**: sair dela dissolve, exactamente
/// como o `ungroup` faz com o último filho. *Um objecto deixa de ser uma relação quando fica com um
/// lado só.*
///
/// Devolve o path do conjunto **se ele tiver de ser removido da cena** (o chamador tem-na à mão),
/// tal como o [`dissolve`].
pub(crate) fn disconnect_row(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    states: &mut ph2d_ui_state::StateSets,
    host: Entity,
    row: usize,
) -> Option<VecPathId> {
    let shapes = graph_of(sim, map, host).shapes();
    let shape = *shapes.get(row)?;
    // ⚠️ **A decisão vem ANTES da escrita**, e com a contagem de agora: depois de desconectar, a
    // lista já encolheu e a fronteira leria-se ao contrário.
    if shapes.len() <= MIN_STATES {
        return dissolve(sim, map, host);
    }
    disconnect(sim, map, shape);
    if let Some(h) = path_of(map, host) {
        crate::vec_ui_state_edit::forget_object_in_all_states(states, h, shape);
        // ⭐⭐⭐ **E OUTRA FORMA DO CONJUNTO TOMA O LUGAR DELA nas poses** (W11h) — Enio,
        // 2026-08-26: *"se o usuário desconectar uma shape, coloque outra shape do conjunto em seu
        // lugar de modo a não quebrar as anims."*
        //
        // ⚠️ **São duas coisas diferentes e as duas são precisas:** a de cima tira a pose da forma
        // que saiu (senão o Show a puxa de volta para dentro do conjunto); esta arruma as poses do
        // **HOSPEDEIRO** que a nomeavam como *«a forma que eu mostro»*.
        let rest: Vec<VecPathId> = shapes.into_iter().filter(|s| *s != shape).collect();
        crate::vec_ui_state_edit::replace_morph_shape_in_all_states(states, h, shape, &rest);
    }
    None
}

/// ⭐⭐⭐ **SER MEMBRO É ESTAR ESCONDIDO — e isso é DERIVADO, nunca guardado** (plano 32 W11f).
///
/// # ⛔⛔ O que estava errado, medido em 2026-08-26
///
/// A W11 fez a lista de estados ser **os filhos**, mas a ocultação continuou a ser uma escrita do
/// [`upkeep`] — um `Visibility::hidden()` guardado no momento da criação. As duas metades ficaram
/// desalinhadas, e o gesto do Enio (*"arrastar na hierarquia"*) apanha **as duas pontas**:
///
/// | gesto | a lista | o canvas (antes) |
/// |---|---|---|
/// | arrastar para DENTRO | entra (3 -> 4 estados) | ⛔ **continua visível**, desenhada por cima do conjunto |
/// | arrastar para FORA | sai (4 -> 3 estados) | ⛔ **continua escondida** — a forma **desaparece** |
///
/// ⚠️ A segunda é a pior das duas, e o doc do [`disconnect`] já a nomeava como *"a pior saída
/// possível"* — para o botão ⊘. O gesto de arrasto chegava lá pela porta que ninguém tinha olhado.
///
/// # A lei
///
/// ⇒ **é a hierarquia que responde**, como responde à lista: *o meu pai tem máquina, logo eu sou um
/// estado, logo eu não me desenho*. É a lei que o módulo 3D Modeling já paga (`CLAUDE.md` §5.1: *«a
/// hierarquia da cena É o documento»*) e que a §8.2 deste plano previu — só que ela foi aplicada à
/// lista e **não** à ocultação.
///
/// ⭐ Com isto os dois gestos ficam de graça e **simétricos**, e não há estado guardado que possa
/// discordar da árvore. ⛔ O `Visibility` do artista (o olho da Hierarquia) fica **intacto**: esta
/// função só ACRESCENTA uma razão para esconder, e sair do conjunto devolve a razão dele.
#[must_use]
pub(crate) fn is_set_member(w: &ph2d_ecs::World, e: Entity) -> bool {
    w.get::<ChildOf>(e)
        .is_some_and(|c| w.get::<VecMorphMachine>(c.parent()).is_some())
}

/// **O `VecPathId` desta entidade** — a busca inversa do [`VecEntityMap`].
///
/// ⚠️ **Uma porta, e não a terceira cópia:** este `find` estava escrito à mão em dois sítios deste
/// arquivo e o despacho precisava do terceiro. *Uma lei escrita em dois sítios ainda não é uma lei —
/// só uma PORTA é* (a lição do `stroke_uniform`, no mesmo módulo).
#[must_use]
pub(crate) fn path_of(map: &VecEntityMap, e: Entity) -> Option<VecPathId> {
    map.iter()
        .find(|&(_, &b)| b == e.to_bits())
        .map(|(&k, _)| k)
}

#[cfg(test)]
#[path = "morph_set_tests.rs"]
mod tests;
