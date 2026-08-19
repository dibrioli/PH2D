# HANDOFF DE INTEGRAÇÃO — `line/motion-value` (command-palette + doc 86: objetos no grafo, Duplicator, ponte de render, preview em moldura)

**Status:** FECHADO 2026-08-02 · no `main` em `eaaa7f1d9` (o commit que trouxe este arquivo).

**Data:** 2026-08-02 · **Branch:** `line/motion-value` · **HEAD:** `72c617dc0` · **Base (merge-base):** `main` `3197c5c9e`
**Ordem do Enio:** *"Smoke OK"* (aprovando cada fatia, incl. o lote final doc 86 B1/A5) — **todos os smokes aprovados.**

> ⚠️ **Este handoff SUPERSEDE** `docs/Motion Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_group_bypass_2026-08-01.md`,
> que cobria só as **5 primeiras** commits (fio-conjunto + bypass de grupo). A linha AVANÇOU: o
> command-palette de tela cheia (8 commits) e o plano **doc 86** inteiro (Wave B + A1–A5 + B1, 8 commits)
> entraram por cima. São **21 commits, `main..HEAD`** — integre o range inteiro de uma vez.
>
> **Você é o integrador. A linha NÃO se integra sozinha.** `git rebase main` → gate da árvore
> combinada → `ship.sh` → push (só por esta ordem do Enio). A linha fecha aqui.

---

## 0. TL;DR do integrador (o que decide a integração)

- **`git rebase main` é TRIVIAL:** main andou **1 commit** desde a base (`a9f5977e9`), e ele toca **só
  `project-memory/`** — **zero conflito de código**. Esta linha não toca `project-memory/`.
- **Contrato congelado INTACTO** (`architecture_contract_surface` **3/3 verde** no tip): `NodeOp=2` /
  `OpResolver=1` / `NodeManifest=8`. **Nenhum ADR.**
- **`PROJECT_SCHEMA` NÃO bumpa.** O único "schema" é o record **`yg`** (bypass de grupo) no formato
  **TEXTUAL** do grafo — **append-only**, o gêmeo do record `y` do node-bypass; o grafo viaja como texto
  e carrega a própria versão (header `v1`, um grafo que ninguém mutou é byte-idêntico a v1).
- **2 crates-nó NOVAS** (`ph2d-node-source-object`, `ph2d-node-motion-duplicator`): **glob members**
  (zero edição de `Cargo.toml` central) + **registradas** em `ph2d-node-registry-init` (conferido).
- **Toque foundational = `ph2d-editor-core`, todo ADITIVO** (command-palette + `paint_batch::draw_image`).
  Nenhum contrato foundational; o gate da árvore combinada confere.
- **Gates de GPU são `#[ignore]`** (A2/A3 paridade de bake + gate-5-flip): **rode-os no adapter (RTX)** —
  sem adapter fazem *skip gracioso*, que **não é verde**.

---

## 1. O que a linha entrega (21 commits, `main..HEAD`) — três clusters

### Cluster 1 — Fio-conjunto + BYPASS de grupo (5 commits, `75ffed08d`..`5724bf231`)
Detalhe completo no handoff anterior (`..._group_bypass_2026-08-01.md`), que este referencia:
- **O fio vira CONJUNTO** (Shift acumula, Delete solta o feixe) + realce próprio (`Accent`, distinto do hover).
- **H num card de grupo BYPASSA o grupo como unidade** — o record **`yg`** (`MotionDoc.bypassed_subgraphs`),
  o cook RE-LIGA a fronteira do grupo bypassed. Derruba a cerca de Chesterton *"card de grupo nasce não-mutável"*.

### Cluster 2 — O COMMAND-PALETTE de tela cheia (8 commits, `33b7f0758`..`6933f1e60`)
O *node picker* de tela cheia que **substitui o dropdown de Add Node em TODOS os casos** (a tecla **A** abre).
- Superfície nova em **`ph2d-editor-core`** (foundational, aditiva): `widget/command_palette.rs`
  (`PaletteModel` + `paint_command_palette`), `screens/hero/chrome/command_palette.rs`, e a máquina de estado
  na interação (`open_command_palette`/`close`/`query`/`push_char`/`backspace`/`model`/`open` em `state/`),
  `CMD_PALETTE_CLOSE` (id por **hash de string**, não numerado ⇒ sem gate de contagem), toques em
  `interaction/dispatch/pointer_down.rs` · `hit.rs` · `chrome_ops.rs`. Fiação de input no shell:
  `command_palette_input.rs`.
- UX: campo de busca/filtro · cada categoria vira um CARD · categorias grandes viram FAIXAS full-width ·
  masonry balanceado (grandes span 2 colunas). O modal ganha os cliques (a v1 não clicava nada — corrigido).

