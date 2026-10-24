//! ⭐ **O Wet do Digital é a orla da aguada** (bug #58, 7.ª volta — o dono 24/10: *«veja como o modo watercolor cria
//! as bordas e veja se pode aplicar aqui no digital»*). O oráculo é o PRÓPRIO meio Watercolor: o mesmo recto, o mesmo
//! raio, a mesma ponta (`Falloff::Watercolor`), o Spread = o Edge width. A régua é a DENSIDADE em função da distância
//! à silhueta (a meia-altura da densidade do miolo, vinda de fora), normalizada pela do miolo: na aguada a densidade
//! óptica do canal verde sobre o papel branco (Beer–Lambert, a cor absorve o verde), no Digital `−ln(1 − alfa)` na
//! tela transparente (a lei do Wet é Beer–Lambert sobre o alfa).

use super::traco_orla_lei_tests::{caminho, cp, pinta};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

const W: usize = 320;
const H: usize = 200;
/// O raio do pincel das medições (px).
const R: f32 = 20.0;
/// O eixo do recto.
const Y: f32 = 100.0;
/// A cor das medições (sRGB 8): o verde a meio (`150`, linear `0,305`) — a régua lê-o sem saturar no aro.
const COR: [u8; 3] = [200, 150, 60];
/// O Spread da aguada de fábrica = o Edge width do Digital nas medições (px).
const SPREAD: f32 = 4.0;

/// sRGB 8 → linear.
fn lin(c: u8) -> f32 {
    let c = f32::from(c) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// A régua: a densidade de um texel.
type Densidade = fn(&[u8], usize, usize) -> f32;

/// A densidade da AGUADA sobre o papel branco (Body 0): `−ln T` do verde, com `verde = pig + (1 − pig)·T` em luz
/// linear (`aparencia`, Beer–Lambert com o espalhamento).
fn d_aguada(px: &[u8], x: usize, y: usize) -> f32 {
    let pig = lin(COR[1]);
    -((lin(px[(y * W + x) * 4 + 1]) - pig) / (1.0 - pig))
        .max(1e-4)
        .ln()
}

/// A densidade do DIGITAL sobre o papel branco: `−ln(1 − a)` do verde, com `verde = cor + (255 − cor)(1 − a)` (o
/// Normal compõe nos tons do ecrã).
fn d_digital(px: &[u8], x: usize, y: usize) -> f32 {
    let c = f32::from(COR[1]);
    -((f32::from(px[(y * W + x) * 4 + 1]) - c) / (255.0 - c))
        .max(1e-4)
        .ln()
}

/// O Watercolor de fábrica (o slot do meio), 40 px, `COR`, sobre o papel branco, com `f` por cima.
fn aguada(f: impl FnOnce(&mut PainterTool)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(crate::PaintMedia::Watercolor);
    t.set_brush_size_px(2.0 * R);
    t.set_brush_color_srgb8(COR);
    f(&mut t);
    t
}

/// A aguada só com a lei do aro: sem Wet (o miolo afina), sem diluição, sem orla rasgada, sem Body, sem grão, a carga
/// cheia — e o Edge Darkening `ganho`.
fn aguada_pura(ganho: f32) -> PainterTool {
    aguada(|t| {
        let b = &mut t.paint.brush;
        (b.wet_rewet, b.wet_dilution, b.wet_smudge, b.wet_pull) = (0.0, 0.0, 0.0, 0.0);
        (b.wet_charge, b.warp, b.opacity, b.granulation) = (1.0, 0.0, 0.0, 0.0);
        t.set_brush_edge_gain(ganho);
    })
}

/// O Digital por omissão com a ponta da aguada (`Falloff::Watercolor`), 40 px, `COR`, Strength 0,6, Edge width =
/// [`SPREAD`], Wet `m`, na tela `fundo`.
fn digital(m: f32, fundo: [u8; 4]) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(fundo.repeat(W * H), W as u32, H as u32);
    t.set_paint_media(crate::PaintMedia::Digital);
    t.set_brush_size_px(2.0 * R);
    t.set_brush_color_srgb8(COR);
    t.set_brush_strength(0.6);
    t.paint.brush.falloff = ph2d_painter_brush::Falloff::Watercolor;
    t.set_traco_tecto(1.0);
    t.set_traco_largura(SPREAD);
    t.set_traco_molhada(m);
    t
}

/// Pinta `pts` e levanta: a tela.
fn tela(mut t: PainterTool, pts: &[[f32; 2]]) -> Vec<u8> {
    pinta(&mut t, pts, |_| 4);
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
    t.paint_tick(1.0 / 60.0);
    t.canvas_rgba.to_vec()
}

