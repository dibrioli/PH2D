//! **SONDA (W5e, ignorada): onde vai o tempo de uma macro do Painter** — a cena a 60 Hz (o `segue` e a
//! leitura da tela, por quadro) e a reprodução do zero ao fim (a conferência `assenta` e a reprodução
//! para editar), partida por passo, com a área que cada pré-visualização sujou. Splash sem papel (a
//! cena de hoje), no papel (a porta do produto), e uma macro longa. Mínimo de rondas intercaladas.
//!
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib -- --ignored
//! --nocapture --test-threads=1 custo_w5e`

use super::em_cena::MacroEmCena;
use super::passo::Passo;
use super::roteiro::{grava_no_papel, grava_o_splash};
use super::tocar::TocadorDoPainter;
use std::time::Instant;

const RONDAS: usize = 3;

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg").unwrap_or_default()
}

struct Doc {
    nome: String,
    bytes: Vec<u8>,
    lado: u32,
    segundos: f64,
    /// A cena: (Σ segue, Σ tela, quadros, quadros com tela nova) no mínimo das rondas.
    cena: (f64, f64, usize, usize),
    /// Do zero ao fim: total, dentro de `Preview*`, `Tique`, `Ponteiro`, resto (ms).
    zero: [f64; 5],
    previews: usize,
    /// Média da área sujada por pré-visualização (fracção da tela; `None` conta 1).
    area: f64,
    /// ms acumulados até ao toque de cada pincelada (a conferência a partir do instantâneo k custa
    /// ~ total − isto).
    ate_toque: Vec<f64>,
}

fn cena(d: &Doc) -> (f64, f64, usize, usize) {
    let mut m = MacroEmCena::de(&d.bytes).unwrap().unwrap();
    let (mut seg, mut tela, mut q, mut novas) = (0.0, 0.0, 0, 0);
    let mut t = 0.0;
    while t <= d.segundos + 1.0 / 60.0 {
        let a = Instant::now();
        m.segue(t).unwrap();
        seg += a.elapsed().as_secs_f64() * 1e3;
        let a = Instant::now();
        if m.toma_a_tela().is_some() {
            novas += 1;
        }
        tela += a.elapsed().as_secs_f64() * 1e3;
        q += 1;
        t += 1.0 / 60.0;
    }
    (seg, tela, q, novas)
}

fn do_zero(d: &Doc) -> ([f64; 5], usize, f64, Vec<f64>) {
    let doc = ph2d_macro::codec::read(&d.bytes).unwrap();
    let tr = &doc.tracks[0];
    let a0 = Instant::now();
    let mut toc =
        TocadorDoPainter::novo(&tr.initial, tr.events.iter().map(|e| e.payload.as_slice()))
            .unwrap();
    let mut ms = [0.0; 5];
    let (mut previews, mut area) = (0usize, 0.0f64);
    let tela = f64::from(d.lado) * f64::from(d.lado);
    let mut ate_toque = Vec::new();
    for e in &tr.events {
        let p = Passo::descodifica(&e.payload).unwrap();
        let k = match p {
            Passo::Preview | Passo::PreviewSujo => 1,
            Passo::Tique(_) => 2,
            Passo::Ponteiro { fase, .. } => {
                if fase == ph2d_editor_core::tool::PointerPhase::Down {
                    ate_toque.push(a0.elapsed().as_secs_f64() * 1e3);
                }
                3
            }
            _ => 4,
        };
        let a = Instant::now();
        toc.aplica(&e.payload).unwrap();
        ms[k] += a.elapsed().as_secs_f64() * 1e3;
        if k == 1 {
            previews += 1;
            area += toc
                .tool_mut()
                .take_preview_upload_bbox()
                .map_or(1.0, |(_, _, w, h)| f64::from(w) * f64::from(h) / tela);
        }
    }
    ms[0] = a0.elapsed().as_secs_f64() * 1e3;
    #[expect(clippy::cast_precision_loss, reason = "contagem pequena")]
    let media = area / previews.max(1) as f64;
    (ms, previews, media, ate_toque)
}

