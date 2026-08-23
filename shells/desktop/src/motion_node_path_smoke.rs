//! **A cena pronta para o smoke do `motion.path`** (`PH2D_MOTION_NODE_PATH_SMOKE=1`, doc 65).
//!
//! Um nó que anda numa forma **desenhada** não pode ser demonstrado sem um desenho, e o editor de
//! Motion abre com a tela VAZIA (Enio, 2026-08-07: *"tire a cena da cachoeira"*). Então esta cena
//! vive atrás de uma env, como as do Shape Builder (`build_smoke.rs`), e monta as duas metades que
//! o artista montaria:
//!
//! 1. **a forma**: uma curva em S no documento vetorial, chamada **`Track`** na Hierarchy — porque o
//!    **nome é a referência inteira** (doc 65: não há id pra copiar);
//! 2. **o grafo**: `value.lfo → motion.path → scale → output`.
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
                y: -220.0, // sua própria faixa (a 0.. era a do antigo grafo de boot)
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

// ---------------------------------------------------------------------------------------------
// A CENA DO ESPAÇAMENTO (`=2`) — a contagem segue o COMPRIMENTO
// ---------------------------------------------------------------------------------------------
//
// ⚠️ **Ela vive AQUI e não no roteador de `PH2D_GPU_COOK_DEMO`, e o motivo é estrutural:** as
// cenas de conferência montam um `MotionDoc` puro, e um nó que anda numa forma DESENHADA precisa
// do documento vetorial — a forma, a entidade que a nomeia e o publisher, que é exactamente o
// roteiro de dois frames que este arquivo já executa. Uma segunda encenação disso seria a segunda
// resposta a *"como uma curva chega ao grafo?"*.

/// Os quatro nomes, na ordem em que as formas entram na cena.
const SPACING_TRACKS: [&str; 4] = ["Count Short", "Count Long", "Spacing Short", "Spacing Long"];

/// Um segmento RETO de comprimento `len`, começando em `x = -4`, na altura `y`.
///
/// ⚠️ **Reto de propósito.** A cena `=1` já responde *"o conjunto percorre uma curva?"*; esta
/// responde *"quantos, e a que distância?"*, e num segmento o vão entre duas peças é o número que
/// o olho lê sem descontar curvatura. A reta é a cúbica DEGENERADA do documento (handles sobre as
/// âncoras), que é como a Pen a deixa.
fn straight(len: f64, y: f64) -> ph2d_vec_scene::VecPath {
    use ph2d_vec_scene::{VecPath, VecVertex, VertexKind};
    let v = |x: f64| VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    };
    VecPath {
        verts: vec![v(-4.0), v(-4.0 + len)],
        closed: false,
        stroke: Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(150, 160, 180, 255),
            0.02,
        )),
        ..VecPath::default()
    }
}

/// Frame 3: as QUATRO trilhas entram, duas curtas e duas longas.
///
/// ⚠️ **Duas trilhas por lei, e não uma trilha com dois grafos:** duas cadeias sobre a MESMA curva
/// desenhariam as peças umas por cima das outras, e o vão — que é a coisa que a cena existe para
/// mostrar — ficaria ilegível.
pub(crate) fn push_spacing_shapes(scene: &mut ph2d_vec_scene::VecScene) {
    for (len, y) in [(4.0, 3.0), (8.0, 1.0), (4.0, -1.0), (8.0, -3.0)] {
        scene.push_path(straight(len, y));
    }
}

