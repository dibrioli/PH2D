//! ⭐⭐⭐ **OS LIMITES QUE O ALVO NÃO TEM** — o tecto de esticão, as âncoras de
//! longo alcance e a conservação de volume.
//!
//! Irmão (`#[path]`) do [`super`], cortado por ASSUNTO: o `verlet.rs` é a **lei
//! da referência** (a espec §5, portada e medida contra `103` fixtures do
//! oráculo); aqui vive o que ela **não tem** e o report de 2026-09-08 pediu.
//!
//! ⚠️⚠️ **O corte é o que faz a paridade continuar legível.** Tudo o que mora
//! neste ficheiro nasce NEUTRO ([`super::Solver::estica_max`] em `∞`,
//! [`super::Solver::volume`] em `0`) e, neutro, não corre uma instrução — é isso
//! que deixa as `86` fixtures do pincel e as `17` do filtro correrem byte a byte
//! como antes. *Um ficheiro que se pode apagar mentalmente e ainda ler a lei do
//! alvo é a forma mais barata de auditar a divergência.*
//!
//! ⚠️ **O gatilho do corte foi o `architecture_workspace_file_loc_cap`** (o
//! `verlet.rs` foi a `986` de um tecto de `700`), e ele estava **latente**: mora
//! em `ph2d-editor-core/tests/`, então nenhum fechamento por
//! `cargo test -p ph2d-cloth` o alcança. É a mesma família estrutural que esta
//! casa já registou seis vezes.
//!
//! # O que cada peça faz
//!
//! | | mecanismo | publicado em |
//! |---|---|---|
//! | tecto local | projecção de desigualdade por aresta, Jacobi com média | Provot, GI 1995 |
//! | âncoras de longo alcance | distância de material ao pino, uma passagem | Kim · Chentanez · Müller, SCA 2012 |
//! | volume | uma restrição escalar sobre a casca fechada | Müller et al., PBD 2007 §4.5 |
//!
//! ⇒ o mecanismo, as tabelas medidas e as recusas estão em
//! [`docs/3D/cloth/10`](../../../docs/3D/cloth/10_o_elastico_que_nao_para_e_o_volume.md).

use super::{Alvo, Solver, Verlet, norm, volume_de};

impl Verlet {
    /// ⭐⭐⭐ **LIGA A CONSERVAÇÃO DE VOLUME** — o chamador entrega os triângulos da
    /// peça e a lei mede o volume de repouso deles.
    ///
    /// ⚠️ **A ORIENTAÇÃO das faces é o sinal do volume**, e a lei não a corrige:
    /// uma peça com faces viradas para dentro dá `V₀ < 0`, e a restrição
    /// funciona na mesma (ela compara `V` com `k·V₀`, com o mesmo sinal dos dois
    /// lados). ⛔ O que ela NÃO tolera é `V₀ = 0`, que é uma superfície aberta —
    /// aí a razão não existe e a restrição desliga-se sozinha.
    pub fn conservar_volume(&mut self, caras: Vec<[u32; 3]>) {
        self.volume0 = volume_de(&self.repouso, &caras);
        self.caras = caras;
    }

