# HANDOFF DE INTEGRAÇÃO — `line/physics` · W-ContactEvents + W-ImpactForce (2026-07-22)

> Para o **agente integrador**, sob ordem explícita do Enio (DIRETRIZ §1.5.9).
> A linha **não** integrou, **não** pushou e **não** rodou `ship.sh`.
>
> Duas waves nesta jornada, a segunda completando a primeira: **W-ContactEvents** (a
> transição *começou/parou de tocar*) e **W-ImpactForce** (*quão forte foi o toque*).

## 1 — Identidade

| | |
|---|---|
| Branch | `line/physics` |
| HEAD | `22803a622` |
| Base (merge-base com `main`) | `13a04c7aa` |
| Commits à frente | **6** |
| Rebase | feito no início da jornada; a linha já estava em cima do `main` |

Os 6 commits, em ordem:

- `660af8fdc` — handoff de REABERTURA (doc-only)
- `3cfd5c5d3` — **W-ContactEvents** (a wave A)
- `b9d948b65` — a 1ª versão deste handoff (doc-only; este arquivo o **substitui**)
- `372d72a57` — memória `feedback_bash_cwd_resets_and_slips_to_the_primary` + índice (`project-memory/`, aditivo)
- `44b6086db` — mensagem da cena 29 mais clara (doc-only)
- `22803a622` — **W-ImpactForce** (a wave B)

## 2 — Foundational / compartilhado tocado, e por quê

| Arquivo | O quê | Risco de merge |
|---|---|---|
| `CLAUDE.md` §5 | **DUAS frases apendadas** ao bloco *"Física global"* — uma por wave (W-ContactEvents smoke-OK; W-ImpactForce novo) | ⚠️ **É o único ponto quente.** Estritamente **ADITIVO**; se outra linha tocar o mesmo parágrafo, mantenha **as duas** adições |
| `project-memory/MEMORY.md` + `.md` novo | 1 memória aditiva (o slip de cwd do Modo L) | trivial (lista compartilhada — só ADICIONE) |
| `shells/desktop/src/main.rs` | (já no commit A) `mod physics_smoke_events;` | trivial |
| `shells/desktop/src/physics_smoke.rs` | 2 braços: `"29"` (A) e `"30"` (B) no `match` | trivial (append) |
| `shells/desktop/src/render_loop/physics_overlay.rs` | (commit A) +1 laço de stroke (o flash) + 2 nomes no `use` | baixo |
| `docs/Physics/00_plano_waves.md` | +2 linhas na tabela de waves | trivial |
| `docs/Physics/HANDOFF_line_physics.md` | +2 seções no fim (tracker DESTA linha) | trivial |

Tudo o mais é **da pasta do módulo** (`crates/ph2d-physics/`, `crates/ph2d-physics-ecs/`,
`render_loop/physics_overlay_contacts*`, `physics_smoke_events.rs`).

## 3 — Símbolos NOVOS (o que grepar por colisão de mesmo-nome)

**Nenhum id de UI, nenhum token, nenhuma const de registro.** Os nomes novos são todos de
código, e todos dentro da física:

**W-ContactEvents (A):**
- `ph2d_physics_ecs::ContactEvent` · `ph2d_physics_ecs::ContactPhase` (`Began`/`Ended`)
- `BodyContact.age_ticks: Option<u64>` — campo NOVO num tipo público de plain-data
- `PhysicsBridge::contact_events()` · `discard_contact_history()` (pub(super)) · `bridge::contacts::ContactMemo`
- **módulo novo** `crates/ph2d-physics-ecs/src/bridge/rewind.rs`
- shell: `contact_flashes` · `CONTACT_FLASH_RGBA` · `FLASH_TICKS`/`FLASH_MIN_PX`/`FLASH_MAX_PX` · `App::physics_smoke_events`

**W-ImpactForce (B):**
- `ph2d_physics::ContactReport.impact: f32` — **campo NOVO num tipo público de plain-data** (⚠️ quem constrói um `ContactReport`/`BodyContact`/`ContactEvent` literal precisa dele)
- `BodyContact.impact` · `ContactEvent.impact` · `ContactMemo.impact`
- `ph2d_physics::world::contacts::{PeakKey, accumulate_peaks, active_pair}` (pub(crate)/pub(super)/priv) · `PhysicsWorld.contact_peaks` (campo priv)
- **módulo novo** `crates/ph2d-physics/src/world/convenience.rs` (`mod convenience;` no `world.rs`) — só MOVEU `add_dynamic_circle`/`add_static_cuboid` (inherent methods, paths intactos)
- shell: `FLASH_IMPACT_BOOST_PX` · `IMPACT_FULL_NS` · `App::physics_smoke_impact_ladder`
- testes novos: `crates/ph2d-physics/tests/measure_impact.rs` (harness `#[ignore]`)

