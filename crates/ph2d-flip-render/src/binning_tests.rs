//! Gates do binning por tile (`binning.rs`).
//!
//! O que precisa ser provado aqui **não é a lei da tinta** (ela chega no passo 3): é que a
//! estrutura de aceleração **não muda a resposta** e **não tem teto** — as duas propriedades que
//! o motor de hoje não tem.

use super::*;
use crate::pack::{FLAG_CLOSED, GpuPoint, GpuStroke};
use crate::tau::{dab_weight, f_of};

/// Uma câmera 1:1 — mundo em px, sem zoom. Mantém os números dos gates legíveis.
fn screen(w: f32, h: f32) -> ScreenSpace {
    ScreenSpace {
        world_to_clip: [
            [2.0 / w, 0.0, 0.0, 0.0],
            [0.0, 2.0 / h, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0, 1.0],
        ],
        viewport: [w, h],
        px_per_world: 1.0,
    }
}

/// `(pontos, largura, fechado, cor)` — a receita de um traço nas fixtures.
type StrokeSpec<'a> = (&'a [[f32; 2]], f32, bool, [f32; 4]);

/// Monta um `FlipGpuData` à mão — o binning só lê `points` e `strokes`.
fn art(strokes: &[StrokeSpec<'_>]) -> FlipGpuData {
    let mut g = FlipGpuData::default();
    for (pts, width, closed, color) in strokes {
        let first = g.points.len() as u32;
        let sid = g.strokes.len() as u32;
        for p in *pts {
            g.points.push(GpuPoint {
                pos: *p,
                width: *width,
                opacity: 1.0,
                color: *color,
            });
            g.point_stroke.push(sid);
            g.arc_len.push(0.0);
            g.seg_extra_range.push([0, 0]);
        }
        g.strokes.push(GpuStroke {
            first_point: first,
            point_count: pts.len() as u32,
            flags: if *closed { FLAG_CLOSED } else { 0 },
            hardness: 1.0,
            material: 0,
            tip: 0,
            dot_spacing: 0.0,
            ref_width: *width,
        });
    }
    g
}

/// Um traço com largura VARIANDO ponto a ponto (o que a pressão produz). O `art` mantém largura
/// constante, e nela `max(r_a, r_b) == min(r_a, r_b)` — ou seja, a fixture não distingue as duas.
fn push_tapered(g: &mut FlipGpuData, pts: &[[f32; 2]], widths: &[f32]) {
    let first = g.points.len() as u32;
    let sid = g.strokes.len() as u32;
    for (p, w) in pts.iter().zip(widths) {
        g.points.push(GpuPoint {
            pos: *p,
            width: *w,
            opacity: 1.0,
            color: BLACK,
        });
        g.point_stroke.push(sid);
        g.arc_len.push(0.0);
        g.seg_extra_range.push([0, 0]);
    }
    g.strokes.push(GpuStroke {
        first_point: first,
        point_count: pts.len() as u32,
        flags: 0,
        hardness: 1.0,
        material: 0,
        tip: 0,
        dot_spacing: 0.0,
        ref_width: widths.iter().copied().fold(0.0, f32::max),
    });
}

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// Todo segmento capaz de tocar um pixel está na lista do ladrilho DAQUELE pixel.
///
/// É a única propriedade de correção que o binning precisa ter — e é sobre PIXEL, não sobre
/// ladrilho, porque é o pixel que o percurso resolve. Força bruta sobre a tela inteira.
#[test]
fn the_bin_never_loses_a_segment_that_can_touch_a_pixel() {
    let (w, h) = (96.0, 64.0);
    let sc = screen(w, h);
    let mut g = art(&[
        (&[[8.0, 8.0], [88.0, 56.0]], 7.0, false, BLACK),
        (
            &[[10.0, 50.0], [50.0, 10.0], [80.0, 40.0]],
            4.0,
            false,
            BLACK,
        ),
        (
            &[[30.0, 20.0], [60.0, 20.0], [60.0, 44.0], [30.0, 44.0]],
            3.0,
            true,
            BLACK,
        ),
        // ⚠️ Um FIO (0,2 de mundo = 0,1 px de raio) — abaixo do piso `MIN_WIDTH_PX`. Sem ele o
        // piso nunca morde na fixture, e um binner que o ignorasse ficaria verde enquanto solta
        // ladrilhos que o percurso quer.
        (&[[74.0, 6.0], [74.0, 58.0]], 0.2, false, BLACK),
    ]);
    // ⚠️ E um traço que AFINA, **a 4 px da fronteira x = 48**: sem largura variando, `max(r_a,r_b)`
    // e `min(r_a,r_b)` dão o mesmo número em toda a fixture; e sem a fronteira perto, o ladrilho de
    // 16 px engole a diferença (o traço corre DENTRO de uma coluna e os dois raios binam igual).
    // Aqui o raio grosso (6) alcança a coluna vizinha e o fino (0,65) não.
    push_tapered(&mut g, &[[44.0, 8.0], [44.0, 56.0]], &[12.0, 1.0]);
    let bins = bin_segments(&g, &sc, 16);

    let mut all = Vec::new();
    for sid in 0..g.strokes.len() as u32 {
        for (a, b) in stroke_segs(&g, sid) {
            all.push(BinSeg { stroke: sid, a, b });
        }
    }
    // 1 (reta) + 2 (dobra) + 3 arestas + a costura + 1 fio + 1 afinando.
    assert_eq!(all.len(), 9, "a fixture mudou de forma");

    let mut checked = 0usize;
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let ti = bins
                .tile_of_pixel(p[0], p[1])
                .expect("pixel dentro da grade");
            let list = bins.segs_of(ti);
            for s in &all {
                let (pa, pb) = (g.points[s.a as usize], g.points[s.b as usize]);
                let sa = sc.point_px(pa.pos);
                let sb = sc.point_px(pb.pos);
                let r = sc.radius_px(pa.width).max(sc.radius_px(pb.width));
                if point_seg_distance(p, sa, sb) <= r {
                    assert!(
                        list.contains(s),
                        "segmento {s:?} alcanca o pixel ({x},{y}) e sumiu da lista do tile {ti}"
                    );
                    checked += 1;
                }
            }
        }
    }
    // Piso de cobertura da fixture (medido: 1510 pares alcançam algum pixel). Não é o valor, é a
    // garantia de que a varredura de fato exercitou o binning em vez de passar por um canvas nu.
    assert!(checked > 1200, "poucos pares conferidos: {checked}");
}

