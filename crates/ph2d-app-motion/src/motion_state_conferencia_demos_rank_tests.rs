//! Os gates da cena `=73` — corte, banda, rampa e forma.
//!
//! ⚠️ **Um destes gates é de LAYOUT, e ele é o que faltava.** Os outros afirmam que
//! cada par difere no número que anuncia e só nele; nenhum deles teria visto a cena
//! ilegível que o Enio recebeu, porque o defeito não estava em param nenhum — estava
//! em ONDE as peças foram parar. Ver `no_band_leaves_its_slot`.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::NodeId;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn scene() -> (MotionDoc, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_rank_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipado");
    (doc, sinks)
}

fn nodes_of(doc: &MotionDoc, ty: &str) -> Vec<NodeId> {
    doc.graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == ty)
        .map(|n| n.id)
        .collect()
}

fn param(doc: &MotionDoc, id: NodeId, name: &str) -> f32 {
    doc.graph
        .node_param_overrides(id)
        .and_then(|m| m.get(name).copied())
        .unwrap_or(f32::NAN)
}

/// A caixa envolvente de uma banda, **cozinhada** — a única leitura honesta de onde
/// as peças de facto ficam.
fn band_box(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> (usize, [f32; 2], [f32; 2]) {
    let mut cook = Cook::new();
    let out = cook.cook(&doc.graph, reg, sink, 0.0).expect("cozinha");
    let s = out[0].as_stream();
    let Some(Column::Vec2(p)) = s.get("P") else {
        panic!("a banda tem de ter geometria")
    };
    assert!(!p.is_empty(), "a banda não pode sair vazia");
    let mut lo = [f32::INFINITY; 2];
    let mut hi = [f32::NEG_INFINITY; 2];
    for q in p {
        for a in 0..2 {
            lo[a] = lo[a].min(q[a]);
            hi[a] = hi[a].max(q[a]);
        }
    }
    (p.len(), lo, hi)
}

// ─────────────────────────────────────────────────────────────────────────────
// O LAYOUT
// ─────────────────────────────────────────────────────────────────────────────

/// **NENHUMA BANDA SAI DO QUADRANTE DELA — e este gate é o que faltava.**
///
/// ⚠️ **Ele nasceu de um smoke reprovado** (Enio, 2026-08-21: *"tudo misturado e
/// bagunçado"*). A causa não estava em param nenhum: eu posicionava cada banda com um
/// `motion.move` **depois** do campo, e todo comportamento desta biblioteca é
/// multiplicado pelo `falloff`. A peça `i` andava `dx · falloff_i` em vez de `dx`, e
/// oito bandas esticavam-se umas por cima das outras. Medido na cena velha: uma
/// fileira de 7,5 de largura saiu com **1,50**; uma grelha de 2,94 saiu com **8,94**.
///
/// ⚠️ **A caixa prevista é DERIVADA** de `BANDS` (`(n − 1) · passo`), nunca escrita ao
/// lado. Escrevê-la à mão seria repor exactamente o erro que o gate existe para
/// apanhar: um segundo número que discorda do primeiro em silêncio.
///
/// A folga de `0,12` cobre o RAIO da peça, que é meio `amount` e não entra no `P`.
#[test]
fn no_band_leaves_its_slot() {
    let (doc, sinks) = scene();
    let reg = registry();
    assert_eq!(sinks.len(), 8);
    for (i, sink) in sinks.iter().enumerate() {
        let (row, right) = (i / 2, i % 2 == 1);
        let (w, h) = footprint(row);
        let cx = if right { COL_X } else { -COL_X };
        let cy = ROW_Y[row];
        let (n, lo, hi) = band_box(&doc, &reg, *sink);
        let slack = 0.12;
        for (a, (c, size)) in [(cx, w), (cy, h)].into_iter().enumerate() {
            let (want_lo, want_hi) = (c - size * 0.5 - slack, c + size * 0.5 + slack);
            assert!(
                lo[a] >= want_lo && hi[a] <= want_hi,
                "banda {} (linha {row}, {}) eixo {a}: [{:.2}..{:.2}] fora de \
                 [{want_lo:.2}..{want_hi:.2}] — n={n}",
                i + 1,
                if right { "direita" } else { "esquerda" },
                lo[a],
                hi[a]
            );
        }
    }
}

/// **AS OITO CAIXAS NÃO SE TOCAM** — a leitura da tela depende disso, e o gate acima
/// sozinho não o garante (duas fatias podiam caber nos quadrantes e ainda encostar-se
/// se os quadrantes se sobrepusessem).
#[test]
fn no_two_slots_overlap() {
    let mut boxes: Vec<([f32; 2], [f32; 2])> = Vec::new();
    for (row, cy) in ROW_Y.iter().enumerate() {
        let (w, h) = footprint(row);
        let cy = *cy;
        for right in [false, true] {
            let cx = if right { COL_X } else { -COL_X };
            boxes.push(([cx - w * 0.5, cy - h * 0.5], [cx + w * 0.5, cy + h * 0.5]));
        }
    }
    // Uma margem MÍNIMA entre quadrantes: encostados eles leem-se como um bloco só.
    let margin = 0.35;
    for a in 0..boxes.len() {
        for b in (a + 1)..boxes.len() {
            let (lo_a, hi_a) = boxes[a];
            let (lo_b, hi_b) = boxes[b];
            let apart = (0..2).any(|k| lo_a[k] - hi_b[k] >= margin || lo_b[k] - hi_a[k] >= margin);
            assert!(
                apart,
                "os quadrantes {} e {} estão a menos de {margin} um do outro: \
                 {lo_a:?}..{hi_a:?} contra {lo_b:?}..{hi_b:?}",
                a + 1,
                b + 1
            );
        }
    }
}

/// **A CENA CABE NO ENQUADRAMENTO** — as legendas incluídas.
///
/// ⚠️ O número não é um palpite: a cena `=71`, que o Enio aprovou, punha conteúdo até
/// `±6` em X e `±5,2` em Y e ele viu tudo. Esta fica dentro disso com folga, e a
/// margem existe para que a próxima linha não seja acrescentada às cegas.
#[test]
fn the_whole_scene_fits_the_frame() {
    let (doc, sinks) = scene();
    let reg = registry();
    let (limit_x, limit_y) = (7.0, 7.0);
    for sink in &sinks {
        let (_, lo, hi) = band_box(&doc, &reg, *sink);
        assert!(
            lo[0] >= -limit_x && hi[0] <= limit_x && lo[1] >= -limit_y && hi[1] <= limit_y,
            "banda fora do enquadramento: [{:.2}..{:.2}] x [{:.2}..{:.2}]",
            lo[0],
            hi[0],
            lo[1],
            hi[1]
        );
    }
}

/// **CADA LINHA TEM O SEU NOME ESCRITO NO CANVAS, e as duas colunas também.**
///
/// ⚠️ Um exemplo que só se entende lendo o terminal não é um exemplo — é um enigma
/// com gabarito noutra folha. FALSIFICADO se alguém remover as legendas *"porque a
/// prosa já diz"*.
#[test]
fn every_row_is_named_on_the_canvas() {
    let (doc, _) = scene();
    let texts = nodes_of(&doc, "source.text");
    assert_eq!(texts.len(), 6, "quatro linhas + as duas colunas");
    let words: Vec<String> = texts
        .iter()
        .filter_map(|t| {
            doc.graph
                .node_text_param_overrides(*t)
                .and_then(|m| m.get(ph2d_node_source_text::TEXT_KEY).cloned())
        })
        .collect();
    for w in ROW_LABELS.iter().chain(["ANTES", "DEPOIS"].iter()) {
        assert!(
            words.iter().any(|s| s == w),
            "falta a legenda `{w}`: {words:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AS QUATRO CÉLULAS
// ─────────────────────────────────────────────────────────────────────────────

/// **A LINHA 1 DIFERE SÓ NO `reindex` — a fracção cortada é a MESMA.**
#[test]
fn the_cull_row_cuts_the_same_half_and_differs_only_in_the_renumbering() {
    let (doc, _) = scene();
    let culls = nodes_of(&doc, "motion.cull");
    assert_eq!(culls.len(), 2);
    for c in &culls {
        assert_eq!(param(&doc, *c, "amount"), KEEP, "a mesma fracção");
        assert_eq!(param(&doc, *c, "mode"), 0.0, "e o mesmo modo (Fraction)");
    }
    assert_eq!(param(&doc, culls[0], "reindex"), 0.0, "esquerda: como era");
    assert_eq!(param(&doc, culls[1], "reindex"), 1.0, "direita: renumera");
}

/// **AS DUAS METADES DA LINHA 1 TÊM O MESMO NÚMERO DE PEÇAS.**
///
/// ⚠️ É a alma do par: se cortassem números diferentes, a diferença de cor teria uma
/// segunda explicação e o smoke leria a errada. Cozinhado, não deduzido.
#[test]
fn both_halves_of_the_cull_row_keep_the_same_piece_count() {
    let (doc, sinks) = scene();
    let reg = registry();
    let a = band_box(&doc, &reg, sinks[0]).0;
    let b = band_box(&doc, &reg, sinks[1]).0;
    assert_eq!(a, b, "esquerda {a} peças, direita {b}");
    assert_eq!(a, 6, "metade de doze");
}

/// **A LINHA 2 DIFERE SÓ NO `key`, E SÓ A DIREITA TEM CAMPO — na porta 1.**
#[test]
fn the_rank_row_changes_only_what_orders_the_band() {
    let (doc, _) = scene();
    let irs = nodes_of(&doc, "field.index_range");
    assert_eq!(irs.len(), 2);
    for k in ["start", "end", "soft", "curve"] {
        assert_eq!(
            param(&doc, irs[0], k),
            param(&doc, irs[1], k),
            "a BANDA tem de ser a mesma; só o que a ordena muda (`{k}`)"
        );
    }
    assert_eq!(param(&doc, irs[0], "key"), 0.0, "esquerda: Index");
    assert_eq!(
        param(&doc, irs[1], "key"),
        ORDER_BY_ATTRIBUTE,
        "direita: Attribute"
    );
    let noises = nodes_of(&doc, "value.noise");
    assert_eq!(noises.len(), 1, "só a direita tem atributo");
    assert!(
        doc.graph
            .edges()
            .iter()
            .any(|e| e.from.0 == noises[0] && e.to == (irs[1], 1) && !e.delayed),
        "o campo tem de alimentar a porta `attr` (índice 1)"
    );
}

/// **O CAMPO DA LINHA 2 DESCORRELACIONA VIZINHOS** — senão a banda "espalhada" sairia
/// em manchas, indistinguíveis de um bloco maior.
#[test]
fn the_rank_attribute_is_decorrelated_across_the_grid() {
    let gap = BANDS[1].2;
    assert!(
        gap * ATTR_FREQ >= 0.5,
        "vizinhas a {} no espaço do ruído: perto demais",
        gap * ATTR_FREQ
    );
}

/// **A LINHA 3 DIFERE SÓ NO `curve_offset`, e o contorno é `Curve` nos DOIS.**
#[test]
fn the_shift_row_changes_only_the_curve_offset() {
    let (doc, _) = scene();
    let rms = nodes_of(&doc, "field.remap");
    assert_eq!(rms.len(), 2);
    for r in &rms {
        assert_eq!(
            param(&doc, *r, "contour"),
            CONTOUR_CURVE,
            "o deslocamento só age no contorno Curve"
        );
    }
    assert_eq!(param(&doc, rms[0], "curve_offset"), 0.0);
    assert_eq!(param(&doc, rms[1], "curve_offset"), CURVE_SHIFT);
}

/// **A RAMPA DA LINHA 3 CHEGA MESMO AO `falloff`.**
///
/// ⚠️ Sem isto, um `channel` errado no `motion.drive` daria duas fileiras idênticas
/// (o remap sobre máscara ausente = 1 constante), e o smoke leria isso como *"o
/// `Curve Offset` não faz nada"* — a conclusão errada sobre o código certo.
#[test]
fn the_shift_row_actually_writes_the_mask_it_then_remaps() {
    let (doc, _) = scene();
    let drives = nodes_of(&doc, "motion.drive");
    assert_eq!(drives.len(), 2, "um por banda da linha 3");
    for d in &drives {
        assert_eq!(param(&doc, *d, "channel"), DRIVE_FALLOFF, "o canal Falloff");
        assert_eq!(param(&doc, *d, "mode"), DRIVE_SET, "Set, não Add");
        assert!(
            doc.graph
                .edges()
                .iter()
                .any(|e| e.to == (*d, 1) && !e.delayed),
            "o valor tem de estar ligado à porta 1 do drive"
        );
    }
    for f in &nodes_of(&doc, "value.instance_field") {
        assert_eq!(
            param(&doc, *f, "mode"),
            FIELD_RAMP,
            "modo Ramp: o índice NORMALIZADO 0..1, não o índice cru"
        );
    }
}

/// **A LINHA 4 DIFERE SÓ NO `Path Mode`, e o pentágono é o MESMO.**
#[test]
fn the_shape_row_changes_only_the_path_mode() {
    let (doc, _) = scene();
    let shapes = nodes_of(&doc, "field.shape");
    assert_eq!(shapes.len(), 2);
    assert_eq!(param(&doc, shapes[0], "mode"), 0.0, "esquerda: Filled Path");
    assert_eq!(param(&doc, shapes[1], "mode"), 1.0, "direita: Path Edges");
    for k in ["distance", "curve"] {
        assert_eq!(param(&doc, shapes[0], k), param(&doc, shapes[1], k));
    }
    let rings = nodes_of(&doc, "motion.distribute_radial");
    assert_eq!(rings.len(), 2, "uma forma por banda");
    for (fs, ring) in shapes.iter().zip(&rings) {
        assert_eq!(param(&doc, *ring, "count"), SHAPE_SIDES);
        assert_eq!(param(&doc, *ring, "radius"), SHAPE_RADIUS);
        // A forma entra pela PORTA 1 — na porta 0 ela seria a arte.
        assert!(
            doc.graph.edges().iter().any(|e| e.to == (*fs, 1)),
            "a forma tem de entrar pela porta `shape`"
        );
    }
}

/// **O PENTÁGONO CABE DENTRO DA GRELHA QUE ELE MASCARA, e está NO MESMO quadrante.**
///
/// ⚠️ Duas armadilhas, e as duas dariam um par verde e mudo: um pentágono maior que a
/// grelha mascararia tudo, e um pentágono deixado na ORIGEM enquanto a grelha está no
/// quadrante não mascararia nada. A conta é derivada de `BANDS`.
#[test]
fn the_pentagon_fits_the_grid_and_shares_its_quadrant() {
    let (doc, _) = scene();
    let (w, h) = footprint(3);
    let half = w.min(h) * 0.5;
    assert!(
        SHAPE_RADIUS + SHAPE_DISTANCE < half,
        "o pentágono (+ penumbra) mede {} contra a meia-largura {half}",
        SHAPE_RADIUS + SHAPE_DISTANCE
    );
    // Toda colocação da linha 4 — a grelha e a forma, de cada lado.
    //
    // ⚠️ **A legenda `FORMA` também vive nesta linha**, centrada no vão (`x = 0`), e a
    // primeira versão deste gate apanhou-a e reprovou código correcto. Ela é excluída
    // pelo que a define — `x = 0` é o vão, nunca um quadrante —, e a exclusão é
    // AFIRMADA logo abaixo em vez de ser silenciosa: um filtro que remove uma linha
    // sem provar que ela existe é como um gate esvazia sem ninguém notar.
    let on_row: Vec<f32> = nodes_of(&doc, "motion.transform")
        .into_iter()
        .filter(|t| (param(&doc, *t, "offset_y") - ROW_Y[3]).abs() < 1e-6)
        .map(|t| param(&doc, t, "offset_x"))
        .collect();
    let in_gutter = on_row.iter().filter(|x| **x == 0.0).count();
    assert_eq!(in_gutter, 1, "a legenda da linha, e só ela, fica no vão");
    let placed: Vec<f32> = on_row.into_iter().filter(|x| *x != 0.0).collect();
    assert_eq!(
        placed.len(),
        4,
        "a grelha e a forma de cada lado: {placed:?}"
    );
    for x in placed {
        assert!(
            (x.abs() - COL_X).abs() < 1e-6,
            "colocação fora da coluna: {x}"
        );
    }
}

/// **AS DUAS METADES DE CADA LINHA TÊM DE SAIR DIFERENTES — COZINHADAS.**
///
/// ⚠️ **Este gate é o terceiro capítulo da mesma lição, e foi o Enio que o pediu de
/// novo** (2026-08-21: *"Em Rampa: Remap: Curve offset ... não tem efeito"*). Os outros
/// gates desta cena afirmam que cada par difere no PARAM que anuncia; nenhum deles
/// olhava a IMAGEM. A linha da RAMPA saía com as duas metades **byte-idênticas** — o
/// `field.remap` com o contorno `Curve` e nada autorado devolvia `t` antes de o
/// deslocamento correr — e os nove gates passavam, porque o param estava lá.
///
/// ⚠️ **A comparação é sobre TODAS as colunas**, não sobre uma escolhida: a linha do
/// corte difere no `tint`, as outras três no `falloff`. Escolher a coluna seria
/// escrever a resposta ao lado da pergunta.
#[test]
fn each_pair_actually_differs_in_the_cooked_result() {
    let (doc, sinks) = scene();
    let reg = registry();
    let cooked = |sink: NodeId| {
        let mut cook = Cook::new();
        let out = cook.cook(&doc.graph, &reg, sink, 0.0).expect("cozinha");
        let s = out[0].as_stream();
        let mut cols: Vec<(String, Column)> =
            s.columns().map(|(n, c)| (n.clone(), c.clone())).collect();
        cols.sort_by(|a, b| a.0.cmp(&b.0));
        cols
    };
    for row in 0..4 {
        let (a, b) = (cooked(sinks[row * 2]), cooked(sinks[row * 2 + 1]));
        assert_eq!(
            a.iter().map(|(n, _)| n).collect::<Vec<_>>(),
            b.iter().map(|(n, _)| n).collect::<Vec<_>>(),
            "linha {row}: as duas metades têm de ter as MESMAS colunas"
        );
        let differs = a.iter().zip(&b).any(|((_, x), (_, y))| x != y);
        assert!(
            differs,
            "linha {row} ({}): as duas metades saíram IDÊNTICAS — o knob não chega à \
             imagem, e um par que não difere não demonstra nada",
            ROW_LABELS[row]
        );
    }
}

/// **O DIAGNOSER DA CASA NÃO ACHA BURACO NESTA CENA.**
#[test]
fn the_house_diagnoser_finds_no_hole_in_this_scene() {
    let (doc, _) = scene();
    let reg = registry();
    let d = ph2d_motion_diagnose::diagnose(&doc.graph, &reg);
    assert!(d.is_empty(), "a cena não encena defeito nenhum: {d:?}");
}
