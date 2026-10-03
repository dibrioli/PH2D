# Continuação — `line/UIUX`, 2026-10-03 — a F3 do Sculpt (spec/06), NESTA linha, sem integrar

> Leitor: a próxima janela desta linha. O bloco de entrada é o do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> `cd` + `pwd` + `git branch --show-current` ANTES de ler.

## §1 — Estado

- **Base (merge-base = `main`) `1ad60a1ce`**; o `main` não andou. `git cherry main HEAD` = **35 `+`**
  com este ficheiro, nenhum integrado (ordem do dono: não integrar agora). Árvore limpa.
- **A F2 + a F3 da Imagem estão FECHADAS e com o smoke do dono APROVADO (03/10).** O handoff de
  integração é [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_OS_MODOS.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_OS_MODOS.md)
  (cita o da escala e o do menu Add). O fecho da F3 do Sculpt escreve um handoff de integração
  NOVO, sobre o diff ACUMULADO desde a base, que cita os três.

## §2 — A TAREFA: *Sculpt ▸ Object · Sculpt · Paint* (o exemplo do dono, D6)

Ordem medida da F3 (spec/06 §2.1): Image ✅ → **Sculpt** → Flip → Model → Vector. Leia o spec/06
§2.1, §3.2, §3.4 e o §4 F2 (o que já existe e os desvios), e o §1/§3 do handoff OS_MODOS.

**O que já existe e se REUSA (não reconstrua):**
- `ph2d_editor_core::object_mode` (vocabulário, `ModeState`, leis) e
  `screens::hero::mode_drive` (`ModeFamily`, `drive`, `refused`, `right_click_on_canvas`). Um modo
  novo é: a variante em `ObjectMode` (+ `label_key`, `row_id`, id em `ids/chrome/rail.rs`, chave
  i18n em `ph2d-i18n/src/object_mode.rs`) + uma `ModeFamily` na crate da família + uma entrada em
  `shells/desktop/src/render_loop/fase_object_mode.rs::MODE_FAMILIES`.
- ⚠️ `Paint` passa a ser declarado por DOIS tipos (Image e Sculpt3D): o `mode_drive` já procura a
  família por `(tipo, modo)`, e a família do Painter só declara `(Image, Paint)`.

**O que a F3 do Sculpt entrega:**
- `ObjectMode::Sculpt` e a `ModeFamily` da escultura (`ph2d-app-sculpt3d`): `(Sculpt3D, Sculpt)` e
  `(Sculpt3D, Paint)`. Entrar = a escultura SOBRE a peça desta entidade (`Sculpt3dScene::active`
  passa a vir da entidade — `Sculpt3dPieceRef`, `entities.rs`); sair = Object.
- O Paint da peça é o Painter na tela da vista (`painter_na_malha`), hoje alcançado por IMG + a aba
  do Painter ao lado do painel da escultura (`ph2d_app_sculpt3d::abas`,
  `screens/hero/slot_tabs_ferramenta.rs`). ⛔ O caminho antigo sai na MESMA fase (spec/06 §5).
- O pill SCULPT (`chrome/sculpt3d_toggle.rs` → `ph2d_app_sculpt3d::mode::apply_toggle`) é o outro
  caminho para o mesmo módulo: ⚠️ MEÇA antes o que ele faz além de armar a cena (a forma tem 3
  posições, barro/luz/desligada, e o `D` percorre-as — `mode.rs` cabeçalho) e decida com o dono se
  ele sai ou vira atalho do modo.
- Gate por módulo (spec/06 §4 F3): duas peças, entrar em modo numa, a outra intocada.

## §3 — Medido nesta janela (e o que NÃO está medido)

- O Painter na peça 3D distingue-se do da imagem por `PainterTool::on_screen_canvas()`
  (`paint_mode::holds_an_image`, gate `the_painter_on_the_sculpt_screen_is_not_the_image_mode`).
- `mode::new_scene` (`pub(crate)`) é a porta única que cria a cena (F1).
- NÃO medido: como o `Sculpt3dScene::active` (índice) se liga à entidade seleccionada hoje; o que
  o pill faz à ferramenta em mãos; se o modo Sculpt precisa de tomar o canvas (o barro decide o
  ponteiro — `FormRole::draws_clay`).

## §4 — Restrições (medidas)

- `screens/hero.rs` **699/700** · `interaction/state/mod.rs` **696/700** · `left_rail.rs`
  **693/700** · `hero/paint.rs` **700/700**: nada solto aí.
- A shell está SOB o tecto da catraca `the_shell_only_shrinks` com POUCA folga (a F2 estourou-a em
  +386 com o quadro na shell): código do modo vai para `mode_drive` / a crate da família; a shell só
  ganha uma linha em `MODE_FAMILIES`.
- A fundação é um DAG (`the_foundation_modules_form_a_dag`): `object_mode` não pode conhecer o Hero.
- Um setter chamado `set_*_mode` cai no censo `every_set_mode_setter_reconciles_or_is_benign`.
- Explorador com perguntas ESTREITAS (≤ 30 passos, uma por módulo).
- ⚠️ O guarda de recursos lê o TEXTO do comando (heredoc com `cargo run` é recusado); o shell é zsh
  (listas por `bash -c` com arrays).

## §5 — Como a onda fecha

DIRETRIZ §1.5.9: gate batched 1× sobre o diff acumulado desde `1ad60a1ce`; mutação dos gates novos
(controlo VERDE antes, lado INDEPENDENTE — família falsa no editor-core, a real na crate dela);
foto do smoke (`fotografa_cena.sh`, um gatilho `PH2D_OBJECT_MODE_SMOKE=3` para a escultura);
handoff de integração NOVO que cita os três anteriores; `rm -rf target/*/incremental`; e por último
o build de smoke 2× com a 2.ª saída colada. ⛔ Não integrar nem pushar.
