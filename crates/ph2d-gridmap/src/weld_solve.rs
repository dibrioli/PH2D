//! ⭐⭐⭐ **G3 SOLDADO** — o mesmo sistema, com as costuras **eliminadas**.
//!
//! # A diferença de espécie contra o [`crate::solve`]
//!
//! Lá a costura é um termo de energia com peso; aqui ela **não existe na energia**. As
//! cópias do outro lado deixaram de ser variáveis ([`crate::weld`]), e o que sobra é a
//! energia de orientação pura:
//!
//! ```text
//!     E = Σ_t A_t · [ |grad u_t − X_t/h|² + |grad v_t − Y_t/h|² ]
//! ```
//!
//! ⇒ ⛔ **não há `SEAM_WEIGHT` neste ficheiro, e não pode haver** — mantê-lo ao lado da
//! eliminação seria a mesma lei escrita duas vezes, com a segunda a ganhar em silêncio.
//!
//! # ⭐⭐⭐ AS DUAS VARIÁVEIS TÊM A MESMA LEI DE ACTUALIZAÇÃO, e ela deduz-se
//!
//! Escreva-se uma cópia como `z_c = R^{rot_c} · y + off_c`, com `y` a variável (a classe
//! soldada, ou a translação de uma costura) e `off_c` o que não depende dela. A energia
//! é, a menos de constantes,
//!
//! ```text
//!     E(y) = Σ_c [ (den_c/2)·|z_c|² − ⟨z_c , num_c⟩ ]
//! ```
//!
//! com `num_c` o numerador de Poisson do vértice ([`poisson_numerator`]) e `den_c` o
//! denominador. Derivando e usando que `R` é ortogonal:
//!
//! ```text
//!     ∂E/∂y = Σ_c R^{−rot_c} · ( den_c·z_c − num_c )  =  0
//! ⇒   Δy   = Σ_c R^{−rot_c} · ( num_c − den_c·z_c )  /  Σ_c den_c
//! ```
//!
//! ⭐ **É um passo em RESÍDUO, e é o mesmo para os dois** — muda só quem são as cópias
//! `c` e de onde sai a rotação: para uma **classe** são as cópias dela, com `rot` a
//! rotação até à raiz; para uma **translação** são as cópias que a derivação atravessa,
//! com a rotação da travessia ([`Weld::crossings`]).
//!
//! ⚠️ *Escrever a segunda como «a média do resíduo da costura» — que é o que o solver
//! penalizado faz — deixaria de funcionar aqui por uma razão exacta: com a costura
//! eliminada esse resíduo é **zero por construção**, e um passo proporcional a ele
//! nunca moveria a translação.*
//!
//! # ⭐⭐⭐ E as que FECHAM CICLO entram no mesmo sistema
//!
//! Uma ligação que fecha ciclo não tem cópia para eliminar — ela vira uma **equação**
//! entre translações (e, quando roda, a imagem do vértice singular). Todas elas vão para
//! **um** sistema linear ([`crate::weld_flat`]), que elimina uma variável por restrição
//! independente e devolve as **livres**. Este módulo relaxa as livres; as dependentes
//! escrevem-se por substituição.
//!
//! ⚠️ **A jacobiana de uma livre já não é uma rotação** — ela move as cópias que
//! atravessa *e* as que as dependentes atravessam, com matrizes `2×2` gerais. ⇒ o passo
//! deixa de ter denominador escalar e passa a ser um sistema `2×2`. *Fingir que ele
//! continua escalar é o que fazia a versão anterior divergir.*

use ph2d_mesh::Mesh;

use crate::comb::Combed;
use crate::cut::CutMesh;
use crate::solve::{Assembly, GridMap, SolveReport, assemble, measure, poisson_numerator, turn2};
use crate::weld::{Weld, WeldReport, weld};
use crate::weld_flat::{ClosureSystem, FlatReport, Var};

/// Quantas rondas de Gauss–Seidel o sistema soldado gasta.
///
/// ⭐⭐ **É `20×` menos que o penalizado, e o número sai de medição** — o sistema
/// penalizado é mal condicionado **por causa do peso**; sem ele é a Poisson pura.
pub const ROUNDS: usize = 8_000;