#[test]
#[ignore = "sonda de medição (W5e): imprime, não reprova"]
fn sonda_custo_w5e() {
    eprintln!("[w5e] load antes: {}", carga().trim());
    let mut docs = Vec::new();
    let mut push = |nome: &str, lado: u32, r: Option<(Vec<u8>, f64)>| {
        let (bytes, segundos) = r.expect("arma no Wet Paint");
        docs.push(Doc {
            nome: nome.into(),
            bytes,
            lado,
            segundos,
            cena: (f64::MAX, 0.0, 0, 0),
            zero: [f64::MAX; 5],
            previews: 0,
            area: 0.0,
            ate_toque: Vec::new(),
        });
    };
    push("splash sem papel 1024²", 1024, grava_o_splash(1024, 0));
    push("splash no papel 1024²", 1024, grava_no_papel(1024, 0, 4));
    push("splash no papel 2048²", 2048, grava_no_papel(2048, 0, 4));
    push(
        "longa no papel 1024² (96)",
        1024,
        grava_no_papel(1024, 0, 96),
    );
    push(
        "longa no papel 2048² (32)",
        2048,
        grava_no_papel(2048, 0, 32),
    );
    let n = docs.len();
    for r in 0..RONDAS {
        for j in 0..n {
            let d = &mut docs[(j + r) % n];
            let c = cena(d);
            if c.0 + c.1 < d.cena.0 + d.cena.1 {
                d.cena = c;
            }
            let (ms, previews, area, ate) = do_zero(d);
            if ms[0] < d.zero[0] {
                d.zero = ms;
                d.ate_toque = ate;
            }
            d.previews = previews;
            d.area = area;
        }
    }
    eprintln!("[w5e] load depois: {}", carga().trim());
    for d in &docs {
        let (seg, tela, q, novas) = d.cena;
        #[expect(clippy::cast_precision_loss, reason = "contagem pequena")]
        let (qf, nf) = (q as f64, novas.max(1) as f64);
        eprintln!(
            "[w5e] {}: macro {:.2} s · CENA {q} quadros ({novas} com tela nova): segue {:.2} ms/quadro, \
             tela {:.2} ms/tela nova · DO ZERO {:.0} ms ({:.0} ms por s de macro): Preview {:.0} ({} passos, \
             área média {:.1} %), Tique {:.0}, Ponteiro {:.0}, resto {:.0}",
            d.nome,
            d.segundos,
            seg / qf,
            tela / nf,
            d.zero[0],
            d.zero[0] / d.segundos,
            d.zero[1],
            d.previews,
            d.area * 100.0,
            d.zero[2],
            d.zero[3],
            d.zero[4],
        );
        if d.segundos > 10.0 {
            conferencias(d);
        }
        let k = d.ate_toque.len();
        if k > 4 {
            eprintln!(
                "[w5e]   até ao toque da pincelada 1/4: {:.0} ms · 1/2: {:.0} ms · 3/4: {:.0} ms · última: {:.0} ms",
                d.ate_toque[k / 4],
                d.ate_toque[k / 2],
                d.ate_toque[3 * k / 4],
                d.ate_toque[k - 1]
            );
        }
    }
}

/// A conferência de uma cor editada na pincelada do meio e na última: do zero contra do instantâneo do
/// palco que tocou até ao fim dela (o clique na etapa leva a cabeça ao fim dela). Mínimo de 2 rondas.
fn conferencias(d: &Doc) {
    use super::etapas::{self, confere, edita_cor};
    use std::sync::atomic::AtomicBool;
    let doc = ph2d_macro::codec::read(&d.bytes).unwrap();
    let es = etapas::etapas(&doc.tracks[0]).unwrap();
    let nunca = AtomicBool::new(false);
    for k in [es.len() / 2, es.len() - 1] {
        let mut ed = doc.clone();
        // LITERAL-COLOR-OK: a cor nova de uma pincelada GRAVADA, não da UI.
        edita_cor(&mut ed.tracks[0], k, [20, 200, 40]).unwrap();
        let fim = f64::from(u32::try_from(es[k].t1.0).unwrap()) / 1e6;
        let (mut zero, mut copia, mut parte) = (f64::MAX, f64::MAX, f64::MAX);
        for _ in 0..2 {
            let a = Instant::now();
            confere(&ed.tracks[0], None, &nunca).unwrap();
            zero = zero.min(a.elapsed().as_secs_f64() * 1e3);
            let mut palco = MacroEmCena::de(&ph2d_macro::codec::write(&ed))
                .unwrap()
                .unwrap();
            palco.segue(fim).unwrap();
            let a = Instant::now();
            let p = palco.partida(&ed.tracks[0]).expect("instantâneo");
            copia = copia.min(a.elapsed().as_secs_f64() * 1e3);
            let a = Instant::now();
            confere(&ed.tracks[0], Some(p), &nunca).unwrap();
            parte = parte.min(a.elapsed().as_secs_f64() * 1e3);
        }
        eprintln!(
            "[w5e]   conferir a cor da pincelada {}/{}: do zero {zero:.0} ms · do instantâneo {parte:.0} ms (+ {copia:.1} ms a tirá-lo, no quadro)",
            k + 1,
            es.len()
        );
    }
}
