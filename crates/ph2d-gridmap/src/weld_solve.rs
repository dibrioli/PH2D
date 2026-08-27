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

use crate::comb::Combed;
use crate::cut::CutMesh;
use crate::solve::{Assembly, GridMap, poisson_numerator, turn2};
use crate::weld::Weld;
use crate::weld_flat::{ClosureSystem, Var};
use crate::weld_solve_driver::solve2;

mod ties;

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
    /// ⭐⭐ Grupos cuja RAIZ é classe simples — os que a `relax_class` também escrevia.
    plain_roots: usize,
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
            plain_roots: 0,
            driven: vec![[false; 2]; nc],
            refused: 0,
            refused_why: [0; 3],
        }
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
        let p = self.sweep_parts(map);
        p.iter().copied().fold(0.0f32, f32::max)
    }

    /// ⭐⭐⭐ **A MESMA VARREDURA, COM O MAIOR MOVIMENTO DE CADA ESCRITOR** —
    /// `[classe, amarra, livre, ciclo de arco]`.
    ///
    /// ⚠️ **Um `last_move` só não diz QUEM diverge.** A esfera estourava na ronda `6134`
    /// com movimento `inf`, e os quatro escritores são candidatos com curas diferentes;
    /// *bisectar por hipótese custa uma corrida de 8 s por tentativa, e esta coluna
    /// responde de uma vez.* ⛔ `max` sobre floats ignora `NaN` — por isso quem lê tem de
    /// perguntar `is_finite`, nunca comparar.
    pub(crate) fn sweep_parts(&self, map: &mut GridMap) -> [f32; 4] {
        let mut p = [0.0f32; 4];
        for c in 0..self.w.classes() {
            p[0] = p[0].max(self.relax_class(map, c));
        }
        for g in 0..self.ties.len() {
            p[1] = p[1].max(self.relax_tie(map, g));
        }
        for i in 0..self.sys.free().len() {
            p[2] = p[2].max(self.relax_free(map, i));
        }
        p[3] = self.apply_arc_cycles(map);
        p
    }
}

#[cfg(test)]
#[path = "weld_solve_tests.rs"]
mod tests;
