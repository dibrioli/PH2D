//! ⭐ **A row *Duplicate* da Hierarquia** — irmão de [`super::hierarchy`] por ASSUNTO (e porque
//! aquele ficheiro voltou ao tecto de 600 LOC).
//!
//! Lá mora o dreno das dezassete intenções da lista; aqui mora **o que duplicar quer dizer** — a
//! escolha de quem sabe copiar cada família ([`DuplicateKind`]) e as duas leis que a cópia tem de
//! cumprir para o artista a VER. Os gates do roteamento continuam no terceiro irmão,
//! `hierarchy_duplicate_routing_tests.rs`.

use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue, screens::hero::HeroScreen};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

/// ⭐ **Quem sabe duplicar esta entidade.**
///
/// ⚠️ A decisão mora aqui, com nome, porque **a escolha errada é silenciosa**: o braço genérico
/// copia `Transform` + `Sprite` + `Name`, e para uma entidade que guarda a geometria noutro sítio
/// isso produz um **sósia que não desenha nada** — uma linha na Hierarchy sobre coisa nenhuma.
///
/// Já aconteceu duas vezes, em dois módulos:
///
/// | Entidade | O que o braço genérico produzia | Quem duplica de verdade |
/// |---|---|---|
/// | um **path vetorial** (`VecPathRef`) | um sósia sem geometria — ou, pior, dois donos do mesmo path | o documento vetorial, pela porta do painel |
/// | um **nó de modelagem 3D** (`FieldNode`) | uma linha sem `FieldNode` nem `FieldPose`, invisível ao traçado | `field3d_scene::duplicate_node`, a porta do painel |
///
/// *Uma entidade cuja geometria não está nela não se duplica clonando-a.*
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DuplicateKind {
    /// Um nó de modelagem 3D (ADR-0161).
    Field,
    /// Um path do editor vetorial (ADR-0110).
    VecPath,
    /// Tudo o resto: sprites e entidades comuns, que **são** o que guardam.
    Entity,
}

/// Ver [`DuplicateKind`].
pub(super) fn duplicate_kind(
    world: &bevy_ecs::world::World,
    src: ph2d_ecs::Entity,
) -> DuplicateKind {
    if world.get::<ph2d_field_ecs::FieldNode>(src).is_some() {
        DuplicateKind::Field
    } else if world.get::<ph2d_ecs::VecPathRef>(src).is_some() {
        DuplicateKind::VecPath
    } else {
        DuplicateKind::Entity
    }
}