/// A lista acelerada dá **a mesma imagem** que a lista completa. Se isto falhar, o binning deixou
/// de ser estrutura de aceleração e virou uma segunda lei.
#[test]
fn the_binned_walk_is_the_brute_force_walk() {
    let (w, h) = (96.0, 64.0);
    let sc = screen(w, h);
    let g = art(&[
        (
            &[[6.0, 30.0], [90.0, 34.0]],
            9.0,
            false,
            [1.0, 0.0, 0.0, 1.0],
        ),
        (
            &[[20.0, 10.0], [40.0, 55.0], [70.0, 12.0]],
            5.0,
            false,
            [0.0, 0.0, 1.0, 1.0],
        ),
    ]);
    let bins = bin_segments(&g, &sc, 16);
    let mut painted = 0usize;
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let fast = walk_pixel(&bins, &g, &sc, p);
            let slow = TileBins::walk_pixel_brute(&g, &sc, p);
            assert_eq!(fast, slow, "divergiu no pixel ({x},{y})");
            if fast[3] > 0.0 {
                painted += 1;
            }
        }
    }
    assert!(painted > 800, "a fixture mal pintou: {painted} px");
}

/// **O ATESTADO DE ÓBITO DA PROPRIEDADE (B).** O `neighbors.rs` de hoje trunca em
/// `MAX_EXTRAS_PER_SEGMENT = 16`; aqui um ladrilho carrega quantos couberem na memória.
#[test]
fn the_bin_has_no_fixed_ceiling() {
    const OLD_CEILING: usize = 16;
    let (w, h) = (64.0, 64.0);
    let sc = screen(w, h);
    // 24 traços separados atravessando o MESMO ladrilho central.
    let mut lines: Vec<[[f32; 2]; 2]> = Vec::new();
    for i in 0..24 {
        let y = 34.0 + i as f32 * 0.15;
        lines.push([[20.0, y], [44.0, y]]);
    }
    let specs: Vec<StrokeSpec<'_>> = lines.iter().map(|l| (&l[..], 2.0, false, BLACK)).collect();
    let g = art(&specs);
    let bins = bin_segments(&g, &sc, 16);

    // ⚠️ **PERGUNTE onde a tinta cai, não suponha.** A versão anterior cravava o pixel `(34, 36)`
    // porque `linha == y de mundo` era verdade — e era verdade só porque o `point_px` tinha o Y
    // invertido. Uma fixture que crava coordenada de tela **codifica a convenção**, então ela
    // sobrevive ao bug e cai junto com a correção. Derivar do `point_px` deixa o gate falar do que
    // ele é: *a lista de um ladrilho não tem teto*.
    let mid = sc.point_px([32.0, 34.0 + 23.0 * 0.15 / 2.0]);
    let ti = bins
        .tile_of_pixel(mid[0], mid[1])
        .expect("o ladrilho onde as 24 linhas passam");
    let list = bins.segs_of(ti);
    assert!(
        list.len() > OLD_CEILING,
        "a lista parou em {} — o teto voltou",
        list.len()
    );
    for sid in 0..24u32 {
        assert!(
            list.iter().any(|s| s.stroke == sid),
            "o traço {sid} nao chegou na lista do tile"
        );
    }
}

