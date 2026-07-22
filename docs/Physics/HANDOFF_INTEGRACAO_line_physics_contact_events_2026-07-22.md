# HANDOFF DE INTEGRAÇÃO — `line/physics` · W-ContactEvents (2026-07-22)

> Para o **agente integrador**, sob ordem explícita do Enio (DIRETRIZ §1.5.9).
> A linha **não** integrou, **não** pushou e **não** rodou `ship.sh`.

## 1 — Identidade

| | |
|---|---|
| Branch | `line/physics` |
| HEAD | `3cfd5c5d3` |
| Base (merge-base com `main`) | `13a04c7aa` |
| Commits à frente | **2** — `660af8fdc` (o handoff de reabertura, doc-only) + `3cfd5c5d3` (a wave) |
| Rebase | feito no início da jornada; era **no-op** (a linha já estava em cima do `main`) |

## 2 — Foundational / compartilhado tocado, e por quê

| Arquivo | O quê | Risco de merge |
|---|---|---|
| `CLAUDE.md` §5 | **UMA frase apendada** ao fim do bloco *"Estado no main: …"* da física | ⚠️ **É o único ponto quente.** Estritamente **ADITIVO** (nada removido, nada reescrito), então funde por adição; se outra linha tocar o mesmo parágrafo, mantenha **as duas** |
| `shells/desktop/src/main.rs` | 1 linha: `mod physics_smoke_events;` | trivial (lista ordenada) |
| `shells/desktop/src/physics_smoke.rs` | 1 braço: `"29" => self.physics_smoke_events(),` | trivial (append no `match`) |
| `shells/desktop/src/render_loop/physics_overlay.rs` | +1 laço de stroke (o flash) + 2 nomes no `use` | baixo |
| `docs/Physics/00_plano_waves.md` | +1 linha na tabela de waves | trivial |
| `docs/Physics/HANDOFF_line_physics.md` | +1 seção **no fim** | trivial (é o tracker DESTA linha) |

Tudo o mais é **da pasta do módulo** (`crates/ph2d-physics-ecs/`, `render_loop/physics_overlay_contacts*`, `physics_smoke_events.rs`).

## 3 — Símbolos NOVOS (o que grepar por colisão de mesmo-nome)

**Nenhum id de UI, nenhum token, nenhuma const de registro** — esta wave não acrescenta
controle de painel. Os nomes novos são todos de código, e todos dentro da física:

- `ph2d_physics_ecs::ContactEvent` · `ph2d_physics_ecs::ContactPhase` (`Began` / `Ended`) — exportados no `lib.rs`
- `BodyContact.age_ticks: Option<u64>` — **campo NOVO num tipo público de plain-data** (⚠️ quem constrói um `BodyContact` literal precisa dele; hoje há **um** sítio, `physics_overlay_contacts_tests.rs`)
- `PhysicsBridge::contact_events()` · `discard_contact_history()` (pub(super))
- `bridge::contacts::ContactMemo` (pub(super)) · campos `contact_since` / `contact_events` / `contacts_continuous`
- **módulo novo** `crates/ph2d-physics-ecs/src/bridge/rewind.rs` (`mod rewind;` no `bridge.rs`)
- shell: `contact_flashes` · `CONTACT_FLASH_RGBA` · `FLASH_TICKS` / `FLASH_MIN_PX` / `FLASH_MAX_PX` · `App::physics_smoke_events`
- teste novo: `crates/ph2d-physics-ecs/tests/contact_events.rs`

**Números que se CONTAM e NÃO se escolhem — esta wave não move nenhum:**
`PROJECT_SCHEMA` fica **29** · registro de componentes fica **18** · `physics_ecs_c9` fica em
**75 corpos**. Se outra linha bumpar o schema, **não há nada meu para somar**.

## 4 — Contratos congelados encostados

