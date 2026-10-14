//! O formato dos passos do Motion, o censo das intenções (gate 4 do plano para o Motion), as recusas
//! e as rotas dos params (fundo, cartão).

use super::passo::{Alvo, Escolha, Escrita, Intencao, IntencaoDeParam, Nomeado, Passo, Toque};
use super::tocar::{ErroDaMacro, Modo, TocadorDoMotion};
use super::{Destino, inicial};
use ph2d_nodegraph::graph::{Edge, NodeId};

/// Acrescenta um campo de etiqueta `50` que nenhum leitor conhece.
fn com_campo_novo(mut b: Vec<u8>) -> Vec<u8> {
    let zero = b.pop();
    assert_eq!(zero, Some(0), "um registo acaba na etiqueta 0");
    b.extend_from_slice(&[50, 1, b'x', 0]);
    b
}

fn passos_de_amostra() -> Vec<Passo> {
    let a = Alvo::Existente(3);
    let c = Alvo::Criado(2);
    let mut v = vec![
        Passo::Params {
            destino: Destino::No,
            intencoes: vec![
                IntencaoDeParam::SetParam {
                    node: a,
                    param: "size".into(),
                    value: 0.35,
                },
                IntencaoDeParam::SetTextParam {
                    node: c,
                    param: "rules".into(),
                    value: "F=F+F".into(),
                },
                IntencaoDeParam::ResetParam {
                    node: a,
                    param: "size".into(),
                },
            ],
        },
        Passo::Params {
            destino: Destino::Fundo(4),
            intencoes: vec![],
        },
        Passo::Params {
            destino: Destino::Cartao(9),
            intencoes: vec![],
        },
        Passo::Escritas(vec![
            Escrita::Valor {
                node: a,
                param: "width".into(),
                valor: Some(1.5),
            },
            Escrita::Valor {
                node: c,
                param: "width".into(),
                valor: None,
            },
            Escrita::Texto {
                node: a,
                param: "path".into(),
                valor: Some("Path 1".into()),
            },
            Escrita::Texto {
                node: c,
                param: "path".into(),
                valor: None,
            },
        ]),
        Passo::Documento {
            texto: "v5\n".into(),
            proximo_id: 12,
            arestas: vec![Edge {
                from: (NodeId(3), 0),
                to: (NodeId(12), 1),
                delayed: true,
            }],
        },
        Passo::Toque(Toque {
            editou: true,
            sujou: false,
        }),
        Passo::Toque(Toque {
            editou: false,
            sujou: true,
        }),
        Passo::Quadro {
            tempo: 1.25,
            a_tocar: true,
            dt: 1.0 / 60.0,
            sujou: false,
        },
        Passo::Recusada("uma porta".into()),
    ];
    #[cfg(feature = "panel-motion-graph")]
    v.push(Passo::Grafo {
        nivel: Some(2),
        ajuda: true,
        intencoes: censo::amostras()
            .iter()
            .filter_map(
                |i| match super::intencao::decide(i, censo::BASE, &censo::forma(), true) {
                    super::intencao::Decisao::Grava(x) => Some(x),
                    _ => None,
                },
            )
            .collect(),
    });
    v.push(Passo::Grafo {
        nivel: None,
        ajuda: false,
        intencoes: vec![
            Intencao::Rename {
                alvo: Nomeado::Fundo(1),
                nome: "Céu".into(),
            },
            Intencao::PickSelection {
                node: c,
                param: "path".into(),
                escolha: Escolha::SemArco,
            },
        ],
    });
    v
}

#[test]
fn cada_passo_vai_e_volta_ao_bit_e_salta_o_campo_que_nao_conhece() {
    for p in passos_de_amostra() {
        let b = p.codifica();
        assert_eq!(Passo::descodifica(&b), Ok(p.clone()), "ida e volta: {p:?}");
        assert_eq!(
            Passo::descodifica(&com_campo_novo(b)),
            Ok(p.clone()),
            "um campo novo salta-se: {p:?}"
        );
    }
}

