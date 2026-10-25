//! ⭐⭐⭐ **Acrescentar ou apagar um ponto numa forma ANIMADA vale em TODAS as chaves** (decisão do
//! dono, 20/10: a forma inteira numa chave, AE/Spine — plano 35 §2 escolha 4).
//!
//! Sem isto, a forma deste quadro passava a ter `n ± 1` vértices e as chaves dos outros instantes
//! continuavam com `n`: o vértice `j` de uma chave casava com o `j` de outra que já era outro ponto,
//! e a forma TORCIA entre elas — o defeito do shape tween do Flash.
//!
//! **Sem adivinhar a correspondência:** as âncoras que a edição não tocou ficam IGUAIS bit a bit
//! (inserir num segmento só mexe nas alças dos vizinhos; apagar só tira), então o alinhamento é a
//! maior subsequência comum de âncoras EXACTAS entre o quadro anterior e este. Um ponto novo acha o
//! seu parâmetro `u` projectando-se no segmento de ontem, e em cada chave esse segmento divide-se no
//! mesmo `u` (de Casteljau) — a forma de cada chave não muda de desenho, ganha um ponto. Um ponto
//! apagado sai de todas. Funciona para TODA porta que edita pontos, sem que nenhuma o declare.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_anim::{AnimValue, AttributeEvaluator, Interp, RationalTime};
use ph2d_ecs::FieldKey;
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as c;
use ph2d_timeline::{PropKind, TimelineDoc};

/// Os 8 campos de um vértice, na ordem do vocabulário (`ANCORA_X ..= TIPO_VERTICE`).
type Vert = [f32; 8];
/// Uma escrita de forma: tempo, interpolação e os campos novos.
type Escrita = (RationalTime, Interp, Vec<(FieldKey, f32)>);
const N_CAMPOS: u16 = c::TIPO_VERTICE - c::ANCORA_X + 1;

fn k(campo: u16, i: u16, j: u16) -> FieldKey {
    FieldKey::new(VETOR, campo, i, j)
}