    /// **A RESTRIÇÃO DE VOLUME** (PBD §4.5) — uma restrição ESCALAR sobre a peça
    /// inteira, projectada como todas as outras.
    ///
    /// `C = V(x) − k·V₀`, com `∇ᵢC = ⅙ Σ_{f ∋ i} (x_j × x_k)` (os dois outros
    /// vértices de cada face incidente, na ordem da face). A correcção é a
    /// projecção de mínimos quadrados `Δxᵢ = −C·wᵢ∇ᵢC / Σ wⱼ|∇ⱼC|²`.
    ///
    /// ⭐⭐ **É UMA restrição, não uma por face**, e é isso que a torna barata
    /// (`O(F)`) e global: ela distribui a diferença de volume por toda a peça de
    /// uma vez, o que nenhuma restrição local consegue.
    ///
    /// ⚠️ **O peso `wᵢ` é o `φ` da relaxação** — um vértice mascarado não paga
    /// pelo volume, e a peça sopra à volta dele.
    pub(super) fn restringir_volume(&mut self, k: f64) {
        if self.caras.is_empty() || self.volume0 == 0.0 || k <= 0.0 {
            return;
        }
        let n = self.x.len();
        if self.dx.len() != n {
            self.dx = vec![[0.0; 3]; n];
            self.dn = vec![0.0; n];
        }
        self.dx.fill([0.0; 3]);
        let mut v = 0.0;
        for f in &self.caras {
            let (a, b, c) = (
                self.x[f[0] as usize],
                self.x[f[1] as usize],
                self.x[f[2] as usize],
            );
            v += (a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0;
            // ∇ do vértice `f[0]` é ⅙ (b × c), e assim por diante em ciclo.
            for (i, (p, q)) in [(b, c), (c, a), (a, b)].into_iter().enumerate() {
                let g = [
                    (p[1] * q[2] - p[2] * q[1]) / 6.0,
                    (p[2] * q[0] - p[0] * q[2]) / 6.0,
                    (p[0] * q[1] - p[1] * q[0]) / 6.0,
                ];
                let vi = f[i] as usize;
                for (c2, gc) in g.iter().enumerate() {
                    self.dx[vi][c2] += gc;
                }
            }
        }
        // ⚠️⚠️ **`k` é a FORÇA, não um volume-alvo — e a diferença foi medida.**
        // A 1.ª redacção lia `k` como *«que fracção do volume de repouso manter»*
        // (`C = V − k·V₀`) e é ingovernável: com `k = 0,5` a peça não assenta em
        // metade, **colapsa a `6,5 %`** e o número de vincos duplica, porque a
        // restrição empurra para um alvo que a superfície não consegue ter com o
        // comprimento de material que tem. Como FORÇA ela é monótona: `0` não
        // faz nada, `1` conserva (medido `97,6 %`), e o meio é o meio.
        let erro = v - self.volume0;
        if erro == 0.0 {
            return;
        }
        let mut denom = 0.0;
        for i in 0..n {
            let w = if self.activo[i] { self.phi[i] } else { 0.0 };
            if w <= 0.0 {
                continue;
            }
            let g = self.dx[i];
            denom += w * (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]);
        }
        if denom <= 0.0 {
            return;
        }
        let lambda = k.clamp(0.0, 1.0) * erro / denom;
        for i in 0..n {
            let w = if self.activo[i] { self.phi[i] } else { 0.0 };
            if w <= 0.0 {
                continue;
            }
            for c in 0..3 {
                self.x[i][c] -= lambda * w * self.dx[i][c];
            }
        }
    }