#[test]
fn cada_recusa_tem_frase_na_tabela() {
    for k in [super::COLAR_DE_ANTES, super::DOCUMENTO_SEM_TEXTO] {
        assert_ne!(ph2d_i18n::tr(k), k, "a recusa {k} não tem frase na tabela");
    }
}

#[test]
fn um_alvo_guarda_a_ordem_de_criacao() {
    assert_eq!(Alvo::de_no(3, 10), Alvo::Existente(3));
    assert_eq!(Alvo::de_no(12, 10), Alvo::Criado(2));
    assert_eq!(
        Alvo::Criado(2).no(20),
        Some(22),
        "noutro documento, a partir do next_id dele"
    );
    assert_eq!(Alvo::Grupo(4).no(10), None);
}

#[test]
fn o_que_este_leitor_nao_conhece_para_o_tocador_em_voz_alta() {
    let m = super::estado_de_teste();
    let ini = inicial::captura(&m);
    let mut t = TocadorDoMotion::novo(&ini, Modo::Fiel).expect("o inicial lê-se");
    let mut r = ph2d_macro::rec::Record::new();
    r.varint(1, 99);
    assert_eq!(
        t.aplica(&r.finish(), None),
        Err(ErroDaMacro::PassoDesconhecido(99))
    );
    let p = Passo::Recusada("a porta".into());
    assert_eq!(
        t.aplica(&p.codifica(), None),
        Err(ErroDaMacro::PortaRecusada("a porta".into()))
    );
    let p = Passo::Params {
        destino: Destino::No,
        intencoes: vec![IntencaoDeParam::Desconhecida(77)],
    };
    assert_eq!(
        t.aplica(&p.codifica(), None),
        Err(ErroDaMacro::IntencaoDesconhecida(77))
    );
}

#[test]
fn o_estado_inicial_vai_e_volta() {
    let mut m = super::estado_de_teste();
    m.default_uv_rect = [0.25, 0.0, 0.75, 1.0];
    m.default_size = [2.0, 0.5];
    let a = m.doc.graph.add_node("source.shape");
    let b = m.doc.graph.add_node("motion.output");
    m.doc.graph.remove_node(b);
    let ini = inicial::le(&inicial::captura(&m)).expect("lê-se");
    assert_eq!(ini.doc.to_text(), m.doc.to_text());
    assert_eq!(
        ini.base,
        m.doc.graph.next_id(),
        "o next_id que o texto não guarda"
    );
    assert_ne!(
        ini.base,
        a.0 + 1,
        "a fixture tem um id apagado acima do maior vivo"
    );
    let p = inicial::palco(&ini);
    assert_eq!(p.doc, m.doc, "o palco é o documento ao armar, ao bit");
    assert_eq!(
        (p.default_uv_rect, p.default_size),
        (m.default_uv_rect, m.default_size)
    );
    assert_eq!(p.pump.a_lei(), m.pump.a_lei());
}

#[cfg(feature = "panel-motion-graph")]
pub(super) mod censo {
    //! **O censo das intenções** (gate 4 do plano, para o Motion).

    use super::super::intencao::{Decisao, decide, para_grafo};
    use super::super::passo::Intencao;
    use super::super::tocar::Nomes;
    use crate::motion_state::FormaEscolhida;
    use ph2d_panel_motion_graph::{GraphIntent as G, RenameTarget, SUBGRAPH_VIEW_TAG};

    pub(crate) const BASE: u32 = 10;

    pub(crate) fn forma() -> FormaEscolhida {
        FormaEscolhida::Nome("Path 1".into())
    }