### Cluster 3 — DOC 86: os objetos da engine no grafo + o preview em moldura (8 commits, `7bcd160b6`..`72c617dc0`)
Plano vivo: **[`docs/Motion Nodes/86_plano_objetos_engine_render_e_preview.md`](../86_plano_objetos_engine_render_e_preview.md)**.
- **Wave B (`7bcd160b6`):** o preview sai de dentro do card para uma **moldura própria** acima/abaixo, com toggle no header.
- **A1 (`393f986d1`):** os objetos da engine entram no grafo e são desenhados — a coluna **`texture_id`**
  (`lower.rs`, fallback 0 = byte-idêntico) + o nó **`source.object`** (media-agnóstico) + o nó **`motion.duplicator`**
  (Shape × Points) + a membrana que resolve o SPRITE (`render_loop/motion_bridge_objects.rs`).
- **A2 (`264ef8755`):** bake-to-tile de **vetor** (`motion_object_bake.rs`, cache por conteúdo, câmera fixa por DPI).
- **A3 (`7ac35f633`):** bake-to-tile de **Flip** (`motion_flip_bake.rs`, compõe as camadas no frame; scratch renderers).
- **A4 (`9417ef63d`):** um **GRUPO** de mídia mista vira **N instâncias VIVAS** (`walk_group_transforms`/`resolve_leaf`/`group_stream`).
- **B1 (`3c6b81828`):** o preview do **Spawn para de PISCAR** — a moldura existe pelo TIPO do output (`geom::has_preview_slot`, `Instances/Vec2`), não pelo conteúdo do frame.
- **A5 (`72c617dc0`):** o preview vira o **thumbnail assado** do objeto (uniform `texture_id` → mini-render na moldura; downscale premultiplicado cacheado; `paint_batch::draw_image`).

---

## 2. Schema / contrato / registro — a prova

| Item | Estado | Como conferi |
|---|---|---|
| Contrato congelado (`NodeOp`/`OpResolver`/`NodeManifest`) | **INTACTO** | `cargo test -p ph2d-nodegraph --test architecture_contract_surface` → **3/3** |
| `PROJECT_SCHEMA` | **NÃO bumpa** | o grafo viaja como TEXTO; o record `yg` é **append-only** (header v1, gate `the bypassed group writes a yg record`) |
| Crates-nó novas (2) | **glob members + registradas** | `Cargo.toml` usa glob; `registry-init` chama `ph2d_node_source_object::register` + `ph2d_node_motion_duplicator::register` |
| ADR | **nenhum** | — |
| Registry ECS / contagem de nós | **não tocado** | nenhum gate de contagem no diff |

⚠️ **Coluna `texture_id` é CONVENÇÃO de stream** (como `uv_rect`/`P`), lida por `lower_to_instances` com fallback 0 — **não** é campo do `NodeManifest`. `PreviewThumb`/`thumbnail` são campo da view do PAINEL (`ph2d-panel-motion-graph`), não foundational-congelado.

---

## 3. Toque foundational (`ph2d-editor-core`) — todo ADITIVO

- **Command-palette** (Cluster 2): widget + chrome + máquina de estado + ids por hash. **Aditivo** — nenhum
  contrato foundational, nenhum id numerado. ⚠️ Se OUTRA linha desta janela tocou `interaction/dispatch` ou
  `screens/hero/chrome`, o rebase pode conflitar textualmente ali — **mas o único commit de main é
  project-memory**, então na prática não há.
- **`paint_batch::draw_image`** (A5): `pub fn` novo, aditivo (o padrão do `fill_dots`).
- **Splits por LOC cap** (todos FILHOS/`#[path]`, sem mudança de visibilidade): `snapshot.rs` → `+snapshot_thumb.rs`;
  `motion_bridge_subgraph.rs` → `+motion_bridge_subgraph_clipboard.rs`; e os `_tests.rs` do painel.

---

## 4. LOC caps — os dois gates (um NÃO roda com `cargo test -p`)

Verde no tip. ⚠️ **Lembrete estrutural** (a família documentada): os gates de LOC de `shells/desktop/tests/` e o
`architecture_panel_loc_cap`/`architecture_workspace_file_loc_cap` (em `ph2d-editor-core/tests/`) **só rodam na
varredura impactada** — um fechamento por `cargo test -p` por crate NÃO os alcança. Rode-os explícitos:

```
cargo test -p ph2d-editor-core --test architecture_panel_loc_cap
cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap
```

Verificados no tip: `snapshot.rs` 592 · `motion_object_bake.rs` 422 · `motion_flip_bake.rs` 521 · todos < 600/700.

---

