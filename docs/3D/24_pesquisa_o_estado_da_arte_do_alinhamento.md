# 24 — O ESTADO DA ARTE DO ALINHAMENTO DE MALHA, e porque o que construímos é a classe errada

> **Ordem do dono (2026-09-19):** *«vamos modificar completamente esse algoritmo que vc trouxe. faça
> uma pesquisa em busca de um melhor. traga o estado da arte»* — depois de dois smokes reprovados
> ([§80](handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-17.md) *«pouca ou nenhuma diferença»* ·
> [§81](handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-17.md) *«pior que o original, com
> irregularidade a 90 graus da direcção do movimento»*).
>
> ⚠️ **Este documento é PESQUISA, não implementação.** Nenhuma linha de produto muda por causa dele.

---

## §1 — O requisito, escrito com os números que já medimos

Não é uma lista de desejos: é o que as réguas de 18 e 19/09 devolveram sobre o produto que temos.

| grandeza | o que ela mede | por pentear | **hoje, no tecto** | o que se quer |
|---|---|---|---|---|
| **grade** (`0–15°`) | que fracção das arestas corre com o traço | `32,9 %` | `41,1 %` | ≥ `43 %` |
| **vinco** `p90` | o ângulo entre normais vizinhas — *o que a luz lê* | `2,563°` | `4,391°` ⛔ | ≤ `2,6°` |
| **razão** | comprimento ao longo ÷ atravessado | `1,011` | `0,653` ⛔ | `~1,0` |

⇒ **o requisito é as TRÊS colunas ao mesmo tempo**, e é isso que a lei de hoje não sabe fazer: ela
compra a primeira pagando as outras duas. ⭐ **A razão é a coluna que diagnostica**: uma lei de
**quatro dobras** — que é a que declarámos — tem de deixar as duas famílias da grade **iguais**;
`0,653` é um esticão de **duas** dobras que ninguém pediu.

### §1.1 — O mecanismo, em três degraus medidos

A mesma lei, a mesma peça, só a curvatura a mudar:

| peça | razão |
|---|---|
| **CHAPA** (sem curvatura) | `1,029` ✅ |
| bola, carimbo a ZERO | `0,885` |
| bola, com o carimbo a levantar relevo | **`0,653`** ⛔ |

⇒ *o esticão não está nos alvos do campo* — eles são simétricos, medidos (`cos 4α` médio `+0,809` ao
longo contra `+0,814` atravessado) — *ele nasce da DINÂMICA sobre a superfície que o próprio traço
encurva.*

⛔⛔ **E há uma razão estrutural para nada o corrigir:** o par refino/colapso tem uma banda de
histerese de **`2,05×`** ([`collapse_target`](../../crates/ph2d-mesh/src/collapse.rs)), e **toda
anisotropia menor que essa banda é invisível aos dois passes**. As arestas ao longo medem `0,0099` e
as atravessadas `0,0151`: as duas estão *dentro* da banda, logo nem o refino as parte nem o colapso
as funde. **Nada, em lado nenhum da lei, puxa a malha de volta à isotropia.**

---

## §2 — As cinco famílias da literatura, e onde cada uma cai

### A. Conectividade primeiro — *o que nós construímos* ⛔ REFUTADA POR MEDIÇÃO

Alinhar **trocando diagonais** (flip por direcção) e enviesando o alvo de aresta por `cos 4α`.

- **Medido:** grade `41,1 %`, vinco `4,391°` (`1,71×`), razão `0,653`.
- **Porque falha:** a troca de diagonal muda **a que par de triângulos a luz pertence** sem mover um
  vértice, e numa superfície curva isso é exactamente um vinco. E o viés de tamanho tem **tecto
  medido em `~41 %`** ([§80.4](handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-17.md)): pregar a
  normalização pela média multiplica os cortes por `23×` e leva o pior triângulo a `0,64°`.

### B. ⭐⭐⭐ Posição primeiro — *o estado da arte* ✅ RECOMENDADA

**Instant Field-Aligned Meshes** (Jakob, Tarini, Panozzo, Sorkine-Hornung — SIGGRAPH Asia 2015;
**Test of Time Award 2025**). Dois campos, os dois resolvidos por **suavização LOCAL** sem solver
global e com custo **linear**:

1. **campo de orientação** (4-RoSy) — para onde a grade corre;
2. **campo de POSIÇÃO** (PoSy) — *onde ela está*: cada vértice carrega o ponto de uma **retícula**
   gerada pela orientação com espaçamento `ρ`, e a energia minimiza `‖p_i − T(p_j)‖²` sobre
   **translações inteiras** da retícula.

⭐⭐⭐ **É a retícula que resolve o nosso defeito por CONSTRUÇÃO:** ela é **quadrada**, de lado `ρ`
nas duas direcções. Um vértice que caminha para o seu ponto de retícula fica *alinhado* **e** com
*espaçamento igual nas duas famílias* — alinhamento sem esticão, que são as três colunas da §1 de uma
vez. *O que nos falta não é uma cerca melhor no flip: é um termo que diga onde o vértice devia
estar.*

### C. ⭐⭐ Deslize incremental de vértices — *a mesma arquitectura que já temos* ✅ VIÁVEL

**Lai, Kobbelt & Hu**, *An Incremental Approach to Feature Aligned Quad Dominant Remeshing* (SPM
2008) e *Feature aligned quad dominant remeshing using iterative local updates* (CAD 2010).

A lei deles é **exactamente a nossa arquitectura com o alinhamento noutra metade**: um esquema de
relaxação que **desliza o vértice no plano tangente** para alinhar as arestas, **interleaved** com
split/collapse/flip — que ali servem só de higiene da conectividade, não de aligner.

No mapa exponencial do 1-anel, com o referencial rodado para o campo, três forças:

| força | fórmula | o que faz |
|---|---|---|
| colinearidade | `x' = (|y₄|·x₂ + |y₂|·x₄)/(|y₂|+|y₄|)` | endireita a fileira |
| **encaixe** | `x'' = x₂` ou `x₄` (o de menor `|x|`) | **alinha** |
| **relaxação** | `x''' = (x₁+x₃)/2` | **espaçamento uniforme AO LONGO de cada direcção** |

`x ← (1−λ)x + λ(α x' + (1−α−β) x'' + β x''')`, com `λ = ½` e um horário: `α` de `0,3` → `0`, depois
`β` → `0,2`.

⭐⭐⭐ **A terceira força é o termo que nos falta, e é literalmente o anti-esticão:** ela mede o
espaçamento **entre os vizinhos da MESMA direcção** (`x₁`, `x₃`). A nossa relaxação (a H1, o
centroide tangencial isotrópico) **não sabe que existem duas famílias** — ela regulariza a vizinhança
inteira em bloco, e é por isso que a razão pode derivar para `0,65` sem que nada se queixe.

⚠️ E o nosso deslocamento não é inútil: medido, ele **ALISA** (`2,9×` na rugosidade; vinco `2,563 →
2,507`). *Ele está certo como suavizador e vazio como alinhador* — a nossa medição de 18/09 já o
dizia: a lei H1 sozinha entrega `Q +0,0000`.

### D. Métrica anisotrópica (Riemanniana) ⛔ FERRAMENTA ERRADA PARA ESTE PROBLEMA

Alliez et al., *Anisotropic Polygonal Remeshing* (SIGGRAPH 2003); a família `MMG`/adaptação por
métrica; `Lp-CVT` (Lévy & Liu 2010); e a tese *Restricted Delaunay-Based Anisotropic Meshing With
Riemannian Metrics* (2025).

Ela **prescreve** o esticão por uma métrica `M(x)` e mede comprimentos nessa métrica. É a resposta
certa quando se QUER anisotropia (camadas-limite, curvatura extrema) — e o esticão é precisamente o
**nosso defeito**. ⚠️ Ela só entraria aqui na forma degenerada `M = identidade + orientação`, que é
**a retícula da família B escrita com outra notação**, e mais cara.

### E. Campos cruzados neurais ⛔ FORA DO ORÇAMENTO DE UM DAB

NeurCross (2024), CrossGen (2025), QuadLink e TriFlow (2026). Geram campos/topologia «de artista»
por rede, **offline e globalmente**, sobre a peça inteira. O nosso orçamento é **`8 ms` por dab**
sobre uma região do tamanho do pincel — a classe não serve, e não há nenhuma variante local
publicada.

### F. Quadrangulação global ⛔ JÁ É NOSSA, E É OUTRA FERRAMENTA

