//! **O som do Play no mixer do host** (P13 T): a família diz, uma vez por quadro e ANTES de o Play
//! andar, como está o sprite activo ([`Vista`]); o motor liga as vozes no Play (a partir do instante
//! do quadro onde ele começa), alimenta-as, pára-as no Stop e, parado, toca o pedaço do quadro para
//! onde o cursor foi (o scrub). O `dt` devolvido é o do som enquanto há som, senão o do host.
//!
//! O mixer é emprestado pela shell a cada quadro (`ph2d_app_audio::AudioSystem::engine_mut`, por
//! composição); sem mixer (testes, máquina sem dispositivo) nada toca e o `dt` é o do host.

use std::cell::RefCell;

use ph2d_audio::AudioEngine;
use ph2d_pixel_core::{FrameId, Sprite};

use crate::{Tocador, dt_do_play};

/// O sprite activo visto pelo som.
#[derive(Clone, Copy)]
pub struct Vista<'a> {
    pub chave: u32,
    pub sprite: &'a Sprite,
    /// A posição do quadro mostrado.
    pub quadro: usize,
    pub tocando: bool,
}

impl<'a> Vista<'a> {
    /// A vista de um documento: o quadro mostrado é o do Play (`play = Some(posição)`) ou `frame`.
    #[must_use]
    pub fn de(chave: u32, sprite: &'a Sprite, frame: FrameId, play: Option<usize>) -> Self {
        let parado = sprite.frames().iter().position(|f| f.id == frame);
        Self {
            chave,
            sprite,
            quadro: play.or(parado).unwrap_or(0),
            tocando: play.is_some(),
        }
    }

    /// O instante em ms onde o quadro mostrado começa.
    fn t_ms(&self) -> u64 {
        let f = self.sprite.frames();
        f[..self.quadro.min(f.len())]
            .iter()
            .map(|f| u64::from(f.duration_ms.max(1)))
            .sum()
    }
}

/// O que o motor pediu ao mixer por último (os gates lêem-no).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acao {
    Tocar { t_ms: u64 },
    Parar,
    Pedaco { t_ms: u64, dur_ms: u32 },
}

#[derive(Default)]
struct Estado {
    tocador: Tocador,
    tocava: bool,
    /// O sprite e o quadro do último quadro do host (o scrub toca quando o quadro parado muda).
    onde: Option<(u32, usize)>,
    acao: Option<Acao>,
}

thread_local! {
    static ESTADO: RefCell<Estado> = RefCell::default();
}

/// Uma vez por quadro do host, antes do Play andar: segue a `vista` no mixer e devolve o `dt` do
/// Play — o do som quando há faixa a tocar, senão `host_ms` tal como hoje.
pub fn quadro(engine: Option<&mut AudioEngine>, vista: Option<Vista<'_>>, host_ms: u32) -> u32 {
    ESTADO.with(|e| {
        let e = &mut *e.borrow_mut();
        let onde = vista.map(|v| (v.chave, v.quadro));
        let antes = std::mem::replace(&mut e.onde, onde);
        let Some(engine) = engine else {
            return host_ms;
        };
        match vista {
            Some(v) if v.tocando && !e.tocava => {
                let t_ms = v.t_ms();
                crate::canal::com_sons(|s| e.tocador.tocar(engine, v.sprite.audio(), s, t_ms)).ok();
                e.tocava = true;
                e.acao = Some(Acao::Tocar { t_ms });
            }
            Some(v) if v.tocando => {}
            _ if e.tocava => {
                e.tocador.parar(engine);
                e.tocava = false;
                e.acao = Some(Acao::Parar);
            }
            // O scrub: o mesmo sprite, com som, parado noutro quadro.
            Some(v)
                if !v.sprite.audio().is_empty()
                    && antes.is_some_and(|(k, q)| k == v.chave && q != v.quadro) =>
            {
                let t_ms = v.t_ms();
                let dur_ms = v
                    .sprite
                    .frames()
                    .get(v.quadro)
                    .map_or(1, |f| f.duration_ms.max(1));
                let faixas = v.sprite.audio();
                crate::canal::com_sons(|s| e.tocador.pedaco(engine, faixas, s, t_ms, dur_ms)).ok();
                e.acao = Some(Acao::Pedaco { t_ms, dur_ms });
            }
            _ => {}
        }
        e.tocador.bombear();
        if e.tocava {
            dt_do_play(e.tocador.relogio(), host_ms)
        } else {
            host_ms
        }
    })
}

/// O último pedido ao mixer, e quantas faixas estão nele agora.
#[must_use]
pub fn ultima_acao() -> (Option<Acao>, usize) {
    ESTADO.with(|e| {
        let e = e.borrow();
        (e.acao, e.tocador.faixas_a_tocar())
    })
}
