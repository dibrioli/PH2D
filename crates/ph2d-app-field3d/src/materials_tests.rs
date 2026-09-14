//! Os gates da tabela de materiais. Ver [`super`].

use super::{Table, surface_of};
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_ecs::FieldMaterial;

/// Duas esferas bem separadas, em união — cada uma tem uma região da tela que é só dela.
fn two_balls() -> FieldDoc {
    let leaf = |x: f32| Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.25 }),
        mods: Vec::new(),
        verb: None,
    };
    FieldDoc::new(
        vec![
            leaf(-0.4),
            leaf(0.4),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("duas esferas")
}

fn a_world() -> (ph2d_ecs::SimWorld, bevy_ecs::entity::Entity) {
    let mut sim = ph2d_ecs::SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &two_balls(), "peça");
    (sim, root)
}

fn leaves_of(
    world: &bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
) -> Vec<bevy_ecs::entity::Entity> {
    world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("a raiz tem filhos")
        .iter()
        .copied()
        .collect()
}

/// ⭐⭐⭐ **O DEFAULT do componente É o da nodedef** — e este gate é a ponte entre as duas crates.
///
/// ⛔⛔ **A `ph2d-field-ecs` NÃO depende da `ph2d-material`**, de propósito: o material é um
/// componente de cena e a lei do OpenPBR é uma crate sem dependências nenhumas. O preço dessa
/// fronteira é o `Default` estar escrito duas vezes — e é **este** gate que impede as duas cópias de
/// divergirem. *Uma constante escrita em dois sítios ainda não é uma constante.*
///
/// **Mutação que deve sangrar:** mexer num dos dois `Default`.
#[test]
fn the_default_material_is_the_one_the_nodedef_declares() {
    let nosso = surface_of(FieldMaterial::default());
    let nodedef = ph2d_material::OpenPbr::default().prepare();
    // A superfície não é comparável campo a campo de fora, então compara-se o que ela FAZ: a luz
    // que devolve. ⚠️ Três normais, para um acerto por acaso numa delas não passar.
    for n in [[0.0, 0.0, 1.0], [0.6, 0.0, 0.8], [0.0, 0.8, 0.6]] {
        let v = [0.0, 0.0, 1.0];
        let l = [0.3, 0.6, 0.74];
        assert_eq!(
            nosso.direct(n, v, l, [3.0; 3]),
            nodedef.direct(n, v, l, [3.0; 3]),
            "o `FieldMaterial::default()` deixou de ser o material da nodedef (normal {n:?})"
        );
    }
}

/// ⭐⭐⭐ **CADA FOLHA LEVA O SEU MATERIAL** — e a tabela sabe qual é qual.
///
/// ⚠️ **O ponto é posto sobre cada esfera**, que é como ele chega do traçado: no meio de uma união
/// qualquer das duas é plausível, e um gate que apontasse ali passaria com a resposta errada.
#[test]
fn each_leaf_wears_its_own_material() {
    let (mut sim, root) = a_world();
    let folhas = leaves_of(sim.world(), root);
    assert_eq!(folhas.len(), 2);
    // A da esquerda fica VERMELHA; a da direita continua no material de omissão.
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(1),
        0.0,
    )
    .expect("o verde");
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(2),
        0.0,
    )
    .expect("o azul");

    let t = Table::build(sim.world(), root, 0.8, 480.0);
    assert_eq!(t.surfaces.len(), 2, "uma superfície por folha");
    let owners = t.owners.as_ref().expect("duas folhas pedem um dono");
    // O pólo de cada esfera: só ela existe ali.
    assert_eq!(owners.at([-0.4, 0.0, 0.25]), Some(0), "a esquerda");
    assert_eq!(owners.at([0.4, 0.0, 0.25]), Some(1), "a direita");
    // E as duas superfícies devolvem luz DIFERENTE — é isso que o artista vê.
    let (n, v, l) = ([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.3, 0.6, 0.74]);
    assert_ne!(
        t.surfaces[0].direct(n, v, l, [3.0; 3]),
        t.surfaces[1].direct(n, v, l, [3.0; 3]),
        "as duas folhas têm materiais diferentes e devolvem a MESMA luz — a tabela não os separou"
    );
}

