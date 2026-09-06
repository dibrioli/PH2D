//! ⭐ **ALCANÇABILIDADE: o painel oferece exatamente o que o gesto FAZ** (W34).
//!
//! # O defeito, e ele é meu
//!
//! A W31 ensinou o **tratador** a criar um grupo com uma forma sozinha — a resposta ao *"ainda não
//! temos como criar novos grupos"* do Enio (2026-08-22). Os gates dela passaram, verdes, e o gesto
//! **continuou inalcançável**: quem decide se os três botões de operação são pintados é
//! `field3d_scene::panel::ops_for`, e lá o caso *«uma forma sozinha»* devolvia lista vazia. A
//! fileira nunca aparecia. *A cura entrou pela metade e a prova não notou.*
//!
//! # ⚠️ Por que a prova não notou — a lei desta wave
//!
//! Todos os gates da W31 empurram a intenção por [`ph2d_panel_model3d::state::push_intent_for_test`]:
//!
//! ```text
//! push_intent(ApplyOp { slot }) → sync_scene → o documento mudou?  ✅ VERDE
//! ```
//!
//! ⛔ **Isso prova o TRATADOR, nunca a ALCANÇABILIDADE.** Empurrar a intenção é encenar um clique que
//! o artista não tem como dar — e um clique impossível passa em qualquer teste que o simule. É a
//! quinta reincidência da família da costura muda deste módulo (o modificador na escultura, a
//! multi-seleção, o olho, o cadeado, o reparentar), e desta vez o buraco estava no **gate**, não no
//! código.
//!
//! # A cura: uma lei, não um caso
//!
//! Acrescentar o caso que falta curaria hoje e nada mais. O que este arquivo prende é a **relação**:
//!
//! > ⭐ Para toda fileira e toda seleção, *«o painel publica a fileira»* tem de valer **exatamente**
//! > *«a intenção daquela fileira muda o documento»*.
//!
//! Ela apanha os dois lados: um botão que aparece e não faz nada (a affordance que mente) **e** um
//! gesto que funciona e ninguém alcança (o desta wave). E ela é lida do **retrato publicado**, não
//! das funções que o montam — um `ops_for` correto que ninguém ligasse ao `publish_snapshot`
//! continuaria vermelho, que é o que se quer de um gate de costura.
//!
//! ⭐ **A generalização pagou no mesmo dia.** Escrita para as operações, a lei apanhou um **segundo**
//! defeito da mesma família que ninguém procurava: com a peça inteira escolhida, a fileira
//! *Duplicar/Apagar* era pintada e os **dois** botões recusam a raiz por decisão escrita. *Uma lei
//! sobre uma fileira teria curado uma; sobre todas, encontrou a irmã.*
//!
//! # ⚠️ Quais fileiras entram
//!
//! Só as que **dependem da seleção**: operações, modificadores e ações. As formas (`adds`), os
//! verbos do gizmo e a exportação são ações sempre disponíveis — a de exportar nem sequer toca o
//! documento (ela anota um pedido), e medi-la por *«o documento mudou»* seria a pergunta errada.

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive};
use ph2d_panel_model3d::{ModelIntent, ModelSnapshot};

fn ball(x: f32) -> Node {
    Node {
        xform: ph2d_field::Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.2 }),
        mods: Vec::new(),
        verb: None,
    }
}

fn combine(op: Op, children: Vec<NodeId>) -> Node {
    Node {
        xform: ph2d_field::Xform::IDENTITY,
        kind: NodeKind::Combine { op, children },
        mods: Vec::new(),
        verb: None,
    }
}

/// Peça **plana**: uma união de duas esferas irmãs.
fn flat() -> FieldDoc {
    FieldDoc::new(
        vec![
            ball(0.0),
            ball(0.6),
            combine(Op::Union(Blend::Sharp), vec![NodeId(0), NodeId(1)]),
        ],
        NodeId(2),
    )
    .expect("a união")
}

