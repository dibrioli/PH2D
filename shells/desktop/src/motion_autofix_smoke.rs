//! **A cena para o smoke da auto-correção de setup** (`PH2D_AUTOFIX_SMOKE`, ADR-0155):
//! uma `force.wind` colocada DIRETO na cadeia horizontal, **sem integrador**. O
//! artista faz o gesto errado (ligar a força ao output); no instante em que a aresta
//! conecta, o app **auto-conserta o setup** — reencaminha a força por um
//! `motion.integrate` (a grid passa a alimentar o integrador, a força vira o ramo de
//! `forces`, e o `reconcile` plumba o laço `pre`), e os pontos **passam a DERIVAR**
//! com o vento. Antes do conserto, nada se movia.
//!
//! No frame 90 o artista **apaga o integrador** (gesto destrutivo): os pontos
//! CONGELAM e o app **NÃO** re-insere — apagar para religar à mão nunca é combatido.
//!
//! **`=2` (REORDER):** um sim que JÁ FUNCIONA (`grid -> integrate -> output`) e o
//! artista solta uma `force.wind` SOBRE o fio de saída (um Splice) — a força cai
//! DEPOIS do integrador, onde o accel dela nunca é consumido. O app **REUSA** o
//! integrador que já está ali (não insere um segundo): a força vira o ramo de forças,
//! e os pontos passam a DERIVAR. Um Ctrl+Z reverte só o reorder.
//!
//! **`=3` (O BADGE + QUICK-FIX, ADR-0155 W3):** dois setups inertes montados SEM gesto
//! construtivo — nada auto-corrige sozinho. Cada nó ganha o pip ⚠: `grid -> force.wind
//! -> output` (a força escreve `accel` que nada consome) e `grid -> pin -> output` (a
//! restrição precisa de ALGUM solver). O artista **CLICA** cada badge: o da força tem
//! cura canônica → o app AUTO-INSERE `motion.integrate` e os pontos derivam; o do pin
//! **não** tem → o app só EXPLICA + seleciona (a lei do ADR-0155 de nunca adivinhar
//! uma escolha criativa). A cena imprime quantos badges o diagnoser marcou.

use ph2d_nodegraph::graph::{Edge, Pos};
use ph2d_panel_motion_graph::GraphIntent;

