//! ⭐ A aba «Collision Layers» (onda 9a, versão B do dono 07/11), pelo gesto real: o ponteiro no rect
//! pintado, o duplo clique, o Enter e o Esc. A área é a da aba central (a família da `ph2d-app-physics`
//! di-la a cada quadro; aqui, fixa).

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_physics::state::{PhysicsPanelState, PhysicsSnapshot};
use ph2d_panel_physics::{PhysicsIntent, PhysicsPanel, aba, cards, drain_intents, ids};
use ph2d_physics_ecs::{LayerMatrix, LayerNames, MAX_LAYERS, PhysicsSettings};
use ph2d_ui_testkit::MockPanelHost;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};
/// A área da aba central, dentro da janela.
const AREA: Rect = Rect {
    x: 300.0,
    y: 60.0,
    w: 1000.0,
    h: 900.0,
};
/// As cinco camadas da maquete B (0..4) à vista.
const CINCO: u32 = 0b1_1111;

struct Mesa {
    host: MockPanelHost,
    st: PhysicsPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn nova(nomes: LayerNames, classica: bool) -> Self {
        cards::force_cards_on_this_thread(Some(!classica));
        ph2d_panel_physics::set_current_physics(Some(PhysicsSnapshot::default()));
        ph2d_panel_physics::set_layer_names(nomes);
        ph2d_panel_physics::set_visible_layers(CINCO);
        ph2d_panel_physics::set_layer_counts([0; MAX_LAYERS]);
        aba::fecha();
        aba::poe_area_fixa(AREA);
        let _ = drain_intents();
        let mut m = Self {
            host: MockPanelHost::with_panel::<PhysicsPanel>(),
            st: PhysicsPanelState,
            rects: Vec::new(),
        };
        m.host.set_collapsed(ids::PHYSICS_SEC_LAYERS, false);
        m.pinta();
        m
    }

    fn pinta(&mut self) {
        for _ in 0..30 {
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<PhysicsPanel>(st, JANELA),
            );
        }
    }

    fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    }

    fn centro(&self, id: NodeId) -> (f32, f32) {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("{id:?} não pintado"));
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }

    fn aplica(&mut self, ev: Vec<WidgetEvent>) -> Vec<PhysicsIntent> {
        for e in ev {
            let _ = self.host.apply_panel_event::<PhysicsPanel>(&mut self.st, e);
        }
        drain_intents()
    }

    fn clica(&mut self, id: NodeId) -> Vec<PhysicsIntent> {
        let (x, y) = self.centro(id);
        assert_eq!(self.host.hit_at(x, y), Some(id), "outro id no centro");
        let ev = self.host.click_at(x, y);
        self.aplica(ev)
    }

    fn duplo(&mut self, id: NodeId) {
        let (x, y) = self.centro(id);
        // O ponteiro acerta no rótulo em ecrã; o despacho promove o 2.º clique a duplo (como os testes do Flip).
        assert_eq!(
            self.host.hit_at(x, y),
            Some(id),
            "outro id no rótulo {:?}",
            self.rect(id)
        );
        let ev = vec![WidgetEvent::DoubleClick(id)];
        let _ = self.aplica(ev);
        self.pinta();
    }

    fn pairar(&mut self, x: f32, y: f32) {
        let _ = self.host.dispatch_pointer_event(PointerEvent {
            kind: PointerKind::Move,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 0.0,
            timestamp_ns: 0,
        });
    }
}

fn celula(a: usize, b: usize) -> NodeId {
    ids::PHYSICS_LAYER_CELL[ids::physics_layer_cell_index(a, b)]
}

fn mundo_de(intents: &[PhysicsIntent]) -> LayerMatrix {
    intents
        .iter()
        .rev()
        .find_map(|i| match i {
            PhysicsIntent::SetSettings(s, _) => Some(LayerMatrix::from_rows(s.layer_matrix)),
            _ => None,
        })
        .expect("a célula escreve")
}

fn abre(mesa: &mut Mesa) {
    aba::abre(None);
    aba::poe_area_fixa(AREA);
    mesa.pinta();
}

fn nomes_b() -> LayerNames {
    let mut n = LayerNames::default();
    for (i, s) in ["Ground", "Player", "Enemy", "Pickup", "Player Shots"]
        .iter()
        .enumerate()
    {
        n.0[i] = (*s).into();
    }
    n
}