/// O que o solver soldado mediu de si próprio, além das réguas do mapa.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WeldSolveReport {
    /// As réguas do mapa — as mesmas do [`crate::solve`].
    pub solve: SolveReport,
    /// A estrutura da soldadura.
    pub weld: WeldReport,
    /// O sistema dos fechos.
    pub flat: FlatReport,
    /// Rondas gastas.
    pub rounds: usize,
    /// ⭐ O maior movimento da última ronda — a régua da convergência.
    pub last_move: f32,
    /// ⭐⭐ **Triângulos VIRADOS depois da 1.ª resolução** — antes de endurecer nada.
    pub folded_before: usize,
    /// ⭐⭐⭐ **Triângulos VIRADOS no fim** — a régua do endurecimento local.
    pub folded_after: usize,
    /// Quantas passagens de endurecimento correram de facto.
    pub stiffen_passes: usize,
    /// ⭐⭐⭐ Grupos de escalares **amarrados** pelos arcos que entraram de facto.
    pub tie_groups: usize,
    /// ⛔⛔ Grupos **RECUSADOS** — algum membro já tem quem o escreva.
    ///
    /// ⚠️ *Recusa-se o grupo INTEIRO, e conta-se.* Aceitar metade seria pôr duas leis
    /// sobre a mesma variável — o defeito que a obra A mediu nos dois subsistemas.
    pub tie_refused: usize,
    /// ⭐ A razão da recusa: `[dependente, livre do sistema, pregada]`.
    pub tie_refused_why: [usize; 3],
    /// ⛔⛔⛔ **Coordenadas NÃO-FINITAS no mapa, no fim do contínuo.**
    ///
    /// ⚠️ *Sem esta coluna, «o solver divergiu» e «o solver produziu um mapa são que
    /// alguém estragou depois» leem o mesmo `NaN` no fim da cadeia* — e as duas pedem
    /// curas em fases diferentes.
    pub nonfinite: usize,
    /// A mesma contagem **logo após a 1.ª ronda** — separa «nasceu torto» de «degradou».
    pub nonfinite_first: usize,
    /// ⭐⭐⭐ **Equações de CICLO de arco que entraram** — o A3, a condição sobre as
    /// translações.
    pub arc_cycles: usize,
}

/// Um grupo amarrado: a raiz, e por membro `(escalar, σ, δ)` — ver [`crate::arcline`].
type TieGroup = (u32, Vec<(u32, f32, f32)>);

/// Uma equação de ciclo de arco: a incógnita livre e o eixo que ela **possui**, e a
/// expressão dele — `(variável, eixo, coeficiente)`. Ver
/// [`WeldRelaxer::attach_arc_cycles`].
type ArcCycle = (usize, usize, Vec<(Var, usize, f32)>);

/// ⭐ **O RELAXADOR SOLDADO** — o sistema reduzido, uma variável de cada vez.
pub(crate) struct WeldRelaxer<'a> {
    a: &'a Assembly,
    w: &'a Weld,
    /// O sistema dos fechos, já eliminado.
    pub(crate) sys: ClosureSystem,
    /// Por classe, o denominador somado das cópias dela.
    den: Vec<f32>,
    /// ⭐ Por classe, que componentes estão **pregadas**.
    frozen: Vec<[bool; 2]>,
    /// Por incógnita livre, que componentes estão pregadas.
    free_frozen: Vec<[bool; 2]>,
    /// Por classe, as classes vizinhas — a fila do degrau local anda por aqui.
    neigh: Vec<Vec<u32>>,
    /// ⭐⭐⭐ As amarras dos arcos — ver [`crate::arcline`]. Vazio = o caminho de sempre.
    ties: Vec<TieGroup>,
    /// Por classe, que componentes são **conduzidas** por uma amarra.
    ///
    /// ⚠️ Elas saem da [`Self::relax_class`] pela mesma porta que as pregadas: *quem tem
    /// quem o escreva não se relaxa sozinho.*
    driven: Vec<[bool; 2]>,
    /// ⭐⭐⭐ **AS EQUAÇÕES DE CICLO DOS ARCOS** — a condição sobre as TRANSLAÇÕES.
    ///
    /// Por equação: a incógnita livre e o eixo que ela **possui**, e a expressão dele nos
    /// outros termos — `y_dono = Σ coef · var[eixo]`.
    ///
    /// ⚠️ **É o A3, e ele é PRÉ-REQUISITO e não sequela** (`ACHADO` §23.18): uma equação
    /// que fecha ciclo não elimina classe nenhuma, e sem esta lista a condição dela
    /// **não está no sistema** — o grupo e as translações escrevem-se um ao outro.
    arc_cycles: Vec<ArcCycle>,
    /// Grupos recusados por algum membro já ter dono.
    refused: usize,
    /// ⭐ Por que razão: `[dependente, livre do sistema, pregada]`.
    ///
    /// ⚠️ *«Recusado» sem a razão manda quem lê adivinhar entre três desenhos
    /// diferentes* — e só um deles é o que a peça de facto tem.
    refused_why: [usize; 3],
}

