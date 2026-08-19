# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, **FECHO DA LINHA** (2026-08-18)

> ⚠️ **Este é o documento do integrador.** Ele consolida as **quatro** waves que a linha
> fechou e traz os oito itens da DIRETRIZ §1.5.9, **medidos hoje**. O *mecanismo* de cada
> wave fica nos handoffs por-wave (§9) — aqui está só o que evita conflito e regressão.
>
> ⚠️ **A linha NÃO integra e NÃO pusha** (CLAUDE.md §0.7). Entrega isto e espera ordem
> explícita do Enio.

---

## §1 — Identidade (§1.5.9 item 1)

| | |
|---|---|
| branch | `line/motion-value` |
| HEAD | **este handoff É o último commit da linha** — `git rev-parse HEAD` (⚠️ não se pina aqui: o commit que escreve o sha muda o sha, que é a mesma doença da caixa logo abaixo) |
| merge-base com `main` | `adc2e3963` |
| commits da linha | **26** · **59 arquivos** |
| `main` hoje | `1338d40b4` — ⚠️ **11 commits à frente do fork** |

⚠️ **O fork NÃO é mais fast-forward, e isto é uma correção a este próprio handoff.** O da
wave do `motion.wave` escreveu *"`main` está a ZERO commits do fork ⇒ merge fast-forward
trivial **hoje**. Esta caixa **ENVELHECE**: reconfira antes de integrar"* — e envelheceu
exactamente como avisou. **Reconfira de novo antes de rodar o gate**; esta caixa tem a mesma
data de validade que aquela.

### O que a `main` andou, e onde ela toca esta linha

Os 11 commits são **docs e processo** (`.claude/commands/*`, `SKILL_Stack`, `backups/`,
`.gitignore`, `scripts/` novos) mais três de gate/perf. **A intersecção de arquivos com esta
linha é UM:**

```
CLAUDE.md
```

⇒ **é o único conflito de merge esperado.** Nenhum `.rs` colide.

⚠️ E a `main` trouxe duas coisas que o integrador deve USAR:
- **`scripts/collision-surface.sh`** (commit `b06f08b37`) — a superfície de colisão numa
  chamada. A tabela do §3 abaixo foi produzida por ele; **re-rode-o depois do rebase**, que é
  quando os números da base mudam.
- **`crates/ph2d-editor-core/tests/architecture_docs_paths_and_smokes_resolve.rs`** (commit
  `528cadb7b`) — um gate novo sobre o cânone instrucional. Esta linha edita o `CLAUDE.md`, logo
  **entra no escopo dele**; os caminhos e smokes que a linha cita foram conferidos contra o
  disco (§5).

---

## §2 — Foundational / compartilhado tocado (§1.5.9 item 2)

| arquivo | o quê | aditivo? |
|---|---|---|
| **`crates/ph2d-nodegraph/src/time.rs`** | 6º `TimeMode::Curve` + `TIME_CURVE_SAMPLES` + o campo `TimeMap.curve` + a lei `rem_euclid` | **SIM** — variant e campo APENDADOS; `TimeMap::default()` é o `Scale` de sempre |
| `shells/desktop/src/motion_state.rs` | 2 `mod` novos (cenas `=58`/`=59`) | SIM (lista) |
| `shells/desktop/src/motion_state_demo_router.rs` | 2 braços de `match` (`=58`/`=59`) | SIM (lista ordenada) |
| `shells/desktop/src/render_loop/motion_bridge_params_channel.rs` | os 3 canais de tamanho num braço de unidade só | não-aditivo, **1 braço** |
| `shells/desktop/src/motion_state_demo_conferencia*.rs` | narração das cenas | SIM |
| `crates/ph2d-gpu-cook/tests/gpu_cpu_parity.rs` | 3 gates de paridade novos | SIM |
| `docs/Motion Nodes/ferramentas/placar_conferencia.py` | a chave da tabela `HAND` passou de nº de linha para TRECHO, + verificação | não-aditivo, **ferramenta de doc** |
| `CLAUDE.md` | §5 do Motion: 1 linha de estado + 2 de *Aberto* | ⚠️ **colide com a `main`** |

⚠️ **`ph2d-nodegraph` é foundational e foi tocado por desenho** (ADR-0107 o permite no Modo L).
O ponto de extensão usado é **append-only**: `TimeMode` ganhou o índice **5** no fim e `TimeMap`
ganhou um campo com `Default`. Uma linha paralela que apende **outro** variant a `TimeMode`
colide em mesmo-símbolo — ver §3.

---

## §3 — Símbolos que podem COLIDIR (§1.5.9 item 3)

