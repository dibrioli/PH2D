//! O grafo de colas: grupos (componentes conexos), sólidos (as colas rígidas contraídas num corpo só) e o fecho
//! analítico da árvore de pinos com um condutor (05 §2.3).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::ez::{self, Pin};
use crate::{Cola, Corpo, Entrada, Pose2, TOL, Tipo, perto};

pub(crate) struct GrupoBruto {
    pub membros: Vec<Corpo>,
    pub colas: Vec<usize>,
}

fn raiz(pai: &mut BTreeMap<Corpo, Corpo>, c: Corpo) -> Corpo {
    let mut r = c;
    while let Some(&p) = pai.get(&r) {
        if p == r {
            break;
        }
        r = p;
    }
    let mut x = c;
    while x != r {
        let p = pai[&x];
        pai.insert(x, r);
        x = p;
    }
    r
}

/// Os componentes conexos, na ordem do menor membro; as colas que não ligam dois objectos existentes ficam fora.
pub(crate) fn grupos(colas: &[Cola], existe: impl Fn(Corpo) -> bool) -> Vec<GrupoBruto> {
    let validas: Vec<usize> = (0..colas.len())
        .filter(|&i| colas[i].a != colas[i].b && existe(colas[i].a) && existe(colas[i].b))
        .collect();
    let mut pai = BTreeMap::new();
    for &i in &validas {
        let c = colas[i];
        pai.entry(c.a).or_insert(c.a);
        pai.entry(c.b).or_insert(c.b);
        let (ra, rb) = (raiz(&mut pai, c.a), raiz(&mut pai, c.b));
        pai.insert(ra.max(rb), ra.min(rb));
    }
    let mut por: BTreeMap<Corpo, GrupoBruto> = BTreeMap::new();
    let membros: Vec<Corpo> = pai.keys().copied().collect();
    for m in membros {
        let r = raiz(&mut pai, m);
        por.entry(r)
            .or_insert_with(|| GrupoBruto {
                membros: Vec::new(),
                colas: Vec::new(),
            })
            .membros
            .push(m);
    }
    for &i in &validas {
        let r = raiz(&mut pai, colas[i].a);
        if let Some(g) = por.get_mut(&r) {
            g.colas.push(i);
        }
    }
    por.into_values().collect()
}

/// Cada membro → (a raiz do seu sólido, a sua pose na raiz; `None` = é a raiz). A raiz é um condutor quando o
/// sólido tem um, para que ele e os seus vizinhos directos saiam ao bit. Solda que não fecha ⇒ `Err`.
type Solidos = BTreeMap<Corpo, (Corpo, Option<Pose2>)>;

fn solidos(
    membros: &[Corpo],
    colas: &[Cola],
    condutor: impl Fn(Corpo) -> bool,
) -> Result<Solidos, ()> {
    let mut adj: BTreeMap<Corpo, Vec<(usize, Corpo, Pose2)>> = BTreeMap::new();
    for (i, c) in colas.iter().enumerate() {
        if let Tipo::Rigida { rel } = c.tipo {
            let inv = Pose2::IDENTITY.relative_to(rel).ok_or(())?;
            adj.entry(c.a).or_default().push((i, c.b, rel));
            adj.entry(c.b).or_default().push((i, c.a, inv));
        }
    }
    // Só a aresta que FECHA um ciclo se confere: a volta pela aresta de onde se veio não prova nada e, com escala
    // não uniforme rodada, a `Pose2` sem cisalhamento não faz a ida e volta ao bit.
    let mut usada = BTreeSet::new();
    let sementes = membros
        .iter()
        .filter(|&&m| condutor(m))
        .chain(membros.iter().filter(|&&m| !condutor(m)));
    let mut em: Solidos = BTreeMap::new();
    for &s in sementes {
        if em.contains_key(&s) {
            continue;
        }
        em.insert(s, (s, None));
        let mut fila = VecDeque::from([s]);
        while let Some(x) = fila.pop_front() {
            let px = em[&x].1;
            for &(i, y, r) in adj.get(&x).map_or(&[][..], Vec::as_slice) {
                if !usada.insert(i) {
                    continue;
                }
                let cand = px.map_or(r, |p| r.under(p));
                match em.get(&y) {
                    Some(&(_, py)) => {
                        if !perto(py.unwrap_or(Pose2::IDENTITY), cand) {
                            return Err(());
                        }
                    }
                    None => {
                        em.insert(y, (s, Some(cand)));
                        fila.push_back(y);
                    }
                }
            }
        }
    }
    Ok(em)
}

