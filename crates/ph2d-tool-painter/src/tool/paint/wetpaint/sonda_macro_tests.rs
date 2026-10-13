//! **Sonda das MACROS do Wet Paint** (ignorada; medição, nunca barra): gravar as ações do artista e
//! reproduzi-las ao byte, contando os PASSOS da sim entre ações em vez de segundos de parede.
//!
//! (a) o custo de um passo fixo e o de reproduzir um traço · (b) o fenómeno: duas corridas no WORKER
//! divergem · (c) a hipótese: a reprodução no relógio fixo com os passos gravados iguala a corrida de
//! onde vieram · (d) o controlo: duas reproduções iguais dão 0.
//!
//! Corre: `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib -- --ignored
//! --nocapture --test-threads=1 sonda_macro`.
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_painter_brush::Falloff;
use std::time::{Duration, Instant};

const AMOSTRAS: u32 = 60;
const POR_QUADRO: u32 = 3;
const QUADRO: Duration = Duration::from_micros(16_666);

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Um traço do roteiro, em coordenadas de uma tela de 1024 (escala com a tela).
#[derive(Clone, Copy)]
struct Traco {
    de: [f32; 2],
    ate: [f32; 2],
    cor: [f32; 3],
}

const SPLASH: [Traco; 4] = [
    Traco { de: [150.0, 400.0], ate: [850.0, 400.0], cor: [0.8, 0.2, 0.1] },
    Traco { de: [200.0, 200.0], ate: [800.0, 700.0], cor: [0.1, 0.3, 0.8] },
    Traco { de: [520.0, 150.0], ate: [520.0, 850.0], cor: [0.9, 0.8, 0.1] },
    Traco { de: [150.0, 600.0], ate: [850.0, 620.0], cor: [0.2, 0.7, 0.3] },
];

/// O relógio da água DESLIGADO: `relogio_fixo = NaN` ⇒ o tique nunca entrega o motor ao worker e
/// nunca dá passo (`NaN as usize == 0`); os passos são só os de `wet_step_sync`.
fn desliga_o_relogio(t: &mut PainterTool) {
    t.set_wet_relogio_fixo(true);
    t.paint.wetpaint.relogio_fixo = Some(f64::NAN);
}

fn ferramenta(n: u32, fixo: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (n * n * 4) as usize], n, n);
    let b = BrushSpec {
        radius_px: 30.0 * n as f32 / 1024.0,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_media(PaintMedia::WetPaint);
    if fixo {
        desliga_o_relogio(&mut t);
    }
    t
}

fn cor(t: &mut PainterTool, c: [f32; 3]) {
    t.paint.brush.color = c;
    for b in &mut t.paint.brush_by_mode {
        b.color = c;
    }
}