**Medido por `scripts/collision-surface.sh main`** (a versão da `main`, rodada contra esta
árvore):

| chave | esta linha | base | veredito |
|---|---|---|---|
| `PROJECT_SCHEMA` | **84** | 84 | intocado (tripla `(84, 13, 14)` idem) |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `DOC_VERSION` | 14 · 13 · 18 | idem | intocados |
| registro `ph2d-ecs` (+2 espelhos) | 57 / 58 / 58 | idem | intocados |
| contrato congelado (§6) | — | — | **INTOCADO** nos dois arquivos |
| ADR | **nenhum criado** | último 0159 | fora de toda disputa de número |
| `Cargo.lock` | **nenhum pacote EXTERNO novo** | — | só arestas internas (§5) |
| marcadores de conflito | nenhum | — | ✓ |

### Os literais que esta linha ESCREVEU — o que outra linha grepa

| símbolo | valor | onde | risco |
|---|---|---|---|
| `TimeMode::Curve` | **5** | `ph2d-nodegraph/src/time.rs` | ⚠️ **APENDADO** — outra linha que apende a este enum escreve o MESMO 5 e a colisão **passa muda** |
| `TIME_CURVE_SAMPLES` | **128** | idem | novo; entra na chave do memo |
| `CH_SIZE_X` / `CH_SIZE_Y` | **10** / **11** | `motion-drive/src/channel.rs` | ⚠️ apendados ao enum de canal; o documento GUARDA o índice |
| `MODE_LABELS` do `time_remap` | 6 rótulos (índice 5 = `Curve`) | `motion-time-remap/src/lib.rs` | espelho do enum acima — **contam-se juntos** |
| porta de entrada **1** (`time`) | em `oscillator` · `noise` · `wiggle` | os 3 `MANIFEST` | ⚠️ apendada; aresta salva guarda o ÍNDICE da porta, e a porta 0 não se moveu |
| `READ_CHANNELS` do `value.attribute` | **4 entradas novas no TOPO** | `value-attribute/src/lib.rs` | ⚠️ a ORDEM mudou — ver a nota abaixo |
| cenas de smoke | **`=58`** e **`=59`** | `motion_state_demo_router.rs` | ⚠️ próxima livre = **60**, e **conta-se lendo o `match`**, nunca esta nota |
| `PH2D_MOTION_NODE_PATH_SMOKE=2` | um **modo** de env que já existia | `motion_node_path_smoke.rs` | não é nível novo do roteador |

⚠️ **A reordenação do `READ_CHANNELS` é segura, e foi MEDIDA — não presumida.** O índice do
chip **não é persistido**: o painel escreve a coluna (`attr`, texto) e o `mode` (f32), e a
selecção é **derivada** a cada quadro
(`channels.iter().position(|c| c.column == attr && c.mode == mode)`, em
`motion_bridge_params.rs`). Um documento que guardou `attr="vel", mode=1` continua a resolver
para `Speed`, onde quer que o chip agora se sente.

⚠️ **Rótulos de UI novos** (`Position X` · `Position Y` · `Radius` · `Angle` · `Curve`) são
literais na mesma tabela dos vizinhos (`Speed`, `Falloff`, …) — é a convenção estabelecida
destes registros, não uma string solta a escapar do HR-15.

### O ponto de merge sensível

**`CLAUDE.md`** — o único arquivo que as duas árvores tocaram. A `main` **compactou o §5**
(917 KB → 41 KB, commit `658494e60`) e esta linha reescreveu **três linhas** do bullet do
Motion. ⚠️ Resolva pelos **estágios do índice**, não pelos marcadores, e **fique com a forma
compactada da `main`**, aplicando por cima só: (a) a linha de estado da porta de tempo, (b) as
duas entradas novas de *Aberto*, (c) o `PH2D_MOTION_NODE_PATH_SMOKE=1|2` na lista de smokes.

⚠️ **Um caso concreto de «fique com a `main`», medido:** a linha 153 desta árvore (o bullet da
**Timeline**) cita `PH2D_MOTION_PATH_SMOKE`, que **não existe em `.rs` nenhum** — o nome real
é `PH2D_PATH_SMOKE`. A `main` **já o corrigiu** (`grep -c` na dela dá **0**), e esta linha não
toca aquela linha. Se a resolução do conflito ressuscitar o nome morto, o gate novo
`architecture_docs_paths_and_smokes_resolve` reprova — e com razão: *um smoke morto ensina que
o produto está partido*.

---

## §4 — Contratos congelados (§1.5.9 item 4)

