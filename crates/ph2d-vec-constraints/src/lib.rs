//! O solver das regras de desenho técnico do vetor (W4.3): `docs/BlenderConstraints/04_W4_cad.md` §3.2.
//! Regra → `kcl_ezpz::Constraint` (a tradução da bancada, `ferramentas/bancada_solvers` `mod ez`); a ezpz é sem
//! estado, então o sistema remonta-se a cada chamada (§1.3: cabe nos 0,7 ms medidos).

#![forbid(unsafe_code)]

mod posto;

use std::collections::{BTreeMap, BTreeSet};

use kcl_ezpz::datatypes::inputs::{DatumCircle, DatumDistance, DatumLineSegment, DatumPoint};
use kcl_ezpz::datatypes::{Angle, AngleKind};
use kcl_ezpz::{Config, Constraint as K, ConstraintRequest as R, IdGenerator};
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, SegCad};
use ph2d_vec_scene::{VecPath, VecPathId, Xform};

pub use ph2d_vec_scene::cad::circulo_de;

/// Os paths que as regras endereçam, cada um com o seu afim local → folha.
pub struct Folha<'a> {
    paths: BTreeMap<VecPathId, (&'a VecPath, Xform)>,
}

impl<'a> Folha<'a> {
    pub fn new(paths: impl IntoIterator<Item = (&'a VecPath, Xform)>) -> Self {
        Self {
            paths: paths.into_iter().map(|(p, x)| (p.id, (p, x))).collect(),
        }
    }

    fn vert(&self, p: PontoCad) -> Option<[f64; 2]> {
        let PontoCad::Vert(id, v) = p else {
            return None;
        };
        let (path, xf) = self.paths.get(&id)?;
        Some(xf.apply(path.vert_by_id(v)?.anchor))
    }

    fn circulo(&self, id: VecPathId) -> Option<([f64; 2], f64)> {
        let (path, xf) = self.paths.get(&id)?;
        circulo_de(path, *xf)
    }
}

/// O ponto agarrado e onde o cursor o quer.
#[derive(Copy, Clone, Debug)]
pub struct Arrasto {
    pub ponto: PontoCad,
    pub alvo: [f64; 2],
}

/// Posições na FOLHA dos pontos resolvidos e o raio de cada círculo.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Solucao {
    pontos: BTreeMap<PontoCad, [f64; 2]>,
    raios: BTreeMap<VecPathId, f64>,
}

impl Solucao {
    /// A pose da folha tal como está (todo vértice, e centro + raio de todo path que é círculo): o palpite de quem
    /// parte do desenho.
    #[must_use]
    pub fn da_folha(folha: &Folha<'_>) -> Self {
        let mut s = Self::default();
        for (&id, (path, xf)) in &folha.paths {
            let todos = path
                .verts
                .iter()
                .chain(path.subpaths.iter().flat_map(|c| &c.verts));
            for v in todos {
                s.pontos
                    .insert(PontoCad::Vert(id, v.id), xf.apply(v.anchor));
            }
            if let Some((c, r)) = circulo_de(path, *xf) {
                s.pontos.insert(PontoCad::Centro(id), c);
                s.raios.insert(id, r);
            }
        }
        s
    }

    #[must_use]
    pub fn ponto(&self, p: PontoCad) -> Option<[f64; 2]> {
        self.pontos.get(&p).copied()
    }

    #[must_use]
    pub fn raio(&self, circulo: VecPathId) -> Option<f64> {
        self.raios.get(&circulo).copied()
    }

    /// Escreve no `path` (local, pelo inverso de `xf`) o que a solução diz dele: cada vértice resolvido anda com as
    /// duas alças pelo mesmo delta; um círculo resolvido re-escala à volta do centro novo, guardando ids e `kind`.
    /// `false` = nada dele foi resolvido (ou o afim não tem inverso).
    pub fn aplica(&self, path: &mut VecPath, xf: Xform) -> bool {
        let Some(inv) = xf.inverse() else {
            return false;
        };
        let id = path.id;
        let circulo = match (
            circulo_de(path, xf),
            self.ponto(PontoCad::Centro(id)),
            self.raio(id),
        ) {
            (Some((c0, r0)), Some(c1), Some(r1)) => Some((c0, r0, c1, r1)),
            _ => None,
        };
        let mut mexeu = false;
        let verts = path
            .verts
            .iter_mut()
            .chain(path.subpaths.iter_mut().flat_map(|c| &mut c.verts));
        for v in verts {
            if let Some((c0, r0, c1, r1)) = circulo {
                let k = r1 / r0;
                for q in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
                    let f = xf.apply(*q);
                    *q = inv.apply([c1[0] + k * (f[0] - c0[0]), c1[1] + k * (f[1] - c0[1])]);
                }
                mexeu = true;
            } else if let Some(novo) = self.ponto(PontoCad::Vert(id, v.id)) {
                let f = xf.apply(v.anchor);
                let d = inv.apply_vec([novo[0] - f[0], novo[1] - f[1]]);
                for q in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
                    *q = [q[0] + d[0], q[1] + d[1]];
                }
                mexeu = true;
            }
        }
        mexeu
    }
}

