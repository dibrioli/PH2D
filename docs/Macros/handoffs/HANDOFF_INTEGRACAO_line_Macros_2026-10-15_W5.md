# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W5 (ETAPAS) → `main` (2026-10-15)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo de cada wave: [plano W5](../02_plano_W5.md) (§7 = W5e) e o
> [handoff de continuação W5a..W5d](HANDOFF_CONTINUACAO_W5_ETAPAS_2026-10-14.md) (tabela de commits da W5a–W5d,
> o que existe, gates, mutação 21/21 · 13/13 · 18/18). Antes: [W4](HANDOFF_CONTINUACAO_W4_COMPONENTE_2026-10-13.md).

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`.
- Merge-base com o `main`: `0aab2c1cb6` (o `main` NÃO andou; 0 atrás). HEAD de código: `77928f37a5`
  (os commits de documentação deste handoff vêm por cima).
- `git diff --stat 0aab2c1cb6..HEAD`: 264 ficheiros, +14 383 / −558 (a linha inteira, W4 incluída);
  só a W5e (`024b153672^..HEAD`): 65 ficheiros, +2 524 / −486.
- Linha única ⇒ a ordem de integração é trivial; o conflito possível é com `line/PainterWatercolor` (§3).

### Commits da W5e (de `024b153672` ao HEAD)

| commit | o quê |
|---|---|
| `024b153672` | apagar uma etapa fecha o tempo dela; a cena compõe 1× por quadro e sobe só o que mudou |
| `c38c44b771` | rustfmt em 3 ficheiros da linha (só formato) |
| `386654361d` | a conferência das edições de valor sai do quadro, recomeça do instantâneo do palco e é derivada dos bytes |
| `dc3d49d001` | os dois riscos do smoke — a sprite gravada mostra o traço vivo; o Motion grava no relógio do quadro em que correu; as cenas mostram as etapas |
| `f6dd4b8878` | as sobreviventes da mutação (15/20): 3 mutantes equivalentes saem do código, 2 gates novos, um nome já não tira a etapa errada da gravação |
| `550ada5e58` | o limite da partida passa para dentro da conferência (M22 não era redundante); a prova de mutação no plano |
| `ac87fda10c` | fecho: o teste da luz passa a fatia das macros, `clone_on_copy` nos instantâneos, rustfmt nos testes da Timeline, tectos de LOC por corte de comentário |
| `f0474060ba` | a família MACROS deixa de depender da família Motion — o vocabulário das etapas desce para a folha e a shell injecta o Motion |
| `77928f37a5` | as reprovações da suíte impactada — a família MACROS no registo, a unidade no campo, os passos nomeados, a sonda com classe e o instantâneo que não depende do relógio do worker |

W5a–W5d (`5b2f45636d`..`0fbff868c5`): tabela no handoff de continuação (§«Commits da W5»).

## §2 — O que a W5e fez

- **Apagar etapa** (dono, 15/10: «o tempo dela fecha-se»). `EdicaoDeEtapa.apagada` (codec tag 7 dentro de
  «etapas» 1.1). `efetiva` salta os eventos da etapa (e o relógio de dentro); o que vem depois recua
  `t[fim]−t[inicio]`. As etapas vivas levam o `k` da GRAVAÇÃO. Motion: `LeitorDaFamilia::sem_o_gesto` (o `Quadro`
  do fim de uma etapa apagada passa a `sujou:false`) e `tempo_deslocado` encaixa o `tempo` na grelha do `dt`.
  UI: «Delete step» no painel MACROS, `MacroRequest::Apaga`, undo pelo diff do mundo.
- **Composição 1× por quadro + subida parcial.** `PainterTool::drena_sem_compor`; a cena compõe UMA vez sobre o
  retângulo sujo acumulado; o palco sobe só esse retângulo (`RanhurasDaCena::substitui_regiao`).
- **Conferência fora do quadro** (`ph2d-app-macros/conferencia.rs`): derivada da `Macro` (tique + hash dos
  bytes), num fio «macro-confere», no 1.º quadro sem mudança nova, a partir do instantâneo do palco
  (`MacroEmCena::partida`, `limite_da_partida` dentro de `confere`); só escreve se a `Macro` ainda for a
  conferida; os mesmos bytes que voltam (undo/redo) recebem a decisão já. O undo suprime o passo enquanto há
  conferência («uma macro a conferir», chave `shell.undo_app.macro_a_conferir`).
- **Os dois riscos do smoke:** (1) a sprite que o Painter grava cede ao traço vivo (`Macros::sprite_gravada` +
  `sobe(.., cede)`); (2) o Motion é drenado DEPOIS da `fase_motion_bridge` (`Macros::depois_do_motion`).
  Cenas `PH2D_MACRO_SMOKE=1/2` abrem o painel MACROS; o Splash grava pela porta do produto (`grava_no_papel`).
- **Fecho (desacoplamento):** a família MACROS não depende mais da família Motion — trait `EtapasDaFamilia`
  em `ph2d-editor-core::macro_etapas`, `EtapasDoMotion` em `ph2d-app-motion/macro_track/vistas.rs`, a shell
  injecta. Família MACROS no registo `FAMILY`/app-sync. Unidade nos campos (`labeled_number_field_unit`).

## §3 — Foundational / partilhado tocado (para o integrador)

**União W5a–W5d** — copiada do handoff de continuação (§«Foundational / partilhado tocado»): `ph2d-macro` (formato
1.1: `Track.edicoes`, `exige`, `CodecError::Exige`, `Gravadora`, `advance_to_limit`) · `ph2d-tool-painter`
(`wetpaint.rs`, `wetpaint/offthread.rs`, `brush_core_settings.rs`, `tool/instantaneo.rs`, `tool/mod.rs`,
`derive(Clone)` em ~40 ficheiros de `tool/paint/`, portas, `bound_document`, `macro_track/*`) · `ph2d-app-motion`
(`macro_track/*`, `motion_state_fita_demo.rs`) · `ph2d-core` (`razao.rs` novo) · `ph2d-eval-motion` (`razao.rs`
reexporta; `Razao::da_fita/dos_aneis` → `razao::da_fita/dos_aneis`; `Cargo.toml` +`ph2d-core`) · `ph2d-wet-paint`
(`painter/instantaneo.rs`, `derive(Clone)`, `SolverScratch`) · `ph2d-painter-brush` (3 `derive`) ·
`ph2d-app-painter` (palco reedita) · `ph2d-editor-core` (`macro_edits`, `EditorAction::Macro`,
`TimelineHitKind::MacroStep`, ids/chrome macros, menus, ícone, `scrollbar_ids` 851, LIVE_SECTIONS, `ComponentEdit::Macro`)
· `ph2d-timeline` · `ph2d-panel-timeline` · `ph2d-panel-inspector` · `ph2d-app-components` · `ph2d-i18n` · registry-init ·
shell · crates novas `ph2d-app-macros`, `ph2d-panel-macros`.

**W5e, a somar:**

- `ph2d-macro`: `EdicaoDeEtapa.apagada`; codec tag 7 dentro de «etapas» 1.1; `LeitorDaFamilia::sem_o_gesto`
  (método com defeito); `Track::apaga_etapa`.
- `ph2d-editor-core`: `macro_etapas.rs` NOVO (`EtapasDaFamilia` + chaves de erro); `MacroRequest::Apaga`;
  `RowCtx::labeled_number_field_unit`; **dependência NOVA** em `ph2d-macro` (`Cargo.toml`).
- `ph2d-tool-painter`: `tool/runtime.rs` — `take_preview_arc_corpo` partido em `drena_o_estado` /
  `drena_sem_compor` / `compoe_a_saida`; `layers/model.rs` `mask_grayscale_view_source`; `macro_track/*`.
  ⚠️ **SUPERFÍCIE DE COLISÃO com `line/PainterWatercolor`** (também toca a ferramenta do Painter): rode
  `collision-surface.sh` pelo caminho absoluto em cada worktree antes de fundir `tool/runtime.rs`.
- `ph2d-wet-paint`: `clone_on_copy` nos instantâneos.
- `ph2d-app-painter`: `Cargo.toml` `ph2d-macro` dev→dep; `RanhurasDaCena::substitui_regiao` (método NOVO do
  trait — um implementador novo noutra linha não compila sem ele); `sobe(.., cede)` mudou de assinatura.
- `ph2d-app-motion`: `macro_track/vistas.rs` (`EtapasDoMotion`), `macro_track/etapas.rs` (`sem_o_gesto`,
  `tempo_deslocado`, apagar).
- `ph2d-app-macros`: `conferencia.rs` (já NÃO depende de `ph2d-app-motion`).
- `ph2d-panel-macros`: botão Delete step. `ph2d-panel-timeline` `tracks.rs` e `ph2d-timeline` `snapshot.rs`:
  só cortes de comentário (tectos de LOC).
- `ph2d-i18n`: `panel.macros.delete_step`, `shell.undo_app.macro_a_conferir`, textos das unidades.
- Shell: `fase_macros`; `fase_hero_frame` +`depois_do_motion`; `fase_extract_inputs` +`cede`; `undo_app` (motivo);
  `macro_smoke` abre o MACROS; `fase_inspector_commits_luz_tests` +1 argumento.
- Registo: `FAMILY` com a família macros; app-sync regenerado.

Contratos congelados (§6): **nenhum**. `PROJECT_SCHEMA`: não muda. Formato da macro 1.1 ganhou a tag 7;
nenhum ficheiro 1.1 saiu da linha (um leitor 1.0 do `main` já recusa via `exige`).

## §4 — O que um merge pode partir

- `tool/runtime.rs` (Painter) com `line/PainterWatercolor`: a divisão em `drena_o_estado` /
  `drena_sem_compor` / `compoe_a_saida`. Se a outra linha mexeu em `take_preview_arc_corpo`, a cura é pôr a
  mudança dela no corpo comum (`drena_o_estado`), nunca duplicá-la; gate `substrate_relief_tests::raising_the_relief_repaints_without_a_stroke`.
- Um CAMPO novo de `PainterTool`/`WetSession`/`Engine` vindo de outra linha não compila (ou o gate de
  instantâneo reprova) até entrar no padrão do `instantaneo`/`clone` — é o gate a funcionar.
- `RanhurasDaCena::substitui_regiao` e `sobe(.., cede)`: um chamador/implementador novo noutra linha precisa
  das duas.
- Registo de famílias (`FAMILY`) e app-sync: **recontar** no ficheiro fundido (contador soma entre linhas),
  `python3 scripts/schema-recount.py` não é necessário (schema não muda).
- `Cargo.toml` de `ph2d-editor-core` (dep nova `ph2d-macro`) e de `ph2d-app-painter` (dev→dep): conferir o DAG
  (gate de fundação verde na linha).
- Tectos de LOC somam entre linhas (cortes de comentário em `tracks.rs`, `snapshot.rs`).

## §5 — Contadores (DELTA, desta linha sobre o merge-base)

- Crates novas (da W5d): 2 (`ph2d-app-macros`, `ph2d-panel-macros`). W5e: **0** novas.
- Chaves i18n novas na W5e: `panel.macros.delete_step`, `shell.undo_app.macro_a_conferir`, textos de unidade.
- Família no registo `FAMILY`: **+1** (MACROS). Schema/ADR/contratos: 0.
- Recontar tudo contra o `main` do dia ao integrar — o número é o do código, não o deste texto.

## §6 — Medido (W5e, 15/10, perfil smoke, mínimo de rondas intercaladas; detalhe: [plano §7](../02_plano_W5.md))

| | antes | depois |
|---|---|---|
| Splash no papel 2048², cena (segue + compor) | 12,8 + 18,2 ms | 10,1 + 8,7 ms |
| Splash no papel 1024², idem | 5,3 + 3,5 ms | 4,2 + 1,9 ms |
| tocar do zero, Splash no papel 1024² / 2048² | 2 043 / 3 622 ms | 1 102 / 2 441 ms |
| tocar do zero, longa 61 s 1024² / 22 s 2048² | 26,6 / 17,7 s | 22,9 / 13,5 s |
| subir à placa 2048² (inteira / 25 % / 5 %) | 8,53 / 2,49 / 0,49 ms | — |

| conferir a cor | do zero | do instantâneo | tirar o instantâneo (no quadro) |
|---|---|---|---|
| longa 61 s, 1024²: pincelada 96/96 | 24,5 s | 3,0 s (fora do quadro) | +7,9 ms |
| longa 22 s, 2048²: 32/32 | 19,4 s | 7,6 s (fora do quadro) | +16,8 ms |

Antes a conferência tocava a macro inteira DENTRO de um quadro (3,6–26,6 s). O que sobra é a água (`Tique` =
70–80 % de tocar do zero). Load antes 23–29, depois 4–55: relógio de máquina carregada, vale pelo mínimo.

⛔ Recusas MEDIDAS: [plano §7.4](../02_plano_W5.md#74--recusas-medidas) (régua «a cena = o editor» diverge no
código antigo; conferir dentro do quadro; conferência só pelo pedido de edição).

## §7 — Prova de mutação W5e (15/10, agente `mutacao`, 3 rondas)

1.ª ronda **15/20**; todas fechadas e vistas a sangrar. M13 e M14 eram EQUIVALENTES (a marca de saída não é
resposta do worker; a mudança da `Macro` já pára o fio) ⇒ saíram do código. M12/M22: o limite da partida NÃO era
redundante ⇒ passou para dentro de `confere`, gate com partida tardia. M8: UMA função `drena_o_estado` com dois
chamadores (sangra em `substrate_relief_tests::raising_the_relief_repaints_without_a_stroke`). M10: `recorta`,
com gate fora do canto. Achado: um NOME numa etapa anterior tirava-a da gravação (`primeira_que_toca`; gate em
`macro_etapas`). O gate G2 (Painter) não vê o tempo nem a marca de saída nos píxeis — o núcleo (G1) cobre-os.

## §8 — Gates (por nome; lista completa e comandos: [plano §6 e §7.5](../02_plano_W5.md#75-gates-w5e-todos-verdes-por-nome))

`ph2d-macro --test it etapas::apagar_uma_etapa_fecha_o_tempo_dela` · `ph2d-tool-painter --lib
macro_track::etapas_tests::apagar_uma_pincelada_e_grava_la_sem_ela`, `macro_track::composicao_tests` ·
`ph2d-tool-painter --test it macro_etapas::conferir_do_instantaneo_do_palco_decide_o_mesmo_que_do_zero` ·
`ph2d-app-motion --lib macro_track::etapas_tests::apagar_um_gesto_e_grava_lo_sem_ele` · `ph2d-app-painter --lib
macro_track::palcos_tests::a_ranhura_recebe_so_o_que_mudou_e_fica_igual_a_tela` · `ph2d-app-macros --test it
gate7` · `ph2d-panel-macros --test it seam_macros::apagar_a_etapa_escolhida_chega_como_pedido_e_larga_a_escolha` ·
`ph2d-app-registry-init` · `motion_ignore_census` · arquitectura do `ph2d-editor-core` (111+ verdes).

## §9 — Prova de fecho (15/10)

- Suíte impactada: `BASE=main PH2D_SUITE=fecho bash scripts/nextest-impacted.sh` — final **19 726 passed / 0
  failed**. A 1.ª corrida teve 6 falhas, curadas em `77928f37a5` (a família MACROS no registo, a unidade no
  campo, os passos nomeados, a sonda com classe, o instantâneo); uma era flake de carga que passou a
  DETERMINÍSTICA: `macro_instantaneos::o_instantaneo_que_consumiu_uma_resposta_mudada_nao_recomeca` (já não
  depende do relógio do worker).
- clippy `--all-targets -D warnings`: limpo em 17 pacotes. `cargo fmt`: limpo.
- `bash scripts/censos-da-arvore-combinada.sh`: 120 passed, 12/12 censos.
- Última verde no `main` antes da linha: 20 874/20 874 (integração de 14/10) — ao integrar, repetir a suíte na árvore combinada.

## §10 — ABERTO

- «Tutorial fazer» (balão, realce, cena `=3`): **W6**.
- Os outros meios do Painter (fica de antes).
- Pintar SEM gravar depois do Parar continua escondido enquanto o clipe toca (por desenho: a sprite mostra a
  macro). Decisão de produto: perguntar ao dono SE incomodar no smoke.
- O roteador `PH2D_MACRO_SMOKE` fica na shell (cenas do Painter e do Motion): movê-lo recriaria a aresta
  família→família.
- Fotos com a máquina calma; o custo da conferência numa macro longa já está medido (§6) — o que sobra é a água.

## Smoke

(o binário: será colado)

## Fotos

(será colado)

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`;
inner loop `bash scripts/cargo-check-narrow.sh <crate>` (cego ao código atrás da feature `panel-motion-graph`
e aos `tests/`: use `bash scripts/ph2d-run.sh cargo test -p <crate> <filtro>`).
