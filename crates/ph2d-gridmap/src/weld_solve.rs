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
//! # ⚠️ O que a eliminação NÃO alcança
//!
//! As ligações que fecham ciclo ([`Weld::closures`]) não têm variável para eliminar. As
//! que **rodam** são os vértices singulares — e a medição mostra que são *exactamente*
//! eles (`8` para `8` nas duas esferas, `12` para `12` no toro). ⛔ Elas ficam medidas e
//! contadas, nunca escondidas: ver [`WeldSolveReport::closures`].

use ph2d_mesh::Mesh;

use crate::comb::Combed;
use crate::cut::CutMesh;
use crate::solve::{
    Assembly, GridMap, SolveReport, assemble, measure, poisson_numerator, turn2,
};
use crate::weld::{Closure, Weld, WeldReport, weld};

/// `(R^d − I)·v`.
fn rot_minus_id(v: [f32; 2], d: i32) -> [f32; 2] {
    let r = turn2(v, d);
    [r[0] - v[0], r[1] - v[1]]
}

/// `M·v`, com `M = R^a·(R^d − I)` — a matriz da holonomia de um vértice singular.
fn m_mul(v: [f32; 2], a: i32, d: i32) -> [f32; 2] {
    turn2(rot_minus_id(v, d), a)
}

/// `Mᵀ·v = (R^{−d} − I)·R^{−a}·v`.
fn mt_mul(v: [f32; 2], a: i32, d: i32) -> [f32; 2] {
    rot_minus_id(turn2(v, -a), -d)
}

/// O vértice singular visto pelo relaxador.
#[derive(Debug, Clone, Copy)]
struct Sing {
    seam: u32,
    jump: i32,
    /// O expoente `a` de `M = R^a·(R^d − I)`.
    a: i32,
    /// O defeito de rotação, em quartos de volta.
    d: i32,
    copies: [u32; 2],
}

impl Sing {
    fn of(w: &Weld, class: usize) -> Option<Self> {
        let (c, d): (&Closure, i32) = w.singular_class(class)?;
        Some(Self {
            seam: c.seam,
            jump: c.jump,
            a: (c.jump + w.rot_of(c.copies[0])).rem_euclid(4),
            d,
            copies: c.copies,
        })
    }

    /// ⭐ `MᵀM = κ·I`, e `κ` sai da conta: `2I − (R^d + R^{−d})`.
    fn kappa(self) -> f32 {
        if self.d == 2 { 4.0 } else { 2.0 }
    }
}

/// Quantas rondas de Gauss–Seidel o sistema soldado gasta.
///
/// ⭐⭐ **É `20×` menos que o penalizado, e o número sai de medição** — ver a sonda
/// `the_welded_system_converges_faster_than_the_penalised_one`. O sistema penalizado é
/// mal condicionado **por causa do peso** (`SEAM_WEIGHT` grande = rigidez artificial);
/// sem ele o sistema é a Poisson pura, e Gauss–Seidel percorre-a depressa.
pub const ROUNDS: usize = 8_000;

/// Quantas passagens o assentamento das translações derivadas faz por varredura.
///
/// ⚠️ **A dependência entre fechos é um GRAFO** (a translação que um escreve entra nos
/// caminhos de outro), então uma passagem só não a resolve em geral. ⭐ O número sai da
/// sonda `the_settling_of_derived_shifts_converges`, que varre `1..8` e imprime o
/// resíduo — *não de um palpite sobre a profundidade do grafo*.
pub const SETTLE_PASSES: usize = 4;

/// Passagens de ajuste do calibre por varredura.
pub const GAUGE_PASSES: usize = 2;

/// O que o solver soldado mediu de si próprio, além das réguas do mapa.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WeldSolveReport {
    /// As réguas do mapa — as mesmas do [`crate::solve`].
    pub solve: SolveReport,
    /// A estrutura da soldadura.
    pub weld: WeldReport,
    /// Rondas gastas.
    pub rounds: usize,
    /// ⭐ O maior movimento da última ronda — a régua da convergência.
    pub last_move: f32,
}

