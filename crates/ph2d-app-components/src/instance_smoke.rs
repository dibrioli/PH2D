//! `PH2D_INSTANCE_SMOKE=<n>` — as cenas PRONTAS-DE-VER do núcleo de instância
//! (ADR-0164 / plano F4).
//!
//! ⚠️ **Nada aqui pré-monta física.** As peças são entidades ECS com `RigidBody`/`Collider`/
//! `PhysicsJoint`, e quem as põe no solver é a ponte — se ela estivesse morta, os pêndulos
//! ficavam pendurados no ar em vez de balançar, que é a falha honesta.
//!
//! # Cena 1 — o ragdoll instanciado 3× (o smoke-gate 1 da F4)
//!
//! Um MESTRE lá em cima (que **não** se mexe — é receita, não objeto) e três instâncias dele em
//! baixo. Cada instância tem o pino DELA a prender os corpos DELA: os três balançam.
//!
//! ⛔ **O que o defeito parecia**, antes do remap da F4.2: as três juntas continuavam a nomear os
//! corpos do MESTRE — que não simulam —, então os braços caíam soltos no chão.
//!
//! # Cena 2 — a receita VETORIAL instanciada 3× (o instrumento que faltava, F4.6)
//!
//! ⛔⛔ **Ela nasce de um report que o gate não apanhou** (Enio, 2026-08-26: *«ao mudo o path, as
//! instâncias não mudaram»*): a F4.6b tem gates verdes e uma sonda headless que imita a ordem do
//! quadro — e no app não acontece. *Um subsistema sem cena de smoke própria recebe sempre o mesmo
//! report — «não funcionou» — sem o meio caminho.*
//!
//! ⚠️ **A receita fica LONGE das cópias, de propósito.** Se ela nascesse por cima, «editei o
//! mestre» e «editei a cópia que está em cima dele» seriam o mesmo gesto na tela — e a diferença
//! entre as duas é exactamente o que esta cena tem de decidir.
//!
//! ⚠️ Ela **imprime o diagnóstico do que montou** (ids de path por peça, e se o conteúdo bate).
//! Com `PH2D_INSTANCE_LOG=1` o passe imprime o dele a cada mudança — ver
//! [`crate::instance_diag`].

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, MasterRoot, Name, SimWorld, StableId, Transform, VecPathRef};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// Onde as três instâncias aterram, e onde a receita fica.
const INSTANCE_X: [f32; 3] = [-2.4, 0.0, 2.4];
const INSTANCE_Y: f32 = 1.2;
const MASTER_AT: Vec2 = Vec2::new(0.0, 3.4);
/// A distância do eixo ao braço — o que faz o pêndulo ter o que balançar.
const ARM: f32 = 0.9;

/// ⭐ **Monta o MESTRE**: eixo estático + braço dinâmico + o pino que os prende, tudo pendurado
/// numa raiz marcada [`MasterRoot`].
///
/// ⚠️ As referências do pino são escritas em `StableId` **diretamente**, e não pelo hash do nome:
/// esta cena vai ser copiada três vezes, e três braços chamados "Arm" tornariam a tradução por
/// nome ambígua. *A identidade é a chave; o nome é só o que o artista lê.*
pub fn spawn_master(sim: &mut SimWorld) -> ph2d_ecs::Entity {
    let root = sim
        .world_mut()
        .spawn((
            Transform::from_translation(MASTER_AT),
            Name::new("Ragdoll"),
            MasterRoot,
        ))
        .id();
    let hub = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Hub"),
            Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], [0.55, 0.57, 0.64, 1.0]),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.15,
                    half_y: 0.15,
                },
                ..Collider::default()
            },
            ChildOf(root),
        ))
        .id();
    let arm = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(ARM, 0.0)),
            Name::new("Arm"),
            Sprite::atlas(WHITE_TILE_KEY, [1.2, 0.24], [0.90, 0.55, 0.25, 1.0]),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.6,
                    half_y: 0.12,
                },
                ..Collider::default()
            },
            ChildOf(root),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let (a, b) = {
        let w = sim.world();
        (
            w.get::<StableId>(hub).expect("id do eixo").0,
            w.get::<StableId>(arm).expect("id do braco").0,
        )
    };
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Pin"),
        PhysicsJoint {
            body_a: a,
            body_b: b,
            ..PhysicsJoint::default()
        },
        ChildOf(root),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
    root
}

