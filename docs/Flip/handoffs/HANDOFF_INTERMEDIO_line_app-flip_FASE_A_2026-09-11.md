# Handoff INTERMÉDIO — `line/app-flip`, W2 Fase A (2026-09-11)

> **Fase A fechada. A Fase B espera a `line/app-host` integrar** (o Enio avisa; então
> `git rebase main` + ler o `HOWTO_partir_uma_familia_da_shell.md` + o corte).
> Contexto: [`BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md`](../../IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md)
> §5 · [auditoria de velocidade](../../DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md) §4-C2.

## 1 — Identidade

| | |
|---|---|
| branch | `line/app-flip` |
| merge-base | `8fa4f115b` (main de 11/09) |
| commits | 5 |
| contadores partilhados movidos | **NENHUM** — `PROJECT_SCHEMA` 128, `FLIP_SCHEMA` 13, `DOC_VERSION` 18, registos 86/86, iguais à base |
| contratos congelados (§6) | **intocados** (`node.rs`, `tool.rs`) |
| ADR novo | nenhum ⇒ fora de toda disputa de número |

## 2 — ⛔⛔ O ACHADO QUE ATRAVESSA AS SEIS LINHAS: o censo do briefing conta `impl App` e o código escreve `impl crate::App`

O §2 do briefing diz **«flip: 0 `impl App`»** e a tarefa desta linha repete-o —
*«0 `impl App` (funções soltas) — a Fase A é quase toda mover e agrupar»*. **É falso, e a causa é
o grep.** Medido na árvore de 11/09, com `grep -lE '^impl crate::App'`:

| família | censo do briefing | `impl crate::App` medido | real |
|---|---:|---:|---:|
| flip | **0** | **33** | **33 ficheiros / 34 blocos / 74 métodos** |
| physics | 22 | 58 | 80 |
| motion | **0** | 10 | 10 |
| vec | 3 | 10 | 13 |
| sculpt3d | 6 | 1 | 7 |
| **field3d (o PILOTO da L0)** | 1 | 1 | **2** |

⚠️ **Isto muda a premissa de escolha do piloto e o orçamento de três linhas** (L1 motion e L5 flip
foram abertas como «quase só mover», e L2 physics foi orçada a 22 quando são 80). Não muda o
*plano*, muda o **preço**. *O censo mediu a forma de escrever, não a coisa escrita.*

## 3 — ⭐⭐⭐ O GATE que prende a família INTEIRA, medido — é o item nº 1 para a L0

A família Flip são **43 módulos / 18 798 LOC**. Medindo o fecho transitivo de dependências (com os
blocos `impl crate::App` extraídos), o que a impede de sair da shell são **DUAS funções**:

```
crate::vec_transform::{world_transform, xform_of_transform}   5 sítios, 4 ficheiros
crate::name_unique::unique_name                               1 sítio,  1 ficheiro
```

| cenário | movível | fica |
|---|---:|---:|
| hoje | 16 mods / 3 620 LOC | 27 / 15 178 |
| só `vec_transform` disponível | 16 / 3 620 | 27 |
| só `name_unique` disponível | 17 / 3 848 | 26 |
| **as duas** | **43 / 18 798** | **0** |

⭐ **O gate é CONJUNTIVO e total:** nenhuma sozinha destrava nada; as duas juntas destravam
**tudo**. A cascata é `transform → vec_transform` e `transform → entities → name_unique`, e
`transform` é lido por `trace`, `edit_gesture`, `tween_correct`, `strip`, `draw`, `fill`, … — daí
27 módulos presos por 6 sítios de chamada.

**O que as duas são, medido:** matemática de ECS **genérica** que por acaso vive em ficheiros da
shell. O doc do nosso próprio `transform.rs` já o escrevia: *«Reusa os helpers GENÉRICOS de
`vec_transform` — eles não tocam `VecScene`, só o `SimWorld`/`Transform`»*. `world_transform` são
6 linhas sobre `ph2d_ecs`; `xform_of_transform` são 10 sobre `ph2d_ecs` + `ph2d_vec_scene::Xform`;
`unique_name` é `ph2d_ecs::SimWorld`.

⛔ **Esta linha NÃO lhes tocou, de propósito.** `vec_transform.rs` é ficheiro da família `vec_*`
(a L4) — movê-lo seria exactamente a colisão de mesmo-símbolo que a DIRETRIZ §1.5.5 manda parar e
reportar, e o briefing §1 proíbe («não desenhe a porta»).

⛔⛔ **E há uma saída que parece óbvia e está RECUSADA COM MEDIÇÃO:** `ph2d_ecs::world_transform`
**já existe** e é *quase* gémea — ela devolve `Option<Transform>` (`None` sem `Transform`) onde a
da shell cai para `Transform::IDENTITY`. **Não é substituição directa**, e re-derivá-la na crate
nova seria criar a segunda resposta à mesma pergunta, que é o defeito que este repo documenta
seis vezes. ⇒ *a cura é RELOCAR as duas, não reescrevê-las.*

