# Registro da jornada de integração — 6 linhas → `main` (2026-07-13)

> Escrito pelo **agente integrador**, por ordem explícita do Enio (opção **B**: integrar as 6
> agora, registrando quais entraram **sem smoke**). DIRETRIZ §1.5.3–1.5.4.
>
> **A integração é `--ff-only`** (ADR-0107): não existe commit de merge onde anotar isto. Este
> documento É o registro.

## 1. ⚠️ O que entrou SEM smoke do Enio

O Enio **ordenou** a integração antes do smoke. Quatro das seis linhas **não foram vistas rodando**:

| Linha | Smoke | O que NÃO foi visto na tela |
|---|---|---|
| `line/motion-value` | ✅ **completo** | — (7/7 fatias aprovadas) |
| `line/audio-w3` | ⚠️ **parcial** | **W6** (Export Set / Delivery, 3 plataformas) · **ADR-0120** (preview incremental) |
| `line/FLIP` | ⚠️ **parcial** | **W7** (multiframe) · **W7.1** (Instance/Unlink) · **W7.2** (a pose do quadro) |
| `line/Vector` | ⚠️ **parcial** | **Blend** (interpolação de formas, crate `ph2d-vec-blend`) |
| `line/Painter` | 🔴 **ZERO** | **a linha inteira** — material per-pixel (Roughness/Metallic/Wax) + Sculpt (8 verbos) |
| `line/anim` | ⚠️ **parcial** | tudo menos Ctrl+S/Ctrl+O (o "Save/open OK") |

**Por que isto está escrito e não apenas dito:** a linha do Painter shipou, nesta MESMA jornada,
uma UI **morta sob o mouse** com todo gate verde, e um card com **bug de design pinado por um gate
verde**. Gate verde prova que o código faz o que você DISSE; nenhum gate diz que o que você disse
está errado. O que sobra é o olho do Enio.

**Cenas prontas para o smoke pós-integração:**

```bash
cd /home/enio/Documentos/Projetos/PH2D
PH2D_BUILD_SMOKE=7          cargo run --release -p ph2d-host-desktop   # Vector: Blend
PH2D_BUILD_SMOKE=6 PH2D_UNDO_LOG=1 cargo run --release -p ph2d-host-desktop  # Vector: undo/z
PH2D_AUDIO_DELIVERY_SMOKE=1 cargo run --release -p ph2d-host-desktop   # Audio: W6 Export Set
cargo run --release -p ph2d-host-desktop                               # Painter / FLIP / anim
```

## 2. Ordem de pouso (e o `PROJECT_SCHEMA` que se CONTOU)

`--ff-only` serializa; cada linha rebaseou sobre a main que a anterior deixou.

| # | Linha | `PROJECT_SCHEMA` | Nota |
|---|---|---|---|
| — | (main) | 7 | — |
| 1 | motion-value | 7 | não toca o schema |
| 2 | audio | 7 | não toca o schema |
| 3 | FLIP | **9** | +2: `FlipStroke.selected` (v8) · `FlipFrame.offset` (v9) |
| 4 | Vector | **10** | +1: `VecVertex.corner_radius` (v10) |
| 5 | Painter | **12** | +2: `mats` novo (v11) · `mats` mudou de FORMA (v12) |
| 6 | anim | **13** | +1: `ProjectFile.timeline` (v13) |

**O valor certo não existia em nenhum lado do conflito.** Cada linha trazia o SEU número
(anim 8 · Vector 8 · FLIP 9 · Painter 9); escolher qualquer um deles faria os saves das outras
passarem na checagem de versão e serem lidos com o layout errado — postcard é **posicional**, não
tem nome de campo para reclamar, e devolve lixo bem-formado. **Contou-se: seis quebras de layout
sobre o 7.**

Pin final (`shells/desktop/src/project_tests.rs`):
`(PROJECT_SCHEMA, FLIP_SCHEMA_VERSION, VEC_SCENE_SCHEMA_VERSION) == (13, 5, 8)`

## 3. Os 3 vermelhos que já estavam na `main` (nenhuma linha os causou)

Corrigidos em `f6deb815`, ANTES de integrar:

1. **`deny.toml`** — o ignore de `RUSTSEC-2023-0089` (atomic-polyfill) ficou **órfão**: o advisory
   saiu/mudou upstream, e o `cargo-deny` erra em ignore não-casado. Removido.
2. **`spin` 0.10.0 e 0.9.8 estavam YANKED** (`yanked = "deny"`). **Nenhum handoff reportou isto** —
   as 6 linhas o herdaram e nenhuma rodou `cargo deny`. `0.10.0` vem do `bevy_platform`; `0.9.8`
   vem do `heapless 0.7` ← `postcard`, a MESMA cadeia do atomic-polyfill. → 0.10.1 / 0.9.9.
