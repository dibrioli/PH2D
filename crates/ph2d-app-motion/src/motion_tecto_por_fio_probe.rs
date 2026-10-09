//! ⭐ **A SONDA DO FIO ACIMA DO DIGITÁVEL** (doc 125 §3): um `value.*` ligado a um param entrega
//! ao `eval` o valor CRU (`fio > override > default`, sem o tecto digitável). Cada param que
//! COMANDA a contagem de um criador (dobrá-lo sobe a contagem), sozinho a `1e9`, cronometrado, em
//! debug como o censo. A linha `comeca` sem a do resultado aponta quem não acaba.
//!
//! `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture sonda_cada_contagem_a_1e9_por_fio`

use super::{LINHAS_DE_ENTRADA, VALOR_DA_SONDA, conta_no_instante};

#[test]
#[ignore = "sonda: o fio acima do digitavel — corra a mao com --nocapture"]
fn sonda_cada_contagem_a_1e9_por_fio() {
    const T: f64 = 10.0;
    let m = crate::motion_state::MotionState::new();
    let reg = &m.registry;
    let anterior = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    for man in reg.manifests() {
        let nome = man.name;
        let params: Vec<String> = man.params.iter().map(|p| p.name.to_string()).collect();
        for n_entradas in [0, man.inputs.len()] {
            let base: Vec<(String, f32)> =
                params.iter().map(|p| (p.clone(), VALOR_DA_SONDA)).collect();
            let (n0, morreu) = conta_no_instante(reg, nome, n_entradas, &base, T);
            let entrada = if n_entradas > 0 { LINHAS_DE_ENTRADA } else { 0 };
            if morreu || n0 <= entrada {
                continue;
            }
            for k in 0..base.len() {
                let mut dobro = base.clone();
                dobro[k].1 *= 2.0;
                if conta_no_instante(reg, nome, n_entradas, &dobro, T).0 <= n0 {
                    continue;
                }
                let mut fio = base.clone();
                fio[k].1 = 1e9;
                eprintln!("  comeca {nome}.{} ({n_entradas} portas)", base[k].0);
                let t0 = std::time::Instant::now();
                let (n, morreu) = conta_no_instante(reg, nome, n_entradas, &fio, T);
                eprintln!(
                    "  {nome:<28} {:<16} {n:>9} linhas · {:>8.2} s{}",
                    base[k].0,
                    t0.elapsed().as_secs_f64(),
                    if morreu { " · ESTOUROU" } else { "" }
                );
            }
        }
    }
    std::panic::set_hook(anterior);
}
