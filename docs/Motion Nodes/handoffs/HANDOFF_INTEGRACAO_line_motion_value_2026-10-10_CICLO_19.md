# HANDOFF de integração — `line/motion-value`, CICLO 19 (2026-10-10)

> Ciclo 19 «O JOGO MANDA» ([doc 128](../128_ciclo_19_o_jogo_manda.md): mecanismo, medições, lições e
> abertos). Aberto depois do smoke dos ciclos 17 e 18 APROVADO (10/10). A linha leva os ciclos 13–19,
> TODOS por integrar sobre o `main` `2a925859d` (o merge-base; `git log --oneline 2a925859d..HEAD`).
> Os ciclos 13–16 têm handoff próprio ([13](HANDOFF_INTEGRACAO_line_motion_value_2026-10-07_CICLO_13.md) ·
> [14](HANDOFF_INTEGRACAO_line_motion_value_2026-10-08_CICLO_14.md) ·
> [15](HANDOFF_INTEGRACAO_line_motion_value_2026-10-08_CICLO_15.md) ·
> [16](HANDOFF_INTEGRACAO_line_motion_value_2026-10-08_CICLO_16.md)); os ciclos 17 (doc
> [126](../126_ciclo_17_o_ritmo.md)) e 18 (doc [127](../127_ciclo_18_a_cor_que_o_olho_ve.md)) só têm o
> doc e o [handoff de continuação](HANDOFF_CONTINUACAO_line_motion_value_2026-10-10_CICLO_19.md) — a
> superfície deles está no §3.2 abaixo. A linha NÃO integra e NÃO pusha.

## §1 — Commits do ciclo 19 (`ff76bcadf..HEAD`)

| commit | o quê |
|---|---|
| `16542586e` | W0–W2: a fita das entradas (`ph2d-eval-motion/src/fita.rs`) e o `TimeMode::Driven` |
| `391c610bd` | W3: `value.input`, `value.follow` e o oráculo Godot do crossfade |
| `79e462fc5` | W4: os escritores — recurso `MotionInputs`, publicador, Luau, componente, sinais |
| `56cfd539e` | W4: `nomes_que_o_grafo_le` (os nomes para o cartão oferecer) |
| `bb6ee2604` | W4: o cartão do `Motion Input` no Inspector (40 ficheiros, um agente) |
| `90a1a97d9` | W5a: a cena `=134` e a saída `pulse` do `value.input` |
| `670215103` | W5b: a fita por CAMADA, a regra do relógio parado, o gate da placa, a sonda do grupo, `PH2D_MOTION_INPUTS` |
| `3c694ecdb` | W5c: o tutorial 12 em PDF, as fotos da `=134`, o gate dos passos |
| `e62216adb` | fecho: o gate batched, as 9 curas, as pendências dos ciclos 17/18 desta linha |

Total do ciclo: 124 ficheiros, +6112 / −179.

## §2 — O que o ciclo entrega

Nós `value.input` (Number · Toggle · Trigger · Color; saídas `out`, `r g b a`, `pulse`) e
`value.follow` (Linear · Smooth · Spring); `TimeMode::Driven` no `motion.time_remap`; a fita das
entradas (o scrub não mente, nas duas rotas); os escritores (componente `Motion Input`, Luau
`motion.set/color/fire`, sinais, `PH2D_MOTION_INPUTS`); a cena `=134`; o tutorial 12. Mecanismo e
desenho: doc 128 §3. Nenhum contrato congelado tocado (`NodeOp=2` · `OpResolver=1` · `NodeManifest=8`
intocados: tudo entrou como nó novo, param apendado ou canal ao lado).

## §3 — Superfície de colisão (para o integrador)

### §3.1 — FOUNDATIONAL do ciclo 19 (todas as mudanças são append-only, salvo a nota)

- **`ph2d-nodegraph`** (foundational):
  - `cook.rs`: a camada `externals_replay` (`Cook::set_externals_replay` · `visible_externals` ·
    `externals_replaying` · `remove_external`) e a resolução do `drive` no `cook_node` (o condutor lê-se
    no relógio de FORA antes de reescrever o de dentro);
  - `time.rs`: `TimeMode::Driven = 6` (apendado à escada), `TIME_DRIVE_PARAM = "drive"` e o campo novo
    **`TimeMap.drive`**. ⚠️ **Um `TimeMap { … }` escrito sem `..Default::default()` noutra linha PARTE o
    build** (campo em falta) — procure `TimeMap {` ao fundir;
  - `external.rs`: `input_of` e as constantes `INPUT_PREFIX` (`"$input:"`) · `INPUT_VALUE` · `INPUT_COLOR` ·
    `INPUT_COUNT`.