/// O modo (`0` off): `1` inserir (força na cadeia horizontal); `2` reorder (força
/// spliceada depois de um integrador que já existe); `3` badge + quick-fix (dois setups
/// inertes SEM gesto, o artista clica o pip para consertar/explicar); `4` aviso da
/// família `falloff` + o toggle "Node Help" que liga/desliga o sistema inteiro; `5` aviso
/// de requisito-a-montante (um deformer/força SEM fonte de pontos); `6` aviso de porta
/// obrigatória (um `motion.duplicator` sem a entrada `points`). Os avisos `4` e `5` vêm
/// por DERIVAÇÃO; o `6` por DECLARAÇÃO opt-in (required-vs-opcional é semântico).
fn mode() -> u32 {
    static M: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *M.get_or_init(|| {
        std::env::var("PH2D_AUTOFIX_SMOKE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    })
}

/// O frame corrente do roteiro (o hook não pode acrescentar campo em `App`).
static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado dos outros smokes. No-op sem a env.
    pub(crate) fn motion_autofix_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if mode() == 0 || self.gfx.is_none() {
            return;
        }
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        match (mode(), f) {
            // =1, frame 3: monta grid -> force.wind + output SOLTO, abre a tool Motion,
            // e empurra o gesto ERRADO (Connect force -> output, sem integrador). O
            // bridge drena o intent, conecta e auto-conserta no mesmo frame.
            (1, 3) => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let (grid, force, out) = {
                    let g = &mut gfx.motion.doc.graph;
                    let grid = g.add_node("motion.grid");
                    let force = g.add_node("force.wind");
                    let out = g.add_node("motion.output");
                    g.set_pos(
                        grid,
                        Pos {
                            x: -220.0,
                            y: -260.0,
                        },
                    );
                    g.set_pos(force, Pos { x: 0.0, y: -260.0 });
                    g.set_pos(
                        out,
                        Pos {
                            x: 260.0,
                            y: -260.0,
                        },
                    );
                    g.connect(Edge {
                        from: (grid, 0),
                        to: (force, 0),
                        delayed: false,
                    })
                    .expect("grid -> force");
                    // Vento forte e para baixo, para a deriva ser óbvia na tela.
                    g.set_param(force, "strength", 8.0);
                    g.set_param(force, "angle", -90.0);
                    (grid, force, out)
                };
                let _ = grid;
                gfx.motion.sinks.push(out);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                // Nasce SELECIONADA a força, para o artista cair no card dela.
                ph2d_panel_motion_graph::request_graph_selection(vec![force.0]);
                // O GESTO ERRADO: liga a força direto ao output.
                ph2d_panel_motion_graph::push_intent(GraphIntent::Connect {
                    from_node: force.0,
                    from_port: 0,
                    to_node: out.0,
                    to_port: 0,
                });
                eprintln!(
                    "[autofix smoke =1] force.wind ligada DIRETO ao output, SEM integrador \
                     (o setup que nao move nada). O app deve AUTO-INSERIR motion.integrate: a \
                     grid passa a alimentar o integrador, a forca vira o ramo de forcas, e os \
                     pontos passam a DERIVAR para baixo com o vento. Um Ctrl+Z remove SO o \
                     conserto (a forca fica onde estava); outro Ctrl+Z desfaz a conexao. No \
                     frame 90 o integrador e' apagado."
                );
            }
            // =1, frame 90: apaga o integrador — gesto destrutivo, o app NAO re-insere.
            (1, 90) => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let integ = gfx
                    .motion
                    .doc
                    .graph
                    .nodes()
                    .iter()
                    .find(|n| n.type_name == "motion.integrate")
                    .map(|n| n.id.0);
                if let Some(id) = integ {
                    ph2d_panel_motion_graph::push_intent(GraphIntent::DeleteSelection {
                        nodes: vec![id],
                    });
                    eprintln!(
                        "[autofix smoke =1] apaguei o integrador (gesto DESTRUTIVO): os pontos \
                         CONGELAM e o app NAO re-insere — apagar para religar a' mao nunca e' \
                         combatido. Refazer (Ctrl+Shift+Z) devolve o movimento."
                    );
                } else {
                    eprintln!(
                        "[autofix smoke =1] AVISO: nenhum motion.integrate encontrado — o \
                         auto-heal nao rodou. Pare e investigue."
                    );
                }
            }
            // =2, frame 3: um sim que JA FUNCIONA (grid -> integrate -> output) e o
            // artista solta uma force.wind SOBRE o fio de saida (SpliceNode). A forca cai
            // DEPOIS do integrador, onde o accel dela nunca e' consumido. O app REUSA
            // esse integrador (nao insere um segundo) e a forca vira o ramo de forcas ->
            // os pontos DERIVAM (+X, com o strength 3 default do wind).
            (2, 3) => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let out = {
                    let g = &mut gfx.motion.doc.graph;
                    let grid = g.add_node("motion.grid");
                    let integ = g.add_node("motion.integrate");
                    let out = g.add_node("motion.output");
                    g.set_pos(
                        grid,
                        Pos {
                            x: -260.0,
                            y: -260.0,
                        },
                    );
                    g.set_pos(integ, Pos { x: 0.0, y: -260.0 });
                    g.set_pos(
                        out,
                        Pos {
                            x: 260.0,
                            y: -260.0,
                        },
                    );
                    g.connect(Edge {
                        from: (grid, 0),
                        to: (integ, 0), // grid -> integrate.rest
                        delayed: false,
                    })
                    .expect("grid -> integrate.rest");
                    g.connect(Edge {
                        from: (integ, 0),
                        to: (out, 0), // integrate -> output (o sim que ja' roda)
                        delayed: false,
                    })
                    .expect("integrate -> output");
                    out
                };
                gfx.motion.sinks.push(out);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                // O GESTO: soltar a forca SOBRE o fio integrate -> output.
                ph2d_panel_motion_graph::push_intent(GraphIntent::SpliceNode {
                    to_node: out.0,
                    to_port: 0,
                    to_type: "force.wind",
                    x: 130.0,
                    y: -260.0,
                });
                eprintln!(
                    "[autofix smoke =2] sim que JA FUNCIONA (grid -> integrate -> output); \
                     solto uma force.wind SOBRE o fio de saida. A forca cai DEPOIS do \
                     integrador (o accel dela nunca e' consumido ali). O app deve REUSAR o \
                     integrador que ja' esta' ali (NAO inserir um segundo): a forca vira o \
                     ramo de forcas e os pontos DERIVAM para +X. Confira que existe UM SO' \
                     motion.integrate. Um Ctrl+Z reverte so' o reorder."
                );
            }
            // =3, frame 3: monta DOIS setups inertes SEM gesto construtivo (nenhum
            // auto-heal dispara). Setup A: grid -> force.wind -> output (a forca escreve
            // accel que nada consome). Setup B: grid -> pin -> output (a restricao precisa
            // de ALGUM solver). Ambos os nos ganham o pip ⚠ (inert + alcancam a saida). O
            // artista CLICA cada badge: o da forca AUTO-conserta (insere motion.integrate);
            // o do pin so' EXPLICA + seleciona (a lei do ADR-0155 de nunca adivinhar).
            (3, 3) => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let (out_f, out_p) = {
                    let g = &mut gfx.motion.doc.graph;
                    // Setup A: a forca sem integrador (badge fixavel).
                    let grid_f = g.add_node("motion.grid");
                    let force = g.add_node("force.wind");
                    let out_f = g.add_node("motion.output");
                    g.set_pos(
                        grid_f,
                        Pos {
                            x: -220.0,
                            y: -300.0,
                        },
                    );
                    g.set_pos(force, Pos { x: 40.0, y: -300.0 });
                    g.set_pos(
                        out_f,
                        Pos {
                            x: 300.0,
                            y: -300.0,
                        },
                    );
                    g.connect(Edge {
                        from: (grid_f, 0),
                        to: (force, 0),
                        delayed: false,
                    })
                    .expect("grid -> force");
                    g.connect(Edge {
                        from: (force, 0),
                        to: (out_f, 0),
                        delayed: false,
                    })
                    .expect("force -> output");
                    g.set_param(force, "strength", 8.0);
                    g.set_param(force, "angle", -90.0);
                    // Setup B: o pin sem solver (badge advisory — sem cura canonica).
                    let grid_p = g.add_node("motion.grid");
                    let pin = g.add_node("motion.pin_constraint");
                    let out_p = g.add_node("motion.output");
                    g.set_pos(
                        grid_p,
                        Pos {
                            x: -220.0,
                            y: -60.0,
                        },
                    );
                    g.set_pos(pin, Pos { x: 40.0, y: -60.0 });
                    g.set_pos(out_p, Pos { x: 300.0, y: -60.0 });
                    g.connect(Edge {
                        from: (grid_p, 0),
                        to: (pin, 0),
                        delayed: false,
                    })
                    .expect("grid -> pin");
                    g.connect(Edge {
                        from: (pin, 0),
                        to: (out_p, 0),
                        delayed: false,
                    })
                    .expect("pin -> output");
                    (out_f, out_p)
                };
                gfx.motion.sinks.push(out_f);
                gfx.motion.sinks.push(out_p);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                let badges =
                    ph2d_motion_diagnose::diagnose(&gfx.motion.doc.graph, &gfx.motion.registry)
                        .len();
                eprintln!(
                    "[autofix smoke =3] montei DOIS setups inertes SEM gesto construtivo: \
                     {badges} produtor(es) inerte(s) marcado(s) (esperado 2 — a force.wind e o \
                     pin, ambos alcancam a saida). SE NAO FOR 2, PARE. Cada um deve pintar um \
                     pip ⚠ no canto do card. CLIQUE o badge da force.wind: o app AUTO-INSERE \
                     motion.integrate e os pontos DERIVAM para baixo (undo reverte so' o \
                     conserto). CLIQUE o badge do pin: o app so' EXPLICA (toast) e SELECIONA o \
                     pin — NADA muda no grafo (nao ha' cura canonica; a lei do ADR-0155 de \
                     nunca adivinhar uma escolha criativa)."
                );
            }
            // =4, frame 3: a capacidade NOVA — o AVISO da familia `falloff` + o TOGGLE.
            // Monta `grid -> field.box -> output`: o campo MOLDA um `falloff` que NENHUMA
            // forca/deformer le, entao ele NAO faz nada (a mesma classe de erro do
            // ADR-0155, agora coberta para o `falloff` — e o field.box NAO tem Coupling
            // anotado: o aviso vem PURAMENTE da binding de GPU que ele ja' declara, a
            // DERIVACAO que e' o coracao desta wave). O field.box ganha o pip ⚠; clicar
            // EXPLICA (um campo precisa de ALGUMA forca/deformer — nao ha' cura canonica)
            // + seleciona, nunca adivinha (Offer). Depois o chip "Node Help" desliga o
            // sistema inteiro (o badge some) e liga de volta — a liberdade do artista.
            (4, 3) => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let out = {
                    let g = &mut gfx.motion.doc.graph;
                    let grid = g.add_node("motion.grid");
                    let field = g.add_node("field.box");
                    let out = g.add_node("motion.output");
                    g.set_pos(
                        grid,
                        Pos {
                            x: -220.0,
                            y: -200.0,
                        },
                    );
                    g.set_pos(field, Pos { x: 40.0, y: -200.0 });
                    g.set_pos(
                        out,
                        Pos {
                            x: 300.0,
                            y: -200.0,
                        },
                    );
                    g.connect(Edge {
                        from: (grid, 0),
                        to: (field, 0),
                        delayed: false,
                    })
                    .expect("grid -> field");
                    g.connect(Edge {
                        from: (field, 0),
                        to: (out, 0),
                        delayed: false,
                    })
                    .expect("field -> output");
                    out
                };
                gfx.motion.sinks.push(out);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                let falloff =
                    ph2d_motion_diagnose::diagnose(&gfx.motion.doc.graph, &gfx.motion.registry)
                        .iter()
                        .filter(|d| {
                            d.deficit == ph2d_motion_diagnose::Deficit::InertProducer("falloff")
                        })
                        .count();
                eprintln!(
                    "[autofix smoke =4] montei `grid -> field.box -> output`: {falloff} campo(s) \
                     de falloff inerte(s) marcado(s) (esperado 1 — o field.box molda um falloff \
                     que nada le). SE NAO FOR 1, PARE. ⚠ O field.box NAO tem Coupling anotado: o \
                     aviso vem PURAMENTE da binding de GPU que ele ja' declara (a DERIVACAO, o \
                     coracao da wave — 29 dos 35 nos de falloff sao cobertos assim, com ZERO \
                     anotacao). CLIQUE o badge do field.box: o app so' EXPLICA (toast: precisa de \
                     uma forca/deformer) e SELECIONA — NADA muda no grafo (Offer, sem cura \
                     canonica; a lei do ADR-0155 de nunca adivinhar). Depois CLIQUE o chip 'Node \
                     Help' (o icone de ajuda, ULTIMO da barra do grafo, no canto inferior \
                     esquerdo): o badge SOME (o sistema inteiro desliga — a liberdade do \
                     artista). Clique de novo: ele VOLTA. Para curar de verdade, ligue uma \
                     force.wind DEPOIS do field.box — o badge some (a forca le o falloff)."
                );
            }
            // =5, frame 3: o AVISO de requisito-a-montante (2a) — um `motion.bend`
            // ligado ao output com NADA alimentando-o. O deformer LÊ `P` (para dobrar
            // pontos) mas não tem stream nenhum — é silenciosamente um no-op (a mesma
            // classe de erro do ADR-0155, agora no eixo do que o nó PRECISA a montante).
            // ⚠ O aviso vem PURAMENTE da binding de GPU que o bend já declara (a mesma
            // DERIVAÇÃO da wave — ZERO anotação no nó). O bend ganha o pip ⚠; clicar
            // EXPLICA (precisa de uma fonte de pontos — grid/emitter) + seleciona, nunca
            // adivinha (Offer). Ligar uma `motion.grid` na entrada dele CURA (some o badge).
            (5, 3) => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let (bend, out) = {
                    let g = &mut gfx.motion.doc.graph;
                    let bend = g.add_node("motion.bend");
                    let out = g.add_node("motion.output");
                    g.set_pos(bend, Pos { x: 0.0, y: -200.0 });
                    g.set_pos(
                        out,
                        Pos {
                            x: 300.0,
                            y: -200.0,
                        },
                    );
                    g.connect(Edge {
                        from: (bend, 0),
                        to: (out, 0),
                        delayed: false,
                    })
                    .expect("bend -> output");
                    (bend, out)
                };
                gfx.motion.sinks.push(out);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                ph2d_panel_motion_graph::request_graph_selection(vec![bend.0]);
                let needy =
                    ph2d_motion_diagnose::diagnose(&gfx.motion.doc.graph, &gfx.motion.registry)
                        .iter()
                        .filter(|d| d.deficit == ph2d_motion_diagnose::Deficit::MissingSource("P"))
                        .count();
                eprintln!(
                    "[autofix smoke =5] montei `motion.bend -> output` com NADA alimentando o \
                     bend: {needy} no(s) sem fonte de pontos marcado(s) (esperado 1 — o bend le \
                     P mas nao tem stream para dobrar). SE NAO FOR 1, PARE. ⚠ O bend NAO tem \
                     Coupling anotado: o aviso vem PURAMENTE da binding de GPU que ele ja' \
                     declara (a mesma DERIVACAO da wave, ZERO anotacao). CLIQUE o badge do bend: \
                     o app so' EXPLICA (toast: precisa de uma fonte de pontos — Grid/Emitter) e \
                     SELECIONA — NADA muda no grafo (Offer, sem cura canonica; a lei do ADR-0155 \
                     de nunca adivinhar). Para curar de verdade, ligue uma `motion.grid` na \
                     entrada do bend — o badge some (o bend passa a ter pontos)."
                );
            }
            // =6, frame 3: o aviso de PORTA obrigatória (2b) — um `motion.duplicator` com
            // a entrada `shape` ligada numa forma DE VERDADE (o no' "Shape",
            // `source.shape`, uma estrela) mas a `points` VAZIA. ⚠ A `shape` diz O QUE
            // carimbar, a `points` diz ONDE: com um grid na `shape` a demo mentia (grid
            // sao PONTOS, nao forma), entao aqui a `shape` recebe a forma e o grid fica
            // SOLTO para o artista ligar em `points`. Sem `points` o duplicator PASSA a
            // forma adiante — uma estrela nitida na origem RENDERIZA (nao ha' "nao
            // renderiza": o duplicator aceita a Shape) — e pinta o ⚠, porque nao ha' onde
            // multiplicar. Diferente do 4/5 (derivados), este vem por DECLARACAO opt-in: o
            // no' declara `["shape","points"]` obrigatorias (required-vs-opcional e'
            // semantico — o `forces` do integrate e' OPCIONAL, os do duplicator nao). O
            // duplicator ganha o pip ⚠; clicar EXPLICA (precisa de um stream na porta
            // 'points') + seleciona, nunca adivinha (Offer). Ligar o grid em `points` CURA
            // (a estrela vira 16 copias, o ⚠ some).
            (6, 3) => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let (dup, out) = {
                    let g = &mut gfx.motion.doc.graph;
                    // A porta `shape` recebe uma forma DE VERDADE (o no' "Shape",
                    // `source.shape`) — nao um grid. O grid entra na cena mas fica SOLTO,
                    // pronto para o artista ligar em `points`.
                    let shape = g.add_node("source.shape");
                    let grid = g.add_node("motion.grid");
                    let dup = g.add_node("motion.duplicator");
                    let out = g.add_node("motion.output");
                    g.set_param(shape, "kind", 5.0); // Star
                    g.set_param(shape, "size", 0.4);
                    g.set_param(grid, "rows", 4.0);
                    g.set_param(grid, "cols", 4.0);
                    g.set_pos(
                        shape,
                        Pos {
                            x: -220.0,
                            y: -200.0,
                        },
                    );
                    g.set_pos(
                        grid,
                        Pos {
                            x: -220.0,
                            y: -40.0,
                        },
                    );
                    g.set_pos(dup, Pos { x: 40.0, y: -200.0 });
                    g.set_pos(
                        out,
                        Pos {
                            x: 300.0,
                            y: -200.0,
                        },
                    );
                    g.connect(Edge {
                        from: (shape, 0),
                        to: (dup, 0), // Shape -> duplicator.shape (porta 0)
                        delayed: false,
                    })
                    .expect("shape -> dup.shape");
                    g.connect(Edge {
                        from: (dup, 0),
                        to: (out, 0),
                        delayed: false,
                    })
                    .expect("dup -> output");
                    // `points` (porta 1) deixada VAZIA — o grid fica solto para o artista
                    // ligar e ver a estrela virar 16 copias.
                    let _ = grid;
                    (dup, out)
                };
                gfx.motion.sinks.push(out);
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
                ph2d_panel_motion_graph::request_graph_selection(vec![dup.0]);
                let missing =
                    ph2d_motion_diagnose::diagnose(&gfx.motion.doc.graph, &gfx.motion.registry)
                        .iter()
                        .filter(|d| {
                            matches!(d.deficit, ph2d_motion_diagnose::Deficit::MissingInput(_))
                        })
                        .count();
                eprintln!(
                    "[autofix smoke =6] montei `Shape (source.shape, estrela) -> duplicator.shape \
                     -> output` com a porta `points` VAZIA (o grid fica SOLTO ao lado): \
                     {missing} porta(s) obrigatoria(s) faltando marcada(s) (esperado 1 — falta \
                     `points`). SE NAO FOR 1, PARE. ⚠ A ESTRELA RENDERIZA (passthrough: sem \
                     `points` o duplicator PASSA a forma adiante) — nao ha' 'nao renderiza', o \
                     duplicator ACEITA a Shape; falta e' ONDE multiplicar. Diferente dos avisos \
                     4/5 (derivados), este vem por DECLARACAO opt-in: o duplicator declara \
                     [shape, points] obrigatorias, porque required-vs-opcional e' SEMANTICO (o \
                     `forces` do integrate e' opcional). CLIQUE o badge do duplicator: o app so' \
                     EXPLICA (toast: precisa de um stream na porta 'points') e SELECIONA — NADA \
                     muda no grafo (Offer, sem cura canonica; a lei do ADR-0155 de nunca \
                     adivinhar). Ligue o grid (solto) na porta `points`: a estrela vira 16 copias \
                     e o badge some."
                );
            }
            _ => {}
        }
    }
}
