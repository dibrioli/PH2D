use super::*;

/// ⭐ Sem `fx.filter` no fio os filtros não correm — e o passe fundido leva o Motion como sempre.
#[test]
fn without_a_filter_nothing_changes() {
    let mut f = MotionFiltro::default();
    f.prepara(&Graph::new());
    assert!(!f.liga(true));
    assert!(f.pilha().is_empty());
}

/// ⭐ **O tecto DIGITADO do comprimento é honrado pelo passe em qualquer janela e zoom** (doc 129
/// §5.5 A8): o σ que ele pede numa cena de 8K (4 320 px de altura) no zoom máximo da câmera cabe no
/// `SIGMA_ALCANCAVEL` da pirâmide. Antes a régua era o raio de SATURAÇÃO medido com os `534` px de
/// UMA janela como entrada — verde por construção, e a 4K a mesma régua saturava em ~0,27.
#[test]
fn the_typed_length_ceiling_is_honoured_at_8k_and_max_zoom() {
    let camera = ph2d_render::Camera2d {
        height_world: ph2d_render::Camera2d::ZOOM_MIN_HEIGHT_WORLD,
        ..ph2d_render::Camera2d::default()
    };
    let janela = ph2d_host::WindowSize {
        width: 7680,
        height: 4320,
    };
    let [a, ..] = mundo_para_pixel(&camera, janela, None);
    let mut op = ph2d_ecs::FxOp::new(ph2d_ecs::FxOp::BLUR);
    op.radius = ph2d_node_fx_filter::COMPRIMENTO_DIGITADO;
    let sigma = FxOpGpu::resolve(&op, [a, 0.0, 0.0, -a]).sigma_px;
    assert!(
        sigma <= ph2d_render::fx_stack::SIGMA_ALCANCAVEL,
        "o tecto digitado pede σ {sigma} px e a pirâmide honra até {}",
        ph2d_render::fx_stack::SIGMA_ALCANCAVEL
    );
    assert!(
        sigma > 32.0 * 1000.0,
        "controlo: a fixtura está MUITO acima do kernel de sempre ({sigma})"
    );
}

/// O Y do mundo sobe e o do pixel desce: uma sombra para BAIXO no mundo cai para baixo na tela.
#[test]
fn world_down_is_screen_down() {
    let camera = ph2d_render::Camera2d::default();
    let janela = ph2d_host::WindowSize {
        width: 800,
        height: 600,
    };
    let [a, b, c, d] = mundo_para_pixel(&camera, janela, None);
    assert!((a - 60.0).abs() < 1e-9 && b == 0.0 && c == 0.0 && (d + 60.0).abs() < 1e-9);
    let mut op = ph2d_ecs::FxOp::new(ph2d_ecs::FxOp::DROP_SHADOW);
    op.offset = [0.0, -0.5];
    assert_eq!(FxOpGpu::resolve(&op, [a, b, c, d]).offset_px, [0.0, 30.0]);
}

/// ⭐ **O ruído da camada inteira anda com o MUNDO** (doc 129 §5.5 A6): um pan da câmera de uma
/// unidade leva a origem do padrão `k` pixels (a escala da câmera), no sentido oposto; antes ela
/// era a margem da pilha — um ponto fixo da JANELA, e o padrão ficava parado com as formas a andar.
#[test]
fn the_layer_noise_is_anchored_to_the_world() {
    let janela = ph2d_host::WindowSize {
        width: 800,
        height: 600,
    };
    let vp = Some([100.0, 40.0, 600.0, 500.0]);
    let mut camera = ph2d_render::Camera2d::default();
    let o0 = origem_do_mundo(&camera, janela, vp);
    camera.center[0] += 1.0;
    camera.center[1] += 1.0;
    let o1 = origem_do_mundo(&camera, janela, vp);
    let k = 500.0 / 10.0;
    assert!(
        (o0[0] - (100.0 + 300.0)).abs() < 1e-3 && (o0[1] - (40.0 + 250.0)).abs() < 1e-3,
        "{o0:?}"
    );
    assert!(
        (o1[0] - o0[0] + k).abs() < 1e-3 && (o1[1] - o0[1] - k).abs() < 1e-3,
        "{o0:?} → {o1:?}"
    );
}

/// ⚠️ A camada não atravessa o tonemap. Hoje ele é `clamp` (bypass) e a conta fecha; se o AgX
/// ligar, a camada filtrada sai com outra cor que o resto da cena — este gate reprova primeiro.
#[test]
fn the_layer_skips_a_tonemap_that_is_a_bypass() {
    // O mapeamento mudou para o `tonemap_conta.wgsl` (Luz 2D W3, no `main`).
    let wgsl = include_str!("../../ph2d-render/src/shaders/tonemap_conta.wgsl");
    assert!(
        wgsl.contains("const BYPASS_LUT: bool = true;"),
        "o tonemap deixou o bypass: a camada filtrada (motion_filter) tem de passar por ele"
    );
}