/// O que uma resolução devolve. `falta` = regras que a ezpz declara por cumprir (o «apague uma destas»);
/// `sem_alvo` = regras cujo ponto ou círculo não existe na folha (não entram no sistema).
#[derive(Clone, Debug)]
pub struct Resultado {
    pub solucao: Solucao,
    pub falta: Vec<usize>,
    pub sem_alvo: Vec<usize>,
}

/// A cor de uma regra.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EstadoRegra {
    Ok,
    /// Marcada REF: só mede.
    Ref,
    /// Não sobe o posto: a mais (uma medida assim nasce REF no editor).
    Redundante,
    Conflito,
    SemAlvo,
}

/// A cor de um ponto.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Liberdade {
    Livre,
    Definido,
}

#[derive(Clone, Debug)]
pub struct Estado {
    pub regras: Vec<EstadoRegra>,
    pub pontos: BTreeMap<PontoCad, Liberdade>,
}

/// O sistema da ezpz montado a partir da folha: um `DatumPoint` por ponto citado, um raio por círculo.
struct Sistema {
    pts: BTreeMap<PontoCad, DatumPoint>,
    raios: BTreeMap<VecPathId, DatumDistance>,
    valores: Vec<f64>,
    reqs: Vec<R>,
    origem: Vec<Option<usize>>,
    ativas: Vec<usize>,
    sem_alvo: Vec<usize>,
}

impl Sistema {
    fn monta(
        folha: &Folha<'_>,
        regras: &[RegraCad],
        arrasto: Option<Arrasto>,
        palpite: Option<&Solucao>,
    ) -> Self {
        // Um path citado como círculo é um datum: os vértices dele não são pontos soltos.
        let circulos: BTreeMap<VecPathId, ([f64; 2], f64)> = regras
            .iter()
            .flat_map(|r| r.lei.pontos())
            .filter_map(|p| match p {
                PontoCad::Centro(c) => folha.circulo(c).map(|g| (c, g)),
                PontoCad::Vert(..) => None,
            })
            .collect();
        let existe = |p: &PontoCad| match *p {
            PontoCad::Centro(c) => circulos.contains_key(&c),
            PontoCad::Vert(id, _) => !circulos.contains_key(&id) && folha.vert(*p).is_some(),
        };
        let (mut ativas, mut sem_alvo) = (Vec::new(), Vec::new());
        let mut citados = BTreeSet::new();
        for (i, r) in regras.iter().enumerate() {
            let ps = r.lei.pontos();
            if !ps.iter().all(existe) {
                sem_alvo.push(i);
            } else if !r.referencia {
                citados.extend(ps);
                ativas.push(i);
            }
        }
        if let Some(a) = arrasto.filter(|a| existe(&a.ponto)) {
            citados.insert(a.ponto);
        }
        let mut ids = IdGenerator::default();
        let pts: BTreeMap<PontoCad, DatumPoint> = citados
            .iter()
            .map(|&p| (p, DatumPoint::new(&mut ids)))
            .collect();
        let usados: BTreeSet<VecPathId> = citados
            .iter()
            .filter_map(|p| match *p {
                PontoCad::Centro(c) => Some(c),
                PontoCad::Vert(..) => None,
            })
            .collect();
        let raios: BTreeMap<VecPathId, DatumDistance> = usados
            .iter()
            .map(|&c| (c, DatumDistance::new(ids.next_id())))
            .collect();
        let mut valores = vec![0.0; ids.next_id() as usize];
        for (p, d) in &pts {
            let x = palpite
                .and_then(|s| s.ponto(*p))
                .or_else(|| match *p {
                    PontoCad::Centro(c) => circulos.get(&c).map(|g| g.0),
                    PontoCad::Vert(..) => folha.vert(*p),
                })
                .unwrap_or_default();
            valores[d.id_x() as usize] = x[0];
            valores[d.id_y() as usize] = x[1];
        }
        for (c, d) in &raios {
            valores[d.id as usize] = palpite.and_then(|s| s.raio(*c)).unwrap_or(circulos[c].1);
        }

        let seg = |s: SegCad| DatumLineSegment::new(pts[&s.0], pts[&s.1]);
        let circ = |c: VecPathId| DatumCircle {
            center: pts[&PontoCad::Centro(c)],
            radius: raios[&c],
        };
        let (mut reqs, mut origem) = (Vec::new(), Vec::new());
        for &i in &ativas {
            let ks = match regras[i].lei {
                Lei::Coincidente(a, b) => vec![K::PointsCoincident(pts[&a], pts[&b])],
                Lei::Horizontal(s) => vec![K::Horizontal(seg(s))],
                Lei::Vertical(s) => vec![K::Vertical(seg(s))],
                Lei::Paralelo(s, t) => vec![K::LinesAtAngle(seg(s), seg(t), AngleKind::Parallel)],
                Lei::Perpendicular(s, t) => {
                    vec![K::LinesAtAngle(seg(s), seg(t), AngleKind::Perpendicular)]
                }
                Lei::Angulo(s, t, a) => {
                    vec![K::LinesAtAngle(
                        seg(s),
                        seg(t),
                        AngleKind::Other(Angle::from_radians(a)),
                    )]
                }
                Lei::Tangente(s, c) => vec![K::LineTangentToCircle(seg(s), circ(c))],
                Lei::Igual(s, t) => vec![K::LinesEqualLength(seg(s), seg(t))],
                Lei::Simetrico(p, q, s) => vec![K::Symmetric(seg(s), pts[&p], pts[&q])],
                Lei::SobreRecta(p, s) => vec![K::PointLineDistance(pts[&p], seg(s), 0.0)],
                Lei::Distancia(a, b, d) => vec![K::Distance(pts[&a], pts[&b], d)],
                Lei::Raio(c, r) => vec![K::CircleRadius(circ(c), r)],
                Lei::Fixo(p, x) => vec![
                    K::Fixed(pts[&p].id_x(), x[0]),
                    K::Fixed(pts[&p].id_y(), x[1]),
                ],
            };
            for k in ks {
                reqs.push(R::highest_priority(k));
                origem.push(Some(i));
            }
        }
        if let Some(a) = arrasto
            && let Some(d) = pts.get(&a.ponto)
        {
            reqs.push(R::highest_priority(K::Fixed(d.id_x(), a.alvo[0])));
            reqs.push(R::highest_priority(K::Fixed(d.id_y(), a.alvo[1])));
            origem.extend([None, None]);
            valores[d.id_x() as usize] = a.alvo[0];
            valores[d.id_y() as usize] = a.alvo[1];
        }
        Self {
            pts,
            raios,
            valores,
            reqs,
            origem,
            ativas,
            sem_alvo,
        }
    }

