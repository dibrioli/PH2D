# HANDOFF DE INTEGRAÇÃO — `line/motion-value` (2026-07-13)

> **Para o agente integrador.** A linha está **fechada e parada**. Não integrei, não pushei.
> Gate em lote verde (fmt · clippy `-D warnings` · 3 caps de LOC · typos · machete · suíte).

---

## 1. Coordenadas

| | |
|---|---|
| **Branch** | `line/motion-value` |
| **HEAD** | `621f93f4` |
| **Base** | `4cd8ef13` (= `main` no início da jornada) |
| **Commits** | 4 |
| **O que entregou** | **FILA 1 — subgrafos** (nesting: card colapsado + duplo-clique entra + breadcrumb) |
| **Nota-ADR** | [`docs/Motion Nodes/57_subgrafos_nota_adr.md`](Motion%20Nodes/57_subgrafos_nota_adr.md) |

```
621f93f4 fix(motion): dois no-ops silenciosos que a auditoria propria achou (doc 57)
0d9aca5c chore(typos): allowlist pt-BR — frase/organizacional/HDA (doc 57)
deaecd86 docs(motion): nota-ADR 57 — subgrafos sao uma dobra da vista
13f816d1 feat(motion): subgrafos — nesting como DOBRA da vista, sobre um grafo que segue PLANO
```

## 2. A decisão de arquitetura, em três linhas (leia antes de qualquer conflito)

**O `Graph` NUNCA aninha.** O documento ganha pertencimento (nó → subgrafo) e o **editor
dobra**. O contrato congelado é que forçou isso: `NodeManifest.inputs` é `&'static` (portas
dinâmicas são impossíveis) e `NodeOp::eval` não recebe `Cook`/`OpResolver` (um nó não consegue
cozinhar um sub-grafo). É o *collapsed graph* da Unreal.

**Consequência que te interessa:** o cook é **byte-idêntico** com e sem grupo. Gate:
`grouping_never_changes_the_cook`. Se ele ficar vermelho depois do teu merge, **alguém quebrou
o desenho, não o teste**.

## 3. ⚠️ Foundational tocado (as superfícies de colisão)

| Arquivo | O que mudou | Risco de colisão |
|---|---|---|
| `ph2d-editor-core/src/interaction/types.rs` | **+2 variantes no fim** do `enum GraphKey`: `Group`, `Ungroup` | **Baixo** (append-only). Se outra linha também apendou, a união é trivial — mas **conte, não escolha** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). |
| `.../dispatch/keymap.rs` | **`KEY_KEY_G: u32 = 0x47`** (const nova) | Baixo |
| `.../dispatch/key.rs` | `graph_key_for` ganhou o parâmetro **`alt`** (assinatura mudou; 1 caller) | **Médio** — se outra linha mexeu em `dispatch_key`, o merge textual pode passar e o `check` pegar. |
| `.../dispatch/mod.rs` | `KEY_KEY_G` no `pub use keymap::{…}` | Baixo |
| `.../dispatch/tests/graph.rs` | `key_cmd` virou wrapper de `key_chord(kc, cmd, alt)`; +1 caso e +1 teste | Baixo |
| **`.typos.toml`** | **+3 palavras** (`frase`, `organizacional`, `HDA`) na `[default.extend-words]` | **ALTO se outra linha também adicionou** — chave duplicada **mata o TOML no parse** e o typos nem escaneia ([[feedback_duplicate_allowlist_key_kills_the_gate_at_parse.md]]). **Funda a união, sem duplicar chave.** |

**Contrato congelado: INTACTO.** `architecture_contract_surface` = 3 verdes (8/2/1).
`IconId` **não** mudou (o `IconId::Group` e o `docs/design/icons/group.svg` já existiam).
**Zero `GraphHitKind` novo** — o breadcrumb anda no canal `Chrome { id }` que já existia.

## 4. Ids / consts novos (para o integrador detectar colisão)

- `ph2d_panel_motion_graph::SUBGRAPH_VIEW_TAG = 0x8000_0000` — bit que marca um view id como
  **card** (os espaços de id de nó e de subgrafo são independentes).
- `paint_chrome::CHROME_GROUP = 6` (7º chip da toolbar) e **`CHROME_CRUMB_BASE = 100`** (crumb
  `i` = `100 + i`). Se outra linha adicionou chip na mesma toolbar, **o ordinal 6 colide** —
  renumere o dela ou o meu; os dois são locais ao painel.
- Records novos do formato textual: **`g` / `m` / `bm`**, na seção nova **`[subgraph]`**.

## 5. Crates / arquivos novos

**Nenhuma crate nova.** 8 arquivos novos, todos módulos irmãos (LOC cap):

