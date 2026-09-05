//! ⭐ **AS TRAVESSIAS que os verbos partilham** — irmão por ASSUNTO do [`super::instance_verbs`].
//!
//! Elas não são um verbo: são as quatro perguntas que TODO verbo faz antes de agir — *quem tem
//! este `StableId`?*, *qual é a raiz da instância a que isto pertence?*, *que entidades estão
//! debaixo desta?*. Estavam no fim do ficheiro dos verbos, e saem para cá quando ele bateu no
//! tecto de 600 LOC do shell (HR-18).
//!
//! ⚠️ **O corte é o mesmo que o `action_bus_queue` fez, e pela mesma razão:** o que sai é o bloco
//! do FIM, onde ninguém escreve. O `drain` cresce por acrescento de braço **no meio**, e movê-lo
//! poria toda linha paralela que acrescenta um verbo em conflito textual com esta.

use ph2d_ecs::{Children, Entity, InstanceOf, MasterRoot, SimWorld, StableId};

/// `StableId → entidade`, do mundo inteiro.
pub(crate) fn stable_index(sim: &mut SimWorld) -> std::collections::BTreeMap<u64, Entity> {
    let mut q = sim.world_mut().query::<(Entity, &StableId)>();
    q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect()
}

/// **A entidade que tem este `StableId`**, em bits — a porta do navegador de assets (plano
/// `docs/Components/07`, wave A7).
///
/// ⚠️ **Ela vive AQUI, ao lado do [`stable_index`] que já existia**, e não no shell: uma segunda
/// travessia `StableId → Entity` escrita noutro ficheiro seria a segunda resposta à mesma
/// pergunta, e as duas divergiriam no dia em que a identidade mudasse de forma.
///
/// ⛔ Devolve `None` para um id que já não existe — é o caso normal, não um erro: o navegador
/// publica o índice de um quadro, o artista apaga a receita, e o duplo-clique chega a seguir.
pub(crate) fn entity_for_stable_id(sim: &mut SimWorld, stable_id: u64) -> Option<u64> {
    stable_index(sim).get(&stable_id).map(|e| e.to_bits())
}

/// A raiz da instância a que `clicked` pertence — a peça cujo mestre é um [`MasterRoot`].
///
/// ⚠️ Sobe por `ChildOf`, nunca pelo elo: o `InstanceOf` de uma peça aponta para a peça do MESTRE,
/// e subir por ele sairia da instância e ia parar à receita.
///
/// ⚠️⚠️ **A 1.ª linha era `get::<InstanceOf>(clicked)?;` — um bail que apagava a travessia inteira**
/// (auditoria §1.8, 2026-08-27), e o doc do [`make_master`] prometia o contrário. Toda peça nascida
/// da cópia profunda tem elo, mas o que for acrescentado DEPOIS não tem: um *Add Child* sobre uma
/// peça, um reparent para dentro da cópia, um path vetorial cunhado pelo `vec_entities::sync` e
/// arrastado para lá. ⇒ a recusa `InsideAnInstance` não disparava e nascia um `MasterRoot` **dentro
/// de uma instância viva**, cuja sub-árvore virava `MasterPiece`: um pedaço de uma cópia que estava
/// visível desaparecia, com a cópia à volta a continuar a desenhar.
///
/// ⚠️ O oráculo do gate que sancionava o bail usava uma peça que TINHA elo (`piece(&sim, inst,
/// "Arm")`), então a travessia ancestral nunca corria — *o oráculo confirmava o caminho curto e
/// assinava o longo*.
pub(crate) fn instance_root_of(sim: &mut SimWorld, clicked: Entity) -> Option<Entity> {
    let by_id = stable_index(sim);
    let mut e = clicked;
    loop {
        let is_root = sim
            .world()
            .get::<InstanceOf>(e)
            .and_then(|l| by_id.get(&l.master))
            .is_some_and(|&m| sim.world().get::<MasterRoot>(m).is_some());
        if is_root {
            return Some(e);
        }
        e = sim.world().get::<ph2d_ecs::ChildOf>(e)?.0;
    }
}

