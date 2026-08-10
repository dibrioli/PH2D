# Handoff de INTEGRAÇÃO — `line/gpu-nodes` (DIRETRIZ §1.5.9)

> **A linha está FECHADA.** Tudo smokado e aprovado pelo Enio. A linha **NÃO integrou e NÃO
> pushou** — espera ordem explícita (§0.7).
>
> O estado técnico completo (o *porquê* de cada decisão, as medições, as armadilhas) vive em
> [`HANDOFF_line_gpu_nodes_continuacao_2026-07-20.md`](HANDOFF_line_gpu_nodes_continuacao_2026-07-20.md)
> — **esse é o documento que o próximo DONO lê**. Este aqui é o que o **integrador** lê:
> superfície tocada, colisões prováveis, ordem, e o que só o ship pega.

---

## 1. Identidade

| | |
|---|---|
| Branch | `line/gpu-nodes` |
| HEAD | `6db7727d` |
| Base (merge-base com `main`) | `5cc54941` |
| Commits | **33** |
| Worktree | `Worktrees/line-gpu-nodes` |

**Cinco corpos de trabalho, cronológicos e dependentes** (não reordene — cada um constrói sobre
a infra do anterior):

1. **`bc4d04e6` … `168bcc7e`** — ADR-0140: a **grade espacial** na GPU (scan reusável → grade →
   boids → collide) + as cenas `=7`/`=8`/`=9`.
2. **`df10ab60` … `cf92e934`** — ADR-0135: a família **`sim.zone`** (o contêiner de laço é um
   passthrough condicional; o recuo parcial) + a cena `=10`.
3. **`6bffcf6b` … `37d1fdad`** — a **GRANDE AUDITORIA** do Motion Nodes (broadcast misto passa a
   RECUSAR; a starvation dos rings medida; 3 varreduras de gate).
4. **`24aa798f` … `2f1691d7`** — a fila §E da auditoria: ADR-0136 (**a família que muda
   contagem**), ADR-0137 (**reforma dos rings**), ADR-0138 (**colunas `Arc`**), os dois **tetos**
   medidos, o **cull do boids**.
5. **`e1e7852d` … `6db7727d`** — ADR-0139: o **voronoi via JFA** + a cena `=11` + a medição que
   **refutou** o item seguinte da fila e o cap do soft-body que caiu com ela.

---

## 2. Foundational / compartilhado tocado

| Arquivo / área | O quê | Aditivo? |
|---|---|---|
| `crates/ph2d-nodegraph/src/gpu.rs` | **4 canais novos** no `KernelResolver`, todos com default `None`: `grid`, `state_select`, `stream_op`, `algorithm` | **Sim** (default = o comportamento de hoje) |
| `crates/ph2d-nodegraph/src/stream_op_meta.rs`, `algorithm_meta.rs` | **arquivos novos** (`StreamOp`, `GpuAlgorithm`) | **Sim** |
| `crates/ph2d-nodegraph/src/lib.rs` | 2 linhas de `pub mod` | **Sim** |
| `crates/ph2d-nodegraph/src/cook.rs` | `CountLawCtx.dt` (campo novo) | ⚠️ **Não** — ver §3.1 |
| `crates/ph2d-nodegraph/src/attr.rs` | `Stream.attrs` passa a `Arc<Column>` (ADR-0138) + `approx_bytes` | ⚠️ **API estável, representação NÃO** — ver §3.2 |
| `crates/ph2d-node-registry/src/lib.rs` | 4 `register_*` novos + 4 impls de resolver | **Sim** |
| `crates/ph2d-eval-motion/src/checkpoint.rs` | reforma do ring (ADR-0137) + `set_ring_budget` | **Sim** (comportamento muda, superfície cresce) |
| `crates/ph2d-gpu-cook/**` | a maior parte da linha (15 fontes + 12 suites) | crate desta linha |
| `shells/desktop/src/motion_state*.rs` | 4 cenas de smoke novas (`=7`..`=11`) + os gates delas | **Sim** |
| `shells/desktop/src/render_loop/motion_bridge_gpu.rs` | a ponte que dirige o cook | modificado |
| `.typos.toml` | 4 palavras pt-BR + regex de hash abreviado | **Sim** — ver §5 |

