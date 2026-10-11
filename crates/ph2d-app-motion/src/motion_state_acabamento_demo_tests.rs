//! Os gates da cena `=135`: o filtro e a lente estão NO fio, a lente nasce só com a vinheta, o
//! filtro é a sombra projectada, e o fio passa intacto — medido na SAÍDA, pela bomba do produto.

use super::*;
use crate::motion_state::MotionState;

fn cozida(tiques: u64) -> (MotionState, Cena) {
    let mut m = MotionState::new();
    let cena = monta(&mut m.doc, &m.registry).expect("a cena monta");
    m.sinks = vec![cena.saida];
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    for t in 0..tiques {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
        let sinks = m.sinks.clone();
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
    }
    (m, cena)
}

/// ⭐ **A cena ensina o que acontece** (`CLAUDE.md` §5.0): ao abrir, a lente CORRE (a vinheta
/// ligada) e os outros três efeitos estão neutros — o passo (1) do roteiro.
#[test]
fn the_scene_opens_with_only_the_vignette_on() {
    let (mut m, cena) = cozida(2);
    let lido = ph2d_node_fx_lens::from_graph(&m.doc.graph).expect("a cena tem a lente");
    assert_eq!(lido.no, cena.lente);
    let l = lido.lente;
    assert!(!l.vinheta.neutra());
    assert!(l.aberracao.neutra() && l.grade.neutra(false) && l.grao.neutra());
    m.lente.prepara(&m.doc.graph, 2);
    assert!(m.lente.liga(true), "a lente corre com o Motion na cena");
}

/// O filtro e a lente estão NO fio, entre a cor e a saída — e o fio passa INTACTO (passthrough).
#[test]
fn the_filter_and_the_lens_sit_on_the_wire_and_pass_it_through() {
    let (m, cena) = cozida(30);
    let g = &m.doc.graph;
    let liga = |de, para| g.edges().iter().any(|e| e.from.0 == de && e.to.0 == para);
    assert!(liga(cena.tint, cena.forma) && liga(cena.forma, cena.camada));
    assert!(liga(cena.camada, cena.lente));
    assert!(
        g.edges()
            .iter()
            .any(|e| e.from.0 == cena.lente && e.to.0 == cena.saida)
    );
    let n = |id| m.pump.cook.peek(id).expect("cozeu")[0].as_stream().count();
    assert!(
        n(cena.saida) > 100,
        "a galáxia tem estrelas: {}",
        n(cena.saida)
    );
    assert_eq!(n(cena.saida), n(cena.tint));
}

/// ⭐ Os passos dos filtros: a cena abre com o RELEVO em cada disco e a SOMBRA da camada inteira,
/// pela ordem do fio — a pilha parte-se entre os dois — e os filtros correm.
#[test]
fn the_scene_opens_with_a_bevel_on_each_shape_and_a_shadow_on_the_layer() {
    let (mut m, cena) = cozida(2);
    m.filtro.prepara(&m.doc.graph, &m.sinks);
    let pilha = m.filtro.pilha();
    assert_eq!(
        pilha.iter().map(|d| d.no).collect::<Vec<_>>(),
        vec![cena.forma, cena.camada]
    );
    assert_eq!(pilha[0].op, ph2d_ecs::FxOp::new(ph2d_ecs::FxOp::BEVEL));
    assert_eq!(pilha[1].op.kind, ph2d_ecs::FxOp::DROP_SHADOW);
    let (forma, camada) = ph2d_node_fx_filter::partition(pilha);
    assert_eq!((forma.len(), camada.len()), (1, 1));
    assert!(
        m.filtro.liga(true),
        "os filtros correm com o Motion na cena"
    );
    assert!(m.filtro.queixa(cena.forma).is_none() && m.filtro.queixa(cena.camada).is_none());
}

/// O passo (4): o 1.º na camada inteira e o 2.º em cada forma ⇒ o cartão do 2.º di-lo.
#[test]
fn an_each_shape_after_a_whole_layer_says_so_on_its_card() {
    let (mut m, cena) = cozida(2);
    m.doc
        .graph
        .set_param(cena.forma, ph2d_node_fx_filter::param::SCOPE, 1.0);
    m.doc
        .graph
        .set_param(cena.camada, ph2d_node_fx_filter::param::SCOPE, 0.0);
    m.filtro.prepara(&m.doc.graph, &m.sinks);
    assert!(m.filtro.queixa(cena.forma).is_none());
    let q = m.filtro.queixa(cena.camada).expect("o cartão diz");
    assert!(q.contains("Whole Layer"), "{q}");
}

