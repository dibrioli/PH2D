//! ⭐ **A LEI DA F3, num sítio só: o Inspector mostra o que o objeto TEM** (ADR-0166).
//!
//! Até esta fase, **todas** as seções abaixo eram publicadas para qualquer entidade com um
//! `Transform`, e o Inspector de um objeto vazio mostrava doze seções de zeros. *Ausência de
//! autoria não é «zeros»*, e a rota para cada feature era uma **face vazia** dentro da seção que
//! ela própria escondia.
//!
//! # Porque é UM ficheiro e não uma linha em cada builder
//!
//! Cada `build_*_info` sabe gatear-se; nenhum deles sabe que **existe uma lei**. Este ficheiro é a
//! lei: uma varredura em que cada linha afirma as **duas** metades sobre uma seção —
//!
//! | metade | o defeito que ela apanha |
//! |---|---|
//! | sem o componente ⇒ **sem seção** | a face vazia a voltar; doze seções de zeros num objeto novo |
//! | com o componente ⇒ **com seção** | a poda a ir longe demais: o artista anexa e **nada aparece** |
//!
//! ⚠️ **A segunda metade é a que mais custou.** Três seções acenderam-se com um componente que a
//! primeira redacção não listava — o `Collider` sozinho (uma PEÇA de um corpo ancestral), o
//! `AnchorMount` (quem MONTA numa âncora do pai, contra quem as OFERECE) e o `SpriteAnimator` (o
//! transporte, contra a biblioteca). Em cada caso a seção tem duas metades que pertencem a lados
//! opostos da mesma relação, e gatear numa só apagava a UI do outro lado.

use ph2d_ecs::{Entity, SimWorld, Transform, World};

/// Uma seção, e um componente que a ACENDE.
struct Case {
    section: &'static str,
    component: &'static str,
    /// O objeto BASE — o mínimo em que a seção poderia existir, e ainda não existe.
    base: fn(&mut World) -> Entity,
    /// Anexa o componente do caso.
    attach: fn(&mut World, Entity),
    /// A seção está viva?
    live: fn(&SimWorld, u64) -> bool,
}

fn plain(w: &mut World) -> Entity {
    w.spawn((Transform::IDENTITY, ph2d_ecs::Name::new("Object")))
        .id()
}

