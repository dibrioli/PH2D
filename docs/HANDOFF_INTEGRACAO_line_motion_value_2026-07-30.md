# HANDOFF DE INTEGRAÇÃO — `line/motion-value` (2026-07-30)

> Para o **agente integrador** (DIRETRIZ §1.5.9). A linha está **FECHADA**, todos os smokes das
> features aprovados pelo Enio, aguardando **ordem explícita de integração** (Enio-only). O
> implementador NÃO integra nem pusha.

## 0. Estado da linha

- **Branch:** `line/motion-value` · **base:** rebased sobre `main` (o `main` é ancestral de HEAD).
- **42 commits** sobre o `main`. Tip: `86610f06a` (+ um `style` de fmt).
- **Suites verdes no tip** (rodadas isoladas): `ph2d-nodegraph` 102 · `ph2d-panel-motion-graph` 97
  · `ph2d-panel-motion-params` 34 · `ph2d-color` 61 · `ph2d-render` 160 · `ph2d-editor-core` 801.
- **fmt limpo** no shell (`cargo fmt -p ph2d-host-desktop -- --check` = 0 diffs).

⚠️ **RE-REBASE no integração:** o `main` pode ter andado desde o rebase local; refaça
`git rebase main` na integração e rode o gate da **árvore combinada** (`scripts/foundational-integrate.sh`),
não um `cargo test -p` por crate — ver §5 (gotcha dos arch-gates de shell).

## 1. O que a linha entrega (features aprovadas)

Uma jornada de usabilidade do **editor de nós Motion** (grafo + painel de params), do mais antigo ao
mais novo:

1. **Adapter automático** (`motion_bridge_adapt.rs`) — arrastar um fio entre portas incompatíveis
   INSERE o conversor no meio, em vez de recusar. Smoke: `PH2D_ADAPTER_SMOKE=1`.
2. **`value.attribute` vira PICKER DE CANAIS** — palavras em vez de jargão; e o **column picker** lê
   os canais VIVOS do stream, GPU-cook-aware (achava vazio quando o grafo cozinha na GPU — corrigido).
   Smokes: `PH2D_ATTR_SMOKE=1`, `PH2D_PICKER_SMOKE=1`.
3. **Editor de GRADIENTE no painel de params** — `color_ramp` Custom vira gradiente **multi-stop**
   (barra + stops arrastáveis + picker OKLCH), com forma **textual compacta** do `ColorRamp` (o canal
   de text param) e os presets como **sementes editáveis**. Smoke: `PH2D_GRADIENT_SMOKE=1`. Nota de
   design: `docs/Motion Nodes/85_gradient_editor_nota_adr.md`.
4. **Sockets têm FORMA por dimensionalidade** (◇ coluna / ○ valor) e **link ilegal em VERMELHO**
   (W-I3) — novo foundational `ph2d-editor-core::paint_shapes`.
5. **Arrange layout** (botão de arrumação, um clique) + **Frame Selected** (F enquadra a SELEÇÃO
   quando há uma, o grafo todo senão).
6. **Splice (drop-on-wire):** soltar um nó SOBRE um fio o INSERE na cadeia (fonte→novo→alvo);
   R-click num fio → menu → nó inserido; **delete-and-reconnect** (deletar um nó do meio re-conecta a
   cadeia, o Ctrl+X do Blender). Smoke: `PH2D_SPLICE_SMOKE=1` (a cena guarda-chuva de todas as QoL).
7. **Snap magnético** (soltar um fio PERTO de um socket conecta, ~22 px) + o **fio-fantasma PULA**
   para o socket alvo (a metade visual).
8. **QoL de fiação/seleção:** replace-on-drop (fio sobre input ocupado o substitui) · drop-no-corpo
   (nos dois sentidos) · Ctrl+A (select-all) · Ctrl+L (select linked, ilha conectada) · Ctrl+arrastar
   (box-subtract) · **fix: o fio-fantasma pra TRÁS aparece desde o arraste** (era invisível até
   conectar).
