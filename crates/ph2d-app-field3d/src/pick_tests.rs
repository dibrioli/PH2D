//! Os gates da seleção por clique.
//!
//! ⚠️ **Eles apontam para um sítio ONDE SE SABE quem está** — não para um pixel qualquer da peça. Um
//! gate que clicasse no meio de uma união e afirmasse *"deu um objeto"* passaria com a resposta
//! errada, porque ali qualquer um dos dois é plausível. Aqui aponta-se a **ponta** de cada cilindro,
//! onde só ele existe, e a resposta certa é uma só.

use super::node_under;
use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_render::{Orbit, Screen};

const W: u32 = 400;
const H: u32 = 320;

/// Três esferas bem separadas: cada uma tem uma região da tela que é só dela.
fn three_spheres() -> FieldDoc {
    let leaf = |x: f32, y: f32| Node {
        xform: Xform::at(x, y, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.15 }),
        mods: Vec::new(),
        verb: None,
    };
    FieldDoc::new(
        vec![
            leaf(-0.45, 0.0),
            leaf(0.0, 0.0),
            leaf(0.45, 0.0),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1), NodeId(2)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(3),
    )
    .expect("três esferas")
}

/// Vista de frente, para o `x` do mundo cair direto no `x` da tela.
fn front() -> Orbit {
    Orbit::from_yaw_pitch(0.0, 0.0)
}

/// ⭐ **Clicar numa esfera devolve AQUELA esfera** — as três, uma a uma.
#[test]
fn clicking_a_shape_returns_that_shape() {
    let mut sim = SimWorld::new();
    let doc = three_spheres();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
    let kids: Vec<Entity> = world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .collect();
    assert_eq!(kids.len(), 3);

    let c = front();
    let s = Screen::new(W, H, c.half_extent);
    for (k, centre) in [(-0.45f32), 0.0, 0.45].into_iter().enumerate() {
        let (px, _) = c
            .project([centre, 0.0, 0.0], s)
            .expect("a fixture olha a peça");
        assert_eq!(
            node_under(sim.world(), root, &doc, &c, s, px),
            Some(kids[k]),
            "o centro da esfera {k} tem de ser dela"
        );
    }
}

/// **Clicar no fundo não devolve nada** — e é o que faz um clique no vazio limpar a seleção em vez
/// de escolher o objeto mais próximo.
#[test]
fn clicking_the_background_returns_nothing() {
    let mut sim = SimWorld::new();
    let doc = three_spheres();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
    let c = front();
    let s = Screen::new(W, H, c.half_extent);
    assert_eq!(node_under(sim.world(), root, &doc, &c, s, [2.0, 2.0]), None);
}

