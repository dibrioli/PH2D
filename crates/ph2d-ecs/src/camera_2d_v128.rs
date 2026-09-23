//! **A `GameCamera` como os ficheiros `128`..`167` a gravavam** — CONGELADA, para a escada do load
//! (auditoria 26 da paralaxe, §2.2).
//!
//! # ⛔⛔ O defeito que ela fecha
//!
//! O degrau `167 → 168` apendou o `dolly` à [`super::GameCamera`], e o postcard é POSICIONAL. A
//! escada do load tem um degrau VIVO no `128` (o `migrate_v128_to_v129` da shell) — e ele só reescrevia
//! os blobs de `SignalActions`. Um ficheiro v128 com uma câmera do jogo abria com o blob dela a falhar
//! na descodificação (`DeserializeUnexpectedEnd`), e o restauro do mundo parava na primeira linha que
//! falha: **a cena abria truncada, sem aviso**. A lei já estava escrita no degrau `148 → 149` (*«o
//! `migrate_v1_blob` escreve o formato VIVO e tem de aprender cada campo apendado»*) e esta linha não
//! a cumpriu.
//!
//! ⛔ **Nunca a corra sobre um blob que não venha de um ficheiro `< 168`**: quem garante a origem é o
//! número do ficheiro. Um blob vivo é recusado (sobram os quatro bytes do `dolly`).

use super::GameCamera;
use serde::Deserialize;

/// A câmera na forma `128..167`. ⚠️ A ordem dos campos É o formato.
#[derive(Deserialize)]
struct GameCameraV128 {
    height_world: f32,
    offset: [f32; 2],
    priority: i32,
    active: bool,
    cull_mask: u32,
}

/// ⭐ **Os bytes de uma `GameCamera` `128..167`, reescritos no formato VIVO** — com `dolly = 0`, que é
/// a omissão e a identidade da lei.
///
/// ⚠️ Ela escreve o formato VIVO, logo cresce a cada campo apendado à câmera — a mesma lei do
/// [`super::super::signal_actions::migrate_v1_blob`]. `None` = os bytes não se leem como a forma
/// antiga: quem chama deixa o blob como estava e conta-o.
#[must_use]
pub fn migrate_v128_blob(bytes: &[u8]) -> Option<Vec<u8>> {
    // ⚠️ `take_from_bytes` e rejeitar o resto — o postcard consome um prefixo válido e ignora o que
    // sobra, e um blob VIVO leria-se aqui como antigo com o `dolly` a sobrar.
    let Ok((a, [])) = postcard::take_from_bytes::<GameCameraV128>(bytes) else {
        return None;
    };
    postcard::to_allocvec(&GameCamera {
        height_world: a.height_world,
        offset: a.offset,
        priority: a.priority,
        active: a.active,
        cull_mask: a.cull_mask,
        dolly: 0.0,
    })
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Antiga {
        height_world: f32,
        offset: [f32; 2],
        priority: i32,
        active: bool,
        cull_mask: u32,
    }

    /// ⭐ Um blob antigo lê-se e volta VIVO, com os cinco campos e o `dolly` a zero; o CONTROLO é que
    /// o tipo vivo o recusa (é o defeito que a migração existe para curar) e que um blob vivo é
    /// recusado aqui.
    #[test]
    fn um_blob_antigo_volta_vivo_com_o_dolly_a_zero() {
        let antiga = Antiga {
            height_world: 12.5,
            offset: [1.0, -2.0],
            priority: 3,
            active: false,
            cull_mask: 0b1011,
        };
        let bytes = postcard::to_allocvec(&antiga).expect("encode");
        assert!(
            postcard::from_bytes::<GameCamera>(&bytes).is_err(),
            "o controlo: o tipo vivo não lê a forma antiga"
        );
        let vivo = migrate_v128_blob(&bytes).expect("migra");
        let c: GameCamera = postcard::from_bytes(&vivo).expect("lê vivo");
        assert_eq!(
            c,
            GameCamera {
                height_world: 12.5,
                offset: [1.0, -2.0],
                priority: 3,
                active: false,
                cull_mask: 0b1011,
                dolly: 0.0,
            }
        );
        let ja_vivo = postcard::to_allocvec(&GameCamera::default()).expect("encode");
        assert!(
            migrate_v128_blob(&ja_vivo).is_none(),
            "um blob vivo é recusado"
        );
    }
}
