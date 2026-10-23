# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W16 (o losango anda sozinho; duas pinceladas tocam ao mesmo tempo) → `main` (2026-10-22)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo integrador (DIRETRIZ §1.5.3).
> Mecanismo, medições e recusas: [doc W16](../15_W16_pinceladas_sobrepostas.md) · plano [14](../14_W16_plano.md) ·
> pedido [PROMPT_W16](PROMPT_W16_pinceladas_sobrepostas_2026-10-22.md).
> ⚠️ **W8–W15 ainda NÃO estão no `main`** (mesma branch): os handoffs
> [W15](HANDOFF_INTEGRACAO_line_Macros_2026-10-22_W15.md) … W8 continuam válidos para a parte deles.
> ⚠️ Ordem do dono (22/10): integrar esta linha depois da W16, **esperando na fila que `line/motion-value`
> integre primeiro**; o smoke fica para depois. A linha **NÃO foi rebaseada**.

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Base da W16: `c15bec075f`.
- Commits (`git log --oneline c15bec075f..HEAD`, do mais velho): `f3d06c1232` (núcleo `desloca_us`) · `3c6214a0a1`
  (Painter: sobrepor) · `0043305329` (UI `Instante`) · `f9587c99ec` (cauda; `=6 sobrepor`; `pedido_ao_largar`) ·
  `2dc6ce8dd8` (fotos + teste da edição vazia) · `2f70ea157b` (3 gates de lacuna) · `0a46c20a48` (fmt) ·
  `0b32441947` (curas do fecho) · o commit destes docs (último).

## §2 — O que a W16 entrega

Pedido do dono (22/10, verbatim no prompt) + «mover key não deve empurrar outros. siga».

1. **O losango largado move SÓ aquela etapa** (`EdicaoDeEtapa::desloca_us`; relógio fica; as outras ficam;
   a cauda segue a última etapa e o clipe encolhe/estica). Ficheiro: tag 9, MINOR 2 + `exige "etapa.desloca"`
   só quando usado (senão 1.1/1.0 como antes). A Espera do painel continua a empurrar.
2. **Duas pinceladas sobrepostas pintam ao mesmo tempo nos 4 meios** (Digital · Aguarela · Impasto · Wet Paint):
   um traço guardado por faixa, o partilhado fecha no último pen-up (grupo = UM passo do desfazer), o pincel
   viaja com a etapa (`Passo::PincelDaFaixa` = 200).
3. **Recusas em voz alta** (chaves i18n): fora do zero, muda o que o Desfazer desfaz, só pinceladas se
   sobrepõem, outro meio, e o Motion (`moves_with_the_next`).

### Testes novos (nome inteiro)

- `ph2d-macro` it `etapas::desloca` (8): `deslocar_move_so_a_etapa_e_o_relogio_fica`,
  `uma_etapa_passa_a_frente_das_outras`, `antes_do_zero_recusa_se`,
  `um_desfazer_gravado_continua_a_desfazer_a_mesma_etapa`, `para_la_do_fim_o_relogio_estende_se`,
  `deslocar_convive_com_espera_e_apagar`, `o_ficheiro_guarda_o_deslocamento_e_exige_quem_o_aplique`,
  `a_ultima_etapa_levada_para_tras_encurta_a_trilha`; `format.rs` cobre a tag 9.
- Painter: `sobrepor_tests` (`duas_pinceladas_sobrepostas_pintam_ao_mesmo_tempo_nos_quatro_meios`,
  `passar_a_frente_de_outra_e_grava_la_antes`, `sobrepor_um_preenchimento_ou_outro_meio_recusa_se`,
  `deslocar_para_o_sitio_dela_nao_guarda_edicao`) · `sobrepor_fecho_tests` (`o_quadro_corre_em_cada_traco_guardado`,
  `o_grupo_sobreposto_e_um_passo_do_desfazer`, `a_agua_do_wet_paint_fecha_no_ultimo_pen_up`) · `tracos_tests`
  (censo `cada_campo_do_paint_state_e_do_traco_ou_partilhado`, 196 campos).
- `ph2d-app-macros` `gate7_largar_o_losango_leva_so_aquela_etapa_e_sobrepoe` (novo); `gate7_gravar_pincelar…`
  adaptado (arrastar a etapa 3 move só ela; clipe +0,5 s com a cauda; desfazer por diff do mundo devolve o clipe).
