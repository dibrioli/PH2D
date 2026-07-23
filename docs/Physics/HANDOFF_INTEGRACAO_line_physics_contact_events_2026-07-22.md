# HANDOFF DE INTEGRAÇÃO — `line/physics` · W-ContactEvents + W-ImpactForce + W-TickContacts (2026-07-22)

> Para o **agente integrador**, sob ordem explícita do Enio (DIRETRIZ §1.5.9).
> A linha **não** integrou, **não** pushou e **não** rodou `ship.sh`.
>
> TRÊS waves nesta jornada, cada uma completando a anterior: **W-ContactEvents** (a
> transição *começou/parou de tocar*, cena 29), **W-ImpactForce** (*quão forte foi o
> toque*, cena 30 — a DEMOLIÇÃO) e **W-TickContacts** (o toque RÁPIDO vira evento — o
> diff por tick, cena 31). As três são **readout puro**: `c9` byte-idêntico ao `main`,
> nenhum `PROJECT_SCHEMA`/registro bumpado.

## 1 — Identidade

| | |
|---|---|
| Branch | `line/physics` |
| HEAD | `7aa6e9543` |
| Base (merge-base com `main`) | `13a04c7aa` |
| Commits à frente | **10** |
| Rebase | feito no início da jornada; a linha já estava em cima do `main` |

Os 10 commits, em ordem:

- `660af8fdc` — handoff de REABERTURA (doc-only)
- `3cfd5c5d3` — **W-ContactEvents** (a wave A)
- `b9d948b65` — a 1ª versão deste handoff (doc-only; este arquivo o **substitui**)
- `372d72a57` — memória `feedback_bash_cwd_resets_and_slips_to_the_primary` + índice (aditivo)
- `44b6086db` — mensagem da cena 29 mais clara (doc-only)
- `22803a622` — **W-ImpactForce** (a wave B)
- `b96d73383` — 2ª versão deste handoff (doc-only; este arquivo o **substitui**)
- `101e74a42` — memória `feedback_two_quantities_that_should_differ_can_coincide_by_fixture_phase` (aditivo)
- `aa75eb048` — **cena 30 vira DEMOLIÇÃO** (o 1º corte batia num chão imóvel e o Enio recusou)
- `7aa6e9543` — **W-TickContacts** (a wave C)

## 2 — Foundational / compartilhado tocado, e por quê

| Arquivo | O quê | Risco de merge |
|---|---|---|
| `CLAUDE.md` §5 | **frases apendadas** ao bloco *"Física global"* — A (smoke-OK), B (smoke-OK, cena virou DEMOLIÇÃO), C (nova) | ⚠️ **É o único ponto quente.** Estritamente **ADITIVO**; se outra linha tocar o mesmo parágrafo, mantenha **todas** as adições |
| `project-memory/MEMORY.md` + `.md`s novos | memórias aditivas (o slip de cwd; fixture-por-fase; e a de C) | trivial (lista compartilhada — só ADICIONE) |
| `shells/desktop/src/main.rs` | (já no commit A) `mod physics_smoke_events;` | trivial |
| `shells/desktop/src/physics_smoke.rs` | 3 braços: `"29"`/`"30"`/`"31"` no `match` | trivial (append) |
| `shells/desktop/src/render_loop/physics_overlay.rs` | (A) laço do flash; (C) `draw` ganhou o param `flashes: &[ContactFlash]` e o `contact_flashes` lê ELE | baixo — um chamador só (`mod.rs`) |
| `shells/desktop/src/render_loop/mod.rs` | (C) `let flashes = physics.contact_flashes().to_vec();` passado ao `draw` | baixo — 2 linhas |
| `docs/Physics/00_plano_waves.md` | +3 linhas na tabela de waves | trivial |
| `docs/Physics/HANDOFF_line_physics.md` | +seções no fim (tracker DESTA linha) | trivial |

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
- `ph2d_physics::world::contacts::{PeakKey, accumulate_peaks, active_pair}` (pub/pub(super)/priv) · `PhysicsWorld.contact_peaks` (campo priv)
- **módulo novo** `crates/ph2d-physics/src/world/convenience.rs` (`mod convenience;` no `world.rs`) — só MOVEU `add_dynamic_circle`/`add_static_cuboid` (inherent methods, paths intactos)
- shell: `FLASH_IMPACT_BOOST_PX` · `IMPACT_FULL_NS` · `App::physics_smoke_impact_demolition`
- testes novos: `crates/ph2d-physics/tests/measure_impact.rs` (harness `#[ignore]`)

