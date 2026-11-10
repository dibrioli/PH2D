//! ⭐ **A fileira dos chips nas abas de faixas** (`docs/Timeline/16_onda3_containers_arrange.md`, maquete B4):
//! «Clips» + um chip por clipe + o «+», depois «Containers» + um chip por container com «used 2×» / «not
//! placed». Só a cara e a arrumação: os mesmos ids, o mesmo clique. Gesto real, rects pintados.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::ids::{self, timeline_source_opt};
use ph2d_panel_timeline::state::{TimelinePanelState, drain_intents, set_current_timeline};
use ph2d_panel_timeline::tab::Tab;
use ph2d_timeline::{
    ContainerView, SCENE, TimelineIntent, TimelineState, TimelineViewSnapshot, animation_view,
    apply_intent,
};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1040.0,
};

const ABAS: [Tab; 2] = [Tab::Containers, Tab::Arrange];

fn documento() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        time_seconds: 2.0,
        frame: 120,
        view_length_seconds: 4.0,
        view_length_explicit: true,
        clips: vec!["Main".to_owned(), "Walk".to_owned()],
        clip_index: vec![0, 1],
        containers: vec![
            ContainerView {
                name: "Run & jump".into(),
                length: 2.0,
                uses: 2,
            },
            ContainerView {
                name: "Idle pack".into(),
                length: 1.0,
                uses: 0,
            },
        ],
        ..TimelineViewSnapshot::default()
    }
}

/// Dezasseis clipes a 1100 px: o que não cabe dobra no «+N».
fn muitos() -> (Rect, TimelineViewSnapshot) {
    let snap = TimelineViewSnapshot {
        clip_index: (0..16).collect(),
        clips: (0..16).map(|i| format!("clip numero {i}")).collect(),
        ..documento()
    };
    (
        Rect {
            w: 1100.0,
            ..JANELA
        },
        snap,
    )
}

fn mesa_em(
    cartoes: bool,
    tab: Tab,
    janela: Rect,
    snap: TimelineViewSnapshot,
) -> Mesa<TimelinePanel> {
    ferramenta::force_cards_on_this_thread(Some(cartoes));
    set_current_timeline(Some(snap));
    let _ = drain_intents();
    let mut mesa = Mesa::nova(TimelinePanelState {
        tab,
        ..TimelinePanelState::default()
    });
    mesa.assenta_em(janela);
    let _ = drain_intents();
    let _ = mesa.host.drained_actions();
    mesa
}

fn mesa(tab: Tab) -> Mesa<TimelinePanel> {
    mesa_em(true, tab, JANELA, documento())
}

fn pintado(m0: &Mesa<TimelinePanel>, id: NodeId) -> Rect {
    m0.rect(id).unwrap_or_else(|| panic!("{id:?} não pintado"))
}