3. **clippy**: cast `i32`→`i32` em `tests/spike/src/bin/c11_flecs.rs:64` (na main desde `cf62198e`).

## 4. 🔴 Os DOIS bugs que a INTEGRAÇÃO causou (ambos consertados)

**Os dois são da mesma família: `merge-tree` verde, árvore quebrada.** Nenhum deles produziu um
único conflito textual — foi preciso ir OLHAR.

### 4.1 — `ph2d-ui-testkit/Cargo.toml`: chave duplicada (fix `29d39365`)

A linha **Painter** (`MockPanelHost::click_at`) e a linha **motion-value**
(`dispatch_pointer_event`) adicionaram, **cada uma**, as deps `ph2d-host` + `bumpalo` ao mesmo
`Cargo.toml` — **em pontos diferentes do arquivo**. O git fundiu os dois lados sem um conflito, e
o resultado tinha:

* chave **duplicada** de `ph2d-host` e de `bumpalo` → **erro de parse de TOML**;
* o comentário do Painter **colado na mesma linha** do `ph2d-text` (sem newline).

Ficou UMA de cada. O `bumpalo` mantém `features = ["collections"]` (o do Painter): feature é
aditiva, então o superconjunto serve às duas linhas — o contrário quebraria o `click_at`.

### 4.2 — `MEMORY.md`: 4 linhas de índice apagadas (fix `9575ea54`)

O commit **`e7503e06`** da linha de áudio — *"o índice carregava 4 linhas de OUTRA linha"* —
removeu do `project-memory/MEMORY.md` **4 linhas de índice** que ela julgava alheias.

**Elas eram da `main`.** A branch forkou ANTES delas pousarem, então a "limpeza" foi feita contra
uma **foto velha** do índice. No rebase, a deleção aplicou **limpa** (a main de fato TINHA as
linhas) e elas sumiram de verdade:

```
feedback_inherited_affordance_must_be_rederived
feedback_a_click_is_a_press_that_drifted
feedback_a_mutation_that_survives_may_mean_a_missing_gate
feedback_absence_gate_needs_a_presence_sibling
```

Os 4 **arquivos** continuaram no repo. Sem linha de índice, viraram **memória morta** — nunca são
recuperadas. Restauradas em `9575ea54`.

**A lição:** uma branch não pode "consertar" um índice compartilhado a partir da própria base. A
única resolução correta de uma lista que **SOMA** é a **UNIÃO**, e ela só pode ser feita contra a
main de HOJE. (Ironia registrada: foi a linha de áudio que documentou essa armadilha no handoff
dela — e caiu nela.)

### 4.3 — E um erro MEU, na resolução (corrigido no próprio commit)

Ao resolver o `project.rs` do anim, removi os marcadores de baixo (`|||||||`/`=======`/`>>>>>>>`)
e **deixei o `<<<<<<< HEAD`** — que foi commitado. A árvore final ficava limpa (o commit seguinte
o removia), mas **um commit do histórico não compilava**. Corrigido no lugar via `rebase -i`
(`0bdf5091`), e não com um commit de conserto por cima: *árvore limpa não prova o histórico*
([[feedback_sweep_conflict_markers_every_commit]]).

Varredura final: **os 119 commits da jornada, zero marcadores.**

## 5. Órfãs PRÉ-EXISTENTES (não são desta jornada — decisão do Enio)

Arquivos de memória sem linha no índice, já assim antes da integração:

```
feedback_widget_is_done_when_a_test_clicks_it.md
project_brush_audit_2026_06_18.md
project_diretriz_v68_2026_05_22.md
project_painting_removed_layers_effects_kept.md
```

Ou se indexam, ou se apagam. **Não decidi por você.**

## 6. ADRs

`main` terminava em **0119**. Entraram **0120** (áudio — preview incremental) e **0121** (Vector —
Live Corners). **Sem colisão**: a linha do Vector contou as worktrees e cedeu o 0120 ao áudio.

**Pendente, decisão sua:** o **`ADR-0115` está duplicado NA MAIN** desde antes desta jornada
(`0115-audio-spectral-fft-via-realfft` vs `0115-clip-composition-sequencer-...`). O Vector trouxe o
gate `architecture_adr_numbers_are_unique`, que **pina a exceção e é auto-limpante** (fica vermelho
pedindo a remoção da allowlist no dia em que alguém renumerar). Recomendação herdada (linha anim):
renumerar **o do áudio** — chegou 11 minutos depois e tem 9 referências contra 36.