**Nenhum.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` intactos. Nenhum ADR novo (a wave não muda o norte do ADR-0131 — é integração
e leitura, não solver).

## 5 — O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **Deps novas: ZERO** ⇒ `machete` / `deny` / `audit` não têm o que reclamar por minha causa.
- `fmt` rodado com **`rustup run 1.95 cargo fmt -p`** nas duas crates (só os meus arquivos, para não reformatar WIP alheio).
- `clippy --all-targets` rodado nas duas crates tocadas: **limpo** (a única lint, `redundant_closure` num teste meu, foi corrigida).
- `typos`: limpo **do jeito que o `ship.sh` o invoca** (sem args, config `.typos.toml`). ⚠️ Invocá-lo com paths explícitos acusa palavras em PORTUGUÊS pré-existentes no `CLAUDE.md` — não são minhas e não são regressão.
- **Não rodei** `cargo check --workspace` — as duas crates tocadas compilam com `--all-targets`, mas o gate da árvore COMBINADA é seu (`foundational-integrate.sh`), e `BodyContact` ganhou campo: se outra linha construir esse literal, aparece só lá.

## 6 — Gate batched desta linha (tudo verde)

- `cargo test -p ph2d-physics-ecs -p ph2d-physics` — **63 grupos, 0 falhas** (debug) · **35 grupos, 0 falhas** (release)
- `cargo test -p ph2d-host-desktop` — **0 falhas** (suíte inteira da shell)
- `file_loc_caps` (shell, HR-18) · `architecture_workspace_file_loc_cap` · `arch_safe_clamp_only` · `every_physics_component_is_authorable` · `architecture_panel_wiring_parity` — **todos verdes**
  *(os três primeiros entram aqui de propósito: o tracker registra que waves anteriores os deixaram vermelho-latentes)*
- **13 gates novos** (9 no kernel + 4 no overlay), **8 mutações, 8 sangram**

⚠️ **Rodei debug E release.** A `line/FLIP` documentou que só-release esconde pânico; aqui
não havia nenhum, mas a disciplina fica.

## 7 — O que SMOKE-TESTAR (nada foi smokado por mim — só gates)

**`env PH2D_PHYSICS_SMOKE=29 cargo run -p ph2d-host-desktop --release`**

A cena imprime os números medidos. O que conferir, em ordem:

1. **A bola (esquerda)** pisca um `×` que abre e some a cada pouso; entre um quique e outro a cruz `+` **desaparece**.
2. **A caixa (meio)** pisca **uma vez** e a cruz fica **para sempre** — é o contraste flash=EVENTO × cruz=ESTADO.
3. **A pilha (direita)** pisca **uma vez, no primeiro tick**, e depois silêncio. É deliberado (leitura da Unity).
4. ⚠️ **Arraste a régua para TRÁS:** as cruzes mudam e **NADA pisca**.
5. ⚠️ **Desmarque `Physics` na barra:** as cruzes **SOMEM**; ao remarcar, voltam **sem piscar**. *(Este é o bug vivo que a wave consertou — antes elas ficavam.)*

**Regressão a olhar de carona:** `=25` (W-Contacts) deve estar **inalterada** exceto pelo
flash agora aparecer nos primeiros ticks; e `=7` (Bake) exercita o `hold`, que mudou.

## 8 — Ordem e dependências

Commit único, sem dependência de outra linha. Se `line/Painter`, `line/anim` ou
`line/Vector` entrarem na mesma janela, o **único** ponto de encontro é o parágrafo do
`CLAUDE.md` §5 — resolva mantendo **as duas** adições.

## 9 — Aberto (para o próximo, não para o integrador)

- O **consumidor de gameplay** (marker de timeline / callback de script) segue **cross-line e decisão do Enio**.
- **O impacto rápido invisível e o pico de impulso são UM item com UMA cura** (amostrar dentro do laço de sub-passos) ⇒ a frente **B** do handoff de reabertura deve absorver os dois. O preço é pago por **toda** cena — medir antes.
- Eventos por-**tick** em vez de por-dispatch · readout de contatos na §11.
