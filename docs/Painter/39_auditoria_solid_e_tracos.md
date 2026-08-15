# 39 — Auditoria do `Style: Solid` e dos tipos de linha

> Ordem do Enio, 2026-08-15, depois do smoke aprovado da W8:
> *"Auditoria completa dos traços e solid. Atenção especial para performance em Symmetry Circular + Tiling."*

Tudo aqui foi **medido pela porta do produto** (`on_canvas_pointer`), nunca por um laço próprio da
sonda. A sonda vive em `ph2d-tool-painter::tool::paint::measure_solid_cost`.

---

## 1. Dois defeitos de correção, os dois na configuração que o Enio apontou

### 1.1 A teia do Sketchy / Wire era **apagada** — sobravam 11,9%

**Mecanismo.** Num evento de ponteiro o produto fazia, nesta ordem: `stamp_dabs` — que abria a
transação do preenchimento, **salvava** o retângulo e escrevia a mancha — e só **depois**
`park_stroke` → `stamp_threads`. Os fios são tinta **cumulativa** e caíam **fora** do instantâneo; o
`peel_drag_preview` do evento seguinte restaura exactamente aquele retângulo, e levava-os junto.

Sob simetria circular o retângulo é a **tela inteira**, então a teia inteira do evento anterior
morria a cada movimento do rato.

**Medido:** 261 de 2186 texels sobreviviam (**11,9%**) → **100,0%** depois da cura.

**A cura.** O instantâneo tem de conter toda a tinta cumulativa do evento e nenhuma transitória,
logo **a mancha é a última escrita do evento**. O `peel` continua a preceder os dabs (senão a mancha
velha entra no instantâneo e vira permanente) e o `save` passa a suceder os fios. As duas metades são
a mesma transação, ligadas por `PaintState::solid_fill_owed` — armado na porta do carimbo, consumido
pelo `park_stroke`, que é a última coisa que **todo** sítio do ciclo de traço faz e cujo modo de
falha, se alguém a esquecer, já é **alto** (o pincel para de pintar).

⚠️ **A fixture do gate põe a `Strength` em ZERO, e é ela que torna a pergunta respondível.** Um fio
que cai dentro da região cheia é invisível por construção (mesma cor sobre mesma cor), então um
oráculo que conte texels sobre a mancha **não distingue apagado de invisível** — a primeira versão
mediu `0 de 117` e não podia dizer qual dos dois. A tinta do fio sai por um canal próprio
(`thread_ink` lê `thread_width_px`/`thread_opacity`, nunca a `strength`).

Gate: `the_web_survives_the_fill`.

### 1.2 A corda de fechamento escapava do retângulo salvo sob Tiling

**Mecanismo.** A transação salvava a caixa dos laços mais **meia-espessura** por causa da corda — e o
Tiling tem **régua própria**. Um laço é replicado quando a **caixa** dele passa a costura; um dab,
quando `centro ± raio` passa. Um caminho colado à borda tem a caixa **dentro** da tela e dabs de
corda cuja pegada passa dela: a cópia envolvida cai na borda **oposta**, a um span inteiro do
retângulo salvo. Nenhum restore a alcança ⇒ cada evento deixa um fantasma, e o desenho passa a
depender da **taxa de eventos** — a lei que este módulo já pagou quatro vezes no relevo.

**Medido:** 197 fantasmas, todos na faixa envolvida → **0**.

**A cura** é não ter uma segunda régua: `tiled_chord_region` pergunta às **mesmas** portas que o
carimbo vai usar (`tiling::tiled_dabs` → `dab_batch_region`).

⚠️ **O primeiro oráculo era contaminado.** Por *"o desenho muda com o número de eventos?"* o próprio
caminho é amostrado diferente — 718 texels de piso já com o Tiling **desligado**. O oráculo exato:
descascar o preview no fim do gesto devolve a tinta cumulativa, que é exactamente o que o mesmo gesto
**sem Solid** pinta; qualquer diferença é fantasma, e o controle com Tiling desligado tem de dar zero.