- **`ph2d-ecs`**: recurso novo `MotionInputs` (`motion_inputs.rs`), componente novo `MotionInput`
  (`motion_input.rs`) registado em `scene/registry.rs`. ⚠️ **Contadores como DELTA:** registo do ECS
  `105 → 106` (`registry_tests.rs`) e, por consequência, os censos da `ph2d-render`
  (`registry.rs`) e da `ph2d-script` (`registry.rs`) `106 → 107` — **delta +1**. Quem integrar CONTA o
  literal no `main` (outras linhas também acrescentam componentes), nunca copia o 106/107.
- **`ph2d-component-desc`** `catalog/logic.rs`: o descritor `ph2d::ecs::MotionInput` (família Logic),
  inserido ENTRE `Lifetime` e `PathFollow` (a lista é de busca binária — a ordem importa). ⚠️ A
  `line/components` mexe em `ph2d-component-desc/src/catalog/mod.rs` e `lib.rs` (aqui não tocados) e em
  `ph2d-app-components` (`component_attach.rs`, `object_add_tests.rs` — não tocados). Este ciclo APENDOU
  ficheiros novos `motion_input_bridge.rs` · `motion_input_inspector.rs` (+ `_tests`) e linhas em `lib.rs`
  — um conflito em `lib.rs` resolve-se mantendo os dois lados.
- **`ph2d-script`**: `motion_api.rs` (novo), o fio em `host.rs`, `scene.rs` (`SceneReport.motion`),
  `lib.rs`, `registry.rs` (o censo acima). ⚠️ `SceneReport` ganha um campo: construtor literal noutra
  linha parte.
- **`ph2d-gpu-cook`**: `ring.rs` e `lifecycle.rs` — `forget_after(tick)` (o ramo da fita esquece o
  futuro velho do anel da placa).
- **`ph2d-eval-motion`**: `fita.rs` (novo), `entradas.rs`, `scrub.rs`, `checkpoint.rs`, `lib.rs`.
- **Inspector / chrome do cartão** (bb6ee2604): `ph2d-editor-core` (`motion_input_edits.rs`,
  `action_bus_component.rs`, `ids/*`, `lib.rs`) · `ph2d-panel-inspector` (20 ficheiros: secção
  `sections/motion_input.rs`, `populate_/sync_/paint_*_motion_input.rs`, `event_motion_input.rs`, ids) ·
  `ph2d-panel-motion-graph` (`snapshot.rs`, `snapshot_cartoes.rs`, `paint_card_params.rs`) ·
  `ph2d-panel-registry-init` (dois gates da ordem das secções) · shell `render_loop` (`fase_*`, 8
  ficheiros; ⚠️ linha UI/UX: são fases partilhadas — a mudança é só a do cartão novo).
- **Shell, UMA linha:** `render_loop/motores_do_quadro.rs` chama
  `ph2d_app_components::motion_input_bridge::frame(sim, signals)` DEPOIS de todos os motores que
  falam e antes do Motion. Mais `project_schema.rs` / `project_schema_tests.rs` (abaixo) e um gate
  em `tests/it/every_membrane_resolves_the_driven_param.rs`.
- **`ph2d-i18n`**: ficheiro novo `inspector_motion_input.rs`, chaves novas em `component_catalog.rs`,
  `node_catalog.rs`, `node_options*.rs`, `node_params*.rs`.
- **`ph2d-node-motion-spring`**: a lei extraída para `law::passo` (bit-idêntica; os 24 testes dele
  verdes) — o `value.follow` Spring chama-a.
- **Nós/registo:** crates novas `ph2d-node-value-input` e `ph2d-node-value-follow` (+ `Cargo.toml` /
  `Cargo.lock`, `ph2d-node-registry-init`); `ph2d-node-motion-time-remap` (param `drive`);
  `ph2d-app-motion` (18 ficheiros: `motion_entradas.rs` — a SEXTA membrana da escada —,
  `motion_externals.rs`, cena `motion_state_jogo_demo*.rs`, router).

