//! **A mão da câmera** (onda 17b): as fotos `camera_arrasta`, `camera_clique`, `camera_borda` e `camera_canto` são
//! GESTOS — o shell aperta, arrasta e solta pelas portas reais do ponteiro (`App::pointer_button`), com o céu por
//! baixo e o passe da câmera a correr com `follow` e limites. A tela virtual não recebe cliques do sistema; estes
//! entram pela mesma porta que o rato do winit.

use crate::smoke::Foto;
use ph2d_ecs::{CameraFollow, CameraLimits, GameCamera, SimWorld};

/// Um gesto num quadro, em metros.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mao {
    Desce([f32; 2]),
    Move([f32; 2]),
    Sobe,
}

/// Quadros até a cena assentar (a seleção, a 1.ª moldura e o rebobinar já correram).
const INICIO: u32 = 40;
/// Movimentos entre o `Down` e o `Up`, um por quadro.
const PASSOS: u32 = 6;

/// A borda de cima-direita da moldura da câmera da cena 4: presa à cerca da esquerda (−13), 8 m de altura, 16:9.
const CANTO: [f32; 2] = [-13.0 + 8.0 * 16.0 / 9.0, 7.0];

/// O gesto de `foto` no `quadro` (contado por [`proximo_quadro_da_mao`]).
#[must_use]
pub fn mao(foto: Foto, quadro: u32) -> Option<Mao> {
    let (de, ate) = match foto {
        // Por dentro da moldura (sobre o céu): 6 m para a direita — o offset do `follow` anda.
        Foto::CameraArrasta | Foto::CameraOlharArrasta => ([-3.0, 4.0], [3.0, 4.0]),
        // O mesmo sítio, sem mexer: o clique é da seleção.
        Foto::CameraClique => ([-3.0, 4.0], [-3.0, 4.0]),
        // A borda esquerda dos limites, 3 m para fora.
        Foto::CameraBorda => ([-13.0, 4.0], [-16.0, 4.0]),
        // O canto de cima-direita, 3 m para fora em x: a cerca empurra o centro e o canto segue o ponteiro.
        Foto::CameraCanto => (CANTO, [CANTO[0] + 3.0, CANTO[1]]),
        _ => return None,
    };
    let k = quadro.checked_sub(INICIO)?;
    #[allow(clippy::cast_precision_loss)]
    let t = k as f32 / PASSOS as f32;
    match k {
        0 => Some(Mao::Desce(de)),
        k if k <= PASSOS => Some(Mao::Move([
            de[0] + (ate[0] - de[0]) * t,
            de[1] + (ate[1] - de[1]) * t,
        ])),
        k if k == PASSOS + 1 => Some(Mao::Sobe),
        _ => None,
    }
}

static QUADRO: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// O quadro da mão (um contador próprio: o do condutor anda à parte).
pub fn proximo_quadro_da_mao() -> u32 {
    QUADRO.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// O que o gesto deixou: quem está selecionado e o estado da câmera — o relato do `stderr`.
#[must_use]
pub fn relato(sim: &mut SimWorld, selecao: Option<u64>) -> String {
    let nome = |sim: &SimWorld, b: u64| {
        sim.world()
            .get::<ph2d_ecs::Name>(ph2d_ecs::Entity::from_bits(b))
            .map_or_else(|| format!("{b:x}"), |n| n.0.clone())
    };
    let sel = selecao.map_or_else(|| "nada".to_owned(), |b| nome(sim, b));
    let Some(cam) = crate::smoke::por_nome(sim, "Camera") else {
        return format!("selecao={sel}");
    };
    let w = sim.world();
    let altura = w.get::<GameCamera>(cam).map_or(0.0, |c| c.height_world);
    let off = w.get::<CameraFollow>(cam).map(|f| f.offset);
    let lim = w.get::<CameraLimits>(cam).map(|l| (l.min, l.max));
    format!("selecao={sel} · altura={altura:.3} · follow_offset={off:?} · limites={lim:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O roteiro: desce, anda `PASSOS` quadros até ao fim, sobe — e só nas fotos da mão.
    #[test]
    fn a_mao_desce_anda_e_sobe() {
        assert_eq!(mao(Foto::CameraArrasta, INICIO - 1), None);
        assert_eq!(
            mao(Foto::CameraArrasta, INICIO),
            Some(Mao::Desce([-3.0, 4.0]))
        );
        assert_eq!(
            mao(Foto::CameraArrasta, INICIO + PASSOS),
            Some(Mao::Move([3.0, 4.0]))
        );
        assert_eq!(
            mao(Foto::CameraArrasta, INICIO + PASSOS + 1),
            Some(Mao::Sobe)
        );
        assert_eq!(mao(Foto::CameraArrasta, INICIO + PASSOS + 2), None);
        assert_eq!(mao(Foto::Camera, INICIO), None);
    }
}