/// Peça **aninhada**: `A ∪ (B − C)`. Ela dá dois casos que a plana não tem — folhas de **pais
/// diferentes** (que não se embrulham) e um **grupo que não é a raiz** (que se destaca).
fn nested() -> FieldDoc {
    FieldDoc::new(
        vec![
            ball(0.0),
            ball(0.6),
            ball(0.9),
            combine(Op::Difference(Blend::Sharp), vec![NodeId(1), NodeId(2)]),
            combine(Op::Union(Blend::Sharp), vec![NodeId(0), NodeId(3)]),
        ],
        NodeId(4),
    )
    .expect("o aninhado")
}

/// Monta a cena e devolve a raiz — pelo caminho real (`sync_scene`), não um mundo à mão.
fn scene(doc: &FieldDoc) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    crate::field3d_scene::sync_scene(&mut sim, Some(doc), 0.0);
    let world = sim.world_mut();
    let mut q = world.query::<(Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    (sim, root)
}

fn nodes_of(world: &bevy_ecs::world::World, root: Entity, leaf: bool) -> Vec<Entity> {
    ph2d_field_ecs::walk(world, root)
        .into_iter()
        .map(|(e, _)| e)
        .filter(|e| {
            let shape = world.get::<ph2d_field_ecs::FieldNode>(*e).map(|n| &n.shape);
            leaf == matches!(shape, Some(ph2d_field::NodeShape::Leaf(_))) && shape.is_some()
        })
        .collect()
}

fn parent_of(world: &bevy_ecs::world::World, e: Entity) -> Option<Entity> {
    world.get::<bevy_ecs::hierarchy::ChildOf>(e).map(|c| c.0)
}

/// Como cada caso da tabela se monta. Devolve a cena e **o que está selecionado**.
type Build = fn() -> (SimWorld, Vec<Entity>);

fn nothing() -> (SimWorld, Vec<Entity>) {
    let (sim, _root) = scene(&flat());
    (sim, Vec::new())
}

fn one_shape() -> (SimWorld, Vec<Entity>) {
    let (sim, root) = scene(&flat());
    let sel = vec![nodes_of(sim.world(), root, true)[0]];
    (sim, sel)
}

fn two_siblings() -> (SimWorld, Vec<Entity>) {
    let (sim, root) = scene(&flat());
    let l = nodes_of(sim.world(), root, true);
    (sim, vec![l[0], l[1]])
}

/// A **raiz** da peça — o caso que expôs o segundo defeito.
fn the_root() -> (SimWorld, Vec<Entity>) {
    let (sim, root) = scene(&flat());
    let group = nodes_of(sim.world(), root, false)[0];
    assert!(
        parent_of(sim.world(), group).is_none(),
        "a fixture plana tem UMA operação e ela é a raiz"
    );
    (sim, vec![group])
}

/// Um **grupo que não é a raiz** — o controle: ele destaca-se e troca de verbo como qualquer nó.
fn an_inner_group() -> (SimWorld, Vec<Entity>) {
    let (sim, root) = scene(&nested());
    let inner = *nodes_of(sim.world(), root, false)
        .iter()
        .find(|e| parent_of(sim.world(), **e).is_some())
        .expect("a fixture aninhada tem um grupo com pai");
    (sim, vec![inner])
}

fn two_strangers() -> (SimWorld, Vec<Entity>) {
    let (sim, root) = scene(&nested());
    let l = nodes_of(sim.world(), root, true);
    let a = l[0];
    let b = *l
        .iter()
        .find(|e| parent_of(sim.world(), **e) != parent_of(sim.world(), a))
        .expect("a fixture aninhada tem folhas de pais diferentes");
    (sim, vec![a, b])
}

/// Uma **fileira** de controles do painel: como se lê no retrato, e que intenção cada botão empurra.
struct Row {
    name: &'static str,
    read: fn(&ModelSnapshot) -> bool,
    intent: fn(usize) -> ModelIntent,
    /// Quantos botões varrer. ⚠️ **Varrer TODOS** e não só o primeiro: um deles pode ser o estado
    /// que o nó já tem — carregar em «União» numa união é um no-op legítimo, e mediria *«não faz
    /// nada»* num gesto que faz.
    slots: usize,
}

