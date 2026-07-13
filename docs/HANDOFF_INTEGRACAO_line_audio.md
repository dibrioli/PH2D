# Handoff de integração — `line/audio` (DIRETRIZ §1.5.9)

> **Status:** linha FECHADA. Não integrei, não pushei, não rodei `ship.sh`. Aguardo ordem do Enio.

---

## 1. Identidade

| | |
|---|---|
| **Branch** | `line/audio` |
| **HEAD** | `77fdcc0eda36e83045024fed0ccf1aec30bae9ef` |
| **Base do fork** (merge-base com `main`) | `3805f650a4443decd6d418f4f24d6e71fdbe2bcd` |
| **Commits** | **51** |
| **Worktree** | `Worktrees/line-audio/`, árvore limpa |

`main` **não andou** desde o fork (o tip de `main` *é* a merge-base) — se seguir assim, a
integração é `--ff-only` trivial. Se `main` andar, o único ponto de atrito real está no §3.

---

## 2. Foundational / compartilhado tocado (fora de `crates/ph2d-audio*`)

### 2.1 `crates/ph2d-audio/` — o **mixer** (é foundational de fato: o runtime que embarca no jogo)

Duas mudanças de superfície, **ambas aditivas**:

- **`PlayParams` ganhou um campo apendado** (`loop_region: Option<LoopRegion>`) — ADR-0119.
  Toda construção via `..PlayParams::default()` segue compilando. **Não é contrato congelado**
  (§6 não lista `ph2d-audio`).
- **Tipo novo `LoopRegion`** + comando novo `AudioCommand::SetPreviewLoopRegion` (variant apendado).
- **Módulo novo `stream.rs`** (ADR-0118) — `Chunk`/`StreamHandle`/`StreamFeeder`/`stream()`.

Gates que provam que nada existente mudou de som: `loop_regions.rs::a_loop_without_a_region_is_
byte_identical_to_the_old_whole_buffer_loop` + os 6 de `streaming_sounds_identical.rs`.

### 2.2 `shells/desktop/` — sites compartilhados

| Arquivo | Natureza | Risco de colisão |
|---|---|---|
| `src/app_state.rs` | **NÃO-aditivo**: `audio_sel_drag: Option<u64>` → `Option<(u64, f32)>` (a âncora ganhou frequência, p/ a caixa do espectrograma) | baixo (campo só-áudio) |
| `src/input_dispatch.rs` | `+144/-11` — press na waveform ramifica na ferramenta armada; drag de peça; release commita | **médio** — outra linha que mexa no dispatch de ponteiro colide aqui |
| `src/input_handlers.rs` | `+41/-0` — Ctrl+X/C/V com ownership no **guard do match** (há copy/paste vetorial no mesmo atalho) | **médio** — mesma razão |
| `src/render_loop/mod.rs` | `+48/-19` — bridge por-frame do editor de áudio | baixo (bloco próprio) |
| `src/render_loop/audio_overlay.rs`, `audio_pieces.rs`, `audio_spectrogram.rs` | overlay de áudio | nenhum |
| `Cargo.toml` (shell) | deps opcionais novas + feature `panel-audio-editor` estendida | baixo |

### 2.3 Outros

- **`Cargo.toml` (workspace)**: 4 entradas `[profile.dev.package.*]` **aditivas** (`ph2d-audio-spectral`,
  `ph2d-audio-edit`, `realfft`, `rustfft` em `opt-level = 2`) — a DSP a opt-0 é 15-25× mais lenta e faz
  um smoke reportar a feature como quebrada. `members` é glob (`crates/*`), então **crate nova não
  edita o `Cargo.toml`**.
- **`crates/ph2d-ui-testkit/src/lib.rs`**: **aditivo** — `MockPanelHost::store()` (view read-only, p/
  um seam test assertar o que `populate` semeou). Nada muda de forma.
