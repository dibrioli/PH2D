# Continuação — line/Macros-Automação, W4 itens 1–4 fechados (fronteira do palco, alvo, componente) → item 5 (Timeline, Play, o desenho dos palcos)

> Para a PRÓXIMA janela. Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`, base
> `757afebd58`. Leia antes de agir:
> - o [ADR-0186 §Emenda W4](../../architecture/decisions/0186-a-macro-grava-comandos-resolvidos-no-relogio-da-macro.md) (e a W3);
> - o [handoff da W3](HANDOFF_CONTINUACAO_W3_MOTION_2026-10-13.md) e o [plano da onda 1](../01_plano_onda1.md).

## Estado (commits desde a W3)

| commit | o quê |
|---|---|
| `9ebdf44f13` | o palco não guia o painel do editor nem é guiado por ele |
| `2e8e4403eb` | o palco recebe a placa e o mundo; o fiel confere as marcas da bomba |
| `02d083459a` | o alvo: a trilha abre com o objecto em edição; o palco coze só o dono |
| `53b6fd7cff` | o componente `Macro`; `PROJECT_SCHEMA` 188 → 189 |
| `75afc54516` | os vermelhos da suíte (contagens dos registadores, censo do Inspector) |

## O que existe (não reconstruir)

- **Painel na instância** (`9ebdf44f13`): `MotionState.ao_painel: PedidosAoPainel { selecao, params }`
  (`motion_state_ao_painel.rs`; `pede_selecao`, `encaminha_param`). 12 sítios `request_graph_selection`
  (motion_bridge_{edit,rewire,heal,intents,subgraph,adapt}) e 5 `push_param_intent` (intents 3, choices 2;
  `write_source`/`write_channel` recebem `&mut MotionState`; `step_choice`/`pick_choice` passam a `&mut`).
  Só o editor publica: `intents::publica_no_painel(motion)` no fim do `apply_graph_intents` e no `dispatch`
  depois do `subgraph::clamp_level`. Leituras: `quadro::MaoDoEditor { selecionado, arrasto }` é argumento de
  `o_quadro(motion, gpu, playhead, dt, mao)`; `warp_gizmo::taps_for(motion, selecionado)` + `warp_de`,
  `pivot_gizmo::taps_for(motion, arrasto)` + `forma_com_pivot`. Censo: flash e `set_card_*` NÃO são
  alcançáveis do palco. Gate red-first `macro_track::painel_tests` (3, vistos vermelhos).
- **Mundo e placa** (`2e8e4403eb`): `motion_mundo.rs::publica_o_mundo(motion, Cena{…})` (saiu da shell
  `fase_motion_bridge.rs`, que encolheu ~25 linhas: três chamadas viraram uma) guarda
  `motion.mundo: MundoDaCena { externos, jogo, donos }`. `MotionState::recebe_o_mundo` e
  `TocadorDoMotion::recebe_o_mundo`. `TocadorDoMotion::aplica(carga, placa: Option<&GpuContext>)`; o palco
  nasce com o `gpu_enabled` do produto.
- **Marcas da bomba**: `MotionCookPump::sujidades()` (contador) no irmão novo
  `crates/ph2d-eval-motion/src/sujidade.rs` (mark_dirty / mark_dirty_keeping_ring / is_dirty movidos;
  `lib.rs` 697 → 665; `isola_na_camada` marca via keeping_ring). O passo `Quadro` renomeia `sujo` → `sujou`
  (etiqueta 5 mantida; «marcada desde o quadro anterior»; nenhum `MacroDoc` persistido existia). A
  testemunha grava `Toque.sujou` pela mesma contagem. Gates: `motion_mundo::tests` (2, cada um com o controlo
  sem o mundo; a cena usa duplicator+source.shape porque «uma corrente só de posições não desenha»,
  lower.rs); `macro_track::placa_tests` (2, `#[ignore]` placa): a sessão do gate 3 na placa toca no palco na
  placa AO BIT (`retrato_visto` lê `read_instances`/`read_formas`); o fiel atravessa placa↔CPU (vermelho
  visto: `Divergiu`). Mutações sangram: fiel que nunca acusa; testemunha sem a marca.
- **Alvo** (`02d083459a`): `MotionState::gravacao_abre(rec)` = arma + `open_track(FAMILIA,
  objeto.editing.unwrap_or(0), inicial)`; `TocadorDoMotion::da_trilha(track, modo)` →
  `palco.objeto.so_o_dono`; `sinks_visiveis` filtra pelo dono. Gate red-first `macro_track::alvo_tests` (2).