```
crates/ph2d-motion-doc/src/subgraph{,_tests}.rs          modelo + travessias + formato
crates/ph2d-panel-motion-graph/src/interact_subgraph{,_tests}.rs   gestos
crates/ph2d-panel-motion-graph/src/interact_key.rs       (split: apply_key saiu de interact.rs)
crates/ph2d-panel-motion-graph/src/snapshot_intent.rs    (split: GraphIntent saiu de snapshot.rs)
crates/ph2d-panel-motion-graph/src/paint_breadcrumb.rs   breadcrumb
crates/ph2d-panel-motion-graph/src/paint_menu.rs         (split: draw_add_menu saiu de paint.rs)
shells/desktop/src/render_loop/motion_bridge_fold.rs     A DOBRA (a vista)
shells/desktop/src/render_loop/motion_bridge_subgraph{,_tests}.rs  os verbos
shells/desktop/src/render_loop/motion_bridge_intents.rs  (split: apply_graph_intents saiu do bridge)
```

**4 splits foram forçados pelo cap de LOC** (`interact.rs`, `snapshot.rs`, `paint.rs`,
`motion_bridge.rs` estouraram). Se outra linha editou esses 4 arquivos, **o merge textual vai
doer** — as funções que ela mexeu podem ter MUDADO DE ARQUIVO. Confira por símbolo, não por
linha.

## 6. Mudanças de assinatura (o `check --workspace` pega; o merge-tree não)

| Símbolo | Antes | Agora |
|---|---|---|
| `hits::push_card_hit` | `(hits, node: u32, body, canvas)` | `(hits, n: &GraphNodeView, body, canvas)` |
| `paint_chrome::draw_split_chrome` | 7 args soltos | `(…, state: ChromeState)` (struct nova) |
| `interact::apply_key` | `(state, k, rect)` | `(state, k, rect, snap)` — e **mudou de arquivo** |
| `edit::duplicate` | `-> Vec<u32>` | `-> BTreeMap<NodeId, NodeId>` (o mapa origem→cópia) |
| `motion_demo_strobe::build` | `-> Option<Vec<NodeId>>` | `-> Option<Demo>` (struct nova) |
| `plumbing::reconcile_after` | (inalterada) | mas os **7 call-sites** agora passam por `motion_bridge::reconcile` |
| `GraphNodeView` | — | **+campo `kind: NodeViewKind`** (toda construção precisa dele) |
| `GraphViewSnapshot` | — | **+campos `level` e `breadcrumb`** |

## 7. O que só o `ship.sh` pega (conte com 2–4 iterações)

Rodei local: **fmt (pinado 1.95) · clippy `-D warnings` · typos · machete · os 3 caps de LOC ·
`architecture_contract_surface` · a suíte das 4 crates tocadas** — todos verdes.
**Não** rodei: `cargo deny`, `cargo audit`, nextest com `--cargo-profile ci-test`, matrix
macOS/Windows. Não adicionei dependência nenhuma, então `deny`/`audit` não deveriam mexer.

## 8. Smoke (o Enio)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
```
Abra o Motion. **A neve tem que cair exatamente como antes** — e no meio da cadeia agora há um
card só, **"Age & Fade"**, com uma pilha desenhada atrás dele e o rótulo "6 nodes".

- **Duplo-clique nele** → você entra. O breadcrumb "Root / Age & Fade" aparece no canto
  superior esquerdo; os vizinhos de fora (o collide e o falloff) estão lá, **velados** — são
  ghosts, não dá pra arrastar nem apagar, mas dá pra puxar fio deles.
- **Clique em "Root"** → você volta.
- **Selecione uns nós e Ctrl+G** (ou o 7º chip da toolbar) → viram um card. **Ctrl+Alt+G**
  desfaz. Selecione o card → o painel de params mostra o nome, editável.
- **Ctrl+Z** desfaz o agrupamento (e se você estiver DENTRO do grupo quando desfizer, o editor
  te devolve pra raiz em vez de te deixar numa sala que não existe mais).

## 9. Fila restante (o próximo da linha escolhe)

1. **FILA 2 — promoção param → socket.** Continua não existindo. Mesmo padrão aditivo.
2. **FILA 3 — FX passes no compositor HDR** (glow/bloom/blur/vignette/levels/hue-shift).
   Reuso obrigatório do compositor GPU do Painter (`ph2d-painter-effects`).
3. **FILA 4 — os 4 nós que faltam** (`motion-delay`, `motion-buoyancy`,
   `motion-distribute-poisson`, `motion-path`).
4. **FILA 5 — W4.T4** (dock da timeline no `motion_timeline_slot`) — coordene com o Enio, encosta
   na linha `anim`.

### Gaps conhecidos DESTA fatia (nomeados, não escondidos)

- **Um fio NOVO para dentro de um grupo se autora de dentro** (os inputs de um card são os fios
  que cruzam, então estão sempre ocupados — ADR §3). O caminho natural pra fechar: **soltar um
  fio no corpo do card abre um menu** das portas de entrada **livres** dos membros — é o mesmo
  maquinário do smart-connect, filtrado por compatibilidade. Seria a FILA 1.b.
- **Um backdrop cujos nós foram agrupados fica órfão** na raiz (emoldurando nada). Ele não é
  apagado nem carregado junto — decisão consciente (um backdrop não possui nada, ele desenha em
  volta), mas dá pra questionar.
- **Reuso/instanciação** (o datablock do Blender / Gizmo / HDA) está **fora de escopo por
  desenho** (ADR §7) — exige o contrato congelado.