/// O recto horizontal das medições.
fn recto() -> Vec<[f32; 2]> {
    caminho(&[[60.0, Y], [260.0, Y]])
}

/// O Z que se cruza e um borrão grande (as fotos e o gate do miolo).
fn z_e_borrao() -> Vec<Vec<[f32; 2]>> {
    let z = caminho(&[
        [40.0, 40.0],
        [170.0, 40.0],
        [45.0, 160.0],
        [170.0, 160.0],
        [110.0, 25.0],
    ]);
    let borrao: Vec<[f32; 2]> = (0..7)
        .flat_map(|k| {
            let y = 60.0 + 14.0 * k as f32;
            let (a, b) = if k % 2 == 0 {
                (235.0, 290.0)
            } else {
                (290.0, 235.0)
            };
            caminho(&[[a, y], [b, y]])
        })
        .collect();
    vec![z, borrao]
}

/// Pinta os traços de `z_e_borrao` (um por um) e devolve a tela.
fn tela_z(mk: &dyn Fn() -> PainterTool) -> Vec<u8> {
    let mut t = mk();
    for pts in z_e_borrao() {
        pinta(&mut t, &pts, |_| 4);
        t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
        t.paint_tick(1.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

/// O perfil de um recto pela régua `d`: `(miolo, perfil)` — a densidade do miolo e `D(s)/miolo` a `s = 0, ½, 1, …`
/// px para DENTRO da silhueta, a média das colunas do meio dos dois lados.
fn perfil(px: &[u8], d: Densidade) -> (f32, Vec<f32>) {
    let cols: Vec<usize> = (120..200).step_by(4).collect();
    let miolo = cols.iter().map(|&x| d(px, x, Y as usize)).sum::<f32>() / cols.len() as f32;
    let mut p = vec![0.0f32; 2 * R as usize];
    let mut n = 0.0;
    for &x in &cols {
        for lado in [-1.0f32, 1.0] {
            let em = |s: f32| {
                let yf = Y + lado * s;
                let (y0, f) = (yf.floor() as usize, yf - yf.floor());
                d(px, x, y0) * (1.0 - f) + d(px, x, y0 + 1) * f
            };
            let borda = (0..1000)
                .map(|k| R + 20.0 - k as f32 * 0.05)
                .find(|&s| em(s) >= 0.5 * miolo)
                .expect("a silhueta");
            for (k, q) in p.iter_mut().enumerate() {
                *q += em(borda - 0.5 * k as f32) / miolo;
            }
            n += 1.0;
        }
    }
    p.iter_mut().for_each(|q| *q /= n);
    (miolo, p)
}

/// `(pico/miolo, onde o pico (px), largura (px) até o excesso cair a 10 % do pico)`.
fn resumo(p: &[f32]) -> (f32, f32, f32) {
    let (k, &pico) = p
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .unwrap();
    let fim = (k..p.len())
        .find(|&j| p[j] - 1.0 <= 0.1 * (pico - 1.0))
        .unwrap_or(p.len());
    (pico, 0.5 * k as f32, 0.5 * fim as f32)
}

/// O pigmento que o aro junta: `∫ (D(s)/miolo − 1) ds` sobre o excesso (px).
fn junta(p: &[f32]) -> f32 {
    p.iter().map(|&q| (q - 1.0).max(0.0) * 0.5).sum()
}

/// O maior desvio entre os dois perfis normalizados nos primeiros `ate` px.
fn desvio(a: &[f32], b: &[f32], ate: f32) -> f32 {
    (0..(2.0 * ate) as usize)
        .map(|k| (a[k] - b[k]).abs())
        .fold(0.0, f32::max)
}

/// O Digital com o MESMO miolo que a aguada (a densidade `miolo`): Strength `1 − e^(−miolo)` (o `1 − a` do Digital faz
/// o papel do `T` da aguada).
fn digital_com_o_miolo(m: f32, miolo: f32) -> PainterTool {
    let mut t = digital(m, [255; 4]);
    t.set_brush_strength(1.0 - (-miolo).exp());
    t
}

/// O maior desvio entre os perfis do pico de `a` para DENTRO (o lado que o borrão do aro desenha), com `b` deslocado
/// de `passos` meios-píxeis (alinhados pelo pico).
fn desvio_alinhado(a: &[f32], b: &[f32], passos: isize, de: f32, ate: f32) -> f32 {
    ((2.0 * de) as isize..(2.0 * ate) as isize)
        .filter_map(|k| {
            let j = k - passos;
            (j >= 0 && (j as usize) < b.len()).then(|| (a[k as usize] - b[j as usize]).abs())
        })
        .fold(0.0, f32::max)
}

/// ⭐ **O gate: o wet do digital é a orla da aguada** — no recto sobre o papel branco, com o mesmo miolo, o perfil
/// normalizado `D(s)/miolo` do Digital com Wet `m` contra o da aguada com Edge Darkening `8·m` (Spread 4 = Edge width
/// 4, a mesma ponta): o pico a ±20 %, o pigmento junto (`∫ excesso`) a ±15 %, a descida do pico ao fim do aro a ±1
/// px, a curva do pico para dentro (alinhada pelo pico) a ±25 % do pico, o pico a ±3 px da silhueta do da aguada, o miolo
/// igual (±3 %). O controlo: o Wet 0 não tem aro (pico ≤ 1,05) e desvia > 2.
///
/// ⚠️ A SUBIDA difere (medido): a aguada desenha a silhueta ~1,6 px POR FORA do contorno `cw = ½` dos campos dela (a
/// amostra de píxel e o antialias à parte, `sonda_os_campos_do_aro`), o Digital no contorno `τ` dele — o pico do
/// Digital fica 2–2,5 px mais perto da borda; a descida (o borrão do aro, a mesma conta) é a mesma.
#[test]
fn o_wet_do_digital_e_a_orla_da_aguada() {
    for m in [0.5f32, 1.0] {
        let (miolo_a, ag) = perfil(&tela(aguada_pura(8.0 * m), &recto()), d_aguada);
        let (miolo_d, dg) = perfil(&tela(digital_com_o_miolo(m, miolo_a), &recto()), d_digital);
        let (_, seco) = perfil(
            &tela(digital_com_o_miolo(0.0, miolo_a), &recto()),
            d_digital,
        );
        let ((pa, oa, la), (pd, od, ld)) = (resumo(&ag), resumo(&dg));
        let (ja, jd) = (junta(&ag), junta(&dg));
        let passos = (2.0 * (oa - od)).round() as isize;
        let e = desvio_alinhado(&ag, &dg, passos, oa, 12.0);
        let e0 = desvio(&ag, &seco, 12.0);
        eprintln!(
            "Wet {m}: aguada miolo {miolo_a:.3} pico {pa:.2} a {oa} px, fim {la} px, junta {ja:.2} · digital miolo {miolo_d:.3} pico {pd:.2} a {od} px, fim {ld} px, junta {jd:.2} · curva alinhada {e:.3} (seco {e0:.3}, pico {:.2})",
            resumo(&seco).0
        );
        assert!(
            e0 > 2.0 && resumo(&seco).0 <= 1.05,
            "o controlo: o seco já tem aro ({e0})"
        );
        assert!(
            (miolo_a - miolo_d).abs() <= 0.03 * miolo_a,
            "o miolo: {miolo_d} × {miolo_a}"
        );
        assert!((pd / pa - 1.0).abs() <= 0.2, "Wet {m}: pico {pd} × {pa}");
        assert!((jd / ja - 1.0).abs() <= 0.15, "Wet {m}: junta {jd} × {ja}");
        assert!(
            ((la - oa) - (ld - od)).abs() <= 1.0,
            "Wet {m}: a descida {} × {} px",
            ld - od,
            la - oa
        );
        assert!(e <= 0.25 * pa, "Wet {m}: a curva alinhada desvia {e}");
        assert!((oa - od).abs() <= 3.0, "Wet {m}: o pico em {od} × {oa} px");
    }
}

/// ⭐ **O miolo do Wet é o do seco** (o pedido nº 1 do dono: *«centro transparente»*; na aguada o aro não mexe no
/// corpo — medido: Edge Darkening 0 e 4 dão o mesmo miolo, `D = 1,530`): no Z e no borrão, todo texel do Wet 1 é
/// pelo menos tão denso como o do seco, e no miolo da lei (a silhueta endurecida cheia a `Edge width + 1` px à volta)
/// é o MESMO, ao byte.
#[test]
fn o_miolo_do_wet_e_o_do_seco() {
    let seco = tela_z(&|| digital(0.0, [0; 4]));
    let wet = tela_z(&|| digital(1.0, [0; 4]));
    let a = |px: &[u8], i: usize| px[i * 4 + 3];
    // O miolo da LEI: `cw = 1` numa caixa do raio do borrão + 1 à volta (`v ≥ 0,84·L`, com `cw` a saturar em
    // `COV_HALF·v/τ = SS1`, `v = 0,833·L`) — aí `inner = 1` e o aro é zero, na aguada como no Digital.
    let nivel = f32::from((0..W * H).map(|i| a(&seco, i)).max().unwrap());
    let alcance = SPREAD as i64 + 1;
    let cheio = |x: i64, y: i64| {
        x >= 0
            && y >= 0
            && (x as usize) < W
            && (y as usize) < H
            && f32::from(a(&seco, y as usize * W + x as usize)) >= 0.84 * nivel
    };
    let fundo: Vec<usize> = (0..W * H)
        .filter(|&i| {
            let (x, y) = ((i % W) as i64, (i / W) as i64);
            (-alcance..=alcance).all(|dy| (-alcance..=alcance).all(|dx| cheio(x + dx, y + dy)))
        })
        .collect();
    assert!(fundo.len() > 3_000, "o Z não tem miolo ({})", fundo.len());
    let mudou = fundo.iter().filter(|&&i| a(&seco, i) != a(&wet, i)).count();

    let claros = (0..W * H).filter(|&i| a(&wet, i) < a(&seco, i)).count();
    let aro = (0..W * H)
        .filter(|&i| a(&wet, i) > a(&seco, i).saturating_add(8))
        .count();
    eprintln!(
        "miolo {} texels, {mudou} mudaram · {claros} mais claros · {aro} no aro",
        fundo.len()
    );
    assert_eq!(mudou, 0, "o Wet mexeu em {mudou} texels do miolo");
    assert_eq!(claros, 0, "o Wet clareou {claros} texels");
    assert!(aro > 1_000, "o Z não tem aro ({aro})");
}

/// ⭐ **O aro não pontilha nem escada**: no recto horizontal de 16 px (Wet 1, papel branco), cada linha da borda é uniforme ao
/// longo do traço — a maior variação de uma linha (colunas do meio) fica a ≤ 3/255 da do seco. Sem o antialias do aro
/// (`× min(sd, 1)`) o contorno, que ondula entre os carimbos, acendia texels alternados.
#[test]
fn o_aro_nao_pontilha() {
    let variacao = |px: &[u8]| {
        (0..H)
            .map(|y| {
                let g: Vec<u8> = (120..200).map(|x| px[(y * W + x) * 4 + 1]).collect();
                g.iter().max().unwrap() - g.iter().min().unwrap()
            })
            .max()
            .unwrap()
    };
    // O pincel de 16 px das fotos (onde o pontilhado se viu): o contorno ondula entre os carimbos.
    let fino = |m: f32| {
        let mut t = digital(m, [255; 4]);
        t.set_brush_size_px(16.0);
        tela(t, &recto())
    };
    let (seco, wet) = (variacao(&fino(0.0)), variacao(&fino(1.0)));
    eprintln!("a maior variação de uma linha: seco {seco} · Wet 1 {wet} /255");
    assert!(wet <= seco + 3, "o aro pontilha: {wet} × {seco}");
}

/// ⛔ Sonda: a tabela da 7.ª volta — o perfil da aguada (fábrica, Edge Darkening 4, pura) e do Digital (Wet 0, ½, 1),
/// pela régua de cada um.
#[test]
#[ignore = "sonda"]
fn sonda_o_perfil_da_aguada_e_do_digital() {
    let casos: Vec<(&str, Vec<u8>, Densidade)> = vec![
        (
            "aguada de fábrica (Body 0,4: a régua lê-a só aproximada)",
            tela(aguada(|_| {}), &recto()),
            d_aguada,
        ),
        (
            "aguada de fábrica, Edge Darkening 4",
            tela(aguada(|t| t.set_brush_edge_gain(4.0)), &recto()),
            d_aguada,
        ),
        (
            "aguada pura, Edge Darkening 4",
            tela(aguada_pura(4.0), &recto()),
            d_aguada,
        ),
        (
            "aguada pura, Edge Darkening 8",
            tela(aguada_pura(8.0), &recto()),
            d_aguada,
        ),
        (
            "digital Wet 0",
            tela(digital(0.0, [255; 4]), &recto()),
            d_digital,
        ),
        (
            "digital Wet 0,5",
            tela(digital(0.5, [255; 4]), &recto()),
            d_digital,
        ),
        (
            "digital Wet 1",
            tela(digital(1.0, [255; 4]), &recto()),
            d_digital,
        ),
    ];
    for (nome, px, d) in &casos {
        let (miolo, p) = perfil(px, *d);
        let (pico, onde, largura) = resumo(&p);
        let q: Vec<String> = p.iter().step_by(2).map(|v| format!("{v:.2}")).collect();
        eprintln!(
            "{nome}: miolo D {miolo:.3} · pico/miolo {pico:.2} a {onde:.1} px · largura {largura:.1} px · D(s)/miolo s=0..: {}",
            q.join(" ")
        );
    }
}

/// ⛔ Fotos (`PH2D_FOTO_DIR`): sobre o MESMO papel branco, o Z que se cruza e um borrão, ×2 — Watercolor de fábrica com
/// Edge Darkening 4 e 8 · Digital Wet 0 · Digital Wet ½ e 1 (a lei nova). A coluna do Wet 1 da lei velha (o patamar do
/// Procreate) tira-se do binário velho (`git worktree` em `18332b9e9f`) com este mesmo teste.
#[test]
#[ignore = "sonda: escreve PNG em PH2D_FOTO_DIR"]
fn foto_das_bordas_lado_a_lado() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let dir = std::path::Path::new(&dir);
    let painel = |px: &[u8], nome: &str| {
        let z = 2;
        let mut c = super::tests::look_watercolor_arc::Canvas::new(W * z, H * z);
        for y in 0..H * z {
            for x in 0..W * z {
                let i = ((y / z) * W + x / z) * 4;
                c.set(x, y, [px[i], px[i + 1], px[i + 2]]);
            }
        }
        super::tests::look_watercolor_arc::write_png(&dir.join(nome), &c).expect("PNG");
    };
    // O pincel das fotos: 16 px (o Z lê-se), o resto o das medições.
    let fino = |mut t: PainterTool| {
        t.set_brush_size_px(16.0);
        t
    };
    let branco = [255u8; 4];
    painel(
        &tela_z(&|| fino(aguada(|t| t.set_brush_edge_gain(4.0)))),
        "1_watercolor_edge4.png",
    );
    painel(
        &tela_z(&|| fino(aguada(|t| t.set_brush_edge_gain(8.0)))),
        "1b_watercolor_edge8.png",
    );
    painel(
        &tela_z(&|| fino(digital(0.0, branco))),
        "2_digital_wet0.png",
    );
    painel(
        &tela_z(&|| fino(digital(0.5, branco))),
        "3_digital_wet05.png",
    );
    painel(
        &tela_z(&|| fino(digital(1.0, branco))),
        "4_digital_wet1.png",
    );
}

/// ⛔ Sonda: os CAMPOS da lei do aro no recto da aguada pura (`cov`, `cw`, `inner`) e o `v` do Digital, por linha.
#[test]
#[ignore = "sonda"]
fn sonda_os_campos_do_aro() {
    let mut t = aguada_pura(8.0);
    pinta(&mut t, &recto(), |_| 4);
    let cov: Vec<f32> = t
        .paint
        .stroke_coverage
        .iter()
        .map(|&c| f32::from(c) / 255.0)
        .collect();
    let cw: Vec<f32> = cov
        .iter()
        .map(|&c| super::watercolor_rim::endurece(c))
        .collect();
    let inner = super::watercolor_field::box_blur(&cw, W, H, 4);
    let vivo = t.canvas_rgba.to_vec();
    t.on_canvas_pointer(cp(*recto().last().unwrap(), PointerPhase::Up));
    t.paint_tick(1.0 / 60.0);
    let fim = t.canvas_rgba.to_vec();
    let mut d = digital(0.0, [0; 4]);
    pinta(&mut d, &recto(), |_| 4);
    let s = d.paint.plano_do_traco.as_ref().unwrap();
    let x = 160usize;
    for y in (Y as usize + 10)..(Y as usize + 40) {
        let i = y * W + x;
        eprintln!(
            "y+{:2}: cov {:.3} cw {:.3} inner {:.3} cw−inner {:+.3} | D vivo {:.3} fim {:.3} previsto {:.3} | digital v {:.3}",
            y - Y as usize,
            cov[i],
            cw[i],
            inner[i],
            cw[i] - inner[i],
            d_aguada(&vivo, x, y),
            d_aguada(&fim, x, y),
            (cw[i] * 0.25 + super::watercolor_rim::densidade_do_aro(8.0, cw[i], inner[i])).max(0.0)
                * 2.0,
            f32::from(s.plano[i * 4 + 3]) / 255.0
        );
    }
}
