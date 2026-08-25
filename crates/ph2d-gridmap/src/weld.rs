//! ⭐⭐⭐ **A COSTURA SOLDADA — os dois lados deixam de ser duas variáveis.**
//!
//! # ⛔ A lei, e ela é a espinha
//!
//! *Uma restrição linear entra **eliminando** uma variável. Nunca como termo de
//! energia.* Um peso `w` produz sempre um compromisso: existe um `w` alto que fecha a
//! restrição e estraga o resto, e um `w` baixo que preserva o resto e deixa a restrição
//! aberta. ⛔ **Não existe `w` que faça as duas**, e isso está MEDIDO nesta casa —
//! ver a tabela do [`crate::solve::SEAM_WEIGHT`], onde `8` dá `2,9°` de ângulo com
//! `0,23` de rasgo e `512` dá `0,0006` de rasgo com `13°`–`17°` de ângulo.
//!
//! # O que esta fase faz
//!
//! Uma costura diz `z_b = R^k z_a + t`. ⇒ **`z_b` deixa de ser uma variável**: ele é o
//! valor de `z_a` transportado. Sobra **uma** variável por CLASSE de cópias — que, para
//! um vértice do corte, é o conjunto das cópias dele em todos os patches que lhe tocam.
//!
//! ⭐ **O resíduo daquela costura passa a ser zero por construção**, e não por peso: a
//! grandeza que o mediria deixou de existir.
//!
//! # ⚠️ O que NÃO se elimina, e por que ele tem de ser contado
//!
//! Uma classe com `n` cópias e `n − 1` ligações independentes elimina `n − 1`
//! variáveis. **A ligação seguinte fecha um ciclo** — não sobra variável para ela
//! eliminar, e ela passa a ser uma condição sobre a raiz:
//!
//! ```text
//!     (R^{rot_b} − R^{rot_a + k}) · y  =  R^k off_a + t − off_b
//! ```
//!
//! | o defeito de rotação | o que a condição é | quem é |
//! |---|---|---|
//! | `≠ 0` | `y` fica **determinado** — o ponto fixo da holonomia | um vértice **singular** |
//! | `= 0` | o lado esquerdo anula-se ⇒ a condição é sobre as **translações** | a compatibilidade da volta |
//!
//! ⛔ **Uma soldadura que não conta estas é uma soldadura que mente**: ela elimina o
//! que pode e deixa o resto a parecer fechado. [`WeldReport::closures`] é a contagem, e
//! [`WeldReport::holonomy_max`] é o que a segunda linha vale hoje.

use crate::comb::Combed;
use crate::cut::CutMesh;
use crate::solve::{GridMap, turn2};

/// ⛔ **UMA LIGAÇÃO QUE A SOLDADURA NÃO PÔDE ELIMINAR** — ela fecha um ciclo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Closure {
    /// A costura de onde ela veio.
    pub seam: u32,
    /// As duas cópias, no espaço plano desta soldadura.
    pub copies: [u32; 2],
    /// ⭐ **O DEFEITO DE ROTAÇÃO**, em quartos de volta: `(rot_a + k − rot_b) mod 4`.
    ///
    /// `0` = a volta fecha em rotação e o que sobra é a condição sobre as translações;
    /// `≠ 0` = a volta roda, e é a assinatura de um vértice **singular**.
    pub turn: i32,
    /// O salto de período da costura desta ligação.
    pub jump: i32,
}

/// Como uma cópia derivada se escreve a partir da anterior.
#[derive(Debug, Clone, Copy)]
struct Step {
    /// A cópia que este passo escreve.
    copy: u32,
    /// A cópia de onde ele a lê.
    from: u32,
    seam: u32,
    /// `true` = a cópia escrita é o lado `1` (`z_b = R^k z_a + t`).
    forward: bool,
    jump: i32,
}