const ROWS: &[Row] = &[
    Row {
        name: "operações",
        read: |s| !s.ops.is_empty(),
        intent: |slot| ModelIntent::ApplyOp { slot },
        slots: 3,
    },
    // ⭐⭐⭐ **O VERBO DA FORMA** (W97). ⚠️ Ela entra nesta tabela e passa a ser varrida pelos seis
    // casos de seleção **de graça** — inclusive o que importa: com a **base** escolhida a fileira
    // não é oferecida *e* o gesto não faz nada, que é a lei da W34 a valer nos dois sentidos.
    Row {
        name: "verbo da forma",
        // ⚠️ Lê o `verb_subject` e não o `verbs`: é ele que o `paint` consulta para desenhar a
        // fileira, então um `verbs` cheio sem sujeito seria **oferecido** para este gate e
        // **invisível** para o artista.
        read: |s| s.verb_subject.is_some(),
        intent: |slot| ModelIntent::SetVerb { slot },
        // ⚠️ Derivado do `VERBS`, nunca um literal — a lição do `ACTS` logo abaixo.
        slots: super::verb::VERBS.len(),
    },
    // ⭐⭐⭐ **O CARÁTER da mistura** (W99). ⚠️ Ela é a **terceira** fileira desta tabela a existir
    // por causa do verbo por forma, e entra aqui pelo mesmo motivo: os seis casos de seleção
    // varrem-na de graça — incluindo o que importa, a **base**, onde não há junta que qualificar.
    Row {
        name: "carácter da mistura",
        read: |s| !s.characters.is_empty(),
        intent: |slot| ModelIntent::SetCharacter { slot },
        // ⚠️ Derivado do documento, nunca um literal.
        slots: ph2d_field::Character::ALL.len(),
    },
    Row {
        name: "modificadores",
        read: |s| !s.mods.is_empty(),
        intent: |slot| ModelIntent::ToggleMod { slot },
        // ⛔⛔ **ERA UM `4` ESCRITO À MÃO, e estava desactualizado havia quatro modificadores**
        // (2026-08-30): a fileira tinha **oito** naturezas e este gate varria as primeiras quatro —
        // o `MirrorZ`, o `Array`, o `Radial` e a inclinação **nunca** foram alcançados por ele.
        //
        // ⚠️ As três fileiras vizinhas derivam, e uma delas até escreve *«derivado do `ACTS`, nunca
        // um literal»* — *a lição estava escrita ao lado do defeito, e o defeito sobreviveu porque
        // ninguém a aplicou à linha de cima.*
        slots: ph2d_field::UnaryKind::ALL.len(),
    },
    Row {
        name: "ações",
        read: |s| !s.acts.is_empty(),
        intent: |slot| ModelIntent::Act { slot },
        // ⚠️ **Derivado do `ACTS`**, nunca um literal: a W38 acrescentou o *Isolate* no fim, e um
        // `2` escrito à mão deixaria o botão novo fora da varredura — verde a medir menos.
        slots: crate::field3d_scene::acts::ACTS.len(),
    },
];

/// ⭐ **A fileira pelo NOME** — e nunca por índice.
///
/// ⚠️ Isto é uma correcção, e o defeito era silencioso: as asserções abaixo endereçavam a tabela
/// pela POSIÇÃO, e inserir a fileira do verbo no meio re-apontou-as para outra fileira. Elas
/// continuaram a compilar e a correr — a medir a coisa errada. *Um índice para dentro de uma lista
/// que cresce é um endereço que muda de dono sem avisar.*
fn row(name: &str) -> &'static Row {
    ROWS.iter()
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("não há fileira «{name}» na tabela"))
}

