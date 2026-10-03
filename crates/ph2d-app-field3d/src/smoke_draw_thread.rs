//! ⭐⭐⭐ **O QUE CORRE NA THREAD DE TRAÇADO** — do pedido à imagem do MATCAP.
//!
//! ⚠️ **Ele saiu do [`crate::smoke_draw`] em 2026-09-15 por TETO DE LINHAS**, e a fronteira é a que
//! o módulo já declarava por escrito: *o estado do módulo não atravessa a fronteira*. O que arma o
//! pedido fica lá; o que o **responde**, sem tocar no `Smoke`, fica aqui.
//!
//! ⛔⛔ **É por isso que o [`Pedido`] é uma struct e não uma lista de argumentos:** cada campo dele
//! foi COPIADO do módulo antes de a thread nascer. Passá-los soltos deixaria a próxima adição livre
//! de ler o módulo em vez de o copiar.
//!
//! ⭐ **Só o Matcap é traçado** desde 03/10 (ordem do dono: *«pode apagar o render antigo»*): o modo
//! Render desenha por malha ([`crate::malha_render_quadro`]) e, sem aparelho, mostra este Matcap.

use super::{BACKGROUND, Ready};
use ph2d_field::FieldDoc;
use ph2d_field_render::Matcap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::SyncSender;

/// ⭐⭐⭐ **O pedido de um traçado**, copiado do módulo antes de a thread nascer.
pub(crate) struct Pedido {
    pub doc: FieldDoc,
    pub reg: ph2d_field_eval::hybrid::Registry,
    pub cam: ph2d_field_render::Orbit,
    pub tw: u32,
    pub th: u32,
    /// O tamanho da ÁREA: a placa amplia o quadro de movimento até ele.
    pub cheio: (u32, u32),
    pub assente: bool,
    pub look: ph2d_view_transform::Look,
    pub matcap: Arc<super::MatcapTexels>,
    /// As fitas já compiladas — um `Arc`: o que viaja é o ponteiro.
    pub tapes: Arc<ph2d_field_render::TapeCache>,
    /// `PH2D_FIELD_TAPE_CACHE=0` traça sem cache (a porta de bissecção).
    pub usa_cache: bool,
    pub flag: Arc<AtomicBool>,
    pub tx: SyncSender<Ready>,
    /// O traçador do dispositivo — ver [`crate::gpu_frame::para_o_quadro`].
    pub gpu: Option<&'static crate::gpu_frame::SharedTracer>,
}

/// ⭐⭐⭐⭐ **O tamanho da imagem que a placa DEVOLVEU** — a ÁREA cheia quando ela a ampliou
/// ([`Pedido::cheio`], `ph2d_field_gpu::amplia`), senão o tamanho traçado. ⚠️ A MEDIÇÃO do divisor
/// continua a ser a dos píxeis TRAÇADOS (`Ready::tracado_px`).
fn tamanho_entregue(p: &Pedido, bytes: usize) -> (u32, u32) {
    if bytes as u64 == u64::from(p.cheio.0) * u64::from(p.cheio.1) * 4 {
        p.cheio
    } else {
        (p.tw, p.th)
    }
}

/// ⭐⭐⭐ **A resposta a um pedido** — corre fora da thread que desenha.
///
/// ⭐ **O Matcap pinta NO DISPOSITIVO quando pode** (report do dono, 23/09: *«ao arrastar fica
/// grosseiro ainda»*): a imagem volta pelo barramento, não o G-buffer. ⚠️ *«A placa sabe MARCHAR esta
/// peça?»* é GEOMETRIA e vive numa porta ([`crate::gpu_frame::takes_the_frame`]): uma peça com
/// ESCULTURA fica na CPU, senão desapareceria em silêncio.
pub(crate) fn traca(p: &Pedido) {
    let t0 = std::time::Instant::now();
    if crate::gpu_frame::takes_the_frame(p.gpu, &p.doc, &p.reg)
        && let Some(t) = p.gpu.as_ref()
        && let Some(pintura) = crate::gpu_frame::pinta_matcap(
            t,
            &p.doc,
            &p.reg,
            &p.cam,
            &ph2d_field_gpu::matcap::MatcapSetup {
                rgb_linear: &p.matcap.rgb,
                side: p.matcap.side,
                chave: p.matcap.chave,
                stops: p.look.exposure_stops,
                view: ph2d_view_transform::wgsl::view_code(p.look.view),
                background: BACKGROUND,
                entrega: Some(p.cheio),
            },
            p.tw,
            p.th,
        )
    {
        // ⚠️ **Os acertos contam-se na IMAGEM**: o G-buffer ficou no dispositivo, e o alfa do fundo
        // é o discriminador que já vive ali.
        let hits = pintura
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|px| px[3] > BACKGROUND[3])
            .count();
        let (iw, ih) = tamanho_entregue(p, pintura.rgba.len());
        let _ = p.tx.try_send(Ready {
            rgba: pintura.rgba,
            width: iw,
            height: ih,
            tracado_px: u64::from(p.tw) * u64::from(p.th),
            hits,
            edges: pintura.edges,
            // ⏱️ **Sem a COMPILAÇÃO** — ver [`ph2d_field_gpu::FieldPipelines::compilado_ms`]: este
            // número decide o tamanho do quadro seguinte, que não a paga.
            millis: (t0.elapsed().as_secs_f64() * 1000.0 - pintura.compilado_ms).max(0.0),
            assente: p.assente,
            pela_placa: true,
        });
        return;
    }
    // Abandonado a meio: não se manda nada, e quem esperava já mudou de pedido.
    let Some(g) = ph2d_field_render::trace_cancellable(
        &p.doc,
        &p.reg,
        &p.cam,
        p.tw,
        p.th,
        &p.flag,
        // ⭐ O motor de REFERÊNCIA re-amostra a silhueta pela mesma lei da placa.
        crate::preview::re_amostra_a_silhueta(),
        p.usa_cache.then_some(&*p.tapes),
    ) else {
        return;
    };
    let rgba = ph2d_field_render::shade_with(
        &g,
        &Matcap {
            side: p.matcap.side,
            rgb_linear: &p.matcap.rgb,
        },
        p.look,
        BACKGROUND,
    );
    let _ = p.tx.try_send(Ready {
        rgba,
        width: p.tw,
        height: p.th,
        tracado_px: u64::from(p.tw) * u64::from(p.th),
        hits: g.hits(),
        edges: g.edges.len(),
        millis: t0.elapsed().as_secs_f64() * 1000.0,
        assente: p.assente,
        pela_placa: false,
    });
}