**NENHUM.** `git diff` **vazio** em `crates/ph2d-nodegraph/src/node.rs` e
`crates/ph2d-editor-core/src/tool.rs`; o gate `architecture_contract_surface` passa.
Todo canal novo desta linha é **side-metadata do registry** (`ReadChannel`, `ParamUiHint`,
`ParamGate`, `GpuKernel`), que é exactamente o ponto de extensão que o §6 deixa aberto.

---

## §5 — O que só o `ship.sh` pega (§1.5.9 item 5)

- **Deps novas: duas arestas INTERNAS**, nenhuma externa —
  `ph2d-node-motion-time-remap → ph2d-curve` e `shells/desktop → ph2d-curve`.
  `Cargo.lock` cresce **2 linhas** e **nenhum `+name`** aparece ⇒ `cargo deny` e `cargo audit`
  não têm superfície nova. ⚠️ **`cargo machete`** é quem pode reclamar: as duas são **usadas**
  (o `time_remap` serializa/parseia a curva; a cena `=58` a autora pela MESMA porta), mas é o
  gate que decide.
- **fmt / typos:** `cargo fmt --all -- --check` **EXIT 0** nesta árvore. Fmt-skew pré-fork é
  possível depois do rebase — o `ship.sh` é quem vê.
- **clippy:** `--all-targets --workspace` **zero** nesta árvore.
- **O gate de docs NOVO da `main`** (`architecture_docs_paths_and_smokes_resolve`) entra em
  cena porque esta linha edita o `CLAUDE.md`. Os caminhos e o smoke que a linha acrescenta
  foram conferidos contra o disco; **re-rode-o depois do rebase**.
- **LOC:** ⚠️ `crates/ph2d-gpu-cook/tests/gpu_cpu_parity.rs` mede **6 247** linhas, e o
  `collision-surface.sh` o assinala. Ele está **fora do gate por desenho** (o
  `workspace_src_files_under_loc_cap` exclui `**/tests/**`) e era **6 120** no fork — esta
  linha acrescentou 127. Não é um bloqueio; é um número que alguém vai querer atacar.

---

## §6 — Ordem, e o que SMOKAR (§1.5.9 item 6)

**Ordem entre commits:** nenhuma dependência escondida — a sequência linear dos 25 commits é a
ordem correcta, e o `--ff-only` depois do rebase basta.

**Smokes desta linha, e o estado de cada um:**

| smoke | wave | estado |
|---|---|---|
| `PH2D_GPU_COOK_DEMO=57` | `motion.wave`, N produtores | ✅ **aprovado pelo Enio** |
| `PH2D_MOTION_NODE_PATH_SMOKE=1` e `=2` | espaçamento | ✅ **aprovado pelo Enio** |
| `PH2D_GPU_COOK_DEMO=58` | dois eixos + relógio curvado | ⚠️ **reprovado, corrigido, AGUARDA re-smoke** — a banda 4 congelava depois de 6 s; a cura está no §9-bis do handoff do grupo Q |
| `PH2D_GPU_COOK_DEMO=59` | a porta de tempo | ⏳ **NUNCA smokado** |

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && \
  env PH2D_GPU_COOK_DEMO=59 cargo run -p ph2d-host-desktop --release
```

⚠️ **Os dois que faltam são do Enio, não do integrador** — integrar não é aprovar (CLAUDE.md
§5.0). O passo-a-passo de cada um está no §8 do handoff do grupo Q.

---

## §7 — O gate de fechamento, rodado

- `cargo fmt --all -- --check` **EXIT 0**
- `cargo clippy --all-targets --workspace` **zero**
- `cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast`:
  **16 380 testes, 16 380 verdes, EXIT 0**
- Paridade de GPU `--ignored` (adapter **encontrado**, *skip gracioso não é verde*):
  `gpu_cpu_parity` **76/77** · `gpu_cpu_parity_sim` **29/29**
- `python3 docs/Motion Nodes/ferramentas/placar_conferencia.py` **EXIT 0**

### Duas vermelhas que NÃO são desta linha

1. **`value_slope_kernel_matches_the_cpu_on_the_device`** — `#[ignore]`, **PRÉ-EXISTENTE**,
   fora desta linha.
2. **`ph2d-render --bench sprites_upload_144b_vs_72b`** — `assert_eq!(size_of::<RenderInstance>(), 176)`
   mede **184**. Exonerada por três testemunhas (diff vazio na crate · reproduz no `main` ·
   o último a mexer foi `d84f1f003`, de outra linha). ⚠️ É um **bench**, então só
   `--all-targets` o alcança. **A cura é do dono do `ph2d-render`.**

### E uma lição de medição que vale para o próximo integrador

