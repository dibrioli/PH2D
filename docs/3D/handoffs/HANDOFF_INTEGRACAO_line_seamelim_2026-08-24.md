# HANDOFF DE INTEGRAÇÃO — `line/seamelim` (2026-08-24)

> **A COSTURA DEIXA DE SER PENALIZADA E PASSA A SER ELIMINADA.** Obra A da
> [`SPEC_restricoes_por_eliminacao.md`](../cleanroom/SPEC_restricoes_por_eliminacao.md).
> ⛔ A obra B (linhas de feição) **não** é desta linha, por ordem do §6 da própria espec.

## 1 — Identidade

| | |
|---|---|
| branch | `line/seamelim` |
| HEAD | `4854223a9` (**10** commits) · ⚠️ a linha ainda ganha 1 commit ao corrigir esta própria tabela |
| **base do fork** | ⚠️⚠️ **`line/quadextract`**, e **NÃO `main`** |
| merge-base com `main` | `5038249c6` |
| worktree | `Worktrees/line-seamelim/` |

⛔⛔ **A ORDEM DE INTEGRAÇÃO NÃO É LIVRE.** Esta linha descende de `line/quadextract`,
que por sua vez descende de `line/sculpt3d`. ⇒ ou as três entram **nessa ordem**
(`sculpt3d` → `quadextract` → `seamelim`), ou entram juntas. Integrar esta primeiro
arrastaria os commits das outras duas para dentro dela, e o integrador veria a mesma obra
duas vezes.

## 2 — Foundational / compartilhado tocado, e por quê

| arquivo | o que mudou | aditivo? |
|---|---|---|
| `crates/ph2d-gridmap/src/weld.rs` · `weld_flat.rs` · `weld_solve.rs` · `weld_round.rs` (+ os `*_tests.rs`) | **novos** — o sistema soldado inteiro | ⭐ ficheiros novos |
| `crates/ph2d-gridmap/src/lib.rs` | 4 `pub mod` + 4 `pub use` | aditivo |
| `crates/ph2d-gridmap/src/solve.rs` | `Assembly` ganha `by_vert`; o numerador de Poisson passa a ter **uma porta** (`poisson_numerator`) | ⚠️ refactor sem mudança de comportamento — o `Relaxer` passou a chamar a porta em vez de repetir a conta |
| `crates/ph2d-gridmap/src/round.rs` | `RoundOptions::welded_rounds` e `RoundReport::{weld, seam}` novos; doc de `shift_frac_max` emendado | aditivo |
| `crates/ph2d-gridmap/src/round_tests.rs` | `chain()` passa a `pub(crate)` (a sonda da soldadura usa a MESMA cadeia) | aditivo |
| `crates/ph2d-quadextract/tests/gate_seam_closes.rs` | **novo** — o gate nº1 medido no mapa por canto | ficheiro novo |
| `crates/ph2d-quadextract/examples/chain_info.rs` | bifurca para o caminho soldado | aditivo |
| `shells/desktop/src/sculpt3d_history_retopo_extract.rs` | bifurca para o caminho soldado **dentro** de `PH2D_RETOPO_EXTRACT` | aditivo |

⚠️ **Nenhuma crate fora de `ph2d-gridmap` mudou de comportamento por omissão.** O
`shells/desktop` só muda dentro de um caminho que já shipa desligado.

## 3 — Símbolos que podem COLIDIR

**Novos `pub` em `ph2d-gridmap`** (todos em módulos novos, exceto onde dito):
`weld::{Weld, WeldReport, Closure, SeamResidual, weld, seam_residual, holonomy}` ·
`weld_flat::{ClosureSystem, FlatReport, Var}` ·
`weld_solve::{WeldSolveReport, solve_welded, ROUNDS}` ·
`weld_round::{round_welded, welded_enabled}` ·
`solve::poisson_numerator` (`pub(crate)`) ·
`round::RoundOptions::welded_rounds` · `round::RoundReport::{weld, seam}`.

