# HANDOFF DE INTEGRAÇÃO — `line/motion-value` (bypass de grupo + seleção de fio)

**Status:** FECHADO 2026-08-01 · no `main` em `c0c1233e0` (o commit que trouxe este arquivo).

**Data:** 2026-08-01 · **Branch:** `line/motion-value` · **HEAD:** `5724bf231` · **Base:** `main` (`3197c5c9e`)
**Ordem do Enio:** *"smoke OK. siga"* (×3, aprovando cada fatia) — todos os smokes aprovados.

> ⚠️ Esta é a **SEGUNDA continuação pós-integração** da `line/motion-value`. A jornada QoL de
> 2026-08-01 (`HANDOFF_INTEGRACAO_line_motion_value_graph_qol_2026-08-01.md`, HEAD `879d9703c`) já
> integrou ao `main`; esta reabertura drenou os **três itens abertos** que aquele handoff deixou na
> seção *Aberto* — e drenou **todos**. São **5 commits novos**, `main..HEAD`.
>
> **Nenhum contrato congelado, nenhum ADR, nenhum toque foundational, nenhum `PROJECT_SCHEMA`.**
> O diff toca **apenas** `ph2d-motion-doc` + `ph2d-panel-motion-graph` + `shells/desktop` ⇒ esta
> linha fica **fora** de qualquer disputa de número/gate da árvore combinada desta janela.

---

## 1. O que a linha entrega (5 commits, `main..HEAD`)

Os três itens que o handoff QoL deixou na seção *Aberto* — e os fecha os três:

| Commit | Fatia | Superfície |
|---|---|---|
| `75ffed08d` | **O fio vira CONJUNTO** — Shift acumula, Delete solta o feixe inteiro | panel `selected_wires` (set, era um único) |
| `bbf4069b5` | **O fio SELECIONADO veste o `Accent`** — distinto do hover | `WireEmphasis {Idle,Hover,Selected}` |
| `7bd019f94` | **O grupo carrega um bit de BYPASS** — schema (peça 1/3) | `MotionDoc.bypassed_subgraphs` + record `yg` |
| `df884bffd` | **O cook RE-LIGA a fronteira** de um grupo bypassed (peça 2/3) | shell `motion_bridge_group_bypass::cook_graph` |
| `5724bf231` | **H num card de grupo BYPASSA o grupo como unidade** (peça 3/3) | shell `intents`/`fold` |

**Resultado:** (a) a seleção de fio fecha a família de seleção do editor (nó, backdrop, e agora
**fio** — múltiplos, com Shift/Delete) e ganha realce próprio; (b) **mutar um grupo** deixou de
**expandir** para os membros e passou a ser um **bypass como unidade** — o idioma Blender/Nuke que
você escolheu: `input[0] → output[0]`, o interior pulado, os membros intactos.

---

## 2. A peça central — o BYPASS de grupo como unidade (a cerca de Chesterton derrubada)

O card de grupo nascia **não-mutável de propósito** (*"o Mute expande — é o certo"*), a cerca que o
handoff QoL registrou. Você escolheu **derrubá-la** com o bypass-unidade. O porquê de ser uma
mini-wave (foundational-adjacente: schema + shell), e não painel-only:

**Um grupo é invisível ao cook, de propósito.** Ele não tem nó nem `output[0]`; a fronteira dele
existe **só** como as arestas que a cruzam (`fold::card_ports`). Então "mutar o grupo como unidade"
**não pode ser** um node-bypass em cada membro — é uma **RE-LIGAÇÃO da fronteira**, aplicada a um
**clone descartável** que o cook lê e **nunca ao documento** (o interior tem de sobreviver ao
un-mute). A convenção é a do Houdini/Nuke, um nível acima do `cook_bypass::bypass_outputs` do próprio
grafo: a saída-slot-0 do grupo passa a fonte da entrada-slot-0, e toda outra saída de fronteira vai
a **Empty** (desplugada).

- **Schema (peça 1):** `MotionDoc.bypassed_subgraphs: BTreeSet<u32>` — o **gêmeo** (um nível acima) do
  `Graph.node_bypassed`, set **paralelo** e não campo no `Subgraph` (construído em ~7 sítios, o
  idioma `widely_constructed_type_favors_optional_component`). Serializado como record **`yg <sid>`**
  na seção `[subgraph]`, **append-only** (o espelho do `y` do node-bypass). ⚠️ **Sem bump de versão
  de formato** — não há constante de versão neste doc; um leitor velho **recusa** o `yg`
  (`ParseError::BadLine`, o bump do caminho-inverso), e um doc sem `yg` faz **round-trip
  byte-idêntico** (`bypassed_subgraphs` nasce vazio; `validate` recusa `yg` fantasma sem grupo).
