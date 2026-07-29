//! **A cena pronta para o smoke do `motion.path`** (`PH2D_MOTION_NODE_PATH_SMOKE=1`, doc 65).
//!
//! Um nó que anda numa forma **desenhada** não pode ser demonstrado num documento de boot que não
//! tem desenho nenhum — e a demo de boot é a NEVE (Enio: *"deixe só o grafo da chuva"*). Então esta
//! cena vive atrás de uma env, como as do Shape Builder (`build_smoke.rs`), e monta as duas metades
//! que o artista montaria:
//!
//! 1. **a forma**: uma curva em S no documento vetorial, chamada **`Track`** na Hierarchy — porque o
//!    **nome é a referência inteira** (doc 65: não há id pra copiar);
//! 2. **o grafo**: `value.lfo → motion.path → scale → output` — um segundo sink, ao lado da neve.
//!
//! O que se vê: 24 instâncias percorrendo a curva em arco-comprimento **uniforme**, **giradas para a
//! tangente**, e **fluindo** (o LFO empurra o `offset`, que dá a volta). Arraste a curva com a tool
//! Vector e o conjunto **segue** — é o memo enxergando o external.
//!
//! **E o campo `Shape` é um PICKER** (não uma caixa de texto): o nó nasce SELECIONADO, e o painel de
//! params mostra a forma desenhada como um chip **`Track`** (destacado, porque o `path` já é "Track"),
//! com o campo de texto como escape. Desenhe outra forma com a tool Vector e um chip novo aparece ao
//! vivo (a lista vem do `Cook::externals`). Clicar um chip é o gesto que ANTES pedia digitar o nome
//! interno exato — o "nó pra artista, não pra matemático" que o Enio pediu.

use ph2d_ecs::Name;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O nome que o artista daria à forma — e que ele digita no campo `Shape` do nó.
const TRACK: &str = "Track";

/// A curva: um S largo, aberto, com quinas suaves. Âncora + 2 handles por vértice (o modelo do
/// Rive, `VecVertex`).
fn s_curve() -> ph2d_vec_scene::VecPath {
    use ph2d_vec_scene::{VecPath, VecVertex, VertexKind};
    let v = |a: [f64; 2], i: [f64; 2], o: [f64; 2]| VecVertex {
        anchor: a,
        in_handle: i,
        out_handle: o,
        kind: VertexKind::Smooth,
        corner_radius: 0.0,
    };
    VecPath {
        verts: vec![
            v([-3.2, -1.6], [-3.2, -1.6], [-1.6, -1.6]),
            v([0.0, 0.0], [-1.4, 0.0], [1.4, 0.0]),
            v([3.2, 1.6], [1.6, 1.6], [3.2, 1.6]),
        ],
        closed: false,
        stroke: Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(150, 160, 180, 255),
            0.02,
        )),
        ..VecPath::default()
    }
}

/// Frame 3: a forma entra no documento vetorial (em MUNDO, `Transform` na identidade — é como a
/// Shape tool deixa uma forma recém-desenhada; o `settle_origins` do frame a centra e põe a pose na
/// entidade, ADR-0111/0112).
pub(crate) fn push_shape(scene: &mut ph2d_vec_scene::VecScene) {
    scene.push_path(s_curve());
}

/// Frame 6: a forma ganha o **nome** (a entidade dela já existe — o `sync` do frame a criou), e o
/// grafo ganha a cadeia que a percorre.
///
/// O nome é dado na ENTIDADE, não no path: é o `Name` do ECS que a Hierarchy mostra e que o
/// publisher lê (doc 65). Se o artista renomear ali, o nó para de achar a curva — que é exatamente o
/// que renomear uma coisa a que você se referiu pelo nome significa.
pub(crate) fn name_and_wire(
    sim: &mut ph2d_ecs::SimWorld,
    map: &crate::vec_entities::VecEntityMap,
    graph: &mut Graph,
) -> Option<NodeId> {
    // A única forma da cena é a nossa.
    let (_, &bits) = map.iter().next()?;
    let e = ph2d_ecs::Entity::from_bits(bits);
    sim.world_mut()
        .get_entity_mut(e)
        .ok()?
        .insert(Name(TRACK.to_string()));

    // `value.lfo → motion.path → scale → output`. O LFO empurra o `offset`, e o `motion.path`
    // embrulha — uma curva é coisa pra dar a volta, não pra cair da ponta.
    let lfo = graph.add_node("value.lfo");
    let path = graph.add_node("motion.path");
    let scale = graph.add_node("motion.scale");
    let out = graph.add_node("motion.output");
    for (i, n) in [lfo, path, scale, out].iter().enumerate() {
        graph.set_pos(
            *n,
            Pos {
                x: i as f32 * 190.0,
                y: -220.0, // acima do grafo da neve, que ocupa a faixa 0..
            },
        );
    }
    for (a, b) in [(lfo, path), (path, scale), (scale, out)] {
        graph
            .connect(Edge {
                from: (a, 0),
                to: (b, 0),
                delayed: false,
            })
            .ok()?;
    }
    graph.set_text_param(path, "path", TRACK);
    graph.set_param(path, "count", 24.0);
    graph.set_param(path, "align", 1.0);
    graph.set_param(scale, "amount", 0.14);
    // Um LFO lento, de 0 a 1: o conjunto dá uma volta na curva a cada ~6 s. (Os params do
    // `value.lfo` são `wave`/`period`/`amplitude`/`offset` — o período é em SEGUNDOS.)
    graph.set_param(lfo, "period", 6.0);
    graph.set_param(lfo, "amplitude", 0.5);
    graph.set_param(lfo, "offset", 0.5);
    graph.set_label(path, "Walk The Track");
    graph.set_label(lfo, "Flow");
    // Select the `motion.path` node so its params show at once — the artist lands on the
    // Shape PICKER (a chip "Track", + the text field), not a blank text box (doc 65).
    ph2d_panel_motion_graph::request_graph_selection(vec![path.0]);
    eprintln!(
        "[motion.path smoke] O no 'Walk The Track' esta SELECIONADO: o painel de params mostra o \
         campo Shape como um PICKER -- um chip 'Track' (destacado) + o campo de texto. Clique o \
         chip em vez de digitar; desenhe outra forma com a tool Vector e um chip novo aparece ao \
         vivo. 24 instancias percorrem a curva."
    );
    Some(out)
}

/// O frame corrente do roteiro (o hook não pode acrescentar campo em `App`).
static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_MOTION_NODE_PATH_SMOKE").is_ok_and(|v| v != "0"))
}

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    ///
    /// **Dois frames, e a ordem importa:** a forma entra primeiro; a ENTIDADE dela só existe depois
    /// que o `vec_entities::sync` do frame a cria — e é na entidade que o **nome** mora (ADR-0110).
    /// Nomear antes seria nomear algo que ainda não existe, e o publisher não veria curva nenhuma.
    pub(crate) fn motion_node_path_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() {
            return;
        }
        match FRAME.fetch_add(1, Ordering::Relaxed) {
            3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                push_shape(&mut gfx.vec_scene);
            }
            6 => {
                // O mapa path↔entidade vive no `App` (o `sync` do frame o preenche); o mundo, a
                // cena e o grafo vivem no `AppGfx`.
                let map = self.vec_entities.clone();
                let gfx = self.gfx.as_mut().expect("gfx");
                if let Some(out) = name_and_wire(&mut gfx.sim, &map, &mut gfx.motion.doc.graph) {
                    gfx.motion.sinks.push(out);
                }
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
            }
            _ => {}
        }
    }
}