/// ⭐ **O painel PUBLICA esta fileira nesta seleção?**
///
/// ⚠️ Lido do retrato que o painel de facto recebe ([`ph2d_panel_model3d::state::current`]): o que o
/// artista alcança é o que foi **publicado**, não o que uma função interna devolveu.
fn offered(row: &Row, build: Build) -> bool {
    let (mut sim, sel) = build();
    let _ = ph2d_panel_model3d::drain_intents();
    crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &sel,
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    (row.read)(&ph2d_panel_model3d::state::current())
}

/// ⭐ **Algum botão desta fileira MUDA o documento nesta seleção?**
///
/// ⚠️ Cada slot num mundo NOVO: os gestos mutam, e reutilizar a árvore mediria o segundo botão sobre
/// o que o primeiro deixou.
fn acts(row: &Row, build: Build) -> bool {
    (0..row.slots).any(|slot| {
        let (mut sim, sel) = build();
        let _ = ph2d_panel_model3d::drain_intents();
        let before = crate::field3d_scene::sync_scene(&mut sim, None, 0.0).expect("cozinha");
        ph2d_panel_model3d::state::push_intent_for_test((row.intent)(slot));
        let after = crate::field3d_scene::sync_scene_and_birth(
            &mut sim,
            None,
            &sel,
            0.0,
            &crate::field3d_scene::no_drawing(),
        )
        .0;
        // ⚠️ `None` conta como MUDANÇA: apagar o último nó deixa a peça sem cozimento, e isso é o
        // gesto a fazer alguma coisa — não a falhar.
        after.is_none_or(|a| a != before)
    })
}

/// As seleções que a lei atravessa.
const CASES: &[(&str, Build)] = &[
    ("nada selecionado", nothing as Build),
    ("uma FORMA sozinha", one_shape as Build),
    ("dois IRMÃOS", two_siblings as Build),
    ("a RAIZ da peça", the_root as Build),
    ("um GRUPO interno", an_inner_group as Build),
    ("duas formas de PAIS diferentes", two_strangers as Build),
];