**W-TickContacts (C):**
- `ph2d_physics::{PeakSample, PeakKey}` — `PeakKey` virou `pub` (era `pub(crate)`), `PeakSample` é NOVO (o valor do `contact_peaks`: `impact`+`point`+`impulse`, era só `f32`)
- `ph2d_physics::PhysicsWorld::tick_contacts()` — expõe o `contact_peaks` como `&BTreeMap<PeakKey, PeakSample>`
- `ph2d_physics_ecs::{ContactFlash, CONTACT_FLASH_TICKS}` — NOVOS (o canal do flash, e sua vida em ticks)
- `PhysicsBridge::contact_flashes()` (pub) · `accumulate_contact_events`/`rebuild_standing_contacts`/`handle_map` (pub(super)) · `light_flash` (priv) · campo `flashes: Vec<ContactFlash>`
- ⚠️ **REMOVIDOS:** `BodyContact.age_ticks` e `ContactMemo.began` (existiam só para o flash antigo; a remoção de um campo público é mais impactante que a adição — quem LÊ `.age_ticks` quebra, mas hoje só os testes desta linha o faziam). `rebuild_contacts` renomeou/dividiu.
- shell: `App::physics_smoke_fast_impact` (cena 31); `physics_overlay_contacts` deixou de usar `FLASH_TICKS` (importa `CONTACT_FLASH_TICKS`)

**Números que se CONTAM e NÃO se escolhem — nenhuma das TRÊS waves move algum:**
`PROJECT_SCHEMA` fica **29** · registro de componentes fica **18** · `physics_ecs_c9`
fica em **75 corpos** (`7d55a4ab…`). Se outra linha bumpar o schema, **não há nada meu para somar**.

## 4 — Contratos congelados encostados