impl<'a> WeldRelaxer<'a> {
    pub(crate) fn new(a: &'a Assembly, w: &'a Weld, cut: &CutMesh, combed: &Combed) -> Self {
        let seams = cut.seams.len();
        let nc = w.classes();
        let mut den = vec![0.0f32; nc];
        for (c, slot) in den.iter_mut().enumerate() {
            for ((p, l), _) in w.members_pub(c) {
                *slot += a.denom[p as usize][l as usize];
            }
        }
        // ⚠️ A vizinhança é entre CLASSES: duas classes são vizinhas se alguma cópia de
        // uma partilha um triângulo com alguma cópia da outra. *É a vizinhança do
        // sistema reduzido, e não a da malha cortada.*
        let mut neigh: Vec<Vec<u32>> = vec![Vec::new(); nc];
        for (c, slot) in neigh.iter_mut().enumerate() {
            for ((p, l), _) in w.members_pub(c) {
                let (p, l) = (p as usize, l as usize);
                for &ti in &a.by_vert[p][l] {
                    for v in a.tris[p][ti as usize].v {
                        if v as usize == l {
                            continue;
                        }
                        if let Some((other, _)) = w.of(p, v as usize) {
                            #[allow(clippy::cast_possible_truncation)]
                            slot.push(other as u32);
                        }
                    }
                }
            }
            slot.sort_unstable();
            slot.dedup();
        }
        let jumped: Vec<bool> = (0..seams)
            .map(|s| combed.jump.get(s).copied().flatten().is_some())
            .collect();
        let (sys, _) = ClosureSystem::build(w, seams, &jumped);
        let nf = sys.free().len();
        Self {
            a,
            w,
            sys,
            den,
            frozen: vec![[false; 2]; nc],
            free_frozen: vec![[false; 2]; nf],
            neigh,
            ties: Vec::new(),
            arc_cycles: Vec::new(),
            driven: vec![[false; 2]; nc],
            refused: 0,
            refused_why: [0; 3],
        }
    }

    /// ⭐⭐⭐ **LIGA AS AMARRAS DOS ARCOS.**
    ///
    /// ⛔⛔ **Um grupo é aceite INTEIRO ou recusado inteiro.** Se qualquer membro já tem
    /// quem o escreva — uma classe **dependente** do sistema dos fechos, uma que **é**
    /// incógnita livre dele, ou uma componente já pregada — o grupo cai fora e é
    /// **contado** ([`WeldSolveReport::tie_refused`]).
    ///
    /// ⚠️ *Aceitar metade de um grupo poria duas leis sobre a mesma variável*, que é
    /// exactamente o defeito que a obra A mediu ao pôr as duas espécies de fecho em
    /// subsistemas separados (esfera a `NaN`, toro a `6,4e17`).
    pub(crate) fn attach_ties(&mut self, ties: &crate::arcline::ScalarTies) {
        for g in 0..ties.groups() {
            let Some((root, members)) = ties.group(g) else {
                continue;
            };
            // ⛔⛔ **A 1.ª redacção recusava um membro que fosse incógnita LIVRE do
            // sistema dos fechos, e isso recusava 100 % dos grupos nas peças reais**
            // (`ACHADO` §23.17): os cantos dos arcos SÃO os cones, e um cone é livre.
            //
            // ⭐⭐⭐ **Mas «livre» quer dizer que NINGUÉM a possui** — e o relaxador já sabe
            // segurar **um eixo** de uma livre ([`Self::freeze_free`]). ⇒ a cura não é
            // recusar; é **aceitar e CONGELAR** aquele eixo, para a
            // [`Self::relax_free`] deixar de o escrever. *Era o contador da recusa que
            // dizia qual porta abrir.*
            //
            // ⛔ A recusa que FICA é a que continua a ser uma segunda lei: uma classe
            // **dependente** já é escrita por substituição, e conduzi-la seria escrever
            // a mesma variável por dois caminhos.
            let why = |x: u32| -> Option<usize> {
                let (c, ax) = (x as usize / 2, x as usize % 2);
                if c >= self.frozen.len() || self.sys.is_dependent_class(c) {
                    return Some(0);
                }
                self.frozen[c][ax].then_some(2)
            };
            if let Some(k) = members.iter().copied().find_map(why) {
                self.refused += 1;
                self.refused_why[k] += 1;
                continue;
            }
            // ⭐ Congela o eixo de cada membro que seja incógnita livre — a raiz
            // **inclusive**: o grupo inteiro passa a ser escrito pela [`Self::relax_tie`],
            // e nenhum membro pode continuar a ser relaxado por conta própria.
            let to_freeze: Vec<(usize, usize)> = members
                .iter()
                .filter_map(|&x| {
                    let (c, ax) = (x as usize / 2, x as usize % 2);
                    self.free_index_class(c).map(|i| (i, ax))
                })
                .collect();
            for (i, ax) in to_freeze {
                self.freeze_free(i, ax);
            }
            let rows: Vec<(u32, f32, f32)> = members
                .iter()
                .map(|&x| {
                    let (_, sigma, delta) = ties.of(x);
                    (x, sigma, delta)
                })
                .collect();
            for &(x, _, _) in &rows {
                if x != root {
                    self.driven[x as usize / 2][x as usize % 2] = true;
                }
            }
            self.ties.push((root, rows));
        }
    }