## 5. Bill of health (verde no tip, ANTES do rebase)

- `architecture_contract_surface` **3/3** · `architecture_panel_loc_cap` **3/3** · `architecture_workspace_file_loc_cap` **2/2**.
- `cargo test -p ph2d-panel-motion-graph --lib` **106/106**.
- Gates doc 86 A5 (bin `ph2d-host-desktop`): `the_thumbnail_is_bounded_and_keeps_aspect` · `a_small_tile_is_never_upscaled` ·
  `the_downsample_does_not_bleed_a_halo_into_a_transparent_edge` · `a_uniform_nonzero_texture_id_earns_a_thumbnail` — **4/4**.
- Gates B1 (painel): `the_preview_moldura_survives_an_empty_stream` · `_is_identical_whether_empty_or_full` ·
  `a_value_node_has_no_preview_moldura` · o do hit do toggle — **verde**.
- `cargo clippy -p ph2d-panel-motion-graph -p ph2d-editor-core -p ph2d-host-desktop` **limpo**.
- ⚠️ **Gates de GPU `#[ignore]`** (A2/A3): NÃO rodei sem adapter no fechamento. **Rode no RTX:**
  `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop -- --ignored` (o `a_baked_flip_object_carries_...` do A3,
  paridade de bake). Sem adapter fazem *skip*, que não é verde.

---

## 6. Passos de integração (você, o integrador — DIRETRIZ §1.5.4)

1. `cd` na worktree · `git rebase main` (trivial — só `project-memory` em main; se conflitar em código, **PARE e reporte**: é colisão de mesmo-símbolo, não prevista aqui).
2. Rode o gate da **árvore combinada** (`scripts/foundational-integrate.sh` OU a suíte impactada) + os 2 gates de LOC acima + o contrato + os gates de GPU no RTX.
3. `./scripts/ship.sh` (paridade CI: fmt · clippy `--all-targets`+features · machete · deny · audit · nextest · typos). Corrija todo `✗`.
4. `git push origin main` → babysit o run até `success` (link `https://github.com/dibrioli/PH2D/actions/runs/<id>`).
5. Atualize o **CLAUDE.md §5** (Motion Nodes) com a entrada da integração + este handoff, e marque o doc 86 como integrado.

---

## 7. Smokes (todos aprovados pelo Enio, `--release`)

- **Doc 86 A1–A5 + B1** — `PH2D_MOTION_OBJ_SMOKE=1..4` (`=1` sprite · `=2` estrela vetor · `=3` Flip · `=4` grupo misto):
  abra o **grafo de Motion**. Os objetos são carimbados numa grade (A1–A4); os cards **Object**/**Duplicator** mostram o
  **thumbnail** do objeto na moldura (A5), e um grupo (`=4`) mantém o scatter (mídia mista).
- **B1 (preview não pisca)** — documento **default** (sem env), grafo aberto, **Play**: a moldura do **Spawn** fica
  parada; num tick vazio mostra uma caixa emoldurada VAZIA, não some. Repro garantido: `PH2D_GPU_COOK_DEMO=5` (emitter fountain).
- **Command-palette** — a tecla **A** abre o picker de tela cheia (busca + cards por categoria); substitui o dropdown antigo.
- **Fio-conjunto + bypass de grupo** — ver o handoff anterior (`..._group_bypass_2026-08-01.md` §8): clicar fio → `Accent`,
  Shift acumula, Delete solta o feixe; **H** num card de grupo bypassa o grupo (documento de boot / `PH2D_SPLICE_SMOKE=1`).

---

## 8. Aberto — follow-ups (NÃO bloqueiam; são decisão do Enio)

O **plano 86 está com TODAS as waves construídas** (B · A1–A5 · B1). Sobram, nomeados no §8/§9.6 do plano:
1. ⚠️ **Grupo rotacionado/escalado re-orientando filhos vetor/flip** — **decisão de arquitetura**. O tile é assado no
   linear de **MUNDO** (`ObjectBake::linear`), então corrigir força *bake CANÔNICO + linear ao vivo* (unifica sprite e
   vetor/flip, mas re-abre o trade de qualidade rotação-assada-crisp vs viva-com-alias de A2/A3). Não construído às cegas.
2. `source.selection`/tag (source nodes de conveniência) · filho vetor/flip **sem nome** pulado (o bake é name-keyed) ·
   FREEZE por-nó.
3. Do bypass de grupo: laço/série de dois grupos bypassed compõe de um jeito que o olho decide (smoke aprovado); GPU-cook
   de grupo bypassed usa o pump CPU (o smoke usou CPU).

---

**Fim do handoff. A linha fecha e PARA.** Integração e push só por ordem explícita do Enio, por um agente integrador.