- **Componente** (`53b6fd7cff`): `ph2d_ecs::Macro { doc: Vec<u8>, modo: ModoDaMacro { Fiel, Explicativo } }`
  em `crates/ph2d-ecs/src/macro_doc.rs`; o `doc` é o `MacroDoc` pelo `ph2d_macro::codec` (cabeçalho
  `PH2DMACRO` major/minor). Descritor INTRÍNSECO em `ph2d-component-desc` `catalog/logic.rs` (`C::Animation`
  vizinho; nasce do Gravar▸Parar, a paleta não o oferece), campo 2 = modo (`K::Enum`). i18n
  `component.macro.name`, `component.field.macro_fields.2`, `ph2d-i18n/src/inspector_macro.rs`
  (`newer_version`, `damaged`). Modelo do cartão: `ph2d-editor-core/src/macro_edits.rs` (`MACRO_MODOS`,
  `MacroFieldEdit::Modo`, `MacroGravacao {Vazia, Lida{trilhas, duracao_us}, Ilegivel(chave)}`,
  `InspectorMacroInfo`) e `ph2d-app-components/src/macro_inspector.rs` (`build_info/apply/apply_all/
  gravacao`; dep nova `ph2d-macro`). Gate 8 verde; o descritor apagado reprova-o (visto).
- **Vermelhos** (`75afc54516`): contagens `ph2d-render` e `ph2d-script` `registry.rs` 108 → 109; censo das
  palavras do Inspector (`inspector_macro.rs` na lista; os nomes dos modos saíram — chave sem leitor — até a
  W5 os pintar).

## Foundational / partilhado tocado (para o integrador)

`ph2d-eval-motion` (`sujidade.rs` novo, campo `sujidades`) · `ph2d-ecs` (`Macro`, `ModoDaMacro`, registo) ·
`ph2d-component-desc` (descritor) · `ph2d-i18n` (chaves + ficheiro) · `ph2d-editor-core` (`macro_edits.rs`) ·
`ph2d-app-components` (`macro_inspector` + dep `ph2d-macro`) · `ph2d-panel-inspector` (só a lista do teste de
censo) · `ph2d-render`/`ph2d-script` (contagens) · `shells/desktop`: `fase_motion_bridge.rs` encolheu (delta
negativo), `project_schema(.rs/_tests.rs)`. Contratos congelados encostados: **nenhum**.

**Contadores como delta:** `PROJECT_SCHEMA` 188 → 189 (escada + tripla `(189,13,22)`); registo ECS +1
(107 → 108; render/script 108 → 109). ⚠️ `python3 scripts/schema-recount.py` NÃO conta: só renumera um
degrau DENTRO de um conflito de rebase (lê `:1/:2/:3`). A conta da linha é o delta (+1 registado); o
integrador reconta com o script.

**O que um merge pode partir:** outra linha que some um componente ECS ou suba o schema (contagens de
`registry.rs`, escada); `fase_motion_bridge.rs` (as 3 chamadas viraram `publica_o_mundo`); quem chame
`request_graph_selection`/`push_param_intent` novos fora das portas da instância (o palco guiaria o painel);
quem escreva `Quadro.sujo`.

## Medido

**Suíte** (`PH2D_SUITE=5-commits`, HEAD `53b6fd7cff`): `20 839/20 843` em `659 s`, loadavg 33–59 (outra
linha corria a suíte). 3 vermelhos reais, curados em `75afc54516`. O 4.º, `ph2d-light2d-gpu
g4_o_custo_do_rc_nao_depende_do_numero_de_luzes` (8,25 % > 5 %), é flake de carga ([FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)):
verde sozinho a load 25. ⇒ **a última suíte VERDE da linha continua a ser `dbcca83405`**; desde ela, 6
commits de código ⇒ a próxima janela corre a suíte cedo (ou no fecho), com `PH2D_SUITE=…`.

