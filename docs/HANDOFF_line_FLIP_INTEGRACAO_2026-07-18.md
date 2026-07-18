# Handoff de INTEGRAÇÃO — `line/FLIP` → `main` (2026-07-18)

> **Para o agente INTEGRADOR.** Ordem do Enio: integrar esta linha ao `main`.
> A linha está **fechada e smokada**; o implementador parou aqui (§0.7 do CLAUDE.md).
>
> **Base:** `Worktrees/line-FLIP`, branch `line/FLIP`, **14 commits à frente** do `main`.
> **`main` NÃO andou** desde o fork (`git rev-list --count HEAD..main` = **0**) ⇒ este é um
> **fast-forward limpo**, sem merge, sem resolução de conflito.

---

## 1. O comando (o caminho feliz)

```bash
cd /home/enio/Documentos/Projetos/PH2D           # árvore primária
git status --short                                # tem de estar limpa
git merge --ff-only line/FLIP
```

Se o `--ff-only` recusar, **PARE**: significa que a `main` andou depois desta escrita. Aí
vale a DIRETRIZ §1.5.5 — resolva pelos **ESTÁGIOS do índice** (`:1` base, `:2` ours, `:3`
theirs), nunca pelos marcadores, e rode `cargo check --workspace` depois (merge limpo pode
estar semanticamente quebrado). Os pontos de colisão prováveis estão na §4.

**Depois do merge, rode o ship COMPLETO** (`./scripts/ship.sh`) — o `nextest-impacted` já
teve false-green em RAM baixa, e esta rodada mexe em foundational.

---

## 2. O que este delta entrega (§4.C — 6 fatias, todas com smoke APROVADO pelo Enio)

| # | Commit | O que o usuário ganha |
|---|---|---|
| §4.C.1 | `a5738e98` | O **pedaço** é a unidade visual do modo Segment: halo por-peça (antes acendia a forma inteira) + **hover** em âmbar fraco. |
| §4.C.2 | `47fd348c` | **Duplicate Layer** — cópia independente acima da original, preservando a instância dentro da camada (ciclo continua ciclo) e o refcount. |
| §4.C.3 | `a4609669` | **Rename Layer** — double-click no nome abre campo inline (espelha o `marker_rename` do timeline); Enter/Blur commitam, Esc cancela. |
| §4.C.4 | `27144941` | **Raio/força próprios da borracha atrás de um LINK** por propriedade (*Unified Paint Settings* do Blender). Default LINKADO = comportamento histórico. |
| §4.C.5 | `d760c745` | **Borracha macia idempotente** (Strength virou a translucidez que sobra) + **cena de boot VAZIA** (os 8 sprites de teste saíram da Hierarquia). |
| §4.C.6 | `9b149bd8` + `1bc6599b` | **O Size mede o MUNDO** — a largura do traço deixou de ser relativa ao zoom + **Strength é Soft-only** + **bump de schema**. |

---

## 3. ⚠️ AS TRÊS COISAS QUE VOCÊ PRECISA SABER (não são detalhe)

### 3.1 O `PROJECT_SCHEMA` subiu 15 → 16: **projetos antigos não abrem mais**

`FLIP_SCHEMA_VERSION` **7 → 8** · `PROJECT_SCHEMA` **15 → 16** · pin em
`project_tests.rs` **(16, 8, 8)**.

O motivo é sutil e vale ler: o §4.C.6 trocou a **UNIDADE** do `Point.width` (px de tela →
mundo) **sem mexer no layout** (segue um `f32`). Sem o bump, o postcard leria um projeto v15
**com sucesso** e desenharia a arte ~100× mais grossa — corrupção **silenciosa**. Todos os
bumps anteriores quebravam layout, que falha alto; este quebra significado, que falha calado.

O `load` recusa com mensagem clara (`Project refused: file format 15, this build reads 16`).
**Não há migração** — seria a primeira do projeto, e é decisão de superfície do Enio, não
contrabando dentro de um fix. Se ele tiver arte salva que importe, o caminho é dividir os
`width` por `SIZE_PX_PER_WORLD` ao ler um v15 (~5 linhas).

> **Se outra linha também bumpou schema nesta jornada:** o valor certo **não está em nenhum
> dos dois lados — ele se CONTA** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
> Conte o `PROJECT_SCHEMA` contra a `main` **do dia** e reconcilie a tripla do pin junto.

### 3.2 Uma decisão do Enio de 2026-07-11 foi REVERTIDA (por ordem dele)

O pincel do Flip era **absoluto em px de tela**; agora o **Size mede o mundo**. Isso estava
documentado em 4 arquivos e todos foram atualizados. Se você encontrar, no merge, comentário
de outra linha afirmando a lei antiga, **a lei nova é esta** (§4.C.6, `size_to_world`,
`SIZE_PX_PER_WORLD = 100`). O renderer sempre quis assim (`thickness_px = raio_mundo ·
px_per_world`); o `camera_raw` é que passava `1.0` para forçar a leitura em tela.

### 3.3 Testes que codificavam as leis antigas foram CORRIGIDOS, não silenciados