/// A ordem da lista **é** o contrato: o percurso agrupa por traço com um scan de run e compõe em
/// z pela mesma passada. Se ela embaralhar, um traço vira dois depósitos e a ordem de z se perde.
#[test]
fn the_list_of_a_tile_is_ordered_by_z_then_by_path() {
    let (w, h) = (64.0, 64.0);
    let sc = screen(w, h);
    let g = art(&[
        (
            &[[8.0, 30.0], [24.0, 30.0], [40.0, 30.0], [56.0, 30.0]],
            6.0,
            false,
            BLACK,
        ),
        (&[[8.0, 34.0], [56.0, 34.0]], 6.0, false, BLACK),
        (
            &[[30.0, 8.0], [30.0, 30.0], [30.0, 56.0]],
            6.0,
            false,
            BLACK,
        ),
    ]);
    let bins = bin_segments(&g, &sc, 16);
    let mut seen_any = false;
    for ti in 0..bins.ranges.len() {
        let list = bins.segs_of(ti);
        if list.len() < 2 {
            continue;
        }
        seen_any = true;
        for pair in list.windows(2) {
            let (x, y) = (pair[0], pair[1]);
            assert!(
                (x.stroke, x.a) < (y.stroke, y.a),
                "lista fora de ordem no tile {ti}: {x:?} antes de {y:?}"
            );
        }
    }
    assert!(seen_any, "nenhum tile com 2+ segmentos — fixture inútil");
}

/// A costura de um traço FECHADO é um segmento como outro qualquer — e é o único cujo `b` não é
/// `a + 1`, então é onde uma enumeração ingênua o perde.
#[test]
fn a_closed_stroke_gets_its_seam_binned() {
    let (w, h) = (64.0, 64.0);
    let sc = screen(w, h);
    let g = art(&[(
        &[[16.0, 16.0], [48.0, 16.0], [48.0, 48.0], [16.0, 48.0]],
        4.0,
        true,
        BLACK,
    )]);
    let bins = bin_segments(&g, &sc, 16);
    let seam = BinSeg {
        stroke: 0,
        a: 3,
        b: 0,
    };
    // A costura vai de (16,48) a (16,16): a aresta ESQUERDA.
    let ti = bins
        .tile_of_pixel(16.0, 32.0)
        .expect("tile da aresta esquerda");
    assert!(
        bins.segs_of(ti).contains(&seam),
        "a costura sumiu: {:?}",
        bins.segs_of(ti)
    );
    // E o pixel em cima dela é pintado pelas duas rotas.
    let p = [16.0, 32.0];
    assert!(
        walk_pixel(&bins, &g, &sc, p)[3] > 0.0,
        "a aresta de costura nao pinta"
    );
}