### §3.2 — Contadores a RECONTAR (delta, nunca literal)

| contador | neste ramo | delta do ciclo 19 | como reconta |
|---|---|---|---|
| `PROJECT_SCHEMA` | `188` (`shells/desktop/src/project_schema.rs`) | `187 → 188`: o `MotionInput` registado; a tripla NÃO muda | `python3 scripts/schema-recount.py` se o `main` andou |
| registo ECS | `106` / `107` (ecs / render e script) | `+1` | contar no `main` e somar 1 |
| `MAX_DEMO_LEVEL` | `134` (`motion_state_demo_router.rs`) | cena nova `=134` (o ciclo 18 deixou `133`; o 17, `132`) | se o `main` acrescentou cenas, reatribuir o número e a entrada de `"134"` em `motion_state_demo_router_ciclos.rs` |
| `ELIDED_TODAY` | `18` | `17 → 18` (pendência dos ciclos 17/18, abaixo) | gate do editor do Ease |

Ciclos 17/18 (por integrar, só nos docs 126/127): crates novas `ph2d-ease`; `ph2d-color` ganha a
mistura perceptual (OKLab/OKLCH); cenas `=132` e `=133`; `ParamWidget::Ease/EasePart` em
`ph2d-editor-core`; WGSL gerado das mesmas tabelas por bits. A superfície completa deles sai do
`collision-surface.sh` — corra-o pelo caminho ABSOLUTO (CLAUDE.md §1).

### §3.3 — O que um merge pode partir

1. `TimeMap { … }` e `SceneReport { … }` literais noutras linhas (campos novos).
2. A ordem do catálogo de componentes (busca binária) se alguém inserir outro descritor ao lado de `Lifetime`.
3. Os três contadores acima (registo ECS, `PROJECT_SCHEMA`, `MAX_DEMO_LEVEL`).
4. `lib.rs` de `ph2d-app-components` / `ph2d-ecs` / `ph2d-i18n` / `ph2d-panel-inspector` (linhas `mod`
   apendadas) — manter os dois lados.
5. `Cargo.lock`: duas crates internas novas; nenhum pacote externo novo.

## §4 — Gate batched (sobre `ff76bcadf..HEAD`, o `verificador` do fecho)

| passo | resultado |
|---|---|
| `nextest-impacted.sh` (`BASE=ff76bcadf`) | **18 952** corridos, **9 vermelhos** — todos curados em `e62216adb` (abaixo) |
| clippy `--all-targets -D warnings` | 3 erros na 1.ª corrida (type_complexity no `value.input`, um `mut`, um `contains_key`), curados; limpo nas 14 crates afectadas |
| `cargo fmt --all --check` | 1 ficheiro (`ph2d-color/perceptual_tests.rs`), curado; limpo |

As 9 curas: as linhas de conferência do `value.input`/`value.follow` na folha 15 (referências corridas
e citadas no doc 128) · o `name` do `value.input` no `ALWAYS_READ` · o `#[ignore]` da sonda com a
classe `sonda:` · o registo do ECS `106 → 107` nos censos da `ph2d-render` e da `ph2d-script` · o
`motion_entradas.rs` como a sexta membrana da escada · o tecto de LOC do `snapshot.rs` (ver abaixo).
Pendências dos ciclos 17/18 DESTA linha, que o gate do fecho viu (os ficheiros não mudaram no 19):
`ph2d-panel-motion-graph/snapshot.rs` a `621 > 600` linhas ⇒ os canais laterais do cartão (opções,
texto inteiro, curva) saíram para `snapshot_cartoes.rs` (`521`); a faixa do gráfico do Ease
(`−0,35 .. 1,35`) anotada como valor adimensional; `ELIDED_TODAY 17 → 18` (o cabeçalho do editor do Ease
corta numa janela de largura fixa, `c7ef7e7e0`); o `rustfmt` do `ph2d-color`.

## § Provas de mutação

