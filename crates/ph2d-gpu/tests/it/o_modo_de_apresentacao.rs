//! B21.1 — G10: o modo do arranque é sempre um que o backend TEM, Mailbox (+ latência 2) quando
//! existe e o refresco é conhecido, Fifo de resto; nunca latência 1.

use ph2d_gpu::surface::{LATENCIA, modo_de_boot, nao_bloqueia};
use wgpu::PresentMode::{AutoNoVsync, AutoVsync, Fifo, FifoRelaxed, Immediate, Mailbox};

#[test]
fn o_modo_de_apresentacao_e_valido() {
    let conjuntos: [&[wgpu::PresentMode]; 6] = [
        &[Fifo],
        &[Fifo, Immediate],
        &[Fifo, Mailbox, Immediate],
        &[Immediate, Mailbox, Fifo, FifoRelaxed],
        &[Mailbox],
        &[FifoRelaxed],
    ];
    for d in conjuntos {
        for refresco in [true, false] {
            for pedido in [None, Some(Fifo), Some(Mailbox), Some(Immediate)] {
                let m = modo_de_boot(d, refresco, pedido);
                assert!(d.contains(&m), "{m:?} ∉ {d:?}");
                if pedido.is_none() {
                    let quer = if refresco && d.contains(&Mailbox) {
                        Mailbox
                    } else if d.contains(&Fifo) {
                        Fifo
                    } else {
                        d[0]
                    };
                    assert_eq!(m, quer, "{d:?} refresco={refresco}");
                }
            }
        }
    }
    assert_eq!(
        LATENCIA, 2,
        "Mailbox+1 no Vulkan = duas imagens: 4K a 30 q/s"
    );
    assert!(
        [Mailbox, Immediate, AutoNoVsync]
            .into_iter()
            .all(nao_bloqueia)
    );
    assert!(![Fifo, FifoRelaxed, AutoVsync].into_iter().any(nao_bloqueia));
}
