//! DIAGNÓSTICO — o preço do **Composite Brush**, dos **fios** e do **Solid** no Wet Paint, ANTES de
//! os construir (doc 46 itens 8 e 9; os kill-criteria do §1 escritos antes de qualquer build).
//!
//! ⚠️ **Isto NÃO implementa nada.** Mede, pela porta do produto (`on_canvas_pointer` + um
//! `paint_tick` por evento, que é o quadro), o que cada peça custaria na água:
//!
//! | linha | o que corre | o que estima |
//! |---|---|---|
//! | `D pincel` · `D pilha 4` | Digital, uma camada · o Composite com Brush · Smear · Blur · Erase | a régua do kill-criterion |
//! | `W Paint/Smear/Blend/Erase` | cada ferramenta da água SOZINHA no mesmo gesto | a pilha na água ≥ a SOMA (cada camada é um despacho por carimbo) |
//! | `D fios` · `W fios` | o Sketchy denso no Digital · os MESMOS fios (o motor de traço a costurá-los ao lado) carimbados na água a 1 px | a teia densa na água |
//! | `D solid` · `W solid` | o Solid no Digital · um carimbo de água da ÁREA da região, a cada quadro | a mancha na água (a porta por máscara não existe; o disco de área igual é o seu custo de depósito) |
//!
//! ⛔ **A soma é um PISO, não uma previsão:** no Digital a pilha corre por CAMADA sobre o traço
//! inteiro (`composite_pilha`), e esse custo a mais não tem análogo medível sem o construir.
//!
//! O gesto é um RABISCO (espiral de 4 voltas que volta à própria vizinhança — o regime em que a
//! premissa da recomposição regional caiu, `composite_pilha` §refutada), num canvas de `1024²`, raio
//! `24`. As variantes correm INTERCALADAS ([`RODADAS`] rodadas, ordem rodada) e fica o MÍNIMO
//! (a mediana ao lado é o controlo); o `loadavg` sai ao lado.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --profile smoke --lib diag_o_preco_da_agua -- --ignored
//! --nocapture` (uma compilação, sem esperar a máquina calma).
use super::*;
use ph2d_painter_brush::line_kind::{LineKind, SKETCHY_DENSITY_MAX};
use ph2d_painter_brush::stroke::threads::Thread;
use ph2d_painter_brush::{Stroke, StrokePoint};
use std::time::Instant;

const SIZE: u32 = 1024;
const RAIO: f32 = 24.0;
/// Rodadas INTERCALADAS (regra do dono, 2026-10-05: `docs/DevOps/MEDIR_VELOCIDADE.md`): cada rodada
/// corre um gesto de CADA variante, com a ordem rodada; fica-se com o MÍNIMO das rodadas.
const RODADAS: usize = 7;
const VOLTAS: usize = 4;
const PASSOS: usize = VOLTAS * 40;

fn ponteiro(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O ponto `i` da espiral (raio 20 → 200 px em torno do centro).
fn ponto(i: usize) -> [f32; 2] {
    #[allow(clippy::cast_precision_loss)]
    let u = i as f32 / PASSOS as f32;
    #[allow(clippy::cast_precision_loss)]
    let ang = u * VOLTAS as f32 * std::f32::consts::TAU;
    let r = 20.0 + u * 180.0;
    let c = (SIZE / 2) as f32;
    [c + r * ang.cos(), c + r * ang.sin()]
}

fn ferramenta() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.paint.brush.radius_px = RAIO;
    t.paint.brush.color = [0.6, 0.0, 0.0];
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t
}

fn molhada(t: &mut PainterTool, ferr: usize) {
    t.set_paint_tool_mode("wetpaint");
    t.pick_wet_tool(ferr);
}

