# O censo de cor — quantas cores precisamos mesmo (2026-08-30)

> O outro pré-requisito nomeado em [`01_o_estado_medido.md §6`](01_o_estado_medido.md).
> O Enio pediu *"menos cores na paleta"*. ⛔ Cortar sem contar é escolher; isto conta.
>
> Fonte única: `docs/design/tokens.json`, campo `themes.<tema>.color`.

## §1 — O que existe

| tema | slots declarados | valores distintos | slots que repetem outro |
|---|---:|---:|---:|
| `forge` (escuro+magenta, default) | 83 | 67 | **16** |
| `sunstone` (claro+laranja) | 83 | 66 | 17 |
| `blueprint` (claro+azul) | 83 | 66 | 17 |
| `workshop` (escuro+ciano) | **16** | 7 | 9 |

⚠️ **O `workshop` declara só 16 slots e herda os outros 67** (mecanismo de alias/override em
`ph2d-tokens/src/alias_walk.rs` + `overrides.rs`). ⇒ ele **já prova** que um tema não precisa de
redeclarar a paleta toda. *A máquina de herança que a spec nova quer já existe e está em uso.*

## §2 — ⭐⭐ Os 83 slots, por prefixo: 62 % são de DOIS módulos

| prefixo | slots | de quem é |
|---|---:|---|
| **`timeline-`** | **16** | Timeline |
| `graph-` | 12 | Motion Nodes |
| `node-` | 7 | Motion Nodes |
| `port-` | 7 | Motion Nodes |
| `bg-` | 6 | geral |
| `accent-` | 5 | geral |
| `text-` | 4 | geral |
| `border-` · `curve-` · `axis-` | 3 cada | geral / nós |
| `danger` `success` `warn` `info` `grid` | 2 cada | geral |
| `rail` `panel` `selection` `focus` `canvas` `wire` `attr` | 1 cada | geral / nós |

⇒ **o sistema GERAL tem ~31 slots.** Os outros **52 são privados de dois módulos** —
Timeline (16) e Motion Nodes (34, contando `graph`+`node`+`port`+`curve`+`axis`+`wire`+`attr`).

## §3 — ⭐⭐⭐ Os 16 do Timeline são apelidos PUROS

Cruzando os três temas plenos (`forge`, `sunstone`, `blueprint`), há **8 grupos de slots que
carregam valor idêntico nos três**. Colidirem nos três não é coincidência de paleta: é o **mesmo
papel** escrito duas vezes.

| grupo | é, na verdade | slots a mais |
|---|---|---:|
| `timeline-curve` · `timeline-handle` · `timeline-key-selected` · `timeline-loop-brace` · `timeline-playhead` · `timeline-summary-ring` | **`accent`** | 6 |
| `timeline-handle-line` · `timeline-loop-region` | `accent-soft` | 2 |
| `timeline-row-alt` · `timeline-ruler-bg` | `bg-2` | 2 |
| `timeline-marker` · `timeline-summary-key` | `warn` | 2 |
| `timeline-key-active` | `accent-press` | 1 |
| `timeline-missing` | `danger` | 1 |
| `timeline-key` | `text-1` | 1 |
| `timeline-ruler-tick` | `text-3` | 1 |
| | | **16** |

⭐⭐⭐ **Os 16 slots `timeline-*` são exactamente os 16 apelidos. Todos eles.** O módulo Timeline
cunhou uma paleta própria em que **nenhuma cor é nova** — cada uma repete um slot geral, nos três
temas.

⇒ **Fundi-los é uma mudança de zero pixels.** `83 → 67 slots`, e o ecrã fica byte-a-byte igual.
Não é uma decisão de gosto: é remover duplicação provada.

⚠️ **E a prova tem de correr no gate**, não ficar neste documento — senão o 17.º apelido nasce
amanhã. A forma é um teste que, para cada par de slots com valor idêntico nos três temas plenos,
falha e nomeia o par.

## §4 — Os 34 dos nós NÃO são apelidos, e isso também é o resultado

`graph-*`, `node-*`, `port-*`, `curve-*`, `axis-*`, `wire`, `attr` **não colidem** com nenhum
slot geral em nenhum tema. São valores genuinamente distintos.

⇒ ⛔ **Não os corte por analogia com o Timeline.** A pergunta sobre eles é outra e é de produto:
*um editor de grafo precisa de 34 cores próprias?* — o que se responde olhando para o que cada
uma pinta, não comparando valores. **Não medido.**

## §5 — O corte, em três degraus

| degrau | de → para | custo visual | natureza |
|---|---|---|---|
| **1. fundir os 16 apelidos do Timeline** | 83 → **67** | **zero** | duplicação provada |
| **2. rever os 34 dos nós** | 67 → ? | a medir | produto |
| **3. reduzir os temas** | 4 → 2 | perde-se `workshop`/`blueprint` | ⚠️ **produto — e o `blueprint` é o ÚNICO com painéis ancorados** |

⛔⛔ **O degrau 3 tem uma armadilha nomeada:** o tema `blueprint` é hoje o único que liga
`PanelLayout::Sidebar` (`theme.rs:54`). **Apagá-lo antes de libertar o layout do tema apaga o
único modo ancorado do app.** ⇒ a ordem é: **primeiro separar layout de paleta, só depois mexer
nos temas.**

## §6 — O impacto no total

Hoje: **355 tokens, 273 de cor (77 %)**.

- Degrau 1 (16 apelidos × 3 temas plenos + os que o `workshop` declare): **−48 a −64 entradas**,
  sem uma linha de produto mudar.
- Degrau 3 (4 → 2 temas): **−~130** entradas.

⇒ um sistema com **~2 temas × ~50 slots ≈ 100 entradas de cor**, contra 273 — mas o degrau 1 é o
único que é dedução, e os outros dois são decisões do Enio.