**15 crates de nó tocadas**, todas do mesmo jeito: `register_gpu_kernel` (+ eventualmente
`register_grid`/`register_stream_op`/`register_gpu_algorithm`) no `register`, mais o WGSL num
módulo irmão. **Nenhuma mudou o `eval` da CPU** — que continua canônico.

**`Cargo.toml` tocado: UM** (`ph2d-gpu-cook`, só dev-dependencies de nós para as suites de
paridade) + `Cargo.lock`. Superfície nova para `deny`/`audit`: **zero** (nenhuma dep externa
nova; `naga` já era dev-dep).

---

## 3. Símbolos novos e as DUAS quebras (o grep de mesmo-símbolo, §1.5.5)

Exports novos em `ph2d-nodegraph::gpu`: `GridSpec`, `StateSelect`, `StreamOp`, `GpuAlgorithm`,
`KEEP_FLAG_COL`, `ROWS_COL`, `SourceWindow`.
Em `ph2d-gpu-cook`: `voronoi`, `grid`, `scan`, `error`, `stream_op`, `GpuCookError`,
`GpuCheckpointRing`, `INT_CENTROID_RES_CEILING`.

### ⚠️ 3.1 — `CountLawCtx.dt` é campo NOVO num struct construído por chamadores

Toda lei de contagem (`sim.spawn`) constrói um `CountLawCtx`. Quem tiver um construtor desse
tipo numa linha paralela **não compila** depois do merge textual (foi exatamente o erro que esta
linha levou dentro de casa, no teste do emitter). É `dt: f64`, espelho do `EvalCtx::dt`.

### ⚠️ 3.2 — `Stream.attrs` virou `Arc<Column>` (ADR-0138) — API estável, representação não

`get`/`columns` continuam devolvendo `&Column` (via `Arc::as_ref`), então **o código que só lê
não muda**. Quebra quem: (a) construir `attrs` diretamente (é privado, mas o `set` agora embrulha
em `Arc`); (b) esperar que `clone()` de um `Stream` **copie** — agora é refcount. A soundness
apoia-se num fato verificável: **não existe mutação in-place de `Column`** (não há `get_mut`).
Uma linha paralela que ADICIONE um `get_mut` quebra o ADR-0138 em silêncio.

### ⚠️ 3.3 — O que NÃO foi tocado

**Contratos congelados (§6): nenhum encostado.** `NodeOp = 2` / `OpResolver = 1` /
`NodeManifest = 8` intactos — e isso é a espinha do desenho: **todo canal novo é side-metadata no
registry**, exatamente como o `register_ui`. Gate `architecture_contract_surface` verde.
`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` intactos. Superfície vetorial intacta.

**`PROJECT_SCHEMA` / `DOC_VERSION`: intocados.** Nada desta linha viaja em arquivo — kernels são
`'static`, o estado de sim vive em buffers de device, e as cenas de smoke são env-gated.

---

## 4. Sobreposição com outras linhas (a ordem se MEDE)

O footprint fora de `ph2d-gpu-cook` é: **`ph2d-nodegraph` (7 arquivos, quase tudo aditivo)**,
`ph2d-node-registry` (aditivo), `ph2d-eval-motion` (o ring), 15 crates de nó (só o `register`) e
`shells/desktop/src/motion_state*` (arquivos novos + o `match` das cenas).

- **Contra uma linha de Motion Nodes / fan-out de nós:** colisão provável no `register` de cada
  nó (uma linha de fan-out adiciona nós; esta adiciona linhas DENTRO do `register` dos que já
  existem). Fusão textual trivial.
- **Contra uma linha que mexa em `Stream`:** §3.2 é a colisão séria — **esta deve integrar
  ANTES**, porque muda a representação e a outra linha compila contra a nova.
- **Contra a timeline / anim:** zero sobreposição (nenhum arquivo comum).
- **`motion_state.rs`:** o `match` de `PH2D_GPU_COOK_DEMO` vai até `11`. Outra linha que
  acrescente uma cena colide no braço — só ADICIONE, não renumere.

---

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

Rodado **nesta árvore**, verde:

- `cargo fmt --all --check` — **limpo** (⚠️ um commit desta linha existe só por causa dele:
  `cargo check -p` não vê ordem de `pub mod`).