/// ⭐⭐⭐ **A SOLDADURA** — quem é derivado de quem, e com que transporte.
#[derive(Debug, Clone, Default)]
pub struct Weld {
    /// Onde começa cada patch no espaço plano de cópias.
    base: Vec<u32>,
    /// Por cópia, `(patch, local)`.
    at: Vec<(u32, u32)>,
    /// Por cópia, a classe de que ela é imagem.
    class: Vec<u32>,
    /// Por cópia, os quartos de volta que levam a classe até esta cópia.
    rot: Vec<i32>,
    /// Por classe, a cópia **raiz** — a que carrega a variável.
    roots: Vec<u32>,
    /// Os passos de derivação, agrupados por classe.
    steps: Vec<Step>,
    /// Por classe, a fatia de [`Self::steps`] que lhe pertence.
    span: Vec<(u32, u32)>,
    /// ⛔ As ligações que fecham ciclo — ver [`Closure`].
    pub closures: Vec<Closure>,
    /// ⭐ Por costura, as cópias cuja derivação a ATRAVESSA, com a rotação do
    /// coeficiente: `∂z_cópia / ∂t_costura = R^m`.
    ///
    /// ⚠️ **É o que torna a translação uma variável como as outras.** Sem esta lista
    /// não há como escrever `∂E/∂t = 0`, e a translação voltaria a ser ajustada pela
    /// média de um resíduo — que, com a costura eliminada, é **zero por construção** e
    /// portanto não diz nada. *A grandeza que a movia deixou de existir.*
    crossings: Vec<Vec<(u32, i32)>>,
    /// Por cópia, a forma linear de `off` — ver [`Weld::cross_of`].
    cross: Vec<Vec<(u32, i32)>>,
    /// Por costura, as classes que a translação dela desloca.
    shift_classes: Vec<Vec<u32>>,
}

/// O que a soldadura mediu de si própria.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WeldReport {
    /// Cópias ao todo — o número de variáveis **antes**.
    pub copies: usize,
    /// ⭐ Classes — o número de variáveis **depois**.
    pub classes: usize,
    /// Ligações de costura ao todo (pares casados).
    pub links: usize,
    /// ⭐ Ligações que ELIMINARAM uma variável.
    pub eliminated: usize,
    /// ⛔ Ligações que fecham ciclo e por isso não eliminaram nada.
    pub closures: usize,
    /// ⚠️ Dessas, as que **rodam** — a assinatura de um vértice singular.
    pub turning: usize,
    /// ⚠️ Dessas, as que fecham em rotação: o que sobra é a condição sobre as
    /// translações.
    pub flat: usize,
    /// ⭐ **A HOLONOMIA DE TRANSLAÇÃO** das ligações planas, em células: o pior.
    ///
    /// ⚠️ *É a grandeza que uma soldadura não pode fechar por eliminação nenhuma* — ela
    /// é uma condição sobre as translações, e quem a tem de honrar é o arredondamento.
    pub holonomy_max: f32,
    /// A mediana da mesma grandeza.
    pub holonomy_p50: f32,
    /// ⛔ Costuras sem salto de período — não entram na soldadura.
    pub loose: usize,
}

impl Weld {
    /// Quantas classes — o número de variáveis do sistema soldado.
    #[must_use]
    pub fn classes(&self) -> usize {
        self.roots.len()
    }

    /// A cópia raiz de uma classe.
    #[must_use]
    pub fn root(&self, class: usize) -> Option<(u32, u32)> {
        self.at.get(*self.roots.get(class)? as usize).copied()
    }

    /// De que classe uma cópia é imagem, e com que rotação.
    #[must_use]
    pub fn of(&self, patch: usize, local: usize) -> Option<(usize, i32)> {
        let c = self.copy_of(patch, local)?;
        Some((*self.class.get(c as usize)? as usize, self.rot[c as usize]))
    }

