//! ⭐ **INSTANCIAR** — a porta do produto (ADR-0164 / plano F4.2).
//!
//! Ela compõe as duas metades que sozinhas não fazem uma instância:
//!
//! 1. [`ph2d_ecs::deep_copy_subtree`] — copia os bytes de toda a subárvore e dá **identidade
//!    nova** a cada peça;
//! 2. [`crate::instance_refs::remap_object_refs`] — reescreve as referências guardadas por
//!    identidade, para que a junta da instância prenda **os corpos dela**.
//!
//! ⛔ **Nunca chame a primeira sozinha do produto.** Uma cópia sem remap é o defeito que esta
//! wave existe para curar, e ele é MUDO: a junta prende no mestre (que não simula), então as
//! peças da instância caem soltas e nada na tela diz porquê. O gate
//! `only_the_instantiate_door_calls_the_deep_copy` mantém esta função como o único chamador.

use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Entity, InstanceOf, MasterRoot, Name, SimWorld, StableId};

/// **Por que uma instanciação foi recusada** — e não um `None`, porque as razões pedem frases
/// diferentes ao artista.
///
/// ⚠️ A mensagem mora no **gesto** (F4.5), não aqui: esta porta responde o FATO, e quem tem UI
/// escolhe as palavras. *Duas recusas que devolvem o mesmo `None` produzem o mesmo aviso inútil.*
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// A subárvore escolhida não é uma receita.
    NotAMaster,
    /// A instância aterraria **dentro do próprio mestre** — ver [`instantiate_master`].
    WouldNestInItself,
}

/// ⭐⭐⭐ **A cópia tem arte PRÓPRIA, ou DIVIDE a do mestre?** (Enio, 2026-08-27.)
///
/// É a escolha do Blender entre `Shift+D` e `Alt+D`, aplicada ao que uma cópia é aqui — e ela vale
/// para as DUAS artes ao mesmo tempo, a tinta e o desenho, que é o ponto: até 2026-08-27 os pixels
/// respondiam uma coisa e a geometria vetorial a outra, e o artista não tinha por onde saber.
///
/// ⚠️ **Um `bool` aqui seria lido ao contrário** no dia em que alguém passasse `true` a pensar em
/// *«é uma instância»*. O nome vive no tipo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ArtLink {
    /// **Arte própria.** Editar o desenho ou a tinta desta cópia vira uma **excepção dela**; as
    /// irmãs não mudam. É o `Shift+D`, e é o que *Instantiate* faz.
    Own,
    /// **Arte do mestre.** A edição **sobe à receita** e o passe seguinte leva-a a todas as
    /// cópias. É o `Alt+D`, e é o que *Instantiate Linked* faz.
    ///
    /// ⚠️ Só a ARTE — a pose, o `tint` e os componentes continuam a ser desta cópia. Ver
    /// [`ph2d_ecs::LinkedArt`].
    Shared,
}

