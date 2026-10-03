//! Gates da cena `=127` — **as estrelas esticadas com contorno** (doc 121 W4).
//!
//! ⚠️ **O que estes gates NÃO podem medir:** a geometria de um `source.shape` é assada pela SHELL,
//! e num arnês headless o duplicador coze `n = 0` (a mesma cegueira dos gates da `=126`). ⇒ aqui
//! prova-se a ESTRUTURA do grafo — que ele CONTÉM o fenómeno da W4 —, e a paridade de pixel da
//! rota vive nos gates de GPU (`a_rota_da_placa_desenha_o_traco_esticado_como_a_casa`).

use super::*;
use ph2d_node_registry::NodeRegistry;

fn cena() -> (MotionDoc, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, sinks)
}

fn um(doc: &MotionDoc, tipo: &str) -> NodeId {
    let v: Vec<NodeId> = doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == tipo)
        .map(|n| n.id)
        .collect();
    assert_eq!(v.len(), 1, "um {tipo} so'");
    v[0]
}

fn param(doc: &MotionDoc, n: NodeId, chave: &str) -> f32 {
    doc.graph
        .node_params()
        .get(&n)
        .and_then(|m| m.get(chave).copied())
        .unwrap_or_else(|| panic!("o param {chave:?} nao foi escrito"))
}

fn liga(doc: &MotionDoc, de: NodeId, para: NodeId, porta: u16) -> bool {
    doc.graph
        .edges()
        .iter()
        .any(|e| e.from.0 == de && e.to == (para, porta))
}

/// ⭐⭐⭐ **A CENA CONTÉM O FENÓMENO DA W4: contorno + escala NÃO uniforme + simulação com campos.**
///
/// ⚠️ Cada metade é uma cena que ensinaria o contrário: sem contorno (a `=126`) a W4 não muda um
/// pixel; com o esticão IGUAL nos dois eixos a cópia é conforme e já ia à placa desde a W2; com
/// tracejado a cópia continua no Vello (doc 121 §9). E sem a simulação a cena violava a regra do
/// dono (doc 103 §1).
#[test]
fn a_cena_tem_contorno_esticao_e_simulacao() {
    let (doc, sinks) = cena();
    assert_eq!(sinks.len(), 1, "uma saida so'");
    let forma = um(&doc, "source.shape");
    let dup = um(&doc, "motion.duplicator");
    let estica = um(&doc, "motion.scale");
    let saida = um(&doc, "motion.output");
    let ig = um(&doc, "motion.integrate");
    use ph2d_node_motion_shape::param as p;
    assert!(
        param(&doc, forma, p::STROKE_WIDTH) > 0.0,
        "sem contorno a W4 nao muda um pixel"
    );
    assert!(
        param(&doc, forma, p::FILL) > 0.5,
        "o preenchimento separa o contorno"
    );
    assert!(
        param(&doc, estica, "uniform") < 0.5,
        "o esticao tem de ter os eixos SEPARADOS"
    );
    let (ax, ay) = (
        param(&doc, estica, "amount"),
        param(&doc, estica, "amount_y"),
    );
    assert!(
        (ax - ay).abs() > 0.5,
        "com {ax} e {ay} a copia e' (quase) conforme e ja' ia 'a placa antes da W4"
    );
    assert!(
        liga(&doc, forma, dup, 0),
        "a forma na porta 0 do duplicador"
    );
    assert!(
        liga(&doc, ig, dup, 1),
        "a SIMULACAO nos pontos do duplicador"
    );
    assert!(liga(&doc, dup, estica, 0), "o esticao depois do carimbo");
    assert!(liga(&doc, estica, saida, 0), "e a saida le' o esticado");
    for campo in ["force.vortex", "force.attractor", "force.curl"] {
        um(&doc, campo);
    }
}

/// **O canto do campo fica DENTRO do núcleo da galáxia, nos DOIS arranjos** — fora dele o ímã ganha
/// à órbita e as estrelas caem para o meio (o doc da `GALAXIA`, na `=126`). E o ímã sai do núcleo
/// pela lei dela: com o núcleo da `=126` a derivação devolve o número dela.
#[test]
fn o_campo_cabe_no_nucleo_da_galaxia() {
    for a in [LEGIVEL, DENSO] {
        let gx = a.galaxia();
        assert!(
            a.meia_diagonal() < gx.nucleo,
            "{a:?}: o canto esta' a {} e o nucleo e' {}",
            a.meia_diagonal(),
            gx.nucleo
        );
        let equilibrio = gx.vortex * gx.vortex / gx.nucleo;
        assert!(
            gx.iman < equilibrio && gx.iman > 0.9 * equilibrio,
            "{a:?}: o iman {} fugiu do equilibrio {equilibrio}",
            gx.iman
        );
    }
    let base = super::super::carimbo_demo::GALAXIA;
    let mesma = LEGIVEL.galaxia();
    assert!(
        (mesma.nucleo - base.nucleo).abs() < 1e-6,
        "o legivel cabe no nucleo da =126"
    );
    assert!(
        (mesma.iman - base.iman).abs() < 0.03 * base.iman,
        "a derivacao devolve o iman da =126 ({} contra {})",
        mesma.iman,
        base.iman
    );
}