QuadriFlow (2018, o sucessor declarado do Instant Meshes, com consistência global de singularidades)
e QuadWild/Bi-MDF — este último **já é o oráculo do botão de retopologia** desta casa
([ADR-0162](../architecture/decisions/0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md)).
São passes globais sob comando, medidos em **segundos**. O pente é outra coisa: local e por dab.

---

## §3 — ⭐⭐⭐⭐ O achado: o estado da arte JÁ ESTÁ NA ÁRVORE, e nunca foi usado como pincel

A triagem de licença (§0.9, passo 1) **parou na primeira porta aberta** — e ela estava aberta há um
ano: o **Instant Meshes é BSD-3-Clause**, e esta casa fez dele um **porte fiel** em
[`ph2d-quadflow`](../../crates/ph2d-quadflow/) sob o
[ADR-0160](../architecture/decisions/0160-quad-remesh-is-a-native-cross-field-port-quadriflow-referenced.md),
com a atribuição já registada no censo de citações.

| peça | onde | o que já faz |
|---|---|---|
| campo de orientação 4-RoSy | [`orientation.rs`](../../crates/ph2d-quadflow/src/orientation.rs) | `field_from(dirs)` — **semeável**; `smooth_on(...)` local |
| **campo de POSIÇÃO (a retícula)** | [`position.rs`](../../crates/ph2d-quadflow/src/position.rs) | `position_round_4` (reduz ao mesmo ponto de referência), `smooth_on(...)`, `solve_position` |
| espaçamento `ρ` | [`scale.rs`](../../crates/ph2d-quadflow/src/scale.rs) | o campo de escala |

⭐⭐ **E as duas `smooth_on` já correm sobre SLICES com adjacência explícita** — elas foram escritas
assim para a hierarquia poder correr *«a MESMA lei sobre níveis que não são malhas»*. É exactamente a
assinatura de que um **pincel** precisa: uma região, não a peça.

⇒ **o trabalho não é portar o estado da arte; é LIGÁ-LO ao dab** — semear a orientação com a direcção
do traço em vez de a resolver da curvatura, e mover o vértice para o seu ponto de retícula em vez de
para o centroide isotrópico.

---

## §4 — A proposta, em waves com a medição de cada uma

> ⛔ **Nada disto shipa sem as três colunas da §1 medidas nos quatro rumos**, e o controlo é a
> **CHAPA** — onde a saída do próprio alvo lê razão `1,04`–`1,13` e a nossa lei de hoje lê `1,029`.

| wave | o que faz | o que tem de medir |
|---|---|---|
| **W1** | a orientação do dab **semeada pelo traço** (`field_from` + `smooth_on` na região), sem tocar em posições | o campo é suave e segue o traço; produto **byte-idêntico** (nada consome ainda) |
| **W2** | o campo de POSIÇÃO na região e o vértice a andar para o ponto de retícula, **substituindo** o centroide isotrópico | as TRÊS colunas; e a `razão` tem de ficar em `~1,0` — *é ela que diz se a classe nova cura* |
| **W3** | o flip por direcção **SAI** (ou fica só como higiene de Delaunay) | grade não desce e o vinco desce; se o flip ainda pagar, ele fica com a cerca de 19/09 |
| **W4** | o carril: o vértice não pode sair da superfície nem brigar com o `Draw` | a fidelidade à escultura, e o relógio do dab contra os `8 ms` |

⚠️ **O risco nomeado da W2** é o mesmo que a literatura reporta para a família B: uma retícula por
vértice pode **discordar** da vizinha por uma célula inteira (é para isso que existe a redução por
translações inteiras da `position_round_4`), e uma redução malfeita produz um salto de célula — que
na nossa peça apareceria como um degrau, não como ondulação. A régua para ele já existe: a
[`vinco_da_faixa`](../../crates/ph2d-sculpt3d/src/medida_do_pente.rs).

### §4.1 — O que NÃO fazer, com o motivo

- ⛔ **Não portar QuadriFlow/QuadWild para o pincel:** são passes globais sob comando, medidos em
  segundos, e já são o botão de retopologia.