/// ⭐ **Cada foto é o passo que diz ser** (`PH2D_ACABAMENTO_PASSO`): a variante muda o que o
/// roteiro manda mudar, e só isso — o passo desconhecido deixa a cena AO BIT.
#[test]
fn every_photo_variant_is_the_step_it_names() {
    let lido = |passo_: &str| {
        let mut m = MotionState::new();
        let cena = monta(&mut m.doc, &m.registry).expect("a cena monta");
        assert!(passo(&mut m.doc.graph, &cena, passo_), "{passo_}");
        m.filtro.prepara(&m.doc.graph, &m.sinks);
        let pilha = m.filtro.pilha().to_vec();
        let (f, c) = ph2d_node_fx_filter::partition(&pilha);
        let partes = (f.len(), c.len());
        let queixa = m.filtro.queixa(cena.camada);
        let lente = ph2d_node_fx_lens::from_graph(&m.doc.graph).expect("a lente");
        (pilha, partes, queixa, lente)
    };
    let (base, partes, queixa, lente0) = lido("0");
    assert_eq!(partes, (1, 1));
    assert!(queixa.is_none());
    assert!(!lente0.lente.vinheta.neutra() && lente0.lut.is_none());
    // Os filtros: o `Apply To` parte a pilha noutro sítio; o `Effect` é o gesto do cartão.
    assert_eq!(lido("1").1, (0, 2));
    assert_eq!(
        lido("2").0[0].op,
        ph2d_ecs::FxOp::new(ph2d_ecs::FxOp::INNER_SHADOW)
    );
    assert_eq!(lido("2").0[1].op, base[1].op);
    assert_eq!(lido("3").1, (2, 0));
    let (_, partes, queixa, _) = lido("4");
    assert_eq!(partes, (0, 2));
    assert!(queixa.is_some_and(|q| q.contains("Whole Layer")));
    // A lente: cada passo mexe na SUA secção, e as outras ficam como abriram.
    let l = |p: &str| lido(p).3;
    assert!(l("5").lente.vinheta.neutra());
    assert!(l("5b").lente.vinheta.amount < 0.0);
    // A mistura: o passo (9) põe a saída em `Add` (com tudo, o alcance de origem).
    let mut m = MotionState::new();
    let cena = monta(&mut m.doc, &m.registry).expect("a cena monta");
    assert!(passo(&mut m.doc.graph, &cena, "9"));
    let g = &m.doc.graph;
    assert_eq!(ph2d_eval_motion::sink_blend_tag(g, cena.saida), 1, "Add");
    assert_eq!(
        ph2d_eval_motion::sink_blend_with(g, cena.saida),
        ph2d_eval_motion::BlendWith::Everything
    );
    assert!(!l("6").lente.aberracao.neutra() && l("6").lente.vinheta == lente0.lente.vinheta);
    assert!(l("7a").lente.grade.temperatura > 0.0);
    assert!((l("7b").lente.grade.saturacao + 1.0).abs() < 1e-6);
    assert!((l("7c").lente.grade.exposure - 1.0).abs() < 1e-6);
    assert_eq!(l("7d").lut.as_deref(), Some(LUT_QUENTE));
    assert!(!l("8").lente.grao.neutra());
    for p in PASSOS.iter().filter(|p| !p.starts_with('5')) {
        assert_eq!(
            l(p).lente.vinheta,
            lente0.lente.vinheta,
            "o passo {p} mexeu na vinheta"
        );
    }
    // O passo desconhecido: a cena de sempre.
    let mut m = MotionState::new();
    let cena = monta(&mut m.doc, &m.registry).expect("a cena monta");
    let antes = m.doc.graph.clone();
    assert!(!passo(&mut m.doc.graph, &cena, "99"));
    assert_eq!(format!("{:?}", m.doc.graph), format!("{antes:?}"));
    // E o `.cube` que o passo (7) nomeia existe, onde o app o procura (a raiz do repo).
    let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");
    assert!(std::path::Path::new(raiz).join(LUT_QUENTE).is_file());
}

/// O roteiro só nomeia rows que o cartão do `Lens` pinta (o nome que aparece NA TELA).
#[test]
fn every_row_the_script_names_is_on_the_lens_card() {
    let m = MotionState::new();
    let id = ph2d_nodegraph::node::NodeTypeId::of(ph2d_node_fx_lens::TYPE_NAME);
    let rotulos: Vec<&str> = m
        .registry
        .param_ui(id)
        .unwrap_or(&[])
        .iter()
        .map(|h| ph2d_i18n::tr(h.label))
        .collect();
    for nome in [
        "Vignette",
        "Aberration",
        "Temperature",
        "Saturation",
        "Exposure",
        "LUT File",
        "LUT Mix",
        "Grain",
    ] {
        assert!(rotulos.contains(&nome), "{nome} ∉ {rotulos:?}");
    }
    for secao in [
        "node.group.vignette",
        "node.group.aberration",
        "node.group.grade",
        "node.group.grain",
    ] {
        assert_ne!(ph2d_i18n::tr(secao), secao, "secção sem nome: {secao}");
    }
}