/// A porta do painel abre a aba (e pede para vir à frente); as células das cinco camadas visíveis cabem na
/// área, e só elas; a célula alta (4, 1) escreve os dois lados do par e só ele; fechar tira as células.
/// ⭐ Onda 9b — a camada aberta, pelo clique real: a caixinha «Bumps into» de Enemy (na camada
/// Player) escreve os DOIS lados do par (1, 2) e só ele; o botão do objeto seleciona ESSE objeto; «Select
/// all» seleciona todos (o 1.º troca, os outros juntam-se); com muitos objetos, «+N more» abre a fila.
/// ⭐ Onda 9b — «Remove», pelo clique real: na Default não faz nada; numa camada vazia sai logo (os
/// objetos vão para a 0 e a linha volta a bater em tudo); com objetos pergunta, «Cancel» desiste e o
/// destino escolhido é o que «Move and remove» manda.
#[test]
fn remove_pergunta_para_onde_vao_os_objetos_e_a_default_fica() {
    let mut mesa = Mesa::nova(nomes_b(), false);
    let mut contagens = [0; MAX_LAYERS];
    contagens[2] = 3;
    ph2d_panel_physics::set_layer_counts(contagens);
    ph2d_panel_physics::set_layer_objects(
        ["Slime", "Slime 2", "Bat"]
            .iter()
            .enumerate()
            .map(|(k, n)| (2, k as u64 + 1, (*n).into()))
            .collect(),
    );
    // Default: desligado.
    aba::abre(Some(0));
    aba::poe_area_fixa(AREA);
    mesa.pinta();
    assert!(mesa.clica(ids::PHYSICS_LAYER_REMOVE).is_empty());
    assert!(mesa.host.drained_actions().is_empty(), "a Default não sai");
    // Vazia (Pickup, 3): sai logo, e a linha volta ao padrão no mesmo quadro.
    let _ = mesa.clica(ids::PHYSICS_LAYER_LIST[3]);
    mesa.pinta();
    let m = mundo_de(&mesa.clica(ids::PHYSICS_LAYER_REMOVE));
    assert_eq!(
        mesa.host.drained_actions(),
        [EditorAction::LayerRemove { layer: 3, to: 0 }]
    );
    assert!(
        (0..MAX_LAYERS).all(|b| m.collides(3, b)),
        "a linha volta ao padrão"
    );
    assert_eq!(aba::camada_aberta(), 0);
    // Com objetos (Enemy, 2): pergunta; Cancel desiste.
    let _ = mesa.clica(ids::PHYSICS_LAYER_LIST[2]);
    mesa.pinta();
    assert!(mesa.clica(ids::PHYSICS_LAYER_REMOVE).is_empty());
    mesa.pinta();
    assert_eq!(aba::removendo(), Some(0));
    assert!(
        mesa.rect(ids::PHYSICS_LAYER_MOVE_TO[2]).is_none(),
        "não vai para si mesma"
    );
    let _ = mesa.clica(ids::PHYSICS_LAYER_MOVE_CANCEL);
    mesa.pinta();
    assert_eq!(aba::removendo(), None);
    assert!(mesa.host.drained_actions().is_empty(), "Cancel não escreve");
    // De novo: escolhe Ground e confirma.
    let _ = mesa.clica(ids::PHYSICS_LAYER_REMOVE);
    mesa.pinta();
    let _ = mesa.clica(ids::PHYSICS_LAYER_MOVE_TO[1]);
    mesa.pinta();
    let _ = mesa.clica(ids::PHYSICS_LAYER_MOVE_OK);
    assert_eq!(
        mesa.host.drained_actions(),
        [EditorAction::LayerRemove { layer: 2, to: 1 }],
        "os objetos vão para o destino escolhido"
    );
    assert_eq!(aba::camada_aberta(), 1);
    ph2d_panel_physics::set_layer_objects(Vec::new());
    aba::fecha();
}

