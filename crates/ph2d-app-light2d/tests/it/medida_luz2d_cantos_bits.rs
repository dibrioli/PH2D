//! **A prova de mesmo bit das curas do custo dos cantos** (handoff W6g §21–§27: os 32 casos do script de
//! diagnóstico, que vivia fora do git): a fluência em bruto das cenas 1–8 × miolo com/sem × {lei W6d, padrão}, na vista
//! de fábrica, sem rebote, 3 quadros. Grava `<PH2D_CANTOS_BITS>/<caso>.bin`; com `PH2D_CANTOS_BITS_BASE`
//! compara cada caso com o da base e conta os bytes diferentes. À mão:
//!
//! ```text
//! PH2D_GPU=1 PH2D_CANTOS_BITS=target/cantos_bits/base bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d \
//!     --profile smoke --test it medida_luz2d_cantos_bits -- --ignored --nocapture --test-threads=1
//! PH2D_GPU=1 PH2D_CANTOS_BITS=target/cantos_bits/cura PH2D_CANTOS_BITS_BASE=target/cantos_bits/base …
//! ```

use ph2d_app_light2d::smoke;
use ph2d_light2d_gpu::{Canto, Config, VERTICE_PADRAO};

use crate::as_formas_redondas::config;
use crate::comum::{gpu, ler};
use crate::medida_luz2d_w5d::{solver, vista};

const QUADROS: usize = 3;

#[test]
#[ignore = "a prova de mesmo bit: corre-se à mão (cabeçalho)"]
fn medida_luz2d_cantos_bits() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    let dir = std::path::PathBuf::from(
        std::env::var("PH2D_CANTOS_BITS").expect("PH2D_CANTOS_BITS=<pasta>"),
    );
    std::fs::create_dir_all(&dir).expect("a pasta");
    let base = std::env::var("PH2D_CANTOS_BITS_BASE")
        .ok()
        .map(std::path::PathBuf::from);
    let a = vista(1.0);
    let c = config(&gpu, a, false);
    let leis = [
        (
            "lei_w6d",
            Config {
                borda_vertice: None,
                ..c.sem_os_cantos()
            },
        ),
        (
            "padrao",
            Config {
                borda_vertice: Some(VERTICE_PADRAO),
                borda_cantos: true,
                canto: Canto::LIGADO,
                ..c
            },
        ),
    ];
    let (mut casos, mut diferentes) = (0, 0);
    for n in 1..=8u32 {
        let mut w = bevy_ecs::world::World::new();
        assert!(smoke::encenar(&mut w, n));
        let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
        for (lei, cfg) in leis {
            for (miolo, m) in [("miolo", cfg.miolo), ("sem_miolo", None)] {
                let mut s = solver(&gpu, a, Config { miolo: m, ..cfg });
                for _ in 0..QUADROS {
                    s.quadro(&gpu, &cena);
                }
                let bits = ler(&gpu, s.fluencia(), 8);
                let caso = format!("c{n}_{lei}_{miolo}");
                std::fs::write(dir.join(format!("{caso}.bin")), &bits).expect("gravar");
                casos += 1;
                if let Some(b) = &base {
                    let velho = std::fs::read(b.join(format!("{caso}.bin"))).expect("a base");
                    let d = if velho.len() == bits.len() {
                        velho.iter().zip(&bits).filter(|(x, y)| x != y).count()
                    } else {
                        bits.len().max(velho.len())
                    };
                    diferentes += usize::from(d > 0);
                    eprintln!("BITS · {caso} · {} bytes · diferentes {d}", bits.len());
                }
            }
        }
    }
    eprintln!("BITS · {casos} casos · {diferentes} com bytes diferentes");
    if base.is_some() {
        assert_eq!(diferentes, 0, "a cura muda a luz");
    }
}