| mutação | o gate | resultado |
|---|---|---|
| M1 o scrub não põe a fita (`entradas_do_replay` apagado) | `a_scrub_replays_the_inputs_…` | ✅ sangra (15 ≠ 11) |
| M2 parado não repete (`!a_tocar` → `false`) | `with_the_clock_stopped_dragging_forward_…` · o gate da PLACA | ✅ sangra (110 ≠ 12) · ✅ sangra |
| M3 o ramo não esquece o anel | `playing_on_after_a_scrub_…` | ✅ sangra (72 ≠ 60) |
| M4 o pulso compara com `agora` em vez de `antes` | ⚠️ SOBREVIVEU ao gate de então → gate NOVO `a_second_derivation_in_the_same_tick_keeps_the_pulse` | ✅ sangra (o re-cozimento do mesmo tique engolia o disparo) |
| M5 o `drive` dirigido não se resolve fora | `a_wired_drive_is_cooked_on_the_outer_clock` | ✅ sangra ([0] ≠ [0,6]) |
| M6 a camada ignorada nas três leituras | `a_scrub_replays_the_inputs_…` | ✅ sangra |
| M7 o `Follow` anda para o alvo de AGORA | `the_linear_follow_is_the_godot_crossfade_frame_by_frame` | ✅ sangra |
| M8 o `value.input` lê o nome cru (sem `$input:`) | `only_the_input_with_this_name_reaches_this_node` | ✅ sangra ([0,7] ≠ [0]) |
| M9 republicar suja sempre | `republishing_the_same_inputs_does_not_dirty_the_pump` | ✅ sangra |
| M10 o dano dispara com `<=` | `health_publishes_its_ratio_and_damage_fires_on_each_drop` | ✅ sangra ([2] ≠ [1]) |
| M11 sem o `clamp` do `Driven` | ⚠️ SOBREVIVEU — o `sample_curve_lut` já encosta a `0..1`: o limite estava escrito DUAS vezes | ✅ curado: a cópia do `apply` saiu (fica só o `NaN`) |

Controlo: cada gate verde antes de mutar; cada âncora casou 1× (3× no M6); árvore limpa no fim.

## §5 — Medições e oráculo

**O oráculo** (CLAUDE.md §0.9): Godot 4.7.2 instalado (MIT, `pacman -Qi`), um `AnimationNodeTransition`
com `xfade_time` 0,25 · 0,5 · 1 s corrido por `ferramentas/godot_xfade_oracle/corre.sh`, fixture
`crates/ph2d-node-value-follow/tests/fixtures/godot_xfade.txt`; o gate
`the_linear_follow_is_the_godot_crossfade_frame_by_frame` compara a cadeia do produto tique a tique:
**pior `≤ 2e-5`** (doc 128 §5.2).

**O scrub, antes da cura** (W0): `11` visto → `15` replayado; ramo novo `60` → `72`. Depois: seis gates
verdes (doc 128 §5.1).

**A cena `=134`** (saída pela bomba do produto, doc 128 §5.3): barra cheia `13 · 0 · 5 · 11 · 24` de
24 peças (cartão 0,6; vida 0 · 0,25 · 0,5 · 1); peso do cruzamento `0` no tique 30 · `0,500` no 48 ·
`1` no 66; discos `(0,2 · 0,4 · 0,9)`; boom: redemoinho `0,031 → 0,103` voltas/s (×3,3), estrelas
`0,130 → 0,323` (×2,5); scrub em 5 alvos igual ao visto. Recusa medida: o empurrão radial (raio médio
`1,95 → 2,26 … 8,92` com K `10 … 160` e NÃO volta).

**O custo** (`measure_the_game_group`, `--release`, load `47,9` — ordem de grandeza; doc 128 §5.4):

| caso | mínimo / mediana |
|---|---|
| `value.follow` Linear (32 761 linhas) | 0,083 / 0,086 ms |
| `value.follow` Smooth | 0,078 / 0,078 ms |
| `value.follow` Spring | 0,136 / 0,136 ms |
| `Driven`, entrada parada (memo) | 0,002 / 0,002 ms |
| `Driven`, entrada a mexer (32 761 recozidas) | 0,008 / 0,526 ms |
| cena `=134`, um quadro | 0,065 / 0,120 ms |
| a fita, 3 entradas a mexer | 14 383 B em 421 tiques = 34,2 B/tique (`FITA_BYTES` 32 MiB ⇒ ~1 M tiques) |
| scrub (5 alvos) | 0,117 / 0,167 ms |