#[test]
fn a_camada_aberta_marca_o_par_e_seleciona_os_objetos() {
    use ph2d_editor_core::action_bus::SelectModifier;
    let mut mesa = Mesa::nova(nomes_b(), false);
    ph2d_panel_physics::set_layer_objects(vec![
        (1, 0xA1, "Hero".into()),
        (1, 0xA2, "Hero 2".into()),
        (2, 0xB1, "Slime".into()),
    ]);
    aba::abre(Some(1));
    aba::poe_area_fixa(AREA);
    mesa.pinta();
    // A caixinha da Enemy na camada Player.
    let antes = LayerMatrix::from_rows(PhysicsSettings::default().layer_matrix);
    let depois = mundo_de(&mesa.clica(ids::PHYSICS_LAYER_BUMPS[2]));
    for a in 0..MAX_LAYERS {
        for b in 0..MAX_LAYERS {
            let par = (a, b) == (1, 2) || (a, b) == (2, 1);
            assert_eq!(
                depois.collides(a, b) != antes.collides(a, b),
                par,
                "a caixinha muda só o par (1, 2), nos dois lados: ({a}, {b})"
            );
        }
    }
    // A caixinha da camada que não é visível não existe.
    assert!(mesa.rect(ids::PHYSICS_LAYER_BUMPS[5]).is_none());
    let _ = mesa.host.drained_actions();
    // O 2.º botão é o «Hero 2».
    let _ = mesa.clica(ids::PHYSICS_LAYER_OBJECT[1]);
    assert_eq!(
        mesa.host.drained_actions(),
        [EditorAction::SelectSprite {
            entity_bits: 0xA2,
            modifier: SelectModifier::Replace
        }],
        "o botão seleciona o seu objeto"
    );
    assert!(
        mesa.rect(ids::PHYSICS_LAYER_OBJECT[2]).is_none(),
        "só os da camada"
    );
    let _ = mesa.clica(ids::PHYSICS_LAYER_SELECT_ALL);
    assert_eq!(
        mesa.host.drained_actions(),
        [
            EditorAction::SelectSprite {
                entity_bits: 0xA1,
                modifier: SelectModifier::Replace
            },
            EditorAction::SelectSprite {
                entity_bits: 0xA2,
                modifier: SelectModifier::Add
            }
        ],
        "Select all seleciona todos"
    );
    // Muitos objetos: «+N more» depois de três linhas, e o clique mostra a fila inteira.
    let muitos: Vec<(usize, u64, String)> = (0..60)
        .map(|k| (1, 0x1000 + k as u64, format!("Coin {k:02}")))
        .collect();
    ph2d_panel_physics::set_layer_objects(muitos);
    mesa.pinta();
    assert!(mesa.rect(ids::PHYSICS_LAYER_MORE).is_some(), "+N more");
    assert!(mesa.rect(ids::PHYSICS_LAYER_OBJECT[59]).is_none());
    let _ = mesa.clica(ids::PHYSICS_LAYER_MORE);
    mesa.pinta();
    assert!(
        mesa.rect(ids::PHYSICS_LAYER_OBJECT[59]).is_some(),
        "a fila inteira"
    );
    assert!(mesa.rect(ids::PHYSICS_LAYER_MORE).is_none());
    // Os cartões da direita têm o tamanho do conteúdo: a camada aberta acaba antes do fim da área.
    let todos = mesa
        .rect(ids::PHYSICS_LAYER_SELECT_ALL)
        .expect("Select all");
    assert!(todos.y + todos.h < AREA.y + AREA.h, "dentro da área");
    ph2d_panel_physics::set_layer_objects(Vec::new());
    aba::fecha();
}

