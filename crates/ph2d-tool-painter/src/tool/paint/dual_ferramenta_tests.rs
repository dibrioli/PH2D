//! Os portões do DUAL nas ferramentas (L6, `super::dual_ferramenta`): a borracha, o Blur, o Smear e o Clone no
//! Digital respeitam a ponta inteira — a 2.ª ponta recorta o efeito só dentro do traço, e sem ela tudo é o de hoje.

use super::Dependente;
use crate::LinhaDaPonta;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: u32 = 160;
const H: u32 = 120;
const RAIO: f32 = 15.0;
const FERRAMENTAS: [&str; 4] = ["eraser", "blur", "smear", "clone"];

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Riscas DIAGONAIS opacas (o Clone com deslocamento vertical vê outra cor; o Blur e o Smear têm o que mexer).
fn riscas() -> Vec<u8> {
    let mut px = vec![0u8; (W * H * 4) as usize];
    for (i, p) in px.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let (x, y) = (i as u32 % W, i as u32 / W);
        *p = if ((x + y) / 5) % 2 == 0 {
            [200, 40, 30, 255]
        } else {
            [20, 60, 210, 255]
        };
    }
    px
}

/// O tool Digital com a ferramenta `f` em mão; `dual` arma a 2.ª ponta (modo `dual`, `1..=7`; `0` = arma e
/// desliga).
fn tool(f: &str, dual: Option<f64>) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(riscas(), W, H);
    t.set_paint_media(crate::PaintMedia::Digital);
    t.set_paint_tool_mode(f);
    t.set_brush_size_px(2.0 * RAIO);
    if f == "clone" {
        t.set_clone_source([80.0, 35.0]);
    }
    if let Some(k) = dual {
        t.set_ponta(LinhaDaPonta::DualMode, if k == 0.0 { 2.0 } else { k });
        t.set_ponta(LinhaDaPonta::DualSize, 0.4);
        t.set_ponta(LinhaDaPonta::DualSpacing, 0.6);
        t.set_ponta(LinhaDaPonta::DualScatter, 0.8);
        t.set_ponta(LinhaDaPonta::DualCount, 3.0);
        t.set_ponta(LinhaDaPonta::DualHardness, 0.3);
        if k == 0.0 {
            t.set_ponta(LinhaDaPonta::DualMode, 0.0);
        }
    }
    t
}

/// Um traço recto em y = 60 de x = 30 a x = 130, um evento e um tique por ponto; a tela.
fn traco(mut t: PainterTool) -> Vec<u8> {
    risca(&mut t);
    t.run_full().0
}

/// O traço recto (sem ler a tela).
fn risca(t: &mut PainterTool) {
    let pts: Vec<[f32; 2]> = (0..=50).map(|k| [30.0 + 2.0 * k as f32, 60.0]).collect();
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        t.paint_tick(1.0 / 60.0);
    }
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
}

