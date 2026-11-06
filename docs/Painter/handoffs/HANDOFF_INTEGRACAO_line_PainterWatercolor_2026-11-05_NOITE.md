# HANDOFF DE INTEGRAÇÃO — linha PainterWatercolor, noite de 05/11 (§1.5.9)

**Estado: PRONTA PARA INTEGRAR, NÃO integrada.** O dono ordenou só este handoff; a ordem de integrar é dele e está **pendente**. Plano de amanhã: [PROXIMA_FASE_2026-11-06.md](../PROXIMA_FASE_2026-11-06.md). Mecanismo e tabelas: [BUGS_painter.md](../BUGS_painter.md) (#69–#72) e [HANDOFF_CONTINUACAO_2026-11-05_noite.md](HANDOFF_CONTINUACAO_2026-11-05_noite.md).

## 1. Identidade
- Branch `line/PainterWatercolor`, HEAD `706de79944`, merge-base com `main` `c61f228b05` (= `git merge-base HEAD main`), **21 commits**, 94 arquivos, +3 709 −132 (`git diff --shortstat c61f228b05..HEAD`).
- **Smoke do dono OK** sobre o estado «lei antiga + relevo + bordas por traço + Borracha + óleos» (`35520a7fd7`). **Veio DEPOIS e ele NÃO viu:** (a) `aa34786e7f`, o preparo do 1.º toque (ele testou e deixou: «por enquanto vamos deixar o travamento como está»; no Watercolor Basic o 1.º traço continua bem mais lento, nos outros pouca diferença); (b) `706de79944`, a cura dos achados do fecho (piso do alfa da Borracha, média real do dente nos tipos que não são Paper).

## 2. O que mudou, por item
| Item | Commit(s) | Efeito |
|---|---|---|
| Bug #69 | `9b8e5ffb3f` | Edge Darkening é de cada traço: `AparenciaCelula::borda`, `Grid::borda_fundo` (f64, o do 1.º pincel que tocou a grade; nem o Clear o muda), `BORDA_DO_FUNDO` (-1) lê o fundo; placa `aux.w` nas duas leis, binding 7 na `liga_tinta` |
| Bug #70 | `26b119b58d`, `706de79944` | A Borracha no Wet apaga também a `base` antiga (`wetpaint/borracha_na_base.rs`); undo por `PassoVivo.base_antes/base_depois`; piso do alfa |
| 13 óleos | `1a2cc2cf9c`, `0937eb7810` | Smooth Edges 0,3 de fábrica **e nas 13 receitas** (`tests/it/familias/oleo_acrilica.rs`, `cap01.rs`); candidatos W2 não herdam `smoothEdges` |
| Bug #71 | `9bd76cd02d` (medido), `35520a7fd7` (ligado) | Relevo sem degrau, ligado para todos (a chave `PH2D_PAPEL_SEM_DEGRAU` saiu); `Substrate::meio_do_dente` (amp/2 nos tipos centrados, média real nos outros) |
| Bug #72 | `35520a7fd7`, `aa34786e7f` | Pen-down sem `marca_tudo` + `wetpaint/preparo.rs` (motor e papel prontos noutro fio; `PH2D_TOQUE_PREPARADO=0` desliga) + `par::plano_cheio`/`copia_plano`. **PARCIAL** |
| Antes, na linha | `5adf91e43c`, `2544bb6b88`, `998cd01a49`, `0a5bd7d2a9` | Hardness por ferramenta (Pincel e Borracha, A5); Bug #68; cena `PH2D_WETPAINT_SMOKE=4` movida para `ph2d-tool-painter` (`wetpaint_casos.rs`; pôs `ph2d-app-painter` 193 linhas acima da catraca); portões da célula de 28 B |

**Lei nova arquivada, não apagada:** [LEI_NOVA_ARQUIVADA.md](../Arquitetura/LEI_NOVA_ARQUIVADA.md); tags `lei-antiga` (`56f890198b`) e `lei-nova-arquivada` (`410076692a`), ramo `backup/lei-antiga-2026-11-05` — **todos LOCAIS** (não vão no push sem ordem). **Reverts no histórico (o efeito líquido é o que conta):** `410076692a` ligou a lei nova; `db946aa38f` voltou à antiga; `35de1cbdec` restaurou `crates/` e `assets/` à tag `lei-antiga`; `35520a7fd7` religou relevo e pen-down. Líquido: lei VELHA em todos os pincéis, sem código da `Pigmento` apagado.

## 3. Foundational e contratos
**Nenhum foundational tocado.** Prova (`git diff c61f228b05..HEAD`, linhas `+`/`-` com o símbolo, recontado depois de `706de79944`): `NodeOp` 0 · `OpResolver` 0 · `NodeManifest` 0 · `trait Tool` 0 · `VectorOp` 0 · `PROJECT_SCHEMA` 0. `RasterEditTool` 5 · `CanvasPaintTool` 7 · `PanelEvent` 11 linhas — **todas `use` ou chamadas** (`handle_panel_event(PanelEvent::SetValue/SelectOption…)`, `as &mut dyn RasterEditTool`), **nenhuma define** (`grep -E '(struct|enum|trait|impl|type) +…'` sobre o diff: 0). Subiu de 6/8 para 7/11 por testes novos de `706de79944`. Arquivos em `crates/ph2d-editor-core`, `crates/ph2d-vec*`: 0; arquivos com `schema`/`persist` no nome: 0. Gates: `architecture_contract_surface` (ph2d-nodegraph 3/0), `architecture_vector_contract_surface` (ph2d-vector-doc 11/0), `architecture_tool_contract_surface` (ph2d-editor-core 4/0).

`bash $PWD/scripts/collision-surface.sh` (colado em síntese; `base:` = merge-base): `PROJECT_SCHEMA` 216 (base 216), tripla `(216, 13, 29)` igual à base, `VEC_SCENE_SCHEMA` 29, `FLIP_SCHEMA` 13, `DOC_VERSION` 25; registro de componentes ecs 116 / render 117 / script 117, todos iguais; `node.rs` e `editor-core/src/tool.rs` intocados; **sem ADR novo** (próximo livre 0198); sem `Cargo.lock` novo; sem marcadores; nenhum arquivo acima do teto de LOC. É referência: o integrador lê o `main` no arquivo.

**Em memória, não gravado:** `AparenciaCelula` 24→28 B, `Grid::borda_fundo`, os campos de `PassoVivo` (`base_antes/base_depois`). Confirmado por: `aparencia.rs` só deriva `Clone, Copy, Debug, PartialEq` (sem `Serialize`) e 0 arquivos de schema/persist no diff. **Não confirmado por teste de ida e volta do `.ph2dproj`.**

## 4. Números medidos
- **#69** vermelho antes: Wet Liner ED 0→56,25 (4 332 px, Δ30); Wet Round ED→0 (9 954 px, Δ22). Depois: (0, 0) nos dois. Sobra: Capillary global (92 px, Δ3).
- **#71** métrica S, 46 pincéis: média +0,048, máx +0,115 → média +0,005, máx +0,024. Paper Press preservado: Knife Stain Δ44→Δ42, Knife Oily Δ136→Δ136. `706de79944` muda o resto **só nos tipos que não são Paper** (tabela de `meio_do_dente` no Bug #71); o Cold aprovado fica ao bit.
- **#70** cópia da base 7,1 ms a 4096×2560.
- **#72** (sonda de CPU, 1920×1080, maior evento do início, 1.º/2.º traço): Wet Wash 35–47/12–13 → 2,5/6,4; Watercolor Basic 19–31 → 22 (sem cura). Tabela inteira no BUGS. **App real, honesto:** 1 corrida por variante sob load 6–13; uma repetição deu Wet Wash 29; **não há ganho firme no app real**, só na sonda.
- **Memória do Dabber:** pico 305→368 MiB a raio 300, 1 610→1 802 a raio 1000; tectos 386 · 1 892.
- **Óleos pequenos engordam:** Flat Bristle 3,0→5,0; Express Oil r26 52,17→49,24.

## 5. Bateria de fecho — o que RODOU
Regra do dono 23/10: **sem `nextest-impacted` e sem suíte completa na linha; ela é do integrador e AINDA NÃO FOI RODADA.**
- Testes por nome verdes (lista montada dos nomes citados no BUGS e nos commits; o relatório da corrida está na conversa da linha): `a_borracha_no_wet_apaga_a_tinta_antiga_de_cada_meio`, `desfazer_a_borracha_no_wet_devolve_a_tinta_antiga`, `a_selecao_protege_a_tinta_antiga_da_borracha`, `apagar_a_base_chega_ao_ecra_sem_o_motor_marcar_sujo`, `preparo_tests`, `meio_do_dente_tests`, `a_tinta_nao_fica_afundada_no_papel_de_alto_relevo`, `raio_igual_ao_anel_tests`, `a_memoria_do_dabber_cabe`, alarmes de largura e de miniatura, `o_molhado_sobre_tinta_fina_nao_clareia_alem_das_tintas`; cercas `file_loc_caps`, `arch_safe_clamp_only`, `the_shell_only_shrinks`, `workspace_file_loc_cap`; GPU na placa real NVIDIA RTX 5060 Ti. fmt e clippy das 4 crates tocadas.
- **Mutações:** M1 M4 M5 M6 M10 sangraram na 1.ª rodada. M2 M3 M7 M8 M9 P1 P2 **sobreviveram** e foram curadas com testes novos (nomes em `706de79944`); todas sangram agora. M9 sangra **só** no teste direto `apagar_a_base_chega_ao_ecra_sem_o_motor_marcar_sujo`. M11 (devolver `marca_tudo`) sobrevive por natureza: só o relógio a vê.
- **Auditoria, 2 lentes** (estado/undo; correção/custo): os achados do fecho (mensagem de `706de79944`): piso do alfa da Borracha, média real do dente fora do Paper, preparo que segurava motor de outra grade, undo/redo da base; curados ali. O resto dos achados está na conversa da linha, não aqui.
- **Pins regravados de propósito:** hash do Clear glass (provado tirando o termo `meio_do_dente` de `height_at`), tecto de memória do Dabber, `larguras_aprovadas.tsv` 13 linhas.

## 6. O que um merge pode partir (risco esperado na suíte do integrador)
Testes fora dos que corri por nome que toquem os 13 óleos (Smooth Edges 0,3), o relevo ou a célula de 28 B. Dica: `render.rs:393` acende o emboss da massa sempre que o campo existe com `paper: true, relevo: false`; o produto corre sempre `relevo: true` (`otica_factos.rs:87`), então alocar o campo não muda a tela — por isso `o_molhado_sobre_tinta_fina_nao_clareia_alem_das_tintas` agora usa `paper: false`. `medida_oleo_real_fenda` fixa Smooth Edges 0 para o hash do Clear glass valer. Regravar largura/miniatura só com aprovação do dono.

## 7. ABERTO (honesto)
1. Bug #72 resto: Watercolor Basic (sem cura) e telas enormes (quadro de ~90 ms do Wet Wash a 3240²).
2. O relevo do Impasto fica depois da Borracha do trilho, com ou sem Wet (105,97 → 105,97): pré-existente.
3. **Redo** depois de um `undo.redo` que falha (`undo_vivo.rs`, `p.depois.take()?`, com `can_redo` só no motor): sem teste. É o item 0 do plano de amanhã.
4. Várias camadas com Borracha: não testado.
5. `PreparoDoToque` segura um motor pronto enquanto nenhum tique chega à ferramenta.
6. Variação residual da cena `PH2D_WETPAINT_SMOKE=4` (se deixou de variar): não verificada.
7. Código da lei `Pigmento` não apagado (alcance a confirmar com o dono).
8. Plano de amanhã e foto 5 pendentes de confirmação do dono: ver o plano.

## 8. Smokes (para o dono; passos curtos)
**Binário já compilado**; segunda corrida de `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`: zero `Compiling`, 0,2 s.
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop`. Abra o Painter, pinte com Dabber e passe um Wet Wash/Wet Liner por cima. Errado: contorno verde fino na junção (é o que o plano de amanhã estuda, não regressão).
2. Comparar o 1.º traço: `cd … && PH2D_TOQUE_PREPARADO=0 ./target/smoke/ph2d-host-desktop` (sem o preparo). Esperado: Watercolor Basic lento nos dois; os outros um pouco melhores com o preparo.
3. Cena de casos da lei: `cd … && PH2D_WETPAINT_SMOKE=4 ./target/smoke/ph2d-host-desktop`.
4. Cena do Painter: `cd … && PH2D_COMPOSITE_SMOKE=1 ./target/smoke/ph2d-host-desktop`.

## 9. Integração
`/pd-integracao` + DIRETRIZ §1.5.3; suíte completa 1× pelo integrador; depois do push, `gh workflow run spike.yml --ref main`. Troque o link do Painter no `CLAUDE.md §5.1` para este arquivo (já trocado nesta linha; se o `main` mudou a entrada, reaplique só o link).
