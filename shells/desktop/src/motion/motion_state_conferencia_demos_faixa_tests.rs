//! Gates da cena `=79` — **a faixa que o nome promete** (doc 89, folha 06).
//!
//! ⚠️ **A cena tem um modo de falhar que nenhuma das outras tem: ela pode desenhar
//! o DEFEITO e o CONSERTO iguais.** Os três primeiros pares são o mesmo pedido
//! escrito de duas maneiras, e nas formas bipolares as duas metades *têm* de
//! coincidir — então um gate que exigisse «as metades diferem» reprovaria sobre
//! produto correto em metade da tabela. O que se afirma aqui é a lei toda: nas
//! bipolares COINCIDEM, nas unipolares DIVERGEM, e o lado direito encosta nas duas
//! marcas em todas.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// O Y de cada peça de uma banda, no instante `t`.
fn ys_at(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId, t: f64) -> Vec<f32> {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, t).expect("a cena coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida e um stream")
    };
    match Stream::get(s, "P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => Vec::new(),
    }
}

/// A caixa `[piso, tecto]` de uma banda ao longo de um período inteiro, MENOS a
/// linha de base da fileira (que a tabela conhece por índice).
fn swing(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId, base: f32) -> [f32; 2] {
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for k in 0..=120 {
        for y in ys_at(doc, reg, sink, f64::from(k) / 30.0) {
            lo = lo.min(y - base);
            hi = hi.max(y - base);
        }
    }
    [lo, hi]
}

/// A linha de base da fileira `r` — a MESMA expressão que o construtor usa.
fn base_of(r: usize) -> f32 {
    (ROWS as f32 - 1.0) * 0.5 * ROW_GAP - r as f32 * ROW_GAP
}

/// As sinks das FILEIRAS (sem as marcas), na ordem da tabela.
fn band_sinks(sinks: &[NodeId]) -> Vec<NodeId> {
    let mut out = Vec::new();
    let mut k = 0;
    for row in ROWS_TABLE {
        out.push(sinks[k]);
        k += 1;
        if row.ticks {
            k += 1; // a régua da fileira
        }
    }
    assert_eq!(k, sinks.len(), "a contagem de sinks bate com a tabela");
    out
}

/// **A cena constrói as oito fileiras e as seis marcas.**
#[test]
fn the_band_scene_builds_every_row_and_every_ruler() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_band_demo_document(&mut doc, &reg).expect("a cena constroi");
    let rulers = ROWS_TABLE.iter().filter(|r| r.ticks).count();
    assert_eq!(sinks.len(), ROWS_TABLE.len() + rulers, "fileiras + reguas");
    let (n, min, max, _) = authored();
    assert_eq!(n, ROWS_TABLE.len(), "o anuncio conta a mesma tabela");
    assert!(min < max, "a faixa tem de ter largura");
    assert!(
        (min + max).abs() > 0.05,
        "a faixa e' ASSIMETRICA de proposito: uma centrada no zero esconderia \
         metade do defeito (um piso levantado ao centro cairia em 0)"
    );
}

/// **AS MARCAS FICAM EXACTAMENTE EM `min` E `max`.**
///
/// ⚠️ Sem este gate a régua é decorativa: ela pode estar em qualquer sítio e o
/// olho num smoke rápido aceita, porque uma marca não tem com o que ser comparada.
/// É a régua que julga a cena inteira, então é ela que precisa de ser julgada.
#[test]
fn the_ruler_marks_sit_exactly_on_min_and_max() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_band_demo_document(&mut doc, &reg).expect("a cena constroi");
    let (_, min, max, _) = authored();
    let mut k = 0;
    for (r, row) in ROWS_TABLE.iter().enumerate() {
        k += 1;
        if !row.ticks {
            continue;
        }
        let ys = ys_at(&doc, &reg, sinks[k], 0.0);
        k += 1;
        assert_eq!(ys.len(), 2, "fileira {r}: a regua tem duas marcas");
        let base = base_of(r);
        let (lo, hi) = (ys[0].min(ys[1]) - base, ys[0].max(ys[1]) - base);
        assert!(
            (lo - min).abs() < 1e-5,
            "fileira {r}: marca de baixo em {lo}"
        );
        assert!(
            (hi - max).abs() < 1e-5,
            "fileira {r}: marca de cima em {hi}"
        );
    }
}

/// **O LADO DIREITO ENCOSTA NAS DUAS MARCAS — nas três formas.** É a promessa
/// inteira da régua nova, medida onde o Enio a lê.
///
/// ⚠️ A folga é maior no ruído do que na onda, e o motivo é a fixture e não o
/// produto: uma onda periódica visita os extremos dela dentro de um período; um
/// campo de ruído sobre 26 amostras não visita os dele. A barra do ruído é a
/// fração da faixa que 26 amostras de facto cobrem.
#[test]
fn the_right_hand_side_touches_both_marks() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_band_demo_document(&mut doc, &reg).expect("a cena constroi");
    let bands = band_sinks(&sinks);
    let (_, min, max, _) = authored();
    for (r, row) in ROWS_TABLE.iter().enumerate() {
        let (by_range, tol) = match row.kind {
            Kind::Osc { by_range, .. } => (by_range, 0.02),
            // O ruído não encosta: ele CABE. A barra afirma o continente e um
            // preenchimento mínimo, que é o que 26 amostras podem provar.
            Kind::Noise { by_range, .. } => (by_range, 0.30),
            Kind::Drive { .. } => continue,
        };
        if !by_range {
            continue;
        }
        let [lo, hi] = swing(&doc, &reg, bands[r], base_of(r));
        assert!(
            lo >= min - 1e-3 && hi <= max + 1e-3,
            "fileira {r}: saiu da faixa [{min}, {max}]: [{lo}, {hi}]"
        );
        assert!(
            (lo - min).abs() < tol && (hi - max).abs() < tol,
            "fileira {r}: nao encostou nas marcas: [{lo}, {hi}] contra [{min}, {max}]"
        );
    }
}