## 4 — ⭐⭐ A shell é BIN-ONLY, e isso explica os 186 k LOC de teste dentro de `src/`

`shells/desktop/Cargo.toml` declara `[[bin]]` e **nenhum `[lib]`**. Logo `shells/desktop/tests/it/`
**não consegue ver `App`**: ele só faz gates de TEXTO (`include_str!` sobre o fonte) e usa outras
crates.

⇒ **A C2 degrau 3 do plano — «o que ficar na shell muda para `tests/`» — NÃO se aplica a nenhum
teste que toque `App`.** Os dois arneses desta família que constroem `crate::App::new()`
(`colorize_tests.rs`, `select_segment_tests.rs`) não podem mudar-se para `tests/`; a única forma de
saírem de `src/` é o sujeito deles sair da shell. Vale para as seis famílias.

## 5 — O que esta Fase A fez (com os números das duas pontas)

| | antes (main `8fa4f115b`) | depois | Δ |
|---|---:|---:|---:|
| `shells/desktop/src` ficheiros | 1 784 | 1 772 | **−12** |
| `shells/desktop/src` LOC | 493 252 | 488 039 | **−5 213** |
| campos de `App` | **300** | **280** | **−20** |
| `app_state.rs` LOC | 2 104 | 2 038 | −66 |
| linhas `mod flip_*;` no `main.rs` | 49 | **1** | −48 |
| `crates/ph2d-app-flip` | — | 32 ficheiros / 5 563 LOC | **novo** |

⚠️ **Uma correcção à minha própria medição:** eu reportei `App` com **311** campos no início. A
régua corria para além do `}` da struct e contava campos das structs seguintes. Com a **mesma**
régua nas duas pontas: `300 → 280`. *Duas leituras da mesma grandeza a discordar É o achado.*

**Os quatro passos, em ordem:**

1. **`FlipState`** (`13c6ef0db`) — os **21** campos `flip_*` de `App` viram **um**
   (`app.flip_state`). 267 sítios de acesso reescritos. `Default` derivado **é** o construtor de
   antes (os 21 nasciam de `false`/`None`/`::default()`), então não há construtor escrito à mão que
   possa divergir. ⚠️ Não confundir com `AppGfx::flip`, que é o **documento**.
2. **`src/flip/`** (`55ecb51e6`) — 78 ficheiros da raiz de `src/` para uma pasta;
   244 `crate::flip_X` → `crate::flip::X`; 29 `#[path]`; 48 citações de caminho reapontadas
   (código + 30 `.md`, fora de `docs/archive`).
3. **A crate nasce** (`bb007bb01`) — 6 árvores de módulo puras (13 ficheiros, 3 238 LOC).
4. **16 módulos partem-se em duas metades** (`02c6bcf5d`) — a lei e a cena na crate, o
   `<mod>_app.rs` na shell com o bloco `impl crate::App`. Mais o corte do `draw.rs` (`782c23471`).

## 6 — ⚠️ As três armadilhas que esta extracção pagou (as outras cinco linhas vão pagá-las)

1. **`include_str!` com caminho relativo, e o `check -p` NÃO o vê.** 8 em
   `shells/desktop/tests/it/` — falham a **ler ficheiro**, não a resolver símbolo, e só aparecem em
   `--all-targets`. *Corra `--all-targets` antes de acreditar que moveu bem.*
2. ⛔ **O gate de TEXTO cujo SUJEITO muda de metade.** `the_gap_reach_is_zoom_invariant` procura
   `let reach =` dentro de `gap_live.rs` — e essa linha vive no `flip_gap_helpers_tick`, que ficou
   na shell. As três asserções do `the_smoke_scene_arms_…` estão **todas** no armar. Os quatro
   sujeitos foram **medidos um a um** (`grep -rl` pelas strings asseridas) em vez de adivinhados, e
   cada gate foi **corrido**: `1 passaram`, não zero. ⚠️ *Um filtro que casa nada imprime verde* —
   a primeira corrida com `'a | b | c'` devolveu **`0 passaram`** e lia-se como sucesso.
3. ⛔ **Um tecto de LOC estoura sem uma linha de lógica nova.** `flip/draw.rs` estava
   **exactamente** em 600; reapontar `crate::flip::X` → `ph2d_app_flip::X` alongou linhas, o
   `rustfmt` reexpandiu-as e ele foi a **605**. Curado por corte (424 + 198), ⛔ nunca por entrada
   nova no `FILE_OVERAGE_OK`. *O `cargo check` não o vê; o `collision-surface.sh` vê.*

