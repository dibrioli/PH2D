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
    /// O `Tab` que sai do Draw.
    SaiDoDraw,
    /// O Apply do Painter, sem sair do Draw.
    Aplica,
    /// O `Ctrl+Z`.
    Desfaz,
    /// Imprime as imagens das duas plataformas: `[sprite-pixel]`.
    Relata,
    /// O campo Position X do Inspector, 1 m à direita, pela porta do clique.
    Desloca,
    /// Um dos 9 pontos do Pivot (coluna, linha), pela porta do clique.
    Pivo([u8; 2]),
    /// A agulha da timeline neste tempo (s).
    Agulha(f64),
    /// Imprime a pose e as chaves da Platform High: `[sprite-prova]`.
    Prova,
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
        // Em Draw: mexe no Transform, Ctrl+Z aí dentro, sai, e Ctrl+Z fora.
        Foto::SpriteTransformNoDraw => {
            return match quadro.checked_sub(INICIO)? {
                0 | 20 | 40 | 70 | 100 => Some(Mao::Prova),
                10 => Some(Mao::Desloca),
                30 | 80 => Some(Mao::Desfaz),
                50 => Some(Mao::SaiDoDraw),
                _ => None,
            };
        }
        // Duas chaves: o pivô no pé em t = 1, à esquerda em t = 2. A timeline toca 1, 2 e o meio; depois Ctrl+Z.
        Foto::SpritePivotChave => {
            return match quadro.checked_sub(INICIO)? {
                0 | 20 | 50 | 70 | 90 | 110 | 140 => Some(Mao::Prova),
                10 => Some(Mao::Pivo([1, 2])),
                30 => Some(Mao::Agulha(2.0)),
                40 => Some(Mao::Pivo([0, 1])),
                60 => Some(Mao::Agulha(1.0)),
                80 => Some(Mao::Agulha(2.0)),
                100 => Some(Mao::Agulha(1.5)),
                120 => Some(Mao::Desfaz),
                _ => None,
            };
        }
        // Um traço na cópia aberta, e fica no Draw: a foto mostra onde ele caiu.
        Foto::SpriteNoPainterTraco => ([8.0, 2.0], [9.0, 2.0]),
        // Sem traço: entra, sai, e conta o que ficou.
        Foto::SpritePainterSemPintar => {
            return match quadro.checked_sub(INICIO + PASSOS) {
                Some(20) => Some(Mao::SaiDoDraw),
                Some(15 | 50) => Some(Mao::Relata),
                _ => None,
            };
        }
        // Um traço na imagem aberta (o centro da Platform High da cena 4), sair do Draw e Ctrl+Z.
        Foto::SpritePainterVolta | Foto::SpritePainterAplica => {
            if quadro == INICIO + PASSOS + 20 {
                return Some(if foto == Foto::SpritePainterAplica {
                    Mao::Aplica
                } else {
                    Mao::SaiDoDraw
                });
            }
            if quadro == INICIO + PASSOS + 15 || quadro == INICIO + PASSOS + 50 {
                return Some(Mao::Relata);
            }
            if quadro == INICIO + PASSOS + 60 {
                return Some(Mao::Desfaz);
            }
            if quadro == INICIO + PASSOS + 80 {
                return Some(Mao::Relata);
            }
            // Pelo Apply ainda se está no Draw: o Ctrl+Z acima é do Painter. Sai-se, relata-se, e desfaz-se o resto.
            if foto == Foto::SpritePainterAplica {
                match quadro.checked_sub(INICIO + PASSOS) {
                    Some(90) => return Some(Mao::SaiDoDraw),
                    Some(120 | 160) => return Some(Mao::Relata),
                    Some(130) => return Some(Mao::Desfaz),
                    _ => {}
                }
            }
            ([8.0, 2.0], [9.0, 2.0])
        }
        // Um vaivém de 0,2 m sobre o «Face», onde o «Swarm» cai (`ordem_smoke::pose`), um passo por quadro.
        Foto::OrdemSonda => {
            let k = quadro.checked_sub(INICIO)?;
            return Some(Mao::Move([2.4 + if k % 2 == 0 { 0.0 } else { 0.2 }, 1.6]));
        }
        // Por dentro da moldura (sobre o céu): 6 m para a direita — o offset do `follow` anda.
        Foto::CameraArrasta | Foto::CameraOlharArrasta | Foto::CameraChave => {
            ([-3.0, 4.0], [3.0, 4.0])
        }
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