**Números que se CONTAM e NÃO se escolhem — nenhuma das duas waves move algum:**
`PROJECT_SCHEMA` fica **29** · registro de componentes fica **18** · `physics_ecs_c9`
fica em **75 corpos**. Se outra linha bumpar o schema, **não há nada meu para somar**.

## 4 — Contratos congelados encostados

**Nenhum.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` intactos. Nenhum ADR novo (ambas as waves são integração e leitura, não solver;
o norte do ADR-0131 não muda).

## 5 — O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **Deps novas: ZERO** ⇒ `machete`/`deny`/`audit` não têm o que reclamar por minha causa.
- `fmt` rodado com **`rustup run 1.95 rustfmt --edition 2024`** nos arquivos tocados (só os meus, para não reformatar nada mais).
- `clippy --all-targets` nas crates tocadas: **limpo** (os 3 `collapsible_if` do harness de medição foram corrigidos).
- `typos`: sem termos novos meus; ⚠️ invocá-lo com paths explícitos acusa PORTUGUÊS pré-existente no `CLAUDE.md` — não é meu.
- **`BodyContact`/`ContactReport`/`ContactEvent` ganharam campo** ⇒ se outra linha construir um desses literais, aparece só no gate da árvore COMBINADA (`foundational-integrate.sh`), que é seu. Hoje os únicos construtores literais estão nos meus testes.

## 6 — Gate batched desta linha (tudo verde)

- `cargo test -p ph2d-physics -p ph2d-physics-ecs` — **debug E release**, 0 falhas (a `line/FLIP` documentou que só-release esconde pânico).
- `cargo test -p ph2d-host-desktop` — 0 falhas (suíte da shell).
- `architecture_workspace_file_loc_cap` · `file_loc_caps` (shell, HR-18) · `no_tofu_glyphs` — **verdes**.
- **c9 byte-idêntico ao `main`:** `physics_ecs_c9` = `7d55a4ab…`, 75 corpos (rodado no worktree; ambas as waves são readout e não tocam a pose dos corpos).
- **Wave A:** 13 gates, 8 mutações, 8 sangram. **Wave B:** 3 gates (wrapper+ponte+overlay), 5 mutações, 5 sangram.
- **Custo medido (B, sempre-ligado):** `tests/measure_impact.rs` — a captura do pico é ≤ **2,4% do HR-4** a 500 pares.

## 7 — O que SMOKE-TESTAR (nada foi smokado por mim — só gates)

**Wave A — `env PH2D_PHYSICS_SMOKE=29 cargo run -p ph2d-host-desktop --release`** (a mensagem da cena explica; aperte **L** para a timeline):
1. Bola (esquerda) pisca um `×` a cada pouso; entre quiques a cruz `+` **some**.
2. Caixa (meio) pisca **uma vez** e a cruz fica **para sempre** (flash=EVENTO × cruz=ESTADO).
3. Pilha (direita) pisca **uma vez, no 1º tick** (leitura da Unity).
4. ⚠️ Arraste a régua para TRÁS: as cruzes mudam e **NADA pisca**.
5. ⚠️ Desmarque `Physics` na barra: as cruzes **SOMEM**; ao remarcar, voltam **sem piscar** (o bug vivo que A consertou).

**Wave B — `env PH2D_PHYSICS_SMOKE=30 cargo run -p ph2d-host-desktop --release`** (a escada de impacto):
6. Quatro bolas IGUAIS caem de alturas crescentes; cada uma pisca um `×` ao pousar, e o **flash é maior quanto mais alto ela caiu** (o tamanho do `×` = a FORÇA do impacto, medida 0,81/1,39/1,95/2,52). Os flashes vêm em sequência (a mais baixa primeiro) — scrub para revê-los lado a lado.

**Regressão a olhar de carona:** `=25` (W-Contacts) inalterada exceto o flash aparecer nos
primeiros ticks; `=7` (Bake) exercita o `hold`, que A mudou.

## 8 — Ordem e dependências

Uma jornada, sem dependência de outra linha. Se `line/Painter`, `line/anim` ou `line/Vector`
entrarem na mesma janela, o **único** ponto de encontro é o parágrafo do `CLAUDE.md` §5 (e a
lista `MEMORY.md`) — resolva mantendo **todas** as adições.

## 9 — Aberto (para o próximo, não para o integrador)

- O **consumidor de gameplay** (marker de timeline / callback de script) segue **cross-line e decisão do Enio**.
- **O impacto RÁPIDO invisível** (toque que começa e termina dentro de um tick) segue sem evento — o **pico** já é capturado (B), mas o **evento** para o toque rápido reestrutura o diff do front A (conjunto permanente → por-tick, com a união dos ticks de um dispatch) e é a **próxima wave**. B é o pré-requisito dela.
- Eventos por-**tick** em vez de por-dispatch · readout de contatos na §11.