/// ⭐ **O RELAXADOR SOLDADO** — o sistema reduzido, uma variável de cada vez.
pub(crate) struct WeldRelaxer<'a> {
    a: &'a Assembly,
    w: &'a Weld,
    /// Por classe, o denominador somado das cópias dela.
    den: Vec<f32>,
    /// ⭐ Por classe, que componentes estão **pregadas**.
    ///
    /// ⚠️ **Por componente e não por classe:** o guloso prega `u` e `v` em momentos
    /// diferentes, e congelar as duas de uma vez faria a segunda deixar de poder
    /// absorver o passo da primeira.
    frozen: Vec<[bool; 2]>,
    /// Por costura, que componentes da translação estão pregadas.
    shift_frozen: Vec<[bool; 2]>,
    /// Quais das duas eliminações extra estão ligadas.
    pub(crate) opts: WeldOptions,
    /// Por classe, as classes vizinhas — a fila do degrau local anda por aqui.
    neigh: Vec<Vec<u32>>,
    /// Por costura, `(patch_a, patch_b, salto)` — o que o calibre precisa.
    sides: Vec<Option<(u32, u32, i32)>>,
    /// Por patch, o deslocamento de calibre.
    gauge: std::cell::RefCell<Vec<[f32; 2]>>,
}

impl<'a> WeldRelaxer<'a> {
    pub(crate) fn new(a: &'a Assembly, w: &'a Weld, cut: &CutMesh, combed: &Combed) -> Self {
        let seams = cut.seams.len();
        let nc = w.classes();
        let mut den = vec![0.0f32; nc];
        for c in 0..nc {
            for ((p, l), _) in w.members(c) {
                den[c] += a.denom[p as usize][l as usize];
            }
        }
        // ⚠️ A vizinhança é entre CLASSES: duas classes são vizinhas se alguma cópia de
        // uma partilha um triângulo com alguma cópia da outra. *É a vizinhança do
        // sistema reduzido, e não a da malha cortada.*
        let mut neigh: Vec<Vec<u32>> = vec![Vec::new(); nc];
        for c in 0..nc {
            for ((p, l), _) in w.members(c) {
                let (p, l) = (p as usize, l as usize);
                for &ti in &a.by_vert[p][l] {
                    for v in a.tris[p][ti as usize].v {
                        if v as usize == l {
                            continue;
                        }
                        if let Some((other, _)) = w.of(p, v as usize) {
                            #[allow(clippy::cast_possible_truncation)]
                            neigh[c].push(other as u32);
                        }
                    }
                }
            }
            neigh[c].sort_unstable();
            neigh[c].dedup();
        }
        let sides: Vec<Option<(u32, u32, i32)>> = cut
            .seams
            .iter()
            .enumerate()
            .map(|(s, seam)| {
                combed
                    .jump
                    .get(s)
                    .copied()
                    .flatten()
                    .map(|k| (seam.side[0].patch, seam.side[1].patch, k))
            })
            .collect();
        Self {
            a,
            w,
            den,
            frozen: vec![[false; 2]; nc],
            shift_frozen: vec![[false; 2]; seams],
            opts: WeldOptions::default(),
            neigh,
            sides,
            gauge: std::cell::RefCell::new(vec![[0.0f32; 2]; a.denom.len()]),
        }
    }