**Variável de ambiente nova:** `PH2D_GRIDMAP_WELD` (`=0` volta ao caminho penalizado).
⚠️ Ela é lida por **uma** função (`ph2d_gridmap::welded_enabled`) — as duas portas
(instrumento e produto) chegaram a lê-la com sentidos opostos, e isso está curado.

**Nenhum id, const de schema, número de ADR ou entrada em lista ordenada foi tomado.**
A saída de `collision-surface.sh` (rodada nesta worktree, 2026-08-24):

```
  merge-base 5038249c6  ·  33 commit(s)  ·  73 arquivo(s)
  PROJECT_SCHEMA 95 (base 95) · tripla (95,13,14) (base igual)
  VEC_SCENE 14 · FLIP 13 · DOC_VERSION 18   — todos iguais à base
  ph2d-ecs 69 · ph2d-render 70 · ph2d-script 70 — todos iguais à base
  contrato congelado: node.rs INTOCADO · tool.rs INTOCADO
  ADR: último 0164, próximo livre 0165  ⚠ o 0164 é da line/quadextract, NÃO desta
  Cargo.lock: 1 pacote novo, `ph2d-quadextract` — INTERNO (vem da line/quadextract)
  marcadores de conflito: nenhum · tetos de LOC: nenhum arquivo passa
```

⚠️ **PRAZO DE VALIDADE:** esta tabela mede contra o `main` de 2026-08-24. O integrador
**re-roda `collision-surface.sh` em cada worktree imediatamente antes de fundir**.

## 4 — Contratos congelados encostados