/// Um gesto inteiro pela porta do produto; `extra(t, i)` corre depois do Move `i` (antes do quadro).
/// Devolve os ms do gesto (Down → Moves com o quadro → Up).
fn gesto(t: &mut PainterTool, extra: &mut dyn FnMut(&mut PainterTool, usize)) -> f64 {
    let t0 = Instant::now();
    t.on_canvas_pointer(ponteiro(ponto(0), PointerPhase::Down));
    for i in 1..=PASSOS {
        t.on_canvas_pointer(ponteiro(ponto(i), PointerPhase::Move));
        extra(t, i);
        t.paint_tick(1.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(ponto(PASSOS), PointerPhase::Up));
    t0.elapsed().as_secs_f64() * 1e3
}

fn nada(_: &mut PainterTool, _: usize) {}

/// A tela depois de UM gesto — o CONTROLO de que a linha da tabela pinta o que diz.
fn tela(prep: &dyn Fn() -> PainterTool, extra: &mut dyn FnMut(&mut PainterTool, usize)) -> Vec<u8> {
    let mut t = prep();
    gesto(&mut t, extra);
    t.canvas_rgba.to_vec()
}

/// Píxeis que diferem entre duas telas.
fn diferem(a: &[u8], b: &[u8]) -> usize {
    a.chunks_exact(4)
        .zip(b.chunks_exact(4))
        .filter(|(x, y)| x != y)
        .count()
}

/// Uma camada de tinta molhada por baixo (fora do relógio), e depois a ferramenta `ferr`.
fn agua_com_tinta(ferr: usize) -> PainterTool {
    let mut t = ferramenta();
    molhada(&mut t, 0);
    gesto(&mut t, &mut nada);
    molhada(&mut t, ferr);
    t
}

fn sketchy(t: &mut PainterTool) {
    t.paint.brush.line_kind = LineKind::Sketchy;
    t.paint.brush.sketchy_reach = 3.0;
    t.paint.brush.sketchy_density = SKETCHY_DENSITY_MAX;
    t.paint.brush.thread_width_px = 1.0;
    t.paint.brush.thread_opacity = 0.25;
}

/// Os fios que o motor de traço costura em cada evento da espiral, com o pincel de `t`.
fn fios_por_evento(t: &PainterTool) -> Vec<Vec<Thread>> {
    let mut s = Stroke::new(t.stroke_spec(), t.paint.dynamics, 7);
    let mut dabs = Vec::new();
    let mut fios = Vec::new();
    s.begin(
        StrokePoint {
            pos: ponto(0),
            pressure: 1.0,
        },
        &mut dabs,
    );
    let mut por_evento = vec![Vec::new()];
    for i in 1..=PASSOS {
        dabs.clear();
        s.extend(
            StrokePoint {
                pos: ponto(i),
                pressure: 1.0,
            },
            &mut dabs,
        );
        s.take_threads(&mut fios);
        por_evento.push(fios.clone());
    }
    por_evento
}

fn carimbo(center: [f32; 2], r: f32, coverage: f32, color: [f32; 3]) -> Dab {
    Dab {
        center,
        radius_px: r,
        coverage,
        color,
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: r,
    }
}

/// Um fio como carimbos de água a 1 px, raio ½ — a rota mais barata que a grelha de 1 px por célula
/// aceita sem uma porta nova.
fn carimbos_do_fio(fios: &[Thread], cor: [f32; 3]) -> Vec<Dab> {
    let mut out = Vec::new();
    for f in fios {
        let (dx, dy) = (f[2] - f[0], f[3] - f[1]);
        let len = dx.hypot(dy).max(1.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = len.ceil() as usize;
        #[allow(clippy::cast_precision_loss)]
        for k in 0..=n {
            let u = k as f32 / n as f32;
            out.push(carimbo([f[0] + dx * u, f[1] + dy * u], 0.5, 0.25, cor));
        }
    }
    out
}

/// A área do polígono dos pontos `0..=i` da espiral (fecho implícito), em px².
fn area_ate(i: usize) -> f32 {
    let p: Vec<[f32; 2]> = (0..=i).map(ponto).collect();
    let mut a = 0.0;
    for k in 0..p.len() {
        let (u, v) = (p[k], p[(k + 1) % p.len()]);
        a += u[0] * v[1] - v[0] * u[1];
    }
    (a * 0.5).abs()
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".into())
}

fn pilha_cheia() -> PainterTool {
    let mut t = ferramenta();
    t.paint.composite_enabled = true;
    t.paint.composite_len = 4;
    for (pos, (op, s)) in [
        (CompositeOp::Blur, 0.5),
        (CompositeOp::Smear, 0.5),
        (CompositeOp::Erase, 0.3),
        (CompositeOp::Brush, 1.0),
    ]
    .into_iter()
    .enumerate()
    {
        t.paint.composite[pos] = CompositeLayer {
            op,
            strength: s,
            ..CompositeLayer::default()
        };
    }
    t
}

fn com_sketchy() -> PainterTool {
    let mut t = ferramenta();
    sketchy(&mut t);
    t
}

fn com_solid() -> PainterTool {
    let mut t = ferramenta();
    t.paint.brush.style_solid = true;
    t
}

fn so_agua() -> PainterTool {
    let mut t = ferramenta();
    molhada(&mut t, 0);
    t
}

/// Os CONTROLOS: cada linha da tabela pinta o que diz. Sem relógio, logo valem sob carga.
fn controlos() {
    let fios = fios_por_evento(&com_sketchy());
    let cor = ferramenta().paint.brush.color;
    let d_base = tela(&ferramenta, &mut nada);
    let c_pilha = diferem(&d_base, &tela(&pilha_cheia, &mut nada));
    let c_dfios = diferem(&d_base, &tela(&com_sketchy, &mut nada));
    let c_dsolid = diferem(&d_base, &tela(&com_solid, &mut nada));
    let base = tela(&so_agua, &mut nada);
    let c_fios = diferem(
        &base,
        &tela(&so_agua, &mut |t, i| {
            t.stamp_dabs(&carimbos_do_fio(&fios[i], cor));
        }),
    );
    let c_solid = diferem(
        &base,
        &tela(&so_agua, &mut |t, i| {
            if i == PASSOS {
                t.stamp_dabs(&[carimbo([512.0, 512.0], 150.0, 1.0, [0.0, 0.0, 0.6])]);
            }
        }),
    );
    // A tinta de baixo SOZINHA (sem o 2.º gesto): é contra ela que a ferramenta tem de mexer.
    let com_tinta = agua_com_tinta(0).canvas_rgba.to_vec();
    let c_smear = diferem(&com_tinta, &tela(&|| agua_com_tinta(2), &mut nada));
    let c_blend = diferem(&com_tinta, &tela(&|| agua_com_tinta(3), &mut nada));
    let c_erase = diferem(&com_tinta, &tela(&|| agua_com_tinta(1), &mut nada));
    let n_fios: usize = fios.iter().map(Vec::len).sum();
    eprintln!(
        "controlos (píxeis que mudam): D pilha {c_pilha} · D fios {c_dfios} · D solid {c_dsolid} · \
         W fios {c_fios} ({n_fios} fios) · W disco {c_solid} · W Smear {c_smear} · W Blend \
         {c_blend} · W Erase {c_erase}"
    );
    for (nome, n) in [
        ("D pilha", c_pilha),
        ("D fios", c_dfios),
        ("D solid", c_dsolid),
        ("W fios", c_fios),
        ("W disco", c_solid),
        ("W Smear", c_smear),
        ("W Blend", c_blend),
        ("W Erase", c_erase),
    ] {
        assert!(n > 0, "a linha «{nome}» mediria uma porta que não pinta");
    }
}

/// Os controlos sozinhos (sem relógio): `cargo test -p ph2d-tool-painter diag_os_controlos_da_agua
/// -- --ignored --nocapture`.
#[test]
#[ignore = "controlo da sonda de relógio abaixo"]
fn diag_os_controlos_da_agua() {
    controlos();
}

/// Uma variante da tabela: como armar a ferramenta (fora do relógio) e o que corre depois de cada Move.
struct Variante<'a> {
    nome: &'static str,
    prep: Box<dyn Fn() -> PainterTool + 'a>,
    extra: Box<dyn FnMut(&mut PainterTool, usize) + 'a>,
}

fn variante<'a>(
    nome: &'static str,
    prep: impl Fn() -> PainterTool + 'a,
    extra: impl FnMut(&mut PainterTool, usize) + 'a,
) -> Variante<'a> {
    Variante {
        nome,
        prep: Box::new(prep),
        extra: Box::new(extra),
    }
}