/// Frame 6: cada trilha ganha o NOME e a cadeia que a percorre.
///
/// As duas de cima contam por NÚMERO (o mundo que sempre shipou) e as duas de baixo por
/// ESPAÇAMENTO. As quatro carregam a mesma peça, então a única coisa que difere entre as linhas é
/// o que a wave acrescenta.
/// **A NORMAL, lado a lado com a TANGENTE** (`=3`, doc 89 folha 06).
///
/// Duas cadeias sobre a **mesma** curva desenhada, uma alinhada à tangente (o modo que
/// sempre shipou) e outra à normal, separadas por um `motion.transform` para o par se
/// julgar num olhar. ⚠️ As peças são **compridas e finas** de propósito: um quadrado
/// não tem como mostrar para onde aponta, e um gate de rotação verde sobre uma peça
/// redonda seria o gate certo sobre a cena errada.
fn name_and_wire_normal(
    sim: &mut ph2d_ecs::SimWorld,
    map: &crate::vec_entities::VecEntityMap,
    graph: &mut Graph,
) -> Vec<NodeId> {
    let Some((_, &bits)) = map.iter().next() else {
        return Vec::new();
    };
    let e = ph2d_ecs::Entity::from_bits(bits);
    let Ok(mut w) = sim.world_mut().get_entity_mut(e) else {
        return Vec::new();
    };
    w.insert(Name(TRACK.to_string()));
    let mut outs = Vec::new();
    for (k, (align, dy)) in [(1.0_f32, 1.1_f32), (2.0, -1.1)].iter().enumerate() {
        let path = graph.add_node("motion.path");
        let scale = graph.add_node("motion.scale");
        let place = graph.add_node("motion.transform");
        let out = graph.add_node("motion.output");
        for (i, n) in [path, scale, place, out].iter().enumerate() {
            graph.set_pos(
                *n,
                Pos {
                    x: i as f32 * 190.0,
                    y: -220.0 + k as f32 * 200.0,
                },
            );
        }
        let wired = [(path, scale), (scale, place), (place, out)]
            .iter()
            .all(|(a, b)| {
                graph
                    .connect(Edge {
                        from: (*a, 0),
                        to: (*b, 0),
                        delayed: false,
                    })
                    .is_ok()
            });
        if !wired {
            return outs;
        }
        graph.set_text_param(path, "path", TRACK);
        graph.set_param(path, "count", 20.0);
        graph.set_param(path, "align", *align);
        // Comprida e fina: a peça TEM de mostrar para onde aponta.
        graph.set_param(scale, "uniform", 0.0);
        graph.set_param(scale, "amount", 0.34);
        graph.set_param(scale, "amount_y", 0.07);
        graph.set_param(place, "offset_y", *dy);
        graph.set_label(
            path,
            if *align >= 1.5 {
                "Normal (agora)"
            } else {
                "Tangente (antes)"
            },
        );
        outs.push(out);
    }
    eprintln!(
        "[motion.path smoke =3] Duas fileiras sobre a MESMA curva. Em cima as pecas deitam-se \
         ao longo do caminho (Tangente, o de sempre); em baixo elas ficam de TRAVESSA, a apontar \
         para fora da curva (Normal, o modo novo). Se as duas fileiras estiverem iguais, o modo \
         nao entrou."
    );
    outs
}

