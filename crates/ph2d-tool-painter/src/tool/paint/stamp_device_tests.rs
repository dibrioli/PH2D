//! Os gates da metade do TOOL da ponte para o dispositivo.
//!
//! ⚠️ **Nenhum deles precisa de GPU, e isso é o desenho.** A paridade do kernel mora na
//! `ph2d-paint-gpu` (contra a função REAL do Painter, `#[ignore]` + adapter); aqui a pergunta é
//! outra e é toda de FIAÇÃO: *o predicado recusa o que o kernel não transcreve?* · *a região que sai
//! e volta é exatamente a que o lote declara?* · *e quando a ponte declina, o lote volta para a CPU
//! sem perder um byte?* Uma ponte falsa responde as três melhor que uma placa de vídeo, porque ela
//! pode MENTIR de propósito.

use super::{DeviceStamp, DeviceStampJob, LutCache, build_lut, device_dabs, eligible};
use ph2d_editor_core::tool::{CanvasPaintTool as _, PointerPhase, RasterEditTool as _};
use ph2d_painter_brush::{BrushBlend, BrushSpec, Dab, DrawTo, Falloff, StrokeMethod};
use std::sync::{Arc, Mutex};

fn cpx(pos: [f32; 2], phase: PointerPhase) -> ph2d_editor_core::tool::CanvasPointer {
    ph2d_editor_core::tool::CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O tool na armação do smoke: Digital de fábrica, elipse viva sobre 1024².
///
/// ⚠️ **A figura tem de estar ACIMA do piso de redundância** ([`super::MIN_REDUNDANCY`]), senão o
/// lote fica legitimamente na CPU e todo gate da ponte vira vácuo. A primeira versão usava um
/// pincel de raio 40 numa elipse larga — redundância abaixo do piso —, e três gates ficaram
/// VERMELHOS no instante em que o piso entrou. *Foi a fixture que deixou de conter o fenômeno, não
/// o produto que quebrou.*
fn figure_tool() -> crate::tool::PainterTool {
    let mut t = crate::tool::PainterTool::default();
    t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
    t.set_brush_size_px(70.0); // ⚠️ este setter é o RAIO, não o diâmetro
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    t
}

/// Uma figura compacta: o `Down` é o CENTRO da elipse, o `Move` dá os semi-eixos.
fn draw(t: &mut crate::tool::PainterTool) {
    t.on_canvas_pointer(cpx([512.0, 512.0], PointerPhase::Down));
    t.on_canvas_pointer(cpx([692.0, 652.0], PointerPhase::Move));
}

/// **Cada recusa nomeia uma LEI que o kernel não transcreve** — e o controle positivo é o pincel de
/// fábrica, que TEM de passar.
///
/// ⚠️ Sem o controle, um predicado cravado em `false` deixaria as três metades negativas verdes e a
/// wave inteira inerte no produto.
#[test]
fn the_predicate_refuses_exactly_the_laws_the_kernel_does_not_carry() {
    assert!(
        eligible(&BrushSpec::default()),
        "o pincel de FÁBRICA tem de ser elegível — sem isto a rota nasce morta"
    );

    let multiply = BrushSpec {
        blend: BrushBlend::Multiply,
        ..BrushSpec::default()
    };
    assert!(
        !eligible(&multiply),
        "um blend que não é o Mix são outras 23 leis"
    );

    let pigment = BrushSpec {
        watercolor: true,
        pigment: true,
        pigment_mix: 0.5,
        ..BrushSpec::default()
    };
    assert!(
        !eligible(&pigment),
        "o crossfade RYB do pigmento é outra lei, e ela lê o alfa do destino"
    );

    let mut body = BrushSpec {
        impasto: true,
        impasto_depth: 0.5,
        impasto_draw_to: DrawTo::ColorAndDepth,
        impasto_smooth_edges: true,
        ..BrushSpec::default()
    };
    assert!(
        !eligible(&body),
        "o AA do filme é um PASSE de nove amostras, não um valor tabelável"
    );
    // …e o MESMO pincel sem o Smooth Edges passa: é o AA que recusa, não o impasto.
    body.impasto_smooth_edges = false;
    assert!(
        eligible(&body),
        "o depósito do Impasto entra pela TABELA — só o AA fica fora"
    );
}

/// **O filme entra na TABELA, e é por isso que o Impasto não precisa ser excluído.**
///
/// A tabela é preenchida com `film_coverage(deposits_height, falloff_weight(t))`, as funções que o
/// `stamp_band` chama. O gate compara nó a nó contra elas — e exige que as duas tabelas DIFIRAM,
/// senão a dobra do filme seria um no-op e o gate seria verde por vácuo.
#[test]
fn the_table_carries_the_film_when_the_brush_lays_body() {
    let flat = BrushSpec::default();
    let body = BrushSpec {
        impasto: true,
        impasto_depth: 0.5,
        impasto_draw_to: DrawTo::ColorAndDepth,
        ..BrushSpec::default()
    };
    assert!(!flat.deposits_height() && body.deposits_height());

    for (spec, tag) in [(&flat, "plano"), (&body, "corpo")] {
        let lut = build_lut(spec);
        assert_eq!(lut.len(), super::LUT_NODES);
        for i in [0usize, 1, 4096, 33_333, super::LUT_NODES - 1] {
            #[allow(clippy::cast_precision_loss)]
            let t = i as f32 / (super::LUT_NODES - 1) as f32;
            let want = ph2d_painter_brush::height::film_coverage(
                spec.deposits_height(),
                spec.falloff_weight(t),
            );
            assert!(
                (lut[i] - want).abs() < f32::EPSILON,
                "{tag}: nó {i} vale {} e a lei do produto diz {want}",
                lut[i]
            );
        }
    }
    assert!(
        build_lut(&flat) != build_lut(&body),
        "as duas tabelas saíram IGUAIS — a dobra do filme seria um no-op e este gate, vácuo"
    );
}

/// **A tabela é reconstruída só quando a LEI que a define muda** — nunca por dab.
///
/// ⚠️ O oráculo é a CONTAGEM de construções, não o conteúdo: *"a tabela mudou"* não separa
/// **reconstruiu** de **reconstruiu e deu o mesmo**, e comparar endereços de `Vec` mediria o
/// alocador.
#[test]
fn the_table_is_rebuilt_only_when_the_law_that_defines_it_moves() {
    let mut cache = LutCache::default();
    let mut b = BrushSpec::default();
    let _ = cache.get(&b);
    assert_eq!(cache.builds(), 1, "a primeira construção tem de acontecer");
    let _ = cache.get(&b);
    assert_eq!(cache.builds(), 1, "o mesmo pincel não reconstrói");

    // O que muda POR DAB — e que reconstruiria 65 536 nós a cada disco se entrasse na chave.
    b.radius_px = 137.0;
    b.color = [0.9, 0.1, 0.2];
    let _ = cache.get(&b);
    assert_eq!(
        cache.builds(),
        1,
        "raio e cor mudam por DAB: eles não podem tocar a tabela"
    );

    b.hardness = 0.42;
    let _ = cache.get(&b);
    assert_eq!(cache.builds(), 2, "a dureza É a lei do perfil");
    b.falloff = Falloff::Sphere;
    let _ = cache.get(&b);
    assert_eq!(cache.builds(), 3, "o preset de falloff É a lei do perfil");
    b.impasto = true;
    b.impasto_depth = 0.5;
    b.impasto_draw_to = DrawTo::ColorAndDepth;
    let _ = cache.get(&b);
    assert_eq!(cache.builds(), 4, "o filme entra na tabela, logo é a lei");

    // ⚠️ A curva editável NUNCA é cacheada: ela não cabe na chave, então um cache a congelaria no
    // perfil de quando o artista abriu o card.
    b.falloff = Falloff::Custom;
    let before = cache.builds();
    let _ = cache.get(&b);
    let _ = cache.get(&b);
    assert_eq!(
        cache.builds(),
        before + 2,
        "a Custom tem de reconstruir SEMPRE — a curva é editável e não entra na chave"
    );
}

type SpyLog = Arc<Mutex<Vec<(u32, u32, u32, u32, usize)>>>;

/// Uma ponte que grava o que recebeu e devolve o que o teste mandar.
fn spy(
    answer: impl Fn(&DeviceStampJob<'_>) -> Option<Vec<u8>> + Send + 'static,
) -> (DeviceStamp, SpyLog) {
    let log: SpyLog = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    let bridge: DeviceStamp = Box::new(move |job| {
        sink.lock()
            .expect("mutex")
            .push((job.x, job.y, job.w, job.h, job.dabs.len()));
        answer(job)
    });
    (bridge, log)
}

/// **Quando a ponte DECLINA, o lote volta para a CPU e não perde um byte.**
///
/// É a metade que torna a wave segura: o modo de falha de um readback perdido, de um adaptador que
/// some ou de um caso que o kernel não cobre é **lento, nunca errado**. O oráculo é a tinta que a
/// CPU teria pintado sozinha — não um limiar.
///
/// ⚠️ E a ponte foi CHAMADA: sem essa metade o gate ficaria verde com a rota do device desligada,
/// que é exatamente o que ele existe para não deixar passar.
#[test]
fn a_bridge_that_declines_hands_the_batch_back_to_the_cpu_byte_for_byte() {
    let mut plain = figure_tool();
    draw(&mut plain);

    let mut with_bridge = figure_tool();
    let (bridge, log) = spy(|_| None);
    with_bridge.set_device_stamp(Some(bridge));
    draw(&mut with_bridge);

    assert!(
        !log.lock().expect("mutex").is_empty(),
        "a ponte nem foi chamada — o gate estaria verde sobre a rota desligada"
    );
    assert!(
        plain.canvas_rgba.iter().any(|&b| b != 255),
        "a fixture não pintou — ela não contém o fenômeno"
    );
    assert_eq!(
        plain
            .canvas_rgba
            .iter()
            .zip(with_bridge.canvas_rgba.iter())
            .filter(|(a, b)| a != b)
            .count(),
        0,
        "uma ponte que recusa mudou a tinta"
    );
}

/// **A ponte é de fato USADA, e escreve EXATAMENTE a região que declarou.**
///
/// A ponte falsa devolve a região invertida byte a byte — uma resposta que nenhum pincel produz —,
/// então todo pixel que muda é uma escrita dela. O gate afirma as duas metades: dentro do retângulo
/// tudo mudou; fora dele, nada.
#[test]
fn the_bridge_writes_exactly_the_region_it_was_handed() {
    let before = figure_tool().canvas_rgba.as_ref().clone();
    let mut t = figure_tool();
    let (bridge, log) = spy(|job| Some(job.base.iter().map(|b| !b).collect()));
    t.set_device_stamp(Some(bridge));
    draw(&mut t);

    let calls = log.lock().expect("mutex").clone();
    assert!(!calls.is_empty(), "a ponte não foi chamada");
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    for (x, y, w, h, dabs) in &calls {
        assert!(
            *dabs > 0,
            "um lote sem disco nenhum não devia ser publicado"
        );
        x0 = x0.min(*x);
        y0 = y0.min(*y);
        x1 = x1.max(x + w);
        y1 = y1.max(y + h);
    }
    let mut outside = 0usize;
    let mut inside_same = 0usize;
    for (i, (a, b)) in before
        .chunks_exact(4)
        .zip(t.canvas_rgba.chunks_exact(4))
        .enumerate()
    {
        #[allow(clippy::cast_possible_truncation)]
        let (x, y) = ((i % 1024) as u32, (i / 1024) as u32);
        if x >= x0 && x < x1 && y >= y0 && y < y1 {
            inside_same += usize::from(a == b);
        } else {
            outside += usize::from(a != b);
        }
    }
    assert_eq!(
        outside, 0,
        "{outside} pixels mudaram FORA da região declarada"
    );
    // Um pixel invertido só coincidiria se valesse `!self` em todo canal, o que 0xFF não vale.
    assert_eq!(
        inside_same, 0,
        "{inside_same} pixels DENTRO da região não foram escritos pela ponte"
    );
}

/// **A rota do device é tomada pelo pincel do ARTISTA** — e o instrumento a CONTA.
///
/// ⚠️ O balde existe pela mesma razão que o `banded`/`serial`: sem ele, *"não melhorou"* admite duas
/// leituras opostas (a rota não é tomada · é tomada e o tempo está noutro lugar), e as curas são
/// opostas. O controle negativo é o mesmo tool sem ponte.
#[test]
fn the_artists_default_brush_takes_the_device_road_when_a_bridge_is_installed() {
    let _ = super::super::stamp_banded::diag::take();
    let mut t = figure_tool();
    let (bridge, _log) = spy(|job| Some(job.base.to_vec()));
    t.set_device_stamp(Some(bridge));
    draw(&mut t);
    let d = super::super::stamp_banded::diag::take();
    assert!(
        d.device > 0,
        "com ponte instalada o pincel default NÃO alcança o device: {} no device, {} em banda",
        d.device,
        d.banded
    );

    let _ = super::super::stamp_banded::diag::take();
    let mut t = figure_tool();
    draw(&mut t);
    let d = super::super::stamp_banded::diag::take();
    assert_eq!(
        d.device, 0,
        "sem ponte nenhum lote pode ir ao device: {} contados",
        d.device
    );
    assert!(d.banded > 0, "o controle não carimbou pela rota em banda");
}

/// **Um pincel que o predicado recusa nunca chega à ponte** — a recusa é do TOOL, não do device.
#[test]
fn a_brush_outside_the_predicate_never_reaches_the_bridge() {
    let mut t = figure_tool();
    t.paint.brush.blend = BrushBlend::Multiply;
    let (bridge, log) = spy(|job| Some(job.base.to_vec()));
    t.set_device_stamp(Some(bridge));
    draw(&mut t);
    assert!(
        log.lock().expect("mutex").is_empty(),
        "um blend Multiply chegou ao device, que só transcreve o Mix"
    );
    assert!(
        t.canvas_rgba.iter().any(|&b| b != 255),
        "o controle não pintou — a fixture não separa 'recusou' de 'não pintou'"
    );
}

/// O mapa do footprint que sai para o device é o do `brush`, e concorda com o do spec POR-DAB que o
/// laço serial constrói — a premissa que dispensa clonar um `BrushSpec` por disco.
#[test]
fn the_footprint_of_the_batch_is_the_footprint_of_each_dab() {
    let brush = BrushSpec {
        dab_flatten: 0.55,
        dab_angle_deg: 31,
        ..BrushSpec::default()
    };
    let dabs: Vec<Dab> = (0u8..4)
        .map(|i| Dab {
            center: [10.0 * f32::from(i), 20.0],
            radius_px: 7.0 + f32::from(i),
            coverage: 0.5,
            color: [0.1 * f32::from(i), 0.2, 0.3],
            rotation: [1.0, 0.0],
            dir: [1.0, 0.0],
            arc_len: 0.0,
            stroke_radius_px: 9.0,
        })
        .collect();
    for (d, dev) in dabs.iter().zip(device_dabs(&dabs, &brush)) {
        // O spec por-dab do laço serial — o que o produto de fato monta.
        let per_dab = BrushSpec {
            radius_px: d.radius_px,
            color: d.color,
            ..brush
        };
        let fp = per_dab.dab_footprint(per_dab.dab_rotor(d));
        let (e0, e1) = (fp.apply([1.0, 0.0]), fp.apply([0.0, 1.0]));
        assert_eq!(dev.m0, [e0[0], e1[0]]);
        assert_eq!(dev.m1, [e0[1], e1[1]]);
        assert_eq!(dev.radius, d.radius_px);
        assert_eq!(dev.color, d.color);
    }
}

/// **O piso de redundância existe porque a rota do device PERDE sem ele** — e essa é a metade que
/// nenhum gate de correção pega.
///
/// As duas rotas escalam com grandezas diferentes (a fronteira com a ÁREA da região, a CPU com as
/// VISITAS), então há um regime em que subir a janela custa mais que carimbá-la. Medido pela porta
/// do artista: sem piso, uma figura de redundância 0,3× fica **4,5× MAIS LENTA** no device.
///
/// ⚠️ O gate afirma a PROPRIEDADE nos dois lados — o lote dos discos empilhados sobe, o lote
/// espalhado não —, e o oráculo é o próprio `wants_device`, que é a porta que o produto consulta.
/// Recomputar a regra aqui deixaria o gate verde com o produto decidindo outra coisa.
#[test]
fn a_spread_out_batch_stays_on_the_cpu_and_a_stacked_one_does_not() {
    let dab = |c: [f32; 2], r: f32| Dab {
        center: c,
        radius_px: r,
        coverage: 0.6,
        color: [0.2, 0.3, 0.9],
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: r,
    };
    // EMPILHADOS: 60 discos de raio 100 num arco de raio 120 — muita visita, região pequena.
    let stacked: Vec<Dab> = (0..60)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let t = (i as f32) / 60.0 * std::f32::consts::TAU;
            dab([800.0 + t.cos() * 120.0, 800.0 + t.sin() * 120.0], 100.0)
        })
        .collect();
    // ESPALHADOS: os MESMOS 60 discos, o mesmo trabalho, numa região dez vezes maior. ⚠️ A tela é
    // folgada de propósito: um arco que encosta na borda tem as pegadas RECORTADAS, e aí os dois
    // lotes deixariam de custar o mesmo — a fixture pararia de isolar a região.
    let spread: Vec<Dab> = (0..60)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let t = (i as f32) / 60.0 * std::f32::consts::TAU;
            dab([800.0 + t.cos() * 620.0, 800.0 + t.sin() * 620.0], 100.0)
        })
        .collect();
    assert!(
        super::wants_device(&stacked, 1600, 1600),
        "um lote empilhado (muita visita sobre pouca região) TEM de subir — é o caso do report"
    );
    assert!(
        !super::wants_device(&spread, 1600, 1600),
        "um lote espalhado sobe a mesma região por muito menos trabalho: no device ele PERDE, \
         medido em 4,5× no pior caso da varredura"
    );
    // E o mesmo trabalho nos dois: o que difere é a REGIÃO, não a quantidade de tinta.
    let (a, b) = (
        super::super::stamp_banded::batch_work(&stacked, 1600, 1600),
        super::super::stamp_banded::batch_work(&spread, 1600, 1600),
    );
    // ⚠️ **Essencialmente o mesmo, não idêntico:** `dab_write_bounds` devolve uma caixa INTEIRA, e
    // dois arcos de raios diferentes pousam os centros em frações distintas de pixel — as caixas
    // diferem por um pixel aqui e ali. Medido: 0,03%. Exigir igualdade exata seria um gate que
    // reprova por aritmética de arredondamento, não pela propriedade que ele afirma.
    let (lo, hi) = (a.min(b) as f64, a.max(b) as f64);
    assert!(
        hi / lo < 1.01,
        "a fixture não isola a região: os dois lotes têm de custar o MESMO trabalho ({a} x {b})"
    );
}
