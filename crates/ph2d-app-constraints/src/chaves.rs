//! **As chaves que o cartão pede à timeline** — a compensação do pegar/largar ([`crate::compensa`]) e
//! o bake ([`crate::bake`]) —, escritas direto no documento num passo de undo (como o bake da física:
//! cada `TimelineIntent::AddKey` abriria um passo seu). A shell chama [`do_quadro`] uma vez por quadro,
//! ANTES do `autokey_pass`: com a chave de f já escrita, o Auto Key lê mundo == curva e não grava
//! outra (nem abre outro passo).

use ph2d_anim::{AnimTarget, KeyId, RationalTime};
use ph2d_core::Playhead;
use ph2d_ecs::SimWorld;
use ph2d_preview_drive::PreviewDrive;
use ph2d_timeline::{AnimValue, Instante, Interp, PropKind, TimelineState};

/// Uma escrita no documento.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Escrita {
    /// Uma chave de `p` em `e` no tempo `t` da CENA (segundos), com o valor que a cena deve mostrar.
    Chave {
        e: u64,
        p: PropKind,
        t: f64,
        v: f32,
        interp: Interp,
    },
    /// O valor de uma chave que já existe.
    Valor {
        target: AnimTarget,
        key: KeyId,
        v: f32,
    },
}

/// Grava `escritas` num passo de undo (junta-se a um já aberto, salvo `tudo`). O tempo e o valor
/// passam pelas portas do Auto Key (`remapped_time`/`key_home`, `key_value_in_active_clip`); uma chave
/// sem casa é saltada — ou, com `tudo`, recusa o lote inteiro (um bake com buracos mentiria).
/// `agora` = o instante da vista, re-preparado no fim. Devolve se gravou.
pub fn gravar(
    tl: &mut TimelineState,
    escritas: &[Escrita],
    solo: bool,
    agora: f64,
    tudo: bool,
) -> bool {
    if escritas.is_empty() || (tudo && tl.history.is_open()) {
        return false;
    }
    let proprio = !tl.history.is_open();
    if proprio {
        tl.history.begin(&tl.doc);
    }
    let raiz = tl.container_open;
    let fps = tl.doc.fps_display;
    for &w in escritas {
        match w {
            Escrita::Chave { e, p, t, v, interp } => {
                let doc = &mut tl.doc;
                let fonte = ph2d_timeline::remapped_time(doc, e, t);
                if !solo {
                    doc.prime_rooted(raiz, t);
                }
                let casa = if solo {
                    Ok(fonte)
                } else {
                    ph2d_timeline::key_home(doc, e, t)
                };
                let valor = if solo {
                    Ok(v)
                } else {
                    ph2d_timeline::key_value_in_active_clip(doc, e, p, v, fonte)
                };
                let (Ok(casa), Ok(valor)) = (casa, valor) else {
                    if tudo {
                        tl.history.cancel();
                        return false;
                    }
                    continue;
                };
                let t = ph2d_timeline::snap_time(RationalTime::from_seconds(casa), fps, true);
                doc.upsert_key(e, p, t, AnimValue::Float(valor), interp);
            }
            Escrita::Valor { target, key, v } => {
                if let Some(track) = tl.doc.active_clip_mut().track_mut(target) {
                    track.set_value(key, AnimValue::Float(v));
                }
            }
        }
    }
    if !solo {
        tl.doc.prime_rooted(raiz, agora);
    }
    if proprio {
        let doc = tl.doc.clone();
        tl.history.commit_if_changed(&doc);
    }
    true
}

/// ⭐ **A porta do quadro**: as chaves de compensação dos gestos do cartão (com o Auto Key armado:
/// `painel` da timeline aberto e a flag) e os bakes pedidos (`Cozer`). `interp` = a da chave nova da
/// casa. Chamar no quadro do commit do cartão, antes de a pilha voltar a correr (ver [`crate::compensa`]).
pub fn do_quadro(
    tl: &mut TimelineState,
    sim: &mut SimWorld,
    preview: &PreviewDrive,
    playhead: &Playhead,
    inst: Instante,
    painel: bool,
    interp: Interp,
) {
    let armado = painel && tl.flags.auto_key;
    let comp = crate::compensa::compensar(&tl.doc, sim, preview, inst, armado, interp);
    gravar(tl, &comp, inst.solo, inst.t_cut, false);
    for dono in crate::bake::pedidos(sim) {
        let (de, ate) = ph2d_timeline::bake_range(&tl.doc, playhead);
        let lote = crate::bake::bake(sim, tl, preview, dono, de, ate);
        gravar(tl, &lote, false, inst.t_cut, true);
    }
}