7 testes mudaram de expectativa (5 do anel do cursor, 1 do raio do sculpt, 2 do Edit, 1 do
soft-erase). Se algum deles conflitar no merge, **o lado desta linha é o correto** — cada um
tem no docstring por que a premissa antiga era o próprio bug escrito como teste.

---

## 4. Sítios FOUNDATIONAL e de colisão provável

**`ph2d-editor-core` (append-only, 2 arquivos):**

- `src/ids/chrome/flip.rs` — **8 ids novos**, todos apendados:
  `FLIP_LAYER_DUPLICATE` · `FLIP_LAYER_RENAME_INPUT` · `FLIP_LINK_SIZE` ·
  `FLIP_LINK_STRENGTH` · `FLIP_ERASE_SIZE` (+`_NUM`) · `FLIP_ERASE_STRENGTH` (+`_NUM`).
  Se outra linha apendou id de chrome, **quem fala é o gate `node_id_collisions`**.
- `tests/architecture_panel_wiring_parity.rs` — **1 entrada** no `HIT_PARITY_ALLOW`:
  `("ph2d-panel-flip", "FLIP_LAYER_RENAME_INPUT")`, ao lado das gêmeas do timeline e da
  hierarchy (campo de rename dinâmico, registrado no paint e não no `populate`).

**Shell fora do namespace `flip_*`** (onde outra linha pode ter tocado):

| Arquivo | O que esta linha mexeu |
|---|---|
| `src/init.rs` | removeu a chamada do `populate_sim_live` (cena de boot vazia) |
| `src/sim_populate.rs` | **deletou** o `populate_sim_live`; o `populate_sim` (Vogel 1000, `PH2D_M5_DEMO=1`) FICA |
| `src/input_dispatch/keyboard.rs` | +1 guarda: o Delete/Backspace do Edit Mode cede a campo de texto focado |
| `src/render_loop/mod.rs` | +1 argumento na chamada do `draw_flip_cursor` (o `px_per_world`) |
| `src/app_state.rs`, `src/main.rs` | campo do hover do Segment (§4.C.1) |
| `src/project.rs`, `src/project_tests.rs` | o bump de schema da §3.1 |

**`.typos.toml`** — **1 palavra pt-BR** apendada: `deram = "deram"`. ⚠️ Se outra linha
adicionou a mesma, **funda sem duplicar**: chave duplicada mata o gate **no parse** e nada
mais é escaneado ([[feedback_duplicate_allowlist_key_kills_the_gate_at_parse]]).

**Arquivos NOVOS (3)** — nascem sem conflito:
`crates/ph2d-panel-flip/src/paint_rows.rs` (split de LOC) ·
`shells/desktop/src/flip_erase_tests.rs` (split de LOC) ·
`shells/desktop/tests/the_eraser_uses_the_erasers_own_numbers.rs` (arch-gate novo).

---

## 5. O que rodar depois do merge (e o que cada um pega)

```bash
cargo test -p ph2d-flip -p ph2d-tool-flip -p ph2d-panel-flip -p ph2d-host-desktop
cargo test -p ph2d-editor-core --test node_id_collisions \
  --test architecture_panel_wiring_parity --test no_magic_numeric \
  --test architecture_panel_loc_cap --test architecture_workspace_file_loc_cap
cargo test -p ph2d-host-desktop --test file_loc_caps
./scripts/ship.sh          # paridade com o CI — é ele que decide
```

Contagens desta linha, isolada, para você comparar: **shell 707** · **panel seam 20** ·
**tool 14** · **flip 101**, zero falha; clippy `--all-targets` limpo; typos limpo (repo
inteiro); LOC caps verdes; release builda; boot sem panic.

Gates **novos** que esta linha traz (se algum falhar depois do merge, é regressão real, não
ruído): `the_eraser_uses_the_erasers_own_numbers` (arch-gate) ·
`the_stroke_thickness_is_fixed_in_the_world_and_scales_with_the_zoom` ·
`the_ring_follows_the_zoom` · `the_strength_row_lives_only_in_the_soft_eraser` ·
`the_soft_erase_is_a_fact_of_the_path_not_of_the_dab_count` ·
`strength_is_the_translucency_that_remains` ·
`an_unlinked_eraser_paints_its_own_slider_and_a_linked_one_paints_the_brushs`.

---

## 6. Estado depois de integrar

- **Todas as waves do plano estão fechadas** (WT, W0–W5, W7) menos a **W6 (timeline
  global)**, que segue **ADIADA por ordem do Enio** até ele declarar a timeline principal
  fechada — e que exige coordenação com o dono dela (`PropKind` é enum fechado).
- **Sem bug aberto** no `docs/Flip/BUGS_flip.md`.
- A próxima fase recomendada (e já aceita pelo Enio em conversa) é a wave **Colorize**
  (LazyBrush + trapped-ball + *onion fill*), especificada em `docs/Flip/04 §3`.
- O tracker vivo da linha é
  [`HANDOFF_line_FLIP_CONTINUACAO_2026-07-17.md`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-17.md)
  (§1 tem o detalhe de cada fatia; §4/§5 têm a fila e o que sobrou).

**Você integra e PARA.** O ship/push é ordem separada do Enio (§0.7).
