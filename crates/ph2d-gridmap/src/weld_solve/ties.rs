//! ⭐⭐⭐ **AS AMARRAS DOS ARCOS** — ligar, contar, e relaxar um grupo.
//!
//! ⚠️ **Módulo FILHO do [`super`], e não irmão — a escolha é de privacidade.** Um filho vê
//! os campos privados do pai, então este corte não alarga a visibilidade de **nada**;
//! um irmão obrigaria a `pub(crate)` em meia dúzia de campos internos do relaxador.
//! *O corte foi forçado pelo tecto de LOC; a linha do corte é a responsabilidade.*
//!
//! O que fica no pai são as leis que **não** são das amarras: a relaxação de uma classe,
//! a de uma incógnita livre, a varredura, e a [`super::WeldRelaxer::class_acc`] — que tem
//! **dois** leitores e por isso não pertence a nenhum dos lados.

use crate::solve::GridMap;
use crate::weld_flat::Var;

use super::WeldRelaxer;

impl WeldRelaxer<'_> {
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
            // ⛔⛔⛔ **A RAIZ ENTRA TAMBÉM, e a 1.ª redacção excluía-a.**
            //
            // Os outros membros saem da [`Self::relax_class`] por `driven`, e os que são
            // incógnitas LIVRES por `freeze_free`. ⚠️ *Uma raiz que seja classe **simples**
            // não é nem uma coisa nem outra* — e a `relax_class` continuava a escrevê-la
            // no eixo amarrado, com o denominador da classe **sozinha**, enquanto a
            // [`Self::relax_tie`] a escrevia com o do grupo. **Duas leis sobre o mesmo
            // escalar**, que é o defeito que a obra A mediu.
            //
            // ⚠️ Marcar a raiz é inócuo quando ela é livre (o `freeze_free` já a tinha
            // travado), e a `relax_tie` escreve-a por [`Weld::set`]/`bump` de qualquer
            // modo — `driven` só fecha a porta da `relax_class`.
            if self.free_index_class(root as usize / 2).is_none() {
                self.plain_roots += 1;
            }
            for &(x, _, _) in &rows {
                self.driven[x as usize / 2][x as usize % 2] = true;
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

    /// ⭐⭐ Quantos grupos têm RAIZ de classe simples — a população em que a `relax_class`
    /// era a segunda lei sobre o escalar amarrado.
    pub(crate) fn plain_roots(&self) -> usize {
        self.plain_roots
    }

    /// ⭐⭐⭐ **O SISTEMA NORMAL DA COORDENADA DE UMA AMARRA** — `(g, H, H_fingida)`.
    ///
    /// ⛔⛔⛔ **Medido (2026-08-27): o denominador FINGIDO era o `NaN` da `sphere_uv`.**
    /// A 1.ª redacção somava `Σ den[classe]` — a curvatura de cada membro **em
    /// isolamento**. ⚠️ Mas por [`Self::attach_ties`] (e pelo `ACHADO` §23.17) *os cantos
    /// dos arcos SÃO os cones, e um cone é incógnita LIVRE do sistema dos fechos*: mexê-lo
    /// move também **todas as dependentes a jusante**, cuja curvatura não estava na soma.
    /// O passo saía `H/H_fingida` vezes maior que o mínimo ⇒ **ganho > 1**, e uma
    /// realimentação de ganho > 1 diverge — que é exactamente o que a escada mediu
    /// (`degrau1 0`, `560 028` visitas, `6` pregos com passo não-finito).
    ///
    /// ⚠️ *A [`Self::relax_free`] já traz esta lei escrita uma função abaixo* — «a versão
    /// que fingia denominador escalar divergia». **Escrevi-a outra vez, e paguei outra vez.**
    ///
    /// A coordenada da amarra desloca o membro `x` de `σ_x` por unidade. Por **cópia**, a
    /// coluna acumulada é `u = Σ_x σ_x · J[:, ax_x]`, e daí `H = Σ den·|u|²`, `g = Σ u·r`.
    ///
    /// ⚠️⚠️ **Isto NÃO é a curvatura exacta, e a distinção foi MEDIDA** (gate
    /// `the_tie_denominator_never_falls_below_the_effective_curvature`): o numerador de
    /// Poisson de uma cópia depende das **vizinhas**, então mover muitas cópias de uma vez
    /// tem termos cruzados **negativos** que esta soma ignora. Na fixtura da esfera o `H`
    /// daqui é `1,46×` a curvatura efectiva ⇒ o passo fica **curto**.
    ///
    /// ⭐⭐⭐ **E é exactamente por isso que ele é seguro:** um denominador ACIMA da
    /// curvatura sub-relaxa (lento, convergente); um ABAIXO sobre-relaxa, e a `ω > 2`
    /// diverge. *Errar para cima é lento; errar para baixo é `inf`.*
    ///
    /// ⚠️ A [`Self::relax_free`] herda a mesma aproximação — o doc dela diz «minimização
    /// exacta ao longo daquela coordenada» e está optimista pela mesma razão: ela move a
    /// livre **e** todas as dependentes ao mesmo tempo.
    ///
    /// ⚠️ Um membro que **não** é livre é escrito em cheio no quadro da classe: para ele
    /// a soma de [`Self::class_acc`] já é exacta, e entra igual nas duas Hessianas.
    pub(crate) fn tie_normal(&self, map: &GridMap, g: usize) -> Option<(f32, f32, f32)> {
        let (_, rows) = self.ties.get(g)?;
        let mut cols: std::collections::BTreeMap<u32, [f32; 2]> = std::collections::BTreeMap::new();
        let (mut num, mut h_plain, mut h_pretend) = (0.0f32, 0.0f32, 0.0f32);
        for &(x, sigma, _) in rows {
            let (c, ax) = (x as usize / 2, x as usize % 2);
            if let Some(i) = self.free_index_class(c) {
                for &(cp, j) in self.sys.touched(i) {
                    let e = cols.entry(cp).or_insert([0.0f32; 2]);
                    e[0] += sigma * j[0][ax];
                    e[1] += sigma * j[1][ax];
                }
                h_pretend += self.den[c];
            } else {
                let (acc, d) = self.class_acc(map, c);
                if d <= 0.0 {
                    continue;
                }
                num += sigma * acc[ax];
                h_plain += d;
            }
        }
        let mut h_free = 0.0f32;
        for (&cp, u) in &cols {
            let (p, l) = self.w.where_is_pub(cp);
            let (r, d) = self.residual(map, p as usize, l as usize);
            if d <= 0.0 {
                continue;
            }
            num += u[0].mul_add(r[0], u[1] * r[1]);
            h_free += d * u[0].mul_add(u[0], u[1] * u[1]);
        }
        let (h, hp) = (h_plain + h_free, h_plain + h_pretend);
        (h > 0.0).then_some((num, h, hp))
    }

    /// ⭐ **O GANHO que a redacção fingida aplicava** — `H / H_fingida`, por grupo.
    ///
    /// É a régua do defeito acima: `1,0` diz que o denominador escalar estava certo
    /// naquele grupo; acima de `1` diz **quantas vezes** o passo dele era maior que o
    /// deste. ⚠️ *Não é «quantas vezes estourava o mínimo»* — nenhum dos dois é a
    /// curvatura exacta (ver [`Self::tie_normal`]); é o rácio entre os dois denominadores.
    pub(crate) fn tie_gain(&self, map: &GridMap, g: usize) -> Option<f32> {
        let (_, h, hp) = self.tie_normal(map, g)?;
        (hp > 0.0).then_some(h / hp)
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
        let Some((num, den, _)) = self.tie_normal(map, g) else {
            return 0.0;
        };
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
}