    /// ⭐⭐⭐ **LIGA AS EQUAÇÕES DE CICLO** — cada uma passa a POSSUIR um escalar de
    /// translação.
    ///
    /// ⚠️ **Só translações servem de dono**, e a razão é medida: uma equação de ciclo tem
    /// `1` a `3` termos de translação (mediana `1`), e os termos de **classe** dela já
    /// pertencem às amarras. *Escolher um dono entre os que já têm dono seria a segunda
    /// lei outra vez, um andar abaixo.*
    ///
    /// ⛔ Uma equação sem translação livre é **recusada e contada** — o desenho do A3 não
    /// a alcança, e dizê-lo é melhor do que a impor a meio.
    pub(crate) fn attach_arc_cycles(
        &mut self,
        eqs: &[crate::arcline::ArcEquation],
        cycles: &[usize],
    ) {
        for &k in cycles {
            let Some(eq) = eqs.get(k) else { continue };
            let own = eq.terms.iter().enumerate().find(|(_, t)| {
                matches!(t.0, Var::Shift(_))
                    && t.2.abs() > 1.0e-6
                    && self
                        .sys
                        .free()
                        .iter()
                        .position(|&v| v == t.0)
                        .is_some_and(|i| !self.free_axis_is_frozen(i, t.1))
            });
            let Some((pos, &(v, ax, k_own))) = own else {
                self.refused += 1;
                continue;
            };
            let Some(i) = self.sys.free().iter().position(|&f| f == v) else {
                continue;
            };
            // `y_dono = −(1/k)·Σ outros`.
            let rest: Vec<(Var, usize, f32)> = eq
                .terms
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != pos)
                .map(|(_, &(vv, aa, c))| (vv, aa, -c / k_own))
                .collect();
            self.freeze_free(i, ax);
            self.arc_cycles.push((i, ax, rest));
        }
    }

    /// Quantas equações de ciclo entraram.
    pub(crate) fn arc_cycle_count(&self) -> usize {
        self.arc_cycles.len()
    }

    /// ⭐⭐⭐ **ESCREVE OS DONOS DAS EQUAÇÕES DE CICLO** — a condição, imposta.
    ///
    /// ⚠️ **Corre no FIM da varredura, e a ordem é load-bearing:** os termos da expressão
    /// são classes e translações que as fases anteriores acabaram de mover. *Impor a
    /// condição antes delas seria impô-la sobre um estado que já não existe quando a
    /// varredura acaba* — e é assim que uma restrição vira um ponto de partida.
    pub(crate) fn apply_arc_cycles(&self, map: &mut GridMap) -> f32 {
        let mut worst = 0.0f32;
        for (i, ax, rest) in &self.arc_cycles {
            let mut want = 0.0f32;
            for &(v, a, c) in rest {
                let val = match v {
                    Var::Shift(s) => map.shift.get(s as usize).copied().unwrap_or([0.0; 2]),
                    Var::Class(cl) => self.w.value_pub(map, cl as usize),
                };
                want = c.mul_add(val[a], want);
            }
            if !want.is_finite() {
                continue;
            }
            let now = self.read_free(map, *i);
            let mut d = [0.0f32; 2];
            d[*ax] = want - now[*ax];
            self.sys.bump(self.w, map, *i, d);
            worst = worst.max(d[*ax].abs());
        }
        worst
    }

    /// Quantos grupos entraram, quantos foram recusados, e por que razão.
    pub(crate) fn tie_counts(&self) -> (usize, usize, [usize; 3]) {
        (self.ties.len(), self.refused, self.refused_why)
    }

    /// ⭐ O acumulado do resíduo de uma classe, no quadro dela, e o denominador.
    ///
    /// ⚠️ **Uma função só, com dois leitores** — a relaxação de uma classe e a de um
    /// grupo amarrado. *A mesma soma escrita duas vezes seria a segunda a envelhecer.*
    fn class_acc(&self, map: &GridMap, class: usize) -> ([f32; 2], f32) {
        let mut acc = [0.0f32; 2];
        for ((p, l), rot) in self.w.members_pub(class) {
            let (r, _) = self.residual(map, p as usize, l as usize);
            let rr = turn2(r, -rot);
            acc[0] += rr[0];
            acc[1] += rr[1];
        }
        (acc, self.den[class])
    }

    /// ⭐⭐⭐ **RELAXA UM GRUPO AMARRADO** — um escalar por todos os membros.
    ///
    /// A lei é a mesma de uma classe, um nível acima: soma-se o resíduo de **todos** os
    /// membros (cada um com o sinal que o liga à raiz) sobre a soma dos denominadores
    /// deles, e escreve-se o grupo inteiro a partir da raiz.
    pub(crate) fn relax_tie(&self, map: &mut GridMap, g: usize) -> f32 {
        let Some((root, rows)) = self.ties.get(g) else {
            return 0.0;
        };
        let (mut num, mut den) = (0.0f32, 0.0f32);
        for &(x, sigma, _) in rows {
            let (c, ax) = (x as usize / 2, x as usize % 2);
            let (acc, d) = self.class_acc(map, c);
            if d <= 0.0 {
                continue;
            }
            num += sigma * acc[ax];
            den += d;
        }
        if den <= 0.0 {
            return 0.0;
        }
        let step = num / den;
        // ⛔⛔⛔ **UM PASSO NÃO-FINITO NÃO SE ESCREVE — CONTA-SE.**
        //
        // ⚠️ **Medido (2026-08-27):** a `sphere_uv` com a restrição activa punha o mapa a
        // `NaN` **à 4.ª ronda e voltava a são à 16.ª**. *Uma realimentação de ganho > 1
        // diverge e FICA divergida* — este ia e vinha porque o [`Weld::set`]
        // **sobrescreve**, e a ronda seguinte lavava o valor. ⇒ não era o subsistema
        // realimentado da obra A, era **um passo isolado a estourar**.
        //
        // ⛔ Escrever `inf` faz a ronda seguinte calcular `inf − inf` e produzir `NaN`, e
        // aí a peça inteira cai. *Recusar o passo e contá-lo separa «o solver estourou»
        // de «o solver foi travado», que sem o contador são o mesmo mapa.*
        if !step.is_finite() {
            return 0.0;
        }
        let (rc, rax) = (*root as usize / 2, *root as usize % 2);
        let y_root = self.w.value_pub(map, rc)[rax] + step;
        for &(x, sigma, delta) in rows {
            let (c, ax) = (x as usize / 2, x as usize % 2);
            let want = sigma.mul_add(y_root, delta);
            let now = self.w.value_pub(map, c);
            // ⛔⛔ **UM MEMBRO QUE É INCÓGNITA LIVRE TEM DE PASSAR PELO `bump`.** Escrevê-lo
            // com [`Weld::set`] move a classe e **não** move as dependentes que o sistema
            // dos fechos escreve a partir dela — e o mapa fica internamente inconsistente,
            // sem nada a acusar. *A [`Self::relax_free`] usa o `bump` por esta mesma
            // razão.*
            if let Some(i) = self.free_index_class(c) {
                let mut d = [0.0f32; 2];
                d[ax] = want - now[ax];
                self.sys.bump(self.w, map, i, d);
            } else {
                let mut y = now;
                y[ax] = want;
                self.w.set(map, c, y);
            }
        }
        step.abs()
    }

    /// O resíduo de Poisson de uma cópia: `num − den·z`, e o `den`.
    fn residual(&self, map: &GridMap, p: usize, l: usize) -> ([f32; 2], f32) {
        let d = self.a.denom[p][l];
        let n = poisson_numerator(self.a, map, p, l);
        let z = map.uv[p][l];
        ([n[0] - d * z[0], n[1] - d * z[1]], d)
    }

    /// ⭐⭐⭐ **RELAXA UMA CLASSE** — ver a lei no doc deste módulo.
    ///
    /// ⚠️ Uma classe que o sistema dos fechos **escreve** não é relaxada aqui: ela é
    /// dependente, e relaxá-la seria a segunda lei sobre a mesma variável.
    pub(crate) fn relax_class(&self, map: &mut GridMap, class: usize) -> f32 {
        if self.sys.is_dependent_class(class) {
            return 0.0;
        }
        if let Some(i) = self.free_index_class(class) {
            return self.relax_free(map, i);
        }
        let den = self.den[class];
        if den <= 0.0 {
            return 0.0;
        }
        let d0 = self.driven[class];
        let f = [
            self.frozen[class][0] || d0[0],
            self.frozen[class][1] || d0[1],
        ];
        if f[0] && f[1] {
            return 0.0;
        }
        let (acc, _) = self.class_acc(map, class);
        let d = [
            if f[0] { 0.0 } else { acc[0] / den },
            if f[1] { 0.0 } else { acc[1] / den },
        ];
        let y = self.w.value_pub(map, class);
        self.w.set(map, class, [y[0] + d[0], y[1] + d[1]]);
        d[0].abs().max(d[1].abs())
    }

    /// ⭐⭐⭐ **PODE ESTA CLASSE SER PREGADA À MÃO?** — só as que ninguém já escreve.
    ///
    /// ⛔ Uma classe **dependente** é escrita por substituição a partir das livres, e
    /// pregá-la seria uma segunda lei sobre a mesma variável; uma que **é** livre já entra
    /// na escada gulosa pelo outro caminho. *O conjunto que sobra são exactamente os
    /// vértices que o sistema dos fechos não tem como tocar.*
    pub(crate) fn class_is_loose(&self, class: usize) -> bool {
        !self.sys.is_dependent_class(class)
            && self.free_index_class(class).is_none()
            && self.den.get(class).copied().unwrap_or(0.0) > 0.0
    }

    /// O valor de uma classe no mapa.
    pub(crate) fn read_class(&self, map: &GridMap, class: usize) -> [f32; 2] {
        self.w.value_pub(map, class)
    }

    /// Escreve o valor de uma classe — e as cópias dela derivam.
    pub(crate) fn write_class(&self, map: &mut GridMap, class: usize, y: [f32; 2]) {
        self.w.set(map, class, y);
    }

    /// Prega um eixo de uma classe: a [`Self::relax_class`] deixa de lhe tocar.
    pub(crate) fn freeze_class(&mut self, class: usize, ax: usize) {
        if let Some(f) = self.frozen.get_mut(class) {
            f[ax] = true;
        }
    }

    fn free_index_class(&self, class: usize) -> Option<usize> {
        u32::try_from(class)
            .ok()
            .and_then(|c| self.sys.free().iter().position(|&v| v == Var::Class(c)))
    }

    /// ⭐⭐⭐ **RELAXA UMA INCÓGNITA LIVRE DO SISTEMA REDUZIDO.**
    ///
    /// A jacobiana `J_c = ∂z_c/∂v` é uma matriz `2×2` geral (a livre move as cópias que
    /// atravessa **e** as que as dependentes atravessam), logo o passo é a solução do
    /// sistema normal:
    ///
    /// ```text
    ///     H = Σ_c den_c · J_cᵀJ_c      g = Σ_c J_cᵀ · r_c      Δv = H⁻¹ g
    /// ```
    ///
    /// ⭐ **É minimização exacta ao longo daquela coordenada** ⇒ a energia desce sempre,
    /// e a varredura assenta. *A versão que fingia denominador escalar divergia.*
    pub(crate) fn relax_free(&self, map: &mut GridMap, i: usize) -> f32 {
        let f = self.free_frozen[i];
        if f[0] && f[1] {
            return 0.0;
        }
        let (mut h, mut g) = ([[0.0f32; 2]; 2], [0.0f32; 2]);
        for &(c, j) in self.sys.touched(i) {
            let (p, l) = self.w.where_is_pub(c);
            let (r, d) = self.residual(map, p as usize, l as usize);
            if d <= 0.0 {
                continue;
            }
            g[0] += j[0][0].mul_add(r[0], j[1][0] * r[1]);
            g[1] += j[0][1].mul_add(r[0], j[1][1] * r[1]);
            h[0][0] += d * j[0][0].mul_add(j[0][0], j[1][0] * j[1][0]);
            h[0][1] += d * j[0][0].mul_add(j[0][1], j[1][0] * j[1][1]);
            h[1][1] += d * j[0][1].mul_add(j[0][1], j[1][1] * j[1][1]);
        }
        h[1][0] = h[0][1];
        let Some(step) = solve2(h, g, f) else {
            return 0.0;
        };
        self.sys.bump(self.w, map, i, step);
        step[0].abs().max(step[1].abs())
    }

    /// Escreve uma incógnita livre num valor, e propaga às dependentes.
    pub(crate) fn write_free(&self, map: &mut GridMap, i: usize, value: [f32; 2]) {
        let now = self.read_free(map, i);
        self.sys
            .bump(self.w, map, i, [value[0] - now[0], value[1] - now[1]]);
    }

    /// O valor de uma incógnita livre.
    pub(crate) fn read_free(&self, map: &GridMap, i: usize) -> [f32; 2] {
        match self.sys.free()[i] {
            Var::Shift(s) => map.shift[s as usize],
            Var::Class(c) => self.w.value_pub(map, c as usize),
        }
    }

    /// ⭐ Esta componente de uma livre já tem dono? (pregada à mão, ou **conduzida por
    /// uma amarra de arco**.)
    ///
    /// ⚠️ **A escada gulosa TEM de perguntar isto.** Ela empurra os **dois** eixos de
    /// toda livre para a fila de pregos, e um eixo que a [`Self::relax_tie`] conduz já
    /// tem lei. *Pregar por cima é a segunda lei sobre o mesmo escalar — e o preço
    /// medido foi um passo `NaN` no meio da escada.*
    pub(crate) fn free_axis_is_frozen(&self, i: usize, ax: usize) -> bool {
        self.free_frozen.get(i).is_some_and(|f| f[ax])
    }

    /// Prega uma componente de uma incógnita livre.
    pub(crate) fn freeze_free(&mut self, i: usize, ax: usize) {
        self.free_frozen[i][ax] = true;
        if let Var::Class(c) = self.sys.free()[i] {
            self.frozen[c as usize][ax] = true;
        }
    }

    /// As classes vizinhas de uma classe.
    pub(crate) fn neighbours(&self, class: usize) -> &[u32] {
        &self.neigh[class]
    }

    /// As classes que uma incógnita livre move.
    pub(crate) fn classes_of_free(&self, i: usize) -> Vec<u32> {
        let mut out: Vec<u32> = self
            .sys
            .touched(i)
            .iter()
            .map(|&(c, _)| u32::try_from(self.w.class_of_pub(c)).unwrap_or(0))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Uma varredura global — devolve o maior movimento.
    pub(crate) fn sweep(&self, map: &mut GridMap) -> f32 {
        let mut worst = 0.0f32;
        for c in 0..self.w.classes() {
            worst = worst.max(self.relax_class(map, c));
        }
        for g in 0..self.ties.len() {
            worst = worst.max(self.relax_tie(map, g));
        }
        for i in 0..self.sys.free().len() {
            worst = worst.max(self.relax_free(map, i));
        }
        worst.max(self.apply_arc_cycles(map))
    }
}