- **Cook (peça 2):** `cook_graph(motion) -> Option<Graph>` devolve **`None`** no caso comum (nenhum
  grupo bypassed) ⇒ o cook lê `doc.graph` byte a byte, e **agrupar-sem-mutar continua o no-op** que
  a feature de subgrafo promete. Com bypass, clona o grafo e short-circuita a fronteira.
- **Fiação (peça 3):** o H / R-click Mute num **card** seta o **flag do grupo** em vez de expandir; os
  membros ficam **intactos**; o card **desenha muted** (o fold lê `subgraph_bypassed`). Ungroup e
  delete-deep **esquecem** o bypass (senão um `yg` pendente seria recusado no load).

---

## 3. Toque no SHELL (`shells/desktop`) — bridge do Motion

- `motion_bridge_group_bypass.rs` (NOVO) — o transform de cook (`cook_graph` + `short_circuit`).
  Gated por `#[cfg(feature = "panel-motion-graph")]` (usa `fold`/`subgraph`, também gated) + um shim
  `cook_graph -> None` no build sem a feature.
- `motion_bridge_clock.rs` (NOVO) — o `ticks_owed` foi extraído para cá (LOC do `motion_bridge.rs`).
- `motion_bridge_subgraph_clipboard.rs` (NOVO) — `duplicate_nesting`/`paste_nesting`/`DUP_OFFSET`
  extraídos do `motion_bridge_subgraph.rs` (que estava no cap de 600).
- `motion_bridge.rs` — declara os mods `#[path]` + o shim; o cook path lê
  `group_bypass::cook_graph(motion)` e passa `cook.as_ref().unwrap_or(&motion.doc.graph)` ao pump. A
  GPU é pulada enquanto há grupo bypassed (v1 — vide §9).
- `motion_bridge_intents.rs` — o handler `SetBypass` roteia por `subgraph::subgraph_of(id)`: card de
  grupo existente ⇒ `set_subgraph_bypassed`; nó ⇒ `graph.set_bypassed` (o caminho antigo).
- `motion_bridge_fold.rs` — o card view lê `subgraph_bypassed(sid)` (era "todos os membros bypassed").
- `motion_bridge_subgraph.rs` — ungroup/delete-deep limpam `bypassed_subgraphs`.

---

## 4. Schema / contrato / registro — **TUDO INTACTO**

- **Contrato congelado (§6):** `NodeOp=2` / `OpResolver=1` / `NodeManifest=8` — **não tocados** (o
  bypass de grupo vive no `MotionDoc`/`Graph`, nunca no manifesto). `Tool=12` / `RasterEditTool=5` /
  `CanvasPaintTool=1` / `PanelEvent=4` — não tocados.
- **`PROJECT_SCHEMA`** — **não bumpou**. O grafo do Motion viaja como **TEXTO** dentro do
  `ProjectFile` e carrega a própria forma; o `yg` é append-only (§2).
- **Foundational (`ph2d-editor-core` etc.)** — **não tocado**. A seleção de múltiplos fios é estado
  de painel (`selected_wires`), sem `GraphKey` novo (Delete já existe; Shift é modificador do
  wire-click). ⇒ **nenhum gate de árvore combinada** desta linha.
- **`ph2d-ecs` registry** — não tocado.

---

## 5. ⚠️ LOC caps — os dois gates (um NÃO roda com `cargo test -p`)

Dois splits mecânicos por causa dos tetos (extração, não hack):
- `motion_bridge.rs` → `ticks_owed` saiu para `motion_bridge_clock.rs` (**596** < 600).
- `motion_bridge_subgraph.rs` → clipboard saiu para `motion_bridge_subgraph_clipboard.rs` (**472**).

**Todos os arquivos tocados < 600** (shell) e **< 700** (crates). O gate de LOC do shell
(`shells/desktop/tests/file_loc_caps.rs`, teste de **integração**) **não sai** no `cargo test -p
ph2d-host-desktop --bins` — o integrador roda `cargo test -p ph2d-host-desktop --test file_loc_caps`
(conferido verde no tip: 2/2).

---

