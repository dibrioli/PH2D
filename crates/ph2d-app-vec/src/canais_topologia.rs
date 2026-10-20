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
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as c;
use ph2d_ecs::FieldKey;
use ph2d_timeline::{PropKind, TimelineDoc};

/// Os 8 campos de um vértice, na ordem do vocabulário (`ANCORA_X ..= TIPO_VERTICE`).
type Vert = [f32; 8];
const N_CAMPOS: u16 = c::TIPO_VERTICE - c::ANCORA_X + 1;

fn k(campo: u16, i: u16, j: u16) -> FieldKey {
    FieldKey::new(VETOR, campo, i, j)
}

fn n16(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

/// Os contornos que uma leitura da família tem: `i` → os vértices, em ordem.
fn contornos(lidos: &[(FieldKey, f32)]) -> BTreeMap<u16, Vec<Vert>> {
    let mut out: BTreeMap<u16, BTreeMap<u16, Vert>> = BTreeMap::new();
    for &(kk, v) in lidos {
        if kk.familia == VETOR && (c::ANCORA_X..=c::TIPO_VERTICE).contains(&kk.campo) {
            out.entry(kk.i).or_default().entry(kk.j).or_insert([0.0; 8])
                [usize::from(kk.campo - c::ANCORA_X)] = v;
        }
    }
    out.into_iter().map(|(i, vs)| (i, vs.into_values().collect())).collect()
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
    let (q0, q1, q2) = (lerp(p[0], p[1], u), lerp(p[1], p[2], u), lerp(p[2], p[3], u));
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
    let mut melhor = (0..=64).map(|i| i as f32 / 64.0).fold(0.0, |b, u| if d(u) < d(b) { u } else { b });
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
fn refazer(ontem: &[Vert], hoje: &[Vert], na_chave: &[Vert], pares: &[(usize, usize)], fechado: bool) -> Vec<Vert> {
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
        let mut us: Vec<(f32, usize)> = (ini..b).map(|j| (projectar(s_ontem, [hoje[j][0], hoje[j][1]]), j)).collect();
        us.sort_by(|x, y| x.0.total_cmp(&y.0));
        let (mut va, mut vb) = (na_chave[oa], na_chave[ob]);
        let mut feito = 0.0;
        let mut novos = Vec::new();
        for (u, j) in us {
            let ur = if feito < 1.0 { ((u - feito) / (1.0 - feito)).clamp(0.0, 1.0) } else { 0.0 };
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
    out.into_iter().enumerate().map(|(j, v)| v.unwrap_or(hoje[j])).collect()
}

/// `true` se algum contorno mudou de nº de pontos entre as duas leituras — a pergunta barata que
/// decide se vale abrir um passo de undo para [`propagar`].
#[must_use]
pub fn mudou(antes: &BTreeMap<FieldKey, f32>, agora: &[(FieldKey, f32)]) -> bool {
    let conta = |it: &mut dyn Iterator<Item = FieldKey>| {
        let mut n: BTreeMap<u16, usize> = BTreeMap::new();
        for kk in it.filter(|kk| kk.familia == VETOR && kk.campo == c::ANCORA_X) {
            *n.entry(kk.i).or_default() += 1;
        }
        n
    };
    let (o, h) = (conta(&mut antes.keys().copied()), conta(&mut agora.iter().map(|&(kk, _)| kk)));
    h.iter().any(|(i, n)| o.get(i).is_some_and(|m| m != n))
}

/// O que [`propagar`] fez — para a sonda e a mensagem.
#[derive(Debug, Default, PartialEq)]
pub struct Propagacao {
    pub chaves: usize,
    pub novos: usize,
    pub apagados: usize,
}

/// ⭐ **Leva a topologia de hoje a todas as chaves da forma de `entidade`.** `antes` = a leitura do
/// quadro anterior, `agora` = a deste. `None` se a forma não está animada ou o nº de pontos não
/// mudou.
pub fn propagar(
    doc: &mut TimelineDoc,
    entidade: u64,
    antes: &BTreeMap<FieldKey, f32>,
    agora: &[(FieldKey, f32)],
) -> Option<Propagacao> {
    let ontem = contornos(&antes.iter().map(|(&kk, &v)| (kk, v)).collect::<Vec<_>>());
    let hoje = contornos(agora);
    let mudou: Vec<u16> = hoje
        .iter()
        .filter(|(i, vs)| ontem.get(i).is_some_and(|o| o.len() != vs.len()))
        .map(|(&i, _)| i)
        .collect();
    if mudou.is_empty() {
        return None;
    }
    let forma: Vec<(PropKind, ph2d_anim::AnimTarget)> = doc
        .bindings()
        .iter()
        .filter(|b| b.entity == entidade)
        .filter_map(|b| match b.prop {
            PropKind::Field(kk) if kk.familia == VETOR && (c::ANCORA_X..=c::FECHADO).contains(&kk.campo) => {
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
    let fechado = |i: u16| agora.iter().find(|(kk, _)| kk.campo == c::FECHADO && kk.i == i).is_some_and(|&(_, v)| v >= 0.5);
    let mut rel = Propagacao::default();
    for i in mudou {
        let (o, h) = (&ontem[&i], &hoje[&i]);
        let pares = alinhar(o, h);
        rel.novos += h.len() - pares.len();
        rel.apagados += o.len() - pares.len();
        for (&t, &interp) in &tempos {
            let na_chave: Vec<Vert> = (0..o.len())
                .map(|j| {
                    let mut v = o[j];
                    for campo in 0..N_CAMPOS {
                        let p = PropKind::Field(k(c::ANCORA_X + campo, i, n16(j)));
                        if let Some(AnimValue::Float(x)) = doc
                            .binding_for(entidade, p)
                            .and_then(|b| doc.active_clip().track(b.target))
                            .filter(|tr| !tr.is_empty())
                            .map(|tr| tr.sample(t.to_seconds()))
                        {
                            v[usize::from(campo)] = x;
                        }
                    }
                    v
                })
                .collect();
            let nova = refazer(o, h, &na_chave, &pares, fechado(i));
            for (j, v) in nova.iter().enumerate() {
                for campo in 0..N_CAMPOS {
                    let p = PropKind::Field(k(c::ANCORA_X + campo, i, n16(j)));
                    doc.upsert_key(entidade, p, t, AnimValue::Float(v[usize::from(campo)]), interp);
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

#[cfg(test)]
#[path = "canais_topologia_tests.rs"]
mod tests;