Uma corrida anterior da suíte marcou **duas** falhas —
`ph2d-timeline::the_cost_of_depth_is_linear_not_explosive` e
`ph2d-host-desktop::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`. As duas são
**gates de RAZÃO**, a corrida partilhava a máquina com duas suítes de GPU, e o `load average`
estava em **14,8**. Sozinhas, passam; na corrida limpa acima, passam. O CLAUDE.md §5.0 já diz
que *nenhuma leitura de relógio desta workstation vale nada acima de `load ~5`* — **fica o
registo de que a suíte inteira é um desses relógios.**

---

## §8 — Aberto, com o preço ao lado

- ⛔ **O diagnóstico de nome do `value.attribute` não olha o MODO.** `unresolved_reads` recebe
  só os **nomes** das colunas (`columns::names_at`), então uma coluna `Vec2` digitada à mão no
  campo livre **resolve** para o diagnóstico e **lê zeros** para o cook — sem badge. Os `Vec2`
  que a tabela conhece agora têm chip (é a cura do caso real), mas **o caso geral fica**.
  Preço: o callback e o `ph2d-motion-diagnose` teriam de carregar a dimensão da coluna.
- **P2 medido, folha 15:** as **lanes** de `vel` e `size` não existem no picker (elas têm reach
  polar, então não é o silêncio que o `P` tinha). ⚠️ É uma assimetria que **esta mesma linha**
  criou do outro lado: o `motion.drive` ganhou `Size X`/`Size Y` e o valor não lê de volta por
  eixo. Quem as acrescentar orça a **altura** da row (o picker já paga 5 linhas de chips).
- **A folha 06 fica com 1 P1** — o *transform do CAMPO* do `motion.noise` (rotation/scale do
  espaço do ruído; o `offset` já sai de `motion.move(+d) → noise → motion.move(−d)`).
- ⛔ **O GESTO das três metades P2 da célula 36** é **UI**, não motor (um preset de cadeia, ou
  o picker aprender colunas de ESTADO).
- ⛔ O **bench do `ph2d-render`** (§7) é do dono daquela crate.

---

## §9 — Onde está o MECANISMO (o roteador desta linha)

| wave | handoff | o que ele guarda |
|---|---|---|
| `motion.wave`, N produtores (cena `=57`) | [`…_wave_2026-08-18.md`](HANDOFF_INTEGRACAO_line_motion_value_wave_2026-08-18.md) | a composição do 2º produtor; a célula 35 envelhecida |
| espaçamento (`motion.path`) | [`…_spacing_2026-08-18.md`](HANDOFF_INTEGRACAO_line_motion_value_spacing_2026-08-18.md) | o `spacing` com FLOOR; a célula 46 envelhecida (a 8ª) |
| **grupo Q** + as três waves de hoje | [`…_grupo_Q_2026-08-18.md`](HANDOFF_INTEGRACAO_line_motion_value_grupo_Q_2026-08-18.md) | §1–§8 o grupo Q · **§9-bis** o relógio que expirava · **§9-ter** a porta de tempo · **§9-quater** a posição sem chip + o placar |

⚠️ **A ordem de leitura daquele terceiro NÃO é a numérica** — as `§9-bis`/`-ter`/`-quater`
vêm **antes** do `§9`, porque foram escritas depois. O próprio arquivo avisa no topo.

**Placar da conferência 89 depois de tudo** (DERIVADO, `EXIT 0`):
**P0 = 0 · P0/P1 = 0 · P1 = 61 · P2 = 123 · ✅ 126 · ⛔ 119 · natureza 3**, em 451 linhas.
A folha 06 foi de **7 P1 → 1**; a folha 15 ganhou uma linha (P2 18 → 19).

---

## §10 — Fecho da worktree (§1.5.9 item 7)

`target/*/incremental` **reclamado** depois do gate batched: **32 GB** em `debug/`
(`ci-test/` e `release/` já estavam a zero) ⇒ o `target` desta worktree foi de **126 GB para
99 GB**. Risco zero (o cargo o recria) e sem ship.

⚠️ **32 GB é acima dos ~25 GB que a DIRETRIZ §1.5.9 orça por worktree** — a linha rodou o
`nextest --workspace` **três** vezes, e é isso que engorda. Vale como número medido para quem
orçar o pico de uma jornada de cinco linhas.

---

**Resumo para o Enio:** *Linha `motion-value` pronta (26 commits sobre `adc2e3963` — a
`main` andou 11). Colisão: só o `CLAUDE.md`. Contrato congelado intocado, ADR
nenhum, nenhuma dep externa nova. Suíte 16 380/16 380. Faltam dois smokes teus (`=58` re-smoke
e `=59`). Aguardo ordem de integração.*
