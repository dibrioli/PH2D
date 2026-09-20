//! ⭐⭐⭐ **O RELÓGIO DO CARIMBO** — o terceiro irmão do [`crate::motion_carimbo_probe`], e o corte
//! é por RESPONSABILIDADE: a rota e a população são **decisões** (imunes à carga da máquina), e o
//! que vive aqui são **relógios**.
//!
//! ⚠️ É a distinção que o `CLAUDE.md` §5.0 cobra — *nenhuma leitura de relógio desta workstation
//! vale nada acima de `load ~5`* —, e é por isso que toda sonda deste ficheiro imprime a carga ao
//! lado do número. Separá-las em ficheiros diferentes é o que impede alguém de ler uma tabela
//! desta e uma tabela de lá como se as duas tivessem o mesmo valor probatório.
//!
//! À mão, em RELEASE e com a máquina calma:
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture audit_the_stamp_encode
//! ```

/// O `loadavg` de 1 minuto — ao lado de todo relógio (`CLAUDE.md` §5.0).
fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or("?")
        .to_string()
}

/// ⭐⭐ **O QUE CUSTA ENCODAR `N` CÓPIAS DA MESMA FORMA** — a escada que o ADR-0154 Fase 3 pede.
///
/// ⚠️ **Isto É um relógio** (as três sondas acima não são), logo a carga vai impressa ao lado e um
/// número tirado acima de `load ~5` não vale nada (`CLAUDE.md` §5.0).
///
/// ⭐ O lote passa pela **porta do produto** ([`ph2d_vec_render::draw_shared_instances`]), que já
/// tessela cada geometria DISTINTA uma vez — logo o que esta escada mede é o que SOBRA depois
/// dessa economia: o encode de `N` preenchimentos no Vello.
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn audit_the_stamp_encode_cost() {
    use ph2d_vector::{Affine, VectorScene};
    let caminho = ph2d_vec_scene::cook(
        ph2d_vec_scene::ShapeKind::Star,
        [-0.5, -0.5],
        [0.5, 0.5],
        &[5.0, 0.5],
    );
    eprintln!(
        "\n  ═══ O ENCODE DE `N` CÓPIAS DA MESMA ESTRELA (load {}) ═══\n",
        carga()
    );
    eprintln!("     cópias |       encode |   % quadro |  por cópia");
    eprintln!("  ----------|--------------|------------|-----------");
    for n in [1_000usize, 10_000, 102_400, 1_000_000] {
        let mut melhor = f64::INFINITY;
        for _ in 0..3 {
            let mut cena = VectorScene::new();
            let t = std::time::Instant::now();
            ph2d_vec_render::draw_shared_instances(
                (0..n).map(|i| {
                    let x = f64::from(u32::try_from(i % 320).unwrap_or(0)) * 0.01;
                    (1u32, Affine::translate((x, 0.0)), [1.0, 1.0, 1.0, 1.0])
                }),
                |_| Some(&caminho),
                &mut cena,
            );
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
        }
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
        let por = melhor * 1e3 / n as f64;
        eprintln!(
            "  {n:>9} | {melhor:>9.2} ms | {:>9.0}% | {por:>7.3} µs",
            melhor / 16.67 * 100.0
        );
    }
    eprintln!("\n  load no fim: {}\n", carga());
}