fn name_and_wire_spacing(
    sim: &mut ph2d_ecs::SimWorld,
    map: &crate::vec_entities::VecEntityMap,
    graph: &mut Graph,
) -> Vec<NodeId> {
    // O `VecEntityMap` é um `BTreeMap` por id de path, e o `push_path` os cunha em ordem ⇒ a
    // ordem de iteração é a ordem em que as trilhas foram empurradas.
    let bits: Vec<u64> = map.values().copied().collect();
    if bits.len() < SPACING_TRACKS.len() {
        eprintln!(
            "[motion.path spacing] PARE: esperava {} trilhas na cena e achei {}.",
            SPACING_TRACKS.len(),
            bits.len()
        );
        return Vec::new();
    }
    for (name, &b) in SPACING_TRACKS.iter().zip(&bits) {
        let e = ph2d_ecs::Entity::from_bits(b);
        if let Ok(mut ent) = sim.world_mut().get_entity_mut(e) {
            ent.insert(Name((*name).to_string()));
        }
    }

    let mut outs = Vec::new();
    for (row, name) in SPACING_TRACKS.iter().enumerate() {
        let path = graph.add_node("motion.path");
        let scale = graph.add_node("motion.scale");
        let out = graph.add_node("motion.output");
        for (i, n) in [path, scale, out].iter().enumerate() {
            graph.set_pos(
                *n,
                Pos {
                    #[expect(clippy::cast_precision_loss, reason = "índice de layout do grafo")]
                    x: i as f32 * 190.0,
                    #[expect(clippy::cast_precision_loss, reason = "índice de layout do grafo")]
                    y: row as f32 * 130.0 - 220.0,
                },
            );
        }
        for (a, b) in [(path, scale), (scale, out)] {
            if graph
                .connect(Edge {
                    from: (a, 0),
                    to: (b, 0),
                    delayed: false,
                })
                .is_err()
            {
                return outs;
            }
        }
        graph.set_text_param(path, "path", *name);
        graph.set_param(path, "align", 0.0);
        if row < 2 {
            graph.set_param(path, "count", 9.0);
        } else {
            graph.set_param(path, "mode", 1.0);
            graph.set_param(path, "spacing", 0.5);
        }
        graph.set_param(scale, "amount", 0.14);
        graph.set_label(path, *name);
        outs.push(out);
    }

    eprintln!(
        "[motion.path spacing] Quatro trilhas RETAS, duas curtas (4) e duas longas (8):\n\
         \x20 1 Count Short    len 4  count 9        -> 9 pecas,  vao 0,444\n\
         \x20 2 Count Long     len 8  count 9        -> 9 pecas,  vao 0,889\n\
         \x20 3 Spacing Short  len 4  spacing 0,50   -> 8 pecas,  vao 0,500\n\
         \x20 4 Spacing Long   len 8  spacing 0,50   -> 16 pecas, vao 0,500\n\
         A cena julga-se PARADA. As duas leituras: (1) as de CIMA tem o MESMO numero de pecas e\n\
         vaos DIFERENTES -- a de baixo do par espalha o dobro; (2) as de BAIXO tem o MESMO vao e\n\
         numeros diferentes -- a longa cabe o dobro de pecas. Se as quatro linhas tiverem a mesma\n\
         contagem, o modo Spacing nao chegou."
    );
    outs
}

/// O frame corrente do roteiro (o hook não pode acrescentar campo em `App`).
static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Qual cena? `0`/ausente = nenhuma · `2` = o ESPAÇAMENTO · `3` = a NORMAL (folha 06) ·
/// qualquer outra = a original.
///
/// Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro). ⚠️ **O braço `_` mantém
/// `=1` — e todo valor que não seja `0` nem `2` — na cena que o Enio já smokou:** um modo novo
/// não pode mudar o que a env que já existe significa.
fn mode() -> u8 {
    static MODE: std::sync::OnceLock<u8> = std::sync::OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("PH2D_MOTION_NODE_PATH_SMOKE") {
        Ok(v) if v == "0" => 0,
        Ok(v) if v == "2" => 2,
        Ok(v) if v == "3" => 3,
        Ok(_) => 1,
        Err(_) => 0,
    })
}

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `build_smoke`. No-op sem a env.
    ///
    /// **Dois frames, e a ordem importa:** a forma entra primeiro; a ENTIDADE dela só existe depois
    /// que o `vec_entities::sync` do frame a cria — e é na entidade que o **nome** mora (ADR-0110).
    /// Nomear antes seria nomear algo que ainda não existe, e o publisher não veria curva nenhuma.
    pub(crate) fn motion_node_path_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        let mode = mode();
        if mode == 0 || self.gfx.is_none() {
            return;
        }
        match FRAME.fetch_add(1, Ordering::Relaxed) {
            3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                if mode == 2 {
                    push_spacing_shapes(&mut gfx.vec_scene);
                } else {
                    push_shape(&mut gfx.vec_scene);
                }
            }
            6 => {
                // O mapa path↔entidade vive no `App` (o `sync` do frame o preenche); o mundo, a
                // cena e o grafo vivem no `AppGfx`.
                let map = self.vec_entities.clone();
                let gfx = self.gfx.as_mut().expect("gfx");
                if mode == 2 {
                    let outs = name_and_wire_spacing(&mut gfx.sim, &map, &mut gfx.motion.doc.graph);
                    gfx.motion.sinks.extend(outs);
                } else if mode == 3 {
                    let outs = name_and_wire_normal(&mut gfx.sim, &map, &mut gfx.motion.doc.graph);
                    gfx.motion.sinks.extend(outs);
                } else if let Some(out) =
                    name_and_wire(&mut gfx.sim, &map, &mut gfx.motion.doc.graph)
                {
                    gfx.motion.sinks.push(out);
                }
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
            }
            _ => {}
        }
    }
}
