//! ⭐⭐ **O QUE UM PASSO DE UNDO PARTILHA COM O ANTERIOR** (F8, 2026-09-02) — irmão por ASSUNTO do
//! [`super::tests`], que responde por *captura · restauro · canonicalização · a fila* e bateu no
//! tecto de 600 LOC do shell.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE, não por tamanho:** aqui a pergunta é sobre **memória** — o
//! que dois passos consecutivos guardam duas vezes —, e a régua é `Arc::ptr_eq`, que não aparece em
//! nenhum gate do irmão. *Igualdade e identidade são perguntas diferentes, e é por isso que elas
//! vivem em ficheiros diferentes.*

use super::ProjectState;
use ph2d_ecs::SimWorld;
use ph2d_ecs::scene::registry::{ComponentRegistry, register_ecs_components};
use ph2d_vec_scene::VecScene;

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    reg
}

/// Um rectângulo fechado — a forma mais barata que um documento tem.
fn rectangle(min: [f64; 2], max: [f64; 2]) -> ph2d_vec_scene::VecPath {
    ph2d_vec_scene::VecPath {
        verts: vec![
            ph2d_vec_scene::VecVertex::corner(min),
            ph2d_vec_scene::VecVertex::corner([max[0], min[1]]),
            ph2d_vec_scene::VecVertex::corner(max),
            ph2d_vec_scene::VecVertex::corner([min[0], max[1]]),
        ],
        closed: true,
        ..ph2d_vec_scene::VecPath::default()
    }
}

/// A porta do produto, com o passo ANTERIOR — é dele que a cena é reaproveitada.
fn capture_with_prev(
    sim: &mut SimWorld,
    vec: &VecScene,
    reg: &ComponentRegistry,
    prev: Option<&ProjectState>,
) -> ProjectState {
    ProjectState::capture(
        &ph2d_preview_drive::PreviewDrive::default(),
        sim,
        vec,
        &ph2d_flip::FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        prev,
    )
}

// ── ⭐⭐ A CENA É PARTILHADA ENTRE PASSOS (F8, 2026-09-02) ───────────────────────────────────────
//
// ⛔⛔ **Medido antes de mudar** (`ph2d-vec-scene/tests/it/measure_scene_clone.rs`): um passo clonava
// a cena INTEIRA — `236 KB` a 1 000 formas, `1,18 MB` a 5 000 — e a pilha guarda `UNDO_CAP` passos
// ⇒ **60 MB** e **303 MB** só de cópias da mesma cena. A esmagadora maioria dos passos não toca no
// documento vetorial (mover um objecto, renomear, pôr um componente), e para esses os dois passos
// consecutivos descrevem **a mesma cena**.
//
// ⚠️ É o argumento que o `WorldSnapshot` já tinha feito com `Arc` por linha (F2). Aqui o grão é o
// DOCUMENTO, e o resíduo está nomeado no gate de baixo.

/// ⭐⭐ **Dois passos seguidos sem edição vetorial partilham o MESMO ponteiro.**
///
/// ⚠️ **A régua é `Arc::ptr_eq`, e não a igualdade** — igualdade é o que já havia, e ela não diz
/// nada sobre memória. *É a diferença entre «os dois descrevem a mesma cena» e «os dois SÃO a
/// mesma cena».*
///
/// **Mutação que deve sangrar:** trocar o braço `Some(p) if …` por `_` no `capture` (isto é, voltar
/// a clonar sempre).
#[test]
fn two_steps_without_a_vector_edit_share_one_scene() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let mut scene = VecScene::new();
    scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    sim.world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::Name::new("a")));

    let a = capture_with_prev(&mut sim, &scene, &reg, None);
    // Uma edição do MUNDO, nenhuma do documento vetorial — o caso comum.
    sim.world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::Name::new("b")));
    let b = capture_with_prev(&mut sim, &scene, &reg, Some(&a));

    assert!(
        std::sync::Arc::ptr_eq(&a.vec, &b.vec),
        "a cena nao foi partilhada — cada passo volta a pagar o documento inteiro"
    );
    assert_ne!(
        a.world, b.world,
        "a fixtura tem de mudar o MUNDO, senao mede nada"
    );
}

/// ⛔ **E uma edição vetorial dá um ponteiro NOVO** — a partilha não pode sobreviver a uma mudança,
/// senão o passo anterior passava a descrever a cena de agora.
///
/// ⚠️ Este é o controlo do gate de cima: sem ele, um `capture` que devolvesse **sempre** o `Arc` do
/// anterior passaria o primeiro e destruiria o undo.
#[test]
fn a_vector_edit_gives_a_new_scene() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let mut scene = VecScene::new();
    scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let a = capture_with_prev(&mut sim, &scene, &reg, None);

    scene.push_path(rectangle([2.0, 2.0], [3.0, 3.0]));
    let b = capture_with_prev(&mut sim, &scene, &reg, Some(&a));

    assert!(
        !std::sync::Arc::ptr_eq(&a.vec, &b.vec),
        "a cena mudou e o passo anterior ficou a apontar para a de agora"
    );
    assert_eq!(
        a.vec.paths().len(),
        1,
        "o passo anterior tem de guardar a cena ANTIGA"
    );
    assert_eq!(b.vec.paths().len(), 2);
}