    /// ⭐⭐⭐ **PUXA AS TRANSLAÇÕES PARA O SUBESPAÇO DE CALIBRE.**
    ///
    /// Ajusta `o_p` por mínimos quadrados ao `t` de agora e depois **escreve**
    /// `t_s = o_b − R^k·o_a`. ⚠️ *A costura de um vértice singular fica de fora* — a
    /// translação dela é escolhida pela imagem daquele vértice, e sobrescrevê-la aqui
    /// seria a segunda lei sobre a mesma variável (o defeito que o `settle_flat`
    /// tinha).
    pub(crate) fn pull_to_gauge(&self, map: &mut GridMap, passes: usize) {
        let mut o = self.gauge.borrow_mut();
        for _ in 0..passes {
            for p in 0..o.len() {
                let (mut acc, mut n) = ([0.0f32; 2], 0.0f32);
                for (s, side) in self.sides.iter().enumerate() {
                    let Some((pa, pb, k)) = *side else { continue };
                    if self.w.is_singular_seam(s) {
                        continue;
                    }
                    let t = map.shift[s];
                    #[allow(clippy::cast_possible_truncation)]
                    let p32 = p as u32;
                    if pa == p32 {
                        // `R^k·o_p = o_b − t`  ⇒  `o_p = R^{−k}(o_b − t)`
                        let v = turn2([o[pb as usize][0] - t[0], o[pb as usize][1] - t[1]], -k);
                        acc[0] += v[0];
                        acc[1] += v[1];
                        n += 1.0;
                    }
                    if pb == p32 {
                        // `o_p = t + R^k·o_a`
                        let v = turn2(o[pa as usize], k);
                        acc[0] += t[0] + v[0];
                        acc[1] += t[1] + v[1];
                        n += 1.0;
                    }
                }
                if n > 0.0 {
                    o[p] = [acc[0] / n, acc[1] / n];
                }
            }
        }
        for (s, side) in self.sides.iter().enumerate() {
            let Some((pa, pb, k)) = *side else { continue };
            if self.w.is_singular_seam(s) || self.shift_frozen[s] != [false; 2] {
                continue;
            }
            let r = turn2(o[pa as usize], k);
            map.shift[s] = [o[pb as usize][0] - r[0], o[pb as usize][1] - r[1]];
            for &c in self.w.shift_classes(s) {
                self.w.derive(map, c as usize);
            }
        }
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
    /// ⚠️ Uma classe **singular** tem lei própria ([`Self::relax_singular`]): a volta à
    /// roda dela determina-a, e a variável livre passa a ser onde ela cai.
    pub(crate) fn relax_class(&self, map: &mut GridMap, class: usize) -> f32 {
        if self.opts.singular {
            if let Some(sing) = Sing::of(self.w, class) {
                return self.relax_singular(map, class, sing);
            }
        }
        let den = self.den[class];
        if den <= 0.0 {
            return 0.0;
        }
        let f = self.frozen[class];
        if f[0] && f[1] {
            return 0.0;
        }
        let mut acc = [0.0f32; 2];
        for ((p, l), rot) in self.w.members(class) {
            let (r, _) = self.residual(map, p as usize, l as usize);
            let rr = turn2(r, -rot);
            acc[0] += rr[0];
            acc[1] += rr[1];
        }
        let d = [
            if f[0] { 0.0 } else { acc[0] / den },
            if f[1] { 0.0 } else { acc[1] / den },
        ];
        let y = self.w.value(map, class);
        self.w.set(map, class, [y[0] + d[0], y[1] + d[1]]);
        d[0].abs().max(d[1].abs())
    }

    /// ⭐⭐⭐ **RELAXA UM VÉRTICE SINGULAR** — a classe e a translação da costura do
    /// fecho são **uma variável só**.
    ///
    /// A volta à roda dá `M·y = R^k·off_a + t − off_b`, com `M = R^b − R^a` invertível.
    /// ⇒ escolher `y` **escolhe** `t`:
    ///
    /// ```text
    ///     t_s = M·y + q,      q = off_b − R^k·off_a
    /// ```
    ///
    /// ⭐⭐ **E é por isso que pregar a imagem num inteiro torna a translação inteira**:
    /// `M` leva inteiros a inteiros. *O contrário — deixar `t` livre e derivar `y` —
    /// põe a imagem num meio-inteiro, porque `M⁻¹` tem um meio.*
    ///
    /// ⭐ **O denominador continua escalar**, e isso não é sorte: `MᵀM = κ·I` com
    /// `κ = 2` (defeito de um quarto ou três quartos) ou `4` (meia volta). *Se `M` não
    /// fosse uma semelhança, este passo seria um sistema `2×2` cheio.*
    fn relax_singular(&self, map: &mut GridMap, class: usize, s: Sing) -> f32 {
        let f = self.frozen[class];
        if f[0] && f[1] {
            return 0.0;
        }
        let (mut acc, mut den) = ([0.0f32; 2], 0.0f32);
        for ((p, l), rot) in self.w.members(class) {
            let (r, d) = self.residual(map, p as usize, l as usize);
            let rr = turn2(r, -rot);
            acc[0] += rr[0];
            acc[1] += rr[1];
            den += d;
        }
        let kappa = s.kappa();
        for &(c, m) in self.w.crossings(s.seam as usize) {
            let (p, l) = self.w.where_is(c);
            let (r, d) = self.residual(map, p as usize, l as usize);
            let rr = mt_mul(turn2(r, -m), s.a, s.d);
            acc[0] += rr[0];
            acc[1] += rr[1];
            den += kappa * d;
        }
        if den <= 0.0 {
            return 0.0;
        }
        let y = self.w.value(map, class);
        let step = [
            if f[0] { 0.0 } else { acc[0] / den },
            if f[1] { 0.0 } else { acc[1] / den },
        ];
        self.write_singular(map, class, s, [y[0] + step[0], y[1] + step[1]]);
        step[0].abs().max(step[1].abs())
    }

    /// `q = off_b − R^k·off_a` — lido do mapa pondo a classe em zero.
    ///
    /// ⚠️ **Ele depende das OUTRAS translações** (os caminhos daquela classe
    /// atravessam-nas), então relê-se sempre. *Guardá-lo uma vez seria congelar o que a
    /// vizinhança move.*
    fn q_of(&self, map: &mut GridMap, class: usize, s: Sing) -> [f32; 2] {
        let y = self.w.value(map, class);
        self.w.set(map, class, [0.0, 0.0]);
        let (pa, la) = self.w.where_is(s.copies[0]);
        let (pb, lb) = self.w.where_is(s.copies[1]);
        let off_a = turn2(map.uv[pa as usize][la as usize], s.jump);
        let off_b = map.uv[pb as usize][lb as usize];
        self.w.set(map, class, y);
        [off_b[0] - off_a[0], off_b[1] - off_a[1]]
    }

    /// ⭐ **Escreve a imagem de um vértice singular** — e a translação que ela escolhe.
    pub(crate) fn write_singular_at(&self, map: &mut GridMap, class: usize, y: [f32; 2]) {
        if let Some(s) = Sing::of(self.w, class) {
            self.write_singular(map, class, s, y);
        } else {
            self.w.set(map, class, y);
        }
    }

    fn write_singular(&self, map: &mut GridMap, class: usize, s: Sing, y: [f32; 2]) {
        let q = self.q_of(map, class, s);
        self.w.set(map, class, y);
        let m = m_mul(y, s.a, s.d);
        map.shift[s.seam as usize] = [m[0] + q[0], m[1] + q[1]];
        for &c in self.w.shift_classes(s.seam as usize) {
            self.w.derive(map, c as usize);
        }
        self.w.derive(map, class);
    }

    /// ⭐⭐⭐ **RELAXA A TRANSLAÇÃO DE UMA COSTURA** — a mesma lei, outras cópias.
    pub(crate) fn relax_shift(&self, map: &mut GridMap, seam: usize) -> f32 {
        let f = self.shift_frozen[seam];
        // ⛔ Uma translação DERIVADA não é variável — quem a escreve é o fecho que a
        // possui ([`Weld::settle`]). *Relaxá-la seria dar-lhe duas leis.*
        let derived = (self.opts.settle_flat && self.w.is_flat_derived(seam))
            || (self.opts.singular && self.w.is_singular_seam(seam));
        if (f[0] && f[1]) || derived {
            return 0.0;
        }
        let (mut acc, mut den) = ([0.0f32; 2], 0.0f32);
        for &(c, m) in self.w.crossings(seam) {
            let (p, l) = self.w.where_is(c);
            let (r, d) = self.residual(map, p as usize, l as usize);
            if d <= 0.0 {
                continue;
            }
            let rr = turn2(r, -m);
            acc[0] += rr[0];
            acc[1] += rr[1];
            den += d;
        }
        if den <= 0.0 {
            return 0.0;
        }
        let d = [
            if f[0] { 0.0 } else { acc[0] / den },
            if f[1] { 0.0 } else { acc[1] / den },
        ];
        map.shift[seam][0] += d[0];
        map.shift[seam][1] += d[1];
        self.rederive(map, seam);
        d[0].abs().max(d[1].abs())
    }

    /// Re-escreve as cópias que aquela translação desloca.
    pub(crate) fn rederive(&self, map: &mut GridMap, seam: usize) {
        for &c in self.w.shift_classes(seam) {
            self.w.derive(map, c as usize);
        }
    }

    /// Prega uma componente de uma classe.
    pub(crate) fn freeze_class(&mut self, class: usize, ax: usize) {
        self.frozen[class][ax] = true;
    }

    /// Prega uma componente da translação de uma costura.
    pub(crate) fn freeze_shift(&mut self, seam: usize, ax: usize) {
        self.shift_frozen[seam][ax] = true;
    }

    /// As classes vizinhas de uma classe.
    pub(crate) fn neighbours(&self, class: usize) -> &[u32] {
        &self.neigh[class]
    }

    /// As classes que a translação de uma costura desloca.
    pub(crate) fn touched_by(&self, seam: usize) -> &[u32] {
        self.w.shift_classes(seam)
    }

    /// Uma varredura global — devolve o maior movimento.
    pub(crate) fn sweep(&self, map: &mut GridMap, seams: usize) -> f32 {
        let mut worst = 0.0f32;
        for c in 0..self.w.classes() {
            worst = worst.max(self.relax_class(map, c));
        }
        for s in 0..seams {
            worst = worst.max(self.relax_shift(map, s));
        }
        if self.opts.settle_flat {
            self.w.settle(map, SETTLE_PASSES);
        }
        if self.opts.gauge {
            self.pull_to_gauge(map, GAUGE_PASSES);
        }
        worst
    }
}

/// ⭐ **AS DUAS ELIMINAÇÕES QUE A LEI PERMITE ALÉM DAS CÓPIAS** — mediveis, porque
/// nenhuma delas é óbvia (`CLAUDE.md` §0.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeldOptions {
    /// Um fecho **plano** determina a translação da costura dele.
    pub settle_flat: bool,
    /// Um fecho que **roda** determina a classe (o vértice singular), e ela escolhe a
    /// translação.
    pub singular: bool,
    /// ⭐ As translações vivem no subespaço de **CALIBRE** (`t_s = o_b − R^k·o_a`).
    ///
    /// ⚠️ **É a condição dos fechos planos imposta por CONSTRUÇÃO, e não por
    /// projecção alternada.** A volta a um canto regular telescopa: com `t` desta
    /// forma, a composição à roda do leque dá `o − R^{Σk}·o`, que é **zero** quando a
    /// volta não roda. *A condição que o `settle_flat` tentava impor equação a equação
    /// é a que esta parametrização torna inexprimível violar.*
    ///
    /// ⛔ Ela proíbe holonomia de translação em ciclos NÃO contráteis (a asa de um
    /// toro), e é por isso que o número tem de ser medido em cada peça, não assumido.
    pub gauge: bool,
}

