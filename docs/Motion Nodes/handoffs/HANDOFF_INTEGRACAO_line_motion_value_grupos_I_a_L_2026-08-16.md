# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, grupos I · J · K · L (2026-08-16)

> **Estado:** linha FECHADA, gate completo VERDE, **aguardando ordem de integração do Enio.**
> A linha não integra e não pusha (CLAUDE.md §0.7). Este documento é o que o **agente integrador** precisa.

## §1 — O que entra

**41 commits**, quatro grupos da segunda volta da
[conferência dos nós](../89_plano_conferencia_dos_nos.md), cada um com a sua cena de smoke
(ordem do Enio: *"a cada grupo uma cena de smoke"*).

| grupo | o que fecha | cena | smoke |
|---|---|---|---|
| **I** | a VIZINHANÇA vira um número (`motion.proximity`) — e *Scale*/*Hide* saem por COMPOSIÇÃO | `=49` | ✅ |
| **J** | o PINO alcança as três simulações (`inv_mass` pela cadeia de estado) | `=50` | ✅ |
| **J′** | o report do smoke: a prescrição do corpo mole + o espaço pessoal do bando | `=51` | ✅ |
| **K** | o peso por partícula (`soft_body`) + os SUB-PASSOS (`verlet_rope`) | `=52` | ✅ |
| **L** | o TETO DA TAXA (`motion.delay`: `max_step` + `max_accel`) | `=53` | ✅ |
| **M** | a CONTAGEM da conferência deixa de ser escrita à mão (sem código de produto) | — | — |
| **N** | o `motion.wiggle` ganha as OITAVAS, o multiplicador e o LAÇO | `=54` | ✅ |
| **O** | o `motion.oscillator` ganha o PULSE WIDTH e o `motion.stagger` o OFFSET | `=55` | ✅ |
| **P** | o `motion.drive` escreve uma COLUNA NOMEADA — a **§10.0 do plano FECHA** | `=56` | ⏳ |

⚠️ **A cena `=56` (grupo P) NÃO foi smokada** — *integrar não é aprovar*.

✅ **As sete anteriores foram smokadas e aprovadas pelo Enio** (`=49`..`=55`).

✅ **As cinco primeiras foram smokadas e aprovadas pelo Enio** — as três primeiras à medida
que fecharam, e as duas últimas (`=49` e `=53`) em 2026-08-16, depois do fechamento da
linha. ⚠️ **O que a aprovação NÃO cobre está nomeado na §6:** o gate `#[ignore]` da `=53`
(o teto sobe a `0,1678` no tique da inversão) segue **aberto** — um smoke aprova o que o
olho vê, e aquele número é menor que a espessura de um traço na tela.

O mecanismo de cada grupo está na **§5 do `CLAUDE.md`**, escrito no commit de cada um —
não foi copiado para cá.

## §2 — Superfície de colisão (MEDIDA na worktree, não auto-relatada)

| eixo | valor | como foi medido |
|---|---|---|
| `PROJECT_SCHEMA` | **84 INTOCADO** | `git diff main...HEAD -- project.rs project_schema.rs` → **vazio** |
| contrato congelado | **intacto** | `git diff` vazio em `ph2d-nodegraph/src/node.rs` e `ph2d-core/src/tool.rs` |
| ADR | **nenhum novo** | ⇒ a linha fica **FORA de toda disputa de número** |
| registro do `ph2d-ecs` | **intocado** ⇒ os **três** espelhos também | |
| `ph2d-i18n` | **intocado** ⇒ a cadeia `vector::tr(k).or_else(sculpt3d::tr)` fica de pé | |
| crates novas | **1** (`ph2d-node-motion-proximity`, folha drop-in, glob member) | |
| pacotes externos novos | **ZERO** — o único `+name` do `Cargo.lock` é a própria crate | |
| `Cargo.toml` | **3** (a nova + o glob do registry + `[dev-dependencies]` do `ph2d-gpu-cook`) | |
| cenas de smoke | **1..53 contínuas** — ⚠️ **próxima livre: 56** | conte no `match` do roteador |

⚠️ **A crate-nó entra em `[dev-dependencies]` da `ph2d-gpu-cook`**, nunca em `[dependencies]`:
ela só existe ali para o gate de paridade CPU×GPU e o `src/` não a usa ⇒ **machete-safe**
(o precedente das cinco crates-nó da `line/gpu-nodes`).

## §3 — Ponto de merge sensível: UM

`shells/desktop/src/motion_state_demo_router.rs` cruzou 600 LOC ⇒ as **nove** cenas de grupo
(`=41..=49`) saíram para o irmão `motion_state_demo_conferencia.rs`, **uma função por cena**.

⚠️ **O irmão NÃO tem `match` nenhum, de propósito.** O roteador continua a ser a ÚNICA lista de
níveis, porque dois `match` em dois arquivos deixariam um nível reivindicado duas vezes passar
**em silêncio** (o compilador só vê `unreachable pattern` dentro de um mesmo `match`).
Uma linha que acrescente uma cena tem de escrevê-la no **roteador**.

## §4 — Gate de fechamento (rodado na worktree, tip `acb74d776`)

- `cargo fmt --all -- --check` **EXIT 0**
- `cargo clippy --workspace --all-targets -- -D warnings` **EXIT 0, zero warnings**
- `cargo nextest run --workspace --cargo-profile ci-test` — **16.174 / 16.174 passaram, 1.581 skipped**

⚠️ **O que este gate NÃO alcança, e o integrador tem de rodar:**
os **`--ignored`** (gates de GPU na RTX, kills de relógio) e a árvore **combinada**.
*Skip gracioso não é verde.*

⚠️ **Kills de relógio exigem `--test-threads=1` com a máquina calma** — precedente medido
neste repo: o mesmo binário mede **11,36 ms** sob `load 41` e **5,50 ms** sob `load 0,6`.

## §5 — Mudanças de comportamento (nomeadas)


⚠️ **UMA MUDANÇA DE COMPORTAMENTO NO DEVICE, e ela é um CONSERTO:** até o grupo O, os
variants de WGSL do `motion.oscillator` para `rot` e `size` **ignoravam `time_mode`/`bpm`**
(só o de `P` os lia). Um grafo a dirigir a **rotação ou o tamanho em BPM** corria a uma taxa
na CPU e a outra na GPU — sem erro e sem aviso. Agora as três rotas concordam, e o gate
`the_bpm_ruler_reaches_every_oscillator_channel` (que nasceu VERMELHO) as pina.
1. **`motion.collide` lê `size` e `falloff`** (grupo H, já integrado) — contexto para as de baixo.
2. **`motion.verlet_rope` / `soft_body` / `boids` honram `inv_mass`** (grupo J) — um grafo com
   `motion.pin_constraint` no laço passa a segurar o que antes ignorava.
3. **Um `soft_body` de massa infinita SEGUE a prescrição** (`anchor + rest[i]`), em vez de
   congelar no lugar — mover `spacing`/`rows`/`cols`/âncora com a sim rodando agora move o pino.
4. **O `boids` ganhou `separation_radius`** (default `0,0` = byte-idêntico) e a GPU **RECUSA o
   device** quando ele passa a percepção (a grade é construída com `cell_param: "radius"`).
5. **`motion.delay` ganhou dois tetos** (default `0,0` ⇒ o passe nem corre; byte-idêntico
   **no valor E no objeto cozido**).

## §6 — Aberto, com o preço ao lado

- ⚠️ **Um gate `#[ignore]` NOVO, com o número e o mecanismo escritos:**
  `the_ceiling_is_honoured_on_every_tick_including_the_turn` (cena `=53`) — o teto vale **ao
  dígito na rampa** (`0,0800`) e sobe a **`0,1678` no tique 70**, a inversão do vaivém. A lei do
  kernel não pode produzir isso (ela clampa `|out − prev|` por construção, e cinco gates de
  unidade sangram sob mutação) ⇒ a diferença mora **entre o kernel e o que a cena monta**, com o
  `prev_out` do gather como candidato nomeado. **Não afrouxe a barra** — o precedente é o par
  `watercolor_app_params_incremental` do Painter.
- A folha 03 tem **6 P1**; a folha 07 tem **3**.
- ⚠️ **A primeira coisa de toda wave desta conferência é MEDIR se a composição já exprime o item** —
  quatro células envelheceram antes de alguém voltar a elas (o `max_force`, o *wander*, a
  idade normalizada, e o `motion.lag` inteiro).

## §7 — Smokes

```
env PH2D_GPU_COOK_DEMO=49 cargo run -p ph2d-host-desktop --release   # a vizinhança
env PH2D_GPU_COOK_DEMO=53 cargo run -p ph2d-host-desktop --release   # o teto da taxa
```

⚠️ Toda cena **imprime as bandas nomeadas** — *se a lista não aparecer, PARE*.
A `=53` **exige PLAY** (a pergunta é *quão DEPRESSA*, e uma foto não a responde).
As cenas `=41..=48` e `=50..=52` **têm de continuar iguais**.