fn sprite(w: &mut World) -> Entity {
    w.spawn((
        Transform::IDENTITY,
        ph2d_ecs::Name::new("Image"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
    ))
    .id()
}

fn body(w: &mut World) -> Entity {
    w.spawn((
        Transform::IDENTITY,
        ph2d_ecs::Name::new("Body"),
        ph2d_physics_ecs::RigidBody::default(),
    ))
    .id()
}

fn ordering(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::ordering(sim.world(), b)
}
fn sampling(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::sampling(sim.world(), b)
}
fn blend(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::blend(sim.world(), b)
}
fn slice(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::slice(sim.world(), b)
}
fn visibility_section(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::visibility_section(sim.world(), b)
}
fn anchors(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::anchors(sim.world(), b)
}
fn anim(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::anim(sim.world(), b)
}
fn physics(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::physics(sim.world(), b)
}
fn player(sim: &SimWorld, b: u64) -> bool {
    crate::render_loop::inspector_presence_probe::player(sim, b)
}

// ⭐⭐ **A §7 Ordering SAIU desta tabela por decisão do Enio** (2026-08-30: *«essa é uma seção que
// por padrão deve começar com todos os objetos»*), e a lei dela vive no
// [`the_ordering_section_belongs_to_every_object`] logo abaixo.
//
// ⚠️ **Ela não é uma isenção da lei da F3 — é a leitura completa dela.** *«O Inspector mostra o que
// o objeto TEM»*, e todo objeto desenhável **tem** uma posição na ordem de desenho: ela existe sem
// componente nenhum, porque o contador de hierarquia a produz. Ausência de `ZIndexOverride` não é
// *«este objeto não tem Z»*, é *«o Z dele vem da árvore»* — que é o que o campo mostra com um `—`.
//
// ⛔ Mesma categoria da caixa «Visible», já declarada fora da tabela na nota da §8: a fronteira não
// é «opcional contra obrigatório», é **componente ANEXADO contra propriedade INTRÍNSECA**.
const CASES: &[Case] = &[
    Case {
        section: "§9 Sampling",
        component: "TextureFilter",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e)
                .insert(ph2d_ecs::TextureFilter(ph2d_ecs::FilterMode::Nearest));
        },
        live: sampling,
    },
    Case {
        section: "§9 Sampling",
        component: "UvTransform",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::UvTransform::default());
        },
        live: sampling,
    },
    Case {
        section: "§10 Blend",
        component: "BlendMode",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::BlendMode::default());
        },
        live: blend,
    },
    // ⚠️ **A §8 ESCAPOU à 1.ª poda e foi o smoke do Enio que a apanhou** — a lista de seções do
    // plano não a nomeava. ⛔ A caixa «Visible» NÃO está aqui de propósito: ela é chrome ao lado do
    // nome, tem snapshot próprio, e vale para todo objeto.
    Case {
        section: "§8 Visibility",
        component: "ClipChildren",
        base: plain,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::ClipChildren::default());
        },
        live: visibility_section,
    },
    Case {
        section: "§8 Visibility",
        component: "OnScreenEnabler",
        base: plain,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::OnScreenEnabler::default());
        },
        live: visibility_section,
    },
    Case {
        section: "§8 Visibility",
        component: "Mask2D",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::Mask2D::default());
        },
        live: visibility_section,
    },
    Case {
        section: "§5 9-Slice",
        component: "SliceNine",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::SliceNine::INERT);
        },
        live: slice,
    },
    Case {
        section: "§12 Anchors",
        component: "NamedAnchorList",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::NamedAnchorList::default());
        },
        live: anchors,
    },
    // ⚠️ O OUTRO lado da montagem — quem ANDA numa âncora do pai, e não tem lista nenhuma.
    Case {
        section: "§12 Anchors",
        component: "AnchorMount",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::AnchorMount::new("hand_r"));
        },
        live: anchors,
    },
    Case {
        section: "§11 Animation",
        component: "SpriteAnimations",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e)
                .insert(ph2d_ecs::SpriteAnimations::default());
        },
        live: anim,
    },
    // ⚠️ E o TRANSPORTE, que é a outra metade da §11.
    Case {
        section: "§11 Animation",
        component: "SpriteAnimator",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e).insert(ph2d_ecs::SpriteAnimator::default());
        },
        live: anim,
    },
    Case {
        section: "§11 Physics",
        component: "RigidBody",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e)
                .insert(ph2d_physics_ecs::RigidBody::default());
        },
        live: physics,
    },
    // ⚠️ Um `Collider` SEM corpo é uma PEÇA de um corpo ancestral, e a §11 tem uma face para ela.
    Case {
        section: "§11 Physics",
        component: "Collider",
        base: sprite,
        attach: |w, e| {
            w.entity_mut(e)
                .insert(ph2d_physics_ecs::Collider::default());
        },
        live: physics,
    },
    // ⚠️ **A §14 NÃO está nesta tabela, e a ausência é a lei** — ver
    // `the_player_section_belongs_to_every_dynamic_body`. Ela pertence a todo corpo `Dynamic`, com
    // ou sem o componente, exactamente como a §7 Ordering pertence a todo objecto.
];

/// ⭐ **A LEI, nos dois sentidos, sobre as treze combinações.**
///
/// (Mutação: apagar qualquer `if !has_… { return None }` de um `build_*_info` ⇒ a metade de
/// AUSÊNCIA reprova naquela seção, com o nome dela na mensagem.)
#[test]
fn a_section_appears_if_and_only_if_one_of_its_components_is_there() {
    for c in CASES {
        let mut sim = SimWorld::new();
        let e = (c.base)(sim.world_mut());
        let bits = e.to_bits();
        assert!(
            !(c.live)(&sim, bits),
            "{} apareceu SEM o {} — a face vazia voltou",
            c.section,
            c.component
        );
        (c.attach)(sim.world_mut(), e);
        assert!(
            (c.live)(&sim, bits),
            "{} nao apareceu COM o {} — o artista anexa e nada acontece",
            c.section,
            c.component
        );
    }
}