/// FNV-1a da tela (a régua do «antes ao bit» entre corridas).
fn fnv(px: &[u8]) -> u64 {
    px.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// ⭐ **Cada ferramenta respeita a 2.ª ponta**: o mesmo traço com o dual armado muda a tela (muitos texels), e SÓ
/// dentro da pegada — fora dela é byte a byte a do traço sem o dual.
#[test]
fn as_ferramentas_respeitam_a_segunda_ponta() {
    for f in FERRAMENTAS {
        let sem = traco(tool(f, None));
        let com = traco(tool(f, Some(1.0)));
        let original = riscas();
        let mexeu = sem
            .as_chunks::<4>()
            .0
            .iter()
            .zip(original.as_chunks::<4>().0)
            .filter(|(a, b)| a != b)
            .count();
        let (mut dentro, mut fora) = (0usize, 0usize);
        for (i, (a, b)) in sem
            .as_chunks::<4>()
            .0
            .iter()
            .zip(com.as_chunks::<4>().0)
            .enumerate()
        {
            if a != b {
                // A pegada é MEDIDA: o que a ferramenta sem o dual mexeu.
                if *a != original.as_chunks::<4>().0[i] {
                    dentro += 1;
                } else {
                    fora += 1;
                }
            }
        }
        eprintln!(
            "{f}: a ferramenta mexeu {mexeu} texels · o dual mudou {dentro} dentro e {fora} fora · sem={:016x}",
            fnv(&sem)
        );
        assert!(mexeu > 1000, "{f}: a fixtura não mexeu na tela ({mexeu})");
        assert!(
            dentro > mexeu / 10,
            "{f}: o dual mudou só {dentro} de {mexeu} texels"
        );
        assert_eq!(fora, 0, "{f}: o dual mudou {fora} texels FORA da pegada");
        // …e RECORTA, não desliga: a ferramenta ainda age onde a 2.ª ponta pinta (uma máscara vazia tirava tudo).
        let ainda = com
            .as_chunks::<4>()
            .0
            .iter()
            .zip(original.as_chunks::<4>().0)
            .filter(|(a, b)| a != b)
            .count();
        eprintln!("{f}: com o dual a ferramenta ainda mexe {ainda} texels");
        assert!(
            ainda > mexeu / 10,
            "{f}: com o dual a ferramenta só mexe {ainda}"
        );
    }
}

/// ⭐ **O dual desligado é o de hoje, ao bit**: armar a 2.ª ponta e voltar a «Off» dá a tela de quem nunca a teve.
#[test]
fn o_dual_desligado_nas_ferramentas_e_o_de_hoje() {
    for f in FERRAMENTAS {
        let sem = traco(tool(f, None));
        let desligado = traco(tool(f, Some(0.0)));
        eprintln!("{f}: sem dual = {:016x}", fnv(&sem));
        assert!(sem == desligado, "{f}: o dual desligado mudou a tela");
    }
}

/// ⭐ **Cada modo do dual é outra borracha** (a cobertura da 1.ª recortada, não um interruptor): Multiply, Darken,
/// Subtract e Overlay dão telas diferentes entre si e da sem dual.
#[test]
fn cada_modo_do_dual_e_outra_borracha() {
    // A borracha a 0,6 com tecto (a cobertura `a < 1`): com `a = 1` o Darken é o Multiply (`min(1, m) = m`).
    let borracha = |k: Option<f64>| {
        let mut t = tool("eraser", k);
        t.paint.brush.strength = 0.6;
        t.paint.brush.accumulate = false;
        traco(t)
    };
    let sem = borracha(None);
    let telas: Vec<Vec<u8>> = [1.0, 2.0, 3.0, 4.0]
        .into_iter()
        .map(|k| borracha(Some(k)))
        .collect();
    for (a, ta) in telas.iter().enumerate() {
        assert!(*ta != sem, "o modo {} não mudou a borracha", a + 1);
        for (b, tb) in telas.iter().enumerate().skip(a + 1) {
            assert!(ta != tb, "os modos {} e {} dão a mesma tela", a + 1, b + 1);
        }
    }
}

/// ⭐ **A linha e o motor respondem pela MESMA porta**: com cada ferramenta em mão as linhas do dual NÃO esmaecem;
/// com a Máscara em mão (onde ele ainda não age) esmaecem.
#[test]
fn as_linhas_do_dual_acordam_nas_ferramentas() {
    for f in FERRAMENTAS {
        let t = tool(f, Some(1.0));
        let i = t.inercias();
        assert!(
            !i.inerte(Dependente::DualForaDoTraco) && !i.inerte(Dependente::DualForaDoDigital),
            "{f}: as linhas do dual esmaecem"
        );
    }
    let mascara = tool("mask", Some(1.0));
    assert!(
        mascara.inercias().inerte(Dependente::DualForaDoTraco),
        "a Máscara não lê o dual e a linha não esmaece"
    );
}

/// ⭐ **Um Ctrl+Z desfaz o traço com o dual ao bit**: a tela volta às riscas em cada ferramenta.
#[test]
fn desfazer_o_traco_com_o_dual_volta_ao_bit() {
    for f in FERRAMENTAS {
        let mut t = tool(f, Some(4.0));
        risca(&mut t);
        assert!(
            t.canvas_rgba.as_slice() != riscas().as_slice(),
            "{f}: o traço não mexeu"
        );
        assert!(t.undo_last(), "{f}: nada para desfazer");
        assert!(
            t.canvas_rgba.as_slice() == riscas().as_slice(),
            "{f}: o desfazer não voltou ao bit"
        );
    }
}

/// A mesma ferramenta `f` com `comum` e, por cima, `extra`: (texels que a ferramenta mexeu sem o `extra`, texels
/// que o `extra` mudou dentro dessa pegada, fora dela).
fn o_extra_age(
    f: &str,
    tilt: [f32; 2],
    comum: impl Fn(&mut PainterTool),
    extra: impl Fn(&mut PainterTool),
) -> (usize, usize, usize) {
    let corre = |com: bool| {
        let mut t = tool(f, None);
        comum(&mut t);
        if com {
            extra(&mut t);
        }
        risca_com(&mut t, tilt);
        t.run_full().0
    };
    let (sem, com, original) = (corre(false), corre(true), riscas());
    let px = |v: &[u8], i: usize| -> [u8; 4] { v.as_chunks::<4>().0[i] };
    let n = (W * H) as usize;
    let mexeu = (0..n).filter(|&i| px(&sem, i) != px(&original, i)).count();
    let (mut dentro, mut fora) = (0, 0);
    for i in (0..n).filter(|&i| px(&sem, i) != px(&com, i)) {
        if px(&sem, i) != px(&original, i) || px(&com, i) != px(&original, i) {
            dentro += 1;
        } else {
            fora += 1;
        }
    }
    (mexeu, dentro, fora)
}

/// O traço recto com a caneta deitada `tilt`.
fn risca_com(t: &mut PainterTool, tilt: [f32; 2]) {
    let pts: Vec<[f32; 2]> = (0..=50).map(|k| [30.0 + 2.0 * k as f32, 60.0]).collect();
    let cp = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt,
        phase,
    };
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        t.paint_tick(1.0 / 60.0);
    }
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
}