    /// Uma amostra de CADA variante: um nó que existia (`3`), um criado a gravar (`12`), um cartão.
    pub(crate) fn amostras() -> Vec<G> {
        let (e, c, g) = (3, 12, 5 | SUBGRAPH_VIEW_TAG);
        vec![
            G::BeginDrag,
            G::MoveNodes {
                nodes: vec![e, g],
                dx: 1.5,
                dy: -2.0,
            },
            G::EndDrag,
            G::DriveParam {
                from_node: g,
                from_port: 1,
                to_node: c,
                param: "size",
            },
            G::DropRefused {
                from_node: e,
                from_port: 0,
                to_node: g,
            },
            G::SetParam {
                node: c,
                param: "size",
                value: 0.5,
            },
            G::PickFile {
                node: c,
                param: "size",
            },
            G::PickSelection {
                node: c,
                param: "size",
            },
            G::StepChoice {
                node: c,
                param: "size",
                delta: -1,
            },
            G::PickChoice {
                node: c,
                param: "size",
                index: 2,
            },
            G::SetTextParam {
                node: c,
                param: "size",
                value: "x".into(),
            },
            G::ToggleParamSection {
                node: c,
                section: "Shape",
            },
            G::Connect {
                from_node: e,
                from_port: 0,
                to_node: g,
                to_port: 2,
            },
            G::Disconnect {
                to_node: c,
                to_port: 1,
            },
            G::OpenLibrary {
                x: 1.0,
                y: 2.0,
                connect_from: Some((e, 0)),
                connect_to: None,
                splice: None,
                compatible: vec!["source.shape"],
            },
            G::AddNode {
                type_name: "source.shape",
                x: 10.0,
                y: 20.0,
            },
            G::DeleteSelection {
                nodes: vec![e, c, g],
            },
            G::SetSplit { t: 0.5 },
            G::SetSplitVertical { vertical: true },
            G::TogglePlay,
            G::SetNodeHelp(false),
            G::AddBackdrop {
                x: 1.0,
                y: 2.0,
                w: 3.0,
                h: 4.0,
            },
            G::MoveBackdrop {
                id: 2,
                dx: 1.0,
                dy: 1.0,
            },
            G::ResizeBackdrop {
                id: 2,
                left: true,
                dx: 1.0,
                dy: 0.0,
            },
            G::DeleteBackdrop { id: 2 },
            G::Rename {
                target: RenameTarget::Node(c),
                name: "Vento".into(),
            },
            G::Rename {
                target: RenameTarget::Subgraph(5),
                name: "Grupo".into(),
            },
            G::Rename {
                target: RenameTarget::Backdrop(2),
                name: "Céu".into(),
            },
            G::SetBackdropColor { id: 2, color: 6 },
            G::DuplicateSelection { nodes: vec![c, g] },
            G::SmartConnect {
                from_node: e,
                from_port: 0,
                to_type: "motion.output",
                x: 0.0,
                y: 0.0,
            },
            G::SwapInChain {
                a: e,
                b: c,
                back_dx: 1.0,
                back_dy: 2.0,
            },
            G::SpliceExistingIntoWire {
                node: e,
                to_node: c,
                to_port: 0,
            },
            G::SmartConnectBack {
                to_node: c,
                to_port: 1,
                to_type: "source.shape",
                x: 3.0,
                y: 4.0,
            },
            G::SetProbe { node: Some(c) },
            G::CutWires {
                targets: vec![(c, 0), (g, 1)],
            },
            G::MoveWireEnd {
                from_node: e,
                from_port: 0,
                old_to_node: c,
                old_to_port: 1,
                new_to: Some((g, 0)),
            },
            G::MoveWireEnd {
                from_node: e,
                from_port: 0,
                old_to_node: c,
                old_to_port: 1,
                new_to: None,
            },
            G::GroupSelection { nodes: vec![e, c] },
            G::Ungroup { id: 5 },
            G::EnterSubgraph { id: 5 },
            G::GoToLevel { level: Some(5) },
            G::GoToLevel { level: None },
            G::SpliceReroute {
                to_node: c,
                to_port: 0,
                x: 1.0,
                y: 1.0,
            },
            G::SpliceNode {
                to_node: c,
                to_port: 0,
                to_type: "motion.output",
                x: 1.0,
                y: 1.0,
            },
            G::ArrangeLayout,
            G::SetBypass {
                nodes: vec![e, g],
                on: true,
            },
            G::CopySelection { nodes: vec![e, g] },
            G::Paste,
            G::FixInert { node: c },
        ]
    }

    fn nome(i: &G) -> String {
        format!("{i:?}")
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect()
    }