    fn palpites(&self) -> Vec<(u32, f64)> {
        (0u32..).zip(self.valores.iter().copied()).collect()
    }

    fn solucao(&self, f: &[f64]) -> Solucao {
        Solucao {
            pontos: self
                .pts
                .iter()
                .map(|(p, d)| (*p, [f[d.id_x() as usize], f[d.id_y() as usize]]))
                .collect(),
            raios: self
                .raios
                .iter()
                .map(|(c, d)| (*c, f[d.id as usize]))
                .collect(),
        }
    }

    fn falta(&self, unsatisfied: &[usize]) -> Vec<usize> {
        let s: BTreeSet<usize> = unsatisfied.iter().filter_map(|&k| self.origem[k]).collect();
        s.into_iter().collect()
    }
}

/// Resolve as regras sobre a folha. `arrasto` prende o ponto agarrado ao cursor; `palpite` = a solução do passo
/// anterior (o arrasto) ou `None` = partir do desenho. Um arrasto impossível (o ponto está preso) não quebra regra:
/// o cursor perde a prioridade e o ponto fica onde as regras deixam.
#[must_use]
pub fn resolve(
    folha: &Folha<'_>,
    regras: &[RegraCad],
    arrasto: Option<Arrasto>,
    palpite: Option<&Solucao>,
) -> Resultado {
    let mut s = Sistema::monta(folha, regras, arrasto, palpite);
    let mut saida = kcl_ezpz::solve(&s.reqs, s.palpites(), Config::default());
    if arrasto.is_some() && saida.as_ref().map_or(true, |o| !o.is_satisfied()) {
        let n = s.reqs.len();
        for k in n.saturating_sub(2)..n {
            if s.origem[k].is_none() {
                s.reqs[k] = R::new(*s.reqs[k].constraint(), 1);
            }
        }
        saida = kcl_ezpz::solve(&s.reqs, s.palpites(), Config::default());
    }
    match saida {
        Ok(o) => Resultado {
            solucao: s.solucao(o.final_values()),
            falta: s.falta(o.unsatisfied()),
            sem_alvo: s.sem_alvo,
        },
        Err(_) => Resultado {
            solucao: s.solucao(&s.valores),
            falta: s.ativas.clone(),
            sem_alvo: s.sem_alvo,
        },
    }
}

