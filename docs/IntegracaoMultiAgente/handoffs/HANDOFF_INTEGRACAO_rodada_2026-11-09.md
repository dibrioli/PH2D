# HANDOFF — rodada de integração de 09/11 (6 linhas)

Leitor: a próxima LLM e o próximo integrador. Ramo `integ/2026-11-09` (worktree `Worktrees/integ-2026-11-09`). Lista exata das curas: `git log --oneline main..integ/2026-11-09 --grep='integ 09/11'` (~55 commits `fix(integ 09/11): …`).

## Ordem, medida (collision-surface + `git merge-tree` par a par)

GameEngineFuncional02 (180 commits, ff) → GameEngineFuncional (78) → PainterWatercolor (42) → PixelArt (97) → ScriptEditor (29) → SfxLab (1). GEF02 primeiro porque, dos commits que tocam os 47 ficheiros comuns, a GEF tem 39 e a GEF02 64.

## Contadores (DELTA)

- `PROJECT_SCHEMA` 217 → 225, tripla `(225,13,29)`: GEF02 218–221, GEF 222–224, Painter 225. Reconta-se por `python3 scripts/schema-recount.py`.
- ADR-0199 (GEF02). Registo `ph2d-ecs` 119. `MODULE_TRUTHS` 26 (contado).
- `ph2d-app-painter` (catraca de LOC) 24451 → 24419 (tecto 24429), por corte de comentários nos smokes impasto/substrate/mask.
- Catraca de altura do Basics 1670 → 1694 (só a linha de título da Color mixing dobrada).
- `App` 170 → 169 campos; `fase_selection_highlight` 208 → 194 LOC; matriz de camadas 528 células; `panel → screens` 10 → 7.

## Conflitos de merge

- GEF sobre GEF02: a limpeza §0.11 da GEF (sem leitura de versões antigas) × testes de versão antiga da GEF02 — apagados (lei §0.11). Idem Painter (teste do v217).
- Paleta de comandos: pintura reescrita da GEF + coluna de atalho do ScriptEditor portada.
- TextInput: o campo `reserva_direita` do Pixel aplicado no `model.rs` partido pelo Painter.
- `CLAUDE.md` §5: entradas fundidas e encurtadas para caber no tecto de 14000 B.

## Suítes e curas

1. **#1 pré-voo:** fmt, 2 deps mortas (código movido para fora das `ph2d-app-*`), typos pt.
2. **#2:** 68 FAIL + 6 TIMEOUT, curados em 5 grupos paralelos (A tool-painter, B painéis Painter, C componentes/física, D registry-init/shell, E1 gates do editor-core) + D2/E2.
3. **#3/#4 pré-voo:** fmt, clippy `single_element_loop`, doc órfão (`dock_reopen.rs`).
4. **#4:** 188 FAIL, todos de UMA causa: a cura A pôs o campo `dente` do `BrushSpec` em `CAMPOS_FORA` como `DA_OBRA` ⇒ `campos_da_macro()` passou a levá-lo e `json/meio.rs` caía no `unreachable`. Cura (`3be3ff7c26`): `dente` é `POR_DAB` (enchido em cada dab pelo papel da tela, `Dente::do_papel`). **Classificar um campo no censo muda o que a MACRO grava**: a categoria decide; corra um teste de macro depois ([memória](../../../project-memory/feedback_a_field_census_category_decides_what_the_macro_records.md)).
   Também: clamp do % do Pixel prefs pela `safe_clamp`; flake `sprite_properties` a passo fixo (como o irmão `layer_user_data`); matriz do undo vivo (6 testes de 28–80 s sozinhos) estourou 180 s e depois 3 deles 480 s na suíte ⇒ tecto 8×60 s + test-group nextest `undo-matriz` (`max-threads 2`, `threads-required 4`, `priority 90`).
5. **Suíte final (#5, tip `3be3ff7c26` + `6a39ac7011`/`faf97fae98`): 34 117 de 34 117 verdes, 0 FAIL, 0 TIMEOUT, 1 847 s** (63 lentos; `.config/testes-lentos.tsv` regenerado e comitado com este handoff).

## Portões mexidos, a rever

- `a_unidade_aparece_uma_vez_so` aceita agora «<número> <unidade>» como VALOR (estreito: 2 palavras, a 1.ª número) — menu Recovery do Pixel («10 seconds»).
- `CollisionOverride` (GEF02 B9): o escritor `physics_colide.rs` entra no censo da UI; a porta no mundo das macros é `Gravado` (camadas por índice). Isenção do losango da matriz viajou: `physics_colide.rs::fila_da_matriz` → `physics_colide_fila.rs::fila`.
- `SpriteGrid.sheet` (Runtime) e `SpriteAnimator.start` (OwnSection) ganham motivo no catálogo.
- Gate do índice de acerto mede os 5 botões do jogo (título Run virou botões, GEF); `ALL_ICONS` do teste ganha `PlayEditor/PlayFullscreen/PlayWindow`.
- Widgets da paleta/TextInput: anel de foco `FRAME-RAW-OK`, passe de medida `LITERAL-COLOR-OK`, fracção `LITERAL-PX-OK`; `look.rs` e `text_input/mod.rs` no `A11Y_OPT_OUT`.
- DAG da fundação: alça (`DOCK_SEAM_PX`, `paint_seam_mark`, `seam_mark`, `ARROW_W`) desce para `crate::seam_mark`; dados do User Data descem ao `pixel_panel`. `input_dispatch/rato.rs` → `despacho_rato.rs` (gates de texto seguem).
- Posse do jogo: no palco um clique fora não tira o teclado (lei da pausa da GEF, dono 07/11).
- Tecto 700 LOC cortado em `hero/paint.rs` e `ph2d-i18n/lib.rs`.

## Para o dono ver no smoke (as curas mudaram o produto)

1. **Painter:** o grupo «Color mixing» do Basics nasce DOBRADO (título que abre), nos dois painéis.
2. **Add Object:** a busca deixou de ser difusa (só nome e palavras de intenção) para o «ensino» voltar quando não há resultado; Ctrl+K continua difuso.
3. **Editor de scripts, cena 4:** o erro do Play abre a lista de problemas sozinho e PARA o jogo (Pause on Error ligado por omissão).
4. **Física, aba Collision Layers:** vão entre controlos 6 → 3 px; recuo da frase aos botões +3 px.
5. **Depurador:** a quina do chip «waiting» vem do tema (no moderno deixa de ser pílula).
6. **Clear glass do óleo:** resultado regravado porque a lei Relief × Tooth (dono 08/11, `60c3bfcb6f`, bisseção exata) mudou a tinta.
7. **Pixel:** Numpad Minus/Plus legíveis na janela de atalhos; cabeçalho do User Data em coluna estreita põe o botão na linha de baixo; ícones próprios das janelas History/Keys/Prefs/Slices (Undo, Kbd, Settings, Knife).

## Aberto

- `.config/testes-lentos.tsv` regenera-se no próximo `python3 scripts/tempos-teste.py` (os 6 `undo_matriz` entram).
- O Add Object «sem difuso» é decisão técnica da cura; se o dono quiser difuso também lá, o ensino precisa de outra porta.
- Nada aqui foi aprovado pelo dono: o smoke é dele; integrar não é aprovar.

## Lições

- **Categoria de censo ≠ etiqueta:** `CAMPOS_FORA` decide o que a macro grava; 188 testes de macro caíam por um campo mal classificado.
- **Cura que um grupo faz pode partir outro grupo:** a cura A só apareceu na suíte #4; testes por nome do grupo não pegam a macro.