/// ⚠️ **E a BASE continua a existir.** A poda não pode comer o `Transform` nem o `Name`: um objeto
/// vazio tem de mostrar **duas** seções, e não zero — senão o Inspector fica em branco e o `+` não
/// tem onde estar.
#[test]
fn an_empty_object_still_shows_transform_and_name() {
    let mut sim = SimWorld::new();
    let e = plain(sim.world_mut());
    assert!(sim.world().get::<Transform>(e).is_some());
    assert!(sim.world().get::<ph2d_ecs::Name>(e).is_some());
    // E nenhuma das treze acima.
    for c in CASES {
        assert!(
            !(c.live)(&sim, e.to_bits()),
            "um objeto vazio mostrou {}",
            c.section
        );
    }
}

/// ⭐⭐ **A §14 Platform Player vale para TODO corpo DYNAMIC** (ordem do dono, 2026-09-14) — e as
/// **três** metades, porque aqui a cerca não é «tem o componente», é a FÍSICA.
///
/// *«Um objeto de física (Physics Body) e todas as opções aparecem com ele (inclusive Collision
/// Shape e Platform Player).»* ⇒ a paleta do `+` tem **uma** entrada de física e a porta do
/// comportamento volta a ser o *Make Platform Player* da face vazia desta secção.
///
/// ⛔ **Isto REVERTE, de propósito e só aqui, a poda da F3** (ADR-0166), que apagara esta face
/// porque a rota nova era o `+`. O que **não** volta é a doença que ela curou: um objecto pelado
/// continua sem secção de física nenhuma, e é a 3.ª metade abaixo que o afirma.
///
/// | metade | o defeito que ela apanha |
/// |---|---|
/// | corpo `Dynamic` **sem** o componente ⇒ **com** secção | a porta a desaparecer: o comportamento fica inalcançável, porque ele já não está na paleta |
/// | corpo `Static` sem o componente ⇒ **sem** secção | um botão que a física recusa em silêncio (a mola do player é um impulso, e um impulso não move massa infinita) |
/// | corpo `Static` **com** o componente ⇒ **com** secção | um componente presente e INVISÍVEL, que se lê como defeito — é o que acontece a um player que um bake pôs `Kinematic` |
///
/// ⚠️ **E a metade que fica na tabela acima é a do objecto pelado** (`an_empty_object_still_…`
/// varre `CASES`, e a §14 saiu dela) — por isso ela é afirmada aqui, explicitamente.
///
/// (Mutação: pôr `player_section_applies` de volta em `has_player` ⇒ a 1.ª metade reprova.)
#[test]
fn the_player_section_belongs_to_every_dynamic_body() {
    let mut sim = SimWorld::new();
    let dyn_body = body(sim.world_mut());
    assert!(
        player(&sim, dyn_body.to_bits()),
        "um corpo Dynamic ficou SEM a §14 — a porta *Make Platform Player* nao tem onde estar, e \
         desde 2026-09-14 o componente NAO esta' na paleta do `+`"
    );

    let mut sim = SimWorld::new();
    let stat = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_ecs::Name::new("Floor"),
            ph2d_physics_ecs::RigidBody {
                kind: ph2d_physics_ecs::BodyKind::Static,
            },
        ))
        .id();
    assert!(
        !player(&sim, stat.to_bits()),
        "um corpo Static mostrou a §14 — o botao ofereceria um gesto que a fisica recusa"
    );
    sim.world_mut()
        .entity_mut(stat)
        .insert(ph2d_physics_ecs::PlatformPlayer::default());
    assert!(
        player(&sim, stat.to_bits()),
        "um Static COM o componente ficou sem a §14 — um componente presente e invisivel le-se \
         como defeito, e e' o caso de um player que um bake pos Kinematic"
    );

    let mut sim = SimWorld::new();
    let nu = plain(sim.world_mut());
    assert!(
        !player(&sim, nu.to_bits()),
        "um objeto sem corpo nenhum mostrou a §14 — a poda da F3 continua a valer para ele"
    );
}