/// ⭐ **O arranjo DENSO põe milhares de estrelas à vista na câmara de arranque, e o legível não** —
/// é a razão de existirem dois (ver o doc do [`Arranjo`]): o relógio só se mexe com milhares à
/// vista, e o contorno só se julga numa estrela grande. A janela de arranque é a da `=126`
/// (`21,8 × 6,8` unidades, medida na foto dela).
#[test]
fn o_denso_enche_o_ecra_e_o_legivel_mostra_o_contorno() {
    let a_vista = |a: Arranjo| (21.8 / a.vao()) * (6.8 / a.vao());
    assert!(
        a_vista(DENSO) > 1500.0,
        "o denso mostra {} estrelas",
        a_vista(DENSO)
    );
    assert!(
        a_vista(LEGIVEL) < 500.0,
        "o legivel mostra {} estrelas",
        a_vista(LEGIVEL)
    );
    const { assert!(LEGIVEL.estrela_px >= 2.0 * DENSO.estrela_px) };
    const { assert!(LEGIVEL.estrela_px * ESTICA_Y >= 8.0 * LEGIVEL.contorno_px) };
}

/// **A porta do arranjo:** ausente, lixo ou `0` ⇒ o legível; só `1` pede o denso.
#[test]
fn a_porta_do_arranjo_so_abre_com_um() {
    assert_eq!(arranjo_por(None), LEGIVEL);
    assert_eq!(arranjo_por(Some("0")), LEGIVEL);
    assert_eq!(arranjo_por(Some("sim")), LEGIVEL);
    assert_eq!(arranjo_por(Some(" 1 ")), DENSO);
}

/// ⭐ doc 121 §9.9 — **a variante TRACEJADA põe o `Dash` e o `Dash Gap` na forma**, e só ela: o
/// contorno da cena de sempre continua contínuo. A porta só abre com `1`.
#[test]
fn a_variante_tracejada_poe_o_dash_na_forma() {
    use ph2d_node_motion_shape::param as p;
    assert!(!tracejado_por(None) && !tracejado_por(Some("0")) && tracejado_por(Some(" 1 ")));
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let dash = |tracejado: bool| {
        let mut doc = MotionDoc::default();
        monta(&mut doc, &reg, LEGIVEL, tracejado).expect("a cena monta");
        doc.graph.validate(&reg).expect("bem-tipada");
        let forma = um(&doc, "source.shape");
        let ps = doc
            .graph
            .node_params()
            .get(&forma)
            .cloned()
            .unwrap_or_default();
        (ps.get(p::DASH).copied(), ps.get(p::DASH_GAP).copied())
    };
    assert_eq!(dash(true), (Some(TRACEJADO.0), Some(TRACEJADO.1)));
    assert_eq!(
        dash(false),
        (None, None),
        "a cena de sempre fica com o contorno continuo"
    );
}

/// **O `=127` monta esta cena, e o tecto da varredura alcança-a.**
#[test]
fn o_roteador_monta_esta_cena_no_127() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = super::super::demo_router::build_level(Some("127"), &mut doc, &reg);
    assert_eq!(sinks.len(), 1, "o `=127` monta a cena e ela tem uma saida");
    um(&doc, "motion.scale");
    const _: () = assert!(super::super::demo_router::MAX_DEMO_LEVEL >= 127);
}

/// **O passo (4) compara com a porta que muda a rota, e o roteiro manda ler o `raw`.**
#[test]
fn o_roteiro_compara_com_a_porta_da_placa_e_le_o_raw() {
    let placa = include_str!("motion_shape_placa.rs");
    let porta = "PH2D_FORMAS_NA_PLACA";
    assert!(
        placa.contains(&format!("std::env::var(\"{porta}\")")),
        "a placa de formas deixou de ler {porta:?} -- o passo (4) aponta para uma porta morta"
    );
    let anuncio = include_str!("motion_state_traco_esticado_demo.rs")
        .split_once("pub(super) fn announce()")
        .expect("a cena tem um roteiro")
        .1;
    let passo4 = anuncio
        .split_once("(4)")
        .and_then(|(_, r)| r.split_once("DEU ERRADO"))
        .expect("o roteiro tem o passo (4) e o deu-errado")
        .0;
    assert!(passo4.contains(porta), "o passo (4) compara com {porta:?}");
    let hud = ph2d_i18n::tr_em(ph2d_i18n::Idioma::Ingles, "chrome.hud.fps");
    assert!(
        hud.contains("raw"),
        "a barra de baixo deixou de dizer `raw`"
    );
    assert!(anuncio.contains("raw"), "o roteiro manda ler o `raw`");
}
