//! Gates + sonda do relevo do papel **no Digital** (a tinta opaca mostra o dente; a marca da tinta
//! do Wet desfaz-se com o traço) — irmão de `substrate_relief_tests.rs`, partido pelo tecto de LOC.
use super::*;
use crate::Region;
use ph2d_editor_core::tool::RasterEditTool;

const N: u32 = 96;

/// A tela final (composite + luz) de um traço Digital OPACO de cor sólida no meio da tela, com o papel a `relief`.
fn digital_opaco(relief: f32) -> Vec<u8> {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    // O documento do produto: a tela branca nasce PAPEL com a camada de tinta vazia por cima (`abre_a_sprite`).
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_substrate_depth(relief);
    // Tooth 0: a cobertura não entra no dente — a régua lê só a LUZ do relevo sobre a tinta.
    t.set_brush_paper_depth(0.0);
    t.set_brush_size_px(18.0);
    t.set_brush_color_srgb8([200, 40, 40]);
    let cp = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp([4.0, 48.0], PointerPhase::Down));
    for k in 1..=46 {
        t.on_canvas_pointer(cp([4.0 + 2.0 * k as f32, 48.0], PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp([96.0, 48.0], PointerPhase::Up));
    t.on_tick(8.0);
    let mut px = t.run_full().0;
    t.apply_impasto_light(
        &mut px,
        Region {
            x: 0,
            y: 0,
            w: N,
            h: N,
        },
    );
    px
}

/// O desvio da luminância que o Relief acrescenta (`px − liso`) nas linhas `ys` (colunas 10..86).
fn dente_nas_linhas(px: &[u8], liso: &[u8], ys: std::ops::Range<usize>) -> f64 {
    let l =
        |p: &[u8]| 0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2]);
    let d: Vec<f64> = ys
        .flat_map(|y| (10..86).map(move |x| (y * N as usize + x) * 4))
        .map(|i| l(&px[i..i + 4]) - l(&liso[i..i + 4]))
        .collect();
    let mu = d.iter().sum::<f64>() / d.len() as f64;
    (d.iter().map(|v| (v - mu).powi(2)).sum::<f64>() / d.len() as f64).sqrt()
}

/// ⭐ **No Digital o relevo do papel vê-se NA tinta** (dono 08/11: «no digital parece que a sombra do relevo não
/// aparece na tinta… é importante paper funcionar bem no digital independente de Grain»). A tinta Digital não tem
/// corpo: segue o dente, e a luz dele desenha-se por cima dela como no papel nu (antes: a opacidade enterrava-o,
/// ~0). O irmão do lado grosso é `o_dente_do_papel_nao_atravessa_a_tinta_grossa` (o óleo continua a enterrá-lo).
#[test]
fn no_digital_o_relevo_do_papel_ve_se_na_tinta_opaca() {
    let liso = digital_opaco(0.0);
    let px = digital_opaco(0.6);
    let a = dente_nas_linhas(&px, &liso, 45..52);
    let b = dente_nas_linhas(&px, &liso, 2..14);
    assert!(
        liso[(48 * N as usize + 48) * 4 + 1] < 120,
        "o traço não pintou o miolo (a régua não mede nada)"
    );
    assert!(b > 1.0, "o papel nu devia mostrar o dente: {b}");
    // A luz MULTIPLICA a cor: o vermelho (luminância ~74 contra 255) mostra menos níveis que o papel branco. Medido
    // 08/11: 7,29 contra 14,32 (51 %); a lei de antes dava 0,00.
    assert!(
        a >= 0.3 * b,
        "a tinta Digital enterra o relevo do papel: {a:.2} × papel nu {b:.2}"
    );
}

