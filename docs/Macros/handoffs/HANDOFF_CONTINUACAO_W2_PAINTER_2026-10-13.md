# Continuação — line/Macros-Automação, W2 (trilha do Painter) fechada → abrir W3 (trilha do Motion)

> Para a PRÓXIMA janela desta linha. Branch `line/Macros-Automação`, worktree
> `Worktrees/line-Macros-Automação/`, base `757afebd58`. Leia antes de agir: o
> [ADR-0186 §Emenda W2](../../architecture/decisions/0186-a-macro-grava-comandos-resolvidos-no-relogio-da-macro.md),
> o [plano da onda 1](../01_plano_onda1.md) e o [handoff da W1](HANDOFF_CONTINUACAO_W1_NUCLEO_2026-10-13.md).

## Estado

| commit | o quê |
|---|---|
| W1 (`7711955529`, `b106a27c27`) | núcleo `ph2d-macro` + ADR-0186 |
| **W2** (este) | a trilha do Painter (Wet Paint), a fita do worker, o estado inicial, o censo das portas, gates 1·2·4·6 + o do estado inicial |

**O que existe (não reconstruir):**
- `crates/ph2d-macro/src/rec.rs` — a lei dos campos etiquetados, PÚBLICA (o `codec` usa-a; as famílias
  codificam as cargas por ela). Antes era privada ao `codec`.
- `crates/ph2d-wet-paint` — `Sim::stage_in_flight()` (só leitura).
- `crates/ph2d-tool-painter/src/tool/paint/wetpaint/fita.rs` — **a fita do worker**: `MarcaDaAgua
  {frame, estagio}`, `RespostaDoWorker {Voltou, NaoVoltou, Passos}`, `Fita {Grava, Toca}`, `anda_ate`. As
  portas do `WetSession` (`offthread.rs`: `bring_home`, `try_bring_home`, `try_bring_home_now`,
  `sim_steps`, `hand_off_sim`) gravam/respondem por ela; `EngineSlot::Roteiro` = o motor «com o worker»
  do roteiro ao tocar.
- `crates/ph2d-tool-painter/src/macro_track/` — `mod.rs` (a gravação no tool: `macro_grava`,
  `macro_passos`, `macro_para`, o `porta!` que só grava a porta de FORA, `macro_recusa`, a recusa do meio),
  `passo.rs` (o formato dos passos, tipo na etiqueta 1, números para sempre), `inicial.rs` (documento
  postcard + configurações JSON), `tocar.rs` (`TocadorDoPainter::{novo, sobre_tela, aplica}`),
  `portas.rs` + `portas_internas.rs` (o censo), `tests.rs`.
- `crates/ph2d-tool-painter/src/tool/paint/macro_estado.rs` — as configurações do estado inicial (o preset
  + `CAMPOS_DA_MACRO` + o meio e o traço).
- `wetpaint_preset/json.rs` converte também os 28 campos de `CAMPOS_DA_MACRO` (obra e gesto).
- `persist.rs::documento_vivo` — a coleta do projeto e o estado inicial partilham a porta.

**Decisões técnicas desta onda (ver o ADR):**
1. ⛔ A «lei nova» do plano (drenar o passo em voo no 1.º lote molhado) foi **RECUSADA**: mudava a pintura de
   todos (até `39,5 ms` de espera no toque a 4096²) e não cobria a Água ao vivo. A fita é exacta por
   construção e não muda lei nenhuma.
2. Armar encerra a sessão molhada viva (`wetpaint_end_session`, o fim da sessão é o bake) — a lei de sempre
   das ações de tela. Sem isto, armar depois de QUALQUER traço molhado recusava-se (a sessão vive depois de
   a tinta secar).
3. Só o Wet Paint grava: armar noutro meio = `Recusa::Meio`; trocar de meio a gravar = `Passo::Recusada(
   PORTA_DO_MEIO)` e o tocador pára.
4. O tocador mora em `ph2d-tool-painter` (headless, sem shell nem UI) e não em `ph2d-app-painter`: precisa
   das portas internas do tool (`wet_instala_fita`, `documento_vivo`). A ligação à cena é a W4.
5. A semente viaja no ESTADO INICIAL (não num `StrokeBegin`): ela avança por gesto de forma determinística.

## Gates (todos verdes; cada um com controlo)

