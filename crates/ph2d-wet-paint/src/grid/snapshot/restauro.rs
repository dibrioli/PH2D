//! O que o restauro e o inverso partilham: os planos preguiçosos ao comprimento da base e os tiles por PLANO
//! guardado (B18.11 cura 3) — filho de [`super`] por LOC.

use super::*;

/// Os planos que nascem VAZIOS (`garante`): o comprimento na base decide se o restauro os esvazia.
pub(super) fn comprimentos_vazios(g: &Grid) -> [usize; 9] {
    [
        g.altura.len(),
        g.espessura.len(),
        g.crista.len(),
        g.nivelado.len(),
        g.material.len(),
        g.aparencia.len(),
        g.oleo.len(),
        g.tinta.len(),
        g.pigmento.len(),
    ]
}

/// Leva um plano preguiçoso ao comprimento da base antes de colar os tiles: vazio na base ⇒ vazio;
/// cheio na base e vazio agora só acontece depois de uma acção de tela inteira, que marcou TODOS os
/// tiles — a amostra é preenchimento que a colagem cobre.
pub(super) fn ajusta<T: Clone>(v: &mut Vec<T>, base: usize, amostra: Option<&T>) {
    if base == 0 {
        v.clear();
    } else if v.len() != base
        && let Some(a) = amostra
    {
        v.resize(base, a.clone());
    }
}

/// O [`ajusta`] do campo esparso: o preenchimento é o chão (a colagem dos tiles cobre a tela inteira).
pub(super) fn ajusta_campo<T: Copy>(
    v: &mut campo_esparso::CampoEsparso<T>,
    base: usize,
    amostra: bool,
) {
    if base == 0 {
        v.clear();
    } else if v.len() != base && amostra {
        *v = campo_esparso::CampoEsparso::novo(v.chao(), base);
    }
}

/// Os planos que o restauro de `a` (com o `extra` `b`) escreve no tile `k`: o inverso guarda esses.
pub(super) fn planos_do_par(a: &Registo, b: Option<&Registo>, k: u32) -> Planos {
    let g = |r: &Registo| r.finos.get(&k).map_or(Planos::default(), |t| t.guardados);
    b.map_or(g(a), |b| g(a).uniao(g(b)))
}

/// Os tiles que o restauro de `a` (com o `extra` `b`) põe, na ordem de sempre: os de `a` (com os planos que só `b`
/// guarda nesse tile), depois os de `b` que `a` não tem.
pub(super) fn tiles_do_restauro<'a>(
    a: &'a Registo,
    b: Option<&'a Registo>,
) -> Vec<(u32, std::borrow::Cow<'a, TileFino>)> {
    use std::borrow::Cow;
    let mut v: Vec<(u32, Cow<'a, TileFino>)> = a
        .finos
        .iter()
        .map(|(k, t)| match b.and_then(|b| b.finos.get(k)) {
            Some(o) if !o.guardados.menos(t.guardados).nenhum() => {
                let mut j = t.clone();
                j.junta(o.clone());
                (*k, Cow::Owned(j))
            }
            _ => (*k, Cow::Borrowed(t)),
        })
        .collect();
    if let Some(b) = b {
        v.extend(
            b.finos
                .iter()
                .filter(|(k, _)| !a.finos.contains_key(k))
                .map(|(k, t)| (*k, Cow::Borrowed(t))),
        );
    }
    v
}
