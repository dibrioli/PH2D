//! O grafo de colas: grupos (componentes conexos), sólidos (as colas rígidas contraídas num corpo só) e o fecho
//! analítico da árvore de pinos com um condutor (05 §2.3).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::construcao::{self, PinI};
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

type P = [f64; 2];

/// Um membro no seu sólido: o índice da raiz e, se não é ela, a pose na raiz com a inversa (`None` = colapsa).
#[derive(Clone, Copy)]
struct NoSolido {
    raiz: usize,
    rel: Option<(Pose2, Option<Pose2>)>,
}

enum Via {
    /// Árvore de pinos com um condutor: `(k, ponto em k, f, ponto em f)` pela ordem da largura.
    Fecho(Vec<(usize, P, usize, P)>),
    Construcao(Vec<construcao::Passo>),
    Ezpz,
}

/// O que não muda enquanto as colas, os objectos e quem conduz não mudam (05 §4.2) — vai na cache do passo
/// do mundo; o quadro só faz a conta.
pub(crate) struct Preparo {
    membros: Vec<Corpo>,
    /// A posição de cada membro e de cada raiz nas entradas (por ordem do `Corpo`).
    ix: Vec<usize>,
    raiz_ix: Vec<usize>,
    raizes: Vec<Corpo>,
    no: Vec<NoSolido>,
    conduz: Vec<bool>,
    mao: Vec<bool>,
    fixa: Vec<bool>,
    alvo: Vec<bool>,
    /// Um pino com as duas pontas no mesmo sólido que não coincidem: conflito, se alguém conduz.
    pino_partido: bool,
    pins: Vec<PinI>,
    via: Via,
}

/// `ceder`: a mão do gesto deixa de ser condutor e vira ALVO de prioridade menor na ezpz.
pub(crate) fn prepara(
    membros: &[Corpo],
    colas: &[Cola],
    corpos: &BTreeMap<Corpo, Entrada>,
    rank: &BTreeMap<Corpo, usize>,
    ceder: bool,
) -> Result<Preparo, ()> {
    let mao_c = |c: Corpo| ceder && corpos[&c].condutor && corpos[&c].cede;
    let em = solidos(membros, colas, |c| corpos[&c].condutor)?;
    let raizes: Vec<Corpo> = em
        .values()
        .map(|v| v.0)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let ri = |c: Corpo| raizes.binary_search(&c).map_err(|_| ());
    let mut no = Vec::with_capacity(membros.len());
    for m in membros {
        let (r, rel) = em[m];
        no.push(NoSolido {
            raiz: ri(r)?,
            rel: rel.map(|rel| (rel, Pose2::IDENTITY.relative_to(rel))),
        });
    }
    let mao: Vec<bool> = membros.iter().map(|&m| mao_c(m)).collect();
    let conduz: Vec<bool> = membros
        .iter()
        .zip(&mao)
        .map(|(m, &h)| corpos[m].condutor && !h)
        .collect();
    let (mut fixa, mut alvo) = (vec![false; raizes.len()], vec![false; raizes.len()]);
    for (i, n) in no.iter().enumerate() {
        if conduz[i] {
            fixa[n.raiz] = true;
        } else if mao[i] {
            alvo[n.raiz] = true;
        }
    }
    for (a, f) in alvo.iter_mut().zip(&fixa) {
        *a &= !f;
    }

    let na_raiz = |c: Corpo, q: P| {
        let (r, rel) = em[&c];
        (r, rel.map_or(q, |p| p.ponto(q)))
    };
    let mut pins = Vec::new();
    let mut pino_partido = false;
    for c in colas {
        if let Tipo::Pino { em_a, em_b } = c.tipo {
            let (a, qa) = na_raiz(c.a, em_a);
            let (b, qb) = na_raiz(c.b, em_b);
            if a == b {
                if (qa[0] - qb[0]).hypot(qa[1] - qb[1]) > TOL * (1.0 + qa[0].abs().max(qa[1].abs()))
                {
                    pino_partido = true;
                    break;
                }
                continue;
            }
            pins.push(PinI {
                a: ri(a)?,
                qa,
                b: ri(b)?,
                qb,
            });
        }
    }

    let sem_alvo = !alvo.contains(&true);
    let n_fixo = fixa.iter().filter(|&&f| f).count();
    let via = if sem_alvo && n_fixo == 1 && pins.len() + 1 == raizes.len() {
        let inicio = fixa.iter().position(|&f| f).ok_or(())?;
        Via::Fecho(ordem_do_fecho(&pins, inicio, raizes.len()))
    } else if sem_alvo {
        construcao::receita(&fixa, &pins).map_or(Via::Ezpz, Via::Construcao)
    } else {
        Via::Ezpz
    };
    Ok(Preparo {
        membros: membros.to_vec(),
        ix: membros.iter().map(|m| rank[m]).collect(),
        raiz_ix: raizes.iter().map(|r| rank[r]).collect(),
        raizes,
        no,
        conduz,
        mao,
        fixa,
        alvo,
        pino_partido,
        pins,
        via,
    })
}

