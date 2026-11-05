//! ⭐ **A proporção do Scale é do OBJECTO, não dos bits.** Os bits são da sessão: o Desfazer troca-os, e um
//! mundo novo volta a dá-los a outro objecto. A chave durável é o `StableId` com o nome (a de
//! `ph2d_card_rope::chave`): o nome impede que o mesmo número noutro projecto apanhe a proporção. Quem o
//! Apagar levou fica em `FORA` até o Desfazer o trazer; cabe um registo por objecto, e abrir a corrente
//! esvazia tudo. O painel não vê o mundo: a composição (`ph2d_host_presos`) responde pelas portas abaixo.

use ph2d_editor_core::par_travado::ParTravado;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// `(StableId, nome)`.
pub type ChaveDaTrava = (u64, String);

struct Trava {
    par: ParTravado,
    chave: Option<ChaveDaTrava>,
}

thread_local! {
    /// A proporção de cada objecto tocado com a corrente fechada; esvazia ao abri-la.
    static TRAVAS: RefCell<BTreeMap<u64, Trava>> = const { RefCell::new(BTreeMap::new()) };
    /// A de quem saiu do mundo, à espera de o Desfazer o trazer.
    static FORA: RefCell<BTreeMap<ChaveDaTrava, ParTravado>> =
        const { RefCell::new(BTreeMap::new()) };
}

/// A corrente mudou: nenhuma proporção fica, salvo a do objecto sobre o qual ela fechou.
pub(crate) fn recomeca(fechada_sobre: Option<(u64, ParTravado)>) {
    FORA.with_borrow_mut(BTreeMap::clear);
    TRAVAS.with_borrow_mut(|travas| {
        travas.clear();
        if let Some((bits, par)) = fechada_sobre {
            travas.insert(bits, Trava { par, chave: None });
        }
    });
}

/// O par de `bits` (o de agora, se ainda não havia): a escrita da corrente fechada.
pub(crate) fn com_par<R>(bits: u64, vivo: [f64; 2], f: impl FnOnce(&mut ParTravado) -> R) -> R {
    TRAVAS.with_borrow_mut(|travas| {
        let t = travas.entry(bits).or_insert_with(|| Trava {
            par: ParTravado::ao_ligar(vivo),
            chave: None,
        });
        f(&mut t.par)
    })
}

/// O espelho muda o sinal de um eixo sem levar o outro, e a proporção guardada muda com ele.
pub(crate) fn espelhou(bits: u64, eixo: usize) {
    TRAVAS.with_borrow_mut(|travas| {
        if let Some(t) = travas.get_mut(&bits) {
            t.par.espelhou(eixo);
        }
    });
}

/// Os objectos do mundo com proporção guardada.
#[must_use]
pub fn travas_de_escala() -> Vec<u64> {
    TRAVAS.with_borrow(|t| t.keys().copied().collect())
}

/// Quantas proporções esperam o Desfazer.
#[must_use]
pub fn travas_fora_do_mundo() -> usize {
    FORA.with_borrow(BTreeMap::len)
}

fn guarda_fora(trava: Trava) {
    if let Some(chave) = trava.chave {
        FORA.with_borrow_mut(|fora| {
            fora.insert(chave, trava.par);
        });
    }
}

/// ⭐ **Cada quadro e antes de um restauro, com o objecto ainda no mundo:** cada trava aprende a chave
/// do objecto dela. Bits mortos, ou que passaram a ser de outro objecto, deixam de lhe dar a proporção.
pub fn anota_chaves_de_trava(
    existe: impl Fn(u64) -> bool,
    chave_de: impl Fn(u64) -> Option<ChaveDaTrava>,
) {
    TRAVAS.with_borrow_mut(|travas| {
        *travas = std::mem::take(travas)
            .into_iter()
            .filter_map(|(bits, mut t)| {
                match (chave_de(bits), &t.chave) {
                    (Some(c), Some(d)) if c.0 != d.0 => {}
                    (Some(c), _) => {
                        t.chave = Some(c);
                        return Some((bits, t));
                    }
                    (None, None) => return existe(bits).then_some((bits, t)),
                    (None, Some(_)) => {}
                }
                guarda_fora(t);
                None
            })
            .collect();
    });
}

/// ⭐ O respawn do Desfazer trocou os bits: a proporção de cada objecto passa aos de agora (`novo`), e a
/// de quem o passo levou fica à espera dele. Sem isto, desfazer para um par com zero dava 1:1
/// (`doc CartoesUI/10`).
pub fn reata_travas_de_escala(novo: impl Fn(u64) -> Option<u64>) {
    TRAVAS.with_borrow_mut(|travas| {
        *travas = std::mem::take(travas)
            .into_iter()
            .filter_map(|(bits, t)| match novo(bits) {
                Some(b) => Some((b, t)),
                None => {
                    guarda_fora(t);
                    None
                }
            })
            .collect();
    });
}

/// O objecto voltou ao mundo (`achar` dá os bits dele): volta com a proporção que tinha.
pub fn devolve_travas(achar: impl Fn(&ChaveDaTrava) -> Option<u64>) {
    FORA.with_borrow_mut(|fora| {
        fora.retain(|chave, par| {
            let Some(bits) = achar(chave) else {
                return true;
            };
            TRAVAS.with_borrow_mut(|travas| {
                travas.entry(bits).or_insert_with(|| Trava {
                    par: *par,
                    chave: Some(chave.clone()),
                });
            });
            false
        });
    });
}