## 7 — A prova (§3 do briefing)

**(a) Nenhum teste se perde** — `cargo nextest list --workspace --cargo-profile ci-test`, antes e
depois, por `scripts/nextest-list-diff.py`:

```
antes: 22635 testes (22011 chaves) | depois: 22635 (22011)
MOVED (mesma chave, outro pacote/binário): 49   → ph2d-app-flip
ONLY-A (perdidos): 0          ✅
ONLY-B (novos):    0
```

**(b) Nenhum gatilho de smoke se perde** — os **24** `PH2D_*` da família, antes e depois,
`diff` **vazio**.

⚠️ **A poda do A1 deu ZERO, e é uma medição, não uma omissão.** ⭐ **Nenhum smoke desta família tem
NÍVEL**: os 19 são booleanos (`env::var(...).is_some()`), não `=<n>` — a unidade de poda é o nome
da env var, não um número. Os **23** gatilhos foram passados um a um contra `docs/`, `CLAUDE.md`,
`project-memory/` e `.claude/` (fora de `docs/archive`): **todos citados**, o mais fraco com 3
docs, e as citações verificadas com contexto são instruções de smoke reais (passos, roteiro), não
menções de passagem. ⇒ **zero cenas apagáveis.** E o Flip **não tem** gate
`no_two_*_scenes_claim_the_same_level`, porque não tem cena numerada.

**(c) A shell encolheu** — tabela do §5. ⛔ **A medição de RELÓGIO não foi feita e a razão é
honesta:** as seis linhas da W2 abriram no mesmo dia e a máquina esteve entre `load 22` e
`load 135` a jornada inteira. *Nenhuma leitura de relógio desta workstation vale acima de `load ~5`*
(CLAUDE.md §5) — um `--timings` a frio tirado hoje mediria a contenção das outras cinco linhas, não
a unidade da shell. **Fica para a Fase B, com o `loadavg` impresso ao lado.**

**(d) Gate de fecho** — §8.

**(e) Smoke** — §9.

## 8 — Foundational tocado, e símbolos novos

**Foundational:** `shells/desktop/src/app_state.rs` (os 21 campos saem, 1 entra),
`shells/desktop/src/main.rs` (49 `mod` → 1; o construtor: 21 linhas → 1),
`shells/desktop/Cargo.toml` (uma dep nova). ⚠️ Os três conflitam **textualmente** com as outras
cinco linhas da W2 — todas por ADIÇÃO/REMOÇÃO em regiões distintas; o Mergiraf funde, o olho
confere.

**Símbolos novos (para o integrador grepar):**

| símbolo | valor | onde |
|---|---|---|
| crate | `ph2d-app-flip` | `crates/` (glob — **zero** edição no `Cargo.toml` da raiz) |
| campo de `App` | `flip_state: crate::flip_state::FlipState` | `app_state.rs` |
| módulo | `mod flip;` | `main.rs` |
| deps novas da crate nova | `ph2d-core`, `ph2d-flip`, `ph2d-flip-fill`, `ph2d-flip-reshape`, `ph2d-painter-effects`, `ph2d-tool-flip`, `ph2d-editor`, `winit`, `libm` | todas **internas** menos `winit`/`libm`, que a shell já usa |

⛔ **Nenhum id, const, variant, token ou número de schema novo.** Nada a colidir.

## 9 — O que smoke-testar (nada mudou de produto — é isso que se confirma)

Esta é uma extracção: o comportamento tem de ser **idêntico**. Os dois que exercitam os caminhos
mais tocados (o `FlipState` e as metades partidas):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-flip && env PH2D_FLIP_HARDNESS_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-flip && env PH2D_FLIP_COLORIZE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

## 10 — O que só o `ship.sh` apanha

`fmt` e `clippy --all-targets` correm aqui (§ abaixo); **não** correram: `machete` (há uma crate
nova com 9 deps — é o candidato mais provável a um `✗`), `deny`, `audit`, `typos`, e o `doc-index`
sobre o `docs/Flip/handoffs/README.md`, que ganha esta entrada.

## 11 — Fase B (o que fica, e a ordem)

1. `git rebase main` depois de a L0 integrar.
2. Ler o `HOWTO_partir_uma_familia_da_shell.md` inteiro.
3. **Mover `src/flip/` inteira** — é por isso que a Fase A agrupou.
4. Os **27 módulos presos** saem no momento em que o §3 se resolver; sem isso, o corte pára em
   3 620 LOC dos 18 798 e a Fase B entrega um quinto do que podia.
5. Fica **NOMEADO e não tocado**: os 15 ficheiros `render_loop/flip_*` (3 748 LOC) — são o passe de
   render, precisam do `AppGfx`/wgpu/laço de desenho, e saem quando o substrato os cobrir.