    /// As cópias de uma classe, com a rotação de cada uma.
    pub(crate) fn members(&self, class: usize) -> impl Iterator<Item = ((u32, u32), i32)> + '_ {
        let root = self.roots[class];
        let (lo, hi) = self.span[class];
        std::iter::once((self.at[root as usize], self.rot[root as usize])).chain(
            self.steps[lo as usize..hi as usize]
                .iter()
                .map(move |s| (self.at[s.copy as usize], self.rot[s.copy as usize])),
        )
    }

    /// O valor da raiz de uma classe — porta pública.
    #[must_use]
    pub fn value_pub(&self, map: &GridMap, class: usize) -> [f32; 2] {
        self.value(map, class)
    }

    /// As cópias de uma classe, com a rotação de cada uma — porta pública.
    pub fn members_pub(&self, class: usize) -> impl Iterator<Item = ((u32, u32), i32)> + '_ {
        self.members(class)
    }

    /// A cópia (no espaço plano) de um `(patch, local)`.
    #[must_use]
    pub fn copy_index(&self, patch: usize, local: usize) -> Option<u32> {
        self.copy_of(patch, local)
    }

    /// A classe de uma cópia — porta pública.
    #[must_use]
    pub fn class_of_pub(&self, copy: u32) -> usize {
        self.class_of(copy)
    }

    /// Os fechos.
    #[must_use]
    pub fn closures(&self) -> &[Closure] {
        &self.closures
    }

    /// Os quartos de volta que levam a classe até uma cópia — porta pública.
    #[must_use]
    pub fn rot_of_pub(&self, copy: u32) -> i32 {
        self.rot[copy as usize]
    }

    /// ⭐⭐⭐ **A FORMA LINEAR de `off_c`** — de que translações a cópia depende, e com
    /// que rotação. `off_c = Σ R^m · t_s`, **sem termo constante**.
    ///
    /// ⚠️ É a mesma tabela das travessias, lida por cópia em vez de por costura. *Sem
    /// ela a condição de um fecho plano não é escrevível como equação — e o que não é
    /// escrevível como equação acaba imposto por alternância, que é o que divergiu.*
    #[must_use]
    pub fn cross_of(&self, copy: u32) -> &[(u32, i32)] {
        self.cross.get(copy as usize).map_or(&[], Vec::as_slice)
    }

    /// `(patch, local)` de uma cópia — a porta pública, para as sondas.
    #[must_use]
    pub fn where_is_pub(&self, copy: u32) -> (u32, u32) {
        self.at[copy as usize]
    }

    /// As classes que a translação de uma costura desloca — porta pública.
    #[must_use]
    pub fn shift_classes_pub(&self, seam: usize) -> &[u32] {
        self.shift_classes.get(seam).map_or(&[], Vec::as_slice)
    }

    pub fn crossings(&self, seam: usize) -> &[(u32, i32)] {
        self.crossings.get(seam).map_or(&[], Vec::as_slice)
    }

    /// A classe de uma cópia.
    pub(crate) fn class_of(&self, copy: u32) -> usize {
        self.class[copy as usize] as usize
    }

    fn copy_of(&self, patch: usize, local: usize) -> Option<u32> {
        let b = *self.base.get(patch)?;
        let c = b + u32::try_from(local).ok()?;
        (c < *self.base.get(patch + 1).unwrap_or(&u32::MAX)).then_some(c)
    }

    /// ⭐⭐⭐ **ESCREVE AS CÓPIAS DERIVADAS de uma classe**, a partir da raiz.
    ///
    /// ⚠️ **A expressão é a MESMA que a régua usa** (`z_b = R^k z_a + t`), e é por isso
    /// que o resíduo daquela ligação não pode ser outra coisa senão o erro de avaliação
    /// da própria substituição. *Escrever aqui uma forma algebricamente igual e
    /// numericamente diferente daria uma régua a medir a diferença entre duas contas.*
    pub fn derive(&self, map: &mut GridMap, class: usize) {
        let (lo, hi) = self.span[class];
        for s in &self.steps[lo as usize..hi as usize] {
            let (pf, lf) = self.at[s.from as usize];
            let (pc, lc) = self.at[s.copy as usize];
            let from = map.uv[pf as usize][lf as usize];
            let t = map.shift[s.seam as usize];
            map.uv[pc as usize][lc as usize] = if s.forward {
                let r = turn2(from, s.jump);
                [r[0] + t[0], r[1] + t[1]]
            } else {
                turn2([from[0] - t[0], from[1] - t[1]], -s.jump)
            };
        }
    }

    /// O valor da raiz de uma classe.
    pub(crate) fn value(&self, map: &GridMap, class: usize) -> [f32; 2] {
        let (p, l) = self.at[self.roots[class] as usize];
        map.uv[p as usize][l as usize]
    }

    /// Escreve a raiz e deriva o resto.
    pub fn set(&self, map: &mut GridMap, class: usize, y: [f32; 2]) {
        let (p, l) = self.at[self.roots[class] as usize];
        map.uv[p as usize][l as usize] = y;
        self.derive(map, class);
    }
}