/// ⭐⭐ **Arrastar um número NÃO recompila a geometria** — é a razão de a tabela ter duas metades.
///
/// **Mutação que deve sangrar:** fazer o `refresh_authored` devolver `false` sempre (o slider de cor
/// deixa de ter efeito), ou `true` sempre (o quadro re-traça para sempre).
#[test]
fn changing_a_number_refreshes_the_surfaces_and_not_the_geometry() {
    let (mut sim, root) = a_world();
    let folhas = leaves_of(sim.world(), root);
    let mut t = Table::build(sim.world(), root, 0.8, 480.0);
    let antes = t.surfaces[0].direct([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.3, 0.6, 0.74], [3.0; 3]);

    assert!(
        !t.refresh_authored(sim.world(), root),
        "nada mudou e a tabela disse que sim — o quadro re-traçaria para sempre"
    );
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(3),
        0.9,
    )
    .expect("a rugosidade");
    assert!(
        t.refresh_authored(sim.world(), root),
        "a rugosidade mudou e a tabela não deu por isso — o slider fica sem efeito"
    );
    assert_ne!(
        t.surfaces[0].direct([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.3, 0.6, 0.74], [3.0; 3]),
        antes,
        "a superfície não seguiu o número"
    );
    // ⭐ **E a geometria não foi tocada:** o dono continua a responder o mesmo.
    assert_eq!(
        t.owners.as_ref().expect("dois donos").at([-0.4, 0.0, 0.25]),
        Some(0)
    );
}

/// Uma peça de UMA folha não constrói dono nenhum — não perguntar é o custo zero.
#[test]
fn a_single_leaf_asks_nobody_who_it_belongs_to() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.3 },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("uma esfera");
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    let t = Table::build(sim.world(), root, 0.8, 480.0);
    assert!(
        t.owners.is_none(),
        "uma peça de uma folha construiu um resolvedor de donos — é trabalho por uma pergunta que \
         não existe"
    );
    assert_eq!(t.surfaces.len(), 1);
}

/// ⭐⭐⭐ **ARRASTAR UMA COR NÃO COMPILA FITA NENHUMA** — o gate sobre o TRABALHO, não sobre a
/// resposta.
///
/// # ⛔⛔ Porque o gate irmão não chega
///
/// O [`changing_a_number_refreshes_the_surfaces_and_not_the_geometry`] afirma que o dono **responde o
/// mesmo** depois de um número mudar. Isso é a **RESPOSTA**, e ela sai certa mesmo que alguém troque
/// o `refresh_authored` por um `Table::build` inteiro: a geometria é a mesma, logo o dono responde o
/// mesmo — *e o quadro paga um JIT por folha a cada pixel de arrasto do slider de cor*.
///
/// É a mesma lei que o [`ph2d_field_eval::owners::Owners::at_counting`] existe para servir, escrita
/// no doc dele: **um gate sobre a RESPOSTA é cego ao PREÇO**, e uma optimização cuja ausência não se
/// vê na saída precisa de um gate sobre o trabalho.
///
/// # A régua
///
/// [`ph2d_field_eval::POINT_TAPES`] — o gémeo, escrito nesta wave, do contador que a W70 construiu
/// para exactamente esta família. ⚠️ **A 1.ª redacção deste gate leu o `FLOAT_TAPES`** — a fita do
/// **traçado**, e não a do **PONTO**, que é a que o `Field::new` compila — e mediu **zero de zero**.
/// Quem a apanhou foi o **piso**: *uma régua que lê zero nos dois lados é verde e não afirma nada.* Construir a tabela compila **uma fita por folha**; re-traduzir os números não
/// pode compilar **nenhuma**.
///
/// ⚠️ **O piso do lado caro é afirmado também**: sem ele, um `Table::build` que deixasse de compilar
/// (porque alguém lhe tirou os donos) leria `0` nos dois lados e o gate ficaria verde a medir nada.
///
/// ⚠️ **Corre por `nextest`, que dá um processo por teste** — o §9 do `docs/Render3d/05` mede o que
/// acontece a este contador sob `cargo test`: as threads vêem-se umas às outras e oito gates caem.
///
/// **Mutação que deve sangrar:** `refresh_authored` a delegar num `Table::build`.
#[test]
fn dragging_a_colour_compiles_no_tape_at_all() {
    use std::sync::atomic::Ordering;
    let (mut sim, root) = a_world();
    let folhas = leaves_of(sim.world(), root);

    // ── O lado CARO: construir a tabela compila uma fita por folha ──
    ph2d_field_eval::POINT_TAPES.store(0, Ordering::Relaxed);
    let mut t = Table::build(sim.world(), root, 0.8, 480.0);
    let compiladas = ph2d_field_eval::POINT_TAPES.load(Ordering::Relaxed);
    assert!(
        compiladas >= folhas.len(),
        "o piso: construir a tabela de {} folhas compilou {compiladas} fitas — se for zero, este \
         gate não tem lado caro e o barato não afirma nada",
        folhas.len()
    );

    // ── O lado BARATO: mexer num número do material não pode compilar nada ──
    ph2d_field_ecs::set_param(
        sim.world_mut(),
        folhas[0],
        ph2d_field::Param::Material(3),
        0.9,
    )
    .expect("a rugosidade");
    ph2d_field_eval::POINT_TAPES.store(0, Ordering::Relaxed);
    let mudou = t.refresh_authored(sim.world(), root);
    let depois = ph2d_field_eval::POINT_TAPES.load(Ordering::Relaxed);
    assert!(
        mudou,
        "o controlo: o número mudou e a tabela não deu por isso"
    );
    assert_eq!(
        depois, 0,
        "⛔ re-traduzir os números compilou {depois} fita(s) — é um JIT por folha a cada quadro de \
         um arrasto do selector de cor"
    );
}