- **`crates/ph2d-editor-core/tests/hr15_no_hardcoded_ui_strings.rs`**: **1 linha** — a allowlist
  apontava `ph2d-panel-audio-editor/src/paint.rs`; a string se mudou p/ `paint_sections.rs` quando o
  painel virou seções colapsáveis. **Mesma string, arquivo novo.**
- **`CLAUDE.md` §5** e **`SKILL_Stack`**: entradas de estado do módulo de áudio.

### 2.4 Crates NOVAS (3) — drop-in, zero edit central

`ph2d-audio-spectral` (ADR-0115) · `ph2d-audio-opus` (ADR-0116) · `ph2d-audio-stream` (ADR-0118).

---

## 3. Símbolos que podem COLIDIR (grep do integrador — §1.5.5)

**Nenhum id foundational.** `crates/ph2d-editor-core/src/` **não foi tocado** — zero `ids::*` novos,
zero tokens, zero `IconId`, zero chave de i18n.

- **28 ids `AEDIT_*` novos**, todos `hash_node_id("audio_editor_*")` — namespace do painel, colisão
  só com outra linha que também mexa no painel de áudio (não existe).
- **`ph2d-audio`**: `LoopRegion`, `AudioCommand::SetPreviewLoopRegion`, `PlayParams.loop_region`,
  `stream::{Chunk, StreamHandle, StreamFeeder, stream, STREAM_CHUNK_FRAMES, STREAM_DEPTH}`.
- **`ph2d-audio-edit`**: `boundaries`, `ranges`, `bake_loop_crossfade`, `loop_seam_step`, `conform`,
  `insert`, `split_at`. **Removidos:** `crossfaded_loop`, `EditClip::loop_audition_buffer` (a
  fabricação de preview do ADR-0119). Se alguma linha os usar, quebra — **nenhuma usa** (grep).
- **Renomeado:** `snapshot::{request_split, take_split}` → `{request_export_pieces, take_export_pieces}`.

### ⚠️ O ponto de atrito real, se `main` andar

`shells/desktop/src/input_dispatch.rs` e `input_handlers.rs`. Qualquer linha que mexa em **dispatch
de ponteiro** ou em **atalhos Ctrl+X/C/V** encosta nos mesmos símbolos. O `Ctrl+X/C/V` de áudio
decide posse **no guard do match** (`audio_editor_owns_clipboard()`), calculado antes do borrow de
`&mut gfx` — se o vetorial mexer no mesmo arm, é merge manual, não sintático.

---

## 4. Contratos congelados (§6)

**NENHUM encostado.** Confirmado por grep: `ph2d-nodegraph`, `ph2d-tool-traits`, `ph2d-vector-doc`,
`ph2d-vector-traits` intactos. Os arch-gates de contrato **passam: 18/18** (`architecture_contract_surface` ·
`architecture_tool_contract_surface` · `architecture_vector_contract_surface`) — rodados, não presumidos.

`ph2d-audio` **não é** contrato congelado — por isso `PlayParams` pôde crescer sem ADR de contrato
(mas tem ADR próprio: 0117/0118/0119).

---

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

**Rodei tudo isso na worktree, e está verde** — mas o integrador deve **re-rodar sobre a árvore
combinada**, porque é aí que o drift aparece ([[project_integration_prefork_lines_ship_drift]]):

| Gate | Estado aqui |
|---|---|
| `cargo nextest run --workspace` | ✅ **5786/5786**, 77 skipped |
| `cargo clippy --workspace --all-targets` | ✅ **0** |
| `rustup run 1.95 cargo fmt --all -- --check` | ✅ limpo |
| `typos` | ✅ limpo |
| `cargo machete` | ✅ limpo |
| `cargo deny check` | ✅ advisories/bans/licenses/sources **ok** |
| build `--release` do shell | ✅ |

### ⚠️ DEPS EXTERNAS NOVAS — `deny`/`audit`/`machete` só as veem no ship

`realfft` · `rustfft` (+ `primal-check`, `strength_reduce`, `transpose`) · `ogg` · `unsafe-libopus`.

