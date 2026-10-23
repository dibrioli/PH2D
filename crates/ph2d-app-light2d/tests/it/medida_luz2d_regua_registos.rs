//! **A recolha da régua de registos** (handoff W6h §5.1 b; a régua é `spikes/regua-registos`): por variante da
//! [`VARIANTES`] do custo dos cantos, o módulo das cascatas e as constantes da fina tal e qual o `pipeline_em`
//! da C3 as monta (as próprias vencem as comuns, + `DISTANTES` 0). À mão:
//!
//! ```text
//! PH2D_GPU=1 PH2D_REGUA_REGISTOS=<pasta> bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d \
//!     --profile smoke --test it medida_luz2d_regua_registos -- --ignored --nocapture
//! spikes/regua-registos/target/release/regua-registos <pasta>
//! ```
//!
//! A placa só decide o `padrao_para` (f16); nada corre nela.

use ph2d_light2d_gpu::{constantes_comuns_da_c3, constantes_da_fina, fonte_do_rc};

use crate::comum::gpu;
use crate::medida_luz2d_cantos_custo::{VARIANTES, variante};
use crate::medida_luz2d_w5d::vista;

#[test]
#[ignore = "a recolha da régua de registos: corre-se à mão (cabeçalho)"]
fn medida_luz2d_regua_registos() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    let pasta = std::path::PathBuf::from(
        std::env::var("PH2D_REGUA_REGISTOS").expect("PH2D_REGUA_REGISTOS=<pasta>"),
    );
    std::fs::create_dir_all(&pasta).expect("a pasta");
    let base = crate::as_formas_redondas::config(&gpu, vista(1.0), false);
    for (v, nome) in VARIANTES.iter().enumerate().take(VARIANTES.len() - 1) {
        let cfg = variante(base, v);
        let fonte = cfg.modulo_rc.unwrap_or_else(|| {
            fonte_do_rc(
                cfg.aritmetica,
                cfg.armazenamento,
                cfg.hrc_niveis_em_textura(),
            )
        });
        let (entrada, proprias) = constantes_da_fina(&cfg);
        let comuns = constantes_comuns_da_c3(&cfg);
        let mut linhas = vec![format!("modulo v{v}.wgsl"), format!("entrada {entrada}")];
        linhas.extend(proprias.iter().map(|(k, x)| format!("{k} {x}")));
        linhas.extend(
            comuns
                .iter()
                .filter(|(k, _)| proprias.iter().all(|(o, _)| o != k))
                .map(|(k, x)| format!("{k} {x}")),
        );
        std::fs::write(pasta.join(format!("v{v}.wgsl")), fonte).expect("o módulo");
        // `v<v>.txt` com `DISTANTES 0` (o de sempre) e `v<v>_d1.txt` com 1 (os degraus da W6h §6).
        for d in [0, 1] {
            let mut l = linhas.clone();
            l.push(format!("DISTANTES {d}"));
            let nome = if d == 0 {
                format!("v{v}.txt")
            } else {
                format!("v{v}_d1.txt")
            };
            std::fs::write(pasta.join(nome), l.join("\n") + "\n").expect("as constantes");
        }
        eprintln!(
            "REGUA · v{v} = {nome} · {entrada} · {} constantes",
            linhas.len() - 2
        );
    }
}