/// ⭐ Os alvos não encolheram com a arrumação nova: cada um é pelo menos o medido ANTES dela (10/11, cartões,
/// 1930 px, abas Containers e Arrange; o «+N» a 1100 px com dezasseis clipes). O chip do container cresceu
/// com o «used 2×», o alvo cresce com ele.
#[test]
fn os_alvos_dos_chips_nao_encolheram() {
    const ANTES: [(NodeId, f32, f32); 6] = [
        (ids::TIMELINE_CLIP_OPT[0], 56.0, 28.0),
        (ids::TIMELINE_CLIP_OPT[1], 61.0, 28.0),
        (ids::TIMELINE_CONT_OPT[0], 101.0, 28.0),
        (ids::TIMELINE_CONT_OPT[1], 85.0, 28.0),
        (ids::TIMELINE_CLIP_MENU, 28.0, 28.0),
        (ids::TIMELINE_ADD_CLIP, 28.0, 28.0),
    ];
    let cabe = |m0: &Mesa<TimelinePanel>, (id, w, h): (NodeId, f32, f32), onde: &str| {
        let r = pintado(m0, id);
        assert!(
            r.w >= w && r.h >= h,
            "{onde} {id:?}: alvo {}×{} < {w}×{h}",
            r.w,
            r.h
        );
    };
    for tab in ABAS {
        let m0 = mesa(tab);
        for alvo in ANTES {
            cabe(&m0, alvo, &format!("{tab:?}"));
        }
    }
    let (janela, snap) = muitos();
    let m0 = mesa_em(true, Tab::Arrange, janela, snap);
    for alvo in [
        (ids::TIMELINE_CLIP_DD, 89.0, 28.0),
        (ids::TIMELINE_ADD_CLIP, 28.0, 28.0),
    ] {
        cabe(&m0, alvo, "1100");
    }
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐⭐ Dois grupos na MESMA fileira (a do Auto Key): os clipes e o «+» antes, os containers depois, com o
/// rótulo «Containers» no vão. A Keys não tem o 2.º grupo. *Mutação: pintar os containers antes do «+» ⇒
/// vermelho na ordem.*
#[test]
fn os_dois_grupos_moram_na_fileira_do_gravar() {
    let rotulo_w = ph2d_text::TextSystem::without_system_fonts()
        .prefix_width(ph2d_i18n::tr("panel.timeline.card.containers"), m::LABEL_PX)
        .ceil();
    for tab in ABAS {
        let m0 = mesa(tab);
        let r = |id| pintado(&m0, id);
        let fileira = r(ids::TIMELINE_AUTOKEY).y;
        let ordem = [
            ids::TIMELINE_CLIP_OPT[0],
            ids::TIMELINE_CLIP_OPT[1],
            ids::TIMELINE_ADD_CLIP,
            ids::TIMELINE_CONT_OPT[0],
            ids::TIMELINE_CONT_OPT[1],
        ];
        for id in ordem {
            assert!(
                (r(id).y - fileira).abs() < 0.5,
                "{tab:?} {id:?} fora da fileira"
            );
        }
        for par in ordem.windows(2) {
            assert!(
                r(par[0]).x + r(par[0]).w <= r(par[1]).x + 0.5,
                "{tab:?}: {:?} não vem antes de {:?}",
                par[0],
                par[1]
            );
        }
        let mais = r(ids::TIMELINE_ADD_CLIP);
        let vao = r(ids::TIMELINE_CONT_OPT[0]).x - (mais.x + mais.w);
        assert!(
            vao >= rotulo_w + m::FIELD_GAP,
            "{tab:?}: sem lugar para «Containers» entre o + e o 1.º container: {vao}"
        );
    }
    let mut keys = mesa(Tab::Keys);
    assert!(
        keys.rect(ids::TIMELINE_CONT_OPT[0]).is_none(),
        "a Keys oferece um container"
    );
    // A aba «Containers» também se escreve assim: o rótulo do grupo é UM a mais que na Keys.
    let quantos = |m0: &mut Mesa<TimelinePanel>| {
        textos(m0)
            .iter()
            .filter(|t| t.texto == "Containers")
            .count()
    };
    assert_eq!(
        quantos(&mut mesa(Tab::Arrange)),
        quantos(&mut keys) + 1,
        "o rótulo do 2.º grupo: um no Arrange, nenhum na Keys"
    );
    ferramenta::force_cards_on_this_thread(None);
}

/// O que um clique pediu: ao documento e à fonte do painel.
fn efeito(mesa: &mut Mesa<TimelinePanel>, id: NodeId) -> String {
    let acoes = mesa.clica(id);
    format!(
        "{acoes:?} | {:?} | {:?} | {:?}",
        drain_intents(),
        mesa.st.source_container,
        mesa.st.source_clip
    )
}

/// ⭐⭐ Cada chip escolhe EXACTAMENTE o que a mesma linha da lista de sempre escolhia (`PH2D_CARDS=0`, lista
/// aberta): o container vira a fonte, o clipe vira o activo. O «+» ainda cria um clipe, o «⋯» ainda abre o
/// menu. *Mutação: o chip do container registar o id do clipe ⇒ vermelho.*
#[test]
fn clicar_um_chip_escolhe_o_mesmo_que_a_lista() {
    for tab in ABAS {
        for id in [
            ids::TIMELINE_CLIP_OPT[1],
            ids::TIMELINE_CONT_OPT[0],
            ids::TIMELINE_CONT_OPT[1],
        ] {
            let antes = {
                let mut m0 = mesa_em(false, tab, JANELA, documento());
                let _ = m0.clica(ids::TIMELINE_CLIP_DD);
                m0.assenta_em(JANELA);
                efeito(&mut m0, id)
            };
            let depois = efeito(&mut mesa(tab), id);
            assert_eq!(antes, depois, "{tab:?} {id:?} mudou de efeito");
        }
        let mut m0 = mesa(tab);
        let _ = efeito(&mut m0, ids::TIMELINE_CONT_OPT[1]);
        assert_eq!(
            m0.st.source_container,
            Some(1),
            "{tab:?}: o container não é a fonte"
        );
        let mut m0 = mesa(tab);
        let _ = m0.clica(ids::TIMELINE_ADD_CLIP);
        assert!(
            drain_intents()
                .iter()
                .any(|i| matches!(i, TimelineIntent::AddClip)),
            "{tab:?}: o + não cria um clipe"
        );
        let _ = m0.clica(ids::TIMELINE_CLIP_MENU);
        assert!(m0.st.menu_do_clip, "{tab:?}: o ⋯ não abriu o menu");
    }
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ Com dezasseis clipes a dobra é UMA regra: os containers vão com os clipes que não couberam para o
/// «+N», que abre a lista de sempre — e ela traz os containers. *Mutação: o «+N» contar só os clipes ⇒ a
/// lista abre, mas o número mente (vermelho no texto).*
#[test]
fn o_mais_n_dobra_os_dois_grupos_e_abre_a_lista() {
    let (janela, snap) = muitos();
    let mut m0 = mesa_em(true, Tab::Arrange, janela, snap);
    let pintados = ids::TIMELINE_CLIP_OPT
        .iter()
        .chain(&ids::TIMELINE_CONT_OPT[..2])
        .filter(|id| m0.rect(**id).is_some())
        .count();
    let escondidos = 18 - pintados;
    assert!(escondidos > 0, "vácuo: tudo coube a 1100 px");
    let mais = pintado(&m0, ids::TIMELINE_CLIP_DD);
    let fileira = pintado(&m0, ids::TIMELINE_CLIP_OPT[0]).y;
    assert!(
        (mais.y - fileira).abs() < 0.5,
        "o «+N» fora da fileira dos chips"
    );
    assert!(
        m0.rect(ids::TIMELINE_CONT_OPT[1]).is_none(),
        "o último container não dobrou com os clipes"
    );
    // O número do «+N» conta os dois grupos: pelo menos os dois containers.
    let n: usize = textos_em(&mut m0, janela)
        .iter()
        .find_map(|t| {
            t.texto
                .strip_prefix('+')?
                .strip_suffix(" more")?
                .parse()
                .ok()
        })
        .expect("o «+N» não tem texto");
    assert_eq!(
        pintados + n,
        18,
        "o «+N» diz {n} com {pintados} chips à vista: os dois têm de somar os 18"
    );
    assert!(n >= 2, "o «+N» diz {n}: não conta os dois containers");
    let _ = m0.clica(ids::TIMELINE_CLIP_DD);
    m0.assenta_em(janela);
    assert!(
        matches!(
            m0.host.store().get(ids::TIMELINE_CLIP_DD),
            Some(InteractiveState::Dropdown { open: true, .. })
        ),
        "o «+N» não abriu a lista"
    );
    assert!(
        m0.rect(ids::TIMELINE_CONT_OPT[1]).is_some(),
        "a lista não mostrou o último container"
    );
    ferramenta::force_cards_on_this_thread(None);
}

fn textos(mesa: &mut Mesa<TimelinePanel>) -> Vec<ph2d_editor_core::text_elide::elisao::Medido> {
    textos_em(mesa, JANELA)
}

/// Os textos pintados à largura `janela`. ⚠️ Medir a OUTRA largura repinta o painel e troca o índice de
/// alvos: o clique seguinte cai na arrumação dessa outra largura (10/11: o «+N» a 1100 px medido a 1930).
fn textos_em(
    mesa: &mut Mesa<TimelinePanel>,
    janela: Rect,
) -> Vec<ph2d_editor_core::text_elide::elisao::Medido> {
    let (h, st) = (&mut mesa.host, &mut mesa.st);
    h.medindo_a_pintura::<TimelinePanel>(st, janela)
}

/// ⭐⭐ O chip do container diz onde é usado, um degrau de letra abaixo do nome e logo a seguir a ele; o do
/// clipe, não. *Mutação: `uso_de` devolver `None` ⇒ vermelho; o uso na letra do nome ⇒ vermelho.*
#[test]
fn o_chip_do_container_diz_onde_e_usado() {
    let mut m0 = mesa(Tab::Arrange);
    let t = textos(&mut m0);
    let depois_de = |nome: &str| {
        let k = t
            .iter()
            .position(|x| x.texto == nome && (x.fonte - m::VALUE_PX).abs() < 0.01)
            .unwrap_or_else(|| panic!("{nome:?} não pintado no chip"));
        t.get(k + 1).cloned()
    };
    for (nome, uso) in [("Run & jump", "used 2×"), ("Idle pack", "not placed")] {
        let seguinte = depois_de(nome).unwrap_or_else(|| panic!("nada depois de {nome:?}"));
        assert_eq!(seguinte.texto, uso, "o chip {nome:?}");
        assert!(
            seguinte.fonte < m::VALUE_PX,
            "o uso de {nome:?} na letra do nome"
        );
        assert!(
            seguinte.coube(),
            "o uso de {nome:?} cortado: {:?}",
            seguinte.pintado
        );
    }
    for nome in ["Main", "Walk"] {
        let seguinte = depois_de(nome).map(|x| x.texto).unwrap_or_default();
        assert!(
            seguinte != "not placed" && !seguinte.starts_with("used "),
            "o chip do clipe {nome:?} diz {seguinte:?}"
        );
    }
    assert!(
        t.iter().any(|x| x.texto == "Containers"),
        "o rótulo do 2.º grupo não foi pintado"
    );
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐⭐ A altura é o recurso deste painel: o 2.º grupo não acrescenta fileira — a folha das abas de faixas
/// tem a mesma altura com e sem containers, a três larguras. *Mutação: `f.quebra()` antes de «Containers» ⇒
/// vermelho.*
#[test]
fn o_segundo_grupo_nao_come_a_folha() {
    for w in [1930.0, 1600.0, 1100.0] {
        let janela = Rect { w, ..JANELA };
        for tab in ABAS {
            let sem = TimelineViewSnapshot {
                containers: Vec::new(),
                ..documento()
            };
            // Onde a folha começa: o «+» das faixas, logo abaixo da barra.
            let topo = |snap| {
                let m0 = mesa_em(true, tab, janela, snap);
                let fundo = m0.rect(ids::TIMELINE_AUTOKEY).map(|r| r.y);
                (m0.rect(ids::TIMELINE_ADD_TRACK).map(|r| r.y), fundo)
            };
            let (antes, depois) = (topo(sem), topo(documento()));
            assert!(antes.1.is_some(), "vácuo: a barra não pintou");
            assert_eq!(
                antes, depois,
                "{tab:?} a {w}: a folha ou a fileira mudaram com os containers"
            );
        }
    }
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ Na Cena com outro dono a lista vem agrupada por dono (`transport_sources`): os cabeçalhos e os clipes
/// de TODOS os donos ficam no grupo dos clipes, antes do «+»; os containers continuam no deles, e o chip
/// escolhe-o como fonte.
#[test]
fn na_cena_agrupada_os_donos_ficam_nos_clipes_e_os_containers_a_parte() {
    const HEROI: u64 = 7;
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    let mut st = TimelineState::new();
    st.doc.add_clip_for("andar".into(), HEROI).unwrap();
    st.doc.add_clip_for("pular".into(), HEROI).unwrap();
    st.doc.add_container("Run & jump".into());
    apply_intent(&mut st, &mut ph, TimelineIntent::AddLane);
    animation_view::follow(&mut st, &mut ph, None, &[]);
    assert_eq!(st.view.owner, SCENE, "a vista é a da Cena");
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    assert!(
        snap.animation.sources.len() > 1,
        "pré-condição: a lista agrupa"
    );
    let mut m0 = mesa_em(true, Tab::Arrange, JANELA, snap);
    let mais = pintado(&m0, ids::TIMELINE_ADD_CLIP);
    // Linhas: 0 «Cena» · 1 Main · 2 o Herói · 3 andar · 4 pular.
    for linha in [1, 3, 4] {
        let r = pintado(&m0, timeline_source_opt(linha));
        assert!(r.x + r.w <= mais.x + 0.5, "a linha {linha} passou do +");
        assert!(
            (r.y - mais.y).abs() < 0.5,
            "a linha {linha} fora da fileira"
        );
    }
    let c = pintado(&m0, ids::TIMELINE_CONT_OPT[0]);
    assert!(c.x >= mais.x + mais.w, "o container não vem depois do +");
    let _ = efeito(&mut m0, ids::TIMELINE_CONT_OPT[0]);
    assert_eq!(
        m0.st.source_container,
        Some(0),
        "o chip não escolheu o container"
    );
    ferramenta::force_cards_on_this_thread(None);
}