/// Entre traços a ordem é o `sid`, e o maior fica por cima — o que o depth GREATER fazia, agora
/// dito no percurso.
#[test]
fn the_walk_composes_the_later_stroke_on_top() {
    let (w, h) = (64.0, 64.0);
    let sc = screen(w, h);
    let g = art(&[
        (
            &[[8.0, 32.0], [56.0, 32.0]],
            10.0,
            false,
            [0.0, 0.0, 1.0, 1.0],
        ),
        (
            &[[8.0, 32.0], [56.0, 32.0]],
            10.0,
            false,
            [1.0, 0.0, 0.0, 1.0],
        ),
    ]);
    let bins = bin_segments(&g, &sc, 16);
    let out = walk_pixel(&bins, &g, &sc, [32.0, 32.0]);
    assert_eq!(out, [1.0, 0.0, 0.0, 1.0], "o traço de cima nao venceu");
}

/// A razão que justifica o módulo existir: o alcance por ladrilho é MUITO mais apertado que a
/// bbox — o número que matou o `set_scissor_rect` no doc 12 §6.1.
#[test]
fn the_bin_is_far_tighter_than_the_bounding_box() {
    let (w, h) = (512.0, 512.0);
    let sc = screen(w, h);
    let g = art(&[(&[[8.0, 8.0], [504.0, 504.0]], 12.0, false, BLACK)]);
    let bins = bin_segments(&g, &sc, 16);
    let touched = bins.ranges.iter().filter(|r| r[1] > 0).count();
    let tile_area = (bins.tile * bins.tile) as f64 * touched as f64;
    let bbox_area = (w * h) as f64;
    let ratio = bbox_area / tile_area;
    assert!(
        ratio >= 8.0,
        "o binning nao apertou nada: bbox/tiles = {ratio:.1}x ({touched} tiles)"
    );
}

/// ⚠️ **PIN EXATO** — o piso de raio do binner tem de ser o do shader. Se eles divergirem, o
/// binner solta segmentos que o consumidor quer e a tinta some **sem erro nenhum**.
#[test]
fn the_min_width_matches_the_shader() {
    let src = include_str!("shaders/flip.wgsl");
    let needle = "const MIN_WIDTH_PX: f32 = ";
    let at = src.find(needle).expect("o shader perdeu o MIN_WIDTH_PX");
    let tail = &src[at + needle.len()..];
    let end = tail.find(';').expect("declaração sem ponto-e-vírgula");
    let shader_value: f32 = tail[..end].trim().parse().expect("valor nao-numérico");
    assert_eq!(
        shader_value, MIN_WIDTH_PX,
        "o piso do binner ({MIN_WIDTH_PX}) divergiu do shader ({shader_value})"
    );
}

/// A distância segmento↔caixa precisa do teste de INTERSEÇÃO: um segmento que atravessa a caixa
/// de lado a lado não tem ponta dentro nem quina perto.
#[test]
fn a_segment_crossing_a_box_is_at_distance_zero() {
    let lo = [0.0, 0.0];
    let hi = [10.0, 10.0];
    assert_eq!(seg_box_distance([-5.0, 5.0], [15.0, 5.0], lo, hi), 0.0);
    assert_eq!(seg_box_distance([5.0, 5.0], [6.0, 6.0], lo, hi), 0.0);
    // E fora ela é exata: o segmento vertical em x = 14 dista 4 da parede direita.
    let d = seg_box_distance([14.0, -3.0], [14.0, 13.0], lo, hi);
    assert!((d - 4.0).abs() < 1e-5, "distancia errada: {d}");
}

/// O piso do raio **é** o do shader — perguntado direto, porque a diferença entre 0,65 e 0,1 px
/// é menor que qualquer fronteira de ladrilho e nenhuma fixture de cobertura a enxerga.
#[test]
fn the_radius_floor_is_the_shader_floor() {
    let sc = screen(64.0, 64.0);
    assert_eq!(sc.radius_px(0.02), MIN_WIDTH_PX * 0.5, "o piso sumiu");
    assert_eq!(
        sc.radius_px(0.0),
        MIN_WIDTH_PX * 0.5,
        "largura zero tem de subir ao piso"
    );
    // E acima do piso ele é inerte.
    assert_eq!(sc.radius_px(8.0), 4.0);
}