/// **Monta o mestre e instancia-o 3×** — o corpo da cena 1, partilhado com os gates.
pub(crate) fn spawn_ragdoll_scene(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> (ph2d_ecs::Entity, Vec<ph2d_ecs::Entity>) {
    let master = spawn_master(sim);
    let mut roots = Vec::new();
    for x in INSTANCE_X {
        let Ok(inst) = crate::instantiate::instantiate_master(
            sim,
            registry,
            master,
            None,
            docs,
            crate::instantiate::ArtLink::Own,
        ) else {
            continue;
        };
        sim.world_mut()
            .entity_mut(inst)
            .insert(Transform::from_translation(Vec2::new(x, INSTANCE_Y)));
        roots.push(inst);
    }
    (master, roots)
}

/// Onde a receita VETORIAL fica, e onde as três cópias dela aterram (cena 2).
///
/// ⚠️ **A receita LONGE das cópias** — ver o cabeçalho: sobrepostas, «editei o mestre» e «editei a
/// cópia por cima dele» são o mesmo gesto, e é essa distinção que a cena existe para permitir.
const VEC_MASTER_AT: Vec2 = Vec2::new(-4.2, 1.6);
const VEC_INSTANCE_X: [f32; 3] = [-0.6, 1.6, 3.8];
const VEC_INSTANCE_Y: f32 = 1.6;

/// ⭐ **Monta a receita vetorial**: uma raiz vazia com DUAS peças (caixa + etiqueta).
///
/// ⚠️ **Duas peças, e não uma:** uma receita de peça única não exercita a sub-árvore, e a
/// sub-árvore é metade do que a F4.6 tem de provar (cada peça leva o documento DELA).
///
/// ⚠️ **A entidade e o par `path ⟺ entidade` nascem aqui**, e não pelo `vec_entities::sync` do
/// quadro seguinte: a cena instancia no MESMO quadro, e uma peça sem entidade não entra na cópia
/// profunda.
fn spawn_vector_master(
    sim: &mut SimWorld,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> ph2d_ecs::Entity {
    use ph2d_vec_scene::{Paint, Rgba8, rectangle};
    let root = sim
        .world_mut()
        .spawn((
            Transform::from_translation(VEC_MASTER_AT),
            Name::new("Badge"),
            MasterRoot,
        ))
        .id();
    for (name, lo, hi, rgb) in [
        ("Box", [-0.9, -0.5], [0.9, 0.5], [58, 96, 168]),
        ("Label", [-0.6, -0.18], [0.6, 0.18], [232, 236, 245]),
    ] {
        let mut path = rectangle(lo, hi);
        path.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
        let id = docs.vec_scene.push_path(path);
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                Name::new(name),
                VecPathRef(id),
                ChildOf(root),
            ))
            .id();
        docs.vec_entities.insert(id, e.to_bits());
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    root
}

/// ⭐ **A cena 2 inteira**: a receita e as três cópias, espalhadas.
///
/// ⚠️ **Porta única, e é o que torna o gate um ORÁCULO em vez de um espelho.** A 1.ª versão tinha
/// o laço no corpo do smoke e o gate montava as cópias por conta própria — uma mutação que fazia a
/// cena instanciar **uma vez só** passava, porque o gate media os ingredientes, não a cena.
pub(crate) fn spawn_vector_scene(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> (ph2d_ecs::Entity, Vec<ph2d_ecs::Entity>) {
    let master = spawn_vector_master(sim, docs);
    let mut roots = Vec::new();
    // ⭐⭐ **A TERCEIRA cópia é LIGADA, e as duas primeiras não** (Enio, 2026-08-27) — é a única
    // forma de o smoke mostrar as DUAS leis: mexer na 1.ª muda só ela, mexer na 3.ª muda todas.
    // *Uma cena que só demonstra um dos modos deixa o outro por descobrir.*
    for (i, x) in VEC_INSTANCE_X.into_iter().enumerate() {
        let link = if i + 1 == VEC_INSTANCE_X.len() {
            crate::instantiate::ArtLink::Shared
        } else {
            crate::instantiate::ArtLink::Own
        };
        let Ok(inst) =
            crate::instantiate::instantiate_master(sim, registry, master, None, docs, link)
        else {
            continue;
        };
        sim.world_mut()
            .entity_mut(inst)
            .insert(Transform::from_translation(Vec2::new(x, VEC_INSTANCE_Y)));
        roots.push(inst);
    }
    (master, roots)
}

/// O `VecPathId` da peça `name` da subárvore de `root`, se ela tiver documento.
pub(crate) fn piece_path(sim: &SimWorld, root: ph2d_ecs::Entity, name: &str) -> Option<u64> {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return sim.world().get::<VecPathRef>(e).map(|v| v.0);
        }
        if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    None
}

/// Prólogo do quadro, uma vez. No-op sem a env.
pub fn instance_smoke(cx: &mut crate::scene_ctx::SceneCtx) {
    let Some(which) = std::env::var("PH2D_INSTANCE_SMOKE").ok() else {
        return;
    };
    match which.trim() {
        "1" => instance_smoke_ragdoll(cx),
        "2" => instance_smoke_vector(cx),
        // ⭐⭐⭐ **A receita DENTRO da receita** (F5 critério 4) — irmão por assunto, ver o
        // cabeçalho de lá.
        "3" => crate::instance_nested_smoke::instance_smoke_nested(cx),
        // ⭐⭐⭐ **A troca por um componente SEM PARENTESCO** (F5, o último critério) — irmã por
        // assunto, ver o cabeçalho de lá.
        "4" => crate::instance_replace_smoke::instance_smoke_replace(cx),
        // ⭐⭐⭐ **O que é SÓ desta cópia** (F5.10) — irmã por assunto, ver o cabeçalho de lá.
        "5" => crate::instance_removed_smoke::instance_smoke_removed(cx),
        // ⭐⭐⭐ **DAR uma peça ao componente** (F5.11) — o espelho da `=5`, e a montagem é a
        // MESMA função; ver o cabeçalho de lá.
        "6" => crate::instance_added_smoke::instance_smoke_added(cx),
        // ⭐⭐⭐ **MUDAR uma peça de lugar no componente** (F5.12) — o contrário das duas
        // acima: ali o que é só de uma cópia, aqui o que a receita manda em todas.
        "7" => crate::instance_move_smoke::instance_smoke_move(cx),
        other => {
            println!("[instance smoke] cena {other:?} nao existe (ha' a 1..7)")
        }
    }
    // ⚠️ **O relógio TEM de partir a andar**, e a linha vive no prólogo pela razão do smoke da
    // física: uma lista por-cena seria a enumeração de que a próxima cena nasce fora. Sem isto
    // os três pêndulos ficam pendurados no ar e o smoke lê-se como *"a física está morta"* —
    // que é precisamente o defeito que ele existe para distinguir.
    if let Some(hero) = cx.hero_screen.as_deref_mut() {
        hero.panel_visibility.insert("timeline", true);
    }
    cx.playhead.rewind();
    cx.playhead.play();
}

/// Cena 2 — ver o cabeçalho do módulo.
pub fn instance_smoke_vector(cx: &mut crate::scene_ctx::SceneCtx) {
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: cx.vec_scene,
        vec_entities: cx.vec_entities,
    };
    let (master, roots) = spawn_vector_scene(cx.sim, cx.registry, &mut docs);
    let master_bits = master.to_bits();
    // ⚠️ **O DIAGNÓSTICO do que ela montou, peça a peça.** Uma cópia sem `VecPathRef` é o modo
    // de falha que não deixa linha nenhuma em lado nenhum — e é o 1.º suspeito do §14.
    // ⚠️ **A receita NAO esta' na tela ate' alguem a escolher** — a marca `MasterEditing`
    // (F4.6) e' derivada da selecao, e ate' 2026-08-27 estas linhas descreviam coordenadas
    // vazias: o smoke que existe para dar o meio-caminho entregava «o mestre ficou invisivel»,
    // que e' exactamente o report que ele existe para evitar.
    // ⛔⛔⛔ **O PASSO 1 que aqui esteve tornou-se IMPOSSÍVEL, e ninguém o notou** (medido em
    // 2026-09-06). Ele mandava *«clique na linha 'Badge'»* — verdade quando foi escrito
    // (27/08) e falso desde 30/08, quando a Hierarquia passou a **retirar da lista** tudo o
    // que o `off_canvas::is_unedited_recipe` acusa: o `MasterRoot` também é `MasterPiece`,
    // logo a receita inteira sai. *É a lei do §5.0 à letra — quando um comportamento muda, a
    // cena que o demonstra é o último sítio a ser lembrado e o primeiro que o dono lê.*
    //
    // ⇒ a cena **abre a receita ela própria** (a marca é derivada da selecção), e o texto passa
    // a dizer o estado em vez de mandar um gesto que não existe.
    println!(
        "[instance smoke 2] o componente ja' esta' ABERTO (a linha 'Badge' nasce acesa) — e' \
         por isso que o cracha' da RECEITA aparece a' ESQUERDA das tres copias"
    );
    println!(
        "[instance smoke 2] (feche-a escolhendo outra coisa: uma receita e' a biblioteca, e so' \
         se ve' enquanto a linha dela esta' escolhida)"
    );
    for name in ["Box", "Label"] {
        let m = piece_path(cx.sim, master, name);
        let copies: Vec<String> = roots
            .iter()
            .map(|&r| {
                piece_path(cx.sim, r, name).map_or_else(
                    || "SEM GEOMETRIA".to_string(),
                    |id| {
                        let same = m
                            .and_then(|mid| cx.vec_scene.path(mid))
                            .zip(cx.vec_scene.path(id))
                            .is_some_and(|(a, b)| a.verts == b.verts);
                        format!("path {id}{}", if same { "" } else { " (FORMA DIFERENTE)" })
                    },
                )
            })
            .collect();
        println!(
            "[instance smoke 2] peca {name:?}: receita = path {:?} · copias = {}",
            m,
            copies.join(" · ")
        );
    }
    println!(
        "[instance smoke 2] PASSO 2: escolha 'Badge > Box' (a RECEITA, a' esquerda) e mova um \
         no' dela: as TRES copias tem de mudar junto"
    );
    // ⭐⭐⭐ **As DUAS leis lado a lado** (Enio, 2026-08-27) — é para isto que a 3.ª cópia nasce
    // LIGADA. Sem estas duas linhas o artista tem o modo novo na cena e nenhuma forma de saber
    // que ele existe, que é o defeito §1.7 outra vez, um nível acima.
    println!(
        "[instance smoke 2] PASSO 3: mova um no' da 1a copia (a mais a' esquerda das tres) — \
         so' ELA muda. E' a copia com desenho PROPRIO."
    );
    println!(
        "[instance smoke 2] PASSO 4: mova um no' da 3a copia (a mais a' direita) — a RECEITA e \
         as outras duas mudam junto. Essa e' LIGADA (o 'Instantiate Linked')."
    );
    println!(
        "[instance smoke 2] se nao mudarem, rode outra vez com PH2D_INSTANCE_LOG=1 — o passe \
         diz em que pergunta ele parou"
    );
    // ⚠️ **Depois dos `println!`, e não antes** — o `gfx` está emprestado ao `docs` até aqui.
    if let Some(hero) = cx.hero_screen.as_deref_mut() {
        hero.gizmo.replace_selection(Some(master_bits));
    }
}