## §6 — Smoke do dono (✅ APROVADO, 10/10: *«o smoke está ok»*)

✅ **Smoke do dono APROVADO (10/10).** No mesmo report, um defeito de TODO o módulo, fora do ciclo: *«apertar botões em todo módulo desde os botões da barra inferior até os campos dentro dos nós é impreciso e inconsistente»* — aberto como BUGS do Motion (a precisão do clique). Tutorial 12 — `docs/Motion Nodes/tutoriais/12_o_jogo_manda.pdf` (5
páginas, gate `every_step_of_the_game_tutorial_is_reachable`) — sobre a cena `=134`:

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=134 cargo run -p ph2d-host-desktop --profile smoke`
   (o comando do tutorial; `env` porque o shell do dono é o `fish`).
2. Siga o PDF, secções 1–6: no cartão `Input` do grafo com `Name` = `vida`, arraste `Value` e veja a
   barra encher; no `estado`, ligue `On` e veja a grelha virar anel em 0,6 s; no `time`, mude a cor;
   no `boom`, clique `Fire` e veja as estrelas incharem e o redemoinho acelerar e acalmar em 2 s;
   pare e arraste a fita para trás e para a frente — tudo repete; depois crie um objecto, junte-lhe
   o componente `Motion Input` no Inspector, escolha `vida` e `Value` 0,25 — a barra vai a 25%.
3. Deu errado se: a barra não segue o `Value`, a grelha salta de uma vez para o anel, o `Fire` não
   mexe na galáxia (ou ela não acalma), voltar a fita mostra outra coisa, ou o `Motion Input` não
   mexe na barra. Fotos de controlo: `target/prova/c19/fotos/`.
4. Porta de teste sem clicar: `PH2D_MOTION_INPUTS=vida=0.25:estado=1:time=0.2,0.5,1`.

## §6.1 — Depois do smoke: BUGS do Motion #18 (`4a85d94ba`)

O report do mesmo smoke (*«apertar botões … impreciso e inconsistente»*) foi curado na linha:
`ph2d-editor-core/src/interaction/press_slop.rs` — UMA folga (`PressSlop`, `PRESS_SLOP_PX` = 4 px
lógicos) para o grafo, a timeline e o Flip; saem `TIMELINE_DRAG_SLOP_PX` e
`FLIP_STRIP_DRAG_SLOP_PX` (sem uso fora do núcleo nesta árvore — ⚠️ o integrador confere se outra
linha os passou a usar); os campos `graph_moved`/`timeline_press`/`timeline_moved` do `WidgetStore`
viram `graph_slop`/`timeline_slop`, e `set_graph_moved` vira `begin_graph_press` +
`note_graph_pointer`. No grafo os `Update` só correm depois da folga. E no cartão, na faixa de zoom
legível-mas-não-agarrável, um toque age na fileira (`tap_on_a_readable_row`). Detalhe e gates:
[BUGS #18](../BUGS_motion_nodes.md). ⏳ smoke do dono.

## §7 — O que fica ABERTO

Do doc 128 §7:

- **A dobra da fita acima do orçamento** (`FITA_BYTES`): um scrub para antes da base repete o estado
  da base; com muitos objectos a mexer são minutos — por medir numa cena de objectos.
- **Um gesto para dirigir uma linha de COR inteira** (hoje três fios `r g b` do `Input` ao `Tint`).
- **A vida e o dano do mesmo objecto** pedem dois objectos (um `Motion Input` por objecto).

Herdados dos ciclos 17/18: o `Bezier` do easing e o `Tint` OKLCH são caros na CPU de reserva (5 ms e
21 ms a 32 761 linhas; na placa 0,15 e 0,16 ms).

## §8 — Próximo ciclo

Ciclo 20 **«O ACABAMENTO»** ([doc 126 §0](../126_ciclo_17_o_ritmo.md), linha 20): pós-processamento como nós
sobre a camada do Motion — blur · glow · sombra de pixel · warp · grade/LUT · vinheta · grão ·
aberração. Reusa o `FxStackPass` (`ph2d-render/src/fx_stack.rs`) e o alvo HDR isolado do bloom
(`ph2d-render/src/motion_fx.rs`). Prompt: [CICLO_20](HANDOFF_CONTINUACAO_line_motion_value_2026-10-10_CICLO_20.md).