- `cargo clippy --all-targets` nas crates tocadas — **0 warnings**.
- `cargo check --workspace` — 0 erros.
- **`cargo test --workspace`: 8062 testes em 743 suites, 0 falhas** (exit code conferido sem
  pipe — [[feedback_pipe_masks_script_exit_code]]); **as suites de dispositivo rodam na RTX** com
  `--release -- --ignored` (elas são `#[ignore]` de propósito: um CI sem adaptador as pula).
- Gate de LOC verde (dois splits foram feitos por causa dele: `error.rs`, `lifecycle.rs`).

⚠️ **`.typos.toml` foi ALTERADO por esta linha e a razão importa:** o `typos` do ship já estava
**vermelho na `main`** por 4 palavras pt-BR e por um **hash de commit abreviado** citado num
handoff (o `typos` tokeniza o hex e reclama do pedaço do meio). Entrou o regex
`\b[0-9a-f]{8,40}\b` em `extend-ignore-identifiers-re` + 4 palavras pt-BR.
Se outra linha também mexeu no arquivo, **funda as duas listas** — chave duplicada mata o gate no
parse ([[feedback_duplicate_allowlist_key_kills_the_gate_at_parse]]).

Fica para o ship: `machete`/`deny`/`audit` (superfície nova zero, mas o ship varre o workspace),
clippy latente do resto da árvore, e o drift pré-fork
([[project_integration_prefork_lines_ship_drift]]).

⚠️ **Rode `cargo check --workspace` depois do merge textual** — as quebras da §3.1/§3.2 são
exatamente a classe que passa por um merge limpo ([[feedback_clean_text_merge_can_be_semantically_broken]]).

---

## 6. Smoke — FEITO e aprovado pelo Enio

| Cena | O quê | Status |
|---|---|---|
| `PH2D_GPU_COOK_DEMO=7` | a murmuração (boids na grade) | aprovado |
| `=8` | o empacotamento que respira (collide iterado) | aprovado |
| `=9` | a varredura diagnóstica | aprovado |
| `=10` | o globo de neve (a família `sim.zone`) | aprovado |
| `=11` | **o favo que respira** (voronoi/JFA) | aprovado (2026-07-21) |

Rodar: `env PH2D_GPU_COOK_DEMO=<n> cargo run -p ph2d-host-desktop --release`.
O caminho GPU é **ON por default**; `PH2D_GPU_COOK=0` volta pra CPU (útil pra bissecar).

---

## 7. O que ficou ABERTO (nomeado, não escondido)

- **A fila acabou, e o último item se dissolveu na medição.** O §2 do handoff de continuação
  listava *"soft_body/verlet (XPBD)"*; medindo, (a) o `soft_body` **não é XPBD** — é shape
  matching, um *reduce→broadcast→map*, sem cadeia sequencial; e (b) o cap dele **não era de
  custo** (1600 partículas = 0,005 ms; 1 milhão = 7,0 ms). O cap foi corrigido pela medição
  (40→512 por lado, recurso = HR-4 física soft 2,0 ms) e **o port pra GPU não se justifica**.
  Quem é sequencial de verdade é o `verlet_rope` (Gauss-Seidel por aresta) — e ele **não tem
  cap**, então também não é o §0.0.
- **O censo de cobertura dá frontier VAZIA** (`motion_gpu_coverage.rs`): não há nó de documento
  real ainda no prefixo CPU que valha um kernel.
- **Cache por-params do cook do voronoi** — `relax` animado re-cozinha a relaxação inteira todo
  frame (3,02 ms a 20k; era o desenho, ADR-0139 §3). Vira item quando alguém quiser 165k **e**
  um relax animado ao mesmo tempo.
- **`seek`/pitch num stream, toggle "Streamed"** — nada disto é desta linha (é do áudio); citado
  só para o integrador não confundir os handoffs.

---

## 8. Detalhe técnico

Tudo — os ADRs 0135–0140, as medições, as armadilhas de fixture, as otimizações **medidas e
reprovadas** (não re-derive) — está em
[`HANDOFF_line_gpu_nodes_continuacao_2026-07-20.md`](HANDOFF_line_gpu_nodes_continuacao_2026-07-20.md)
e nos próprios ADRs. O relatório da auditoria que abriu a fila §E vive em
[`HANDOFF_line_gpu_nodes_auditoria_RESULTADO_2026-07-20.md`](HANDOFF_line_gpu_nodes_auditoria_RESULTADO_2026-07-20.md).