- ADRs: [0115](architecture/decisions/0115-audio-spectral-fft-via-realfft.md) (realfft) ·
  [0116](architecture/decisions/0116-audio-export-opus-isolated-unsafe-crate.md) (opus).
- `unsafe-libopus` é ABI transpilada com `unsafe` — **isolada** em `ph2d-audio-opus`, e
  `ph2d-audio-encode` mantém `#![forbid(unsafe_code)]` (gate mutation-tested).
- **Nota pré-existente (não é minha):** `deny.toml` tem um ignore obsoleto (`RUSTSEC-2023-0089` — "no
  crate matched advisory criteria"). Não toquei; vale limpar num ship.

### ⚠️ Achado ESCREVENDO este handoff

**4 gates de arquitetura estavam vermelhos** e eu não sabia: eles moram em `ph2d-editor-core`, então
`cargo test -p <minhas crates>` **nunca os rodou**. Só o nextest do **workspace** pega. Corrigidos
por **split, não allowlist** (commit `77fdcc0e`): `snapshot.rs` 663→276 (+`snapshot/fx.rs`),
`populate` 208→ok (+`register_buttons`), `apply_event` 212→ok (+`rack_click`), e um `3.0` mágico
virou `TOOL_COLS`.

**Recomendação ao integrador:** rode `cargo nextest run --workspace` (não `-p`) na árvore combinada.

---

## 6. Ordem, dependências e o que smoke-testar

### Ordem

Os 51 commits são **sequenciais e coerentes** — cada um compila e passa os gates. Não há dependência
entre linhas. **Se `main` não andou, é `--ff-only` direto.**

### O que o Enio JÁ smokou (OK)

W5 espectral · Opus · memória (ADR-0117) · streaming (ADR-0118) · W2 clipboard · cortes/peças
(Move/Scale).

### ⚠️ O que NÃO foi smokado — **regiões de loop (ADR-0119), a última feature**

```bash
cd Worktrees/line-audio && PH2D_AUDIO_LOOP_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

1. Abrir o Audio Editor → **Loop** + **Play**: o começo toca **uma vez**, depois só o miolo, para
   sempre. Deve **estalar** no wrap (a região é desalinhada de propósito).
2. Subir o **Crossfade** → clicar **Crossfade Loop**: o estalo some **na forma de onda**. Ctrl+Z traz
   de volta.
3. **Export** um WAV → **Load** de volta: o loop e os markers voltam com o arquivo.

### O que NÃO é gateável headless (verificar no smoke)

- **A fiação do gesto de peça no shell** (press→grab→drag→release): `AudioSystem::new()` precisa de
  device de áudio, e nenhum teste em `shells/desktop/tests/` constrói um. O **modelo** tem 18 gates;
  o **gesto** é smoke-only.
- **Áudio saindo do device**: idem.

---

## 7. Resumo (§1.5.9)

> Linha `audio` pronta (HEAD `77fdcc0e`, **51 commits**, base `3805f650` = tip de `main`).
> **Foundational tocado:** `ph2d-audio` (`PlayParams` +1 campo apendado, `LoopRegion`, `stream.rs`);
> `shells/desktop` (input_dispatch/input_handlers = **o único atrito real**); `ph2d-ui-testkit` (+1 fn
> aditiva); `Cargo.toml` (4 profiles aditivos); 1 linha de allowlist do gate HR-15.
> **Contratos congelados: nenhum.** **Símbolos novos:** 28 ids `AEDIT_*` (namespace do painel) + a
> superfície de `ph2d-audio` acima; **zero** ids/tokens/ícones/i18n foundational.
> **Só o ship pega:** 6 deps externas novas (realfft/rustfft/ogg/unsafe-libopus + 3 transitivas) →
> `deny`/`audit`/`machete`. **Rode `nextest --workspace`, não `-p`** — 4 gates de arquitetura moram em
> `ph2d-editor-core` e escapam do gate por-crate (foi assim que os achei).
> **Estado:** workspace 5786/5786, clippy 0, fmt/typos/machete/deny limpos, release builda.
> **Aguardo ordem de integração.**