    /// As variantes do enum, lidas do FONTE do painel.
    fn variantes_no_fonte() -> std::collections::BTreeSet<String> {
        let p = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../ph2d-panel-motion-graph/src/snapshot_intent.rs"
        );
        let fonte = std::fs::read_to_string(p).expect("o fonte do painel lê-se");
        let corpo = fonte
            .split_once("pub enum GraphIntent {")
            .expect("o enum está lá")
            .1
            .split_once("\n}\n")
            .expect("o enum fecha")
            .0;
        corpo
            .lines()
            .filter_map(|l| {
                let r = l.strip_prefix("    ")?;
                let nome: String = r.chars().take_while(char::is_ascii_alphanumeric).collect();
                nome.chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_uppercase())
                    .then_some(nome)
            })
            .collect()
    }

    #[test]
    fn a_amostra_tem_cada_variante_do_painel() {
        let na_amostra: std::collections::BTreeSet<String> = amostras().iter().map(nome).collect();
        assert_eq!(na_amostra, variantes_no_fonte());
    }

    #[test]
    fn cada_intencao_gravada_volta_a_mesma_intencao_do_painel() {
        let m = crate::motion_state::MotionState::new();
        let nomes = Nomes::de(&m.registry);
        for i in amostras() {
            if let Decisao::Grava(x) = decide(&i, BASE, &forma(), true) {
                let lida = Intencao::descodifica(&x.codifica()).expect("lê-se");
                assert_eq!(lida, x, "ida e volta do formato: {i:?}");
                assert_eq!(
                    para_grafo(&lida, BASE, &nomes).as_ref(),
                    Ok(&i),
                    "de volta ao painel: {i:?}"
                );
            }
        }
    }

    #[test]
    fn o_que_nao_se_grava_nao_decide_a_cura_do_lote() {
        // O dreno cura o lote se ele CONSTRÓI e não DESTRÓI: uma intenção que a gravação deixa de fora
        // (resolvida nos params, ou de vista) não pode pesar nessa conta, senão o palco curaria outro lote.
        for i in amostras() {
            if matches!(
                decide(&i, BASE, &forma(), true),
                Decisao::Encaminha | Decisao::Vista
            ) {
                assert!(
                    !crate::motion_bridge::is_constructive(&i)
                        && !crate::motion_bridge::is_destructive(&i),
                    "{i:?} pesa na cura e não se grava"
                );
            }
        }
    }

    #[test]
    fn colar_o_que_se_copiou_antes_de_gravar_recusa_se() {
        assert_eq!(
            decide(&G::Paste, BASE, &forma(), false),
            Decisao::Recusa(super::super::COLAR_DE_ANTES)
        );
        assert!(matches!(
            decide(&G::Paste, BASE, &forma(), true),
            Decisao::Grava(Intencao::Paste)
        ));
    }

    #[test]
    fn a_decisao_das_intencoes_nao_tem_curinga() {
        let fonte = include_str!("intencao.rs");
        let corpo = fonte
            .split_once("pub fn decide(")
            .expect("a decisão")
            .1
            .split_once("pub fn para_grafo(")
            .expect("a volta")
            .0;
        assert!(
            !corpo.contains("_ =>") && !corpo.contains("_ if"),
            "um curinga na decisão deixaria uma intenção nova passar sem decisão"
        );
    }
}

#[cfg(feature = "panel-motion-graph")]
mod rotas {
    //! As edições que o dreno de params desvia pela SELECÇÃO (um fundo, um cartão): a gravação leva o
    //! destino, o palco aplica-o sem selecção nenhuma.

    use super::super::passo::Passo;
    use super::super::tocar::{Modo, TocadorDoMotion};
    use crate::motion_state::FormaEscolhida;
    use ph2d_panel_motion_graph::{GraphIntent, push_intent, set_graph_backdrop_selection};