Gate: `the_fill_writes_nothing_outside_the_rect_it_saved`.

---

## 2. Performance — o retrato, e o que ele diz

**A transação do preenchimento É o evento.** A 1024² com simetria circular de 12 + Tiling, no evento
96 de um traço, ela custava **4,232 ms** de um evento de **4,266** — o depósito dos dabs é ~0,05 ms.

### 2.1 O que multiplica o custo

| config (1024², 96 eventos) | laços | pontos | rect px | ms p50 |
|---|---:|---:|---:|---:|
| `sym=off  tiling=off` | 1 | 1 210 | 56 434 | 0,300 |
| `sym=mirror tiling=off` | 2 | 2 420 | 284 672 | 0,629 |
| `sym=circ12 tiling=off` | 12 | 14 520 | **1 048 576** | 3,276 |
| `sym=off  tiling=on` | 2 | 2 420 | 284 672 | 0,660 |
| `sym=mirror tiling=on` | 4 | 4 840 | 284 672 | 0,841 |
| `sym=circ12 tiling=on` | **24** | **29 040** | **1 048 576** | 3,838 |

**A simetria circular põe o retângulo na TELA INTEIRA já no primeiro evento** (1 048 576 = 1024²), e é
daí que vem tudo o resto: a transação salva, preenche, escreve e restaura o canvas inteiro **por
movimento do rato**. O Tiling sozinho é barato (0,30 → 0,66 ms); combinado, ele dobra os pontos e o
retângulo já era a tela ⇒ +17%.

⚠️ **Duas fixtures nasceram sem o fenómeno e teriam feito a tabela mentir:** um arco centrado no
**eixo** da simetria é invariante sob rotação (as doze cópias caem umas sobre as outras e a rosácea
não abre), e um laço **interior** faz o `tiled_loops` devolver a entrada verbatim (a coluna do Tiling
mediria *"é de graça"*).

### 2.2 De que ela é feita, e o que já foi curado

| peça (1024², circ12 + tiling) | antes | depois |
|---|---:|---:|
| construir os laços | 0,029 | 0,029 |
| `solid::fill_coverage` | 1,472 | 1,414 |
| `write_solid` (o `over`) | **1,647** | **0,260** |
| `save_region` | 0,058 | 0,068 |
| `restore_region` | 0,072 | 0,079 |
| **transação** | **4,232** | **2,926** (1,45×) |

**O `over` passou a correr por LINHA.** As linhas são disjuntas, a leitura de `cov` é imutável e não
há RNG nem transcendental ⇒ ADR-0109, **byte-idêntico por construção**. Um corpo, dois walkers: o
kernel é `blend_solid_row` e o `par` escolhe só quem o percorre. Piso do pool = o `PARALLEL_MIN_AREA`
do kernel de dab, e não um número novo.

⚠️ **A rota paralela shipava sem gate**, e isso fica registado: toda fixture de Solid roda a 256²
(65 k texels) contra um piso de ~131 k, então os 15 gates existentes exercitavam só a rota serial.
Gate novo: `both_walkers_of_the_solid_over_write_the_same_bytes`.

**E um lote vazio deixou de abrir a transação:** o tique do motor corre a cada quadro, e um pincel
**parado** pagava um preenchimento de canvas por quadro para não mudar um byte.

### 2.3 Escala com a tela

| canvas | rect px | ms p50 (antes) | ms p50 (depois) |
|---|---:|---:|---:|
| 512² | 262 144 | 0,963 | 0,980 |
| 1024² | 1 048 576 | 2,748 | 2,381 |
| 2048² | 4 194 304 | 9,387 | 7,521 |

**Limitado pela ÁREA, como um re-carimbo tem de ser** — o que a wave mudou foi a constante, não a
forma. A 4096² a extrapolação dá ~30 ms/move sob simetria circular; **não medido a 4096²**, e um
número extrapolado não é um número medido.