/// **Instancia o mestre `master_root`**, devolvendo a raiz da instância.
///
/// `parent` diz onde ela aterra (`None` = raiz da cena).
///
/// ⛔ **Recusa** (ver [`Refusal`]):
///
/// - `master_root` não é um mestre. Pôr um [`InstanceOf`] a apontar para uma subárvore que não é
///   receita daria ao sync (F4.3) um mestre que o artista edita como um objeto qualquer, e cada
///   edição da cena seria propagada como se fosse autoria de biblioteca.
/// - o destino está **dentro do próprio mestre**. Isso poria a receita a conter uma instância de
///   si mesma: o sync propagaria o mestre para dentro do mestre — que cresce a cada quadro — e a
///   cópia profunda seguinte copiaria a cópia. ⚠️ *A recusa é no GESTO e não um tecto de
///   profundidade*: um limite numérico transformaria um erro de autoria numa contagem, e o artista
///   veria a árvore crescer até um número que ninguém lhe explicou.
///
/// `link` escolhe **qual das duas leis** a cópia segue — ver [`ArtLink`].
pub(crate) fn instantiate_master(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    master_root: Entity,
    parent: Option<Entity>,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
    link: ArtLink,
) -> Result<Entity, Refusal> {
    if sim.world().get::<MasterRoot>(master_root).is_none() {
        return Err(Refusal::NotAMaster);
    }
    if let Some(p) = parent
        && is_self_or_descendant(sim, p, master_root)
    {
        return Err(Refusal::WouldNestInItself);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let Some(master_id) = sim.world().get::<StableId>(master_root).map(|s| s.0) else {
        return Err(Refusal::NotAMaster);
    };
    let base = sim
        .world()
        .get::<Name>(master_root)
        .map_or_else(|| "Instance".to_string(), |n| n.0.clone());

    let Ok(copy) = ph2d_ecs::deep_copy_subtree(sim.world_mut(), registry, master_root, parent)
    else {
        return Err(Refusal::NotAMaster);
    };
    // ⭐⭐ **Os DOCUMENTOS possuídos** (F4.6) — a cópia profunda salta-os de propósito, e sem esta
    // metade uma peça vetorial nasce **sem geometria nenhuma**: uma linha na Hierarquia que não
    // desenha um pixel. Ver [`crate::instance_docs`], onde a lista dos quatro está declarada.
    let report = crate::instance_docs::clone_owned_documents(sim, registry, docs, &copy);
    report.warn("instanciar");
    let pieces = copy.copies();

    // ⚠️⚠️ **A ORDEM destes dois passos é load-bearing, e o erro é silencioso.**
    //
    // O mapa contém `mestre → cópia do mestre` (tem de conter: uma junta ancorada na raiz da
    // receita precisa dele). Se o `InstanceOf` fosse inserido ANTES, o remapeador dele — que é
    // uma linha da mesma tabela — reescreveria o elo para a identidade da **própria cópia**, e a
    // instância passaria a dizer-se instância de si mesma. O sync da F4.3 leria isso como *"o
    // mestre sou eu"* e nunca mais propagaria nada.
    //
    // ⇒ remapear primeiro, ligar depois. Gate: `the_instance_points_at_the_master_not_at_itself`.
    crate::instance_refs::remap_object_refs(sim.world_mut(), &pieces, &copy.stable_ids);

    // ⭐⭐ **CADA PEÇA guarda de que peça do mestre nasceu** (F4.3), e não só a raiz.
    //
    // É esta a correspondência DURÁVEL de que o sync vive: ela sobrevive ao save, ao undo e —
    // sobretudo — a **o mestre ganhar ou perder uma peça**, que é o momento em que emparelhar por
    // posição na árvore (o caminho óbvio e barato) passa a emparelhar peças erradas em silêncio.
    //
    // ⚠️ A raiz é o caso particular: ela é a peça cujo `master` é um [`MasterRoot`], e é assim que
    // *«esta entidade é a raiz de uma instância»* se responde sem um segundo componente.
    for (&src, &dst) in &copy.entities {
        let Some(id) = sim.world().get::<ph2d_ecs::StableId>(src).map(|s| s.0) else {
            continue;
        };
        sim.world_mut()
            .entity_mut(dst)
            .insert(InstanceOf { master: id });
        // ⭐⭐ **A marca da cópia LIGADA acompanha o elo, peça a peça** — ver [`ArtLink`] e
        // [`ph2d_ecs::LinkedArt`]. Os dois consumidores (a tinta e o documento) têm em mão a peça
        // que o artista tocou, nunca a raiz.
        if link == ArtLink::Shared {
            sim.world_mut().entity_mut(dst).insert(ph2d_ecs::LinkedArt);
        }
    }

    let unique = crate::name_unique::unique_name(sim, &base);
    let mut root = sim.world_mut().entity_mut(copy.root);
    // ⚠️ A instância NÃO é um mestre: com o marcador ela nasceria **inerte** (F4.1) — três
    // ragdolls no lugar certo, nenhum a cair.
    root.remove::<MasterRoot>();
    root.insert(InstanceOf { master: master_id });
    root.insert(Name::new(unique));

    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
    // As peças da cópia deixam de ser peças de mestre no mesmo quadro em que nascem — sem isto
    // elas só voltariam a simular no próximo passe da ponte.
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    Ok(copy.root)
}

/// **`candidate` é o próprio `root` ou está debaixo dele?** — a pergunta do ciclo.
///
/// Sobe por `ChildOf`, que é `O(profundidade)` e corre uma vez por gesto. ⚠️ Sem guarda de ciclo
/// na travessia **de propósito**: a hierarquia da casa não tem ciclos (o reparent recusa-os), e
/// inventar aqui uma segunda defesa esconderia a primeira se ela algum dia partisse.
fn is_self_or_descendant(sim: &SimWorld, candidate: Entity, root: Entity) -> bool {
    let mut e = candidate;
    loop {
        if e == root {
            return true;
        }
        match sim.world().get::<ph2d_ecs::ChildOf>(e) {
            Some(c) => e = c.0,
            None => return false,
        }
    }
}

/// ⭐ **DUPLICAR** — a mesma cópia profunda, **sem** elo ao original.
///
/// A cópia aterra ao lado da fonte (mesmo pai) e é um objeto independente. As referências internas
/// são remapeadas pela mesma tabela: *a junta de uma cópia prende os corpos DELA*.
///
/// ⚠️ **Isto substitui uma cópia RASA** que levava quatro componentes (`Transform`, `Sprite`,
/// `Name`, `ChildOf`) e **nenhum filho** — duplicar um ragdoll dava uma linha vazia na Hierarquia.
/// O ADR-0164 nomeia esse defeito; ele existia porque copiar bytes de tipos que a shell não conhece
/// não tinha porta, e agora tem.
///
/// ⚠️ **Uma cópia de um MESTRE é outro mestre** (o `MasterRoot` viaja no blob), e uma cópia de uma
/// INSTÂNCIA é outra instância do mesmo mestre (o elo aponta para fora do que se copiou, e por isso
/// o remap não lhe toca). As duas são o que o artista espera de *Duplicar*.
///
/// ⚠️⚠️ **`step` é um degrau de MUNDO derivado da tela**, e ele existe porque a cópia aterrava
/// **exactamente em cima da fonte** (auditoria §1.4, 2026-08-27): o ramo VETORIAL do mesmo `if`
/// deslocava por `PASTE_OFFSET_PX` e este não deslocava nada, então o toast dizia «Duplicated
/// entity» e a tela ficava idêntica. ⛔ **Não é a lei do `cascade`** do *Instantiate*, e confundi-las
/// escreve o defeito outra vez com outro sinal: aquele conta as instâncias que já existem
/// (`instances_of`), e um *Duplicate* de uma sprite não tem mestre para contar. Aqui é **um** degrau,
/// sempre — a pergunta é *«saiu de cima do que veio?»*, não *«a quantas cópias vai?»*.
///
/// ⚠️ **O degrau soma-se ao `Transform` LOCAL**, como no `cascade`: sob um pai escalado ele sai
/// maior ou menor na tela. A propriedade que o gate defende é a separação ser **> 0**, e não o
/// número de pixels; convertê-lo para o espaço do pai custaria um inverso por gesto para mover um
/// artefacto que ninguém vê.
pub(crate) fn duplicate_subtree(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    src: Entity,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
    step: [f32; 2],
) -> Option<Entity> {
    let parent = sim.world().get::<ph2d_ecs::ChildOf>(src).map(|c| c.0);
    let base = sim
        .world()
        .get::<Name>(src)
        .map_or_else(|| "Entity".to_string(), |n| n.0.clone());

    let copy = ph2d_ecs::deep_copy_subtree(sim.world_mut(), registry, src, parent).ok()?;
    // ⭐ A mesma metade que a instanciação paga: sem ela, duplicar um GRUPO com formas vetoriais
    // dentro devolve as peças sem geometria (F4.6).
    crate::instance_docs::clone_owned_documents(sim, registry, docs, &copy).warn("duplicar");
    crate::instance_refs::remap_object_refs(sim.world_mut(), &copy.copies(), &copy.stable_ids);

    let unique = crate::name_unique::unique_name(sim, &base);
    sim.world_mut()
        .entity_mut(copy.root)
        .insert(Name::new(unique));
    // ⭐ **Sai de cima da fonte** — ver o doc: sem isto o gesto inteiro é um toast.
    if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(copy.root) {
        t.translation.x += step[0];
        t.translation.y += step[1];
    }
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
    // A cópia de um mestre é um mestre: as peças dela têm de ser marcadas já.
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    Some(copy.root)
}

#[cfg(test)]
#[path = "instantiate_tests.rs"]
mod tests;