/// As cores do CAD, partindo do desenho: conflito (`unsatisfied` da ezpz — a lista do Sketcher, §1.4), senão
/// redundante pelo posto em ordem de nascimento; e cada ponto citado livre ou definido. Só quando a ESTRUTURA muda.
#[must_use]
pub fn estado(folha: &Folha<'_>, regras: &[RegraCad]) -> Estado {
    let s = Sistema::monta(folha, regras, None, None);
    let mut cor: Vec<EstadoRegra> = regras
        .iter()
        .map(|r| {
            if r.referencia {
                EstadoRegra::Ref
            } else {
                EstadoRegra::Ok
            }
        })
        .collect();
    for &i in &s.sem_alvo {
        cor[i] = EstadoRegra::SemAlvo;
    }
    let mut pontos = BTreeMap::new();
    match kcl_ezpz::solve_analysis(&s.reqs, s.palpites(), Config::default()) {
        Err(_) => s
            .ativas
            .iter()
            .for_each(|&i| cor[i] = EstadoRegra::Conflito),
        Ok(o) => {
            let falta = s.falta(o.outcome.unsatisfied());
            falta.iter().for_each(|&i| cor[i] = EstadoRegra::Conflito);
            if falta.is_empty() {
                for i in redundantes(&s, regras, o.outcome.final_values()) {
                    cor[i] = EstadoRegra::Redundante;
                }
            }
            let livres: BTreeSet<u32> = o.analysis.underconstrained().iter().copied().collect();
            for (p, d) in &s.pts {
                let livre = livres.contains(&d.id_x()) || livres.contains(&d.id_y());
                pontos.insert(
                    *p,
                    if livre {
                        Liberdade::Livre
                    } else {
                        Liberdade::Definido
                    },
                );
            }
        }
    }
    Estado {
        regras: cor,
        pontos,
    }
}

/// `nova` não sobe o posto das `regras` na pose da folha ⇒ é a mais (o editor dá-lhe REF, se for medida). O teste de
/// posto sem a análise da ezpz: barato o bastante para correr quando uma guia acende.
#[must_use]
pub fn a_mais(folha: &Folha<'_>, regras: &[RegraCad], nova: RegraCad) -> bool {
    let mut todas = regras.to_vec();
    todas.push(RegraCad {
        referencia: false,
        ..nova
    });
    let n = todas.len() - 1;
    let s = Sistema::monta(folha, &todas, None, None);
    s.ativas.contains(&n) && redundantes(&s, &todas, &s.valores).contains(&n)
}

fn redundantes(s: &Sistema, regras: &[RegraCad], f: &[f64]) -> Vec<usize> {
    let valor = |v: &posto::Var| match *v {
        posto::Var::X(p) => f[s.pts[&p].id_x() as usize],
        posto::Var::Y(p) => f[s.pts[&p].id_y() as usize],
        posto::Var::Raio(c) => f[s.raios[&c].id as usize],
    };
    let global = |v: &posto::Var| match *v {
        posto::Var::X(p) => s.pts[&p].id_x() as usize,
        posto::Var::Y(p) => s.pts[&p].id_y() as usize,
        posto::Var::Raio(c) => s.raios[&c].id as usize,
    };
    let mut base = posto::Posto::default();
    let mut fora = Vec::new();
    for &i in &s.ativas {
        let (vars, res) = posto::residuos(&regras[i].lei, &valor);
        if !base.aceita(posto::linhas(&vars, &res, f.len(), &global)) {
            fora.push(i);
        }
    }
    fora
}

/// A ordem da recta que deixa o centro do círculo à ESQUERDA — o lado em que a tangente nasce e fica (a ezpz:
/// `cruz(p1 − p0, c − p1) / |v| = +r`). `None` se um dos pontos não existe, o path não é círculo ou a recta é um ponto.
#[must_use]
pub fn orienta_tangente(folha: &Folha<'_>, seg: SegCad, circulo: VecPathId) -> Option<SegCad> {
    let (c, _) = folha.circulo(circulo)?;
    let ponto = |p: PontoCad| match p {
        PontoCad::Centro(id) => folha.circulo(id).map(|g| g.0),
        PontoCad::Vert(..) => folha.vert(p),
    };
    let (a, b) = (ponto(seg.0)?, ponto(seg.1)?);
    let cruz = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    if a == b {
        None
    } else if cruz >= 0.0 {
        Some(seg)
    } else {
        Some(SegCad(seg.1, seg.0))
    }
}
