//! ⭐⭐⭐ **OS FECHOS PLANOS SÃO UM SISTEMA LINEAR — e resolvem-se por SUBSTITUIÇÃO.**
//!
//! # Por que este módulo existe, e a razão é MEDIDA
//!
//! Um fecho **plano** (`turn = 0`) diz `off_b − R^k·off_a − t_s = 0`. ⭐ O termo da
//! classe **cancela-se** — é uma condição só sobre as **translações**. Ela não é
//! opcional: é o que faz o mapa ser de valor único à volta de um canto regular, e sem
//! ela a costura rasga por **8 a 18 células** (medido nas três peças).
//!
//! ⛔⛔ **Duas maneiras de a impor foram construídas e REJEITADAS por medição:**
//!
//! | tentativa | o que dá |
//! |---|---|
//! | escrever `t_s` a partir do fecho a cada varredura (alternância) | o toro vai a `568` de resíduo e o passo a `2,9e-1` — não assenta |
//! | pôr `t` no subespaço de **calibre** (`t_s = o_b − R^k·o_a`) | ângulo `60°`–`85°`, escala `0,22`–`0,50` — o subespaço é **pequeno demais** (um ciclo que envolve singularidades tem holonomia legítima) |
//!
//! ⇒ *A condição é uma equação; impô-la por projecção alternada não é impô-la.* A
//! terceira via é a que o próprio *paper* de 2009 descreve: **eliminar uma variável por
//! restrição independente**.
//!
//! # Como
//!
//! `off_c` é uma forma linear nas translações ([`Weld::cross_of`]), logo cada fecho
//! plano é `Σ_s C_s · t_s = 0` com `C_s` matrizes `2×2` de inteiros (somas de rotações).
//! Cada equação **possui** uma costura — uma em que o coeficiente é invertível — e
//! escreve-a:
//!
//! ```text
//!     t_dono = −C_dono⁻¹ · Σ_outros C_o · t_o
//! ```
//!
//! ⭐ A substituição corre em **ordem topológica**, e é por isso que ela assenta de uma
//! vez em vez de alternar. ⚠️ *Se o grafo de dependência tiver ciclo, isso é um facto
//! sobre a peça e é **contado** ([`FlatReport::cyclic`]), nunca escondido.*
//!
//! ⭐⭐⭐ **E as duas espécies de fecho entram no MESMO sistema.** Tratá-las em dois
//! subsistemas — um a escrever `t` a partir de `y`, o outro a escrever `t` a partir de
//! `t` — cria uma realimentação `t_sing = M·y + f(t_sing)` cujo ganho é maior que `1`:
//! medido, a esfera vai a **NaN** e o toro a `6,4e17`, e ⛔ **amortecer não cura**
//! (`1,0` · `0,5` · `0,25` · `0,125` · `0,0625` divergem todos). *Duas eliminações que
//! leem o que a outra escreve não são duas eliminações.*
//!
//! ⭐⭐ **A INTEGRALIDADE tem um preço nomeado:** a substituição leva inteiros a
//! inteiros **se e só se** `det C_dono = ±1`. O construtor prefere esses; quando não há,
//! ele conta-o ([`FlatReport::worst_det`]) — *um determinante `2` é meia célula, que é
//! exactamente o meio-inteiro que a modalidade das singularidades já tinha encontrado.*

use crate::solve::{GridMap, turn2};
use crate::weld::Weld;

/// Uma matriz `2×2`.
type M2 = [[f32; 2]; 2];

const ZERO: M2 = [[0.0, 0.0], [0.0, 0.0]];

/// `R^m` como matriz.
fn rot(m: i32) -> M2 {
    let e0 = turn2([1.0, 0.0], m);
    let e1 = turn2([0.0, 1.0], m);
    [[e0[0], e1[0]], [e0[1], e1[1]]]
}

fn add(a: M2, b: M2) -> M2 {
    [
        [a[0][0] + b[0][0], a[0][1] + b[0][1]],
        [a[1][0] + b[1][0], a[1][1] + b[1][1]],
    ]
}