/// A régua de uma linha numa ferramenta: a falha (se a há), para o portão as juntar todas antes de reprovar.
fn relata(item: &str, f: &str, (mexeu, dentro, fora): (usize, usize, usize)) -> Option<String> {
    eprintln!(
        "{item} · {f}: a ferramenta mexeu {mexeu} · a linha mudou {dentro} dentro e {fora} fora"
    );
    if mexeu <= 500 {
        Some(format!(
            "{item} · {f}: a fixtura não mexeu na tela ({mexeu})"
        ))
    } else if dentro <= mexeu / 20 {
        Some(format!(
            "{item} · {f}: a linha mudou só {dentro} de {mexeu}"
        ))
    } else {
        (fora > 0).then(|| format!("{item} · {f}: a linha mudou {fora} texels fora da pegada"))
    }
}

fn sem_falhas(falhas: Vec<Option<String>>) {
    let falhas: Vec<String> = falhas.into_iter().flatten().collect();
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// A Shape de imagem de teste: um degradê em X (32×32).
fn forma_degrade(t: &mut PainterTool) {
    let lum: Vec<u8> = (0..32 * 32).map(|i| ((i % 32) * 8) as u8).collect();
    t.set_brush_shape_image(lum, 32, 32);
}

/// ⭐ **O tom da Shape (a rampa B&W) age em toda a ferramenta que não pinta cor**: a borracha apaga e o Smear
/// arrasta pela cobertura remapeada, como o Blur e o Clone (o painel tranca o B&W nelas).
#[test]
fn o_tom_da_shape_age_na_borracha_e_no_smear() {
    let rampa = |t: &mut PainterTool| {
        t.paint.shape_color_ramp = ph2d_color::ColorRamp::new(
            vec![
                ph2d_color::RampStop::new(0.0, [1.0, 1.0, 1.0, 1.0]),
                ph2d_color::RampStop::new(1.0, [0.0, 0.0, 0.0, 1.0]),
            ],
            ph2d_color::RampColorMode::Rgb,
            ph2d_color::RampInterp::Linear,
        );
        t.toggle_shape_ramp_enabled();
    };
    sem_falhas(
        ["blur", "clone", "eraser", "smear"]
            .map(|f| {
                relata(
                    "Shape Tone",
                    f,
                    o_extra_age(f, [0.0; 2], forma_degrade, rampa),
                )
            })
            .into(),
    );
}

/// ⭐ **A ponta de píxel age no Blur, no Smear e no Clone**: a pegada binária na grelha; no Clone cada texel
/// copiado é o da fonte INTEIRO (sem orla que misture).
#[test]
fn a_ponta_de_pixel_age_nas_ferramentas() {
    let pixel = |t: &mut PainterTool| t.set_ponta(LinhaDaPonta::Pixel, 1.0);
    sem_falhas(
        ["blur", "smear", "clone"]
            .map(|f| relata("Pixel", f, o_extra_age(f, [0.0; 2], |_| {}, pixel)))
            .into(),
    );
    // O Clone copia as riscas: com a pegada binária cada texel copiado é uma das DUAS cores (o RGB; o alfa da
    // fonte fora da tela é outro assunto); com a orla macia há misturas.
    let mesclados = |com_pixel: bool| {
        let mut t = tool("clone", None);
        if com_pixel {
            pixel(&mut t);
        }
        risca_com(&mut t, [0.0; 2]);
        let fim = t.run_full().0;
        let paleta = [[200u8, 40, 30], [20, 60, 210]];
        fim.as_chunks::<4>()
            .0
            .iter()
            .filter(|p| !paleta.contains(&[p[0], p[1], p[2]]))
            .count()
    };
    let (macio, binario) = (mesclados(false), mesclados(true));
    eprintln!(
        "Pixel · clone: {macio} texels misturados com a orla macia, {binario} com a ponta de píxel"
    );
    assert!(
        macio > 100,
        "a fixtura: a orla macia não misturou ({macio})"
    );
    assert_eq!(binario, 0, "o Clone de píxel misturou {binario} texels");
}

/// ⭐ **A mão no Smear**: a pressão e a velocidade endurecem/amaciam a queda, e a caneta deitada alisa o grão —
/// como no pincel, na borracha, no Blur e no Clone.
#[test]
fn a_mao_age_no_smear() {
    // O grão e a caneta deitada AO COMPRIDO do traço (o azimute em X); o extra é o alisar.
    let grao = |t: &mut PainterTool| {
        t.set_brush_texture_kind(1);
        t.paint.brush.pen_tilt.start_deg = 0.0;
        t.paint.brush.pen_tilt.full_deg = 1.0;
    };
    // O `pressureHardness` com a pressão 1 deste risco é a Hardness (o rato não muda nada, 25/10): a dureza da mão
    // chega ao smear pelo MESMO canal (`dureza_da_mao`) que o `speedHardness` prova aqui.
    sem_falhas(vec![
        relata(
            "speedHardness",
            "smear",
            o_extra_age(
                "smear",
                [0.0; 2],
                |t| t.paint.brush.hardness = 0.8,
                |t| t.paint.dynamics.speed_hardness = 1.0,
            ),
        ),
        relata(
            "tiltSmooth",
            "smear",
            o_extra_age("smear", [0.9, 0.0], grao, |t| {
                t.paint.brush.pen_tilt.smooth = 1.0
            }),
        ),
    ]);
}

/// SONDA (o caminho do painel do censo): Sync, a ponta de imagem COLORIDA, a ferramenta pelo fio do trilho.
/// Medido 23/10: na ponta de IMAGEM a dureza não entra na silhueta (`compose_shape_silhouette_kind`: Image =
/// a amostra); o que muda no pincel/borracha/Blur (|Δ| ≤ 6) é a troca de rota (dab com `tilt` ⇒ sem cache).
/// O tom com a rampa de fábrica (preto→branco) é a identidade: só age com um stop movido.
#[test]
#[ignore = "sonda"]
fn sonda_caminho_do_painel() {
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::PanelEvent;
    let ponta = |t: &mut PainterTool| {
        const L: u32 = 16;
        let mut px = vec![255u8; (L * L * 4) as usize];
        for (i, p) in px.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % L, i as u32 / L);
            p[0] = (x * 16) as u8;
            p[1] = (y * 16) as u8;
            p[2] = 60;
        }
        t.set_brush_shape_image_rgba(&px, L, L, Some(7));
    };
    let corre = |f: &str, extra: &dyn Fn(&mut PainterTool)| {
        let mut t = PainterTool::default();
        t.set_source(riscas(), W, H);
        t.set_paint_media(crate::PaintMedia::Digital);
        t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_BRUSH_SYNC));
        ponta(&mut t);
        t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            f.to_owned(),
        ));
        extra(&mut t);
        risca_com(&mut t, [0.0; 2]);
        t.run_full().0
    };
    let mede = |rot: &str, f: &str, extra: &dyn Fn(&mut PainterTool)| {
        let (a, b) = (corre(f, &|_| {}), corre(f, extra));
        let n = a
            .as_chunks::<4>()
            .0
            .iter()
            .zip(b.as_chunks::<4>().0)
            .filter(|(x, y)| x != y)
            .count();
        let max = a
            .iter()
            .zip(&b)
            .map(|(x, y)| x.abs_diff(*y))
            .max()
            .unwrap_or(0);
        eprintln!("SONDA {rot} · {f}: {n} texels, |Δ| máx {max}");
    };
    for f in ["brush", "eraser", "blur"] {
        mede("ShapeTone", f, &|t| {
            t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_SHAPE_RAMP_ENABLE))
        });
    }
    for f in ["brush", "eraser", "blur", "clone", "smear"] {
        mede("pressureHardness", f, &|t| {
            t.paint.dynamics.pressure_hardness = 0.6
        });
        mede("speedHardness", f, &|t| {
            t.paint.dynamics.speed_hardness = 0.6
        });
        mede("tiltSmooth", f, &|t| t.paint.brush.pen_tilt.smooth = 0.6);
    }
    // O tom com uma rampa que NÃO é a identidade (o branco a 0,3).
    for f in ["brush", "eraser", "blur", "smear"] {
        mede("ShapeTone+stop", f, &|t| {
            t.handle_panel_event(PanelEvent::Click(crate::ids::PAINTER_SHAPE_RAMP_ENABLE));
            t.shape_ramp_move_stop(1, 0.3);
        });
    }
}
