# Handoff de integração — `line/motion-value` (DIRETRIZ §1.5.9)

> **Para o agente integrador.** A linha está FECHADA. Não integrei, não shipei, não pushei.
> Data: 2026-07-12.

---

## 1. Coordenadas

| | |
|---|---|
| **Branch** | `line/motion-value` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` |
| **HEAD** | `dd540b78` |
| **Base (merge-base com `main`)** | `3805f650` |
| **Commits** | 34 |
| **`main` andou desde a base?** | **Não** — 0 commits. A base está limpa; o merge deve ser `--ff-only` sem rebase. |

Se `main` tiver andado até você ler isto, veja §6 (superfícies de colisão).

---

## 2. O que a linha entrega

Motion Nodes saiu de "grafo que cozinha" para **módulo editável e persistente**.

**Editor de nós (F2/F3)** — o grafo agora se explica sozinho: backdrops (grupos que carregam
o que emolduram), faca (cortar fios), probe + sparkline, smart-connect, Ctrl+D, waypoints
(reroute é um nó de verdade), readouts inline por card, véu nos nós inertes, marcha nos fios
vivos, largura ∝ massa do stream, e um *postage stamp* por card. Mais: pan no botão do meio,
box-select no esquerdo, scroll + barra arrastável no add-menu (86 nós não cabiam na tela).

**Zona de Simulação (O4)** — o horizonte do doc 03, fechado: `sim.zone` (estado vivo entre
ticks), `sim.step` (Euler), `sim.spawn` (nascimento com ids replay-estáveis), `sim.lifetime`
(idade/vida), `sim.collide` (chão/disco/tigela). O documento de boot é a **chuva**: nasce,
acelera, envelhece, desvanece, colide e assenta num regime estacionário.

**M4** — rig (`rig.skeleton`/`fk`/`ik_2bone`/`fabrik`/`rubber_hose`/`skin_deformer`), FX
por-instância (`fx.rgb_split`/`drop_shadow`), cauda do M3 (`pin_constraint`, `slit_scan`).

**Persistência** — o grafo entra no projeto (Ctrl+S / Ctrl+O). Ele **já tinha** serialização
textual completa e ninguém a chamava.

**Perf** — o FPS era o número de **draw objects** (~5000 → ~250/quadro). A Vello cobra por
objeto, não por vértice.

Notas-ADR: `docs/Motion Nodes/36..56`.

---

## 3. ⚠️ Foundational tocado (o que pode colidir)

Modo L autoriza (ADR-0107), mas **é aqui que a integração dói**. Tudo abaixo é
**estritamente aditivo — zero remoções, zero assinaturas alteradas**. Nenhuma outra linha
perde símbolo; o pior caso é conflito *textual* de linhas vizinhas, que o Mergiraf funde.

### `ph2d-nodegraph` — **CRATE DO CONTRATO CONGELADO** (§6, ADR-0039)

**+96 linhas, −0.** O contrato **NÃO foi tocado**: `NodeOp=2` / `OpResolver=1` /
`NodeManifest=8` intactos. Gate `architecture_contract_surface` **verde**.

O que foi adicionado são **métodos inerentes** e uma const — fora da superfície congelada:

| Símbolo | Arquivo | O quê |
|---|---|---|
| `EvalCtx::started()` | `cook.rs` | o nó já rodou? (a Zona distingue `init` de `state`) |
| `EvalCtx::dt()` | `cook.rs` | passo de tempo da lane raiz (0 num escopo de tempo) |
| `Cook::peek()` | `cook.rs` | lê o memo do cook do quadro — o readout é **lookup, nunca cook** |
| `Cook.prev_playhead` | `cook.rs` | + carregado no `CheckpointRing` |
| `attr::SIZE_IDENTITY` | `attr.rs` | `[1.0, 1.0]` — o fallback de `size` é a IDENTIDADE |

> `SIZE_IDENTITY` **corrigiu um bug real**: o fallback era `0.4`, então um `motion.scale` em
> `amount = 1` escalava a cena por 2.5× só de existir. Se outra linha depender do 0.4, ela
> depende de um bug (doc 39).

### `ph2d-editor-core` (+182, −0)

- **`paint_batch.rs`** (módulo NOVO, irmão): `fill_dots` / `stroke_subpaths` — N formas em
  **um** objeto de desenho. É o coração do fix de FPS e é **reusável por qualquer painel**.
- `graph_double` (`set_`/`take_`) no `WidgetStore` — o grafo nunca via duplo-clique.
- `pointer_down.rs` / `pointer_up.rs` / `graph_ops.rs`: dispatch da superfície de grafo.
- `tests/hr12_widgets_a11y.rs`: **3 entradas novas** no `PANEL_A11Y_DELEGATE_OK`
  (`paint_wire.rs`, `paint_wire_tests.rs`, `paint_stamp.rs` — pintura pura dentro do card,
  cujo nó AccessKit é o do `hits.rs`).

### `ph2d-tokens` (+2 tokens)

`ColorToken::GraphMarquee` e `ColorToken::GraphInert`. Ambos **translúcidos por contrato**
(um véu sobre a carta, não um repinte dela).

### `ph2d-eval-motion` (+9)

`MotionCookPump::is_dirty()` — para um edit que NÃO pode re-cozinhar poder provar isso
(o drag de backdrop é decoração; não muda o que o grafo cozinha).

### `shells/desktop`

`input_handlers.rs` é o ponto sensível: as teclas do grafo (`Ctrl+D`, `K`, `P`) entram
**acima** do `K` global da timeline — um braço abaixo dele é inalcançável e a faca nunca
armaria (foi o `unreachable_pattern` do clippy que pegou). Se outra linha mexeu nesse `match`,
**cheque a ordem dos braços**, não só o merge textual.

Também: `project.rs` (`PROJECT_SCHEMA` **2 → 3**), `motion_state.rs`, e o `motion_bridge`
partido em irmãos (`_readout`, `_edit`, `_rewire`, `_connect`, `_backdrops`, `_color`).

---

## 4. Crates novas: 17 nós (71 → **88**)

`sim-zone` · `sim-step` · `sim-spawn` · `sim-lifetime` · `sim-collide` · `value-attribute` ·
`util-reroute` · `rig-skeleton` · `rig-fk` · `rig-ik-2bone` · `rig-fabrik` · `rig-rubber-hose` ·
`rig-skin-deformer` · `fx-rgb-split` · `fx-drop-shadow` · `motion-pin-constraint` ·
`motion-slit-scan`

**`Cargo.lock` e `ph2d-node-registry-init` vão conflitar. NÃO resolva à mão — REGENERE:**

```bash
cargo run -p ph2d-node-sync
```

Crates-nó **modificadas** (existiam no `main`): `motion-drive` (canal Opacity),
`motion-collide`, `motion-integrate`, `motion-scale`, `motion-spring`, `debug-wave`.

---

## 5. Estado do gate (o que EU rodei — e o que só o `ship.sh` pega)

Rodado no worktree, sobre `dd540b78`, **verificado por exit code** (não por texto — o pipe
mascara o código de saída):

| Gate | Resultado |
|---|---|
| `cargo test --workspace` (exceto `asset-cooker`) | **exit 0** — 582 suítes |
| `cargo clippy --workspace --all-targets -- -D warnings` | **exit 0** |
| `rustup run 1.95 cargo fmt --all -- --check` | **exit 0** |
| `architecture_contract_surface` (contrato congelado) | **verde** |
| `architecture_panel_loc_cap` · `architecture_workspace_file_loc_cap` · `file_loc_caps` (shell) | **verde** |
| `hr12_widgets_a11y` | **verde** |

**O último commit (`dd540b78`) existe por causa deste gate**, e é o aviso principal desta
seção: o gate de fechamento achou **duas bombas latentes** que o inner loop (`cargo check -p`)
nunca veria e que teriam feito o CI vermelho **na sua mão**:

1. `ph2d-node-motion-drive` estava **desformatado** desde um commit anterior desta linha.
2. `motion_bridge_params.rs` bateu **604 > 600** — o cap do **shell**
   (`shells/desktop/tests/file_loc_caps.rs`), que é um gate **distinto** do cap de 700 do
   workspace. Resolvido por **split** (`motion_bridge_color.rs`), nunca por allowlist.

**Ainda assim, conte com 2–4 iterações de vermelho no `ship.sh`.** Ele roda o que o gate
per-linha NÃO roda: `machete`, `deny`, `audit`, `typos`, e o `nextest --cargo-profile ci-test`.
Com 17 crates novas, o `machete` (deps não usadas) e o `typos` são os candidatos mais
prováveis — e o `typos` tem allowlist pt-BR própria.

---

## 6. Superfícies de colisão, em ordem de risco

1. **`ph2d-node-registry-init` + `Cargo.lock`** — colisão *garantida* se outra linha criou
   nós. **Regenere** (§4), não resolva.
2. **`input_handlers.rs`** — a **ordem dos braços** do `match` é semântica, não cosmética
   (§3). Um merge textual "limpo" pode deixar a faca morta.
3. **`ph2d-nodegraph/src/cook.rs`** — crate do contrato congelado. Meu diff é aditivo e o
   gate está verde, mas se outra linha também apendou métodos em `EvalCtx`/`Cook`, é
   conflito de linhas vizinhas (Mergiraf funde; **rode o gate depois**).
4. **`hr12_widgets_a11y.rs` / `FILE_OVERAGE_OK`** — allowlists são ímãs de conflito. As
   entradas são independentes; una as duas listas.
5. **`ph2d-tokens/src/color.rs`** — 2 variantes apendadas ao enum.

---

## 7. Pendências (NÃO são bloqueio de integração)

- **Smoke do Enio pendente** nas duas últimas fatias: o fix dos sliders (`d9cbc10b`) e a
  persistência do grafo (`36bdb80a`).
- `PROJECT_SCHEMA` 2 → 3: postcard é posicional, então **saves v2 não abrem**. Sem custo real
  (a persistência ainda é stub — path fixo, sem diálogo de arquivo).
- Aberto no módulo: GPU (linha foundational dedicada, **nunca** enxertada aqui) · keyframes
  (deferidos até a timeline) · `Cook::checkpoint`/`restore` já landou (M2.N2).
- O contrato do motor novo (`ph2d-vec-*`) segue **não congelado** — não é desta linha.

---

## 8. Lições que valem além desta linha

- **Uma régua não pode ser função do que ela mede.** O fallback do slider era
  `max = valor × 4` — realimentação positiva com ponto fixo em ¼ da trilha: bilhões acima,
  zero abaixo (doc 55).
- **Um filtro dentro de um gate é um buraco nele.** O guard que devia pegar o item acima
  estava verde porque filtrava `.starts_with("motion.")` — e os nós novos são `sim.*`.
  O nome do teste prometia "every node and param". Pergunte sempre **sobre o quê** um gate
  está verde.
- **Verde-de-compilação vale zero no audit.** As duas bombas do §5 passaram por 33 commits de
  `cargo check -p` verde.
- **O custo da Vello é por objeto de desenho**, não por vértice (doc 53).
