//! **A cena pronta para o smoke de A1/A2** (`PH2D_MOTION_OBJ_SMOKE`, doc 86 §7):
//! um OBJETO da engine, trazido para o grafo e **carimbado numa grade**.
//!
//! Até A o sistema de nós só sabia produzir ONDE (posições); nunca O QUE. Esta
//! cena monta as duas metades da resposta:
//!
//! 1. **o objeto** chamado **`Object`** na Hierarchy (o nome é a referência
//!    inteira, doc 86 §2) — um **sprite** (`=1`, A1), uma **forma vetorial**
//!    (`=2`, A2: a estrela é rasterizada numa tile pela membrana), um **objeto
//!    Flip** (`=3`, A3: as camadas do Flip são compostas no frame atual e assadas
//!    numa tile pela membrana) ou um **grupo de mídia mista** (`=4`, A4: o grupo
//!    emite os filhos como N instâncias vivas no seu layout relativo);
//! 2. **o grafo**: `source.object → motion.duplicator ← motion.grid → output` —
//!    o MESMO grafo nos dois modos (o `source.object` é media-agnóstico).
//!
//! O que se vê: a arte do objeto carimbada em cada ponto da grade — a MESMA
//! figura repetida, não um quad chapado. É a "conexão (renderização) com os
//! objetos da game engine" que o Enio pediu, ponta a ponta.
//!
//! **E o campo `Object` é um PICKER** (não uma caixa de texto): o nó nasce
//! SELECIONADO. Renomeie o objeto na Hierarchy e o nó para de achá-lo — o nome
//! É a referência. No modo vetor, **edite a forma** e as cópias re-assam (LIVE).

use ph2d_ecs::{Name, Transform};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};
use ph2d_render::Sprite;

// A cena das DUAS FASES (`=7`, o `time_offset` do doc 89 folha 14) mora num irmão:
// ela é um ASSUNTO — *o mesmo objeto em dois tempos* — e traz a própria fixture
// animada, que nenhum dos outros modos precisa. Cortado pelo teto de LOC da shell.
/// O modo `=8` — a POSE do objeto (doc 89 folha 14), irmão pelo corte que o `=7` já fez.
#[path = "motion_object_smoke_pose.rs"]
mod pose;

/// **A ARTE E AS ENTIDADES** que os modos partilham — o quarto corte por responsabilidade
/// (este ficheiro diz *qual modo liga o quê*; o irmão diz *o que existe para ser ligado*).
#[path = "motion_object_smoke_art.rs"]
mod art;

/// ⭐ A cena `=12` — a FOLHA à frente dos galhos, com uma folha DESENHADA (a única media que
/// pode ficar acima do vector; ver o cabeçalho do módulo).
#[path = "motion_object_smoke_leaf.rs"]
mod leaf;
use art::{
    child_at, find_group, flip_rect, name_vector_entity, name_vector_entity_as, spawn_flip_object,
    spawn_flip_object_named, spawn_sprite, star_shape,
};

#[path = "motion_object_smoke_times.rs"]
pub mod times;

/// A cena do **RITMO** (`=11`) — os *holds* do `motion.sub_uv`, com o metrónomo ao lado.
#[path = "motion_object_smoke_holds.rs"]
pub mod holds;
/// O modo `=9` — o ESTILO DO SINK (doc 89, folha 17). Irmão pelo mesmo corte: ele traz
/// oito cadeias próprias e um segundo objecto, e este despachante está no teto de LOC.
#[path = "motion_object_smoke_sink.rs"]
pub mod sink;
use times::{build_two_times_graph, spawn_flip_walk_named};

/// O nome que o artista daria ao objeto — e que ele escolhe no campo `Object`.
const OBJECT: &str = "Object";

/// Um tile de demo com cor (o átlas do boot empacota `0..16` como tiles HSV), pra
/// a arte carimbada ser inconfundivelmente a do sprite (modo `=1`).
const DEMO_TILE_KEY: u32 = 5;

