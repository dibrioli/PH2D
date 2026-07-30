//! Gates do binning por tile (`binning.rs`).
//!
//! O que precisa ser provado aqui **não é a lei da tinta** (ela chega no passo 3): é que a
//! estrutura de aceleração **não muda a resposta** e **não tem teto** — as duas propriedades que
//! o motor de hoje não tem.

use super::*;
use crate::pack::{FLAG_CLOSED, GpuPoint, GpuStroke};

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
//
// ⚠️ **Ela mora no arquivo AO LADO** (`tau_tests.rs`), e o corte é o que o doc-comment no topo
// deste arquivo já declarava: aqui prova-se que o BINNING não muda a resposta; ali, que a resposta
// é a certa. O irmão fica pendurado AQUI (e não em `tau.rs`) porque as fixtures — `screen`, `art`,
// `push_tapered` — são as do binning, e uma segunda cópia delas seria uma segunda cena.
#[path = "tau_tests.rs"]
mod tau_tests;

/// Os DOIS FATORES FORA DO `τ` — o fade sub-pixel e a tampa chata. Irmão do `tau_tests`, e pendurado
/// aqui pelo mesmo motivo: as fixtures são as do binning.
#[path = "cover_tests.rs"]
mod cover_tests;

/// A medição do RISCO da antiderivada (§21.5) — irmão dos dois acima, e separado porque é uma
/// pergunta de PROJETO (*vale trocar a quadratura por duas leituras de tabela?*), não um gate da lei.
#[path = "antiderivative_tests.rs"]
mod antiderivative_tests;