9. **⬛ BYPASS/MUTE de nó (H)** — a feature final, o *disable* do Blender/Nuke. Detalhe em §2.

⛔ **REVERTIDO na própria linha, NÃO integra:** o **Post Stack / ADR-0145** (grade HDR de tela
inteira) foi construído (`9a36d4a27`) e **revertido inteiro** (`f2daa787a`) — o smoke reprovou. O
diff cumulativo da linha tem **ZERO ADR novo** e **ZERO mudança em `ph2d-render`**. ⚠️ **Não há
colisão de ADR-0145** desta linha; se o Enio reviver o post-stack, aí sim re-numerar contra o
`0145-3d-layer-*` (rascunho não-rastreado na árvore primária).

## 2. BYPASS/MUTE (H) — a feature final (4 commits: `df45e5866`, `9cf45a02b`, `afe19700f`, `86610f06a`)

Um nó mutado **não roda o op**: o cook passa `input[0] → output[0]`, demais outputs `Empty`
(convenção Houdini/Nuke). Três camadas, cada uma gateada+mutação-provada:

- **Motor** (`ph2d-nodegraph`): `Graph` ganha `node_bypassed: BTreeSet<NodeId>` (o padrão aditivo de
  `node_text_params`); entra no **fingerprint** do cook (`cook_bypass.rs`, passthrough) e no **formato
  textual** (record `y`, header **v5**). `set_bypassed`/`toggle_bypass`/`node_bypassed`/`bypassed_nodes`;
  `remove_node` limpa o set. Splits por LOC: `cook_bypass.rs`, `cook_read.rs`.
- **Vista + visual** (`ph2d-panel-motion-graph`): `GraphNodeView.bypassed` (lido do Graph em
  `snapshot_from`); o card mutado desenha **véu de inerte + risco de quina a quina**.
- **Gesto** (`ph2d-editor-core` + shell): `GraphKey::Bypass` (H, sem modificador) via `graph_key_for`
  + normalizador do shell (`KeyCode::KeyH`); H liga/desliga a seleção pela **regra do rove**
  (mista/alguma-ligada→off; tudo-mutado→on); `GraphIntent::SetBypass{nodes,on}`; o consumidor só muta
  **nós reais** (id de card de subgrafo é tagueado → filtra), marca dirty, 1 undo.

Smoke: `PH2D_SPLICE_SMOKE=1` → selecione um nó, aperte **H** (card apaga + risco); splice um
`motion.twist` e mute ELE (a deformação some); H religa; Ctrl+Z desfaz.

**Aberto, nomeado:** mutar um **grupo** (card de subgrafo) inteiro é decisão de design (flag no grupo?
mutar membros?) — hoje o card de grupo nasce não-mutável, de propósito.

## 3. Schema / contrato / foundational

- ✅ **`PROJECT_SCHEMA` / `VEC_SCENE_SCHEMA` / `DOC_VERSION` INTACTOS** — nenhum bump de schema
  persistido (conferido por grep no diff). O grafo Motion viaja como **TEXTO** no projeto e carrega a
  própria versão.
- ✅ **Formato textual do nodegraph: v5** (record `y` do bypass). `from_text` aceita **v1..v5**; grafo
  sem mute é **byte-idêntico a v1**. Forward-compat: um build ANTIGO (só v1..v4) recusaria um arquivo
  v5 — padrão esperado. A forma textual do `ColorRamp` é outro **canal de text param** (carrega a
  própria versão, não mexe em struct persistido).
- ✅ **Contrato congelado INTACTO** — `NodeOp=2`/`OpResolver=1`/`NodeManifest=8`, gate
  `architecture_contract_surface` **verde no tip**. Todo canal novo (adapter, socket shapes, bypass) é
  side-metadata / text param, nunca o manifest (a lei da linha).