| gate | onde | controlo |
|---|---|---|
| 1 água ao byte, worker com esperas reais, a fixture EXIGE um toque a meio de um passo | `tests/it/macro_agua.rs` | uma marca adulterada no toque muda a tela |
| 2 semente + `dt` irregular (Airbrush, relógio fixo), semente avançada antes de armar | `tests/it/macro_traco.rs` | semente mexida e um tique a menos mudam a tela |
| 6 tela 2× — o traço pousa no mesmo lugar relativo (< 1 px) | `macro_traco.rs` | sem escala cairia a > 10 px |
| 4 censo: 434 portas (89 decididas, 345 internas), o corpo grava/recusa, nenhuma interna chamada de fora | `tests/it/macro_portas.rs` | a recusa PÁRA o tocador (comportamento) |
| só a porta de FORA grava (`wet_preset_aplicar` chama 3 portas gravadas por dentro → 1 passo) | `macro_portas.rs` | nasceu da mutação M4, que sobreviveu a todos os outros |
| estado inicial: os 276 controlos do Wet Paint chegam à reprodução, antes de armar E a gravar | `ph2d-panel-painter-layers/tests/it/macro_estado_inicial.rs` | um inicial velho muda a tela |
| formato dos passos (ida e volta, tipo desconhecido, campo novo salta), a fita, a versão do documento | `src/macro_track/tests.rs` | — |

Fecho da onda (sobre o diff inteiro, base `757afebd58`): `nextest-impacted` 11 956/11 958 — as duas falhas
eram o `COLOR-RAW-OK` de `passo.rs` (corrigido: `pub type Carga`) e o flake de carga
`ph2d-physics-ecs measure_player_budget` (load 190; sozinho 3/3 verde, crate fora do diff); depois das
correções as 4 crates 2 027/2 027, `arch_color_space_typed` verde · `architecture*` do `ph2d-editor-core`
111/111 · clippy `-D warnings --all-targets` das 4 crates limpo · `cargo fmt` · `cargo machete` limpo ·
`cargo check --workspace --all-targets` verde.

## Mutação (12/12 sangram, cada uma vista verde antes)

| | mutação | portão que sangra |
|---|---|---|
| M1 | `anda_ate` devolve `true` sem andar | gate 1 (+40 passos: 51 862 texels) |
| M2 | a marca grava `estagio: None` | gate 1 (a fixture deixa de conter o fenómeno) |
| M3 | o tocador entrega a um worker real | gate 1 (174 respostas por pedir) |
| M4 | a porta de dentro também grava | `a_porta_chamada_por_dentro_de_outra_nao_grava` (sobreviveu aos outros — o gate nasceu dela) |
| M5 | sem a recusa do meio | `cada_controlo_mexido_a_gravar_reproduz_se` |
| M6 | o substrato fora do inicial | `o_estado_inicial_leva_cada_controlo_do_wet_paint` |
| M7 | a semente fora do inicial | gate 2 (1 915 texels) |
| M8 | o tique ignorado ao tocar | gates 1 e 2 |
| M9 | a posição sem escala | gates 2 e 6 |
| M10 | uma porta sem decisão | `toda_porta_do_painter_tem_decisao` |
| M11 | `macro_recusa` vazia | `uma_porta_recusada_a_gravar_para_o_tocador` |
| M12 | o `Tique` lê 0 | `cada_passo_vai_e_volta_ao_bit` |

## Medido (sob carga 61–66 — sem valor de veredito)

Quadro a tocar × a pintar (`tests/it/sonda_macro_custo.rs`, `--profile smoke`, mínimo de 3 reproduções):
1024² mediana `6,60` × `5,82 ms`, pior `59,2` × `56,4`; 2048² mediana `7,20` × `8,94`, pior `62,7` × `39,4`.
⇒ **Re-medir com a máquina calma** antes do smoke de performance. Se não couber: o tocador entrega os estágios
a um worker PRÓPRIO até a próxima marca (ele já a conhece — está na fila). O composite já é um por quadro
(segue os tiques do original), então o item 7 do roteiro da W2 deixou de existir neste desenho.

## Aberto

- **W3 — a trilha do Motion** (plano §2 e §5.3): gravação nos dois drenos de intenção, palco, gate 3. A
  sonda `crates/ph2d-app-motion/src/motion_bridge_macro_replay_probe.rs` tem o roteiro (24 intenções).
- Os outros meios do Painter (Digital, Watercolor, Impasto, Composite): o estado inicial deles não viaja
  (o `brush_by_mode` e o estado de cada meio). Ligar = estender `macro_estado.rs` e o censo dos controlos
  por meio — o gate do estado inicial já é por meio (`gestos(media, …)`).
- As recusas e erros (`Recusa`, `ErroDaMacro`) chegam ao artista pela UI da W5, por `ph2d-i18n`.
- As portas recusadas por escopo (camadas pela Hierarchy, formas, seleção, preenchimento, colagem) — cada
  uma tem o porquê em `portas.rs`; gravá-las é estender `Passo` + `tocar.rs` e mudar a decisão.