/// ⭐⭐⭐ **SOLDA AS COSTURAS** — devolve quem é derivado de quem.
///
/// ⚠️ A ordem é **determinista** (a raiz de cada classe é a cópia de menor
/// `(patch, local)`, e a travessia é por fila): duas corridas sobre a mesma peça dão a
/// mesma árvore, senão o mapa deixaria de ser reprodutível.
#[must_use]
pub fn weld(cut: &CutMesh, combed: &Combed) -> (Weld, WeldReport) {
    let mut rep = WeldReport::default();
    let np = cut.origin.len();
    let mut base: Vec<u32> = Vec::with_capacity(np + 1);
    let mut at: Vec<(u32, u32)> = Vec::new();
    let mut acc = 0u32;
    for (p, o) in cut.origin.iter().enumerate() {
        base.push(acc);
        for l in 0..o.len() {
            #[allow(clippy::cast_possible_truncation)]
            at.push((p as u32, l as u32));
        }
        acc += u32::try_from(o.len()).unwrap_or(0);
    }
    base.push(acc);
    let n = at.len();
    rep.copies = n;

    // ── As ligações: um par casado de uma costura é uma aresta entre duas cópias.
    // ⭐ `(vizinho, costura, sentido, salto, id da ligação)` — o tipo tem nome porque o
    // lint pede, e o nome diz o que a tupla é.
    type Link = (u32, u32, bool, i32, u32);
    // ⚠️ **Cada ligação leva um ID**, e não é decoração: sem ele o fecho detecta-se pelo
    // SENTIDO (`forward`), e uma aresta percorrida como árvore no sentido inverso é
    // recontada como fecho quando o outro extremo a olha. ⛔ Medido: `469 + 30 ≠ 497`, e
    // as duas a mais apareciam depois como equações **órfãs** — restrições redundantes a
    // pedir uma variável que já não havia.
    let mut adj: Vec<Vec<Link>> = vec![Vec::new(); n];
    for (s, seam) in cut.seams.iter().enumerate() {
        let Some(k) = combed.jump.get(s).copied().flatten() else {
            rep.loose += 1;
            continue;
        };
        let Ok(sid) = u32::try_from(s) else { continue };
        let (pa, pb) = (seam.side[0].patch as usize, seam.side[1].patch as usize);
        for (la, lb) in seam.side[0].local.iter().zip(&seam.side[1].local) {
            let (Some(la), Some(lb)) = (la, lb) else {
                continue;
            };
            let (Some(ca), Some(cb)) = (base.get(pa).map(|b| b + la), base.get(pb).map(|b| b + lb))
            else {
                continue;
            };
            if ca as usize >= n || cb as usize >= n {
                continue;
            }
            // `z_b = R^k z_a + t` — de `a` para `b` é para a frente.
            #[allow(clippy::cast_possible_truncation)]
            let eid = rep.links as u32;
            adj[ca as usize].push((cb, sid, true, k, eid));
            adj[cb as usize].push((ca, sid, false, k, eid));
            rep.links += 1;
        }
    }

    let mut used = vec![false; rep.links];
    let mut class = vec![u32::MAX; n];
    let mut rot = vec![0i32; n];
    let mut roots: Vec<u32> = Vec::new();
    let mut steps: Vec<Step> = Vec::new();
    let mut span: Vec<(u32, u32)> = Vec::new();
    let mut closures: Vec<Closure> = Vec::new();

    for start in 0..n {
        if class[start] != u32::MAX {
            continue;
        }
        #[allow(clippy::cast_possible_truncation)]
        let cid = roots.len() as u32;
        #[allow(clippy::cast_possible_truncation)]
        roots.push(start as u32);
        class[start] = cid;
        rot[start] = 0;
        #[allow(clippy::cast_possible_truncation)]
        let lo = steps.len() as u32;
        let mut queue = std::collections::VecDeque::from([start]);
        while let Some(a) = queue.pop_front() {
            for &(b, seam, forward, k, eid) in &adj[a] {
                if used[eid as usize] {
                    continue;
                }
                let b = b as usize;
                if class[b] == u32::MAX {
                    used[eid as usize] = true;
                    class[b] = cid;
                    // ⭐ `z_b = R^k z_a + t` ⇒ `rot_b = rot_a + k`; ao contrário,
                    // `rot_a = rot_b − k`.
                    rot[b] = if forward { rot[a] + k } else { rot[a] - k };
                    steps.push(Step {
                        #[allow(clippy::cast_possible_truncation)]
                        copy: b as u32,
                        #[allow(clippy::cast_possible_truncation)]
                        from: a as u32,
                        seam,
                        forward,
                        jump: k,
                    });
                    queue.push_back(b);
                } else if class[b] == cid {
                    // ⛔ Fecha ciclo. ⚠️ **A ligação marca-se USADA** — é a identidade
                    // dela que a conta uma vez, não o sentido em que se olhou.
                    used[eid as usize] = true;
                    let (ca, cb) = if forward { (a, b) } else { (b, a) };
                    closures.push(Closure {
                        seam,
                        #[allow(clippy::cast_possible_truncation)]
                        copies: [ca as u32, cb as u32],
                        turn: (rot[ca] + k - rot[cb]).rem_euclid(4),
                        jump: k,
                    });
                }
            }
        }
        #[allow(clippy::cast_possible_truncation)]
        span.push((lo, steps.len() as u32));
    }

    // ── ⭐ AS TRAVESSIAS: de que translações cada cópia depende, e com que rotação.
    //
    // ⚠️ A lei é uma linha, e vale nos dois sentidos: `∂z_filho/∂z_pai = R^{rot_filho −
    // rot_pai}`. O termo PRÓPRIO é `R^0` a descer a costura (`z_b = R^k z_a + t`) e
    // `R^{rot−rot_pai+2}` a subi-la (`z_a = R^{−k}(z_b − t)`, e `−I = R²`).
    let mut cross: Vec<Vec<(u32, i32)>> = vec![Vec::new(); n];
    for st in &steps {
        let (c, f) = (st.copy as usize, st.from as usize);
        let delta = rot[c] - rot[f];
        let mut mine: Vec<(u32, i32)> = cross[f].iter().map(|&(s, m)| (s, m + delta)).collect();
        mine.push((st.seam, if st.forward { 0 } else { delta + 2 }));
        cross[c] = mine;
    }
    let mut crossings: Vec<Vec<(u32, i32)>> = vec![Vec::new(); cut.seams.len()];
    for (c, list) in cross.iter().enumerate() {
        for &(s, m) in list {
            #[allow(clippy::cast_possible_truncation)]
            crossings[s as usize].push((c as u32, m.rem_euclid(4)));
        }
    }
    for list in &mut cross {
        for e in list.iter_mut() {
            e.1 = e.1.rem_euclid(4);
        }
    }

    let mut shift_classes: Vec<Vec<u32>> = vec![Vec::new(); cut.seams.len()];
    for (s, list) in shift_classes.iter_mut().enumerate() {
        for &(c, _) in &crossings[s] {
            list.push(class[c as usize]);
        }
        list.sort_unstable();
        list.dedup();
    }

    // ── ⭐⭐⭐ CADA FECHO ELIMINA A TRANSLAÇÃO DA COSTURA EM QUE ASSENTA.
    //
    // ⚠️ **Uma costura só pode ser possuída por um fecho.** Um segundo fecho sobre a
    // mesma costura é uma restrição sem variável para eliminar — ele fica **órfão** e
    // é contado, nunca calado.
    // ⭐⭐⭐ **A DIRECÇÃO DA ELIMINAÇÃO NÃO É UMA ESCOLHA — a álgebra escolhe-a**, e
    // medi-la ao contrário custou uma corrida inteira (o ângulo foi de `3,8°` a `10,1°`
    // e a varredura deixou de assentar):
    //
    // | o fecho | a equação | quem está determinado |
    // |---|---|---|
    // | **plano** (`turn = 0`) | `0·y = R^k off_a + t − off_b` | a **translação** — o termo da classe cancela-se |
    // | **que roda** (`turn ≠ 0`) | `(R^b − R^a)·y = R^k off_a + t − off_b` | a **classe** — a matriz é invertível |
    //
    // ⛔ Derivar a TRANSLAÇÃO de um fecho que roda arrasta a costura inteira atrás de
    // uma classe que ainda está a relaxar-se, e as duas alternam sem assentar. *Uma
    // eliminação escrita no sentido errado não é uma eliminação: é uma projecção
    // alternada.*
    // ⚠️ Uma ligação de árvore aparece uma vez em `steps`; as restantes fecham ciclo.
    rep.classes = roots.len();
    rep.eliminated = steps.len();
    rep.closures = closures.len();
    rep.turning = closures.iter().filter(|c| c.turn != 0).count();
    rep.flat = rep.closures - rep.turning;

    (
        Weld {
            base,
            at,
            class,
            rot,
            roots,
            steps,
            span,
            closures,
            crossings,
            cross,
            shift_classes,
        },
        rep,
    )
}

