//! **O custo ÚNICO que só o PRIMEIRO traço paga** (doc 28 §4.8).
//!
//! O `PainterGpuPreview` da shell é criado **lazily** (`get_or_insert_with`), ou seja no primeiro frame
//! que precisa do preview GPU — que é o primeiro traço do artista. Ele constrói três coisas, e cada uma
//! **COMPILA shaders**: o `LayerCompositor`, o `ImpastoLightPass` e o `PreviewPremul`.
//!
//! Compilação de pipeline é o clássico *hitch de primeiro frame*, e é invisível para toda sonda do
//! `ph2d-tool-painter` — elas medem o tool, e o tool não tem GPU. Esta mede o que falta.
//!
//! `#[ignore]`: precisa de um adapter real.

use ph2d_gpu::GpuContext;
use ph2d_render::{ImpastoLightPass, LayerCompositor, PreviewPremul};

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn ms(f: &mut dyn FnMut()) -> f64 {
    let t = std::time::Instant::now();
    f();
    t.elapsed().as_secs_f64() * 1000.0
}

#[test]
#[ignore = "perf measurement — needs a real adapter; run with --release --ignored"]
fn what_the_first_stroke_pays_to_build_the_gpu_preview() {
    let Some(gpu) = try_headless_gpu() else {
        println!("[first-gpu] sem adapter — pulado");
        return;
    };
    // ⚠️ Uma criação a MAIS antes de medir, para separar o custo da PEÇA do custo de acordar o driver
    // (a primeira chamada a qualquer coisa de wgpu paga inicialização que não é desta peça). Sem esse
    // controle a 1ª linha da tabela leva a culpa das outras duas.
    let warm = ms(&mut || {
        let _ = LayerCompositor::new(&gpu);
    });
    let comp = ms(&mut || {
        let _ = LayerCompositor::new(&gpu);
    });
    let light = ms(&mut || {
        let _ = ImpastoLightPass::new(&gpu);
    });
    let premul = ms(&mut || {
        let _ = PreviewPremul::new(&gpu);
    });
    let total = comp + light + premul;
    println!("[first-gpu] (aquecimento do driver, descartado)  {warm:>8.2} ms");
    println!("[first-gpu] LayerCompositor::new                 {comp:>8.2} ms");
    println!("[first-gpu] ImpastoLightPass::new                {light:>8.2} ms");
    println!("[first-gpu] PreviewPremul::new                   {premul:>8.2} ms");
    println!("[first-gpu] TOTAL pago no 1o traco               {total:>8.2} ms");
}