- `ph2d-panel-timeline`: `arrastar_uma_etapa_mostra_so_ela_a_andar`, `largar_pede_o_instante_do_losango_mais_o_delta`.
- `ph2d-app-motion`: `largar_o_losango_de_um_gesto_do_motion_recusa_se_e_diz_porque`.
- `ph2d-app-painter` caminho: `a_sobrepor_recua_o_losango_da_segunda_depois_de_parar_e_antes_de_tocar`.

### §2-bis — Medição e mutação

Sonda 22/10 (128², dois traços disjuntos, o azul levado a começar a meio do vermelho). ANTES: o Down azul levava
o traço vermelho (topo 4911 texels contra 3938 em série; azul pintado de vermelho, R 756712 · B 287704); Wet Paint
roxo (R 1038970 · B 1050033). DEPOIS: Digital byte-idêntico à série; Aguarela/Impasto/Wet Paint cada região a sua
cor, contagens a 2 %; a meio da sobreposição as duas regiões mudam no mesmo quadro; reordenar dois traços que se
cruzam = byte-idêntico a gravar de novo trocado; ▷ repete ao byte.
**Mutação: 18; 15 sangraram à 1.ª; M3 (sem `tique_das_faixas`), M4 (`sozinho=true`), M8 (sem a guarda
`wetpaint_stroke_end`) sobreviviam → 3 gates (`sobrepor_fecho_tests`) → 18/18.**

## §3 — Foundational / partilhado tocado (aditivo)

| Crate | O quê |
|---|---|
| `ph2d-macro` | `EdicaoDeEtapa.desloca_us` · `ErroDeEdicao::{ComecaAntesDoZero, MudaOQueODesfazerDesfaz}` · `LeitorDaFamilia` +3 métodos com defeito (`na_faixa`, `marca_da_faixa`, `e_barreira`) · codec MINOR 2 |
| `ph2d-editor-core` | `CampoDaEtapa::Instante` · `desloca_para` · consts de erro/CHAVES |
| `ph2d-i18n` | 5 chaves `panel.macros.error.*` |
| `ph2d-tool-painter` | `PaintState.tracos_fora` · `stroke_lifecycle` (gating) + `paint_tick` partido · `wetpaint.rs` (guarda) + `wetpaint/faixas.rs` · `tracos.rs` · `Passo` 200 + tag 15 |
| `ph2d-panel-timeline` | `pedido_ao_largar` (pub) |
| `ph2d-app-macros` · `ph2d-app-motion` · `ph2d-app-painter` | Instante; recusa do Motion; `caminho.rs` (`=6 sobrepor`) |
| `host-desktop` (shell) | `macro_smoke.rs` (+1 braço) |

Ficheiros novos: `tracos.rs`, `tracos_tests.rs`, `wetpaint/faixas.rs`, `etapas_tocadas.rs`, `etapas_desloca.rs`,
`sobrepor_tests.rs`, `sobrepor_fecho_tests.rs`, `ph2d-macro/tests/it/etapas_desloca.rs`.
Contratos congelados (§6) **não tocados** (o `Tool` não mudou: a API de faixas são métodos inerentes). Sem
`Cargo.toml`, crate nova, ADR ou `PROJECT_SCHEMA`.

## §4 — O que um merge pode partir

- ⛔ **`line/PainterWatercolor`** toca `state.rs`, `stroke_lifecycle.rs`, `wetpaint.rs`, `wetpaint/session.rs`,
  `stamp_route.rs`: (1) **todo campo NOVO do `PaintState`** tem de ser classificado em `tracos.rs`
  (`DO_TRACO` ou `PARTILHADO`) ou o censo reprova — o integrador classifica; (2) `state.rs` está em **700 LOC
  exactos** aqui: a árvore combinada pode passar o tecto (cura por corte, nunca isenção); (3) `wetpaint.rs` ~674;
  (4) `Passo` 53–56 (deles) contra 53–56 (nossos) segue aberto (W14 §4): a W16 só acrescenta o 200.
- Um `match` novo sobre `CampoDaEtapa` ou `ErroDeEdicao` noutra linha deixa de compilar (bom).
- `paint_tick` partido: trabalho NOVO por traço, no fim de `paint_tick`, vai para `paint_tick_do_traco`
  (correcto); trabalho global tem de ficar acima da chamada.
- Os §4 da [W15](HANDOFF_INTEGRACAO_line_Macros_2026-10-22_W15.md) (`ScrubExact`, `.fim`) e da
  [W14](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W14.md) continuam.

## §5 — Contadores (DELTA sobre `c15bec075f`, só a W16)