/// ⭐⭐ **O dreno da row.** Devolve `true` quando alguma coisa mudou (o `title_dirty` do chamador).
///
/// ⚠️⚠️ **As DUAS leis que o ramo genérico não tinha** (auditoria de 2026-08-27, §1.4 e §1.2): o
/// ramo de MODELAGEM seleccionava a cópia, o VETORIAL deslocava-a um degrau de tela, e o genérico
/// — sprites, grupos, instâncias e **receitas** — não fazia nem uma coisa nem outra. Duplicar
/// deixava a cópia exactamente em cima da fonte (o gesto inteiro era um toast) e duplicar uma
/// receita deixava-a **invisível**. *Três ramos do mesmo `if` a responder três coisas à mesma
/// pergunta.*
///
/// ⛔⛔ **E a 1.ª cura da invisibilidade era o SINTOMA:** seleccionar a cópia mostra-a **uma vez**,
/// e o clique seguinte apaga-a outra vez — o Enio voltou a reportá-lo no mesmo dia. A causa era a
/// cópia de uma receita ser uma segunda receita; hoje ela é um objeto comum
/// ([`crate::instantiate::duplicate_subtree`]). *Curar o sintoma de um objeto invisível é
/// mostrá-lo uma vez.*
#[allow(clippy::too_many_arguments)] // o mundo, a câmara, a voz, os dois documentos e a saída
pub(super) fn drain(
    src: ph2d_ecs::Entity,
    entity_bits: u64,
    hero: &mut HeroScreen,
    sim: &mut SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    toasts: &mut ToastQueue,
    vec_scene: &mut ph2d_vec_scene::VecScene,
    vec_entities: &mut crate::vec_entities::VecEntityMap,
    vec_history: &mut ph2d_vec_edit::History,
    vec_pen: &mut ph2d_vec_edit::PenTool,
    duplicate_made: &mut Option<(u64, u64)>,
    registry: &ph2d_ecs::scene::ComponentRegistry,
) -> bool {
    // ⚠️ **Uma forma VETORIAL não se duplica clonando a entidade.** O dono da geometria é o
    // documento, e `vec_entities::sync` mantém UMA entidade por path, nas duas direções:
    //
    // - clonar a entidade sem o `VecPathRef` dá um sósia que **não desenha nada** — uma linha
    //   na Hierarchy sobre geometria nenhuma, que era o que esta row fazia;
    // - copiar o `VecPathRef` seria pior: duas entidades a apontar para o MESMO path, e o
    //   `sync` tem de escolher uma.
    //
    // Então o clone é um **PATH**, feito pela porta que o botão Duplicate do painel usa, e o
    // `sync` cunha a entidade dele (com nome único e `RootOrder`) no mesmo frame.
    // ⚠️ **Quem duplica esta entidade não é óbvio, e a escolha errada é SILENCIOSA** — ver
    // [`duplicate_kind`], que é onde a decisão mora (e onde um gate lhe chega).
    if duplicate_kind(sim.world(), src) == DuplicateKind::Field {
        if let Some(copy) = crate::field3d_scene::duplicate_node(sim.world_mut(), src) {
            // ⭐ A cópia fica selecionada, como no botão do painel: é o que põe o gizmo em cima
            // dela sem ninguém a ter de procurar.
            hero.gizmo.replace_selection(Some(copy));
            toasts.push(Toast::success("Duplicated shape"));
            return true;
        }
        return false;
    }
    // O degrau de TELA, convertido pela câmara — o mesmo dos dois ramos que sobram.
    let (dx, dy) = crate::input_dispatch::screen_offset_world(
        camera,
        window_size,
        crate::input_dispatch::PASTE_OFFSET_PX,
    );
    if let Some(vp) = sim.world().get::<ph2d_ecs::VecPathRef>(src).copied() {
        if crate::input_dispatch::duplicate_vec_paths(
            vec_scene,
            vec_history,
            vec_pen,
            &[vp.0],
            dx,
            dy,
        ) {
            toasts.push(Toast::success("Duplicated shape"));
            return true;
        }
        return false;
    }
    // ⭐ **A cópia é PROFUNDA** (ADR-0164 / F4.2) — a subárvore inteira, todo componente
    // registado, identidade nova, e as referências internas remapeadas.
    //
    // ⚠️ **O que estava aqui antes copiava QUATRO componentes** (`Transform`, `Sprite`, `Name`,
    // `ChildOf`) **e nenhum filho**: duplicar um ragdoll dava uma linha vazia na Hierarquia, e
    // duplicar um corpo com junta dava um corpo solto. O ADR-0164 nomeia este defeito, e ele
    // existia por falta de porta, não por decisão.
    //
    // ⚠️ **O nome único continua a ser lei** — a Hierarquia já teve seleção por rótulo, e qualquer
    // código que volte a chavear pelo nome amigável merece a mesma defesa.
    let sprite = sim.world().get::<ph2d_render::Sprite>(src).is_some();
    let recipe = sim.world().get::<ph2d_ecs::MasterRoot>(src).is_some();
    // ⚠️ **A outra metade do report** (Enio, 2026-08-27): duplicar uma PEÇA de dentro de um
    // componente deixa a cópia dentro dele — o que está certo (ela passou a fazer parte da
    // receita) e some assim que o artista muda de selecção. Aqui o defeito não é o resultado,
    // é o SILÊNCIO: um toast de sucesso sobre um objeto que não aparece lê-se como o mesmo
    // bug. Lido ANTES da cópia, senão a resposta é sobre a cópia e não sobre onde ela cai.
    let inside = !recipe && ph2d_ecs::master_root_of(sim.world(), src).is_some();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene,
        vec_entities,
    };
    let Some(copy) = crate::instantiate::duplicate_subtree(
        sim,
        registry,
        src,
        &mut docs,
        [dx as f32, dy as f32],
    ) else {
        return false;
    };
    // Report the pair so the caller can fork the copy's texture off the source (independent
    // object) + flush any live paint on the source first. Only matters for sprite entities.
    if sprite {
        *duplicate_made = Some((entity_bits, copy.to_bits()));
    }
    // ⭐ A cópia fica seleccionada, como no ramo de MODELAGEM: é o que põe o gizmo em cima dela sem
    // ninguém a ter de procurar. ⚠️ **Já não é isto que a torna visível** — ver o doc do dreno.
    hero.gizmo.replace_selection(Some(copy.to_bits()));
    // ⚠️ **O toast diz o que a cópia É** (report do Enio, 2026-08-27). A 1.ª versão anunciava a
    // regra que a cópia herdava (*«it shows while its row is selected»*) — descrevia com exactidão
    // um objeto que desaparece, em vez de o não produzir. Hoje a cópia de uma receita é um objeto
    // comum, e o toast nomeia a diferença para o artista não a procurar na biblioteca.
    toasts.push(if recipe {
        Toast::success("Duplicated as a plain object — use Make Component for a second component")
    } else if inside {
        Toast::warning("Duplicated inside the component — it shows while the component is selected")
    } else {
        Toast::success("Duplicated entity")
    });
    true
}

/// ⚠️ Os gates do ROTEAMENTO são deste assunto, e vieram com ele do irmão.
#[cfg(test)]
#[path = "hierarchy_duplicate_routing_tests.rs"]
mod duplicate_routing;