/// Esta entidade — ou algum ancestral dela — é peça de uma instância?
///
/// ⚠️⚠️ **NÃO é a mesma pergunta que [`is_a_recipe_given_piece`], e eu colapsei as duas em
/// 2026-09-05 antes de um gate mo dizer na primeira corrida.** Esta responde *«estou DENTRO de uma
/// cópia?»* — é o que o `make_master` precisa, porque um `MasterRoot` a meio de uma cópia viva
/// encurta a sub-árvore de edição **venha a peça de onde vier**. A outra responde *«a receita DEU
/// isto?»*, que exclui o que o artista pendurou lá dentro. *Duas leis que só se parecem: apertar
/// uma para servir a outra recusa um gesto legítimo.*
pub(crate) fn belongs_to_an_instance(sim: &mut SimWorld, entity: Entity) -> bool {
    instance_root_of(sim, entity).is_some()
}

/// ⭐⭐⭐ **Esta peça foi DADA pela receita** — está dentro de uma cópia e **não** é a raiz dela.
///
/// # A lei, escrita uma vez
///
/// *Só o que a receita deu é que a receita tira* — é a frase que o passe estrutural
/// ([`crate::instance_structure`]) já vive por: uma entidade **sem** elo dentro de uma cópia é
/// autoria do artista e ninguém lhe toca; uma **com** elo veio do mestre, e a forma da cópia é a
/// forma da receita.
///
/// ⚠️ **A RAIZ é a excepção, e é ela que faz a pergunta ter sentido:** apagar uma cópia inteira é
/// um gesto normal (é um objecto da cena), e apagar **uma peça dela** não é — a peça volta no passe
/// seguinte, porque o mestre continua a tê-la.
///
/// ⚠️ **Uma cópia ANINHADA responde `true`**: a roda que vive dentro de um carro da cena é a raiz
/// de uma instância *da Roda*, mas continua a ser uma peça que a receita do Carro deu — e o
/// `instance_root_of` sobe até à raiz **mais externa**, que é o que dá esta resposta de graça.
///
/// ⛔ **Ela nasceu porque a condição estava escrita DUAS vezes** — aqui e no `make_master` (a
/// recusa `InsideAnInstance`) — e um report de 2026-09-05 mostrou que faltava um terceiro leitor:
/// o **apagar**. *Uma lei escrita em dois sítios ainda não é uma lei; só uma PORTA é.*
pub(crate) fn is_a_recipe_given_piece(sim: &mut SimWorld, entity: Entity) -> bool {
    // ⚠️ **O ELO é a metade que separa esta pergunta da irmã de cima** — sem ele, um *Add Child*
    // do artista dentro de uma cópia lia-se como peça da receita e o apagar era recusado. O gate
    // apanhou-o na primeira corrida.
    if sim.world().get::<InstanceOf>(entity).is_none() {
        return false;
    }
    matches!(instance_root_of(sim, entity), Some(root) if root != entity)
}

/// A subárvore de `root`, ela incluída.
pub(crate) fn subtree(sim: &SimWorld, root: Entity) -> Vec<Entity> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        out.push(e);
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    out
}

/// ⭐⭐ **A RECEITA que esta linha representa** — ela própria, se for uma; a de que é cópia, se for
/// uma cópia; ela própria (para o verbo recusar com voz) se não for nem uma coisa nem outra.
///
/// ⚠️ **Uma porta, e não uma escada em cada chamador**: a resolução `cópia -> receita` é a mesma
/// travessia que o *Apply*, o *Revert* e o *Detach* já fazem (`instance_root_of`), e escrevê-la
/// outra vez daria duas respostas a *«de que receita esta linha é?»*.
pub(crate) fn master_subject(sim: &mut SimWorld, clicked: Entity) -> Entity {
    if sim.world().get::<MasterRoot>(clicked).is_some() {
        return clicked;
    }
    let Some(root) = instance_root_of(sim, clicked) else {
        return clicked;
    };
    let Some(master_id) = sim.world().get::<InstanceOf>(root).map(|l| l.master) else {
        return clicked;
    };
    entity_for_stable_id(sim, master_id).map_or(clicked, Entity::from_bits)
}
