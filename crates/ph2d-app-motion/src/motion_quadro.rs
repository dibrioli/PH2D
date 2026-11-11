//! **O Motion no quadro a pedido** (B23.2): o que ele anima com o playhead PARADO, fora do Play. O playhead a
//! tocar já pede por `Produtor::Animacao` na shell; aqui ficam os relógios que andam sem ele.

use ph2d_app_host::quadro::Produtor;

use crate::motion_state::MotionState;

/// Os produtores do Motion parado: o eco de uma largada (relógio de parede) e a leitura dos cartões ainda na
/// placa (`tap_sem_espera` recolhe-a no quadro SEGUINTE; parado, não se encomenda outra, logo acaba).
pub fn quer_quadro(motion: &MotionState, ativos: &mut Vec<Produtor>) {
    if motion.piscada.is_some() {
        ativos.push(Produtor::UiAnima);
    }
    if motion.gpu_live && motion.gpu_cook.tap_em_voo() {
        ativos.push(Produtor::LeituraDaPlaca);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O eco de uma largada pisca pelo relógio de parede (`MotionState::ui_now`): a 4 q/s do despertador ele
    /// saltava as passagens do triângulo. *Vermelho sem o produtor.*
    #[test]
    fn o_eco_da_largada_pede_quadro() {
        let mut m = MotionState::new();
        let mut ativos = Vec::new();
        quer_quadro(&m, &mut ativos);
        assert!(
            ativos.is_empty(),
            "parado e sem eco, o Motion não pede quadro: {ativos:?}"
        );
        m.piscada = Some(crate::motion_state::PiscadaPendente {
            nos: vec![1],
            fios: Vec::new(),
            inicio: 0.0,
        });
        quer_quadro(&m, &mut ativos);
        assert_eq!(
            ativos,
            [Produtor::UiAnima],
            "o eco da largada não pede quadro"
        );
    }
}