/// `Ok(None)` = ninguém conduz; `Err` = conflito; senão a pose mundial de cada membro.
pub(crate) fn resolve_grupo(
    membros: &[Corpo],
    colas: &[Cola],
    corpos: &BTreeMap<Corpo, Entrada>,
) -> Result<Option<BTreeMap<Corpo, Pose2>>, ()> {
    let conduz = |c: Corpo| corpos[&c].condutor;
    let em = solidos(membros, colas, conduz)?;

    let mut fixo: BTreeMap<Corpo, Pose2> = BTreeMap::new();
    for &m in membros.iter().filter(|&&m| conduz(m)) {
        let (r, rel) = em[&m];
        let p = corpos[&m].pose;
        let pr = match rel {
            None => p,
            Some(rel) => Pose2::IDENTITY.relative_to(rel).ok_or(())?.under(p),
        };
        match fixo.get(&r) {
            Some(&q) if !perto(q, pr) => return Err(()),
            Some(_) => {}
            None => {
                fixo.insert(r, pr);
            }
        }
    }
    if fixo.is_empty() {
        return Ok(None);
    }

    let na_raiz = |c: Corpo, q: [f64; 2]| {
        let (r, rel) = em[&c];
        (r, rel.map_or(q, |p| p.ponto(q)))
    };
    let mut pins = Vec::new();
    for c in colas {
        if let Tipo::Pino { em_a, em_b } = c.tipo {
            let (a, qa) = na_raiz(c.a, em_a);
            let (b, qb) = na_raiz(c.b, em_b);
            if a == b {
                if (qa[0] - qb[0]).hypot(qa[1] - qb[1]) > TOL * (1.0 + qa[0].abs().max(qa[1].abs()))
                {
                    return Err(());
                }
                continue;
            }
            pins.push(Pin { a, qa, b, qb });
        }
    }

    let raizes: BTreeSet<Corpo> = em.values().map(|v| v.0).collect();
    let mut pose: BTreeMap<Corpo, Pose2> = raizes
        .iter()
        .map(|&r| (r, fixo.get(&r).copied().unwrap_or(corpos[&r].pose)))
        .collect();
    if fixo.len() == 1 && pins.len() + 1 == raizes.len() {
        let inicio = *fixo.keys().next().ok_or(())?;
        fecho(&mut pose, &pins, inicio);
    } else {
        pose = ez::resolve(&pose, |c| fixo.contains_key(&c), &pins).ok_or(())?;
    }

    Ok(Some(
        membros
            .iter()
            .map(|&m| {
                let (r, rel) = em[&m];
                let p = if conduz(m) {
                    corpos[&m].pose
                } else {
                    rel.map_or(pose[&r], |rel| rel.under(pose[&r]))
                };
                (m, p)
            })
            .collect(),
    ))
}

/// Árvore de pinos com um condutor: cada seguidor mantém a SUA rotação e escala e translada até o pino coincidir.
fn fecho(pose: &mut BTreeMap<Corpo, Pose2>, pins: &[Pin], inicio: Corpo) {
    let mut feito = BTreeSet::from([inicio]);
    let mut fila = VecDeque::from([inicio]);
    while let Some(k) = fila.pop_front() {
        for pin in pins {
            let (qk, f, qf) = if pin.a == k {
                (pin.qa, pin.b, pin.qb)
            } else if pin.b == k {
                (pin.qb, pin.a, pin.qa)
            } else {
                continue;
            };
            if !feito.insert(f) {
                continue;
            }
            let alvo = pose[&k].ponto(qk);
            let p = pose
                .get_mut(&f)
                .expect("todo pino liga duas raízes do grupo");
            let agora = p.ponto(qf);
            p.x += alvo[0] - agora[0];
            p.y += alvo[1] - agora[1];
            fila.push_back(f);
        }
    }
}
