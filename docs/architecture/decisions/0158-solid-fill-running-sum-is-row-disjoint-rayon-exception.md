# ADR-0158 — A soma corrida do preenchimento é POR LINHA, e por isso o `rayon` entra na `ph2d-painter-brush`

- **Status:** **ACEITO** pelo Enio em 2026-08-15 (*"siga e corrija os abertos"*), sobre o item 1 da
  auditoria do `Style: Solid` — que foi apresentado com o preço ao lado (*"48% do que sobrou, e a
  crate não tem `rayon`: é dep nova, logo ADR e ordem"*).
- ⚠️ **O 0158 é o próximo livre no `main` de 2026-08-15** (o último é o 0157). Número de ADR escolhido
  numa linha paralela é **provisório**: se outra linha reivindicar o mesmo na mesma janela,
  **renumera na integração** — já aconteceu **oito** vezes neste repo
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
- **Data:** 2026-08-15
- **Linha:** `line/Painter`
- **Auditoria:** [`docs/Painter/39_auditoria_solid_e_tracos.md`](../../Painter/39_auditoria_solid_e_tracos.md)
- **Ampara-se em:** [ADR-0109](0109-rayon-exception-watercolor-composite.md) (a regra e a cerca) ·
  [ADR-0147](0147-wet-paint-order-invariant-solver.md) (o precedente da soma em `f32` privada da linha)

## O problema, e a força que obriga a decidir agora

Um `Style: Solid` vivo é um **re-carimbo**: a cada ponto novo o polígono inteiro muda, então o produto
refaz a mancha do zero a cada evento de ponteiro. Sob **simetria circular** o retângulo dessa mancha é
a **tela inteira já no primeiro evento** — a rosácea de doze cópias abre para 1 048 576 px num canvas
de 1024².

**Medido pela porta do produto** (1024², circ12 + Tiling, evento 96 de um traço), depois de o `over`
já ter ido para as linhas:

| peça | ms |
|---|---:|
| construir os laços | 0,029 |
| **`solid::fill_coverage`** | **1,414** |
| `write_solid` (o `over`, já paralelo) | 0,260 |
| `save_region` + `restore_region` | 0,147 |
| **transação** | **2,926** |

O `fill_coverage` é **48% do que sobrou**, e a `ph2d-painter-brush` **não tem `rayon`**. A força que
obriga é a do `CLAUDE.md` §0: a máquina tem 32 núcleos e o passe roda em **um**, num caminho que o
artista paga **por movimento do rato**.

## Decisão

> **O `rayon` entra na `ph2d-painter-brush` exclusivamente na SOMA CORRIDA do `solid::fill_coverage`,
> porque a cobertura de uma linha é a soma da derivada DAQUELA linha e nada atravessa a fronteira
> horizontal. O depósito das ARESTAS continua serial, e o mecanismo é aritmético — não preguiça.**

### Por que a soma corrida qualifica — os 3 invariantes do ADR-0109

1. **Sem redução ENTRE linhas.** `out[y][x]` é função pura de `acc[y][0..=x]`, que é imutável nesta
   altura. A soma corrida é **privada da linha** e percorre as células **na mesma ordem** nas duas
   rotas ⇒ o resultado em `f32` é o mesmo bit a bit — o precedente exacto do
   [ADR-0147](0147-wet-paint-order-invariant-solver.md). O que a condição 3 do
   [ADR-0145](0145-wet-paint-solver-row-parallel-passes-rayon-exception.md) recusa é soma cuja **ordem
   entre threads** possa mudar; esta não atravessa thread nenhuma.
2. **Sem estado mutável compartilhado.** Cada tarefa escreve **só** a sua fatia de `out`
   (`par_chunks_mut(w)`), e lê `acc` por `&`.
3. **Sem RNG e sem transcendental.** Dentro do laço há `+`, `abs`, `min`, uma multiplicação e um
   `as u8` — todos especificados exactamente pelo IEEE-754.

### ⛔ Por que as ARESTAS **não** qualificam — e não é escolha

O depósito faz `acc[cell] += d`, e **a adição em `f32` não é associativa**: duas arestas que caem na
mesma célula têm de ser somadas na **mesma ordem**. Uma banda de linhas preserva isso — uma célula
pertence a uma linha, logo a uma banda —, **mas** o `x` de uma aresta é caminhado
**incrementalmente** (`x = x_next`, linha a linha). Uma banda que comece no meio de uma aresta teria
de recomputar `x` directamente, e `p0.x + dxdy·(r − p0.y)` **não é bit-igual** à soma incremental.

Para manter a identidade cada banda teria de caminhar a aresta **desde o começo dela**, o que exige
um pré-filtro por banda: **29 040 arestas × 32 bandas ≈ 930 k testes de intervalo**, medidos como
**mais caros que o passe serial inteiro** (0,94 ms). Pré-binar as arestas por banda é possível
(counting sort por linha) e fica **NOMEADO e não construído** — é wave própria, com o número dela ao
lado.

## A prova, que é medição e não raciocínio

⚠️ **O A/B é costas-com-costas DENTRO da mesma corrida**, sobre a MESMA entrada, e a forma é a lição
do [doc 28 §5.46](../../Painter/28_otimizacoes_o_que_funcionou.md): esta workstation é compartilhada e
o MESMO passe foi medido entre **1,01 e 1,43 ms sem uma linha de código mudar**. Comparar duas
corridas atribuiria a deriva da máquina ao ganho.

Janela 1024×1024, sob `load average 16`:

| conjunto | pontos | serial | paralela | ganho |
|---|---:|---:|---:|---:|
| piso de ÁREA (um triângulo de 3 px) | 3 | 0,895 | 0,184 | **4,86×** |
| rosácea de 12 | 14 520 | 1,377 | 0,635 | **2,17×** |
| rosácea de 24 (com Tiling) | 29 040 | 1,970 | 1,122 | **1,76×** |

A diferença entre a rosácea e o piso é o depósito serial das arestas (0,94 ms), que esta decisão
**não** toca — e é por isso que o ganho cai com o número de pontos.

**Identidade:** `both_walkers_of_the_running_sum_write_the_same_bytes`, sobre uma fixture com laços
que **se cruzam** (uma estrela de cinco pontas e um losango oblíquo de coordenada quebrada) — sem
cruzamento o `nonzero` nunca soma duas contribuições na mesma célula e a identidade sairia verde por
vácuo, e sem obliquidade não há borda anti-aliased (o controle da fixture nasceu **vermelho** por
isso, com 59 texels de meio-tom). **Mutação que sangra:** inverter a ordem das linhas na rota
paralela ⇒ 350 texels divergem.

⚠️ **A porta `fill_coverage_routed` existe para o gate poder existir**, e não como knob: toda fixture
de Solid do repo roda **abaixo** do piso do pool, então sem ela a rota paralela shipava **sem um
único teste** — o mesmo buraco que o irmão dela no tool já tinha.

## Consequências

- A cerca do ADR-0109 vale: **todo uso novo de `rayon` nesta crate exige ADR novo**, e o
  `Cargo.toml` dela passa a dizer isso ao lado da dep.
- O piso do pool é o `PARALLEL_MIN_AREA` do kernel de dab, e **não um número novo**: a pergunta é a
  mesma — *quantos texels é preciso percorrer para o fork valer a pena?* — e dois números para uma
  pergunta divergem no dia em que alguém afinar um deles.
- Esta é a **4ª exceção** do repo (as duas do `ph2d-wet-paint`, a da `ph2d-sdf`, e esta).