/// `(mínimo, mediana)` em ms POR QUADRO de cada variante, sobre [`RODADAS`] rodadas intercaladas com a
/// ordem rodada (a rodada `r` começa na variante `r`), para a deriva da máquina cair em todas.
fn intercalado(vs: &mut [Variante<'_>]) -> Vec<(f64, f64)> {
    let n = vs.len();
    let mut ms = vec![Vec::with_capacity(RODADAS); n];
    #[allow(clippy::cast_precision_loss)]
    let quadros = PASSOS as f64;
    for r in 0..RODADAS {
        for k in 0..n {
            let v = &mut vs[(r + k) % n];
            let mut t = (v.prep)();
            ms[(r + k) % n].push(gesto(&mut t, &mut *v.extra) / quadros);
        }
    }
    ms.into_iter()
        .map(|mut m| {
            m.sort_by(f64::total_cmp);
            (m[0], m[m.len() / 2])
        })
        .collect()
}

/// SONDA — as três peças do Wet Paint contra o kill-criterion, numa rodada só, INTERCALADA.
#[test]
#[ignore = "diagnóstico de relógio: roda sob demanda, no perfil smoke"]
fn diag_o_preco_da_agua() {
    controlos();
    let fios = fios_por_evento(&com_sketchy());
    let cor = ferramenta().paint.brush.color;
    let n_fios: usize = fios.iter().map(Vec::len).sum();
    let n_carimbos: usize = fios.iter().map(|f| carimbos_do_fio(f, cor).len()).sum();
    let antes = carga();
    let mut vs = [
        variante("D pincel", ferramenta, nada),
        variante("D pilha 4", pilha_cheia, nada),
        variante("D fios", com_sketchy, nada),
        variante("D solid", com_solid, nada),
        variante("W Paint", so_agua, nada),
        variante("W Smear", || agua_com_tinta(2), nada),
        variante("W Blend", || agua_com_tinta(3), nada),
        variante("W Erase", || agua_com_tinta(1), nada),
        variante("W Paint + fios", so_agua, |t: &mut PainterTool, i| {
            t.stamp_dabs(&carimbos_do_fio(&fios[i], cor));
        }),
        // O Solid na água: um carimbo da ÁREA da região, a cada quadro.
        variante("W Paint + solid", so_agua, |t: &mut PainterTool, i| {
            let p: Vec<[f32; 2]> = (0..=i).map(ponto).collect();
            #[allow(clippy::cast_precision_loss)]
            let n = p.len() as f32;
            let c = p
                .iter()
                .fold([0.0, 0.0], |a, q| [a[0] + q[0] / n, a[1] + q[1] / n]);
            let r = (area_ate(i) / std::f32::consts::PI).sqrt().max(1.0);
            t.stamp_dabs(&[carimbo(c, r, 1.0, [0.6, 0.0, 0.0])]);
        }),
    ];
    let r = intercalado(&mut vs);
    let m = |nome: &str| r[vs.iter().position(|v| v.nome == nome).expect("variante")].0;
    eprintln!(
        "loadavg antes {antes} · depois {} · canvas {SIZE}² · raio {RAIO} · espiral {VOLTAS} voltas \
         ({PASSOS} quadros) · {RODADAS} rodadas intercaladas · {n_fios} fios / {n_carimbos} \
         carimbos de água no gesto",
        carga()
    );
    eprintln!("ms POR QUADRO          mínimo   (mediana)");
    for (v, (min, med)) in vs.iter().zip(&r) {
        eprintln!("  {:<18} {min:>8.3}   ({med:.3})", v.nome);
    }
    let regua = m("D pilha 4");
    let soma = m("W Paint") + m("W Smear") + m("W Blend") + m("W Erase");
    eprintln!("kill-criterion (régua = D pilha 4 = {regua:.3} ms):");
    eprintln!("  W soma das 4 (piso) {soma:>8.3} → {:.2}×", soma / regua);
    eprintln!(
        "  W Paint + fios      {:>8.3} → {:.2}× a régua · {:.2}× o D fios",
        m("W Paint + fios"),
        m("W Paint + fios") / regua,
        m("W Paint + fios") / m("D fios")
    );
    eprintln!(
        "  W Paint + solid     {:>8.3} → {:.2}× a régua · {:.2}× o D solid",
        m("W Paint + solid"),
        m("W Paint + solid") / regua,
        m("W Paint + solid") / m("D solid")
    );
}