    /// **O LIMITADOR DE ESTICÃO** — a desigualdade de Provot, sobre as restrições
    /// ESTRUTURAIS.
    ///
    /// Para cada par `(a,b)` com comprimento de repouso `ℓ`, se a razão
    /// `|x_a − x_b| / ℓ` sair da faixa `[piso, tecto]`, os dois extremos são
    /// levados de volta **exactamente** à borda mais próxima.
    ///
    /// ⚠️⚠️ **A correcção reparte-se pelo `φ`, e não meio-a-meio.** Um vértice
    /// mascarado ou congelado pela banda tem `φ = 0` e **não pode mover-se** — se
    /// a repartição fosse fixa, metade de cada correcção contra um pino
    /// evaporava, e o tecto deixava de valer justamente onde o pano é esticado.
    /// Com `φ_a/(φ_a+φ_b)` a correcção inteira vai para o lado que pode andar, e
    /// **degenera em meio-a-meio quando os dois são livres**.
    ///
    /// ⛔⛔⛔ **E ele é JACOBI COM MÉDIA, nunca Gauss-Seidel — a 1.ª redacção era
    /// Gauss-Seidel e AMPLIFICAVA o defeito que vinha curar.** Medido por uma
    /// sonda que imprimia o pior esticão da rede à entrada e à saída deste laço:
    /// no passo 115 de um gesto de gravidade sobre uma esfera ele **entrava a
    /// `4,50` e saía a `8,84`**.
    ///
    /// O mecanismo: a rede desta lei põe **~21 restrições sobre cada vértice**
    /// (as `6` arestas do anel-1 mais os `~15` pares dele). Uma projecção
    /// *inteira* por restrição, resolvida em sequência, faz cada vizinha desfazer
    /// a anterior e a soma das correcções sobre um vértice valer muitas vezes o
    /// que ele precisava — *é a divergência clássica de Gauss-Seidel sobre um
    /// sistema sobredeterminado com projecções duras*, e ela não aparece nas
    /// varreduras da §5.2 porque lá cada correcção já vem multiplicada por
    /// `RIGIDEZ = 0,6` e pelo `½`, que é sub-relaxação a fazer o mesmo trabalho.
    ///
    /// ⇒ acumula-se a correcção de TODAS as restrições num vector e aplica-se **a
    /// MÉDIA** delas por vértice. Uma média é combinação convexa, logo o passo
    /// nunca ultrapassa a mais exigente e o laço é incondicionalmente estável.
    ///
    /// ⚠️ **Só `Alvo::Vertice` entra.** As outras três espécies têm `ℓ = 0` (a
    /// âncora, o pino e a memória apontam para um ponto, não para uma distância)
    /// e uma razão contra zero não existe.
    ///
    /// ⚠️ **O `ℓ` lido é o mesmo das varreduras**, `τ` incluído — senão o `Expand`
    /// (que desloca o repouso) seria limitado contra um comprimento que a lei já
    /// não usa.
    /// ⭐⭐⭐ **SEMEIA O MAPA DE ÂNCORAS** — Dijkstra multi-origem sobre a REDE DE
    /// RESTRIÇÕES, com peso `ℓ`.
    ///
    /// ⚠️⚠️ **O grafo é o das restrições, não o das arestas da malha, e isso é
    /// mais CERTO e mais apertado:** esta lei liga cada vértice a todos os pares
    /// do anel-1 dele, e uma «diagonal de `2h`» é material tanto quanto uma
    /// aresta — um caminho que passe por ela é um caminho que o pano tem. Usar só
    /// as arestas daria uma distância MAIOR, e portanto um tecto mais frouxo.
    ///
    /// ⚠️ **Âncora é «o que a lei não pode mover»** (`φ = 0`): na área *Global* —
    /// a do filtro, a única onde o limitador nasce ligado — isso é exactamente a
    /// máscara do artista, porque a banda vale `1` em toda a peça.
    ///
    /// ⛔ **Sem uma única âncora o mapa fica vazio e a lei desliga-se**, e está
    /// certo: sem nada preso, a gravidade sobre a peça inteira é uma TRANSLAÇÃO
    /// rígida — não há esticão nenhum a limitar.
    fn semear_lra(&mut self) {
        let n = self.x.len();
        self.lra_dist = vec![f64::INFINITY; n];
        self.lra_raiz = vec![u32::MAX; n];
        self.lra_em = self.restricoes.len();
        // Adjacência da rede, pesada por `ℓ`.
        let mut viz: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
        for r in &self.restricoes {
            let Alvo::Vertice(b) = r.b else { continue };
            if r.l <= 0.0 {
                continue;
            }
            viz[r.a as usize].push((b, r.l));
            viz[b as usize].push((r.a, r.l));
        }
        // As origens: quem a lei não move.
        let mut fila = std::collections::BinaryHeap::new();
        let mut ancoras = 0usize;
        for v in 0..n {
            if self.phi[v] > 0.0 {
                continue;
            }
            ancoras += 1;
            self.lra_dist[v] = 0.0;
            self.lra_raiz[v] = u32::try_from(v).unwrap_or(u32::MAX);
            fila.push((
                std::cmp::Reverse(ordenavel(0.0)),
                u32::try_from(v).unwrap_or(u32::MAX),
            ));
        }
        if ancoras == 0 {
            self.lra_dist.clear();
            self.lra_raiz.clear();
            return;
        }
        while let Some((std::cmp::Reverse(d), v)) = fila.pop() {
            let d = de_ordenavel(d);
            let vi = v as usize;
            if d > self.lra_dist[vi] {
                continue;
            }
            for &(w, l) in &viz[vi] {
                let wi = w as usize;
                let nd = d + l;
                if nd < self.lra_dist[wi] {
                    self.lra_dist[wi] = nd;
                    self.lra_raiz[wi] = self.lra_raiz[vi];
                    fila.push((std::cmp::Reverse(ordenavel(nd)), w));
                }
            }
        }
    }