/// O roteiro só nomeia rows e opções que o cartão do `Filter` pinta.
#[test]
fn every_row_the_script_names_is_on_the_filter_card() {
    let m = MotionState::new();
    let id = ph2d_nodegraph::node::NodeTypeId::of(ph2d_node_fx_filter::TYPE_NAME);
    let hints = m.registry.param_ui(id).unwrap_or(&[]);
    let rotulos: Vec<&str> = hints.iter().map(|h| ph2d_i18n::tr(h.label)).collect();
    for nome in [
        "Effect", "Apply To", "Radius", "Offset X", "Offset Y", "Opacity",
    ] {
        assert!(rotulos.contains(&nome), "{nome} ∉ {rotulos:?}");
    }
    let tipos: Vec<&str> = ph2d_node_fx_filter::KIND_LABELS
        .iter()
        .map(|c| ph2d_i18n::tr(c))
        .collect();
    for tipo in ["Bevel", "Inner Shadow", "Drop Shadow"] {
        assert!(tipos.contains(&tipo), "{tipo} ∉ {tipos:?}");
    }
    let opcoes: Vec<&str> = ph2d_node_fx_filter::SCOPE_LABELS
        .iter()
        .map(|c| ph2d_i18n::tr(c))
        .collect();
    assert_eq!(opcoes, ["Each Shape", "Whole Layer"]);
}

/// ⭐⭐ **W4b — com a camada filtrada ligada, o `Output` em `Add` com o CENÁRIO não agrupa** (doc 129
/// §3.5), medido na bomba do PRODUTO sobre a `=135`: as formas saem sem modo (a placa leva-as e a
/// camada pousa somando) e as sprites sem os bits de mistura (senão o `Add` corria DUAS vezes — na
/// camada e na pousa). ⚠️ Os dois controlos: sem a camada o grupo de sempre, e com ela o
/// `Everything` continua a agrupar (mistura ENTRE as cópias, dentro da camada).
#[test]
fn a_scene_blend_with_the_layer_on_lowers_without_a_group() {
    for (isolada, alcance, agrupa) in [(false, 2.0, true), (true, 2.0, false), (true, 0.0, true)] {
        let mut m = MotionState::new();
        let cena = monta(&mut m.doc, &m.registry).expect("a cena monta");
        m.sinks = vec![cena.saida];
        let g = &mut m.doc.graph;
        g.set_param(cena.saida, ph2d_eval_motion::SINK_BLEND_PARAM, 1.0);
        g.set_param(cena.saida, ph2d_eval_motion::SINK_BLEND_WITH_PARAM, alcance);
        m.pump.isola_na_camada(isolada);
        let scopes = ph2d_nodegraph::cook::TimeScopes::new();
        for t in 0..2 {
            #[expect(clippy::cast_precision_loss, reason = "um tique")]
            crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
            let sinks = m.sinks.clone();
            let _ = m.pump.advance_or_scrub_scoped(
                &m.doc.graph,
                &m.registry,
                &sinks,
                t,
                |t| t as f64 / 60.0,
                m.default_uv_rect,
                m.default_size,
                &scopes,
            );
        }
        let caso = format!("camada {isolada}, alcance {alcance}");
        assert!(
            !m.pump.vector_instances.is_empty(),
            "{caso}: a =135 tem formas"
        );
        let tag = if agrupa { 1 } else { 0 };
        assert!(
            m.pump
                .vector_instances
                .iter()
                .all(|v| v.mistura.blend == tag),
            "{caso}: as formas deviam levar o tag {tag}"
        );
        assert!(
            m.pump
                .instances
                .iter()
                .all(|i| { ph2d_render::RenderInstance::unpack_blend(i.flip_uv) == 0 }),
            "{caso}: uma sprite leva o modo — ele correria na camada E na pousa"
        );
    }
}

/// ⭐ **Ligar a camada no MESMO tique recompõe** (auditoria da W4b): a camada muda o lowering de um
/// sink `Scene` sem tocar no grafo — sem o `dirty`, a bomba devolvia as cópias do tique com o modo
/// antigo e o `Add` corria na camada E na pousa.
#[test]
fn turning_the_layer_on_in_the_same_tick_relowers() {
    let mut m = MotionState::new();
    let cena = monta(&mut m.doc, &m.registry).expect("a cena monta");
    m.sinks = vec![cena.saida];
    m.doc
        .graph
        .set_param(cena.saida, ph2d_eval_motion::SINK_BLEND_PARAM, 1.0);
    m.doc
        .graph
        .set_param(cena.saida, ph2d_eval_motion::SINK_BLEND_WITH_PARAM, 2.0);
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let coze = |m: &mut MotionState| {
        crate::motion_externals::publish_all(m, 0.0);
        let sinks = m.sinks.clone();
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            0,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
    };
    coze(&mut m);
    assert!(
        m.pump.vector_instances.iter().all(|v| v.mistura.blend == 1),
        "sem camada: o grupo"
    );
    m.pump.isola_na_camada(true);
    coze(&mut m);
    assert!(
        !m.pump.vector_instances.is_empty()
            && m.pump.vector_instances.iter().all(|v| v.mistura.blend == 0),
        "a camada ligou no mesmo tique e as formas ficaram com o modo antigo"
    );
}