/// ⚠️ **ARCH-GATE.** O binner tem de perguntar o raio à MESMA porta que o percurso
/// (`ScreenSpace::radius_px`). Um sítio que recomputa `width * 0.5 * px_per_world` na mão perde o
/// piso e solta segmentos que o percurso quer — e a janela onde isso é observável por pixel tem
/// **0,14 px**, ou seja: nenhum gate de comportamento o pega.
#[test]
fn the_binner_asks_the_screen_for_the_radius() {
    let src = include_str!("binning.rs");
    let at = src
        .find("pub fn bin_segments(")
        .expect("o binner mudou de nome — reaponte este gate");
    let body = &src[at..];
    let end = body.find("\n// ").unwrap_or(body.len());
    let body = &body[..end];
    assert!(
        body.contains("screen.radius_px("),
        "o binner parou de perguntar a porta única"
    );
    assert!(
        !body.contains("px_per_world"),
        "o binner recomputa o raio na mão em vez de perguntar — o piso se perde ali"
    );
}

// ═══════════════════════════════ passo 3 — A LEI ═══════════════════════════════

/// ⚠️ **O CONTROLE DE TODOS OS SMOKES** (§8 do handoff): em `hardness = 1` o traço não pode mudar.
///
/// A integral tem de reproduzir a união dura — e a medição diz **onde** ela não reproduz: só a
/// borda, onde a união é um degrau e a integral é uma rampa de largura sub-pixel.
#[test]
fn at_hardness_one_the_integral_is_the_hard_union() {
    let (w, h) = (96.0, 64.0);
    let sc = screen(w, h);
    let g = art(&[
        (&[[10.0, 20.0], [86.0, 44.0]], 11.0, false, BLACK),
        (
            &[[20.0, 50.0], [50.0, 12.0], [78.0, 50.0]],
            7.0,
            false,
            BLACK,
        ),
    ]);
    let bins = bin_segments(&g, &sc, 16);
    let (mut differ, mut worst_band) = (0usize, 0.0f32);
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
                continue;
            };
            let list = bins.segs_of(ti);
            let mut i = 0;
            while i < list.len() {
                let sid = list[i].stroke;
                let mut j = i;
                while j < list.len() && list[j].stroke == sid {
                    j += 1;
                }
                let run = &list[i..j];
                let tau_cover = stroke_deposit(run, &g, &sc, p).map_or(0.0, |d| d.cover);
                let hard = hard_union_deposit(run, &g, &sc, p).map_or(0.0, |d| d.cover);
                if (tau_cover - hard).abs() > 1.0 / 255.0 {
                    differ += 1;
                    // Quão longe da silhueta esse pixel está? (a distância ao contorno)
                    let band = edge_distance(run, &g, &sc, p);
                    worst_band = worst_band.max(band.abs());
                }
                i = j;
            }
        }
    }
    // Medido: a discordância vive numa casca de menos de meio pixel em torno da silhueta.
    assert!(
        worst_band < 0.75,
        "a integral discorda a {worst_band:.3} px da borda em {differ} pixels — nao e' so a borda"
    );
}

/// Distância com sinal do pixel à silhueta do traço (negativa = dentro).
fn edge_distance(run: &[BinSeg], g: &FlipGpuData, sc: &ScreenSpace, p: [f32; 2]) -> f32 {
    let mut best = f32::MAX;
    for seg in run {
        let (pa, pb) = (g.points[seg.a as usize], g.points[seg.b as usize]);
        let sa = sc.point_px(pa.pos);
        let sb = sc.point_px(pb.pos);
        let (t, cx, cy) = closest_on_seg(p, sa, sb);
        let dist = ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt();
        let r = sc.radius_px(pa.width) * (1.0 - t) + sc.radius_px(pb.width) * t;
        best = best.min(dist - r);
    }
    best
}

/// A curva de UM dab **é** a do Painter — conferida contra a função REAL dele, não contra uma
/// reescrita. É esta âncora que faz o motor novo mirar no depósito digital que o Enio pediu.
#[test]
fn the_dab_weight_is_the_painters_falloff() {
    for hi in 0..20 {
        let hardness = hi as f32 / 20.0;
        for di in 0..=100 {
            let dn = di as f32 / 100.0;
            let ours = dab_weight(dn, hardness);
            let theirs = {
                let h = hardness.clamp(0.0, 1.0);
                if h >= 1.0 {
                    f32::from(dn < 1.0)
                } else {
                    let remapped = ((dn - h) / (1.0 - h)).clamp(0.0, 1.0);
                    ph2d_painter_brush::Falloff::Smooth.weight(remapped)
                }
            };
            assert_eq!(ours, theirs, "divergiu em dn={dn}, hardness={hardness}");
        }
    }
}