#[test]
fn a_porta_abre_a_aba_so_com_as_visiveis_e_a_celula_alta_escreve_o_par() {
    let mut mesa = Mesa::nova(nomes_b(), false);
    assert!(mesa.rect(celula(0, 0)).is_none(), "a grelha saiu do painel");
    let _ = mesa.clica(ids::PHYSICS_MATRIX_OPEN);
    assert!(aba::aberta() && aba::tira_pedido(), "abre e vem à frente");
    aba::poe_area_fixa(AREA);
    mesa.pinta();
    for a in 0..MAX_LAYERS {
        for b in 0..=a {
            let r = mesa.rect(celula(a, b));
            if a < 5 {
                let r = r.unwrap_or_else(|| panic!("({a},{b}) não pintada"));
                assert!(
                    r.x >= AREA.x && r.y >= AREA.y && r.x + r.w <= AREA.x + AREA.w,
                    "({a},{b}) fora da aba: {r:?}"
                );
            } else {
                assert!(r.is_none(), "({a},{b}) não é visível e foi pintada");
            }
        }
    }
    let intents = mesa.clica(celula(4, 1));
    let cru = intents
        .iter()
        .rev()
        .find_map(|i| match i {
            PhysicsIntent::SetSettings(s, _) => Some(s.layer_matrix),
            _ => None,
        })
        .expect("a célula escreve");
    assert_eq!(cru[4] & 1 << 1, 0, "a metade 4→1");
    assert_eq!(cru[1] & 1 << 4, 0, "a metade espelho 1→4");
    let m = LayerMatrix::from_rows(cru);
    for a in 0..MAX_LAYERS {
        for b in 0..MAX_LAYERS {
            if (a, b) != (4, 1) && (a, b) != (1, 4) {
                assert!(m.collides(a, b), "({a},{b}) não devia mudar");
            }
        }
    }
    aba::fecha();
    mesa.pinta();
    assert!(mesa.rect(celula(4, 1)).is_none(), "fechada, nada se pinta");
}

/// O «Open in tab…» do Inspector abre a aba JÁ na camada do objeto, mesmo com o painel fechado: a linha da
/// lista fica selecionada, a linha da grade acesa e o título da camada aberta é o dela.
#[test]
fn o_inspector_abre_a_aba_na_camada_do_objeto() {
    let mut mesa = Mesa::nova(nomes_b(), false);
    mesa.host.set_panel_visible("physics", false);
    let mut contagens = [0; MAX_LAYERS];
    contagens[2] = 3;
    ph2d_panel_physics::set_layer_counts(contagens);
    ph2d_panel_physics::abre_a_matriz(&mut mesa.host, 2);
    assert!(!mesa.host.panel_visible("physics"), "o painel não abre");
    assert_eq!(aba::camada_aberta(), 2);
    aba::poe_area_fixa(AREA);
    mesa.pinta();
    let l = aba::lente();
    assert_eq!(l.linhas, [2], "a linha da camada do objeto acesa");
    assert_eq!(l.titulo.0, "Enemy · 3 objects");
    assert_eq!(l.titulo.1, "5 bump · 0 pass through");
    aba::fecha();
}

/// A lista diz cada camada visível com a contagem dela; clicar numa linha abre essa camada.
#[test]
fn a_lista_conta_os_objetos_e_o_clique_abre_a_camada() {
    let mut mesa = Mesa::nova(nomes_b(), false);
    ph2d_panel_physics::set_layer_counts({
        let mut c = [0; MAX_LAYERS];
        c[..5].copy_from_slice(&[6, 1, 3, 5, 2]);
        c
    });
    abre(&mut mesa);
    let l = aba::lente();
    let lidas: Vec<(String, u32)> = l.lista.iter().map(|(_, n, c)| (n.clone(), *c)).collect();
    assert_eq!(
        lidas,
        [
            ("Ground".to_owned(), 6),
            ("Player".to_owned(), 1),
            ("Enemy".to_owned(), 3),
            ("Pickup".to_owned(), 5),
            ("Player Shots".to_owned(), 2)
        ]
    );
    let _ = mesa.clica(ids::PHYSICS_LAYER_LIST[3]);
    assert_eq!(aba::camada_aberta(), 3);
    mesa.pinta();
    assert_eq!(aba::lente().titulo.0, "Pickup · 5 objects");
    aba::fecha();
}