fn sub(a: M2, b: M2) -> M2 {
    [
        [a[0][0] - b[0][0], a[0][1] - b[0][1]],
        [a[1][0] - b[1][0], a[1][1] - b[1][1]],
    ]
}

fn mul_vec(m: M2, v: [f32; 2]) -> [f32; 2] {
    [
        m[0][0].mul_add(v[0], m[0][1] * v[1]),
        m[1][0].mul_add(v[0], m[1][1] * v[1]),
    ]
}

fn det(m: M2) -> f32 {
    m[0][0].mul_add(m[1][1], -(m[0][1] * m[1][0]))
}

fn inv(m: M2) -> Option<M2> {
    let d = det(m);
    (d.abs() > 1.0e-6).then(|| [[m[1][1] / d, -m[0][1] / d], [-m[1][0] / d, m[0][0] / d]])
}

/// O que o sistema plano mediu de si próprio.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FlatReport {
    /// Condições de fecho plano ao todo.
    pub equations: usize,
    /// ⭐ Quantas eliminaram uma translação.
    pub resolved: usize,
    /// ⛔ Quantas ficaram sem variável para eliminar.
    pub orphans: usize,
    /// ⚠️ A substituição tem ciclo? (então ela itera em vez de assentar de uma vez)
    pub cyclic: bool,
    /// ⭐ O pior `|det|` de um pivô — `1` é o que preserva inteiros.
    pub worst_det: f32,
}

/// Uma incógnita do sistema: a translação de uma costura, ou a imagem de um vértice
/// singular.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Var {
    /// A translação de uma costura.
    Shift(u32),
    /// A imagem da classe de um vértice singular.
    Class(u32),
}

/// ⭐⭐⭐ **O SISTEMA DOS FECHOS**, já eliminado.
#[derive(Debug, Clone, Default)]
pub struct ClosureSystem {
    /// As incógnitas que sobraram livres.
    free: Vec<Var>,
    /// Por dependente, a expressão dela nas livres.
    dep: Vec<(Var, Vec<(u32, M2)>)>,
    /// ⭐⭐⭐ **Por dependente, que COMPONENTES o sistema escreve.**
    ///
    /// `[true, true]` é a eliminação de sempre — uma restrição `2×2` possui a variável
    /// inteira. ⭐ **`[true, false]` (ou o inverso) é uma restrição ESCALAR**: ela possui
    /// **meia** variável, e a outra metade continua livre.
    ///
    /// ⚠️ **Por que a expressão continua a ser uma `M2`, e não uma linha:** uma matriz
    /// com a linha não-escrita a **zeros** dá o mesmo valor e deixa o
    /// [`ClosureSystem::bump`] correcto **sem uma linha de código mudar** — ele soma
    /// `mul_vec(a, Δ)`, e um zero exacto soma zero. *A alternativa (mudar o contentor)
    /// tocaria em quatro sítios para o mesmo resultado, e nenhum deles ficaria mais fácil
    /// de ler.*
    ///
    /// ⛔ Quem **precisa** de saber é o [`ClosureSystem::apply`], que escreve o valor
    /// **absoluto**: sem a máscara ele zeraria a metade que não possui.
    dep_axes: Vec<[bool; 2]>,
    /// Por livre, as cópias que ela move e a jacobiana `∂z/∂v`.
    touch: Vec<Vec<(u32, M2)>>,
    /// ⭐ Por livre, as dependentes que ela move — pré-computado.
    ///
    /// ⚠️ **Sem isto cada `bump` varre TODAS as dependentes** à procura de si próprio, e
    /// o passo do guloso passa a custar `O(dependentes)` em vez de `O(as que dependem
    /// desta)`. *Medido: a cadeia da peça enrugada estava em 60 s.*
    by_free: Vec<Vec<(u32, M2)>>,
    /// Por costura, `true` se ela é dependente.
    dep_seam: Vec<bool>,
    /// Por classe singular, `true` se ela é dependente.
    dep_class: std::collections::BTreeSet<u32>,
}

impl ClosureSystem {
    /// As incógnitas livres.
    #[must_use]
    pub fn free(&self) -> &[Var] {
        &self.free
    }

