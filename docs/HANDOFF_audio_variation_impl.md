# HANDOFF de integração — Áudio W6 · Variation containers (`line/audio`)

> DIRETRIZ §1.5.9. A linha fechou o bloco, comitou local e **PAROU** — não integra
> nem faz ship (Enio-only, via integrador dedicado). Módulo tracker vivo:
> [`HANDOFF_audio_module.md`](HANDOFF_audio_module.md). Plano: [`Audio/02_plano_implementacao_completo.md`](Audio/02_plano_implementacao_completo.md) §W6.

## 1. Identidade
- **Branch:** `line/audio` · **HEAD:** `ecd2587a` · **merge-base com main:** `1c7c9a22` (= HEAD do main integrado).
- **Commits à frente do main:** **1** — `ecd2587a feat(audio): W6 variation containers — random/sequence/shuffle + jitter + weights`.
- Árvore limpa. Fast-forward puro sobre o main atual (a linha foi resetada ao main recém-integrado antes de começar).

## 2. Foundational / compartilhado tocado (e por quê)
Só **um** arquivo fora das crates do módulo de áudio, e **aditivo**:
- **`shells/desktop/src/render_loop/mod.rs`** — um bloco novo dentro da seção `#[cfg(feature="panel-audio-editor")]` que já existe (a ponte de áudio por-frame), logo após o bloco de Markers. Drena os intents de variação (`ed::take_add_variation`/`take_remove_variation`/`take_play_variation`/`take_save_variation_set`/`take_load_variation_set`/`take_strategy_step`/`take_weight_step` + `ed::variation_sel`/`pitch_jitter_norm`/`gain_jitter_norm`) e publica de volta `set_variation_names`/`set_strategy_name`. **Nenhuma assinatura existente mudou; puro append.** (Este arquivo tem `// ph2d-loc-cap:` — segue exempto do cap de 600.)

Os outros 2 arquivos de shell tocados são do **próprio módulo de áudio** (não compartilhados por outra linha): `shells/desktop/src/audio/editor.rs` (3 campos novos em `AudioEditorRuntime` + `mod variation;`) e `shells/desktop/src/audio/editor/loops.rs` (1 linha: chama `editor_variation_smoke()` no fim do smoke existente). O resto é 100% dentro de `crates/ph2d-audio-edit/` e `crates/ph2d-panel-audio-editor/` (as crates do módulo).

## 3. Símbolos novos que poderiam COLIDIR (grep de mesmo-símbolo, §1.5.5)
- **NENHUM `NodeId(NNN)` inteiro alocado.** Todos os 23 ids novos do painel são `hash_node_id("audio_editor_var_*")` — strings namespaced pelo prefixo `audio_editor_var_`, definidas **na crate do painel** (`ph2d-panel-audio-editor/src/lib.rs`), não em `editor-core`. Colisão com outra linha exigiria a mesma string literal — praticamente impossível fora do módulo de áudio. (Contraste com a lição `NodeId(832)`/`AUDIO_EDITOR_SCROLLBAR_ID=NodeId(834)`: **este bloco NÃO alocou nenhum id foundational de `editor-core`.**)
- Consts novas, todas locais às crates do módulo: `MAX_VARIATIONS = 12` (painel + um espelho local no shell, com comentário), `WEIGHT_RANGE`/`MAX_JITTER` (exportadas de `ph2d-audio-edit`), `MAX_PITCH_JITTER_ST`/`MAX_GAIN_JITTER_DB = 12.0` (shell). Sem entrada em lista ordenada compartilhada, sem token novo, sem variant de enum foundational.
- Tipos/APIs públicos novos em `ph2d-audio-edit`: `PickStrategy`, `Variation`, `VariationSet`, `VariationPicker`, `Jitter`, `WEIGHT_RANGE`, `parse_variation_set`, `serialize_variation_set` — aditivos (novo módulo `variation`), não mexem em `EditClip`/`ops`/`fx`.