/// ⭐⭐ **A §11 alcança um FILHO de um corpo, ainda sem física nenhuma** (ordem do dono,
/// 2026-09-14) — a porta da PEÇA composta.
///
/// O `Collider` deixou de ser oferecido na paleta, e a rota que ele era — *esta forma é mais uma
/// peça do corpo ancestral* (W-Compound) — é o botão **Add Shape to X** da face vazia da §11.
/// ⛔ **A paleta genérica não sabe NOMEAR o dono**, e sem o nome o gesto não existe: um collider é
/// invisível e a hierarquia pode ter um grupo no meio. É a MESMA excepção do `rig_parts`, que já
/// vivia ali pela mesma razão (um gesto sobre a subárvore que a paleta não exprime).
///
/// ⚠️ **A metade de ausência é o irmão SEM pai**: um sprite solto continua sem §11. Sem ela, esta
/// lei seria indistinguível de *«a §11 voltou para toda a gente»*, que é a face vazia da F3 de volta.
///
/// (Mutação: tirar o `&& part_owner.is_empty()` do `build_physics_info` ⇒ a 2.ª metade reprova.)
#[test]
fn the_physics_section_reaches_a_child_of_a_body() {
    let mut sim = SimWorld::new();
    let dono = body(sim.world_mut());
    let filho = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            ph2d_ecs::Name::new("Shape"),
            ph2d_ecs::ChildOf(dono),
        ))
        .id();
    assert!(
        physics(&sim, filho.to_bits()),
        "um filho de um corpo ficou SEM a §11 — o *Add Shape to X* e' a unica porta da peca \
         composta, porque a paleta nao sabe nomear o dono"
    );

    let mut sim = SimWorld::new();
    let orfao = sprite(sim.world_mut());
    assert!(
        !physics(&sim, orfao.to_bits()),
        "um sprite SEM corpo acima mostrou a §11 — a face vazia da F3 voltou para toda a gente"
    );
}

/// ⭐⭐ **A §7 Ordering vale para TODO objeto** (Enio, 2026-08-30) — e as duas metades, como toda
/// linha da tabela acima.
///
/// **Mutação que deve sangrar:** repor o `if !has_any_ordering(…) { return None; }` no
/// `build_ordering_info` — que é literalmente o estado em que o smoke a apanhou.
#[test]
fn the_ordering_section_belongs_to_every_object() {
    // Metade 1 — um objeto NU, sem componente de ordenação nenhum, já tem a seção.
    let mut sim = SimWorld::new();
    let bare = plain(sim.world_mut());
    assert!(
        ordering(&sim, bare.to_bits()),
        "um objeto sem autoria de ordenacao ficou SEM a secao — o artista nao tem onde por o Z"
    );
    // ⚠️ E o que ela mostra é a AUSÊNCIA de autoria, não um zero fabricado: `z_index == None`
    // é o `—` que significa «vem da árvore».
    assert_eq!(
        crate::render_loop::inspector_presence_probe::ordering_z_index(sim.world(), bare.to_bits()),
        Some(None),
        "um objeto sem `ZIndexOverride` tem de mostrar `—`, nunca um zero fabricado"
    );

    // Metade 2 — anexar um override não faz a seção desaparecer, e o valor CHEGA.
    sim.world_mut()
        .entity_mut(bare)
        .insert(ph2d_ecs::ZIndexOverride(3));
    assert_eq!(
        crate::render_loop::inspector_presence_probe::ordering_z_index(sim.world(), bare.to_bits()),
        Some(Some(3)),
        "a secao sumiu ou o valor autorado nao chegou ao painel"
    );

    // ⛔ A cerca que fica: uma entidade SEM `Transform` continua fora do Inspector.
    let nowhere = sim.world_mut().spawn(ph2d_ecs::Name::new("Ghost")).id();
    assert!(
        !ordering(&sim, nowhere.to_bits()),
        "uma entidade sem Transform nao e' um objeto do Inspector"
    );
}