**NENHUM.** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` intocados.

## 5 — O que só o `ship.sh` pega

- `cargo fmt` — os ficheiros novos foram escritos à mão e **não** passaram por `fmt`.
- `machete` — **nenhuma dependência nova** foi acrescentada a nenhum `Cargo.toml`.
- `clippy --all-targets` latente nas três crates tocadas (rodado nesta linha; ver §7).
- `typos` — os docs são densos em português; nunca correu aqui.
- `RUSTSEC` — sem deps novas.

## 6 — O que SMOKAR, e o que ficou por smokar

⭐ **O smoke é o botão `Quad Retopology` com o caminho de extracção ligado.** Comando
inteiro, da worktree desta linha:

```
env PH2D_SCULPT3D_SMOKE=35 PH2D_RETOPO_EXTRACT=1 cargo run -p ph2d-host-desktop --release
```

⚠️ **Nada disto foi visto por olho humano nesta janela** — a medição é toda por
instrumento (`chain_info`) e por gate. O veredito de produto é do Enio.

## 7 — O que foi MEDIDO (é o que justifica a obra)

### 7.1 — A estrutura que a eliminação encontra

| peça | cópias → classes | ligações | **eliminadas** | fecham ciclo | que **rodam** | singularidades |
|---|---|---|---|---|---|---|
| esfera 24×36 | 3 013 → 2 544 | 497 | **469 (94,4 %)** | 28 | **8** | **8** |
| esfera fina 96×144 | 3 092 → 2 608 | 513 | **484 (94,3 %)** | 29 | **8** | **8** |
| toro 64×32 | 3 042 → 2 482 | 605 | **560 (92,6 %)** | 45 | **12** | **12** |

⭐ As ligações que **rodam** são *exactamente* as singularidades, nas três peças — a
derivação bate contra o produto sem nenhum ajuste.
⭐⭐ **Zero órfãs:** toda restrição de fecho elimina uma variável.

### 7.2 — O produto, A/B no corpus (mesmo binário, mesma corrida)

| peça | resíduo de costura | células não-fechadas | arestas de bordo | `χ` | enviesamento p50 | `>60°` | G3+G5 |
|---|---|---|---|---|---|---|---|
| enrugada | `1,000` → ⭐ **`0,000`** | 12 → ⭐ **0** | 46 → ⭐ **0** | `−8` → ⭐ **`+2`** | 5,7° → 6,1° | 4 → ⚠️ 10 | 19,5 → **5,6 s** |
| orelha | `1,000` → ⭐ **`0,000`** | 20 → ⭐ **0** | 50 → ⭐ **0** | `−6` → ⭐ **`+2`** | 7,1° → ⚠️ 8,1° | 7 → 7 | 15,8 → **4,7 s** |
| gancho | `1,000` → ⭐ **`0,000`** | 22 → ⭐ **0** | 78 → ⭐ **0** | `−13` → ⭐ **`+2`** | 6,9° → 7,5° | 3 → ⚠️ 9 | 14,5 → **3,8 s** |
| encrespada | `1,000` → ⭐ **`0,000`** | 10 → ⭐ **0** | 30 → ⭐ **0** | `−4` → ⭐ **`+2`** | 5,5° → 5,6° | 6 → 7 | 14,7 → **4,4 s** |
| esfera ruidosa | `1,000` → ⭐ **`0,000`** | 13 → ⭐ **0** | 46 → ⭐ **0** | `−8` → ⭐ **`+2`** | 7,0° → 7,0° | 0 → 1 | 15,4 → **4,1 s** |
| ⛔ **perfurada** (tem bordo) | `1,423` → ⭐ `0,000` | 47 → 11 | 142 → 76 | `−8` → `+1` | ⛔ 15,9° → 23,4° | ⛔ 58 → 87 | 17,4 → **3,0 s** |

⚠️ **A barra do oráculo é aspecto p50 `1,08`–`1,22`, enviesamento p50 `4,8°`–`7,1°`,
`>60°` entre `0` e `4`.** O aspecto fica dentro em todas; o enviesamento sai por `1,0°`
na orelha e por `0,4°` no gancho; e ⚠️ **o `>60°` regride em três das seis**.

### 7.3 — ⚠️ A REGRESSÃO QUE FICA, e a cura já tem nome

O `>60°` sobe de `4` para `10` na enrugada e de `3` para `9` no gancho. **A causa é
distorção métrica local perto de singularidades** (o mapa soldado tem `2` triângulos
dobrados no domínio onde o penalizado tinha `0`), e o mecanismo publicado que a ataca é o
*local stiffening* do mesmo *paper* de 2009 (§5.4): pesar por triângulo o que ficou
distorcido e re-resolver, iterativamente.

⛔ **Ele NÃO entrou nesta wave, de propósito** (§6 da espec): com dois mecanismos dentro,
uma regressão de forma fica sem dono. ⚠️ E os coeficientes dele **têm de ser medidos no
nosso corpus**, não copiados do *paper* (§7 da espec).

### 7.4 — A peça PERFURADA é o caso de bordo, e ela é o gate nº8 por medir

⚠️ Ela **tem** bordo de verdade, então `χ = +2` não é o alvo dela (`+1` = disco é
plausível). O caminho soldado **melhora** a topologia (bordo `142` → `76`) e **piora** a
forma. ⇒ *o gate nº8 da espec («o bordo é preservado») ainda não tem régua* — o número
está aqui, o veredito não.

## 8 — Gates novos, e as provas

| gate | onde | o que afirma |
|---|---|---|
| `an_eliminated_seam_link_is_closed_to_the_floor_of_f32` | `ph2d-gridmap` | o resíduo de uma ligação eliminada é o chão da representação |
| `the_welded_seam_residual_is_zero_on_both_kinds_of_link` | `ph2d-gridmap` (`#[ignore]`) | e vale nas DUAS espécies de fecho |
| `every_transition_of_the_welded_map_is_an_integer_translation` | `ph2d-gridmap` (`#[ignore]`) | a distância a inteiro **antes** do encaixe está no chão de `f32` |
| `every_seam_link_is_either_eliminated_or_a_closure` | `ph2d-gridmap` | a contagem fecha — foi ele que achou o defeito da §9.1 |
| `the_crossings_predict_how_a_translation_moves_a_copy` | `ph2d-gridmap` | ⭐ controlo directo da tabela de derivadas |
| `the_incremental_bump_agrees_with_the_full_apply` | `ph2d-gridmap` | ⭐ as duas portas da propagação concordam |
| `the_reference_maps_close_their_seams_at_the_floor_of_their_own_precision` | `ph2d-quadextract` | a barra **lê-se** da referência |
| `our_welded_map_closes_its_seams_at_the_floor_of_f32` | `ph2d-quadextract` (`#[ignore]`) | ⭐⭐ o gate nº1, medido **no mapa por canto** |

