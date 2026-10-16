//! ⭐ **Toda cena de ciclo coze pela PLACA e os cartões leem-na** — o smoke do dono corre a rota do
//! dispositivo (o `PH2D_GPU_COOK` é ON por omissão) com o painel aberto, e os gates de cada cena
//! cozem pela CPU. A `=118` pânicava ao quadro 12 na LEITURA DOS CARTÕES (`le_tap`: o anel `Mat4`
//! do `Slit Scan` lido como `Vec4`, `36 × 4 = 144`) e nenhum gate a varria: este varre a
//! [`super::CICLOS`] inteira pela porta do quadro e pela leitura sem espera, como o `dispatch`.

use crate::motion_state::MotionState;

/// Os quadros de cada cena — passa o 12 da `=118` e a cauda inteira do `Trail` dela (`12 × 4`).
const QUADROS: u32 = 60;
/// Os tiques por quadro: um quadro lento do produto recupera VÁRIOS (`ticks_owed`).
const SALTOS: [u32; 6] = [1, 2, 7, 1, 4, 3];

/// ⭐ **Cada cena de ciclo coze `QUADROS` quadros pela porta do quadro do produto, na placa, e a
/// leitura dos cartões recolhe-os sem pânico.** Uma cena que pânica não esconde as seguintes: o
/// veredito lista todas. ⚠️ O controlo: toda cena que coze na placa tem de RECOLHER uma leitura
/// (senão o gate não leu nada e passaria sobre o defeito).
#[test]
#[ignore = "placa: precisa de GPU"]
fn toda_cena_de_ciclo_coze_pela_placa() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        panic!("sem adapter — este gate mede a rota do device");
    };
    let mut caidas = Vec::new();
    let mut sem_leitura = Vec::new();
    let mut na_placa = Vec::new();
    for n in super::CICLOS {
        let corrida = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut m = MotionState::new();
            m.sinks = crate::motion_demo_legend::monta(n, &mut m.doc, &m.registry).0;
            assert!(!m.sinks.is_empty(), "a cena {n} monta");
            let mut playhead = ph2d_core::Playhead::new(1.0 / 60.0);
            playhead.play();
            let (mut viva, mut leu) = (false, false);
            for q in 0..QUADROS {
                playhead.advance_ticks(SALTOS[q as usize % SALTOS.len()]);
                crate::motion_bridge::quadro::o_quadro(
                    &mut m,
                    Some(&gpu),
                    &playhead,
                    1.0 / 60.0,
                    crate::motion_bridge::quadro::MaoDoEditor::default(),
                );
                // A leitura dos cartões do `dispatch` (`readout::take_tap`), só num quadro da placa.
                if m.gpu_live {
                    viva = true;
                    let _ = m
                        .gpu_cook
                        .tap_sem_espera(&gpu, ph2d_gpu_cook::tap::TAP_SAMPLES);
                    leu |= m.gpu_cook.tap_fresca();
                }
            }
            (viva, leu)
        }));
        match corrida {
            Err(_) => caidas.push(*n),
            Ok((true, false)) => sem_leitura.push(*n),
            Ok((true, true)) => na_placa.push(*n),
            Ok((false, _)) => {}
        }
    }
    assert!(caidas.is_empty(), "cenas que pânicam na placa: {caidas:?}");
    assert!(
        sem_leitura.is_empty(),
        "cenas na placa cuja leitura nunca chegou: {sem_leitura:?}"
    );
    assert!(
        na_placa.contains(&"118"),
        "a =118 coze e lê-se na placa: {na_placa:?}"
    );
}