/// Resolve `H·x = g` em `2×2`, honrando as componentes pregadas.
fn solve2(h: [[f32; 2]; 2], g: [f32; 2], frozen: [bool; 2]) -> Option<[f32; 2]> {
    match frozen {
        [true, true] => None,
        [true, false] => (h[1][1].abs() > 1.0e-12).then(|| [0.0, g[1] / h[1][1]]),
        [false, true] => (h[0][0].abs() > 1.0e-12).then(|| [g[0] / h[0][0], 0.0]),
        [false, false] => {
            let d = h[0][0].mul_add(h[1][1], -(h[0][1] * h[1][0]));
            (d.abs() > 1.0e-12).then(|| {
                [
                    g[0].mul_add(h[1][1], -(h[0][1] * g[1])) / d,
                    h[0][0].mul_add(g[1], -(g[0] * h[1][0])) / d,
                ]
            })
        }
    }
}

/// ⭐⭐⭐ **RESOLVE O MAPA COM AS COSTURAS ELIMINADAS.**
#[must_use]
/// ⭐⭐⭐ **QUANTAS VEZES o endurecimento local corre** (MIQ 2009 §5.4).
///
/// A lei publicada: uma parametrização harmónica **não é injectiva** por construção — onde
/// a superfície comprime, triângulos viram do avesso. A cura é **pesar mais** os triângulos
/// virados e **resolver outra vez**, repetindo: cada passagem torna aquela região mais
/// rígida e empurra a distorção para onde há folga.
///
/// ⛔ **O número tem de ser MEDIDO** (`CLAUDE.md` §0.0) — ver a tabela em
/// [`STIFFEN_FACTOR`]. `0` devolve o caminho antigo bit a bit.
///
/// ⚠️ **O preço é linear**: cada passagem é uma resolução inteira do contínuo.
fn stiffen_passes(_rounds: usize) -> usize {
    std::env::var("PH2D_STIFFEN_PASSES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(STIFFEN_PASSES)
}

/// ⛔⛔⛔ **ZERO — o endurecimento local foi CONSTRUÍDO, MEDIDO e REJEITADO.**
///
/// A `0` esta função é bit a bit o caminho de sempre, e há gate a afirmá-lo
/// (`stiffening_at_zero_passes_is_the_old_path`).
///
/// # ⛔ A tabela da rejeição (2026-08-25, peça do artista, cadeia inteira)
///
/// | passagens | factor | triângulos VIRADOS | bordo | `χ` | enviesamento p50 | `>60°` |
/// |---|---|---|---|---|---|---|
/// | ⭐ **`0`** | — | **`14` ⇒ `14`** | **`10`** | **`1`** | `7,3°` | `5` |
/// | 2 | `2` | ⛔ `14` ⇒ **`15`** | `12` | `1` | `7,8°` | `2` |
/// | 5 | `2` | ⛔ `14` ⇒ **`18`** | `14` | `0` | `7,8°` | `7` |
/// | 2 | `4` | ⛔ `14` ⇒ **`19`** | `12` | `1` | `7,4°` | `5` |
/// | 5 | `4` | ⛔ `14` ⇒ **`23`** | `12` | `1` | `7,0°` | `3` |
/// | 2 | `10` | ⛔ `14` ⇒ **`19`** | `14` | `1` | `7,4°` | `5` |
/// | 2 | `0,5` | `14` ⇒ `14` | `10` | `1` | `6,7°` | `4` |
/// | 2 | `0,25` | `14` ⇒ `14` | ⛔ `18` | ⛔ `0` | `7,3°` | `4` |
/// | 2 | `0,1` | `14` ⇒ `14` | `12` | `1` | `7,5°` | `6` |
///
/// # ⭐⭐⭐ O MECANISMO, e é ele que fecha a família
///
/// ⛔ **Endurecer AUMENTA as dobras** — monotonamente no factor e nas passagens. Isso não
/// é afinação a faltar: é o sinal de que a cura publicada assume **outra energia**.
///
/// No *paper* a parametrização é **harmónica**, e pesar mais um triângulo virado força-o a
/// ser mais rígido, o que o desvira. ⭐⭐ **A nossa energia não é essa: ela é
/// `Σ area·|∇z − X/h|²`, ou seja «SEGUIR O CAMPO».** Pesar mais um triângulo virado manda-o
/// obedecer ao campo **com mais força** — exactamente no sítio onde obedecer ao campo é o
/// que o vira. *A cura empurra na direcção do defeito.*
///
/// ⚠️ **E o sinal contrário também não serve:** amolecer (`factor < 1`) deixa a contagem de
/// dobras **exactamente igual** (`14` ⇒ `14`) e move as outras colunas dentro da banda de
/// caos que o guloso já tem — medida à parte, `8k/16k/32k/64k` rondas dão `14/14/10/12`
/// arestas de bordo sobre o **mesmo** mapa. *Uma melhoria menor que a banda de caos do
/// sistema não é uma melhoria; é uma amostra.*
///
/// ⇒ **Duas hipóteses boas da mesma família falharam ⇒ a família está fechada.** A dobra
/// no domínio não se cura pesando triângulos: ela é uma propriedade do CAMPO que o mapa
/// está a seguir.
///
/// ⭐ **O que fica de valor é a RÉGUA:** [`WeldSolveReport::folded_before`] /
/// [`WeldSolveReport::folded_after`] medem as dobras no **contínuo**, antes do
/// arredondamento inteiro — e essa contagem não existia. A sonda independente do
/// `chain_info` mede-as **depois** (`20`), e as duas juntas dizem quanto o arredondamento
/// acrescenta.
pub const STIFFEN_PASSES: usize = 0;

/// ⭐ **POR QUANTO o peso de um triângulo virado é multiplicado, por passagem.**
///
/// ⛔ Inerte enquanto [`STIFFEN_PASSES`] for `0` — ver a tabela da rejeição lá.
pub const STIFFEN_FACTOR: f32 = 2.0;

fn stiffen_factor() -> f32 {
    std::env::var("PH2D_STIFFEN_FACTOR")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(STIFFEN_FACTOR)
}

pub fn solve_welded(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    h: f32,
    rounds: usize,
) -> (GridMap, WeldSolveReport) {
    solve_welded_with(mesh, cut, combed, h, rounds, None, None)
}

/// ⭐⭐⭐ **O MESMO, com as AMARRAS DOS ARCOS ligadas.**
///
/// ⚠️ **A escolha do eixo atravessado de cada arco sai do mapa que gerou as amarras**, que
/// na cadeia é a solução **livre**. *É por isso que isto é um segundo passe e não um
/// parâmetro:* a restrição precisa de saber para que lado cada arco corre, e quem o diz é
/// a solução sem ela.
///
/// ⛔ `ties = None` é **byte-idêntico** ao [`solve_welded`] — é o controlo.
#[must_use]
pub fn solve_welded_with(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    h: f32,
    rounds: usize,
    ties: Option<&crate::arcline::ScalarTies>,
    arcs: Option<(&[crate::arcline::ArcEquation], &[usize])>,
) -> (GridMap, WeldSolveReport) {
    let mut rep = WeldSolveReport::default();
    let (w, wrep) = weld(cut, combed);
    rep.weld = wrep;
    let mut a = assemble(mesh, cut, combed, h, &mut rep.solve);
    let jumped: Vec<bool> = (0..cut.seams.len())
        .map(|s| combed.jump.get(s).copied().flatten().is_some())
        .collect();
    rep.flat = ClosureSystem::build(&w, cut.seams.len(), &jumped).1;
    let mut map = GridMap {
        uv: cut
            .origin
            .iter()
            .map(|o| vec![[0.0f32; 2]; o.len()])
            .collect(),
        shift: vec![[0.0; 2]; cut.seams.len()],
    };
    // ⭐⭐⭐ **O ENDURECIMENTO LOCAL** — ver [`STIFFEN_PASSES`].
    //
    // ⚠️ **A 1.ª passagem é a de sempre, bit a bit**, e é isso que faz `passes = 0` ser o
    // caminho antigo: só depois de uma solução existir é que há dobras para endurecer.
    for pass in 0..=stiffen_passes(rounds) {
        let mut r = WeldRelaxer::new(&a, &w, cut, combed);
        if let Some(t) = ties {
            r.attach_ties(t);
            // ⭐⭐⭐ **O A3:** as equações que FECHAM CICLO passam a possuir um escalar de
            // translação. ⚠️ *Sem elas a condição não está no sistema* — e a §23.18 mediu
            // o preço: a esfera diverge.
            if let Some((eqs, cyc)) = arcs {
                r.attach_arc_cycles(eqs, cyc);
            }
            let (g, refused, why) = r.tie_counts();
            rep.tie_groups = g;
            rep.tie_refused = refused;
            rep.tie_refused_why = why;
            rep.arc_cycles = r.arc_cycle_count();
        }
        let count_bad = |m: &GridMap| -> usize {
            m.uv.iter()
                .flatten()
                .filter(|z| !z[0].is_finite() || !z[1].is_finite())
                .count()
                + m.shift
                    .iter()
                    .filter(|t| !t[0].is_finite() || !t[1].is_finite())
                    .count()
        };
        for round in 0..rounds {
            rep.last_move = r.sweep(&mut map);
            rep.rounds = round + 1;
            if round == 0 {
                rep.nonfinite_first = count_bad(&map);
            }
        }
        rep.nonfinite = count_bad(&map);
        let folded = a.folded(&map);
        rep.folded_after = folded.len();
        if pass == 0 {
            rep.folded_before = folded.len();
        }
        if folded.is_empty() || pass == stiffen_passes(rounds) {
            break;
        }
        rep.stiffen_passes = pass + 1;
        a.stiffen(&folded, stiffen_factor());
    }
    measure(&a, cut, combed, &map, h, &mut rep.solve);
    crate::weld::holonomy(&w, &map, &mut rep.weld);
    (map, rep)
}

#[cfg(test)]
#[path = "weld_solve_tests.rs"]
mod tests;