- ⛔ **Não construir métrica anisotrópica:** ela prescreve o esticão, que é o defeito.
- ⛔ **Não tocar na cadeia de retopologia:** o `ph2d-quadflow` é consumido pelo botão, e a wave é
  sobre **consumir a mesma biblioteca de outro sítio** — não sobre mudá-la. *A quinta recusa do
  colapso já mostrou o preço de mexer num motor partilhado: onze gates vermelhos.*

---

## §5 — Triagem de licença (§0.9, passo 1)

| artefacto | licença | uso |
|---|---|---|
| **Instant Meshes** (`wjakob/instant-meshes`) | **BSD-3-Clause** | ✅ **portado** em `ph2d-quadflow`, com atribuição no cabeçalho de cada ficheiro |
| Lai–Kobbelt SPM 2008 / CAD 2010 | **papers** | ✅ clean-room de papers é o método desta casa ([ADR-0167](../architecture/decisions/0167-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)) |
| Alliez 2003 · Lp-CVT · MMG | papers / LGPL | ⛔ não é preciso — família recusada na §2.D |
| QuadriFlow · QuadWild/Bi-MDF | ⚠️ por triar | ⛔ não é preciso — família F, já coberta pelo botão |

⛔ **Nenhum fonte de alvo restrito foi lido para esta pesquisa.** As fontes são papers públicos, a
nossa própria medição e o nosso próprio porte BSD.

---

## §6 — O que fica para o DONO decidir

1. **Seguir a família B (recomendada)** — ligar a retícula ao dab, com as quatro waves da §4.
2. **Seguir a família C** — as três forças de Lai–Kobbelt dentro da nossa relaxação. Mais pequena,
   e entrega o anti-esticão sem trazer um segundo campo; ⚠️ mas ela alinha às direcções
   **principais** no paper, e nós queremos a **direcção do traço** — a troca é directa, e não está
   medida.
3. **Parar aqui** — o pente fica como está, com a troca declarada e medida, e o `Edge Flow` é um
   botão que o artista usa por sua conta.

⚠️ **As famílias B e C não são exclusivas:** a retícula É o encaixe e a colinearidade de Lai escritos
num campo, e a terceira força dele é a mesma isotropia que a retícula tem por construção. *A pergunta
não é qual, é se começamos pelo campo (B, mais estrutura, mais poder) ou pelas forças (C, menos
código, mesma correcção).*

---

## §7 — Fontes

- [Instant Field-Aligned Meshes — página do projecto, IGL/ETH Zürich](https://igl.ethz.ch/projects/instant-meshes/)
- [Instant field-aligned meshes — ACM TOG (SIGGRAPH Asia 2015)](https://dl.acm.org/doi/10.1145/2816795.2818078)
- [Instant field-aligned meshes — Semantic Scholar (Test of Time Award 2025)](https://www.semanticscholar.org/paper/Instant-field-aligned-meshes-Jakob-Tarini/0754a185544ea26f18befa294af3a44ec7829082)
- [An Incremental Approach to Feature Aligned Quad Dominant Remeshing — Lai, Kobbelt, Hu (SPM 2008)](https://www.graphics.rwth-aachen.de/media/papers/spm08_011.pdf)
- [Feature aligned quad dominant remeshing using iterative local updates — CAD 2010](https://www.sciencedirect.com/science/article/abs/pii/S001044850900089X)
- [Anisotropic Polygonal Remeshing — Alliez et al. (SIGGRAPH 2003)](https://geometry.caltech.edu/pubs/ACDLD03.pdf)
- [Isotropic Remeshing with Inter-Angle Optimization (2025)](https://arxiv.org/html/2507.13641v1)
- [Restricted Delaunay-Based Anisotropic Meshing With Riemannian Metrics (2025)](https://digitalcommons.wayne.edu/oa_dissertations/4295/)
- [NeurCross — neural cross fields (2024)](https://arxiv.org/pdf/2405.13745)
- [CrossGen — learning and generating cross fields (2025)](https://arxiv.org/pdf/2506.07020)
- [TriFlow — artist-like mesh topology (2026)](https://arxiv.org/pdf/2606.20131)
- [Interactively controlled quad remeshing of high resolution 3D models — ACM TOG 2016](https://dl.acm.org/doi/10.1145/2980179.2982413)
- [instant-meshes — licença BSD](https://github.com/wjakob/instant-meshes/blob/master/LICENSE.txt)