/// O grafo comum aos dois modos: `source.object(name) → duplicator ← grid →
/// output`. A fonte traz a APARÊNCIA (na origem, um template), a grade dá os
/// PONTOS, e o duplicator cruza os dois. Devolve o sink.
fn build_stamp_graph(graph: &mut Graph, name: &str) -> NodeId {
    let src = graph.add_node("source.object");
    let grid = graph.add_node("motion.grid");
    let dup = graph.add_node("motion.duplicator");
    let out = graph.add_node("motion.output");
    // Layout acima do grafo da neve (que ocupa a faixa 0..).
    graph.set_pos(src, Pos { x: 0.0, y: -260.0 });
    graph.set_pos(grid, Pos { x: 0.0, y: -140.0 });
    graph.set_pos(
        dup,
        Pos {
            x: 210.0,
            y: -200.0,
        },
    );
    graph.set_pos(
        out,
        Pos {
            x: 420.0,
            y: -200.0,
        },
    );
    // shape → duplicator.shape (input 0); grid → duplicator.points (input 1).
    let wire = |g: &mut Graph, a: NodeId, ap: u16, b: NodeId, bp: u16| {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .expect("connect");
    };
    wire(graph, src, 0, dup, 0);
    wire(graph, grid, 0, dup, 1);
    wire(graph, dup, 0, out, 0);

    graph.set_text_param(src, "object", name);
    // Uma grade folgada 4×4 = 16 carimbos.
    graph.set_param(grid, "rows", 4.0);
    graph.set_param(grid, "cols", 4.0);
    graph.set_param(grid, "gap_x", 1.3);
    graph.set_param(grid, "gap_y", 1.3);
    graph.set_label(src, "The Object");
    graph.set_label(dup, "Stamp On Grid");

    // Nasce SELECIONADO: o artista cai no PICKER (um chip "Object").
    ph2d_panel_motion_graph::request_graph_selection(vec![src.0]);
    out
}