    /// As cópias que uma incógnita livre move, com a jacobiana de cada uma.
    #[must_use]
    pub fn touched(&self, i: usize) -> &[(u32, M2)] {
        self.touch.get(i).map_or(&[], Vec::as_slice)
    }

    /// A translação desta costura é escrita pelo sistema?
    #[must_use]
    pub fn is_dependent_seam(&self, seam: usize) -> bool {
        self.dep_seam.get(seam).copied().unwrap_or(false)
    }

    /// A imagem desta classe é escrita pelo sistema?
    #[must_use]
    pub fn is_dependent_class(&self, class: usize) -> bool {
        u32::try_from(class).is_ok_and(|c| self.dep_class.contains(&c))
    }

    /// ⭐⭐⭐ **MEXE uma livre e propaga o efeito** — incremental, e exacto.
    ///
    /// ⚠️ **Exacto porque a dependência é LINEAR:** se a livre `i` anda `Δ`, cada
    /// dependente anda `A·Δ`, com `A` o coeficiente dela na expressão. *Reconstruir
    /// todas as dependentes a cada passo dá o mesmo número e custa `O(dependentes)` por
    /// relaxação — medido, era o que punha a cadeia em 69 s.*
    pub fn bump(&self, w: &Weld, map: &mut GridMap, i: usize, delta: [f32; 2]) {
        let write = |v: Var, d: [f32; 2], map: &mut GridMap| match v {
            Var::Shift(s) => {
                map.shift[s as usize][0] += d[0];
                map.shift[s as usize][1] += d[1];
                for &cl in w.shift_classes_pub(s as usize) {
                    w.derive(map, cl as usize);
                }
            }
            Var::Class(c) => {
                let y = w.value_pub(map, c as usize);
                w.set(map, c as usize, [y[0] + d[0], y[1] + d[1]]);
            }
        };
        let Some(&v) = self.free.get(i) else { return };
        write(v, delta, map);
        let Some(list) = self.by_free.get(i) else {
            return;
        };
        for &(d, a) in list {
            write(self.dep[d as usize].0, mul_vec(a, delta), map);
        }
    }

    /// ⭐ **ESCREVE as dependentes** a partir das livres, e re-deriva o que se mexeu.
    ///
    /// ⚠️ **Uma passagem chega, e não é sorte:** cada dependente está expressa nas
    /// LIVRES (a substituição já foi feita na construção), então nenhuma delas lê outra
    /// dependente. *Era isso que faltava às duas versões que alternavam.*
    pub fn apply(&self, w: &Weld, map: &mut GridMap) {
        for (k, (v, expr)) in self.dep.iter().enumerate() {
            let mut acc = [0.0f32; 2];
            for &(i, c) in expr {
                let val = match self.free[i as usize] {
                    Var::Shift(s) => map.shift[s as usize],
                    Var::Class(cl) => w.value_pub(map, cl as usize),
                };
                let p = mul_vec(c, val);
                acc[0] += p[0];
                acc[1] += p[1];
            }
            // ⭐ A máscara: uma dependente ESCALAR possui meia variável, e a outra metade
            // fica onde estava. ⚠️ *Escrever `acc` inteiro ali zeraria uma incógnita que
            // ninguém pediu para mexer* — e o mapa sairia mudo sobre o porquê.
            let axes = self.dep_axes.get(k).copied().unwrap_or([true, true]);
            if !(axes[0] && axes[1]) {
                let now = match *v {
                    Var::Shift(s) => map.shift[s as usize],
                    Var::Class(cl) => w.value_pub(map, cl as usize),
                };
                if !axes[0] {
                    acc[0] = now[0];
                }
                if !axes[1] {
                    acc[1] = now[1];
                }
            }
            match *v {
                Var::Shift(s) => {
                    map.shift[s as usize] = acc;
                    for &cl in w.shift_classes_pub(s as usize) {
                        w.derive(map, cl as usize);
                    }
                }
                Var::Class(cl) => w.set(map, cl as usize, acc),
            }
        }
    }
}