    fn quadro(m: &mut crate::motion_state::MotionState) {
        let mut toasts = ph2d_editor_core::ToastQueue::default();
        let store = ph2d_editor_core::interaction::WidgetStore::default();
        let mut split = ph2d_editor_core::screens::layout::CenterSplit::None;
        let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
        crate::motion_bridge::apply_graph_intents(m, &mut ph, &mut toasts, &mut split);
        crate::motion_bridge::params::apply_param_edits_for_tests(m, &store, &mut toasts);
        crate::motion_bridge::quadro::o_quadro(
            m,
            None,
            &ph,
            1.0 / 60.0,
            crate::motion_bridge::quadro::MaoDoEditor::do_painel(),
        );
    }

    fn toca(inicial: &[u8], cargas: &[Vec<u8>]) -> TocadorDoMotion {
        let mut t = TocadorDoMotion::novo(inicial, Modo::Fiel).expect("o inicial lê-se");
        for c in cargas {
            t.aplica(c, None).expect("o passo aplica-se");
        }
        t
    }

    #[test]
    fn o_titulo_de_um_fundo_escolhido_chega_ao_palco_sem_seleccao() {
        let mut m = super::super::estado_de_teste();
        let ini = m.gravacao_arma().expect("arma");
        let mut cargas = Vec::new();
        push_intent(GraphIntent::AddBackdrop {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 80.0,
        });
        quadro(&mut m);
        cargas.extend(m.gravacao_passos());
        let id = m.doc.backdrops[0].id;
        set_graph_backdrop_selection(Some(id));
        crate::push_param_intent(crate::MotionParamIntent::SetTextParam {
            node: 0,
            param: "title",
            value: "Céu".into(),
        });
        quadro(&mut m);
        set_graph_backdrop_selection(None);
        cargas.extend(m.gravacao_passos());
        assert!(
            cargas.iter().any(|c| matches!(
                Passo::descodifica(c),
                Ok(Passo::Params {
                    destino: super::super::Destino::Fundo(_),
                    ..
                })
            )),
            "a gravação leva o destino"
        );
        let t = toca(&ini, &cargas);
        assert_eq!(t.palco().doc.backdrops[0].title, "Céu");
        assert_eq!(t.palco().doc, m.doc);
    }

    #[test]
    fn um_no_criado_dentro_de_um_grupo_nasce_no_grupo_do_palco() {
        let mut m = super::super::estado_de_teste();
        let a = m.doc.graph.add_node("source.shape");
        let b = m.doc.graph.add_node("motion.output");
        push_intent(GraphIntent::GroupSelection {
            nodes: vec![a.0, b.0],
        });
        quadro(&mut m);
        let sid = m.doc.subgraphs[0].id;
        push_intent(GraphIntent::EnterSubgraph { id: sid });
        quadro(&mut m);
        assert_eq!(
            m.level,
            Some(sid),
            "o artista está dentro do grupo ao armar"
        );
        let ini = m.gravacao_arma().expect("arma");
        push_intent(GraphIntent::AddNode {
            type_name: "force.wind",
            x: 0.0,
            y: 0.0,
        });
        quadro(&mut m);
        let cargas = m.gravacao_passos();
        let novo = m.doc.graph.nodes().last().expect("o nó novo").id;
        assert!(
            ph2d_motion_doc::subgraph::member_nodes_deep(&m.doc.subgraphs, &m.doc.members, sid)
                .contains(&novo),
            "CONTROLO: na sessão o nó novo nasce no grupo"
        );
        let t = toca(&ini, &cargas);
        assert_eq!(t.palco().doc, m.doc, "no palco também");
    }

    #[test]
    fn copiar_e_colar_dentro_da_gravacao_toca() {
        let mut m = super::super::estado_de_teste();
        let a = m.doc.graph.add_node("source.shape");
        let ini = m.gravacao_arma().expect("arma");
        push_intent(GraphIntent::CopySelection { nodes: vec![a.0] });
        quadro(&mut m);
        push_intent(GraphIntent::Paste);
        quadro(&mut m);
        let cargas = m.gravacao_passos();
        assert_eq!(
            m.doc.graph.nodes().len(),
            2,
            "CONTROLO: a colagem criou o nó"
        );
        assert!(
            !cargas
                .iter()
                .any(|c| matches!(Passo::descodifica(c), Ok(Passo::Recusada(_)))),
            "a cópia foi gravada, logo a colagem também"
        );
        let t = toca(&ini, &cargas);
        assert_eq!(t.palco().doc, m.doc);
    }

