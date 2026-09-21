# ADR-0171 — O borrão de CAIXA da pilha parte-se em fatias, e a largura da banda é MEDIDA

- **Status:** Accepted
- **Data:** 2026-09-21
- **Linha:** `line/PainterWatercolor`
- **Amparo:** ADR-0109 (a cerca de contenção do `rayon`) · ADR-0158 (a 1.ª excepção nesta crate) ·
  ADR-0145 (o precedente do solver row-parallel)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — quem integrar reconta-o contra o `main` do
> dia, nunca o copia daqui.

## Contexto

Report do dono (2026-09-21): *«Performance ruim»* ao arrastar uma figura com o Composite Brush.

A atribuição está medida (`--release`, `91`–`96 %` de CPU ociosa, por evento de carimbo):

| `2048²` | CARIMBAR | pre | acumular | **COMPOR** | cópias |
|---|---|---|---|---|---|
| `Brush/Brush` | `9,48` | `0,05` | `3,22` | **`5,60`** | `1,55` |
| `Blur/Brush` | `51,86` | `0,06` | `3,24` | **`49,31`** | `1,97` |
| 7 camadas | `84,74` | `0,04` | `9,44` | **`79,92`** | `1,68` |

⇒ **UMA camada `Blur` custa `8×` uma `Brush`** (`43,7` dos `49,31 ms`), e o COMPOR é `90`–`99 %`
do carimbo.

**Três coisas foram medidas e fecham as saídas fáceis:**

1. **Não é o núcleo nem o avental.** Varrido o tamanho da camada, `k` anda `14×` (`8` → `112`) e o
   COMPOR fica **plano por unidade de área** (`82,3 · 76,2 · 75,5 · 77,0 · 78,8` ms por tela cheia,
   `±4 %`). A caixa é `O(1)` em `k` por construção.
2. ⛔ **Não é a alocação.** A cadeia aloca **sete** `Vec<[f32; 4]>` do tamanho da região (`~250 MB`
   por composição a `2048²`) e isso custa **`0,00 ms`**: `vec![[f32; 4]; n]` é `alloc_zeroed`, as
   páginas chegam preguiçosas e quem as toca é a passagem. *Contar bytes alocados não é medir o
   custo de os alocar.*
3. ⛔ **Não se cura reduzindo a ÁREA.** Um anel ocupa `~5 %` da caixa envolvente dele, mas um cover
   por BLOCOS paga o avental em cada bloco. Medido sobre a lista de dabs REAL (`1,00×` = a caixa):

   | | `16 px` | `32` | `64` | `128` | `256` |
   |---|---|---|---|---|---|
   | `1024²` | `6,66×` | `3,16×` | `1,99×` | `1,62×` | `2,39×` |
   | `2048²` | `3,63×` | `1,68×` | `1,01×` | **`0,89×`** | `0,99×` |

   A `1024²` **todo** tamanho é pior que a caixa; a `2048²` o melhor poupa `11 %`. *A família
   inteira está refutada, não um tamanho.*

Sobra o custo POR PIXEL: `31,1 ns/px` para `~224 B/px` (sete passagens de `[f32; 4]`, lidas e
escritas) = **`6,8 GB/s`**, que é da ordem de UM núcleo. O soquete tem folga medida:

| fios | 1 | 2 | 4 | 8 | 16 |
|---|---|---|---|---|---|
| débito | `6,8` | `12,0` | `20,2` | `27,4` | `30,1 GB/s` |
| ganho | `1,00×` | `1,78×` | `2,99×` | **`4,05×`** | `4,46×` |

## Decisão

**As passagens do núcleo de CAIXA (`blur_caixa`) partem-se em fatias com `rayon`** — o avental, as
três horizontais, as três verticais e a divisão que desfaz a premultiplicação.

Isto é o **segundo** uso do `rayon` nesta crate, e a cerca do ADR-0109 exige que ele seja nomeado.
Ele é tão estreito como o primeiro:

- **Só o núcleo de CAIXA**, que um censo já afirma ser pedido por **um** sítio de toda a crate — o
  laço da pilha do Composite (`o_nucleo_de_caixa_e_do_blur_da_pilha_e_so_dele`). ⛔ **O Blur como
  ferramenta isolada continua no binomial e não é tocado**, que é a ordem do dono de 2026-09-20.