/// Pairar sobre a célula (Player Shots × Player) em ECRÃ acende a linha e a coluna dela e muda a frase do pé
/// para o par por nome e o efeito; sem ponteiro, o pé diz o resumo.
#[test]
fn pairar_acende_linha_e_coluna_e_a_frase_do_pe_diz_o_par() {
    let mut mesa = Mesa::nova(nomes_b(), false);
    let mut m = LayerMatrix::from_rows(PhysicsSettings::default().layer_matrix);
    m.set(4, 1, false);
    let vivo = PhysicsSettings {
        layer_matrix: m.rows(),
        ..PhysicsSettings::default()
    };
    ph2d_panel_physics::set_current_physics(Some(PhysicsSnapshot {
        settings: vivo,
        vivo,
        ..PhysicsSnapshot::default()
    }));
    abre(&mut mesa);
    assert_eq!(
        aba::lente().pe,
        "15 pairs · 1 pass through. Double-click a name to rename it."
    );
    let (x, y) = mesa.centro(celula(4, 1));
    mesa.pairar(x, y);
    mesa.pinta();
    assert_eq!(
        mesa.host.store().button_state(celula(4, 1)),
        Some(ph2d_editor_core::widget::ButtonState::Hovered)
    );
    let l = aba::lente();
    assert_eq!(l.linhas, [4], "a linha do par acesa");
    assert_eq!(l.colunas, [1], "a coluna do par acesa");
    assert_eq!(
        l.pe,
        "Player Shots × Player: they pass through each other. Click to make them bump."
    );
    assert_eq!(
        mesa.host.store().tooltip_for(celula(4, 1)),
        None,
        "sem dica flutuante"
    );
    let (x, y) = mesa.centro(celula(2, 0));
    mesa.pairar(x, y);
    mesa.pinta();
    assert_eq!(
        aba::lente().pe,
        "Enemy × Ground: they bump into each other. Click to make them pass through."
    );
    aba::fecha();
}

/// Duplo clique no nome da lista e nos das duas bordas abre o campo ali; Enter grava UM pedido; Esc desiste
/// sem pedido; vazio grava vazio (volta ao padrão).
#[test]
fn duplo_clique_renomeia_na_lista_e_nas_bordas() {
    let mut mesa = Mesa::nova(nomes_b(), false);
    abre(&mut mesa);
    let _ = mesa.host.drained_actions();
    let enter = ph2d_editor_core::interaction::dispatch::keymap::KEY_ENTER;

    mesa.duplo(ids::PHYSICS_LAYER_LIST[2]);
    assert_eq!(aba::editando(), Some(2));
    let campo = ids::PHYSICS_LAYER_NAME[2];
    assert_eq!(mesa.host.store().focus_id(), Some(campo));
    assert_eq!(
        mesa.host.store().text(campo),
        Some("Enemy"),
        "abre com o nome"
    );
    let r = mesa.rect(campo).expect("o campo pintado na lista");
    assert!(
        r.w >= mesa.rect(ids::PHYSICS_LAYER_LIST[2]).expect("a linha").w,
        "o campo tem a largura da linha"
    );
    mesa.host.set_text(campo, "Bats");
    let _ = mesa.aplica(vec![WidgetEvent::TextChanged(campo)]);
    assert!(mesa.host.drained_actions().is_empty(), "escrever não grava");
    let ev = mesa.host.key_down(enter);
    let _ = mesa.aplica(ev);
    assert_eq!(
        mesa.host.drained_actions(),
        [EditorAction::LayerRename {
            layer: 2,
            name: "Bats".into()
        }],
        "Enter = um passo"
    );
    assert!(aba::editando().is_none() && aba::aberta());
    mesa.pinta();

    mesa.duplo(ids::PHYSICS_LAYER_ROW_LABEL[3]);
    assert_eq!(aba::editando(), Some(3));
    let r = mesa
        .rect(ids::PHYSICS_LAYER_NAME[3])
        .expect("o campo na grade");
    assert!(
        r.w > mesa
            .rect(ids::PHYSICS_LAYER_ROW_LABEL[3])
            .expect("o rótulo")
            .w,
        "o campo da grade é mais largo que o rótulo («Enemy» virava «nemy»)"
    );
    mesa.host.set_text(ids::PHYSICS_LAYER_NAME[3], "Mud");
    let ev = mesa.host.esc_key();
    let _ = mesa.aplica(ev);
    assert!(mesa.host.drained_actions().is_empty(), "Esc nunca grava");
    assert!(
        aba::aberta() && aba::editando().is_none(),
        "Esc não fecha a aba"
    );
    mesa.pinta();

    mesa.duplo(ids::PHYSICS_LAYER_COL_LABEL[1]);
    assert_eq!(aba::editando(), Some(1));
    mesa.host.set_text(ids::PHYSICS_LAYER_NAME[1], "   ");
    let ev = mesa.host.key_down(enter);
    let _ = mesa.aplica(ev);
    assert_eq!(
        mesa.host.drained_actions(),
        [EditorAction::LayerRename {
            layer: 1,
            name: String::new()
        }]
    );
    aba::fecha();
}