fn n16(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

/// Os contornos que uma leitura da família tem: `i` → os vértices, em ordem.
fn contornos(lidos: impl IntoIterator<Item = (FieldKey, f32)>) -> BTreeMap<u16, Vec<Vert>> {
    let mut out: BTreeMap<u16, BTreeMap<u16, Vert>> = BTreeMap::new();
    for (kk, v) in lidos {
        if kk.familia == VETOR && (c::ANCORA_X..=c::TIPO_VERTICE).contains(&kk.campo) {
            out.entry(kk.i).or_default().entry(kk.j).or_insert([0.0; 8])
                [usize::from(kk.campo - c::ANCORA_X)] = v;
        }
    }
    out.into_iter()
        .map(|(i, vs)| (i, vs.into_values().collect()))
        .collect()
}

/// A maior subsequência comum de âncoras EXACTAS: os pares `(índice de ontem, índice de hoje)`.
fn alinhar(antes: &[Vert], agora: &[Vert]) -> Vec<(usize, usize)> {
    let (n, m) = (antes.len(), agora.len());
    let igual = |a: usize, b: usize| antes[a][0] == agora[b][0] && antes[a][1] == agora[b][1];
    let mut t = vec![vec![0u32; m + 1]; n + 1];
    for a in (0..n).rev() {
        for b in (0..m).rev() {
            t[a][b] = if igual(a, b) {
                t[a + 1][b + 1] + 1
            } else {
                t[a + 1][b].max(t[a][b + 1])
            };
        }
    }
    let (mut a, mut b, mut pares) = (0, 0, Vec::new());
    while a < n && b < m {
        if igual(a, b) {
            pares.push((a, b));
            a += 1;
            b += 1;
        } else if t[a + 1][b] >= t[a][b + 1] {
            a += 1;
        } else {
            b += 1;
        }
    }
    pares
}

type P = [f32; 2];
fn lerp(a: P, b: P, u: f32) -> P {
    [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u]
}
fn cubica(p: [P; 4], u: f32) -> P {
    let (q0, q1, q2) = (
        lerp(p[0], p[1], u),
        lerp(p[1], p[2], u),
        lerp(p[2], p[3], u),
    );
    lerp(lerp(q0, q1, u), lerp(q1, q2, u), u)
}
fn seg(a: &Vert, b: &Vert) -> [P; 4] {
    [[a[0], a[1]], [a[4], a[5]], [b[2], b[3]], [b[0], b[1]]]
}

/// O parâmetro do ponto `x` sobre o segmento — amostragem e refinamento (o ponto ESTÁ na curva).
fn projectar(s: [P; 4], x: P) -> f32 {
    let d = |u: f32| {
        let p = cubica(s, u);
        (p[0] - x[0]).powi(2) + (p[1] - x[1]).powi(2)
    };
    let mut melhor = (0..=64)
        .map(|i| i as f32 / 64.0)
        .fold(0.0, |b, u| if d(u) < d(b) { u } else { b });
    let mut passo = 1.0 / 64.0;
    for _ in 0..24 {
        passo *= 0.5;
        for cand in [melhor - passo, melhor + passo] {
            if (0.0..=1.0).contains(&cand) && d(cand) < d(melhor) {
                melhor = cand;
            }
        }
    }
    melhor
}

/// Divide o segmento `a→b` em `u`: devolve o vértice novo e corrige as alças dos vizinhos.
fn dividir(a: &mut Vert, b: &mut Vert, u: f32, modelo: &Vert) -> Vert {
    let [p0, p1, p2, p3] = seg(a, b);
    let (q0, q1, q2) = (lerp(p0, p1, u), lerp(p1, p2, u), lerp(p2, p3, u));
    let (r0, r1) = (lerp(q0, q1, u), lerp(q1, q2, u));
    let s = lerp(r0, r1, u);
    [a[4], a[5]] = q0;
    [b[2], b[3]] = q2;
    [s[0], s[1], r0[0], r0[1], r1[0], r1[1], modelo[6], modelo[7]]
}

/// A forma de UMA chave refeita na topologia de hoje.
fn refazer(
    ontem: &[Vert],
    hoje: &[Vert],
    na_chave: &[Vert],
    pares: &[(usize, usize)],
    fechado: bool,
) -> Vec<Vert> {
    let de_hoje: BTreeMap<usize, usize> = pares.iter().map(|&(a, b)| (b, a)).collect();
    let mut out: Vec<Option<Vert>> = vec![None; hoje.len()];
    for (&b, &a) in &de_hoje {
        out[b] = na_chave.get(a).copied();
    }
    // Cada corrida de pontos novos entre dois vizinhos alinhados divide o segmento deles.
    let mut b = 0;
    while b < hoje.len() {
        if out[b].is_some() {
            b += 1;
            continue;
        }
        let ini = b;
        while b < hoje.len() && out[b].is_none() {
            b += 1;
        }
        let ante = ini.checked_sub(1).or(fechado.then(|| hoje.len() - 1));
        let depois = (b < hoje.len()).then_some(b).or(fechado.then_some(0));
        let (Some(pa), Some(pb)) = (ante, depois) else {
            // Uma ponta aberta ganhou pontos: ficam como hoje (não há segmento de ontem).
            for (j, slot) in out.iter_mut().enumerate().take(b).skip(ini) {
                *slot = Some(hoje[j]);
            }
            continue;
        };
        let (Some(&oa), Some(&ob)) = (de_hoje.get(&pa), de_hoje.get(&pb)) else {
            for (j, slot) in out.iter_mut().enumerate().take(b).skip(ini) {
                *slot = Some(hoje[j]);
            }
            continue;
        };
        let s_ontem = seg(&ontem[oa], &ontem[ob]);
        let mut us: Vec<(f32, usize)> = (ini..b)
            .map(|j| (projectar(s_ontem, [hoje[j][0], hoje[j][1]]), j))
            .collect();
        us.sort_by(|x, y| x.0.total_cmp(&y.0));
        let (mut va, mut vb) = (na_chave[oa], na_chave[ob]);
        let mut feito = 0.0;
        let mut novos = Vec::new();
        for (u, j) in us {
            let ur = if feito < 1.0 {
                ((u - feito) / (1.0 - feito)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let mut esquerda = novos.last().map_or(va, |&(_, v)| v);
            let nv = dividir(&mut esquerda, &mut vb, ur, &hoje[j]);
            match novos.last_mut() {
                Some((_, v)) => *v = esquerda,
                None => va = esquerda,
            }
            novos.push((j, nv));
            feito = u;
        }
        if let Some(sa) = out.get_mut(pa) {
            *sa = Some(va);
        }
        if let Some(sb) = out.get_mut(pb) {
            *sb = Some(vb);
        }
        for (j, v) in novos {
            out[j] = Some(v);
        }
    }
    out.into_iter()
        .enumerate()
        .map(|(j, v)| v.unwrap_or(hoje[j]))
        .collect()
}

/// Um contorno de hoje é o mesmo de ontem se TODAS as âncoras são iguais bit a bit.
fn mesmas_ancoras(a: &[Vert], b: &[Vert]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| x[0].to_bits() == y[0].to_bits() && x[1].to_bits() == y[1].to_bits())
}

/// Casa cada contorno de hoje com um de ontem (`hoje → ontem`): primeiro o de âncoras iguais (o
/// primeiro livre, pela ordem); depois, entre os que sobram, o que partilha mais âncoras exactas
/// (um contorno a que a mesma edição também mexeu nos pontos). Sem par ⇒ contorno novo.
fn casar(ontem: &BTreeMap<u16, Vec<Vert>>, hoje: &BTreeMap<u16, Vec<Vert>>) -> BTreeMap<u16, u16> {
    let mut par: BTreeMap<u16, u16> = BTreeMap::new();
    let mut livre: BTreeSet<u16> = ontem.keys().copied().collect();
    for (&hi, h) in hoje {
        if let Some(&oi) = livre.iter().find(|oi| mesmas_ancoras(&ontem[oi], h)) {
            livre.remove(&oi);
            par.insert(hi, oi);
        }
    }
    for (&hi, h) in hoje {
        if par.contains_key(&hi) {
            continue;
        }
        let melhor = livre
            .iter()
            .map(|&oi| (alinhar(&ontem[&oi], h).len(), oi))
            .filter(|&(n, _)| n > 0)
            .fold(None, |b: Option<(usize, u16)>, x| match b {
                Some(b) if b.0 >= x.0 => Some(b),
                _ => Some(x),
            });
        if let Some((_, oi)) = melhor {
            livre.remove(&oi);
            par.insert(hi, oi);
        }
    }
    par
}

/// `true` se os CONTORNOS mudaram de nº ou de lugar (apagar, inserir, reordenar). Leituras sem
/// contornos (forma paramétrica, caminho ausente) não contam.
fn contornos_mudaram(ontem: &BTreeMap<u16, Vec<Vert>>, hoje: &BTreeMap<u16, Vec<Vert>>) -> bool {
    if ontem.is_empty() || hoje.is_empty() {
        return false;
    }
    ontem.len() != hoje.len()
        || hoje.iter().any(|(&hi, h)| {
            !mesmas_ancoras(&ontem[&hi], h)
                && ontem.iter().any(|(&oi, o)| {
                    oi != hi
                        && mesmas_ancoras(o, h)
                        && !hoje.get(&oi).is_some_and(|x| mesmas_ancoras(o, x))
                })
        })
}

/// `true` se algum contorno mudou de nº de pontos, ou os contornos mudaram de nº ou de lugar — a
/// pergunta barata que decide se vale abrir um passo de undo para [`propagar`].
#[must_use]
pub fn mudou(antes: &BTreeMap<FieldKey, f32>, agora: &[(FieldKey, f32)]) -> bool {
    let (o, h) = (
        contornos(antes.iter().map(|(&kk, &v)| (kk, v))),
        contornos(agora.iter().copied()),
    );
    contornos_mudaram(&o, &h)
        || h.iter()
            .any(|(i, vs)| o.get(i).is_some_and(|m| m.len() != vs.len()))
}

/// O valor de `kk` na trilha da forma no instante `t` (`None` sem trilha ou sem chaves).
fn amostra(doc: &TimelineDoc, entidade: u64, kk: FieldKey, t: RationalTime) -> Option<f32> {
    match doc
        .binding_for(entidade, PropKind::Field(kk))
        .and_then(|b| doc.active_clip().track(b.target))
        .filter(|tr| !tr.is_empty())
        .map(|tr| tr.sample(t.to_seconds()))
    {
        Some(AnimValue::Float(x)) => Some(x),
        _ => None,
    }
}

/// O que [`propagar`] fez — para a sonda e a mensagem.
#[derive(Debug, Default, PartialEq)]
pub struct Propagacao {
    pub chaves: usize,
    /// Pontos (ou CONTORNOS, quando foram os contornos que mudaram) novos e apagados.
    pub novos: usize,
    pub apagados: usize,
}

/// As trilhas da forma (vértices + `FECHADO`) e os instantes das suas chaves.
type Forma = (
    Vec<(PropKind, ph2d_anim::AnimTarget)>,
    BTreeMap<RationalTime, Interp>,
);

fn forma_animada(doc: &TimelineDoc, entidade: u64) -> Option<Forma> {
    let forma: Vec<(PropKind, ph2d_anim::AnimTarget)> = doc
        .bindings()
        .iter()
        .filter(|b| b.entity == entidade)
        .filter_map(|b| match b.prop {
            PropKind::Field(kk)
                if kk.familia == VETOR && (c::ANCORA_X..=c::FECHADO).contains(&kk.campo) =>
            {
                Some((b.prop, b.target))
            }
            _ => None,
        })
        .collect();
    if forma.is_empty() {
        return None;
    }
    // Os instantes das chaves da forma e a interpolação de cada um.
    let mut tempos: BTreeMap<RationalTime, Interp> = BTreeMap::new();
    for &(_, tg) in &forma {
        if let Some(tr) = doc.active_clip().track(tg) {
            for key in tr.keys() {
                tempos.entry(key.t).or_insert(key.interp);
            }
        }
    }
    Some((forma, tempos))
}

fn fechado_em(lidos: &[(FieldKey, f32)], i: u16) -> Option<f32> {
    lidos
        .iter()
        .find(|(kk, _)| kk.familia == VETOR && kk.campo == c::FECHADO && kk.i == i)
        .map(|&(_, v)| v)
}

/// ⭐ **Leva a topologia de hoje a todas as chaves da forma de `entidade`.** `antes` = a leitura do
/// quadro anterior, `agora` = a deste. `None` se a forma não está animada ou nem os contornos nem o
/// nº de pontos mudaram.
pub fn propagar(
    doc: &mut TimelineDoc,
    entidade: u64,
    antes: &BTreeMap<FieldKey, f32>,
    agora: &[(FieldKey, f32)],
) -> Option<Propagacao> {
    let ontem = contornos(antes.iter().map(|(&kk, &v)| (kk, v)));
    let hoje = contornos(agora.iter().copied());
    if contornos_mudaram(&ontem, &hoje) {
        return propagar_contornos(doc, entidade, antes, agora, &ontem, &hoje);
    }
    let mudou: Vec<u16> = hoje
        .iter()
        .filter(|(i, vs)| ontem.get(i).is_some_and(|o| o.len() != vs.len()))
        .map(|(&i, _)| i)
        .collect();
    if mudou.is_empty() {
        return None;
    }
    let (forma, tempos) = forma_animada(doc, entidade)?;
    let fechado = |i: u16| fechado_em(agora, i).is_some_and(|v| v >= 0.5);
    let mut rel = Propagacao::default();
    for i in mudou {
        let (o, h) = (&ontem[&i], &hoje[&i]);
        let pares = alinhar(o, h);
        rel.novos += h.len() - pares.len();
        rel.apagados += o.len() - pares.len();
        for (&t, &interp) in &tempos {
            let na_chave: Vec<Vert> = (0..o.len())
                .map(|j| na_chave(doc, entidade, t, i, j, o[j]))
                .collect();
            let nova = refazer(o, h, &na_chave, &pares, fechado(i));
            for (j, v) in nova.iter().enumerate() {
                for campo in 0..N_CAMPOS {
                    let p = PropKind::Field(k(c::ANCORA_X + campo, i, n16(j)));
                    doc.upsert_key(
                        entidade,
                        p,
                        t,
                        AnimValue::Float(v[usize::from(campo)]),
                        interp,
                    );
                }
            }
        }
        // As trilhas dos índices que deixaram de existir saem.
        let sobra: BTreeSet<ph2d_anim::AnimTarget> = forma
            .iter()
            .filter(|(p, _)| matches!(p, PropKind::Field(kk) if kk.i == i && kk.campo != c::FECHADO && usize::from(kk.j) >= h.len()))
            .map(|&(_, tg)| tg)
            .collect();
        for tg in sobra {
            doc.purge_binding(tg);
        }
    }
    rel.chaves = tempos.len();
    Some(rel)
}

/// O vértice `(i, j)` da forma na chave `t`; o que não tem trilha fica com o valor de ontem.
fn na_chave(
    doc: &TimelineDoc,
    entidade: u64,
    t: RationalTime,
    i: u16,
    j: usize,
    ontem: Vert,
) -> Vert {
    let mut v = ontem;
    for campo in 0..N_CAMPOS {
        if let Some(x) = amostra(doc, entidade, k(c::ANCORA_X + campo, i, n16(j)), t) {
            v[usize::from(campo)] = x;
        }
    }
    v
}

/// ⭐ **Apagar, inserir ou reordenar um CONTORNO re-chaveia a forma inteira** (lei do dono 20/10):
/// cada contorno de hoje leva, em cada chave, os valores do seu par de ontem ([`casar`]); o que
/// também mudou de pontos passa pela mesma [`refazer`] do caso dos pontos; um contorno novo entra
/// com o valor de hoje; as trilhas dos `i`/`j` que deixaram de existir saem. Lê TODAS as amostras
/// antes de escrever: mover `1→0` e `2→1` em sítio sobrescreveria a fonte do seguinte.
fn propagar_contornos(
    doc: &mut TimelineDoc,
    entidade: u64,
    antes: &BTreeMap<FieldKey, f32>,
    agora: &[(FieldKey, f32)],
    ontem: &BTreeMap<u16, Vec<Vert>>,
    hoje: &BTreeMap<u16, Vec<Vert>>,
) -> Option<Propagacao> {
    let (forma, tempos) = forma_animada(doc, entidade)?;
    let par = casar(ontem, hoje);
    let lidos_antes: Vec<(FieldKey, f32)> = antes.iter().map(|(&kk, &v)| (kk, v)).collect();
    let mut escritas: Vec<Escrita> = Vec::new();
    for (&t, &interp) in &tempos {
        let mut vals = Vec::new();
        for (&hi, h) in hoje {
            let fechado_hoje = fechado_em(agora, hi).unwrap_or(0.0);
            let (verts, fechado) = match par.get(&hi) {
                Some(&oi) => {
                    let o = &ontem[&oi];
                    let na: Vec<Vert> = (0..o.len())
                        .map(|j| na_chave(doc, entidade, t, oi, j, o[j]))
                        .collect();
                    let fch = amostra(doc, entidade, k(c::FECHADO, oi, 0), t)
                        .or_else(|| fechado_em(&lidos_antes, oi))
                        .unwrap_or(fechado_hoje);
                    let verts = if mesmas_ancoras(o, h) {
                        na
                    } else {
                        refazer(o, h, &na, &alinhar(o, h), fechado_hoje >= 0.5)
                    };
                    (verts, fch)
                }
                None => (h.clone(), fechado_hoje),
            };
            vals.push((k(c::FECHADO, hi, 0), fechado));
            for (j, v) in verts.iter().enumerate() {
                for campo in 0..N_CAMPOS {
                    vals.push((k(c::ANCORA_X + campo, hi, n16(j)), v[usize::from(campo)]));
                }
            }
        }
        escritas.push((t, interp, vals));
    }
    for (t, interp, vals) in escritas {
        for (kk, v) in vals {
            doc.upsert_key(
                entidade,
                PropKind::Field(kk),
                t,
                AnimValue::Float(v),
                interp,
            );
        }
    }
    let sobra: BTreeSet<ph2d_anim::AnimTarget> = forma
        .iter()
        .filter(|(p, _)| {
            matches!(p, PropKind::Field(kk) if hoje.get(&kk.i).is_none_or(|h| {
                kk.campo != c::FECHADO && usize::from(kk.j) >= h.len()
            }))
        })
        .map(|&(_, tg)| tg)
        .collect();
    for tg in sobra {
        doc.purge_binding(tg);
    }
    Some(Propagacao {
        chaves: tempos.len(),
        novos: hoje.len() - par.len(),
        apagados: ontem.len() - par.len(),
    })
}

#[cfg(test)]
#[path = "canais_topologia_tests.rs"]
mod tests;