- **Foundational tocado, tudo ADITIVO:**
  - `ph2d-nodegraph`: bypass set + cook passthrough + format v5.
  - `ph2d-editor-core`: `GraphKey::{SelectAll,SelectLinked,Bypass}` + `KEY_KEY_H/L` + re-exports; novo
    `paint_shapes.rs` (socket shapes). `GraphKey` **não** é contrato congelado.
  - `ph2d-color`: `color_ramp_text.rs` + `gradient_preset.rs` (forma textual do ColorRamp).
- **Sem crate nova.** Deps: `ph2d-panel-motion-params` ganhou `ph2d-color` (path, leaf) para o widget
  de gradiente.

## 4. ADR / uniqueness

- **Nenhum ADR numerado novo** no diff cumulativo (o 0145 do post-stack foi revertido). Gate
  `architecture_adr_numbers_are_unique` **verde** nesta árvore.
- `docs/Motion Nodes/85_gradient_editor_nota_adr.md` é uma **nota de design**, não um ADR numerado.

## 5. Como integrar + GOTCHAS

1. **`git rebase main`** (o main pode ter andado) e rode **`scripts/foundational-integrate.sh`** (o
   gate da árvore COMBINADA) + Mergiraf no resíduo textual.
2. ⚠️ **Arch-gates de shell só rodam na varredura IMPACTADA** — os gates em `shells/desktop/tests/`
   (e os `#[cfg(test)]` do `render_loop`) NÃO são alcançados por um `cargo test -p <crate>` por crate
   (a lição que a `line/Vector` e a `line/physics` pagaram: `file_loc_caps`, os `architecture_*` de
   parity). Rode `cargo test -p ph2d-host-desktop` **inteiro** na árvore combinada, além do
   `ship.sh`.
3. ⚠️ **Colisão de mesmo-símbolo com outras linhas da jornada:** se `line/anim`/`line/physics`/etc.
   integraram no meio, confira se `KEY_KEY_H`/`GraphKey::Bypass`/`GraphIntent::SetBypass` ou os
   verbos de seleção não colidem. São todos apêndices; um conflito seria textual (mesma lista),
   resolvível por Mergiraf/`--ff-only` — **só ADICIONE em listas compartilhadas**.
4. **`ship.sh`** (paridade CI: fmt, clippy `--all-targets`+features, machete, deny, audit, nextest
   `--cargo-profile ci-test`, typos). A linha está fmt-limpa e sob os caps de LOC; drene os latentes
   (2-4 iterações, o normal do integrador).
5. **Atualize `CLAUDE.md §5`** (a entrada do Motion Nodes) e **`project-memory`** com o que landou —
   e registre que o **post-stack foi revertido** (para ninguém reconstruir a ADR-0145 achando que é
   nova).

## 6. Smokes (todos com a ferramenta Motion; `--release` recomendado)

| Flag | O quê |
|---|---|
| `PH2D_SPLICE_SMOKE=1` | Splice + snap + replace + drop-no-corpo + Ctrl+A/L + box-subtract + **BYPASS (H)** |
| `PH2D_ADAPTER_SMOKE=1` | Adapter automático (fio incompatível insere conversor) |
| `PH2D_ATTR_SMOKE=1` | `value.attribute` picker de canais |
| `PH2D_PICKER_SMOKE=1` | Column picker (canais vivos, GPU-cook-aware) |
| `PH2D_GRADIENT_SMOKE=1` | Editor de gradiente multi-stop |

Rodar: `cd <worktree> && env PH2D_<FLAG>=1 cargo run -p ph2d-host-desktop --release`. Cada cena
imprime `[<nome> smoke] …` com as instruções do gesto.

## 7. Gotchas técnicos (para o integrador não tropeçar)

- **`motion.scale` não move `P`** — seu `amount` escala outra coisa; o smoke do bypass usa um
  **deformer** (`motion.twist`) para a saída-visível. (Custou um gate red no fechamento.)
- **`NodeRegistry::new()`** não resolve tipos de teste (`test.a`) — o `snapshot_from` lê `bypassed`
  direto do Graph, então o gate de plumbing funciona com registry vazio.
- **Rodar suites em DEBUG também** — o `--release` esconde pânico (a lição do `ph2d-flip-colorize`).
