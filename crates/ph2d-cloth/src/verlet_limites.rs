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
use crate::V3;

/// ⭐⭐ **A SOBRE-RELAXAÇÃO do limitador** (SOR) — e ela é de GRAÇA.
///
/// A média de Jacobi é uma combinação convexa, logo cada passagem só remove
/// `1/k` da violação de um vértice tocado por `k` restrições — e nesta rede `k`
/// chega a `21`. Multiplicar a média por `ω ∈ (1, 2)` é a sobre-relaxação
/// clássica: o passo continua a ser de descida e a convergência acelera.
///
/// ⚠️ **O número é MEDIDO, e o recurso dele é a estabilidade, não o tempo** — o
/// relógio não se move (as quatro passagens são as mesmas):
///
/// | `ω` | esticão máx (`4 514` vért.) | esticão máx (`24 386`) | ms/passo |
/// |---:|---:|---:|---:|
/// | `1,0` | `1,563` | `6,717` | `19,14` |
/// | `1,5` | `1,441` | `5,043` | `19,10` |
/// | **`1,9`** | **`1,427`** | **`3,467`** | `19,69` |
///
/// ⛔ **`2` é o tecto teórico** da família (acima dele a iteração deixa de
/// contrair), e `1,9` é o que a medição escolheu de dentro dela.
///
/// ⚠️⚠️ **E a tabela nomeia o que NÃO se cura com isto:** a `24 386` vértices o
/// pior esticão continua em `3,47` com o tecto em `1,10`, porque a convergência
/// de um limitador LOCAL é `O(diâmetro da malha em arestas)` — quem segura o
/// global é a âncora de longo alcance, e ela não fala do vizinho.
const SOBRE_RELAXACAO: f64 = 1.9;

impl Verlet {
    /// ⭐⭐⭐ **LIGA A CONSERVAÇÃO DE VOLUME** — o chamador entrega os triângulos da
    /// peça e a lei mede o volume de repouso deles.
    ///
    /// ⚠️ **A ORIENTAÇÃO das faces é o sinal do volume**, e a lei não a corrige:
    /// uma peça com faces viradas para dentro dá `V₀ < 0`, e a restrição
    /// funciona na mesma (ela compara `V` com `k·V₀`, com o mesmo sinal dos dois
    /// lados). ⛔ O que ela NÃO tolera é `V₀ = 0`, que é uma superfície aberta —
    /// aí a razão não existe e a restrição desliga-se sozinha.
    pub fn conhecer_as_caras(&mut self, caras: Vec<[u32; 3]>) {
        // ⚠️⚠️ **O volume de repouso é o do MATERIAL, não o da pose de agora**
        // (report do dono de 2026-09-09). Com a base persistente, o segundo gesto
        // conserva o volume ORIGINAL da peça; sem isto ele conservaria o que o
        // primeiro gesto deixou, e a perda de cada gesto compunha-se em silêncio.
        // ⭐ Sem base, `base_de` **é** o repouso ao bit, logo o primeiro gesto não
        // se mexe.
        let base: Vec<V3> = (0..self.len()).map(|v| self.base_de(v)).collect();
        self.volume0 = volume_de(&base, &caras);
        // ⚠️ As dobradiças e os ângulos de repouso saem do MESMO material, e é o
        // que faz o repouso ser um ponto fixo da lei de dobra.
        let topo = crate::ClothTopology::build(&caras, self.len());
        self.dobradicas = topo.dobradicas().to_vec();
        self.dobra_repouso = self
            .dobradicas
            .iter()
            .map(|h| crate::bending::dihedral(&base, *h))
            .collect();
        self.caras = caras;
    }