- **Byte-idêntico, e não «igual a menos de um epsilon»** — com gate (`as_duas_rotas_dao_o_mesmo_f32`).
- O `blend_blurred`, que é partilhado com o caminho binomial, **fica em série**.

### A forma da fatia é a decisão, e ela é diferente nos dois eixos

- **Horizontal:** uma linha de saída é função só da linha de entrada dela ⇒ uma fatia por thread,
  e ela escala **`3,6×`** (contra o tecto do soquete, `4,05×`).
- **Vertical:** a fatia tem de ser de **COLUNAS**. ⛔ Com fatias de LINHAS cada uma teria de
  re-semear o acumulador somando `lado` linhas de fresco, e esse `f32` **não é** o que a soma
  corrida acumulou até ali — *a mesma não-associatividade que a cerca desta crate já nomeia para o
  depósito das arestas*.

### ⛔⛔ E o número de fatias da vertical NÃO é o número de threads

Em série a vertical percorre cada linha INTEIRA e é um fluxo sequencial; parti-la em `nb` bandas de
colunas transforma-a em `nb` fluxos com passo `w`. Abaixo de uma certa largura ela **perde para si
própria** (região `1 484²`, `k = 24`, `91 %` ocioso, mínimo de 7):

| fatias | largura | ms | ganho |
|---|---|---|---|
| 1 | `1 484` | `8,417` | `1,00×` |
| 2 | `742` | `5,341` | `1,58×` |
| 3 | `494` | `4,203` | `2,00×` |
| **4** | **`371`** | **`3,749`** | **`2,25×`** |
| 6 | `247` | `3,973` | `2,12×` |
| 8 | `185` | `5,082` | `1,66×` |
| 16 | `92` | `7,352` | `1,14×` |

⇒ o que se fixa é a **LARGURA** (`LARGURA_MINIMA_DA_BANDA = 384`, o meio do planalto) e a
**CONTAGEM sai dela**, com tecto na pool.

⚠️ *Isto não contradiz a lei de «não fixar o número de pedaços»* (que a `line/motion-value` pagou
com o app do dono a piorar `18,6 → 30,4 ms`): a contagem continua a seguir a máquina pelo tecto, e
o que a medição acrescenta é um **PISO que vem do acesso à memória**, não do escalonador.

### E há um joelho para partir de todo

| região | `64²` | `128²` | `192²` | **`256²`** | `384²` | `768²` | `1 484²` |
|---|---|---|---|---|---|---|---|
| ganho | `0,44×` | `0,76×` | `0,94×` | **`1,01×`** | `1,23×` | `1,67×` | **`2,09×`** |

`PIXEIS_PARA_PARALELIZAR = 256²` é o primeiro tamanho em que ele **deixa de perder**.

## Consequências

- Na região que o produto compõe (`1 484²` num carimbo de figura a `2048²`) o borrão passa de
  `54,0` para **`25,9 ms`** (`2,09×`), medido a `43 %` de CPU ociosa — ou seja, com a máquina
  ocupada por outra linha.
- ⚠️ **O ganho é `2,09×` e não `4,05×`**, e a diferença está medida e nomeada: a sonda do soquete
  mede `N` borrões INDEPENDENTES, e partir UM paga sete barreiras de junção mais o piso de largura
  da vertical, que a limita a `2,25×` enquanto a horizontal faz `3,6×`.
- ⏳ **O que fica NOMEADO e não feito:** os sete intermédios são `[f32; 4]` = `16 B/px`, e o
  borrão é limitado por LARGURA DE BANDA ⇒ *metade dos bytes vale o mesmo que dois fios, sem
  política de concorrência nenhuma*. Um intermédio de `u16` premultiplicado é `2×` de graça — mas
  ele **não é byte-idêntico**, logo pede a mesma barra de qualidade que a caixa já tem contra o
  binomial (pior byte `1`–`3`), e é wave própria.

## Alternativas medidas e recusadas

- **O cover por blocos** — a tabela acima; refutada a família inteira, não um tamanho.
- **Reaproveitar os sete buffers** — refutada: alocá-los custa `0,00 ms`.
- **Fatias de LINHAS na vertical** — recusada por MECANISMO: não é byte-idêntica.
- **Uma fatia por thread na vertical** — recusada por MEDIÇÃO: `1,14×` contra `2,25×` de quatro.