## 4. Contratos congelados (§4)
**Nenhum encostado.** Áudio não adiciona `Tool`/`Node` gateado; `SCHEMA_VERSION` intacto (o manifesto de variação é um arquivo-texto próprio `.txt`, **não** o save do projeto). Nenhum ADR necessário.

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda) — [[project_integration_prefork_lines_ship_drift]]
- **fmt:** rodei `rustup run 1.95 rustfmt --edition 2024 --check` em **todos** os 13 arquivos → canônico (exit 0). Sem fmt-skew esperado.
- **Deps novas:** **ZERO** (variação é Rust puro; sem crate nova) → machete/deny/audit não têm o que reclamar deste bloco.
- **clippy:** rodei `cargo clippy --all-targets` em `ph2d-audio-edit`, `ph2d-panel-audio-editor` e `ph2d-host-desktop` → limpo. **NÃO** rodei `clippy --workspace --all-targets` (o ship roda; se acusar latente cross-crate, é pré-existente, não deste bloco).
- **typos:** não rodei o typos-cli; texto en-US canônico.

## 6. Ordem / dependências / smoke
- **1 commit, sem ordem interna.** Fast-forward direto.
- **Gates verdes rodados (1× no fechamento, sobre o diff):** `ph2d-audio-edit` lib **86** (11 do modelo de variação) · painel lib **19** (5 de `variation_state`) + seam **27** (6 de variação) · `ph2d-editor-core --tests` **32 suites arch-gate** (wiring parity, no_literal_color, no_magic_numeric, clamp, **panel fn-LOC cap**) · shell `--tests` **6 suites** (file_loc_caps etc.) · clippy limpo · fmt canônico.
- **Smoke MANUAL do Enio pendente** (não tenho display p/ o GUI). Turnkey:
  ```
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && PH2D_AUDIO_LOOP_SMOKE=1 cargo run -p ph2d-host-desktop
  ```
  Abra o pill **Audio Editor** → seção **Variations** já vem com 4 blips (C-E-G-C, em `$TMPDIR/ph2d_variation_smoke/`). Aperte **Play Variation** repetido: **Shuffle** nunca repete o mesmo em seguida; troque a estratégia no `◀ ▶` (Random/Sequence/Shuffle); suba **Pitch jitter**/**Gain jitter** e cada toque varia; **Weight ×2** numa linha selecionada a torna mais provável; **Save…**/**Load…** gravam/leem o manifesto `.txt`. **Veredito: APPROVE pending smoke.**

## 7. O que landou (resumo p/ o tracker)
Container de variação estilo Wwise/FMOD, **autorado + auditado + salvo** no painel do Audio Editor (o caminho de trigger em runtime segue **bloqueado** — sem tick de script por-frame; por isso a **audição é o consumidor vivo** e o **manifesto** é o entregável persistido, mesma forma dos presets de FX-chain — NÃO virou entidade ECS / asset novo, o que seria fio órfão).
- **Modelo puro** `ph2d-audio-edit/src/variation.rs`: `PickStrategy{Random,Sequence,Shuffle}`, `VariationSet{entries,strategy,pitch/gain_jitter}`, `VariationPicker` (splitmix64, pick ponderado, shuffle avoid-repeat, jitter `2^(±st/12)`/`10^(±dB/20)` via `exp2` — HR-5 não vale aqui, é control-thread), manifesto texto tolerante (`serialize`/`parse`, keyed-by-content, pula lixo).
- **Painel** (`variation_state.rs` + `paint_variation.rs`, UI-only + thread-local bridge): lista selecionável · seletor de estratégia · Add/Remove/Play · Weight ÷2/×2 · sliders Pitch/Gain jitter · Save/Load. `apply_event` ganhou `variation_click`; extraí `edit_cmd_for` p/ manter `apply_event` sob o cap de 200 LOC/fn (fmt re-expandiu → 207).
- **Shell** (`audio/editor/variation.rs`): dona `VariationSet` + cache de clipes decodados (index-aligned) + `VariationPicker`; `editor_play_variation` toca o pick com jitter pela **preview voice** (borrow transiente — a audição é one-shot, não mexe no transporte do clipe carregado). Smoke `editor_variation_smoke` semeia 4 blips via `write_wav`.
- **Aberto no W6:** export OGG/Opus (dep + ADR) · import por convenção. Follow-ups da variação: enable-toggle por-entry na UI (o modelo/manifesto já carregam `enabled`, só falta o botão); overlay não desenha o set (é set de arquivos, não timeline — proposital).

*"Linha `audio` pronta (HEAD `ecd2587a`, 1 commit). Handoff de integração acima. Aguardo ordem de integração."*