/// **A IDENTIDADE QUE TROCA O PRODUTO POR UMA SOMA** — a peça inteira do motor novo.
/// `1 − Π(1−w_k)` e `1 − exp(−Σ f(d_k))` são o MESMO número; é a segunda forma que é comutativa,
/// sem ordem e sem teto.
#[test]
fn the_sum_of_f_is_the_product_of_the_dabs() {
    for hi in 1..20 {
        let hardness = hi as f32 / 20.0;
        let dns = [0.05f32, 0.2, 0.35, 0.5, 0.65, 0.8, 0.95];
        let product: f32 = dns.iter().map(|d| 1.0 - dab_weight(*d, hardness)).product();
        let sum: f32 = dns.iter().map(|d| f_of(*d, hardness)).sum();
        let (a, b) = (1.0 - product, 1.0 - (-sum).exp());
        assert!(
            (a - b).abs() < 2e-6,
            "hardness {hardness}: produto {a} != exp(-soma) {b}"
        );
    }
}

/// **O DEFEITO QUE CUSTOU A SAGA.** Onde o traço cruza a si mesmo há MAIS caminho perto do pixel,
/// então `τ` é estritamente maior — a lei responde ao cruzamento por construção, sem canal
/// lateral, sem teto e sem depth. O motor de hoje integra uma reta fictícia, que não tem
/// cruzamento nenhum para ver.
#[test]
fn the_crossing_carries_more_tau_than_a_single_arm() {
    let (w, h) = (96.0, 96.0);
    let sc = screen(w, h);
    // Um X: as duas pernas se cruzam no centro.
    let g = art(&[(
        &[[16.0, 16.0], [80.0, 80.0], [48.0, 80.0], [48.0, 16.0]],
        9.0,
        false,
        BLACK,
    )]);
    let bins = bin_segments(&g, &sc, 16);
    let tau_at = |p: [f32; 2]| {
        let ti = bins.tile_of_pixel(p[0], p[1]).unwrap();
        let list = bins.segs_of(ti);
        crate::tau::stroke_tau(list, &g, &sc, 0.4, p).map_or(0.0, |(t, _)| t)
    };
    let crossing = tau_at([48.0, 48.0]);
    let single_arm = tau_at([48.0, 24.0]);
    assert!(
        crossing > single_arm * 1.2,
        "o cruzamento nao acumulou: {crossing:.3} contra {single_arm:.3} de um braço só"
    );
}