/// **A marca da tinta do Wet é história**: o 1.º traço Wet marca a camada, desfazê-lo tira a marca (a luz
/// do papel volta a ver-se sob a tinta Digital dela) e refazê-lo devolve-a; o traço Digital não marca.
#[test]
fn a_marca_da_tinta_do_wet_desfaz_se_com_o_traco() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (N * N * 4) as usize], N, N);
    let camada = t.layers.active().expect("a camada de tinta");
    let traco = |t: &mut PainterTool| {
        let cp = |pos, phase| CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(cp([10.0, 48.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([40.0, 48.0], PointerPhase::Move));
        t.on_tick(8.0);
        t.on_canvas_pointer(cp([40.0, 48.0], PointerPhase::Up));
        t.on_tick(8.0);
    };
    traco(&mut t);
    assert!(
        t.tintas_molhadas.is_empty(),
        "o traço Digital marcou a camada"
    );
    t.set_paint_media(crate::PaintMedia::WetPaint);
    traco(&mut t);
    assert!(
        t.tintas_molhadas.contains(&camada),
        "o traço Wet não marcou"
    );
    assert!(t.undo_last(), "há o que desfazer");
    assert!(t.tintas_molhadas.is_empty(), "o desfazer deixou a marca");
    assert!(t.redo_last(), "há o que refazer");
    assert!(
        t.tintas_molhadas.contains(&camada),
        "o refazer não a devolveu"
    );
}

/// **A 1.ª marca do Wet refaz a tinta que a camada já tinha**: ela muda a lei da camada (o dente sob a tinta
/// Digital que já lá estava), e o traço Wet só suja o retângulo dele — sem refazer a janela dessa tinta, o resto
/// fica com a lei velha até um recompose qualquer. Desde o B18.4 a marca refaz a janela da tinta da camada (a do
/// undo), não a tela (a paridade ao bit com a tela inteira: `camada_nova_paridade_tests`); por isso a tinta
/// Digital fica LONGE do toque Wet. O 2.º traço Wet na mesma camada não muda nada e não derruba o cache.
#[test]
fn a_primeira_marca_do_wet_refaz_a_imagem_inteira() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_substrate_depth(0.5);
    t.set_brush_size_px(12.0);
    let cp = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    // A tinta Digital da camada, em baixo (y 76), longe do toque Wet (20, 20).
    t.on_canvas_pointer(cp([8.0, 76.0], PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(cp([8.0 + 2.0 * k as f32, 76.0], PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp([88.0, 76.0], PointerPhase::Up));
    t.on_tick(8.0);
    let desce = |t: &mut PainterTool| {
        t.on_canvas_pointer(CanvasPointer {
            pos: [20.0, 20.0],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase: PointerPhase::Down,
        });
    };
    let solta = |t: &mut PainterTool| {
        t.on_canvas_pointer(CanvasPointer {
            pos: [20.0, 20.0],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase: PointerPhase::Up,
        });
        t.on_tick(8.0);
    };
    t.set_paint_media(crate::PaintMedia::WetPaint);
    let _ = t.current_preview();
    assert!(t.composited.is_some(), "a régua precisa do cache cheio");
    assert!(t.dirty_rect.is_none(), "a régua precisa do sujo vazio");
    desce(&mut t);
    let cobre = |r: Region| r.x <= 14 && r.x + r.w >= 82 && r.y <= 72 && r.y + r.h >= 80;
    assert!(
        t.composited.is_none() || t.dirty_rect.is_some_and(cobre),
        "a 1.ª marca do Wet não refez a tinta Digital da camada: sujo {:?}",
        t.dirty_rect
    );
    solta(&mut t);
    let _ = t.current_preview();
    assert!(t.composited.is_some(), "a régua precisa do cache cheio");
    desce(&mut t);
    assert!(
        t.composited.is_some(),
        "o 2.º traço Wet na camada já marcada derrubou o cache à toa"
    );
    solta(&mut t);
}

/// Foto ao dono: dois traços Digitais opacos (Tooth de fábrica e Tooth 0) sobre o papel Cold a Relief 0,6, sem Grain.
/// `PH2D_FOTO=<pasta> cargo test … foto_relevo_do_papel_no_digital -- --ignored` grava `digital_relevo.ppm`.
#[test]
#[ignore = "foto — corre-se à mão"]
fn foto_relevo_do_papel_no_digital() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    const W: u32 = 360;
    const H: u32 = 220;
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_brush_paper_kind(1);
    t.set_substrate_depth(0.6);
    t.set_brush_size_px(46.0);
    let cp = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    for (y, cor, dente) in [
        (60.0, [200, 40, 40], None),
        (150.0, [40, 80, 200], Some(0.0)),
    ] {
        if let Some(d) = dente {
            t.set_brush_paper_depth(d);
        }
        t.set_brush_color_srgb8(cor);
        t.on_canvas_pointer(cp([30.0, y], PointerPhase::Down));
        for k in 1..=60 {
            t.on_canvas_pointer(cp([30.0 + 5.0 * k as f32, y], PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp([330.0, y], PointerPhase::Up));
        t.on_tick(8.0);
    }
    let mut px = t.run_full().0;
    t.apply_impasto_light(
        &mut px,
        Region {
            x: 0,
            y: 0,
            w: W,
            h: H,
        },
    );
    let d = std::env::var("PH2D_FOTO").expect("PH2D_FOTO=<pasta>");
    let mut ppm = format!("P6\n{W} {H}\n255\n").into_bytes();
    for p in px.as_chunks::<4>().0 {
        ppm.extend_from_slice(&p[..3]);
    }
    std::fs::write(format!("{d}/digital_relevo.ppm"), ppm).expect("grava");
}

/// **O Tooth do Digital não esmaece com um Grain escolhido** (auditoria 08/11): desde `ac3afbdad5` o papel
/// morde por cima do Grain, e a mordida lê o Tooth — esmaecê-lo ali escondia um controlo que age.
#[test]
fn o_tooth_do_digital_age_com_um_grain_escolhido() {
    use super::inercia::Dependente;
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_substrate_depth(1.0);
    t.set_brush_texture_kind(ph2d_painter_brush::TextureKind::Noise as u8);
    assert!(
        t.paint.brush.texture.is_active(),
        "a régua precisa do Grain"
    );
    assert!(t.papel_pode_morder(), "a régua precisa do papel a morder");
    assert!(
        !t.inercias().inerte(Dependente::ToothDoDigital),
        "o Tooth esmaece com um Grain escolhido, e ele continua a morder"
    );
    t.set_substrate_depth(0.0);
    assert!(
        t.inercias().inerte(Dependente::ToothDoDigital),
        "sem Relief o Tooth não age e devia esmaecer"
    );
}