impl Default for WeldOptions {
    /// ⛔⛔ **`settle_flat` NASCE DESLIGADO, e é uma recusa MEDIDA** — não uma
    /// preferência. Derivar a translação de um fecho **plano** é algebricamente legítimo
    /// e numericamente instável: a translação é partilhada por toda a costura, então
    /// escrevê-la a partir de um vértice arrasta a costura inteira, e a varredura deixa
    /// de assentar. Medido a 8 000 rondas, contra a versão só-cópias:
    ///
    /// | peça | ângulo | fechos que rodam | fechos planos | passo |
    /// |---|---|---|---|---|
    /// | esfera, só cópias | ⭐ `3,83°` | `6,16` | `9,91` | ⭐ `1,1e-6` |
    /// | esfera, + planos | ⛔ `10,49°` | ⛔ `22,89` | ⛔ `17,47` | ⛔ `2,9e-1` |
    /// | toro, + planos | `3,58°` | ⛔⛔ **`567,1`** | ⛔⛔ **`568,5`** | ⛔ `2,9e-1` |
    ///
    /// ⭐ A eliminação **singular** fica ligada, e paga o que lhe compete: o resíduo dos
    /// fechos que rodam vai de `6,16` · `3,55` · `2,80` para **`0,14`** · **`0,07`** ·
    /// **`0,42`** nas três peças.
    fn default() -> Self {
        Self {
            settle_flat: false,
            singular: true,
            gauge: false,
        }
    }
}

