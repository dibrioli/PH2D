# HANDOFF DE INTEGRAÇÃO — `line/Painter` (2026-07-18)

> Para o **agente integrador**. Escrito conforme DIRETRIZ §1.5.9. A linha está **fechada e parada**; não
> integrou nem pushou nada.

## 1. Identidade

| | |
|---|---|
| Branch | `line/Painter` |
| HEAD | `71684821` |
| Base do fork (merge-base com `main`) | `cdc3acc1` |
| Commits | **6** |
| Diff total | 32 arquivos, +3737 / −973 |

```
71684821 test(sculpt): o harness de perf media custo UNICO como custo por move
08d76b4f perf(sculpt): a bola do Inflate para de percorrer o que nao pode contribuir
caccf9cf feat(impasto): a LUZ roda na GPU -- e com ela a pilha inteira de um desenho esculpido
ba03ed84 feat(sculpt): o footprint do Inflate virou um FECHAMENTO MORFOLOGICO
38e2bf61 docs(sculpt): a borda do Inflate FECHOU -- banner no handoff dedicado
e167926d feat(sculpt): o Inflate agora e a BOLA LIMITADA EXATA
```

`main` **não se moveu** desde o fork (merge-base = HEAD de `main` = `cdc3acc1`). Se outra linha integrar
antes desta, o `--ff-only` deixa de valer e o rebase passa pelos arquivos do §2.

## 2. Foundational / compartilhado tocado

14 dos 32 arquivos estão fora de `crates/ph2d-tool-painter/`. **Todos aditivos**; nenhum remove ou
renomeia símbolo existente.

| arquivo | o que | risco de colisão |
|---|---|---|
| **`CLAUDE.md`** | §5, entrada nova do Painter + 2 itens riscados na lista ABERTO | 🔴 **ALTO** — toda linha edita o §5 |
| `Cargo.lock` | +2 linhas (dev-deps de path) | 🟡 regenerável — descarte o lado deles e rode `cargo check` |
| `crates/ph2d-render/src/lib.rs` | `pub mod impasto_light;` + bloco `pub use` | 🟡 lista **alfabética** — outra linha exportando algo cai adjacente |
| `crates/ph2d-render/Cargo.toml` | +2 dev-deps de path (§5) | 🟡 mesma seção `[dev-dependencies]` |
| `crates/ph2d-render/src/impasto_light.rs` | **NOVO** (644 L) | 🟢 arquivo novo |
| `crates/ph2d-render/src/shaders/impasto_light.wgsl` | **NOVO** (227 L) | 🟢 arquivo novo |
| `crates/ph2d-render/tests/impasto_light_gpu.rs` | **NOVO** — gates de paridade GPU | 🟢 arquivo novo |
| `crates/ph2d-painter-brush/src/material.rs` | `+ SpecLut::table()` (aditivo) | 🟢 crate da família Painter |
| `shells/desktop/.../painter_gpu_preview.rs` | portão de elegibilidade + o passe de luz | 🟢 específico do Painter |
| `shells/desktop/.../painter_preview_handoff_tests.rs` | alavanca do gate trocada (§6) | 🟢 específico do Painter |
| `shells/desktop/.../push_look_probe.rs` | +cena 12 (sonda) | 🟢 específico do Painter |
| `docs/HANDOFF_line_Painter_*` (3 arq.) | 2 novos + 1 banner | 🟢 |

⚠️ **Não toquei** `ph2d-editor-core/src/`, `ph2d-core`, tokens, i18n, `IconId`, nem qualquer lista
gerada (`node-sync`/`tool-sync`/`chrome-sync`/`widget-sync`).

## 3. Símbolos novos (superfície de colisão para o grep de mesmo-símbolo)

**Nenhum id numérico, nenhum variant em enum compartilhado, nenhum token, nenhum `IconId`, nenhum
`NodeId`.** Só nomes de tipo, em crates que ninguém mais deveria estar editando:

```
ph2d_render::ImpastoLightPass          ph2d_render::ImpastoLightInput
ph2d_render::ImpastoLightError         ph2d_render::ImpastoLamp
ph2d_render::IMPASTO_MAX_LIGHTS = 4    (espelha impasto_rig::MAX_LIGHTS, pinado por gate)

ph2d_tool_painter::ImpastoLamp         ph2d_tool_painter::ImpastoPlanes
```

⚠️ **`ImpastoLamp` existe nos DOIS crates, de propósito.** Não é colisão (crates diferentes): o tipo do
`ph2d-render` é o que o shader consome, o do `ph2d-tool-painter` é o que o rig resolve. O shell importa os
dois e aliasa (`ImpastoLamp as GpuLamp` no gate). Se o integrador vir os dois nomes num grep, **não são
duplicata**.

Mudança de visibilidade (não é símbolo novo): `PainterTool::apply_impasto_light` passou de `pub(crate)`
para **`pub`** — é a passagem canônica contra a qual o gate de paridade GPU reconcilia.

## 4. Contratos congelados (§4)

**NENHUM encostado.** Verificado rodando os gates, não por memória:

| gate | resultado |
|---|---|
| `architecture_tool_contract_surface` (`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`) | ok, 4 |
| `architecture_contract_surface` (`NodeOp`/`OpResolver`/`NodeManifest`) | ok, 3 |
| `architecture_vector_contract_surface` | ok, 11 |
| `architecture_workspace_file_loc_cap` | ok, 2 |
| `architecture_adr_numbers_are_unique` | ok, 1 |