**Provas de mutação** (os três controlos no arnês — verde-antes · `Compiling ph2d-gridmap` · `running 1 test`):

| mutação | resultado |
|---|---|
| desligar a eliminação das **cópias** | ⭐ VERMELHO: resíduo `1,86e10` |
| desligar **só** a eliminação dos fechos | ⭐ VERMELHO **cirúrgico**: eliminadas ficam em `2,38e-7`, fechos vão a `7,07` e `10,20` |

## 9 — ⛔ RECUSAS MEDIDAS (não reconstrua)

| recusa | mecanismo | número |
|---|---|---|
| escrever `t` de um fecho **plano** por alternância | a translação é partilhada pela costura inteira; escrevê-la de um vértice arrasta a costura | toro a `568` de resíduo, passo `2,9e-1` |
| pôr as translações no subespaço de **calibre** | o subespaço é pequeno demais — um ciclo que envolve singularidades tem holonomia legítima | ângulo `60°`–`85°`, escala `0,22`–`0,50` |
| as duas eliminações em **subsistemas separados** | realimentação `t_sing = M·y + f(t_sing)` com ganho `> 1` | esfera **NaN**, toro `6,4e17` |
| **amortecer** essa realimentação | não é ganho, é estrutura | diverge em `1,0` · `0,5` · `0,25` · `0,125` · `0,0625` |
| deixar o caminho soldado herdar as `160 000` rondas do penalizado | ele é a Poisson pura e assenta em `8 000` | `60 s` contra `3 s` |

### 9.1 — Dois defeitos que os gates apanharam nesta janela

1. ⭐ **Uma ligação percorrida como árvore no sentido inverso era recontada como fecho**
   (`469 + 30 ≠ 497`), e as duas a mais reapareciam a jusante como equações **órfãs**.
   ⇒ a ligação conta-se pela **identidade**, nunca pelo sentido.
2. ⭐ **A propagação incremental apanhava só o PRIMEIRO termo de cada livre** numa
   expressão em que ela pode aparecer duas vezes (dois caminhos, e os coeficientes
   somam). ⚠️ **O erro ficava mascarado** pela reconstrução do fim da cadeia.

## 10 — ⚠️ A PERGUNTA QUE ESTA LINHA DEVOLVE

A barra do gate nº1 passou a ser **lida** dos mapas de referência (emenda do E, 24/08).
⛔ **Mas eles são `f64` e o nosso `GridMap` é `f32`:** eles fecham a `1,4e-14`, o nosso
chão é `2,3e-5`. ⇒ a barra da referência é inalcançável **por representação, não por
algoritmo**, e a emenda não alcança essa metade.

⭐ **A forma que o gate usa hoje** (a mesma lei, sem depender da precisão): cada mapa
compara-se com o **chão da precisão em que foi calculado**.

| mapa | resíduo no canto | chão | razão |
|---|---|---|---|
| referência gancho (`f64`) | `1,42e-14` | `9,77e-14` | `0,145` |
| referência toro (`f64`) | `1,42e-14` | `9,68e-14` | `0,147` |
| ⭐ **nosso soldado (`f32`)** | `9,54e-7` | `2,34e-5` | ⭐ **`0,041`** |

⇒ *o nosso mapa está mais perto do chão dele do que a referência do dela.* **Se o E
preferir outra forma, é uma emenda** — o número está medido.

## 11 — Ler

- a espec: [`SPEC_restricoes_por_eliminacao.md`](../cleanroom/SPEC_restricoes_por_eliminacao.md)
- a porta do cofre da sala limpa: [`docs/3D/cleanroom/`](../cleanroom/)
- o instrumento: `cargo run --release -p ph2d-quadextract --example chain_info -- <peça|ficheiro.obj>`