    #[test]
    fn a_ajuda_dos_nos_decide_a_cura_no_palco_como_na_sessao() {
        // `grid → wind` e ligar o vento à saída: com a ajuda ligada o lote cura-se (um integrador).
        let mut m = super::super::estado_de_teste();
        m.pump.define_a_lei(false);
        let grid = m.doc.graph.add_node("motion.grid");
        let wind = m.doc.graph.add_node("force.wind");
        let out = m.doc.graph.add_node("motion.output");
        m.doc
            .graph
            .connect(ph2d_nodegraph::graph::Edge {
                from: (grid, 0),
                to: (wind, 0),
                delayed: false,
            })
            .expect("liga");
        assert!(m.node_help_enabled, "a ajuda nasce ligada");
        let ini = m.gravacao_arma().expect("arma");
        push_intent(GraphIntent::Connect {
            from_node: wind.0,
            from_port: 0,
            to_node: out.0,
            to_port: 0,
        });
        quadro(&mut m);
        let cargas = m.gravacao_passos();
        assert!(
            m.doc
                .graph
                .nodes()
                .iter()
                .any(|n| n.type_name == "motion.integrate"),
            "CONTROLO: na sessão o lote curou-se"
        );
        let t = toca(&ini, &cargas);
        assert_eq!(t.palco().doc, m.doc, "no palco também");
    }

    #[test]
    fn a_forma_que_o_botao_viu_chega_ao_palco() {
        let mut m = super::super::estado_de_teste();
        let n = m.doc.graph.add_node("source.shape");
        let ini = m.gravacao_arma().expect("arma");
        m.selected_shape = FormaEscolhida::Nome("Path 7".into());
        push_intent(GraphIntent::PickSelection {
            node: n.0,
            param: "size",
        });
        quadro(&mut m);
        let cargas = m.gravacao_passos();
        m.selected_shape = FormaEscolhida::Nada;
        let t = toca(&ini, &cargas);
        assert_eq!(
            t.palco().doc,
            m.doc,
            "o palco escreveu o nome que o botão viu"
        );
        assert_eq!(
            t.palco()
                .doc
                .graph
                .node_text_param_overrides(n)
                .and_then(|p| p.get("size"))
                .map(String::as_str),
            Some("Path 7")
        );
    }

    #[test]
    fn colar_sem_copia_gravada_fica_na_gravacao_como_recusa() {
        let mut m = super::super::estado_de_teste();
        let _ = m.gravacao_arma().expect("arma");
        push_intent(GraphIntent::Paste);
        quadro(&mut m);
        let cargas = m.gravacao_passos();
        assert!(cargas.iter().any(|c| Passo::descodifica(c)
            == Ok(Passo::Recusada(super::super::COLAR_DE_ANTES.to_owned()))));
    }

    #[test]
    fn o_fiel_para_quando_a_bomba_chega_ao_quadro_de_outra_forma() {
        let mut m = super::super::estado_de_teste();
        let ini = m.gravacao_arma().expect("arma");
        quadro(&mut m);
        let cargas: Vec<Vec<u8>> = m
            .gravacao_passos()
            .into_iter()
            .map(|c| match Passo::descodifica(&c) {
                Ok(Passo::Quadro {
                    tempo,
                    a_tocar,
                    dt,
                    sujou,
                }) => Passo::Quadro {
                    tempo,
                    a_tocar,
                    dt,
                    sujou: !sujou,
                }
                .codifica(),
                _ => c,
            })
            .collect();
        let mut t = TocadorDoMotion::novo(&ini, Modo::Fiel).expect("o inicial lê-se");
        let r: Result<(), _> = cargas.iter().try_for_each(|c| t.aplica(c, None));
        assert_eq!(r, Err(super::super::tocar::ErroDaMacro::Divergiu));
    }
}