/// ⭐ **A resposta usa a pose de MUNDO**, e o gate prova-o com um grupo deslocado.
///
/// ⚠️ Avaliar cada folha com a pose **local** dela responderia sobre um sítio onde ela não está. Numa
/// peça plana (grupo na identidade) as duas contas dão o mesmo, e o defeito ficaria escondido até
/// alguém agrupar e mover — aí o clique passaria a escolher o vizinho.
#[test]
fn the_answer_uses_the_world_pose_not_the_local_one() {
    let mut sim = SimWorld::new();
    let doc = {
        let leaf = |x: f32| Node {
            xform: Xform::at(x, 0.0, 0.0),
            kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.12 }),
            mods: Vec::new(),
            verb: None,
        };
        FieldDoc::new(
            vec![
                leaf(-0.3),
                leaf(0.3),
                Node {
                    // O grupo inteiro anda para a direita e para cima.
                    xform: Xform::at(0.35, 0.25, 0.0),
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
        .expect("duas esferas num grupo deslocado")
    };
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
    let kids: Vec<Entity> = world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .collect();

    let c = front();
    let s = Screen::new(W, H, c.half_extent);
    // A esfera da ESQUERDA está, no mundo, em (0.05, 0.25) — e é ali que se clica.
    let (px, _) = c
        .project([0.05, 0.25, 0.0], s)
        .expect("a fixture olha a peça");
    assert_eq!(
        node_under(sim.world(), root, &doc, &c, s, px),
        Some(kids[0]),
        "o clique caiu na esfera da esquerda vista no MUNDO"
    );
}

/// ⚠️ **Quanto custa um clique**, medido — não afirmado.
///
/// A rota escolhida compila **uma árvore por folha**, uma vez por clique. O número abaixo é o que
/// diz se essa rota se aguenta ou se ela precisa de cache; ele está no doc 07 ao lado da alternativa
/// que foi recusada (o *id-buffer*, que espalharia o custo por cada pixel de cada quadro).
#[test]
#[ignore = "medição, não gate — corre com --ignored"]
fn measure_pick_cost() {
    let mut sim = SimWorld::new();
    let doc = three_spheres();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
    let c = front();
    let s = Screen::new(1600, 1000, c.half_extent);
    let (px, _) = c
        .project([0.0, 0.0, 0.0], s)
        .expect("a fixture olha a peça");

    let t0 = std::time::Instant::now();
    const N: u32 = 20;
    for _ in 0..N {
        let _ = node_under(sim.world(), root, &doc, &c, s, px);
    }
    let each = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(N);
    println!("[pick] {each:.2} ms por clique — 3 folhas, quadro de 1600x1000");
}

/// ⏱️ **SONDA (`--ignored`): quanto custa perguntar «de quem é este pixel» para a PEÇA INTEIRA.**
///
/// # ⛔⛔ Porquê agora: uma recusa medida mudou de premissa
///
/// O cabeçalho de [`super`] recusa o *id-buffer* — *«o custo espalhado por cada pixel de cada quadro
/// para responder a uma pergunta que só se faz **num clique**»*. ⚠️ **Material por objecto faz dela
/// uma pergunta POR PIXEL**, e o `CLAUDE.md` §0.0 é explícito: *quem move o número que tornava algo
/// inalcançável tem de reconferir a nota*.
///
/// ⇒ esta sonda mede a rota que a recusa **não** cobria: resolver o dono **só no ponto final**, uma
/// vez por pixel de peça, em vez de arrastar um segundo canal por cada passo da marcha.
///
/// # As três colunas, e o que cada uma vale
///
/// | coluna | o que é | quanto dela é custo NOVO |
/// |---|---|---|
/// | `traçado` | a marcha que o quadro já paga | zero — já acontece |
/// | `pontos` | uma 2.ª marcha só para saber ONDE cada pixel bateu | zero **se o G-buffer guardar o ponto**, que ele hoje DEITA FORA (`march` devolve-o e o `trace` ignora-o) |
/// | `donos` | avaliar cada folha no ponto e ficar com a de menor módulo | **este é o preço da feature** |
#[test]
#[ignore = "sonda de medição"]
fn measure_what_a_material_per_object_would_cost() {
    use ph2d_field_render::{Lens, surfaces_under};
    use std::time::Instant;

    let (w, h) = (640u32, 360u32);
    // ⚠️ **Ortográfica e enquadrada**, para a contagem de pixels de peça não depender da lente.
    let cam = Orbit {
        lens: Lens::Ortho,
        ..front()
    };
    let screen = Screen::new(w, h, cam.half_extent);
    let reg = crate::smoke::sampled_registry();

    println!(
        "folhas ·  peça px ·  traçado ·   pontos ·    donos ·  por pixel ·  c/ caixa ·  visitadas"
    );
    for k in [1usize, 2, 4, 8, 16] {
        // `k` esferas numa grelha, todas dentro do enquadramento e em união.
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

        let t = Instant::now();
        let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
        let tracado = t.elapsed().as_secs_f64() * 1e3;

        let pixels: Vec<[f32; 2]> = (0..(w as usize * h as usize))
            .filter(|i| g.hit[*i])
            .map(|i| [(i % w as usize) as f32 + 0.5, (i / w as usize) as f32 + 0.5])
            .collect();
        let t = Instant::now();
        let pontos = surfaces_under(&doc, &reg, &cam, screen, &pixels);
        let ms_pontos = t.elapsed().as_secs_f64() * 1e3;

        // ⚠️ **Uma fita por folha, compilada UMA vez** — é a lei que o `owners_under` já paga, e
        // medir sem ela mediria o JIT, não a pergunta.
        let folhas: Vec<ph2d_field_eval::Field> = (0..k)
            .map(|i| {
                let placed = FieldDoc::new(vec![doc.nodes()[i].clone()], NodeId(0))
                    .expect("a folha sozinha");
                ph2d_field_eval::Field::new(&placed)
            })
            .collect();
        let t = Instant::now();
        let mut donos = 0usize;
        for p in pontos.iter().flatten() {
            let mut melhor = (f32::INFINITY, 0usize);
            for (n, f) in folhas.iter().enumerate() {
                let v = f
                    .at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
                    .abs() as f32;
                if v < melhor.0 {
                    melhor = (v, n);
                }
            }
            donos += melhor.1;
        }
        let ms_donos = t.elapsed().as_secs_f64() * 1e3;
        assert!(
            donos < usize::MAX,
            "o laço não pode ser optimizado para fora"
        );

        // ⭐⭐⭐ **A MESMA resposta com a CAIXA à frente** — a bola de cada folha já é derivada pela
        // casa (`bounds::bounding_ball`), e um ponto fora dela não pode ser o dono: numa união o
        // vencedor vale ~0, logo ele está SOBRE a superfície da própria folha.
        //
        // ⚠️ **Com uma mistura a superfície sai para FORA das bolas das folhas** (o `fold_children`
        // diz-o por escrito: uma união suave empurra o vinco), então a rede tem de existir — aqui
        // ela é contada, não escondida.
        let bolas: Vec<ph2d_field_eval::bounds::Ball> = (0..k)
            .map(|i| {
                let placed = FieldDoc::new(vec![doc.nodes()[i].clone()], NodeId(0))
                    .expect("a folha sozinha");
                ph2d_field_eval::bounds::bounding_ball(&placed, &reg).expect("a bola da folha")
            })
            .collect();
        let t = Instant::now();
        let (mut donos2, mut visitadas, mut redes) = (0usize, 0usize, 0usize);
        for p in pontos.iter().flatten() {
            // ⚠️⚠️ **A MARGEM não é folga, é OBRIGATÓRIA — e a 1.ª redacção desta sonda não a
            // tinha, e a rede disparou em 26 216 de 26 216 pixels.** A marcha pára quando o campo
            // desce abaixo de uma tolerância, ou seja **ligeiramente FORA** da superfície: o ponto
            // está a um epsilon da bola, sempre, e `d² <= r²` reprova em todo o lado. *Um filtro
            // exacto sobre um ponto que é aproximado por construção rejeita a resposta certa.*
            //
            // ⚠️ Aqui ela é derivada do ENQUADRAMENTO (a tolerância da marcha escala com ele); numa
            // implementação a sério ela sai da `Sharpness`, que é quem a escolhe.
            let margem = 0.01 * cam.half_extent;
            let dentro = |b: &ph2d_field_eval::bounds::Ball| {
                let d = [p[0] - b.center[0], p[1] - b.center[1], p[2] - b.center[2]];
                let r = b.radius + margem;
                d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= r * r
            };
            let cabe = folhas.len() > 1 && bolas.iter().any(dentro);
            if !cabe {
                redes += 1;
            }
            let mut melhor = (f32::INFINITY, 0usize);
            for (n, f) in folhas.iter().enumerate() {
                if cabe && !dentro(&bolas[n]) {
                    continue;
                }
                visitadas += 1;
                let v = f
                    .at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
                    .abs() as f32;
                if v < melhor.0 {
                    melhor = (v, n);
                }
            }
            donos2 += melhor.1;
        }
        let ms_caixa = t.elapsed().as_secs_f64() * 1e3;
        assert_eq!(
            donos2, donos,
            "a caixa à frente mudou a RESPOSTA, não só o relógio"
        );

        let n = pixels.len().max(1);
        println!(
            "{k:6} · {n:8} · {tracado:6.1} ms · {ms_pontos:6.1} ms · {ms_donos:6.1} ms · \
             {:6.0} ns · {ms_caixa:6.1} ms · {:4.1} folhas/px · rede {redes}",
            ms_donos * 1e6 / n as f64,
            visitadas as f64 / n as f64
        );
    }
}