/// ⭐⭐⭐ **O RESÍDUO DA COSTURA, SEPARADO POR ESPÉCIE.**
///
/// ⚠️ **A régua é INDEPENDENTE da construção**: ela relê `z_b − R^k z_a − t` do mapa
/// guardado, que é a mesma expressão do [`crate::solve::measure`] — *um gate que
/// perguntasse à soldadura se ela soldou seria tautológico*.
///
/// ⭐ Nas ligações ELIMINADAS o resultado não pode ser outra coisa senão o erro de
/// **avaliação** da própria substituição: a igualdade é exacta em ℝ, e o que sobra é a
/// diferença entre associar `(R^k z_a + t)` de uma maneira ou de outra em `f32`.
/// ⛔ *Não é uma folga; é o chão da representação.*
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SeamResidual {
    /// Ligações eliminadas medidas.
    pub links: usize,
    /// A mediana do resíduo das eliminadas, em células.
    pub p50: f32,
    /// ⭐ O pior resíduo de uma ligação ELIMINADA.
    pub max: f32,
    /// Ligações que fecham ciclo, medidas.
    pub closures: usize,
    /// ⛔ O pior resíduo de uma ligação que fecha ciclo — o que a eliminação não alcança.
    pub closure_max: f32,
    /// ⛔ O pior resíduo de um fecho que **roda** (um vértice singular).
    pub turning_max: f32,
    /// ⛔ O pior resíduo de um fecho **plano** (a compatibilidade da volta).
    pub flat_max: f32,
}