/// O frame corrente do roteiro (o hook não pode acrescentar campo em `App`).
static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// O modo: `0` off · `1` sprite (A1) · `2` vetor (A2) · `3` Flip (A3) · `4` grupo
/// (A4) · `5` A WAVE (objeto vetor + oscillator GPU) · `8` a POSE do objeto · `9` o
/// ESTILO DO SINK (doc 89 folha 17) · `11` o RITMO (os *holds* do sub-UV) · `12` a
/// FOLHA À FRENTE dos galhos (a terceira média — ver `motion_object_smoke_leaf`).
///
/// ⚠️ **O `12` faltava nesta lista** — a cena existia e o roteador dela não a nomeava
/// (auditoria de seis lentes, doc 96 §1.4). *Uma cena que o roteador não nomeia é encontrada
/// por `grep`, não alcançada por leitura.*
fn mode() -> u32 {
    static M: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *M.get_or_init(|| {
        std::env::var("PH2D_MOTION_OBJ_SMOKE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    })
}

/// Roda no prólogo do frame, ao lado dos outros smokes. No-op sem a env.
pub fn motion_object_smoke(cx: &mut crate::motion_scene_ctx::MotionSceneCtx<'_>) {
    use std::sync::atomic::Ordering;
    let mode = mode();
    if mode == 0 {
        return;
    }
    let f = FRAME.fetch_add(1, Ordering::Relaxed);
    match mode {
        // A1 — sprite: entidade direta, tudo num frame.
        1 if f == 3 => {
            spawn_sprite(cx.sim);
            let out = build_stamp_graph(&mut cx.motion.doc.graph, OBJECT);
            cx.motion.sinks.push(out);
            let _ = cx
                .tools
                .set_active(&ph2d_editor_core::ToolId::new("motion"));
            eprintln!(
                "[motion.obj smoke =1] O SPRITE 'Object' (tile colorido) esta carimbado numa \
                 grade 4x4 = 16 copias. A arte de CADA copia e a do sprite. Renomeie o sprite \
                 na Hierarchy e as copias somem (o nome E a referencia)."
            );
        }
        // A8 — a POSE do objeto (doc 89 folha 14). O corpo mora no irmão `pose`:
        // ele traz a fiação própria da cena, e este despachante estava no teto.
        8 if f == 3 => pose::run(cx),
        // A2 — vetor: a forma entra primeiro (frame 3); a ENTIDADE dela só
        // existe depois do `vec_entities::sync`, e e nela que o nome mora.
        2 if f == 3 => {
            cx.vec_scene.push_path(star_shape());
        }
        // =9 — o ESTILO DO SINK: a dança de duas fases do `=3` (a ENTIDADE do
        // objecto Flip nasce no `flip_entities::sync`, e é nela que o nome mora),
        // mais um sprite que é a SEGUNDA textura — sem duas texturas, a fileira da
        // ordem não tem o que mostrar.
        //
        // ⚠️ **O objecto é FLIP e não VECTOR, e a 1.ª versão errou nisto** (smoke do
        // Enio: os quatro pares idênticos). Um `source.object` de vector emite
        // `geometry_id` desde o ADR-0154 e a linha vai para o passe vectorial, onde
        // não existe `anchor`, `sampling`, `uv_xform` nem `sub_order` — os params
        // nunca chegavam ao lowering que os lê. Um Flip ASSA numa tile.
        // O RITMO (`=11`): dois flipbooks, o mesmo `speed`, e só um com poses seguras.
        // ⚠️ A arte é a MESMA da `=9` — quatro quadrantes de cores distintas — porque é a
        // única fixtura desta casa em que uma célula de sub-UV é inconfundível.
        // ⭐⭐ **=12 — A FOLHA A' FRENTE DOS GALHOS** (report do Enio, 2026-08-30). A folha
        // desta cena e' uma IMAGEM de proposito: e' o caso que o report nomeia, e o que a
        // TERCEIRA MEDIA destrava. ⚠️ **A razao que aqui estava escrita caiu** (doc 96 §1.4):
        // ela dizia que a `=108` nao podia mostrar isto *porque todo sprite desenha antes de
        // todo vector* -- verdade sobre os passes, e ja' nao sobre a folha, que com
        // `front > 0` passa a ser um quad no passe do vector.
        // Ver [`super::motion_object_smoke_leaf`].
        12 if f == 3 => {
            leaf::spawn_leaf_sprite(cx.sim);
            leaf::run(cx);
        }
        11 if f == 3 => {
            holds::spawn_art(cx.flip);
        }
        11 if f == 6 => holds::run(cx),
        9 if f == 3 => {
            sink::spawn_flip_art(cx.flip);
            sink::spawn_chip(cx.sim);
            // ⚠️ E a ESTRELA VECTORIAL — a fileira do pivô desenha-se por ela, que é a
            // prova de que o pivô alcança um objecto que NUNCA vira textura.
            cx.vec_scene.push_path(star_shape());
        }
        9 if f == 6 => {
            let map = cx.vec_entities.clone();
            if name_vector_entity_as(cx.sim, &map, sink::STAR) {
                sink::run(cx);
            }
        }
        2 if f == 6 => {
            let map = cx.vec_entities.clone();
            if name_vector_entity(cx.sim, &map) {
                let out = build_stamp_graph(&mut cx.motion.doc.graph, OBJECT);
                cx.motion.sinks.push(out);
            }
            let _ = cx
                .tools
                .set_active(&ph2d_editor_core::ToolId::new("motion"));
            // ⚠️ **Esta mensagem dizia «ASSADA numa tile pela membrana» e estava VELHA
            // desde o ADR-0154** — o modo `=5`, três braços abaixo, já dizia o
            // contrário no mesmo ficheiro. Ela custou uma cena inteira (a `=9` nasceu
            // com uma estrela e os quatro pares sairam identicos, smoke de 2026-08-25).
            // *Dois modos do mesmo smoke a descreverem comportamentos opostos para a
            // mesma entrada: o mais novo era o certo, e nada ligava os dois.*
            eprintln!(
                "[motion.obj smoke =2] A ESTRELA vetorial 'Object' esta carimbada numa grade \
                 4x4 = 16 copias, e cada copia e' desenhada VIVA pelo passe vectorial \
                 (`geometry_id`, ADR-0154) -- CRISP em qualquer zoom, sem tile raster. A tile \
                 assada so' volta acima de LOD_COUNT = 16.000 copias (smoke =6). Edite a forma \
                 com a tool Vector e as copias seguem (LIVE); renomeie e as copias somem."
            );
        }
        // A3 — Flip: o objeto entra no FlipDoc (frame 3); a ENTIDADE dele (Name
        // "Object") e criada pelo `flip_entities::sync`, entao o grafo o acha pelo
        // nome no frame 6 (sem nomear a mao — o sync copia o nome do objeto).
        3 if f == 3 => {
            spawn_flip_object(cx.flip);
        }
        3 if f == 6 => {
            let out = build_stamp_graph(&mut cx.motion.doc.graph, OBJECT);
            cx.motion.sinks.push(out);
            let _ = cx
                .tools
                .set_active(&ph2d_editor_core::ToolId::new("motion"));
            eprintln!(
                "[motion.obj smoke =3] O OBJETO Flip 'Object' (BG azul + FG laranja, 2 camadas) \
                 foi COMPOSTO no frame atual e ASSADO numa tile pela membrana, carimbado numa \
                 grade 4x4 = 16 copias. A arte de CADA copia e o objeto composto (as duas \
                 camadas, nao um quad chapado). Cacheada por conteudo (LIVE): editar/animar o \
                 Flip re-assa; renomear na Hierarchy e as copias somem."
            );
        }
        // A4 — grupo: um GRUPO 'Object' com filhos de MIDIA MISTA (sprite + vetor
        // + flip) nos seus lugares relativos. O grupo emite os filhos como N
        // instancias VIVAS; carimbar o grupo replica o layout inteiro.
        4 if f == 3 => {
            // O grupo (Name "Object" + GroupedChildren) + um sprite filho a esquerda.
            let group = cx
                .sim
                .world_mut()
                .spawn((
                    Name::new(OBJECT),
                    ph2d_ecs::GroupedChildren,
                    Transform::IDENTITY,
                ))
                .id();
            cx.sim.world_mut().spawn((
                Sprite::atlas(DEMO_TILE_KEY, [0.7, 0.7], [1.0, 1.0, 1.0, 1.0]),
                child_at(-1.1, 0.0),
                ph2d_ecs::ChildOf(group),
            ));
            // O vetor + o flip entram agora; suas ENTIDADES nascem no sync dos
            // frames seguintes, quando serao nomeadas e parenteadas ao grupo.
            cx.vec_scene.push_path(star_shape());
            spawn_flip_object_named(cx.flip, "GFlip");
        }
        4 if f == 6 => {
            let vec_map = cx.vec_entities.clone();
            let flip_map = cx.flip_state.entities.clone();
            if let Some(group) = find_group(cx.sim, OBJECT) {
                // O vetor (a unica forma da cena) vira filho SEM NOME no centro — o
                // caso do item 3 (doc 86 §9.6): um filho vetor/flip de grupo sem Name
                // continua carimbado, resolvido pelo seu DRAWING id (`VecPathRef`), nao
                // pelo nome. O bake o tila porque ele esta num grupo NOMEADO.
                if let Some((_, &bits)) = vec_map.iter().next() {
                    let e = ph2d_ecs::Entity::from_bits(bits);
                    if let Ok(mut ent) = cx.sim.world_mut().get_entity_mut(e) {
                        ent.insert((child_at(0.0, 0.0), ph2d_ecs::ChildOf(group)));
                    }
                }
                // O flip (o unico objeto Flip) vira filho a direita. O sync ja
                // carimbou a entidade com Name "GFlip" (do push_object), entao so
                // parenteamos + posicionamos.
                if let Some((_, &bits)) = flip_map.iter().next() {
                    let e = ph2d_ecs::Entity::from_bits(bits);
                    if let Ok(mut ent) = cx.sim.world_mut().get_entity_mut(e) {
                        ent.insert((child_at(1.1, 0.0), ph2d_ecs::ChildOf(group)));
                    }
                }
            }
        }
        4 if f == 9 => {
            let out = build_stamp_graph(&mut cx.motion.doc.graph, OBJECT);
            cx.motion.sinks.push(out);
            let _ = cx
                .tools
                .set_active(&ph2d_editor_core::ToolId::new("motion"));
            eprintln!(
                "[motion.obj smoke =4] O GRUPO 'Object' (sprite SEM NOME + estrela vetor SEM \
                 NOME + objeto Flip 'GFlip', MIDIA MISTA) esta carimbado numa grade 4x4 = 16 \
                 copias. Cada copia mostra o GRUPO INTEIRO (3 filhos lado a lado) e cada filho \
                 continua VIVO (o Flip anima independente). A estrela vetor do CENTRO NAO tem \
                 nome (item 3): ela e tilada porque esta num grupo nomeado e resolvida pelo seu \
                 drawing id, nao pelo nome. SE ela aparecer nas 16 copias, o item 3 passou; se \
                 o centro sair em branco, FALHOU. Renomeie o grupo e as copias somem."
            );
        }
        7 if f == 3 => {
            spawn_flip_walk_named(cx.flip, OBJECT);
        }
        7 if f == 6 => {
            let outs = build_two_times_graph(&mut cx.motion.doc.graph, OBJECT);
            cx.motion.sinks.extend(outs);
            let _ = cx
                .tools
                .set_active(&ph2d_editor_core::ToolId::new("motion"));
            // O relógio ANDA: um offset de tempo só é visível numa animação que
            // corre. Uma cena parada mostraria dois desenhos diferentes e não
            // diria se a diferença é de FASE.
            cx.playhead.play();
            eprintln!(
                "[motion.obj smoke =7] o MESMO objeto Flip ({OBJECT}, 12 fps, 4 desenhos \
                 em 0/3/6/9) trazido DUAS vezes: a grade da ESQUERDA em time_offset = 0 \
                 (o quadro atual) e a da DIREITA em +0,25 s, que a 12 fps sao TRES \
                 desenhos a frente. O QUE OLHAR: as duas grades desenham a MESMA arte \
                 (campo azul + quadrado laranja) e o quadrado da direita esta sempre UM \
                 PASSO a frente do da esquerda — se os dois andarem juntos, o offset nao \
                 chegou ao bake; se a direita SUMIR, o canal deslocado nao foi publicado. \
                 Pare o play e faca scrub: a diferenca de fase tem de se manter em \
                 qualquer quadro."
            );
        }
        _ => {}
    }
}