### 2.4 ⚠️ ABERTO, com o preço ao lado — decisão do Enio

O outro item de área é o **`solid::fill_coverage`**: **1,414 ms de 2,926 = 48%** do que sobrou. Ele
mora na **`ph2d-painter-brush`, que não tem `rayon`** — pô-lo lá é **dep nova**, logo ADR e ordem.

Decomposto (mesmo retângulo, 1024², circ12 + tiling):

- **área** (a alocação do acumulador de ~4,2 MB, o zero dele e a soma corrida): **0,808 ms** (57%)
- **arestas** (os 29 304 pontos): **0,606 ms** (43%)

Ou seja: **paralelizar as linhas** ataca os 0,808 e **decimar o caminho de tinta** ataca os 0,606.
A segunda é medida — um ponto em cada oito leva o `fill_coverage` de 1,414 a 1,056 — e **não** exige
dep nova, mas muda a **geometria da mancha**, logo é decisão de LOOK com smoke próprio.

Não construído, com motivo, nos dois casos.

---

## 3. Censo dos tipos de linha sob Solid

Gesto em C, canvas 256, r=4, `Rough` armado a 0,4. O oráculo é a **diferença contra o mesmo gesto em
`None`** — metade dos tipos move a tinta e metade decora o traço, então *"quantos texels ele pinta"*
não os compara.

| tipo | vs `None` SEM Solid | vs `None` COM Solid | tinta SEM | tinta COM |
|---|---:|---:|---:|---:|
| Speed | 0 | 0 | 3 263 | 11 768 |
| Sketchy | 1 075 | 799 | 3 767 | 12 250 |
| Wire | 1 341 | **0** | 3 989 | 11 768 |
| Ribbon | 3 194 | 12 227 | **44** | **44** |
| Rough | 3 119 | 1 778 | 4 309 | 12 202 |
| None | 0 | 0 | 3 263 | 11 768 |

**Nenhum tipo é apagado pelo Solid** — a coluna `tinta COM` é ≥ a do `None` em todos. Os três zeros
têm três naturezas, e nenhuma é defeito da mancha:

- **Wire dá 0 sob Solid.** Os laços do arame cortam a **concavidade** do C, que é exactamente a
  região que o preenchimento enche: tinta da mesma cor dentro de uma região cheia dessa cor é
  **invisível por construção**. O Sketchy dá 799 porque a teia dele alcança fora.
- **Speed dá 0 nas duas colunas** — fixture, não produto: o arremesso é `v · T` e um arco de passos
  curtos com o estabilizador ligado quase não tem `v`. A régua do tipo é o `line_speed_probe`.
- **Ribbon pinta 44 texels com E sem Solid**, então **o Solid não é a variável**. A fita é uma mola e
  passos de ~1,65 px nunca a aceleram; 160 eventos sobre o **mesmo** arco dão os mesmos 44, e o probe
  próprio dela (reta rápida a 2048²) mede faixas de 42 a 356 px. **Pergunta da fita, com dono e sonda
  próprios** — nomeada, não perseguida aqui.

### 3.1 ⚠️ Um default que contradiz o argumento do vizinho — decisão do Enio

`rough_amount` e `rough_bowing` nascem em **0,0**: escolher `Rough` no dropdown **não muda um pixel**
até o artista mexer no slider. O `spec_default.rs` argumenta o **contrário** quatro linhas acima, para
o Ribbon — *"um tipo escolhido tem de FAZER alguma coisa"* — e o Spray teve de armar um default pela
mesma razão. Não alterado aqui: é número de produto.

---

## 4. Nota operacional

O `a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs` **falha na suíte completa e
passa serial** — medido: `--test-threads=1` dá **1234/1234**, com `load average 0,52`. Ele é uma
razão de **relógio** entre duas telas de 4096², e 32 testes concorrentes a tocar centenas de MB a
destroem. Não é carga da máquina e não é código: o Wet Paint nem entra no Solid
(`solid_owns_the_gesture` o recusa).