/// ⭐⭐⭐ **RESOLVE O MAPA COM AS COSTURAS ELIMINADAS.**
#[must_use]
pub fn solve_welded(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    h: f32,
    rounds: usize,
) -> (GridMap, WeldSolveReport) {
    solve_welded_with(mesh, cut, combed, h, rounds, WeldOptions::default())
}

/// ⭐ **O MESMO, com as duas eliminações explícitas** — ver [`WeldOptions`].
#[must_use]
pub fn solve_welded_with(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    h: f32,
    rounds: usize,
    opts: WeldOptions,
) -> (GridMap, WeldSolveReport) {
    let mut rep = WeldSolveReport::default();
    let (w, wrep) = weld(cut, combed);
    rep.weld = wrep;
    let a = assemble(mesh, cut, combed, h, &mut rep.solve);
    let mut r = WeldRelaxer::new(&a, &w, cut, combed);
    r.opts = opts;
    let mut map = GridMap {
        uv: cut
            .origin
            .iter()
            .map(|o| vec![[0.0f32; 2]; o.len()])
            .collect(),
        shift: vec![[0.0; 2]; cut.seams.len()],
    };
    for round in 0..rounds {
        rep.last_move = r.sweep(&mut map, cut.seams.len());
        rep.rounds = round + 1;
    }
    measure(&a, cut, combed, &map, h, &mut rep.solve);
    crate::weld::holonomy(&w, &map, &mut rep.weld);
    (map, rep)
}

#[cfg(test)]
#[path = "weld_solve_tests.rs"]
mod tests;