/// Mede [`SeamResidual`] sobre um mapa vivo.
#[must_use]
pub fn seam_residual(w: &Weld, map: &GridMap) -> SeamResidual {
    let mut out = SeamResidual::default();
    let mut v: Vec<f32> = Vec::with_capacity(w.steps.len());
    let one = |ca: u32, cb: u32, seam: u32, jump: i32| -> f32 {
        let (pa, la) = w.at[ca as usize];
        let (pb, lb) = w.at[cb as usize];
        let za = turn2(map.uv[pa as usize][la as usize], jump);
        let zb = map.uv[pb as usize][lb as usize];
        let t = map.shift[seam as usize];
        let d = [zb[0] - za[0] - t[0], zb[1] - za[1] - t[1]];
        d[0].mul_add(d[0], d[1] * d[1]).sqrt()
    };
    for st in &w.steps {
        let (ca, cb) = if st.forward {
            (st.from, st.copy)
        } else {
            (st.copy, st.from)
        };
        v.push(one(ca, cb, st.seam, st.jump));
    }
    for c in &w.closures {
        let r = one(c.copies[0], c.copies[1], c.seam, c.jump);
        out.closure_max = out.closure_max.max(r);
        if c.turn == 0 {
            out.flat_max = out.flat_max.max(r);
        } else {
            out.turning_max = out.turning_max.max(r);
        }
    }
    out.closures = w.closures.len();
    out.links = v.len();
    v.sort_by(f32::total_cmp);
    out.p50 = v.get(v.len() / 2).copied().unwrap_or(0.0);
    out.max = v.last().copied().unwrap_or(0.0);
    out
}

/// ⭐ **A HOLONOMIA DE TRANSLAÇÃO** das ligações que fecham em rotação, medida sobre um
/// mapa vivo.
///
/// ⚠️ **Ela não se lê da soldadura sozinha** — depende das translações, que o
/// arredondamento move. *Medi-la na construção seria medir o mapa contínuo e chamar-lhe
/// o mapa final.*
pub fn holonomy(w: &Weld, map: &GridMap, rep: &mut WeldReport) {
    let mut v: Vec<f32> = Vec::with_capacity(w.closures.len());
    for c in &w.closures {
        if c.turn != 0 {
            continue;
        }
        let (pa, la) = w.at[c.copies[0] as usize];
        let (pb, lb) = w.at[c.copies[1] as usize];
        let za = map.uv[pa as usize][la as usize];
        let zb = map.uv[pb as usize][lb as usize];
        let t = map.shift[c.seam as usize];
        let k = c.jump;
        let r = turn2(za, k);
        let d = [zb[0] - r[0] - t[0], zb[1] - r[1] - t[1]];
        v.push(d[0].abs().max(d[1].abs()));
    }
    v.sort_by(f32::total_cmp);
    rep.holonomy_p50 = v.get(v.len() / 2).copied().unwrap_or(0.0);
    rep.holonomy_max = v.last().copied().unwrap_or(0.0);
}

#[cfg(test)]
#[path = "weld_tests.rs"]
mod tests;