/// O traço como a macro o reproduz: `Down`, 60 amostras em quadros de 3 + o tique, `Up` (o tique
/// seguinte é de quem chama: o gravador lê o contador ANTES dele, que é quem devolve o motor ao worker).
/// `ritmo` = dorme o resto do quadro (o worker anda ao relógio real). Devolve o pior quadro (ms).
fn traco(t: &mut PainterTool, tr: Traco, n: u32, ritmo: bool) -> f32 {
    let s = n as f32 / 1024.0;
    let p = |k: u32| {
        let f = k as f32 / AMOSTRAS as f32;
        [
            s * (tr.de[0] + (tr.ate[0] - tr.de[0]) * f),
            s * (tr.de[1] + (tr.ate[1] - tr.de[1]) * f),
        ]
    };
    cor(t, tr.cor);
    let mut pior = 0.0f32;
    t.on_canvas_pointer(cp(p(0), PointerPhase::Down));
    for q in 0..AMOSTRAS / POR_QUADRO {
        let a = Instant::now();
        for m in 1..=POR_QUADRO {
            t.on_canvas_pointer(cp(p(q * POR_QUADRO + m), PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        t.paint_tick(1.0 / 60.0);
        let gasto = a.elapsed();
        pior = pior.max(gasto.as_secs_f32() * 1e3);
        if ritmo && let Some(r) = QUADRO.checked_sub(gasto) {
            std::thread::sleep(r);
        }
    }
    t.on_canvas_pointer(cp(p(AMOSTRAS), PointerPhase::Up));
    pior
}

/// Espera de parede ao ritmo do app (tique + o resto do quadro).
fn espera(t: &mut PainterTool, ms: u64) {
    let fim = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < fim {
        let a = Instant::now();
        t.paint_tick(1.0 / 60.0);
        if let Some(r) = QUADRO.checked_sub(a.elapsed()) {
            std::thread::sleep(r);
        }
    }
}

/// O estado do contador com o motor EM CASA: `(sim.frame, passo em voo?, sim_steps() do worker)`.
fn contador(t: &mut PainterTool) -> (u64, bool, u64) {
    t.wet_bring_home();
    t.paint.wetpaint.session.as_ref().map_or((0, false, 0), |s| {
        (s.engine.sim.frame, s.engine.sim.step_pending(), s.sim_steps())
    })
}

fn drena(t: &mut PainterTool) {
    t.wet_bring_home();
    if let Some(s) = t.paint.wetpaint.session.as_mut() {
        s.engine.drain_step();
    }
}

/// Passos síncronos até `sim.frame == alvo` (o passo fecha sempre: `wet_step_sync` roda estágios até ele).
fn passos_ate(t: &mut PainterTool, alvo: u64) {
    let mut guarda = 0;
    while contador(t).0 < alvo {
        t.wet_step_sync(1);
        guarda += 1;
        assert!(guarda < 100_000, "o motor parou antes do alvo {alvo}");
    }
    assert_eq!(contador(t).0, alvo, "passou do alvo");
}

fn texels_diferentes(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

/// A tela depois do composite, com o motor em casa e sem passo em voo.
fn tela(t: &mut PainterTool) -> Vec<u8> {
    drena(t);
    t.wetpaint_composite();
    t.canvas_rgba.to_vec()
}

const ROTEIRO: [Traco; 3] = [SPLASH[0], SPLASH[1], SPLASH[2]];
const ESPERAS_MS: [u64; 3] = [150, 400, 250];
const N: u32 = 1024;

/// O que a gravação guarda de uma corrida.
#[derive(Debug, Clone, Default)]
struct Gravacao {
    /// por traço: (frame no Down, em voo no Down, sim_steps no Down, frame no Up, em voo no Up)
    tracos: Vec<(u64, bool, u64, u64, bool)>,
    fim: u64,
    fim_atomico: u64,
}

/// (b) uma corrida no WORKER, ao relógio real. `drena_no_down`: o gravador fecha o passo em voo antes
/// do 1.º dab (a variante que torna o ponto de leitura um passo INTEIRO).
fn corrida_worker(drena_no_down: bool) -> (Vec<u8>, Gravacao, PainterTool) {
    let mut t = ferramenta(N, false);
    let mut g = Gravacao::default();
    for (k, tr) in ROTEIRO.iter().enumerate() {
        if drena_no_down {
            drena(&mut t);
        }
        let (f0, voo0, a0) = contador(&mut t);
        traco(&mut t, *tr, N, true);
        let (f1, voo1, _) = contador(&mut t);
        g.tracos.push((f0, voo0, a0, f1, voo1));
        espera(&mut t, ESPERAS_MS[k]);
    }
    let (_, _, a) = contador(&mut t);
    let px = tela(&mut t);
    g.fim = contador(&mut t).0;
    g.fim_atomico = a;
    (px, g, t)
}

/// (c) a reprodução no relógio FIXO: entre traços, EXATAMENTE os passos gravados.
fn reproducao(g: &Gravacao) -> (Vec<u8>, PainterTool) {
    let mut t = ferramenta(N, true);
    for (k, tr) in ROTEIRO.iter().enumerate() {
        passos_ate(&mut t, g.tracos[k].0);
        traco(&mut t, *tr, N, false);
        t.paint_tick(1.0 / 60.0);
    }
    passos_ate(&mut t, g.fim);
    (tela(&mut t), t)
}

/// +40 passos nos dois lados e compara outra vez (a água escondida que a tela ainda não mostra).
fn depois_de_40(a: &mut PainterTool, b: &mut PainterTool) -> usize {
    desliga_o_relogio(a);
    desliga_o_relogio(b);
    let (fa, fb) = (contador(a).0 + 40, contador(b).0 + 40);
    passos_ate(a, fa);
    passos_ate(b, fb);
    let (pa, pb) = (tela(a), tela(b));
    texels_diferentes(&pa, &pb)
}

fn resumo(v: &mut [f32]) -> (f32, f32) {
    v.sort_by(f32::total_cmp);
    (v[0], v[v.len() / 2])
}

/// (a) o CUSTO: um passo fixo a 1024²/2048²/4096² (o passo cru e o `wet_step_sync(1)` com o composite),
/// e o quadro de REPRODUZIR um traço a 1024². Variantes intercaladas, 7 rodadas rodadas, MÍNIMO e mediana.
#[test]
#[ignore = "medicao — rode com --profile smoke --ignored --nocapture --test-threads=1"]
fn sonda_macro_a_custo() {
    println!("[macro a] load antes {}", carga());
    let tamanhos = [1024u32, 2048, 4096];
    let mut ts: Vec<PainterTool> = tamanhos
        .iter()
        .map(|&n| {
            let mut t = ferramenta(n, true);
            for tr in SPLASH {
                traco(&mut t, tr, n, false);
                t.paint_tick(1.0 / 60.0);
            }
            t.wet_step_sync(2);
            t
        })
        .collect();
    let mut repro = ferramenta(N, true);
    traco(&mut repro, SPLASH[0], N, false);
    let mut cru: Vec<Vec<f32>> = vec![Vec::new(); 3];
    let mut sync: Vec<Vec<f32>> = vec![Vec::new(); 3];
    let (mut traco_ms, mut pior_q, mut ev_ms) = (Vec::new(), Vec::new(), Vec::new());
    for r in 0..7usize {
        for i in 0..4usize {
            let k = (r + i) % 4;
            if k == 3 {
                let y = 100.0 + 120.0 * r as f32;
                let tr = Traco { de: [100.0, y], ate: [900.0, y + 30.0], cor: [0.5, 0.1, 0.6] };
                let a = Instant::now();
                let pq = traco(&mut repro, tr, N, false);
                repro.paint_tick(1.0 / 60.0);
                let ms = a.elapsed().as_secs_f32() * 1e3;
                traco_ms.push(ms);
                pior_q.push(pq);
                ev_ms.push(ms / (AMOSTRAS + 2) as f32);
                continue;
            }
            let t = &mut ts[k];
            for _ in 0..3 {
                t.wet_bring_home();
                let s = t.paint.wetpaint.session.as_mut().expect("sessao");
                let a = Instant::now();
                while s.engine.step_stage().is_none() {}
                cru[k].push(a.elapsed().as_secs_f32() * 1e3);
                let a = Instant::now();
                t.wet_step_sync(1);
                sync[k].push(a.elapsed().as_secs_f32() * 1e3);
            }
        }
    }
    for (k, n) in tamanhos.iter().enumerate() {
        let (cm, cd) = resumo(&mut cru[k]);
        let (sm, sd) = resumo(&mut sync[k]);
        println!(
            "[macro a] {n}²: passo cru min {cm:.2} med {cd:.2} ms · wet_step_sync(1) (passo+composite) min {sm:.2} med {sd:.2} ms"
        );
    }
    let (tm, td) = resumo(&mut traco_ms);
    let (qm, qd) = resumo(&mut pior_q);
    let (em, ed) = resumo(&mut ev_ms);
    println!(
        "[macro a] reproduzir um traço de {AMOSTRAS} amostras a 1024² (20 quadros de 3 eventos + tique, Down/Up): \
         total min {tm:.1} med {td:.1} ms · pior quadro min {qm:.2} med {qd:.2} ms · por evento min {em:.3} med {ed:.3} ms"
    );
    println!("[macro a] load depois {}", carga());
}

/// (b)(c)(d) o fenómeno, a hipótese e o controlo — numa rodada só.
#[test]
#[ignore = "medicao — rode com --profile smoke --ignored --nocapture --test-threads=1"]
fn sonda_macro_bcd_reproducao() {
    println!("[macro bcd] load antes {}", carga());
    for drena_no_down in [false, true] {
        let nome = if drena_no_down { "DRENA no Down" } else { "produto (sem drenar)" };
        let mut corridas = Vec::new();
        for _ in 0..4 {
            corridas.push(corrida_worker(drena_no_down));
        }
        // (b) worker × worker
        let d01 = texels_diferentes(&corridas[0].0, &corridas[1].0);
        let d23 = texels_diferentes(&corridas[2].0, &corridas[3].0);
        println!("[macro b] {nome}: worker×worker texels diferentes: c0×c1 {d01} · c2×c3 {d23}");
        for (i, (px, g, tw)) in corridas.iter_mut().enumerate() {
            println!(
                "[macro b] {nome} c{i}: traços (frame Down, em voo, sim_steps Down, frame Up, em voo Up) {:?} · fim frame {} sim_steps {}",
                g.tracos, g.fim, g.fim_atomico
            );
            // (c) reprodução pelos passos gravados
            let (rp, mut tr) = reproducao(g);
            let d = texels_diferentes(px, &rp);
            let d40 = depois_de_40(tw, &mut tr);
            println!("[macro c] {nome} c{i}: reprodução × worker: {d} texels · +40 passos: {d40}");
        }
        // (d) controlo: duas reproduções dos mesmos números
        let g = corridas[0].1.clone();
        let (r1, mut t1) = reproducao(&g);
        let (r2, mut t2) = reproducao(&g);
        let d = texels_diferentes(&r1, &r2);
        let d40 = depois_de_40(&mut t1, &mut t2);
        // e a régua VÊ: a reprodução com um passo a menos antes do traço 2
        let mut g1 = g.clone();
        g1.tracos[1].0 -= 1;
        let (r3, _) = reproducao(&g1);
        let dm = texels_diferentes(&r1, &r3);
        println!(
            "[macro d] {nome}: reprodução×reprodução {d} texels (+40: {d40}) · controlo positivo (1 passo a menos antes do traço 2): {dm}"
        );
    }
    println!("[macro bcd] load depois {}", carga());
}