/// Os vectores de um grupo, reutilizados de grupo em grupo (a conta é por quadro).
#[derive(Default)]
pub(crate) struct Rascunho {
    fixo: Vec<Option<Pose2>>,
    alvo: Vec<Option<Pose2>>,
    pose: Vec<Pose2>,
    palpite: Vec<Pose2>,
}

/// `Ok(false)` = ninguém conduz; `Err` = conflito; `Ok(true)` = a pose mundial de cada membro foi para `out`.
/// `poses`: a de cada entrada, por ordem do `Corpo`.
pub(crate) fn executa(
    p: &Preparo,
    poses: &[Pose2],
    r: &mut Rascunho,
    out: &mut Vec<(Corpo, Pose2)>,
) -> Result<bool, ()> {
    let n = p.raizes.len();
    let Rascunho {
        fixo,
        alvo,
        pose,
        palpite,
    } = r;
    fixo.clear();
    fixo.resize(n, None);
    alvo.clear();
    alvo.resize(n, None);
    for (i, no) in p.no.iter().enumerate() {
        if !(p.conduz[i] || p.mao[i]) {
            continue;
        }
        let pm = poses[p.ix[i]];
        let pr = match no.rel {
            None => pm,
            Some((_, inv)) => inv.ok_or(())?.under(pm),
        };
        if p.mao[i] {
            alvo[no.raiz] = Some(pr);
            continue;
        }
        match fixo[no.raiz] {
            Some(q) if !perto(q, pr) => return Err(()),
            Some(_) => {}
            None => fixo[no.raiz] = Some(pr),
        }
    }
    if fixo.iter().all(Option::is_none) {
        return Ok(false);
    }
    if p.pino_partido {
        return Err(());
    }
    pose.clear();
    pose.extend((0..n).map(|r| fixo[r].or(alvo[r]).unwrap_or(poses[p.raiz_ix[r]])));
    match &p.via {
        Via::Fecho(passos) => {
            for &(k, qk, f, qf) in passos {
                let alvo = pose[k].ponto(qk);
                let agora = pose[f].ponto(qf);
                pose[f].x += alvo[0] - agora[0];
                pose[f].y += alvo[1] - agora[1];
            }
        }
        Via::Construcao(passos) => {
            palpite.clone_from(pose);
            if !construcao::executa(passos, pose, &p.pins) {
                *pose = pela_ezpz(p, palpite)?;
            }
        }
        Via::Ezpz => *pose = pela_ezpz(p, pose)?,
    }
    out.extend(
        p.membros
            .iter()
            .zip(&p.no)
            .enumerate()
            .map(|(i, (&m, no))| {
                let q = if p.conduz[i] {
                    poses[p.ix[i]]
                } else {
                    no.rel
                        .map_or(pose[no.raiz], |(rel, _)| rel.under(pose[no.raiz]))
                };
                (m, q)
            }),
    );
    Ok(true)
}

fn pela_ezpz(p: &Preparo, palpite: &[Pose2]) -> Result<Vec<Pose2>, ()> {
    let pose: BTreeMap<Corpo, Pose2> = p
        .raizes
        .iter()
        .copied()
        .zip(palpite.iter().copied())
        .collect();
    let i = |c: Corpo| p.raizes.binary_search(&c).unwrap_or(0);
    let pins: Vec<Pin> = p
        .pins
        .iter()
        .map(|q| Pin {
            a: p.raizes[q.a],
            qa: q.qa,
            b: p.raizes[q.b],
            qb: q.qb,
        })
        .collect();
    let fixo_ou_alvo = |c: Corpo| p.fixa[i(c)] || p.alvo[i(c)];
    let saida = ez::resolve(&pose, fixo_ou_alvo, |c| p.alvo[i(c)], &pins).ok_or(())?;
    Ok(p.raizes.iter().map(|r| saida[r]).collect())
}

/// Árvore de pinos com um condutor: cada seguidor mantém a SUA rotação e escala e translada até o pino coincidir
/// — a ordem (largura a partir do condutor) é estrutural.
fn ordem_do_fecho(pins: &[PinI], inicio: usize, n: usize) -> Vec<(usize, P, usize, P)> {
    let mut feito = vec![false; n];
    feito[inicio] = true;
    let mut fila = VecDeque::from([inicio]);
    let mut passos = Vec::new();
    while let Some(k) = fila.pop_front() {
        for pin in pins {
            let (qk, f, qf) = if pin.a == k {
                (pin.qa, pin.b, pin.qb)
            } else if pin.b == k {
                (pin.qb, pin.a, pin.qa)
            } else {
                continue;
            };
            if feito[f] {
                continue;
            }
            feito[f] = true;
            passos.push((k, qk, f, qf));
            fila.push_back(f);
        }
    }
    passos
}