    /// ⭐⭐⭐ **A RESISTÊNCIA À DOBRA** — a restrição de ângulo diedro da família
    /// PBD (Müller · Heidelberger · Hennix · Ratcliff, 2007, §4.3), sobre o
    /// ângulo de repouso do MATERIAL.
    ///
    /// `C = θ(x) − θ̄`, e a correcção é a projecção de mínimos quadrados
    /// `Δxᵢ = −k·C·wᵢ∇ᵢθ / Σ wⱼ|∇ⱼθ|²`.
    ///
    /// ⭐⭐ **A matemática JÁ EXISTIA na crate e estava sem consumidor**: o
    /// [`crate::bending`] foi escrito para o caminho VBD que a auditoria de 05/09
    /// refutou, e o que sobreviveu dele — o ângulo com SINAL por `atan2`, as
    /// quatro derivadas cuja soma é zero **por construção**, e a diferença
    /// dobrada para `(−π, π]` — é exactamente o que uma restrição posicional
    /// precisa. *A lei mudou de família e a geometria não.*
    ///
    /// ⚠️⚠️ **O ângulo de repouso é o do MATERIAL e não zero**, e é a mesma razão
    /// que o `bending.rs` já escreve: o repouso de uma escultura é a superfície
    /// esculpida, que é curva em todo sítio interessante — um modelo com repouso
    /// plano daria força no repouso e a peça mexia-se sozinha ao encostar nela.
    ///
    /// ⛔⛔ **E ela é JACOBI COM MÉDIA, como o limitador de esticão — a 1.ª
    /// redacção era Gauss-Seidel e DESTRUÍA a peça no topo da faixa.** Cada
    /// vértice pertence a ~`12` dobradiças (~`6` como ponta da aresta, ~`6` como
    /// ápice), e uma projecção inteira por dobradiça resolvida em sequência soma
    /// doze correcções sobre o mesmo vértice. Medido com rigidez `1,0`: a área ia
    /// a **`4,04×`** o repouso, o volume caía a `0,27` e a peça encolhia de
    /// `2,64` para `1,41` de altura. *É a mesma divergência que o tecto de
    /// esticão já tinha pago, um mês de leis mais tarde.*
    ///
    /// ⚠️ **O peso do *Discrete Shells* (`3‖ē‖²/(A₀+A₁)`) NÃO entra aqui.** Ele é
    /// o que converte um ângulo em ENERGIA; uma restrição posicional projecta o
    /// ângulo directamente, e multiplicar a projecção por um peso de área
    /// tornaria a rigidez função da densidade da malha — que é precisamente o
    /// defeito que o artista relata do outro lado.
    pub(super) fn resistir_a_dobra(&mut self, k: f64) {
        if k <= 0.0 || self.dobradicas.is_empty() {
            return;
        }
        let k = k.clamp(0.0, 1.0);
        let voltas: u32 = std::env::var("PH2D_DOBRA_N")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        for _ in 0..voltas {
            let n = self.x.len();
            if self.dx.len() != n {
                self.dx = vec![[0.0; 3]; n];
                self.dn = vec![0.0; n];
            }
            self.dx.fill([0.0; 3]);
            self.dn.fill(0.0);
            for i in 0..self.dobradicas.len() {
                let h = self.dobradicas[i];
                let v = h.verts();
                let c = crate::bending::dihedral(&self.x, h) - self.dobra_repouso[i];
                // ⚠️ **A diferença é DOBRADA para `(−π, π]`** — sem isso uma
                // dobradiça que cruza `±π` (o pano a fechar sobre si) lê um salto de
                // `2π` e a correcção explode numa agulha.
                let c = if c > core::f64::consts::PI {
                    c - 2.0 * core::f64::consts::PI
                } else if c < -core::f64::consts::PI {
                    c + 2.0 * core::f64::consts::PI
                } else {
                    c
                };
                if c == 0.0 {
                    continue;
                }
                let g = crate::bending::grads(&self.x, h);
                let mut denom = 0.0;
                let mut w = [0.0f64; 4];
                for (slot, gi) in g.iter().enumerate() {
                    let vi = v[slot] as usize;
                    w[slot] = if self.activo[vi] { self.phi[vi] } else { 0.0 };
                    denom += w[slot] * (gi[0] * gi[0] + gi[1] * gi[1] + gi[2] * gi[2]);
                }
                if denom <= 1e-18 {
                    continue;
                }
                let lambda = k * c / denom;
                for (slot, gi) in g.iter().enumerate() {
                    if w[slot] <= 0.0 {
                        continue;
                    }
                    let vi = v[slot] as usize;
                    for (ch, gc) in gi.iter().enumerate() {
                        self.dx[vi][ch] -= lambda * w[slot] * gc;
                    }
                    self.dn[vi] += 1.0;
                }
            }
            for i in 0..n {
                if self.dn[i] <= 0.0 {
                    continue;
                }
                for ch in 0..3 {
                    self.x[i][ch] += SOBRE_RELAXACAO * self.dx[i][ch] / self.dn[i];
                }
            }
        }
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

    /// ⭐⭐⭐ **O TECTO PASSA A VER O CRESCIMENTO DO REPOUSO** — e sem constante
    /// nova nenhuma.
    ///
    /// # O defeito, medido
    ///
    /// O tecto de esticão compara `|aresta|` contra `tecto × ℓ`, e o `ℓ` dele é o
    /// repouso **corrente**, `ℓ_material + τ`. De cinco tipos de filtro, o
    /// *Expand* é o único que mexe no `τ` ⇒ *o denominador da régua cresce com a
    /// lei que ela devia limitar*, e o tecto lê «não está esticado» enquanto a
    /// peça incha. Medido na esfera de fábrica, um arrasto de curso inteiro
    /// (`sonda_do_expand_que_nao_para`, em `ph2d-sculpt3d/tests/`):
    ///
    /// | tipo | 1 gesto | 3 gestos |
    /// |---|---:|---:|
    /// | Gravity | `1,000` | `1,000` |
    /// | Inflate | `1,126` | `1,327` |
    /// | **Expand** | **`7,743`** | **`14,358`** |
    /// | Pinch | `1,191` | `1,512` |
    /// | Scale | `1,128` | `1,435` |
    ///
    /// ⭐⭐ **E a prova de que o tecto não o alcança é a linha do tecto MÍNIMO:**
    /// com `tecto = 1,00` ele proíbe *todo* esticão elástico, e o esticão contra
    /// o material continua em **`6,854`** — logo aquilo é `τ` INTEIRO.
    ///
    /// ⚠️ **O que ele produz não é uma peça maior, é uma peça AMARROTADA:** o
    /// volume dá `1,067 → 0,245 → 0,761` em um, dois e três gestos. *Não é o
    /// volume que explode — é a folga que faz o pano dobrar sobre si.*
    ///
    /// # A lei, e por que ela não escolhe um número
    ///
    /// O `stretch_max` já promete *«nenhuma aresta passa de `tecto ×` o repouso
    /// dela»*. Esta porta faz a promessa valer também para a metade PLÁSTICA:
    /// `τ` de cada vértice é limitado a `(tecto − 1) ×` a **menor** aresta de
    /// material que lhe toca.
    ///
    /// ⭐ **O `min` não é conservadorismo, é o que torna a garantia demonstrável:**
    /// para a aresta `(a, b)` o repouso corrente é `ℓ + (τₐ + τ_b)/2`, e como
    /// `min_a ≤ ℓ` e `min_b ≤ ℓ`, o limite dá `ℓ + (tecto − 1)·ℓ = tecto·ℓ`.
    /// Com a média em vez do mínimo a desigualdade não fecha.
    ///
    /// ⛔ **Só o CRESCIMENTO é limitado, e a assimetria é deliberada.** Um `τ`
    /// negativo encolhe o repouso, e ninguém reportou defeito nesse lado — a
    /// fixture `plano_filtro_expandir_negativo` bate com o oráculo a `0,022`.
    /// *Limitar o que não foi medido é escolher um número*, e o §0.0 da casa
    /// proíbe-o. Quem medir o lado que encolhe põe aqui a outra metade.
    ///
    /// ⛔⛔ **A lei do alvo NÃO é tocada.** O único sítio que escreve `τ` é a lei
    /// da referência (`Modo::Expandir`), e mexer lá partiria as fixtures do
    /// oráculo. Esta porta vive no ficheiro dos limites e é chamada de dentro do
    /// [`Self::limitar_esticao`], que **já sai por `return`** quando
    /// `estica_max` é `∞` — que é a omissão do [`Solver`] e o caminho por onde as
    /// `103` fixtures correm. ⇒ *a paridade é byte-idêntica por construção, não
    /// por promessa.*
    fn limitar_o_repouso(&mut self, tecto: f64, n: usize) {
        if self.restricoes.is_empty() {
            return;
        }
        // A menor aresta de MATERIAL que toca cada vértice. ⚠️ Reusa o buffer de
        // trabalho do limitador em vez de alocar por passo.
        if self.dn.len() != n {
            self.dx = vec![[0.0; 3]; n];
            self.dn = vec![0.0; n];
        }
        self.dn.fill(f64::INFINITY);
        for r in &self.restricoes {
            let Alvo::Vertice(b) = r.b else { continue };
            let (ai, bi) = (r.a as usize, b as usize);
            self.dn[ai] = self.dn[ai].min(r.l);
            self.dn[bi] = self.dn[bi].min(r.l);
        }
        let folga = (tecto - 1.0).max(0.0);
        for i in 0..n {
            if self.dn[i].is_finite() {
                self.tau[i] = self.tau[i].min(folga * self.dn[i]);
            }
        }
    }

    pub(super) fn limitar_esticao(&mut self, solver: &Solver) {
        let tecto = solver.estica_max;
        if !tecto.is_finite() || solver.passagens_limite == 0 {
            return;
        }
        let n = self.x.len();
        self.limitar_o_repouso(tecto, n);
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
                    self.x[i][c] += SOBRE_RELAXACAO * self.dx[i][c] / self.dn[i];
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