impl ClosureSystem {
    /// ⭐ **Monta um sistema À MÃO** — só para os gates desta crate.
    ///
    /// ⚠️ **Ela existe porque a máscara de componentes não é alcançável pela
    /// [`Self::build`]**: ali toda eliminação é `2×2` por construção. *Um gate que só
    /// pudesse exercitar o que o construtor produz nunca tocaria no caminho que a wave
    /// dos arcos vai usar.*
    #[cfg(test)]
    pub(crate) fn probe(
        free: Vec<Var>,
        dep: Vec<(Var, Vec<(u32, M2)>)>,
        dep_axes: Vec<[bool; 2]>,
    ) -> Self {
        Self {
            free,
            dep,
            dep_axes,
            ..Self::default()
        }
    }

    /// ⭐⭐⭐ **CONSTRÓI o sistema e ELIMINA uma variável por restrição independente.**
    ///
    /// ⚠️ **A expansão corre ao CONTRÁRIO da eliminação**, e tem de correr: a expressão
    /// que a equação `i` escreveu pode nomear uma variável que a equação `j > i` veio a
    /// eliminar. *Expandir na ordem da eliminação deixaria dependentes dentro de
    /// dependentes, que é a alternância outra vez, com outro nome.*
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn build(w: &Weld, seams: usize, jumped: &[bool]) -> (Self, FlatReport) {
        let mut rep = FlatReport {
            worst_det: 1.0,
            ..FlatReport::default()
        };
        // ── 1. As incógnitas.
        let mut vars: Vec<Var> = (0..seams)
            .filter(|&s| jumped[s])
            .filter_map(|s| u32::try_from(s).ok().map(Var::Shift))
            .collect();
        let n_shift = vars.len();
        let mut sing_classes: Vec<u32> = Vec::new();
        for c in w.closures() {
            if c.turn != 0 {
                #[allow(clippy::cast_possible_truncation)]
                sing_classes.push(w.class_of_pub(c.copies[0]) as u32);
            }
        }
        sing_classes.sort_unstable();
        sing_classes.dedup();
        vars.extend(sing_classes.iter().map(|&c| Var::Class(c)));
        let index = |v: Var| -> Option<usize> { vars.iter().position(|&x| x == v) };

        // ── 2. As equações, uma por fecho.
        // ⭐⭐ **Os fechos que RODAM entram primeiro, e a razão é a INTEGRALIDADE.**
        // A equação de um fecho que roda tem duas famílias de pivô: as translações
        // (`|det| = 1`) e a imagem do vértice singular (`M`, com `|det| = 2` ou `4`).
        // Se as translações dela já estiverem tomadas, o único pivô que sobra é `M` — e
        // um pivô de determinante `2` põe a dependente num **meio-inteiro**. *É o mesmo
        // meio-inteiro que a modalidade das singularidades do caminho penalizado já
        // tinha encontrado, um andar acima.*
        let mut order: Vec<&crate::weld::Closure> = w.closures().iter().collect();
        order.sort_by_key(|c| i32::from(c.turn == 0));
        let mut eqs: Vec<Vec<(usize, M2)>> = Vec::new();
        for c in order {
            rep.equations += 1;
            let mut terms: Vec<(usize, M2)> = Vec::new();
            let put = |v: Var, m: M2, terms: &mut Vec<(usize, M2)>| {
                let Some(i) = index(v) else { return };
                if let Some(e) = terms.iter_mut().find(|e| e.0 == i) {
                    e.1 = add(e.1, m);
                } else {
                    terms.push((i, m));
                }
            };
            for &(s, m) in w.cross_of(c.copies[1]) {
                put(Var::Shift(s), rot(m), &mut terms);
            }
            for &(s, m) in w.cross_of(c.copies[0]) {
                put(Var::Shift(s), sub(ZERO, rot(m + c.jump)), &mut terms);
            }
            put(Var::Shift(c.seam), sub(ZERO, rot(0)), &mut terms);
            if c.turn != 0 {
                // `M = R^{rot_b} − R^{k + rot_a}` — zero exactamente quando não roda.
                let m = sub(
                    rot(w.rot_of_pub(c.copies[1])),
                    rot(c.jump + w.rot_of_pub(c.copies[0])),
                );
                #[allow(clippy::cast_possible_truncation)]
                put(
                    Var::Class(w.class_of_pub(c.copies[0]) as u32),
                    m,
                    &mut terms,
                );
            }
            terms.retain(|e| e.1.iter().flatten().any(|x| x.abs() > 1.0e-6));
            eqs.push(terms);
        }

        // ── 3. A eliminação. ⭐ Prefere pivô com `|det| = 1` (leva inteiros a inteiros)
        //    e prefere eliminar uma TRANSLAÇÃO a uma imagem de singularidade — quem tem
        //    de ficar livre para o guloso pregar é a imagem.
        let mut owner: Vec<Option<usize>> = vec![None; vars.len()];
        let mut solved: Vec<(usize, Vec<(usize, M2)>)> = Vec::new();
        for terms in &eqs {
            let mut t = terms.clone();
            // substitui o que já foi eliminado
            let mut guard = 0;
            while guard < 64 {
                guard += 1;
                let Some(pos) = t.iter().position(|&(i, _)| owner[i].is_some()) else {
                    break;
                };
                let (i, c) = t.remove(pos);
                let Some(k) = owner[i] else { continue };
                let expr = solved[k].1.clone();
                for (j, a) in expr {
                    let m = matmul(c, a);
                    if let Some(e) = t.iter_mut().find(|e| e.0 == j) {
                        e.1 = add(e.1, m);
                    } else {
                        t.push((j, m));
                    }
                }
                t.retain(|e| e.1.iter().flatten().any(|x| x.abs() > 1.0e-6));
            }
            let best = t
                .iter()
                .filter(|(i, m)| owner[*i].is_none() && inv(*m).is_some())
                .min_by(|a, b| {
                    let key = |e: &&(usize, M2)| {
                        (
                            (det(e.1).abs() - 1.0).abs(),
                            usize::from(e.0 >= n_shift),
                            e.0,
                        )
                    };
                    let (ka, kb) = (key(a), key(b));
                    ka.0.total_cmp(&kb.0)
                        .then(ka.1.cmp(&kb.1))
                        .then(ka.2.cmp(&kb.2))
                })
                .copied();
            let Some((pi, pm)) = best else {
                rep.orphans += 1;
                continue;
            };
            let Some(pmi) = inv(pm) else {
                rep.orphans += 1;
                continue;
            };
            rep.worst_det = rep.worst_det.max(det(pm).abs());
            let neg = sub(ZERO, pmi);
            let expr: Vec<(usize, M2)> = t
                .iter()
                .filter(|(i, _)| *i != pi)
                .map(|&(i, c)| (i, matmul(neg, c)))
                .collect();
            owner[pi] = Some(solved.len());
            solved.push((pi, expr));
            rep.resolved += 1;
        }

        // ── 4. A expansão AO CONTRÁRIO: cada expressão fica só em livres.
        for k in (0..solved.len()).rev() {
            let mut expr = solved[k].1.clone();
            let mut guard = 0;
            while guard < 256 {
                guard += 1;
                let Some(pos) = expr.iter().position(|&(i, _)| owner[i].is_some()) else {
                    break;
                };
                let (i, c) = expr.remove(pos);
                let Some(j) = owner[i] else { continue };
                if j == k {
                    continue;
                }
                for (u, a) in solved[j].1.clone() {
                    let m = matmul(c, a);
                    if let Some(e) = expr.iter_mut().find(|e| e.0 == u) {
                        e.1 = add(e.1, m);
                    } else {
                        expr.push((u, m));
                    }
                }
                expr.retain(|e| e.1.iter().flatten().any(|x| x.abs() > 1.0e-6));
            }
            rep.cyclic |= guard >= 256;
            solved[k].1 = expr;
        }

        // ── 5. As livres, e o que cada uma move.
        let free: Vec<Var> = vars
            .iter()
            .enumerate()
            .filter(|(i, _)| owner[*i].is_none())
            .map(|(_, &v)| v)
            .collect();
        let fidx = |i: usize| -> Option<u32> {
            free.iter()
                .position(|&v| v == vars[i])
                .and_then(|p| u32::try_from(p).ok())
        };
        // ⚠️ **Os termos repetidos SOMAM-SE.** Duas entradas da mesma livre na mesma
        // expressão são dois caminhos até ela, e o coeficiente é a soma dos dois. ⛔ A
        // primeira redacção da propagação incremental apanhava só o PRIMEIRO (`find`), e
        // ficava verde porque o [`Self::apply`] do fim reconstruía tudo e tapava a
        // diferença — *um erro mascarado pela função que corre a seguir.*
        let dep: Vec<(Var, Vec<(u32, M2)>)> = solved
            .iter()
            .map(|(pi, expr)| {
                let mut out: Vec<(u32, M2)> = Vec::with_capacity(expr.len());
                for &(i, c) in expr {
                    let Some(f) = fidx(i) else { continue };
                    if let Some(e) = out.iter_mut().find(|e| e.0 == f) {
                        e.1 = add(e.1, c);
                    } else {
                        out.push((f, c));
                    }
                }
                (vars[*pi], out)
            })
            .collect();

        let mut touch: Vec<Vec<(u32, M2)>> = vec![Vec::new(); free.len()];
        let push = |slot: &mut Vec<(u32, M2)>, c: u32, m: M2| {
            if let Some(e) = slot.iter_mut().find(|e| e.0 == c) {
                e.1 = add(e.1, m);
            } else {
                slot.push((c, m));
            }
        };
        for (fi, &v) in free.iter().enumerate() {
            match v {
                Var::Shift(s) => {
                    for &(c, m) in w.crossings(s as usize) {
                        push(&mut touch[fi], c, rot(m));
                    }
                }
                Var::Class(cl) => {
                    for ((p, l), r) in w.members_pub(cl as usize) {
                        if let Some(c) = w.copy_index(p as usize, l as usize) {
                            push(&mut touch[fi], c, rot(r));
                        }
                    }
                }
            }
        }
        for (v, expr) in &dep {
            for &(fi, a) in expr {
                match *v {
                    Var::Shift(s) => {
                        for &(c, m) in w.crossings(s as usize) {
                            push(&mut touch[fi as usize], c, matmul(rot(m), a));
                        }
                    }
                    Var::Class(cl) => {
                        for ((p, l), r) in w.members_pub(cl as usize) {
                            if let Some(c) = w.copy_index(p as usize, l as usize) {
                                push(&mut touch[fi as usize], c, matmul(rot(r), a));
                            }
                        }
                    }
                }
            }
        }

        let mut by_free: Vec<Vec<(u32, M2)>> = vec![Vec::new(); free.len()];
        for (d, (_, expr)) in dep.iter().enumerate() {
            for &(fi, a) in expr {
                #[allow(clippy::cast_possible_truncation)]
                by_free[fi as usize].push((d as u32, a));
            }
        }
        let mut dep_seam = vec![false; seams];
        let mut dep_class = std::collections::BTreeSet::new();
        for (v, _) in &dep {
            match *v {
                Var::Shift(s) => dep_seam[s as usize] = true,
                Var::Class(c) => {
                    dep_class.insert(c);
                }
            }
        }
        // ⭐ A eliminação `2×2` possui a variável INTEIRA — é o caso de sempre, e é por
        // isso que ligar a máscara aqui é byte-idêntico. *Quem escrever meia variável
        // (a restrição de arco) marca-a como escalar na construção dela.*
        let dep_axes = vec![[true, true]; dep.len()];
        (
            Self {
                free,
                dep,
                dep_axes,
                touch,
                by_free,
                dep_seam,
                dep_class,
            },
            rep,
        )
    }
}

fn matmul(a: M2, b: M2) -> M2 {
    [
        [
            a[0][0].mul_add(b[0][0], a[0][1] * b[1][0]),
            a[0][0].mul_add(b[0][1], a[0][1] * b[1][1]),
        ],
        [
            a[1][0].mul_add(b[0][0], a[1][1] * b[1][0]),
            a[1][0].mul_add(b[0][1], a[1][1] * b[1][1]),
        ],
    ]
}

#[cfg(test)]
#[path = "weld_flat_tests.rs"]
mod tests;