Smokes da placa (comando inteiro):
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --profile smoke -- --ignored macro_track::placa`
(o resto: `bash scripts/cargo-test-narrow.sh ph2d-app-motion`). Smoke do Enio: ainda nenhum — o cartão pintado é W5.

## Premissas do briefing derrubadas pela medição

- O `TimelineDoc` não é componente: é `ProjectFile.timeline: Vec<u8>` com `DOC_VERSION` próprio; o «molde» é o blob com versão no cabeçalho.
- Autoplay/laço/duração NÃO entram no componente (do clipe na Timeline; duas verdades divergiriam).
- O `TocadorDoPainter` vive em `crates/ph2d-tool-painter` (o plano dizia `ph2d-app-painter`).
- Crates `ph2d-app-*` não dependem umas das outras (só `ph2d-app-registry-init`) ⇒ a «ponte do Play» não pode ser uma crate que chame Motion e Painter: cada família toca a SUA trilha lendo o componente.
- O cartão PINTADO no Inspector (`ph2d-panel-inspector` sections/sync/populate/event/ids + editor-core `action_bus` + shell `fase_bus_inspector`…) é W5 (molde `a777f2f541`); a W4 entregou o modelo.

## ABERTO — item 5 (próxima janela), com o desenho já medido por leitura

1. **A Timeline dirige o tempo.** `PropKind::MacroTime = 18` (appended; `prop.rs from_target` COM a linha 18 — o
   `_ => None` engole variantes, ver o comentário do 17 —, `i18n_suffix`, fora de `ALL` e `AUTOKEYED`; rótulo
   em `ph2d-panel-timeline` `tracks_label.rs`; molde do commit `52174a7d35` IkBendSide). Sem `DOC_VERSION`
   (variante não é campo posicional). `write_prop` (`crates/ph2d-timeline/src/apply_prop.rs:23`) insere um
   componente de RUNTIME não registado `ph2d_ecs::MacroTempo { segundos }` (molde `VecDrivenStyle`: sem
   Serialize, fora do save/undo).
2. ⚠️ **A peça SEM precedente:** quando a Timeline deixa de dirigir, NADA repõe nem remove (o ledger da
   preview cobre só Transform/tint/VecMorph/PhysicsJoint; `settle` promove o vivo a documento) ⇒ limpar o
   `MacroTempo` uma vez por quadro ANTES do apply, no início da aplicação da timeline na shell
   (`timeline_bridge.rs`, antes do `if let Some(c) = container`; o `toca_os_objectos` do Play corre no mesmo
   fluxo), e ver que a fase corre todo quadro.
3. **A porta de criação do clipe:** `add_clip_for(name, owner)` → `set_active` → `bind(MacroTime, owner)` →
   keys LINEARES `(0,0),(D,D)` → `set_clip_length_override(D)`; laço no Play = `keys_loop_range` (o
   `clip_local_time` dobra pelo keys_loop, NÃO pelo loop_range); armadilha: um `TimeRemap` no dono remapeia.
   Molde `shells/desktop/src/render_loop/animacao_smoke.rs:14-30,115-161`. O Arrange (`stack_eval`) não foi auditado.
4. **O tocador segue o tempo:** avançar (`MacroPlayer::advance_to`) e para trás `MacroPlayer::seek` (reset =
   `TocadorDoMotion::da_trilha` de novo). Gate: scrub para trás dá o mesmo retrato que tocar do zero até lá.
   Os instantâneos que encurtam o seek ficam por MEDIR (plano «Abertos»).
5. **A cena (um bloco, nunca em partes):** o sistema do Motion `toca_as_macros(editor_motion, world, gpu)`
   (palcos por entidade em `MotionState`, só com `Macro`+`MacroTempo`); o editor EXCLUI os sinks do dono cujo
   palco toca; e ⚠️ **O DESENHO:** o quadro desenha hoje UMA saída do Motion em vários passes
   (`fase_vector_bands.rs:120-145` placa/misturas, `present.rs:444` formas do dispositivo, `present_fx` glow,
   `present_luz`; os gates `present_placa_tests` leem o fonte). Decisão técnica (padrão-ouro): o present
   aprende a desenhar cada fonte do Motion (editor + palcos) pelas MESMAS portas (uma `PlacaDeFormas` por
   palco). ⛔ Excluir o dono sem desenhar o palco é pior que não existir (o objecto sumiria no Play). Também o
   Painter: o `TocadorDoPainter` em cena por objecto (a imagem do objecto).
6. Da W3/W2 ainda abertos: recusas/erros chegam ao artista pela UI da W5; os outros meios do Painter;
   re-medir o custo do quadro a tocar com a máquina calma.

## Régua do laço do agente

`bash scripts/agent-loop-profile.sh` (20 sessões mais recentes, 13/10):

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                228   alvo: <= 800
  ✗ cargo test : cargo check                724 : 153   alvo: <= 1,0  razao 4.7x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  47%   alvo: >= 80%  (702 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         358 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```