/// ⭐⭐⭐ **E o FORMATO não se mexe** — a serde com `rc` escreve um `Arc<T>` como o próprio `T`.
///
/// ⛔⛔ **É a metade que decide se esta mudança podia acontecer sem um degrau de
/// `PROJECT_SCHEMA`.** Sem ela, embrulhar um campo num `Arc` seria uma mudança de formato silenciosa
/// — todo ficheiro gravado passaria a ser lido errado, e o postcard é POSICIONAL, portanto sem erro
/// nenhum. A régua é os bytes de uma cena solta contra os bytes da mesma cena dentro do `Arc`.
#[test]
fn wrapping_the_scene_in_an_arc_does_not_move_a_byte_of_the_format() {
    let mut scene = VecScene::new();
    scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let bare = postcard::to_allocvec(&scene).expect("cena solta");
    let shared = postcard::to_allocvec(&std::sync::Arc::new(scene)).expect("cena partilhada");
    assert_eq!(
        bare, shared,
        "o `Arc` mudou os bytes — isto seria um degrau de PROJECT_SCHEMA por acidente"
    );
}

// ── ⭐⭐ O DOCUMENTO FLIP É PARTILHADO ENTRE PASSOS (F8, 2026-09-07) ────────────────────────────
//
// ⛔⛔ **Medido antes de mudar** (`ph2d-flip/tests/it/measure_doc_clone.rs`), e o número é PIOR que o
// da cena: um passo clonava o documento inteiro — `934 KB` a **24 quadros**, que é *um segundo de
// animação* — e a pilha guarda `UNDO_CAP` passos ⇒ **228 MB**. A `96` quadros são **912 MB**, e a
// pilha cheia chega a `1 GB` com **~108 quadros desenhados**.
//
// ⚠️ **A cena vetorial precisava de 5 000 formas para chegar aos `303 MB`; o Flip chega lá com
// quatro segundos de desenho.** *A mesma forma de defeito, num documento em que o artista atinge o
// limite fazendo o trabalho normal dele.*
//
// ⚠️ **O relógio ilibava os dois** (`0,044 ms` a 24 quadros, `0,3 %` de um quadro) — é a residência
// que condena. *Uma fase que só olhasse o relógio teria deixado o GB de pé.*

/// A porta do produto com o passo anterior, variando o **documento Flip** — a irmã da
/// [`capture_with_prev`], que varia a cena.
fn capture_flip_with_prev(
    sim: &mut SimWorld,
    flip: &ph2d_flip::FlipDoc,
    reg: &ComponentRegistry,
    prev: Option<&ProjectState>,
) -> ProjectState {
    ProjectState::capture(
        &ph2d_preview_drive::PreviewDrive::default(),
        sim,
        &VecScene::new(),
        flip,
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        prev,
    )
}

/// ⭐⭐ **Dois passos seguidos sem edição no Flip partilham o MESMO ponteiro.**
///
/// **Mutação que deve sangrar:** trocar o braço `Some(p) if …` por `_` no `capture` (isto é, voltar
/// a clonar sempre).
#[test]
fn two_steps_without_a_flip_edit_share_one_document() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let mut flip = ph2d_flip::FlipDoc::new();
    flip.push_object("Personagem");
    sim.world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::Name::new("a")));

    let a = capture_flip_with_prev(&mut sim, &flip, &reg, None);
    // Uma edição do MUNDO, nenhuma do documento Flip — o caso comum.
    sim.world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::Name::new("b")));
    let b = capture_flip_with_prev(&mut sim, &flip, &reg, Some(&a));

    assert!(
        std::sync::Arc::ptr_eq(&a.flip, &b.flip),
        "o documento Flip nao foi partilhado — cada passo volta a pagar a animacao inteira"
    );
    assert_ne!(
        a.world, b.world,
        "a fixtura tem de mudar o MUNDO, senao mede nada"
    );
}

/// ⛔ **E uma edição no Flip dá um ponteiro NOVO** — o controlo do gate de cima.
///
/// ⚠️ Sem ele, um `capture` que devolvesse **sempre** o `Arc` do anterior passaria o primeiro e
/// destruiria o undo do Flip em silêncio: cada passo passaria a descrever a animação de agora.
#[test]
fn a_flip_edit_gives_a_new_document() {
    let mut sim = SimWorld::new();
    let reg = registry();
    let mut flip = ph2d_flip::FlipDoc::new();
    flip.push_object("Personagem");
    let a = capture_flip_with_prev(&mut sim, &flip, &reg, None);

    flip.push_object("Fundo");
    let b = capture_flip_with_prev(&mut sim, &flip, &reg, Some(&a));

    assert!(
        !std::sync::Arc::ptr_eq(&a.flip, &b.flip),
        "a animacao mudou e o passo anterior ficou a apontar para a de agora"
    );
    assert_eq!(
        a.flip.objects().len(),
        1,
        "o passo anterior tem de guardar a animacao ANTIGA"
    );
    assert_eq!(b.flip.objects().len(), 2);
}

/// ⭐⭐⭐ **E o FORMATO não se mexe** — a irmã exacta do gate da cena, e pela mesma razão.
///
/// ⛔⛔ Sem ela, embrulhar este campo num `Arc` seria uma mudança de formato **silenciosa**: o
/// postcard é POSICIONAL, então todo `.ph2dproj` gravado passaria a ser lido errado sem um erro.
#[test]
fn wrapping_the_flip_doc_in_an_arc_does_not_move_a_byte_of_the_format() {
    let mut flip = ph2d_flip::FlipDoc::new();
    flip.push_object("Personagem");
    let bare = postcard::to_allocvec(&flip).expect("documento solto");
    let shared = postcard::to_allocvec(&std::sync::Arc::new(flip)).expect("documento partilhado");
    assert_eq!(
        bare, shared,
        "o `Arc` mudou os bytes — isto seria um degrau de PROJECT_SCHEMA por acidente"
    );
}