    pub(super) fn limitar_esticao(&mut self, solver: &Solver) {
        let tecto = solver.estica_max;
        if !tecto.is_finite() || solver.passagens_limite == 0 {
            return;
        }
        let n = self.x.len();
        // ── ⭐⭐⭐ AS ÂNCORAS DE LONGO ALCANCE ────────────────────────────────
        //
        // ⛔⛔ **Sem elas o tecto local NÃO CONVERGE, e o número está medido.** Um
        // limitador de Provet é uma projecção LOCAL: para segurar uma tira
        // pendurada ele tem de propagar a notícia do pino até à ponta, um vértice
        // por passagem. Medido num gesto de gravidade de 120 passos com tecto
        // `1,10`: `4` passagens deixam o pior esticão em `4,63`, `16` em `3,56`,
        // `64` em `1,95` e são precisas **`128`** para chegar a `1,46` — e cada
        // passagem varre a rede inteira.
        //
        // ⭐ A âncora de longo alcance resolve o mesmo em **UMA** passagem, porque
        // não é local: *nenhum ponto do pano pode estar mais longe do que o
        // caminho que o liga ao pino*.
        //
        // ⚠️ **A semeadura corre quando a REDE CRESCE.** Na área *Global* — a do
        // filtro, a única onde o tecto nasce ligado — ela nasce inteira no
        // primeiro passo e o Dijkstra corre **uma vez por gesto**. ⛔ Num traço de
        // área *Local* a rede cresce a cada passo, e ligar o tecto ali passaria a
        // pagar um Dijkstra por passo: é mais uma razão para o traço ficar com a
        // lei do alvo até haver corpus que a meça.
        if self.lra_em != self.restricoes.len() {
            self.semear_lra();
        }
        for i in 0..n {
            if self.lra_dist.is_empty() || !self.activo[i] || self.phi[i] <= 0.0 {
                continue;
            }
            let d0 = self.lra_dist[i];
            if !d0.is_finite() || d0 <= 0.0 {
                continue;
            }
            // ⚠️ A raiz é um vértice de `φ = 0`, logo a posição dela é a de
            // repouso: a integração multiplica por `φ` e não a move.
            let p = self.x[self.lra_raiz[i] as usize];
            let d = [
                self.x[i][0] - p[0],
                self.x[i][1] - p[1],
                self.x[i][2] - p[2],
            ];
            let dist = norm(d);
            let max = d0 * tecto;
            if dist <= max || dist <= 0.0 {
                continue;
            }
            let k = max / dist;
            for c in 0..3 {
                self.x[i][c] = p[c] + d[c] * k;
            }
        }
        if self.dx.len() != n {
            self.dx = vec![[0.0; 3]; n];
            self.dn = vec![0.0; n];
        }
        for _ in 0..solver.passagens_limite {
            self.dx.fill([0.0; 3]);
            self.dn.fill(0.0);
            let mut tocou = false;
            for k in 0..self.restricoes.len() {
                let r = self.restricoes[k];
                let Alvo::Vertice(b) = r.b else { continue };
                let (ai, bi) = (r.a as usize, b as usize);
                let l = r.l + (self.tau[ai] + self.tau[bi]) * 0.5;
                if l <= 0.0 {
                    continue;
                }
                let (pa, pb) = (self.x[ai], self.x[bi]);
                let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
                let dist = norm(d);
                if dist <= 0.0 {
                    continue;
                }
                if dist <= tecto * l {
                    continue;
                }
                // ⚠️ **O peso é `φ` VEZES «este vértice é escrito»** — um vértice
                // inactivo tem a posição relida da malha no passo seguinte, logo
                // uma correcção nele evapora em silêncio e a que ele *devia* ter
                // recebido faltaria ao vizinho. Na área *Global* todos são
                // activos e o factor é `1`.
                let peso = |sf: &Self, i: usize| if sf.activo[i] { sf.phi[i] } else { 0.0 };
                let (wa, wb) = (peso(self, ai), peso(self, bi));
                let soma = wa + wb;
                if soma <= 0.0 {
                    continue;
                }
                tocou = true;
                let corr = (dist - tecto * l) / dist;
                for (c, dc) in d.iter().enumerate() {
                    self.dx[ai][c] += dc * corr * (wa / soma);
                    self.dx[bi][c] -= dc * corr * (wb / soma);
                }
                self.dn[ai] += 1.0;
                self.dn[bi] += 1.0;
            }
            if !tocou {
                break;
            }
            for i in 0..n {
                if self.dn[i] <= 0.0 {
                    continue;
                }
                for c in 0..3 {
                    self.x[i][c] += self.dx[i][c] / self.dn[i];
                }
            }
        }
    }
}

/// `f64` ordenável para a fila de prioridade do Dijkstra.
///
/// ⚠️ **Os bits de um `f64` não ordenável são `NaN` e `-0.0`**, e nenhum dos dois
/// chega aqui: as distâncias são somas de comprimentos de repouso positivos.
/// A transformação é a canónica (inverte o sinal do bit de sinal para os
/// positivos), e é uma bijecção — daí a volta ser exacta.
fn ordenavel(x: f64) -> u64 {
    let b = x.to_bits();
    if b & (1 << 63) == 0 {
        b | (1 << 63)
    } else {
        !b
    }
}

/// A volta de [`ordenavel`].
fn de_ordenavel(b: u64) -> f64 {
    if b & (1 << 63) == 0 {
        f64::from_bits(!b)
    } else {
        f64::from_bits(b & !(1 << 63))
    }
}