/// ⭐ **Os rótulos do `fx.filter` não colidem onde se encontram** (doc 129 §5.5 U1: «três Filter e
/// dois Glow»): o nome do nó é único no catálogo de nós; as linhas do cartão são únicas e nenhuma
/// repete o nome do nó; e nenhum EFEITO do seletor `Effect` é o NOME de outro nó (o nó `fx.glow` e
/// a opção «Glow» eram dois efeitos diferentes com o mesmo nome; idem o `fx.drop_shadow`, que faz
/// CÓPIAS e passou a «Shadow Copies»). ⚠️ Os MODOS (Proximity, Smooth…) são palavras de modo, não
/// efeitos: o «Proximity» da queda e o nó de campo «Proximity» não se confundem.
#[test]
fn the_filter_labels_are_unique_where_they_meet() {
    use ph2d_node_registry::{NodeRegistry, ParamWidget};
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nós registam");
    let nomes: Vec<(String, &str)> = reg
        .manifests()
        .filter_map(|m| {
            reg.ui_manifest(m.id)
                .map(|u| (ph2d_i18n::tr(u.display_key).to_lowercase(), m.name))
        })
        .collect();
    let id = ph2d_node_fx_filter::MANIFEST.id;
    let nome = ph2d_i18n::tr(reg.ui_manifest(id).expect("ficha").display_key).to_lowercase();
    let iguais: Vec<&str> = nomes
        .iter()
        .filter(|(n, _)| *n == nome)
        .map(|(_, t)| *t)
        .collect();
    assert_eq!(
        iguais,
        vec![ph2d_node_fx_filter::TYPE_NAME],
        "o nome «{nome}» repete-se"
    );
    let fichas = reg.param_ui(id).expect("fichas");
    let mut linhas: Vec<String> = fichas
        .iter()
        .map(|h| ph2d_i18n::tr(h.label).to_lowercase())
        .collect();
    assert!(
        !linhas.contains(&nome),
        "uma linha do cartão diz o nome do nó: {linhas:?}"
    );
    linhas.sort();
    let n = linhas.len();
    linhas.dedup();
    assert_eq!(linhas.len(), n, "duas linhas do cartão com o mesmo rótulo");
    for h in fichas
        .iter()
        .filter(|h| h.param == ph2d_node_fx_filter::param::KIND)
    {
        let ParamWidget::Enum { labels } = h.widget else {
            continue;
        };
        for chave in labels {
            let opcao = ph2d_i18n::tr(chave).to_lowercase();
            let outro: Vec<&str> = nomes
                .iter()
                .filter(|(n, _)| *n == opcao)
                .map(|(_, t)| *t)
                .collect();
            assert!(
                outro.is_empty(),
                "a opção «{opcao}» de `{}` é o nome do nó {outro:?}",
                h.param
            );
        }
    }
}

/// ⭐ **Formas fora da camada: o cartão de cada filtro di-lo** (doc 129 §5.7) — a foto da `=135`
/// com a saída em `Add` mostrou os discos sem relevo nem sombra, e nada o dizia. A lei: há cópias
/// vectoriais e a placa não as desenhou.
#[test]
fn shapes_drawn_off_the_layer_are_named_on_every_filter_card() {
    assert!(formas_fora_da_camada(3, false));
    assert!(!formas_fora_da_camada(3, true), "a placa desenhou-as");
    assert!(!formas_fora_da_camada(0, false), "sem formas, nada a dizer");

    let mut g = Graph::new();
    let um = g.add_node(ph2d_node_fx_filter::TYPE_NAME.to_string());
    let dois = g.add_node(ph2d_node_fx_filter::TYPE_NAME.to_string());
    let solto = g.add_node(ph2d_node_fx_filter::TYPE_NAME.to_string());
    let saida = g.add_node("motion.output".to_string());
    let liga = |g: &mut Graph, de, para| {
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (de, 0),
            to: (para, 0),
            delayed: false,
        })
        .expect("liga")
    };
    liga(&mut g, um, dois);
    liga(&mut g, dois, saida);
    let mut f = MotionFiltro::default();
    f.prepara(&g);
    assert!(f.liga(true));
    assert!(f.queixa(um).is_none(), "sem formas fora, o cartão cala");
    f.formas_fora = true;
    for no in [um, dois] {
        let q = f.queixa(no).expect("o cartão diz");
        assert!(q.contains("Blend") && q.contains("skip"), "{q}");
    }
    let q = f.queixa(solto).expect("o solto diz o seu");
    assert!(
        q.contains("Not on the wire"),
        "a queixa de fora do fio vem primeiro: {q}"
    );
    // O quadro seguinte sem o Motion na cena apaga-a.
    assert!(!f.liga(false));
    assert!(f.queixa(um).is_none());
}
