# ADR-0139 — `motion.voronoi` na GPU: Lloyd via Jump Flooding, centroides em INTEIROS, e o cap de 600 cai

- **Status:** aceito (implementado nesta linha, `line/gpu-nodes`)
- **Data:** 2026-07-21
- **Contexto:** item 5 da fila §2 do handoff de continuação ("Voronoi (JFA) —
  algoritmo GPU PRÓPRIO, não reusa a grade de vizinhança"); ordem do Enio
  ("siga nesta linha") após a fila §E fechar.

## Por que o voronoi primeiro (medido)

Dos O(N²) restantes, o `motion.voronoi` é o único cujo **algoritmo** CPU tem a
forma errada — não só a constante. O `nearest` é varredura linear por amostra
⇒ custo `O(iterations · res² · count)`; MEDIDO no cap: 2,4 ms/frame a 600
pontos (res 96, 8 iterações) — barato **porque o cap o mantém barato**.
Descapado ao que stippling/blue-noise de verdade quer (10k+ pontos, res pela
própria lei ≈ 400+), a aritmética dá **~600 ms/frame**; o cap de **600 pontos
é o menor da biblioteca inteira** (4+ ordens abaixo do alvo M5) e é exatamente
o caso do §0.0: o caminho lento definindo o produto. Soft-body (1600) e a
corda são O(N) — caps de conforto de outra classe, e o XPBD deles fica
nomeado para depois.

## Decisões

### 1. Jump Flooding (JFA) substitui o nearest linear — count-independente

Por iteração de Lloyd: (a) semear um grid `res²` com o id do ponto na célula
do ponto; (b) `log₂(res)+1` passes de jump flood (offsets res/2 … 1; cada
texel adota, entre os 9 vizinhos no offset corrente, o dono mais próximo);
(c) reduzir centroides; (d) mover cada ponto ao seu centroide. Custo
`O(res² · log res · iterations)`, **independente da contagem** — o JFA é o
algoritmo canônico de Voronoi discreto em GPU (Rong & Tan 2006). Tudo num
encoder, zero readback.

### 2. Os centroides acumulam em INTEIROS (atomics u32 sobre índices de texel)

WGSL não tem atomics f32 — e não precisa: a posição de uma amostra é função
afim do seu índice de texel (`sample_pos`), então acumular `Σgx, Σgy, n` em
u32 é **exato e independente de ordem** (adição inteira comuta), e o
centroide sai da mesma fórmula afim aplicada à média. O dispositivo fica
DETERMINISTA por construção — mais estável que a própria soma f32 sequencial
da CPU. Overflow: `Σgx ≤ res³`; res 1024 ⇒ 2³⁰ < 2³² ✓ (guard no service).

### 3. Canal novo de side-metadata: `GpuAlgorithm` (o 5º, padrão GridSpec/StateSelect/StreamOp)

Um nó-fonte cujo cook é um ALGORITMO multi-pass não é um kernel por-elemento
nem uma operação estrutural de stream: `KernelResolver::algorithm(ty)`
(default `None`) devolve `GpuAlgorithm::LloydVoronoi { …param names… }`; a
MAQUINARIA vive no sequencer (`gpu-cook/src/voronoi.rs`, irmão de `grid.rs`).
O nó registra `GpuKernel::PASSTHROUGH` + a lei de contagem via o canal (o
plano reclama; o cook intercepta como Concat/Project). O `relax` (porta VALUE)
é lido na row 0 DO DISPOSITIVO no passe final de lerp (`P = raw +
(relaxed−raw)·clamp(relax,0,1)`) — um LFO animando o relax deixa de re-rodar
a relaxação: ela é re-cozida só quando params mudam… **não**: o cook é puro
por frame; a relaxação re-roda por cook, mas a JFA a torna ~ms. (Cache por
params é follow-up honesto, não pré-requisito.)

### 4. Paridade: a CPU segue canônica; a doutrina é a do ADR-0127 D4 (MEDIDO)

Lloyd iterado é um sistema SEQUENCIAL (`x_{k+1} = f(x_k)`), então a paridade
segue o D4: **um passo se gateia apertado; a trajetória ganha banda medida**.
Resultados na RTX (gates em `gpu-cook/tests/gpu_voronoi.rs`):

- **Oráculo de assignment** (grid JFA vs `nearest` exato, texel a texel,
  fixture livre de colisão): **0 texels divergentes** em counts 40 e 96 — o
  1+JFA com aritmética idêntica de centro de texel saiu EXATO nestes tamanhos
  (a taxa ~10⁻⁴ da literatura é o teto, não o piso). O gate ainda exige que
  qualquer divergente futuro seja near-tie (razão de distâncias ≤ 1,05).
- **Um passo de Lloyd** (seed sem colisão): Δ máx **1e-6** — o resto é a soma
  f32 sequencial da CPU vs a média inteira exata do dispositivo.
- **`iterations = 0` é BIT-EXATO** (600 pontos): o hash é o avalanche inteiro
  portado instrução a instrução; não há ε atrás do qual se esconder.
- **Trajetória cheia** (8 iterações, com colisões): mean 0,009–0,023 ·
  p95 0,047–0,086 · max 0,25–0,34 (domínio 5×5) — pinado como banda
  (0,04/0,15/0,55). O mecanismo: **colisão de seed** — dois pontos no mesmo
  texel deixam um invisível ao grid por uma rodada (ele segura parado, a regra
  de célula vazia da CPU), o vencedor se afasta e o par se separa; transiente
  e auto-curável, mas compõe pela sequência. Documentado no módulo; é a
  razão de a banda ser banda.
- **Empate exato prefere o id MENOR** (o keep-first do `nearest` da CPU), no
  seeding e nos passes: a chave é `count − id` sob `atomicMax` (0 = vazio ⇒
  `clear_buffer` esvazia o grid). Gate com fixture simétrica bit-igual.

6/6 mutações sangram (lerp invertido · centroide sem +0,5 · tie-break
invertido · JFA truncada no passo 2 · chave de seed descasada · nó sem
`register_gpu_algorithm`), cada uma no gate que existe para ela.

### 5. Os caps CAÍRAM para números medidos (tabela na RTX, 8 iterações/frame)

| count | res (lei) | ms/frame (device) |
|---|---|---|
| 600 (cap velho) | 96 | 1,05 |
| 600 | 98 | 1,14 |
| 2 000 | 179 | 1,27 |
| 10 000 | 400 | 1,94 |
| 50 000 | 895 | 6,38 |
| **165 000 (cap novo)** | **1625** | **20,2** |
| 200 000 | 1625 (saturado) | 21,0 |
| 1 000 000 | 1625 (saturado) | 43,6 |

`MAX_RES 96 → 1625` — o recurso é a **representação**: o centroide inteiro
acumula `Σgx ≤ res³` em u32 e `1625³ < 2³² < 1626³` (o engine guarda a própria
cópia, `INT_CENTROID_RES_CEILING`, e um gate pina as duas juntas).
`MAX_POINTS 600 → 165 000` — **o maior count em que a lei de amostragem
(16 samples/ponto) se sustenta sob esse teto** (`165 000·16 ≤ 1625²`, gate);
acima disso a relaxação degradaria em silêncio (< 16 samples/ponto), então o
cap diz onde a QUALIDADE prometida acaba, não onde a implementação cansa.
A faixa **577–600** re-relaxa com res 98 em vez de 96 (deriva mínima de
layout, medida na banda do gate: a 600 a banda até ENCOLHEU — mean 0,0094).
A CPU continua computando a MESMA resposta; no topo do range ela é minutos
por cook (extrapolado dos 2,4 ms medidos a 600) — o §0.0 ao pé da letra: a
referência computa a mesma resposta, o teto é do dispositivo. VRAM no teto:
ping-pong do grid 2·1625²·4B ≈ 21 MB + acumuladores 3·count·4B ≈ 2 MB.