/// Cena 1 — ver o cabeçalho do módulo.
pub fn instance_smoke_ragdoll(cx: &mut crate::scene_ctx::SceneCtx) {
    // O chão da cena — o MESMO que as cenas de física põem (`ph2d_app_physics::common::spawn_floor`),
    // escrito aqui porque uma família não chama outra (auditoria de arquitectura A1, 2026-09-12). É
    // conteúdo de uma cena de demonstração, não uma lei.
    cx.sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, -1.0)),
        Sprite::atlas(WHITE_TILE_KEY, [8.0, 0.4], [0.40, 0.42, 0.48, 1.0]),
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 4.0,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
    ));
    // Campos DISJUNTOS do `AppGfx` (+ o mapa, que é do `App`) — empréstimos separados, sem
    // clonar o registo nem o documento.
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: cx.vec_scene,
        vec_entities: cx.vec_entities,
    };
    let (master, roots) = spawn_ragdoll_scene(cx.sim, cx.registry, &mut docs);
    let master_bits = master.to_bits();
    // ⚠️ A cena **imprime o que montou** — se estas linhas não aparecerem, PARE: o que está
    // na tela não é o que este smoke descreve.
    // ⛔⛔⛔ **O PASSO 1 que aqui esteve era IMPOSSÍVEL desde 2026-08-30** — ver a nota da
    // cena 2: a Hierarquia retira da lista a receita inteira, e ele mandava clicar na linha
    // dela. A cena abre-a agora, e o texto diz o estado.
    println!(
        "[instance smoke 1] o componente ja' esta' ABERTO (a linha 'Ragdoll' nasce acesa) — e' \
         por isso que a RECEITA aparece la' em cima (ela NAO se mexe)"
    );
    for (i, r) in roots.iter().enumerate() {
        let name = cx
            .sim
            .world()
            .get::<Name>(*r)
            .map(|n| n.0.clone())
            .unwrap_or_default();
        println!(
            "[instance smoke 1] instancia {} = {name:?} em x = {}",
            i + 1,
            INSTANCE_X[i]
        );
    }
    println!(
        "[instance smoke 1] os {} bracos tem de BALANCAR cada um no eixo dele; \
         braco no chao = a junta prendeu no mestre",
        roots.len()
    );
    // ⭐ **A segunda metade do smoke é o SYNC** (F4.3) — e ela precisa de um gesto, então a
    // cena diz qual. Sem esta linha o artista vê três pêndulos e não descobre sozinho que
    // editar a receita muda os três.
    println!(
        "[instance smoke 1] PASSO 2: escolha 'Ragdoll > Arm' (o de CIMA, a receita) e mude a \
         cor em 'Color & Tint': os tres bracos de baixo mudam com ele"
    );
    // ⭐ E a terceira metade é o OVERRIDE (F4.4): a excepção que o artista faz numa cópia tem
    // de sobreviver à edição seguinte da receita. Sem a instrução, ele nunca a descobre.
    println!(
        "[instance smoke 1] e a EXCEPCAO: pinte o 'Arm' de UMA das copias de baixo, depois \
         pinte o da receita outra vez — a que voce tocou fica com a cor dela"
    );
    println!(
        "[instance smoke 1] para desfazer a excepcao: botao direito na linha da copia -> \
         'Revert to Master'"
    );
    // ⚠️ **Depois dos `println!`, e não antes** — o `gfx` está emprestado ao `docs` até aqui.
    if let Some(hero) = cx.hero_screen.as_deref_mut() {
        hero.gizmo.replace_selection(Some(master_bits));
    }
}