/// Sem nome, a camada 0 lê-se «Default» e as outras «Layer n»; nada disto vai ao ficheiro.
#[test]
fn default_e_layer_n_e_nao_grava_nomes() {
    let mut sem = LayerNames::default();
    assert_eq!(aba::rotulo(&sem, 0), "Default");
    for n in 1..MAX_LAYERS {
        assert_eq!(aba::rotulo(&sem, n), format!("Layer {n}"));
    }
    assert!(sem.to_bytes().is_empty(), "o padrão nunca vai ao ficheiro");
    sem.0[0] = "Ground".into();
    assert_eq!(aba::rotulo(&sem, 0), "Ground", "o nome do artista vence");
    // Um projeto só com a 0: a frase ensina o primeiro passo.
    let mut mesa = Mesa::nova(LayerNames::default(), false);
    ph2d_panel_physics::set_visible_layers(1);
    abre(&mut mesa);
    assert_eq!(aba::lente().lista[0].1, "Default");
    assert!(aba::lente().pe.starts_with("Everything is on Default"));
    aba::fecha();
}

/// O «+ Add» mostra a primeira camada escondida e abre-a; o pé diz que ela nasce batendo em tudo.
#[test]
fn mais_add_mostra_a_primeira_escondida_e_o_pe_diz_que_bate_em_tudo() {
    let mut mesa = Mesa::nova(nomes_b(), false);
    abre(&mut mesa);
    let _ = mesa.host.drained_actions();
    let _ = mesa.clica(ids::PHYSICS_LAYER_ADD);
    assert_eq!(
        mesa.host.drained_actions(),
        [EditorAction::LayerShow { layer: 5 }]
    );
    assert_eq!(aba::camada_aberta(), 5);
    ph2d_panel_physics::set_visible_layers(CINCO | 1 << 5);
    mesa.pinta();
    assert!(
        aba::lente()
            .pe
            .starts_with("New layers bump into everything")
    );
    aba::fecha();
}

/// A face clássica (`PH2D_CARDS=0`) tem a mesma porta, que abre a MESMA aba.
#[test]
fn a_face_classica_abre_a_mesma_aba() {
    let mut mesa = Mesa::nova(LayerNames::default(), true);
    assert!(mesa.rect(celula(0, 0)).is_none(), "sem grelha no painel");
    let _ = mesa.clica(ids::PHYSICS_MATRIX_OPEN);
    aba::poe_area_fixa(AREA);
    mesa.pinta();
    assert!(aba::aberta());
    assert!(mesa.rect(celula(4, 4)).is_some());
    aba::fecha();
    cards::force_cards_on_this_thread(None);
}

/// O arrasto que pinta atravessa a linha 4 na aba: cada célula cruzada muda, e só ela.
#[test]
fn arrastar_na_aba_pinta_cada_celula_cruzada() {
    let mut mesa = Mesa::nova(LayerNames::default(), false);
    abre(&mut mesa);
    let (x0, y0) = mesa.centro(celula(4, 0));
    let (x1, y1) = mesa.centro(celula(4, 4));
    let ev = mesa.host.drag_at(x0, y0, x1, y1);
    let m = mundo_de(&mesa.aplica(ev));
    for j in 0..5 {
        assert!(!m.collides(4, j), "o par (4,{j}) ficou por pintar");
    }
    assert!(m.collides(3, 0), "fora do caminho não muda");
    aba::fecha();
}

/// O losango da matriz (canal MUNDO) vive no canto dos rótulos da grade, com as células por campos.
#[test]
fn o_losango_da_matriz_mora_na_aba() {
    use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_target, set_key_marks};
    set_key_marks(std::collections::BTreeMap::from([(
        celula(0, 0),
        KeyMark::Chave,
    )]));
    let mut mesa = Mesa::nova(LayerNames::default(), false);
    abre(&mut mesa);
    let tem = mesa
        .rects
        .iter()
        .filter_map(|(alvo, _)| key_mark_target(*alvo))
        .any(|c| c.fields.contains(&celula(0, 0)));
    set_key_marks(std::collections::BTreeMap::new());
    aba::fecha();
    assert!(tem, "a matriz perdeu a marca de animação na aba");
}