**Nenhum.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` intactos. Nenhum ADR novo (ambas as waves são integração e leitura, não solver;
o norte do ADR-0131 não muda).

## 5 — O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **Deps novas: ZERO** ⇒ `machete`/`deny`/`audit` não têm o que reclamar por minha causa.
- `fmt` rodado com **`rustup run 1.95 rustfmt --edition 2024`** nos arquivos tocados (só os meus, para não reformatar nada mais).
- `clippy --all-targets` nas crates tocadas (`ph2d-physics`, `ph2d-physics-ecs`, `ph2d-host-desktop`): **limpo**.
- `typos`: sem termos novos meus; ⚠️ invocá-lo com paths explícitos acusa PORTUGUÊS pré-existente no `CLAUDE.md` — não é meu.
- **`ContactReport`/`ContactEvent` ganharam campo (`impact`) E `BodyContact` PERDEU um (`age_ticks`)** ⇒ se outra linha construir um desses literais, ou LER `.age_ticks`, aparece só no gate da árvore COMBINADA (`foundational-integrate.sh`), que é seu. Hoje os únicos construtores/leitores literais estão nos meus testes.

## 6 — Gate batched desta linha (tudo verde)

- `cargo test -p ph2d-physics -p ph2d-physics-ecs` — **debug E release**, 0 falhas (a `line/FLIP` documentou que só-release esconde pânico).
- `cargo test -p ph2d-host-desktop` — 0 falhas (suíte da shell).
- `architecture_workspace_file_loc_cap` · `file_loc_caps` (shell, HR-18) · `no_tofu_glyphs` — **verdes**.
- **c9 byte-idêntico ao `main`:** `physics_ecs_c9` = `7d55a4ab…`, 75 corpos (rodado no worktree; as TRÊS waves são readout e não tocam a pose dos corpos).
- **Wave A:** 13 gates, 8 mutações, 8 sangram. **Wave B:** 3 gates, 5 mutações, 5 sangram. **Wave C:** 12 gates ecs (2 fast-touch red-first + flash-decay + live-flash) + 10 overlay retunados, **6 mutações, todas sangram** (união→fim-de-passo · once-per-dispatch · light_flash no-op · nunca-descartar · discard-não-limpa · re-baseline-off).
- **Custo:** captura do pico ≤ **2,2% do `step`** (`measure_impact.rs`); o diff por-tick de C é BTreeMap-sobre-contatos (µs contra os 57 ms/tick do `step` a 500 contatos), e em play normal (1 tick/dispatch) roda as MESMAS vezes que o diff por-dispatch do front A.

## 7 — O que SMOKE-TESTAR (nada foi smokado por mim — só gates)

**Wave A — `env PH2D_PHYSICS_SMOKE=29 cargo run -p ph2d-host-desktop --release`** (a mensagem da cena explica; aperte **L** para a timeline):
1. Bola (esquerda) pisca um `×` a cada pouso; entre quiques a cruz `+` **some**.
2. Caixa (meio) pisca **uma vez** e a cruz fica **para sempre** (flash=EVENTO × cruz=ESTADO).
3. Pilha (direita) pisca **uma vez, no 1º tick** (leitura da Unity).
4. ⚠️ Arraste a régua para TRÁS: as cruzes mudam e **NADA pisca**.
5. ⚠️ Desmarque `Physics` na barra: as cruzes **SOMEM**; ao remarcar, voltam **sem piscar** (o bug vivo que A consertou).

**Wave B — `env PH2D_PHYSICS_SMOKE=30 cargo run -p ph2d-host-desktop --release`** (a DEMOLIÇÃO):
6. Duas raias iguais (torre de caixas leves + bola pesada lançada), só a velocidade muda: EM CIMA lenta (5 m/s) → a torre balança, `×` pequeno; EMBAIXO rápida (16 m/s) → a torre EXPLODE, `×` enorme. O tamanho do `×` = a FORÇA do impacto (medido ~1,4 a 6 m/s, ~4,5 a 16 m/s), e ela tem CONSEQUÊNCIA visível — as caixas voam. (Tecla **L** dá scrub para rever os impactos.) ⚠️ A 1ª versão batia num chão imóvel e o Enio recusou (*"não mostra o efeito"*).

**Wave C — `env PH2D_PHYSICS_SMOKE=31 cargo run -p ph2d-host-desktop --release`** (o toque RÁPIDO acende):
7. Duas bolas que quicam, mesma restituição, só a ALTURA muda. ESQUERDA baixa (1,2 m) → pousos lentos, sempre acendeu (controle). DIREITA alta (8 m) → pousos rápidos que **eram invisíveis** (ela quicava alto e não acendia `×` nenhum, e quanto mais forte, mais invisível). Agora TODO pouso acende, e o `×` da alta é MAIOR (impacto maior). Medido: pico ~1,6 N·s na baixa, ~4,8 na alta.

**Regressão a olhar de carona:** `=25` (W-Contacts) inalterada exceto o flash aparecer nos
primeiros ticks; `=29`/`=30` seguem valendo (o flash agora é o canal event-sourced, mas o
comportamento visível é o mesmo — a bola pisca a cada pouso, e agora inclusive nos pousos rápidos
que a cena 29 largava de perto para evitar); `=7` (Bake) exercita o `hold`, que A/C mudaram.

## 8 — Ordem e dependências

Uma jornada, sem dependência de outra linha. Se `line/Painter`, `line/anim` ou `line/Vector`
entrarem na mesma janela, o **único** ponto de encontro é o parágrafo do `CLAUDE.md` §5 (e a
lista `MEMORY.md`) — resolva mantendo **todas** as adições.

## 9 — Aberto (para o próximo, não para o integrador)

- O **consumidor de gameplay** (marker de timeline / callback de script) segue **cross-line e decisão do Enio** — a fronteira que o W7 desenhou. Este canal é o primitivo + a leitura visível, não o consumidor.
- ~~O impacto RÁPIDO invisível~~ **FECHADO pela wave C** (diff por tick sobre a união dos sub-passos). O único toque que ainda escapa é o que começa **e** termina no MESMO sub-passo — que o solver discreto nem produz (túnel; trabalho do CCD).
- readout de contatos na §11 (um número, não a cruz do overlay) — não pedido ainda.