/// ⏱️ **SONDA — o que CONSTRUIR a tabela custa por quadro** (a medição que o `docs/Render3d/05`
/// §11.6 encomendou por escrito: *«a primeira medição da wave seguinte»*).
///
/// # ⚠️ A pergunta, e porque a outra sonda não lhe responde
///
/// A [`pick_tests::measure_what_a_material_per_object_would_cost`] mediu a **RESOLUÇÃO** — *«de quem
/// é este pixel?»*, `1,6 ms` a 16 folhas — com as fitas **já compiladas**. Ela não mediu a
/// compilação, e o doc dela diz-o: *«medir sem ela mediria o JIT, não a pergunta»*.
///
/// ⛔ **Mas o produto paga o JIT.** O [`super::sync`] reconstrói a tabela sempre que o documento
/// muda, e o documento muda em **todo quadro de um arrasto do gizmo** — mover uma forma reescreve a
/// pose, o cozimento devolve outro `FieldDoc`, e o `mudou_o_doc` do `scene.rs` fica `true`. ⇒ o
/// número desta sonda é um **custo por quadro**, não um custo de arranque.
///
/// ⚠️ Corra-a com a máquina calma — ela imprime o `loadavg` ao lado, e acima de `~5` a leitura não
/// vale nada (`CLAUDE.md` §5.0).
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_building_the_table_costs_per_frame() {
    use std::time::Instant;

    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("load: {}", carga.split_whitespace().next().unwrap_or("?"));
    println!("folhas ·   build ·  leaves ·  Owners ·  % de um quadro de 16,7 ms");
    for k in [1usize, 2, 4, 8, 16, 32, 64] {
        let lado = (k as f32).sqrt().ceil() as usize;
        let passo = 0.9 / lado as f32;
        let mut nodes: Vec<Node> = (0..k)
            .map(|i| Node {
                xform: Xform::at(
                    ((i % lado) as f32 - (lado - 1) as f32 * 0.5) * passo,
                    ((i / lado) as f32 - (lado - 1) as f32 * 0.5) * passo,
                    0.0,
                ),
                kind: NodeKind::Leaf(Primitive::Sphere {
                    radius: passo * 0.45,
                }),
                mods: Vec::new(),
                verb: None,
            })
            .collect();
        nodes.push(Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: (0..k).map(|i| NodeId(i as u32)).collect(),
            },
            mods: Vec::new(),
            verb: None,
        });
        let doc = FieldDoc::new(nodes, NodeId(k as u32)).expect("a grelha de esferas");
        let mut sim = ph2d_ecs::SimWorld::new();
        let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
        let world = sim.world();

        // ⚠️ **A mediana de N, nunca uma leitura** — e um aquecimento antes, senão a 1.ª corrida
        // mede o `malloc` a crescer.
        let med = |f: &mut dyn FnMut()| -> f64 {
            f();
            let mut v: Vec<f64> = (0..9)
                .map(|_| {
                    let t = Instant::now();
                    f();
                    t.elapsed().as_secs_f64() * 1e3
                })
                .collect();
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        let build = med(&mut || {
            let _ = Table::build(world, root, 0.8, 480.0);
        });
        let folhas = med(&mut || {
            let _ = super::leaves(world, root);
        });
        let (_, placed) = super::leaves(world, root);
        let reg = crate::smoke::sampled_registry();
        let margem = ph2d_field_render::hit_tolerance(0.8, 480.0);
        let owners = med(&mut || {
            let _ = ph2d_field_eval::owners::Owners::new(&placed, &reg, margem);
        });
        println!(
            "{k:6} · {build:7.3} · {folhas:7.3} · {owners:7.3} · {:5.1} %",
            build / 16.7 * 100.0
        );
    }
}