## 6. Bill of health (verde no tip, antes do rebase)

- `cargo fmt -p ph2d-motion-doc -p ph2d-panel-motion-graph -p ph2d-host-desktop --check` — limpo.
- `cargo clippy -p ph2d-motion-doc -p ph2d-panel-motion-graph -p ph2d-host-desktop --all-targets` — limpo.
- `cargo test -p ph2d-motion-doc` — **20** verdes.
- `cargo test -p ph2d-panel-motion-graph` — **122** verdes (108 + 5 + 6 + 3).
- `cargo test -p ph2d-host-desktop --bins motion_bridge` — **127** verdes (2 `#[ignore]`).
- `cargo test -p ph2d-host-desktop --test file_loc_caps` — **2** verdes.

**Provas de mutação (9 ao todo):** o fio selecionado veste `Accent` (não hover) · o record `yg`
round-trips + accessors são o gêmeo do node-bypass + `yg` fantasma recusado · o cook passa
`input0→output0` e pula o interior · só a saída-slot-0 passa (as demais → Empty) · mutar o card seta
o flag do GRUPO (membros intactos) · card fantasma inerte (nada de `yg`) · ungroup **e** delete-deep
esquecem o bypass (cada sítio) · o card desenha muted só quando o grupo está bypassed-como-unidade.
Cada mutação restaurada por `cp` + `touch`.

---

## 7. Passos de integração (o integrador)

1. `git rebase main` (esta linha não tem foundational nem schema ⇒ conflito improvável; se houver
   textual **fora** dos arquivos da linha, é colisão de mesmo-símbolo → PARE, DIRETRIZ §1.5.5).
2. Gate da árvore combinada: `cargo fmt --check`, `cargo clippy --workspace --all-targets`,
   `nextest`/`cargo test` impactado, **os dois gates de LOC** (`architecture_workspace_file_loc_cap`
   **e** `shells/desktop/tests/file_loc_caps.rs`).
3. `--ff-only`. Sem ADR, sem número a disputar, sem contrato.

---

## 8. Smokes (todos aprovados pelo Enio, `--release`)

- **Seleção de fio + realce distinto** (`smoke OK . siga` / `smoke OK. SIGA`): clicar um fio o
  seleciona (veste `Accent`), Shift acumula o feixe, Delete solta todos.
- **Bypass de grupo** (`smoke OK. siga`): usa o **documento de boot** — abrir a ferramenta **Motion**
  mostra o card **"Age & Fade"** inline na cadeia (a neve strobe). Clicar o card → **H** → o grupo
  bypassa como unidade (os seis nós de envelhecer/desvanecer são pulados; a neve para de envelhecer)
  e o card **apaga**. **H** de novo religa · **Ctrl+Z** desfaz · salvar/reabrir preserva o mute.
  Alternativa mais crísp: `PH2D_SPLICE_SMOKE=1` → splice um `motion.twist`, selecionar scale+twist,
  **Ctrl+G**, **H** no card → a deformação some.

---

## 9. Notas / gotchas — v1 scope do bypass de grupo (smoke-decides, não gap silencioso)

Documentado no doc-módulo do `motion_bridge_group_bypass.rs`:

- **A GPU é pulada** enquanto qualquer grupo está bypassed (o caller força o pump CPU) — um preview
  bypassed nunca é cozido do grafo não-religado. O smoke aprovado usou o pump CPU.
- **`output_nodes`/`time_scopes` leem `doc.graph`** — um bypass não remove NÓ, então os sinks e
  remappers são o mesmo conjunto; só o fluxo de valor (que o pump coze) é religado.
- **Múltiplas saídas de fronteira:** "output[0]" é a de menor `(nó, porta)` (ordem determinística,
  não autorada). Se um grupo tiver várias saídas semânticas, **qual é a "principal" é decisão sua**.
- **Dois grupos bypassed em SÉRIE** compõem de um jeito que o **olho decide** (o smoke com o grupo
  único "Age & Fade" não exercitou isso). Grupos independentes ou **aninhados** (o externo vence — o
  interno já está dentro do interior pulado) estão tratados.

**Aberto (não desta linha — waves grandes, decisão de produto do Enio):** M4 rig+FX · Zona de
Simulação (O4) · keyframes deferidos à timeline. Nenhum é smoke-per-slice; nenhum começa sem escopo.

---

**A linha está FECHADA e ship-clean. Aguarda ordem explícita de integração (Enio-only).**