Crates novas 0 · chaves i18n +5 · ADR 0 · `PROJECT_SCHEMA` 0 · contratos 0 · cenas de smoke: nenhum número novo
(`=6` ganhou `PH2D_MACRO_CAMINHO=sobrepor`) · ficheiros novos 8 · tag de `Edicao` +1 (9) · tag de `Passo` +1 (15) ·
`Passo` derivado +1 (200) · MINOR do ficheiro da Macro 1 → 2 (só quando usado).

## §6 — Prova de fecho

| Portão | Resultado |
|---|---|
| `cargo fmt --all -- --check` | ✓ |
| clippy `-D warnings --all-targets`: macro, tool-painter, app-macros, app-motion, app-painter, panel-timeline, editor-core, i18n, host-desktop | ✓ |
| `PH2D_SUITE=fecho` `nextest-impacted.sh` (BASE = merge-base, em `0a46c20a48`) | «Summary [1046.8s] 19430 tests run: 19428 passed (32 slow), 2 failed, 10547 skipped», `load` 55–110 |
| as 2 falhas (determinísticas), curadas em `0b32441947` | (A) `every_word_this_family_shows_comes_from_the_string_table` (HR-15): o texto da `=6 sobrepor` estava em `caminho_cef.rs`, passou para `caminho.rs` (registo isento); (B) `architecture_workspace_file_loc_cap`: `state.rs` 702 > 700, doc encurtado → 700 |
| re-corrido por nome depois | ✓ as duas · `censos-da-arvore-combinada.sh` ✓ 12/12, 121/121 · `the_shell_only_shrinks` · `no_orphan_source_file` · `claude_md_cabe_no_orcamento` · `tool_contract_surface` ✓ |

⚠️ A suíte inteira NÃO foi re-corrida depois de `0b32441947` (só texto movido + comentário de doc): o integrador corre a sua.

`agent-loop-profile.sh`: cargo test:check 324 : 124 (2,6×) · contexto por passo 269 mil · edições sem conferir 0,7 %.

## §7 — Smoke do dono (ADIADO por ele) e fotos

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=sobrepor ./target/smoke/ph2d-host-desktop`
   (depois da integração o binário é o do `main`).
2. Não mexa no rato: a cena grava, pinta um traço PRETO (Digital), escolhe o vermelho no Conta-gotas e faz um
   traço vermelho por baixo, pára, arrasta o losango da 2.ª etapa 1,85 s para trás, volta ao início e toca.
3. O que tem de acontecer: as duas pinceladas passam a ser pintadas AO MESMO TEMPO; a vermelha começa quando a
   preta vai a meio, cada uma na sua cor; a linha «Macro steps» tem os losangos mais juntos e o laço encurta.
4. Errado: a vermelha sai preta (ou roxa); a 1.ª pincelada anda junto com a 2.ª; o clipe não encolhe.

Fotos (`smoke_2026-10-22_W16/`): [as duas a meio, 1,35 s](smoke_2026-10-22_W16/sobrepor_01_as_duas_a_meio_t1.35.png) ·
[a preta acabada, a vermelha a meio, 1,73 s](smoke_2026-10-22_W16/sobrepor_02_a_preta_acabou_a_vermelha_a_meio_t1.73.png) ·
[log](smoke_2026-10-22_W16/sobrepor.log) (`steps` antes `["1.62","3.78"]` → depois `["1.62","1.94"]`, laço 4,76 → 2,899 s).

## §8 — ABERTO

1. **Smoke do dono da W16** (adiado por ele).
2. **O Motion recusa a colocação livre** (`moves_with_the_next`): perguntar ao dono se o Motion também deve mover
   sozinho — pede a lei do `sujou` no Quadro.
3. **A união da Aguarela** tinge um pouco o traço anterior com o pincel posterior (lei da sessão húmida
   contínua: uma lavagem; motor da `line/PainterWatercolor`) — reportado, não mudado.
4. O tutorial «fazer» sobre uma trilha com etapas sobrepostas não tem teste (o mapeamento usa `tocadas`; a ordem
   é a da gravação).
5. W15 §8: **Dur(s) 6,208** (pedir ao dono a sequência de gestos) e o veredito do smoke da W14; seguem de pé os §8
   da W14, W13 e W12.

## Para o integrador (`CLAUDE.md` §5.1, entrada «Macros & Automação»)

Novo «Último» = este handoff (W8–W15 valem para a parte deles). Integrar SÓ por ordem do Enio, **depois** de
`line/motion-value` estar no `main`.

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