Nenhum ADR novo foi criado (⇒ **zero risco de colisão de numeração de ADR**, que já mordeu antes).

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

1. ⚠️ **2 dev-deps de path novas** em `crates/ph2d-render/Cargo.toml` (`ph2d-tool-painter`,
   `ph2d-editor-core`) → **`cargo machete`**. As duas SÃO usadas, mas só dentro de
   `tests/impasto_light_gpu.rs`; se o machete reclamar de dev-dep usada apenas em `tests/`, é este o
   ponto. Não há cycle (`ph2d-tool-painter` não depende de `ph2d-render`; confirmado por `cargo check`).
2. **`typos`** — introduzi termos técnicos que podem não estar na allowlist: `Gil-Werman`, `pdqsort`,
   `argmax`, `unorm`, `premultiply`, `naga`, `Felzenszwalb`. Docs e comentários trazem **português**.
3. **`cargo fmt --all`** — rodei `rustfmt` só nos arquivos que toquei (deliberado: `cargo fmt -p`
   reformata WIP alheio). Drift de fmt pré-fork em outros arquivos não foi verificado.
4. **RUSTSEC / `cargo deny`** — nenhuma dep EXTERNA nova (só path), então não espero advisory novo.
5. `clippy --workspace --all-targets` e `check --workspace --all-targets`: **0 issues** nesta árvore.

## 6. Ordem, dependências e o que smokar

**Ordem:** linear, sem cherry-pick. Cada commit compila e passa a suíte sozinho. `08d76b4f` e `71684821`
mexem no sculpt que `ba03ed84`/`e167926d` montaram — não reordene.

### ⚠️ Gates que o integrador NÃO vai executar por padrão

Todos os gates de GPU e de perf são **`#[ignore]`**. Precisam de **dispositivo GPU** e `--release`:

```
cargo test -p ph2d-render --test impasto_light_gpu -- --ignored --nocapture
cargo test -p ph2d-host-desktop --bins -- --ignored the_gpu_producer_shows the_screen_survives
cargo test -p ph2d-tool-painter --release -- --ignored --test-threads=1 --nocapture sculpt_perf_kill_criterion
```

Se a máquina de integração não tiver GPU, eles fazem `return` limpo (não falham) — **e a paridade
CPU/GPU deixa de ser verificada**. Nesta árvore, na RTX: paridade **byte-idêntica** (0 de 16384 bytes)
nos 5 materiais; e2e byte-idêntico; perf sob o kill.

### Já smokado pelo Enio (aprovado)
- `e167926d` + `ba03ed84` — bola limitada + fechamento morfológico + falloff Sphere por default.
- `caccf9cf` — a luz na GPU.
- `08d76b4f` — perf do Inflate (byte-idêntico por construção e por gate).

### NÃO smokado
- `71684821` — **só teste**, não muda produto: conserta o harness de perf (media custo único como custo
  por move). Nada para o artista ver.

### Vermelhos herdados, medidos, que NÃO são desta linha
Rodei contra o HEAD shipado (`ba03ed84`) em worktree separado antes de afirmar:
- `write_mobile_to_disk` (áudio) — sonda manual, exige `PROBE_OUT`.
- `watercolor_app_params_incremental_matches_full_{diluted,mixer_on}` — Δ2 stale, caminho não tocado.
- `sculpt_perf_kill_criterion` estava vermelho ANTES de `71684821` por medir custo único como por-move;
  o commit conserta a medida e **não** a barra (kill segue 8).

## 7. Duas coisas fora do git que o integrador precisa saber

1. **Memória:** escrevi 2 memórias novas + 3 edições de índice/tópico em
   `project-memory/`, que vive no **repo PRIMÁRIO** e está **uncommitted** lá. Não são deste branch e não
   viajam no merge. Arquivos: `feedback_a_magnitude_bound_misses_a_systematic_off_by_one.md`,
   `feedback_a_deferral_notes_bar_may_exceed_the_projects_policy.md`, + edições em `MEMORY.md`,
   `reference_topic_oracle_discipline.md`, `feedback_sed_relative_path_hits_primary_cwd.md`.
   ⚠️ O working tree do primário também tem pendências de **outras** sessões (`docs/Physics/`,
   `reference_topic_impasto_physics.md`, `feedback_a_token_rewrite_...md`) — **não são minhas, não
   clobbe**. `MEMORY.md` é lista compartilhada: só ADICIONE, nunca remova.

2. **Se este merge conflitar com outra linha em `CLAUDE.md` §5**, o resíduo é textual e a regra é a de
   sempre: as duas entradas ficam, nenhuma some. A minha é o bloco `**💡 A LUZ RODA NA GPU**` mais os dois
   itens riscados (`~~a borda do Inflate~~`, `~~passe de luz na GPU~~`) na lista ABERTO do Painter.

## 8. Detalhe técnico completo

- [`HANDOFF_line_Painter_gpu_light_2026-07-18.md`](HANDOFF_line_Painter_gpu_light_2026-07-18.md) — a luz
  na GPU + os 2 adendos (reach bound do Inflate; conserto do harness de perf).
- [`HANDOFF_line_Painter_inflate_closing_2026-07-18.md`](HANDOFF_line_Painter_inflate_closing_2026-07-18.md)
  — o fechamento morfológico.

---

**Linha `Painter` pronta (HEAD `71684821`, 6 commits). Aguardo ordem de integração — não integro nem
pusho.**
