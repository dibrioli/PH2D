//! ⭐ **O `VecFilter` de um caminho, escrito na entidade dele** — a folha que a pilha de FX raster
//! e as cenas de smoke partilham.
//!
//! # Por que isto não vive na shell
//!
//! A lei era `shells/desktop/src/fx_live.rs::set_filter`, e é **pura**: `SimWorld` + o mapa
//! `caminho ⟺ entidade` + os componentes do [`ph2d_ecs`], zero `App` e zero `gfx`. Enquanto ela
//! estava na shell, **11 ficheiros** a chamavam — entre eles a cena `vec_fade_smoke`, que é um dos
//! quatro roteadores da família `vec`. Um roteador preso à shell mantém a família inteira na
//! catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`, que é **all-or-nothing por família**.
//!
//! ⚠️ **Duas famílias que partilham código partilham uma FOLHA, nunca uma delas à outra**
//! ([HOWTO §1.2]). Esta é a mesma peça que o [`crate::entity_map`] já é um degrau acima: *dado um
//! `VecPathId`, mexer na entidade que lhe corresponde*.
//!
//! ⛔ **O `fx_live::set_filter` da shell NÃO foi apagado** — ele delega para aqui numa linha, e os
//! 11 sítios de chamada ficam byte a byte iguais.
//!
//! [HOWTO §1.2]: ../../../docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md

use crate::entity_map::VecEntityMap;
use ph2d_ecs::{Entity, SimWorld, VecFilter};
use ph2d_vec_scene::VecPathId;

/// Põe (ou tira) o [`VecFilter`] das entidades de `ids`. Devolve **quantas mudaram**.
///
/// ⚠️ **Uma pilha VAZIA é a ausência do componente, não um componente vazio** — é o que o
/// `want.filter(|f| !f.ops.is_empty())` compra, e é load-bearing: o passe de FX varre por
/// `VecFilter` presente, então um componente com zero ops faria a forma pagar o isolamento em
/// textura para não aplicar operação nenhuma.
///
/// ⚠️ **E uma escrita que não muda nada é SALTADA** (`cur == want`): a comparação é o que impede
/// um quadro parado de sujar o `World` e nascer um passo de undo espúrio — a mesma lei que o
/// `App::post_frame_undo` da shell aplica um nível acima.
pub fn set_filter(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    want: Option<VecFilter>,
) -> usize {
    let want = want.filter(|f| !f.ops.is_empty());
    let mut n = 0;
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let cur = sim.world().get::<VecFilter>(e).cloned();
        if cur == want {
            continue;
        }
        let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
            continue;
        };
        match &want {
            Some(v) => {
                em.insert(v.clone());
            }
            None => {
                em.remove::<VecFilter>();
            }
        }
        n += 1;
    }
    n
}