/// **NAS BIPOLARES AS DUAS METADES COINCIDEM; NAS UNIPOLARES DIVERGEM — e é o piso
/// que diverge.**
///
/// ⚠️ **É a lei inteira num gate, e as duas metades são necessárias.** Só a
/// segunda faria a cena parecer dizer *"a conta do artista está sempre errada"*,
/// que é falso e injusto. Só a primeira não provaria nada. E o gate mede o PISO em
/// vez da excursão porque é lá que o defeito mora: a conta de cabeça acerta o topo
/// por acidente (o topo natural é `1` nos dois casos), e é isso que a torna
/// invisível.
#[test]
fn the_head_arithmetic_agrees_where_it_is_right_and_lifts_the_floor_where_it_is_not() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_band_demo_document(&mut doc, &reg).expect("a cena constroi");
    let bands = band_sinks(&sinks);
    let (_, min, max, _) = authored();
    let centre = (min + max) * 0.5;
    // Os pares são fileiras VIZINHAS: `(esquerda, direita, unipolar?, barra)`.
    //
    // ⚠️ **A barra é POR-PAR, e é derivada de uma medição — não escolhida.** Uma
    // barra única de 25% da faixa reprovou o par do ruído sobre produto CORRECTO:
    // os pisos divergem `0,119` de uma faixa de `0,60` (**19,8%**), e não mais,
    // porque 26 amostras de um `Ridged` não visitam o piso da forma (o piso
    // empírico dele a 3 oitavas é `≈0,1`, não `0` — medido em
    // `motion.noise::measure_natural_range`). A onda periódica não tem esse
    // problema: ela visita os extremos dela dentro de um período, e por isso
    // aguenta a barra alta.
    for (a, b, unipolar, bar) in [
        (0, 1, false, 0.0f32),
        (2, 3, true, 0.25),
        (4, 5, true, 0.15),
    ] {
        let head = swing(&doc, &reg, bands[a], base_of(a));
        let ruled = swing(&doc, &reg, bands[b], base_of(b));
        if unipolar {
            // O piso da conta de cabeça sobe ao CENTRO; o da régua desce à marca.
            assert!(
                (head[0] - centre).abs() < 0.2 * (max - min),
                "par {a}/{b}: o piso da conta de cabeca devia subir ao centro, deu {}",
                head[0]
            );
            assert!(
                head[0] - ruled[0] > bar * (max - min),
                "par {a}/{b}: os dois pisos tem de divergir mais que {bar} da faixa: \
                 {} contra {}",
                head[0],
                ruled[0]
            );
        } else {
            // ⚠️ A metade que impede a cena de mentir sobre o artista.
            assert!(
                (head[0] - ruled[0]).abs() < 0.02 && (head[1] - ruled[1]).abs() < 0.02,
                "par {a}/{b}: numa forma BIPOLAR as duas metades tem de coincidir: \
                 {head:?} contra {ruled:?}"
            );
        }
    }
}

/// **O PAR DO `drive`: o `Add` SOBE a rampa inteira, o `Min` ACHATA-A num tecto.**
///
/// ⚠️ O oráculo é o TECTO, não a excursão: um `Min` que estivesse a somar também
/// mudaria a excursão. O que só o tecto responde é *o valor virou um limite?*
#[test]
fn the_drive_pair_lifts_on_add_and_clips_on_min() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_band_demo_document(&mut doc, &reg).expect("a cena constroi");
    let bands = band_sinks(&sinks);
    let (_, _, _, v) = authored();
    let (a, b) = (6, 7);
    let add = swing(&doc, &reg, bands[a], base_of(a));
    let clip = swing(&doc, &reg, bands[b], base_of(b));
    assert!(
        (clip[1] - v).abs() < 1e-4,
        "o Min tem de encostar o tecto em {v}, deu {}",
        clip[1]
    );
    assert!(
        add[1] > clip[1] + 0.2,
        "o Add tem de subir acima do tecto: {} contra {}",
        add[1],
        clip[1]
    );
    // E os dois têm forma — uma fileira plana não é um gráfico.
    for (k, s) in [(a, add), (b, clip)] {
        assert!(s[1] - s[0] > 0.1, "fileira {k} e' chata: {s:?}");
    }
}

/// **Nenhuma fileira invade a vizinha** — a mesma lei das cenas irmãs.
#[test]
fn no_row_climbs_into_its_neighbour() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_band_demo_document(&mut doc, &reg).expect("a cena constroi");
    let bands = band_sinks(&sinks);
    for (r, sink) in bands.iter().enumerate() {
        let [lo, hi] = swing(&doc, &reg, *sink, base_of(r));
        assert!(
            hi - lo < ROW_GAP,
            "fileira {r} percorre {}, mais que o vao de {ROW_GAP}",
            hi - lo
        );
    }
}
