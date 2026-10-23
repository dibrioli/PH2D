# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, d5 (cena 8) e 8b parte 1: assets e scripts no código (23/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último handoff de
> INTEGRAÇÃO continua o [Onda 2 até ao 8d](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-22_ONDA2_ATE_8D.md); este vem por cima.
> Antecessor: [item 8a](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-21_ITEM8A_ARRASTAR.md) · plano:
> [07_arrastar_para_o_codigo.md](../07_arrastar_para_o_codigo.md) (§3.10 = linhas do Inspector) ·
> [08_scripts_tocam_propriedades.md](../08_scripts_tocam_propriedades.md) · fila: [03_onda2.md](../03_onda2.md) item 8.

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor`. Merge-base
  com o `main` = `972f411a6f` (linha rebaseada, nada a rebasear).
- Commits desde o main: `28f33f41ee` (smoke do dono da cena 7: OK) · `055cba0473` (d5) · `a2761e96c0` (8b parte 1).

## 2. d5 FEITO — cena `PH2D_CODE_SMOKE=8`

- `ph2d-app-components/src/code_smoke_props.rs` (+ `_tests.rs`). O Hero (sprite, corpo Kinematic, Health 100) corre
  `aventura.luau` e anda sozinho para a direita. Spikes: `ph2d.damage 40`. Coin: `health.max` +50, `ph2d.heal 50`,
  `visibility.visible=false`. Door: `sprite.image "door_open"`. Camera: `game_camera.height` 16→5 (a propriedade é
  `game_camera.height`, NÃO `camera.height` como o plano 08 supunha). As barras «Life Max»/«Life» são sprites
  esticados pelo que o script lê.
- PNGs das portas em `docs/CodeEditor/ferramentas/smoke/cena8/` (gerados por `gera_portas.py`); `code_smoke_props::sobe_as_portas`
  sobe-os ao atlas, chamado de `shells/desktop/src/render_loop/fase_sprite_inspector_smokes.rs`. `components_scenes.rs`
  liga `simulate_physics` na cena 8 (a vida só corre em corpos). Roteadores `CENAS=8` em `code_smoke.rs` e
  `ph2d-app-code/src/smoke.rs`.
- Prova: `code_smoke_props::tests::a_porta_abre_pelo_nome_o_maximo_sobe_a_camera_aproxima_e_rebobinar_devolve`.
- Corrida conduzida + 6 fotos: `docs/CodeEditor/ferramentas/conduz_propriedades/` (`corre.sh`). A 1.ª corrida ensinou
  ao contrário (porta fora do ecrã no início, herói escondido atrás dela): curado (câmera a 16 m, porta criada primeiro).
- **Smoke do dono da cena 8: OK (23/10).**

## 3. 8b parte 1 FEITA (`a2761e96c0`) — mestres, imagens e scripts largados no código escrevem o NOME

- `ph2d_code_syntax::dropping::{name_insertion, constant_name}`: sem modificador, `"nome"` no cursor; Ctrl,
  `local NOME = "nome"` no início da linha (também no topo).
- `Dropping::Names` em `ph2d-editor-core/src/doc_surface.rs`; `ph2d-app-code/drop_doc.rs` despacha; i18n
  `code.drop.names`, `code.drop.ctrl_const`.
- `asset_drop::resolve` (Prefab|Image, `Code{on_text:true}`) = `WriteCode` (antes `Refuse`).
- `object_drag::{Libraries, library_name}` (nome do mestre; nome da imagem na biblioteca); a shell
  `asset_drag_wire.rs` passa `Libraries{sim, db, atlas}`.
- Arrastar da lista de scripts: `ph2d-app-code/src/{list.rs,surface.rs}` (`ListDrag`); clicar/abrir/renomear
  passaram a acontecer ao SOLTAR, sem arrasto.
- Provas verdes por nome: `dropping` (8) · `arrastar` (9, incl. `um_nome_cai_entre_aspas_e_com_ctrl_vira_uma_constante`,
  `arrastar_um_script_da_lista_para_o_texto_escreve_o_nome`) · `lista` (25) · `object_drag`+`asset_drop` (17, incl.
  `um_mestre_e_uma_imagem_chegam_ao_script_pelo_nome`) · shell `the_drag_promises` (3).

## 4. Ferramentas (curas e defeito)

- CURADO: `docs/Components/ferramentas/fotografa_cena.sh` comparava o `WAYLAND_DISPLAY` interior com `wayland-0`
  fixo; a sessão do dono hoje é `wayland-1`, o kwin virtual nascia `wayland-0` e toda foto era recusada. Agora compara
  com os valores de FORA (expandidos no heredoc; o script recusa crases nesse heredoc).
- Sob o HOME isolado o assistente não é achado: passe `PH2D_LUAU_LSP=$HOME/.cache/ph2d/luau-lsp/1.70.1/luau-lsp` no
  ambiente da foto (os passos da cena 5 precisam).
- **NÃO curado:** `scripts/cargo-test-narrow.sh` imprime «0 falharam · 0 passaram» com exit 1 quando o alvo de teste
  não compila (o contrato diz exit 2).
- Fotos da cena 5 **refeitas** (`9e7bda1861`; `PH2D_CODE_SMOKE=5 PH2D_CODE_SMOKE_PASSO=0..10`, 1330×772, espera 45 s,
  `PH2D_LUAU_LSP` apontado): cada log diz «passo n feito». O condutor passou a ROLAR até o alvo fora da vista (`docs.scroll_to`) — o 1.º passo parava e as 11 saíam iguais.

## 5. O que um MERGE pode partir

- `CENAS` 7 → 8 nos dois sítios; CLAUDE.md §5.1 passa a `PH2D_CODE_SMOKE=1..8` na integração (a linha não o editou).
- `DocumentSurface` ganhou `Dropping::Names` (foundational, append-only); `asset_drop::resolve` mudou de resposta.
- **Catraca `the_shell_only_shrinks`: TECTO 189 041; a shell contava 188 820 antes das ~15 linhas de hoje.** Toda
  linha nova na shell obriga a MOVER código.
- Recontagem no fecho: `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor/scripts/collision-surface.sh`.

## 6. ABERTO (por ordem)

1. **8b parte 2 — linhas do Inspector → código** (07 §3.10): nome da propriedade entre aspas (`"health.max"`, pose
   `"x"`); propriedade do próprio script do objecto → `self.<nome>`. Hoje nenhuma linha do Inspector arrasta. Linhas
   pintadas em `ph2d-panel-inspector/src/sections/rows.rs`; o cartão pinta a chave de catálogo
   `component.field.<bloco>.<id>` (d3) — candidato a gancho para mapear linha → `FieldDesc.name`. `DragPayload` tem de
   continuar primitivo/Copy: variante nova (ex. Property com endereço estável u64/u32).
2. **8c — Alt** → `local porta = ph2d.property("porta", ph2d.object("Porta"))`. `ScriptValue` não tem Object
   (`props.rs:48-69`; acrescentar no FIM, o fio é posicional). Recomendação: guardar o NOME (como
   `View::Named(Target::Name)`); o cartão Script (`ph2d-panel-inspector/src/sections/script.rs`, ponte
   `ph2d-app-components/src/script_inspector.rs`) precisa de uma ranhura que aceite largada da Hierarquia.
3. Smoke conduzido + fotografado do 8b/8c (cena 7 estendida ou cena 9 nova — contar o número).
4. Prova de mutação das leis do 8d (d1–d4b: nome, vista, linha no neutro, nome do alvo, imagem sobreposta) e do 8b/8c.
5. `/pd-linha-fechar` (gate batched inteiro, auditoria, handoff de integração, §5.1 com `1..8`).

Perguntas do dono ainda abertas (plano 08 §7): script que renomeia um objecto em execução · cartão «Render Source» ×
componente «Sprite» / mostrar que imagem da biblioteca · legendas nos cartões compostos.
