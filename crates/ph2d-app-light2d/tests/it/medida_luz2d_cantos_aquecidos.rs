//! ⭐ Item 3a (linha Luz2D, 24/10) — **a anomalia da r5b** (handoff W6g §12): o passe dos cantos da r5c
//! (`Canto::passe_proprio`) custa menos depois de um passe que aquece a MESMA memória? O fantasma é o
//! próprio passe dos cantos corrido antes (mesmo tamanho, mesmas leituras, a escrita idempotente). Precisa
//! do patch `docs/Luz2D/ferramentas/item3_cantos_vezes.patch` (o `PH2D_ITEM3_CANTOS` = quantas vezes o
//! `FinaCantos::depois` despacha o passe dos cantos, um número por `FinaCantos` criada, em ciclo — a ordem
//! das variantes com passe próprio em [`VARIANTES`]; 0 = sem ele nem a junta), FORA do produto. À mão:
//!
//! ```text
//! git apply docs/Luz2D/ferramentas/item3_cantos_vezes.patch
//! PH2D_GPU=1 PH2D_ITEM3_CANTOS=0,1,2,3,1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_cantos_aquecidos -- --ignored --nocapture --test-threads=1
//! git apply -R docs/Luz2D/ferramentas/item3_cantos_vezes.patch
//! ```
//!
//! MEDIDO (24/10, RTX 5060 Ti, `smoke`, `load` ~6, c3_fina mín de 9): cena 5 — sem os cantos 1,925 · 1× 2,495 ·
//! 2× 3,069 · 3× 3,588 (A × A −0,007) ⇒ cada passe +0,57 / +0,57 / +0,52 ms; cena 6 — 0,912 · 1,448 · 1,959 · 2,517
//! (A × A 0,000) ⇒ +0,54 / +0,51 / +0,56. O passe dos cantos depois de um fantasma IGUAL (a mesma memória, quente)
//! custa o mesmo: a «cache aquecida» não explica a r5b (+0,10 / +0,005 depois da pré-lista), nem o relógio da placa
//! (a fina inteira corre antes). A pré-lista nunca esteve em commit (`git log --all -S` por `pre_lista`,
//! `prelista`, `pré-lista` no código: só o texto do handoff) ⇒ a r5b não se reproduz sem a reconstruir, e o caminho
//! que a usaria (sobrepor cantos e fina) está recusado (W6g §12).

use ph2d_app_light2d::smoke;
use ph2d_light2d_gpu::{Canto, Config};

use crate::as_formas_redondas::config;
use crate::comum::gpu;
use crate::medida_luz2d_w4b::loadavg;
use crate::medida_luz2d_w5d::{solver, vista};

const RODADAS: usize = 9;

/// (nome, passe próprio, vezes do passe dos cantos).
const VARIANTES: [(&str, bool, u32); 6] = [
    ("em linha (padrão)", false, 1),
    ("r5c, sem os cantos (0×)", true, 0),
    ("r5c, os cantos 1×", true, 1),
    ("r5c, os cantos 2× (o 1.º é o fantasma)", true, 2),
    ("r5c, os cantos 3×", true, 3),
    ("r5c, os cantos 1× (controlo A × A)", true, 1),
];

#[test]
#[ignore = "item 3a: o passe dos cantos depois de um fantasma (precisa do patch, cabeçalho)"]
fn medida_luz2d_cantos_aquecidos() {
    let Some(gpu) = gpu() else { return };
    eprintln!(
        "# cantos aquecidos · {} · load {}",
        gpu.adapter.get_info().name,
        loadavg().trim()
    );
    for n in [5, 6] {
        let mut w = bevy_ecs::world::World::new();
        assert!(smoke::encenar(&mut w, n));
        let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
        let a = vista(1.0);
        let base = crate::medida_luz2d_cantos_custo::variante(config(&gpu, a, false), 4);
        let mut s: Vec<_> = VARIANTES
            .iter()
            .map(|&(_, proprio, _)| {
                let cfg = Config {
                    canto: Canto {
                        passe_proprio: proprio,
                        ..base.canto
                    },
                    ..base
                };
                assert_eq!(cfg.adia_o_canto(), proprio, "o passe próprio não liga nesta vista");
                solver(&gpu, a, cfg)
            })
            .collect();
        for x in &mut s {
            for _ in 0..3 {
                x.quadro(&gpu, &cena);
            }
            let _ = x.tempos(&gpu);
        }
        let mut minimo = [f64::INFINITY; VARIANTES.len()];
        for r in 0..RODADAS {
            for o in 0..s.len() {
                let k = (o + r) % s.len();
                for _ in 0..3 {
                    s[k].quadro(&gpu, &cena);
                }
                minimo[k] = minimo[k].min(s[k].tempos(&gpu).expect("TIMESTAMP_QUERY").c3_fina);
            }
        }
        eprintln!("\n## cena {n} · vista de fábrica · c3_fina (mín {RODADAS}) ms");
        for (k, (nome, _, _)) in VARIANTES.iter().enumerate() {
            eprintln!("  {nome:<40} {:.3}", minimo[k]);
        }
        eprintln!(
            "  ⇒ o 1.º passe dos cantos {:+.3} · o 2.º {:+.3} · o 3.º {:+.3} · A × A {:+.3}",
            minimo[2] - minimo[1],
            minimo[3] - minimo[2],
            minimo[4] - minimo[3],
            minimo[5] - minimo[2]
        );
    }
    eprintln!("# load depois {}", loadavg().trim());
}