/// ⭐ **O GATE-MÃE desta wave**: oferecer e fazer são a mesma pergunta, em toda fileira.
#[test]
fn the_panel_offers_exactly_what_the_gesture_does() {
    let mut bad = Vec::new();
    for row in ROWS {
        for (case, build) in CASES {
            let (o, a) = (offered(row, *build), acts(row, *build));
            if o != a {
                bad.push(format!(
                    "  {:<16} {case:<32}  oferecido={o:<5}  age={a}",
                    row.name
                ));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "o painel e o gesto discordam:\n{}\n\
         ⚠️ `oferecido=false age=true` é um gesto que existe e ninguém alcança (o defeito da W31).\n\
         ⚠️ `oferecido=true age=false` é um botão pintado e mudo (a affordance que mente).",
        bad.join("\n")
    );
}

/// ⚠️ **O que o produto quer que ACONTEÇA** — sem esta metade, a lei acima seria satisfeita por
/// *«nunca oferecer e nunca agir»*, que é a cura degenerada.
#[test]
fn the_gestures_the_product_promises_are_all_reachable() {
    // ⭐ O sintoma do Enio, 2026-08-22: *"ainda não temos como criar novos grupos"*.
    assert!(
        offered(row("operações"), one_shape),
        "com UMA forma escolhida a fileira de operações tem de aparecer — é o gesto de criar grupo"
    );
    assert!(
        offered(row("operações"), two_siblings),
        "dois irmãos embrulham-se"
    );
    assert!(
        offered(row("operações"), the_root),
        "a raiz é uma operação, e trocar-lhe o verbo é o gesto mais usado do módulo"
    );
    assert!(
        offered(row("ações"), an_inner_group),
        "um grupo interno duplica-se e apaga-se como qualquer nó"
    );
    assert!(
        offered(row("modificadores"), one_shape),
        "uma forma aceita casca e afastamento"
    );
}

/// ⚠️ **O controle da direção oposta**: as fileiras continuam a NÃO aparecer onde o gesto não pode
/// acontecer. Sem ele, a cura «publicar sempre» passaria no gate-mãe pela metade errada.
#[test]
fn the_rows_stay_silent_where_the_gesture_is_refused() {
    for row in ROWS {
        assert!(
            !offered(row, nothing),
            "sem seleção a fileira «{}» não é pintada",
            row.name
        );
    }
    assert!(
        !offered(row("operações"), two_strangers),
        "duas formas de pais diferentes não se embrulham — a fileira mentiria"
    );
    // ⭐ O segundo defeito, que a generalização da lei encontrou: `duplicate` e `remove` recusam a
    // raiz **por decisão escrita** (ela *é* a peça), e a fileira aparecia à mesma.
    assert!(
        !offered(row("ações"), the_root),
        "com a peça inteira escolhida, Duplicar e Apagar recusam os dois — a fileira não é pintada"
    );
}

// ───────── W47: a lei da alcançabilidade estendida à CÂMERA ─────────

/// Arma o módulo e devolve-o ao repouso — só possível desde a W42 (ver `field3d_view_tests`).
fn armed<R>(f: impl FnOnce() -> R) -> R {
    crate::field3d_smoke::set_armed_by_panel(true);
    let out = f();
    crate::field3d_smoke::set_armed_by_panel(false);
    let _ = crate::field3d_smoke::with_smoke(|_| ());
    out
}

/// ⭐ A costura dos chips de CÂMERA — ver [`camera`].
#[path = "field3d_reach_camera_tests.rs"]
mod camera;

/// ⭐⭐⭐ **TODA LINHA DE TODA FORMA TEM RÓTULO** — o censo que faltava (auditoria de 06/09).
///
/// # ⛔⛔ O buraco, e por que ele é pior do que parece
///
/// O `ph2d_i18n::tr` de uma chave desconhecida faz `leak_key`: ele **pinta o identificador cru** e
/// **vaza a string**. Num painel repintado a cada quadro, uma chave sem rótulo é um vazamento **por
/// quadro por linha** — e o que o artista vê é `field.dim.top_n1` em vez de *Top N1*.
///
/// ⚠️ **Nada gateava isto.** A W128 acrescentou **oito** chaves de uma vez; se uma tivesse ficado
/// por traduzir, a suíte inteira ficava verde e o defeito só aparecia no smoke — se alguém reparasse
/// na linha feia.
///
/// ⭐ A lista é **derivada** de `PrimitiveKind::ALL` × `dims()`, logo uma forma nova entra sozinha.
#[test]
fn every_row_of_every_shape_has_a_label() {
    let mut sem = Vec::new();
    for k in ph2d_field::PrimitiveKind::ALL {
        // ⭐ **A peça sai do próprio CATÁLOGO**, casando a chave do botão com a da família — nada
        // de uma segunda tabela de representantes que envelhece ao lado desta.
        let alvo = format!("panel.model3d.add.{}", k.key());
        let Some(slot) = crate::field3d_shapes::SHAPES
            .iter()
            .position(|s| s.key == alvo)
        else {
            continue;
        };
        let Some(p) = crate::field3d_shapes::shape_at(slot, 0.3) else {
            continue;
        };
        for d in ph2d_field::dims(&p) {
            // ⚠️ **A prova é a IDENTIDADE**: o `tr` devolve a própria chave quando não a conhece, e
            // é exactamente isso que se proíbe. *Comparar com uma lista de chaves escrita à mão
            // seria a segunda resposta à mesma pergunta.*
            if ph2d_i18n::tr(d.key) == d.key {
                sem.push(format!("{} · {}", k.key(), d.key));
            }
        }
    }
    assert!(
        sem.is_empty(),
        "{} linha(s) do painel pintam o IDENTIFICADOR CRU (e vazam uma string por quadro):\n  {}",
        sem.len(),
        sem.join("\n  ")
    );
}