/// **A LEI É FATO DO CAMINHO, NÃO DA DENSIDADE DA POLILINHA** — a doença que esta linha curou
/// quatro vezes, agora pinada no PRODUTO e não só na sonda: o MESMO caminho amostrado em 4 e em
/// 40 pontos tem de pintar a mesma imagem.
#[test]
fn the_ink_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled() {
    let (w, h) = (96.0, 64.0);
    let sc = screen(w, h);
    // ⚠️ A MESMA GEOMETRIA, amostrada de dois jeitos — a versão fina insere pontos **sobre** as
    // pernas da grossa. Uma senoide reamostrada mediria a CORDA, não a lei: com 4 pontos ela é
    // outro desenho, e o gate estaria medindo geometria diferente e chamando de dependência de
    // amostragem (a fixture que eu escrevi primeiro, e que falhou por isso).
    let coarse: Vec<[f32; 2]> = vec![[12.0, 20.0], [50.0, 46.0], [84.0, 18.0]];
    let mut fine: Vec<[f32; 2]> = Vec::new();
    for leg in coarse.windows(2) {
        for i in 0..12 {
            let t = i as f32 / 12.0;
            fine.push([
                leg[0][0] + (leg[1][0] - leg[0][0]) * t,
                leg[0][1] + (leg[1][1] - leg[0][1]) * t,
            ]);
        }
    }
    fine.push(*coarse.last().unwrap());
    assert_eq!(
        (coarse.len(), fine.len()),
        (3, 25),
        "a fixture mudou de forma"
    );

    // ⚠️ **Dureza MACIA, e a escolha é medida.** Em `hardness = 1` a cobertura é um DEGRAU: a
    // borda é resolvida até um passo de quadratura (~0,06 px), e um pixel cujo centro cai nessa
    // casca **flipa 255 de uma vez**. Um gate de densidade ali mede o degrau, não a lei — medido:
    // pior desvio 254,8/255 com `SUB = 2` e 1,06 com `SUB = 4`, contra os números abaixo em 0,4.
    // (A metade dura tem gate PRÓPRIO: `at_hardness_one_the_integral_is_the_hard_union`.)
    let mut gc = art(&[(&coarse, 9.0, false, BLACK)]);
    let mut gf = art(&[(&fine, 9.0, false, BLACK)]);
    gc.strokes[0].hardness = 0.4;
    gf.strokes[0].hardness = 0.4;
    let bc = bin_segments(&gc, &sc, 16);
    let bf = bin_segments(&gf, &sc, 16);
    let mut worst = 0.0f32;
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            let p = [x as f32 + 0.5, y as f32 + 0.5];
            let a = walk_pixel(&bc, &gc, &sc, p)[3];
            let b = walk_pixel(&bf, &gf, &sc, p)[3];
            worst = worst.max((a - b).abs());
        }
    }
    // O resíduo que sobra é a GRADE de quadratura (cada segmento arredonda o próprio `n`), não a
    // lei. Medido: sub-nível de byte.
    assert!(
        worst * 255.0 < 1.0,
        "a densidade mexeu na tinta: pior desvio {:.2}/255",
        worst * 255.0
    );
}

/// A cobertura segue o raio **LOCAL**, não uma média do segmento — a pressão é o caso normal, e
/// com raio médio um traço que afina sairia com espessura constante.
#[test]
fn the_coverage_follows_the_local_radius_of_a_tapering_stroke() {
    let (w, h) = (128.0, 64.0);
    let sc = screen(w, h);
    let mut g = FlipGpuData::default();
    push_tapered(&mut g, &[[10.0, 32.0], [118.0, 32.0]], &[24.0, 6.0]);
    let bins = bin_segments(&g, &sc, 16);
    let half_at = |x: f32| -> f32 {
        let mut best = 0.0f32;
        for k in 0..400 {
            let dy = k as f32 * 0.1;
            if walk_pixel(&bins, &g, &sc, [x, 32.0 + dy])[3] > 0.5 {
                best = dy;
            }
        }
        best
    };
    let (thick, thin) = (half_at(20.0), half_at(108.0));
    let ratio = thick / thin.max(1e-3);
    // As larguras autoradas nesses x são ~22,2 e ~7,8 (o lerp), razão ~2,85.
    assert!(
        (2.4..3.4).contains(&ratio),
        "o traço nao afinou: meia-largura {thick:.1} contra {thin:.1} (razao {ratio:.2})"
    );
}

/// ⚠️ **A REGRA DO GP que o `flip.wgsl` documenta:** *um traço a opacity 0,5 não escurece sobre si
/// mesmo*. É por isso que o `opacity` multiplica DEPOIS da cobertura e **nunca entra no `f`** — se
/// entrasse, o cruzamento acumularia opacidade e a regra cairia.
#[test]
fn opacity_scales_the_ink_and_never_darkens_the_crossing() {
    let (w, h) = (96.0, 96.0);
    let sc = screen(w, h);
    let mut g = art(&[(
        &[[16.0, 16.0], [80.0, 80.0], [48.0, 80.0], [48.0, 16.0]],
        11.0,
        false,
        BLACK,
    )]);
    for p in &mut g.points {
        p.opacity = 0.5;
    }
    let bins = bin_segments(&g, &sc, 16);
    let arm = walk_pixel(&bins, &g, &sc, [48.0, 24.0])[3];
    let crossing = walk_pixel(&bins, &g, &sc, [48.0, 48.0])[3];
    assert!(
        (arm - 0.5).abs() < 1.0 / 255.0,
        "opacity 0,5 nao virou meia tinta: {arm:.4}"
    );
    assert!(
        (crossing - arm).abs() < 1.0 / 255.0,
        "o cruzamento ESCURECEU: {crossing:.4} contra {arm:.4} do braço"
    );
}
