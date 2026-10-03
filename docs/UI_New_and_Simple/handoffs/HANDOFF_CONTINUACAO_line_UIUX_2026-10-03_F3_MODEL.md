# Continuação — `line/UIUX`, 2026-10-03 — a F3 do Model (spec/06), NESTA linha, sem integrar

> Leitor: a próxima janela desta linha. O bloco de entrada é o do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> `cd` + `pwd` + `git branch --show-current` ANTES de ler.

## §1 — Estado

- **Base (merge-base = `main`) `1ad60a1ce`**; o `main` não andou. `git cherry main HEAD` = **48 `+`**
  com este ficheiro, nenhum integrado (ordem do dono: não integrar agora). Árvore limpa.
- **F2 + F3 da Imagem + F3 do Sculpt + F3 do Flip FECHADAS, smoke do dono APROVADO nas três (03/10).**
  O último handoff de integração é
  [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_FLIP.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_FLIP.md)
  (cita A_ESCALA, O_MENU_ADD, OS_MODOS e O_SCULPT). O fecho do Model escreve um handoff NOVO sobre o
  diff ACUMULADO desde a base, que cita os cinco.

## §2 — A TAREFA: *Model ▸ Object · Edit* (D6: peça sólida / SDF → Object · Edit)

Ordem medida da F3 (spec/06 §2.1): Image ✅ → Sculpt ✅ → Flip ✅ → **Model** → Vector.

**O que já existe e se REUSA (não reconstrua):**
- `ph2d_editor_core::object_mode` — o vocabulário JÁ TEM `ObjectMode::Edit` (entrou com o Flip; o
  tipo é `(Model3D, Edit)`, a família é outra). `screens::hero::mode_drive::ModeFamily` é um trait:
  `modes`, `holds`, `enter`, `leave`, `follow(current)`, `wants(tools)` (o nascido pede o modo; e a
  ferramenta/módulo que chega à mão por uma porta antiga também pode pedir — o desenho do Flip).
- Os moldes vivos: `ph2d_app_sculpt3d::sculpt_mode` (módulo que NÃO é `Tool`, recursos emprestados
  por quadro) e `ph2d_app_flip::flip_mode` (o ALVO num tipo só, `FlipTarget`; a porta antiga
  adoptada por `wants`). A shell compõe as famílias em `render_loop/fase_object_mode.rs` — e o
  censo `shells/desktop/tests/it/every_object_mode_has_a_composed_family.rs` (tabela `FAMILIAS`)
  tem de ganhar a linha da família nova.
- `mode_drive::object_gizmo_shows` já esconde o gizmo do objecto em todo modo de criação.

**O que a F0 já mediu do Model (spec/06 §2.1):** o documento é `Smoke::doc` thread-local, cozido da
1.ª raiz `FieldObject` (`ph2d-app-field3d/src/scene.rs:296`, `sync_scene_and_birth`:
`q.iter(world).next()`); ⛔ **uma 2.ª raiz nunca é cozida**; um activo de outro tipo é ignorado. Por
isso o menu Add oferece **um** Model por cena e di-lo no rótulo, até esta F3.

**Comece por MEDIR, sem código de produto** (perguntas estreitas, um explorador por pergunta):
- (a) o que o pill **MODEL** faz hoje (`chrome/model3d_toggle.rs`; `menu_bar::MODULE_TRUTHS` tem
  `(TOPBAR_MODEL3D, ModuleTruth::Panel("model3d"))` — é PAINEL, não `Tool`), o que o módulo arma ao
  entrar (painel, o canvas «Model», a vista/sombreamento no cabeçalho da área, o modo Render) e TODAS
  as portas que o ligam (pill, *Window*, aba **Model** de cima — `CanvasOwner::Model3d` —, layout
  gravado, `PH2D_FIELD_SMOKE=<n>`);
- (b) o que é **Edit** dentro do módulo hoje: o que se edita (a árvore de campos, os nós, o gizmo da
  peça) e o que seria **Object** (mover a peça inteira no canvas 2.5D, D9) — e se o gizmo do módulo
  (`scene_gizmo.rs`) é a mesma coisa que o gizmo do objecto;
- (c) quantas raízes `FieldObject` a ponte consegue ter, e o que custa cozer «a raiz da entidade
  activa» em vez da 1.ª (o `Smoke::doc` é UM documento: troca-se a raiz cozida, ou passa a haver um
  documento por raiz?). ⚠️ É o item caro: meça antes de prometer N Models.
- Mostre o desenho ao Enio numa frase por peça, sem jargão, ANTES de codar; e pergunte, com a sua
  recomendação, se o pill **MODEL** sai (como SCULPT e FLIP) ou fica — e se a peça nova do menu Add
  nasce em **Edit** ou em Object (o Flip nasceu em Draw porque nasce vazio; a peça SDF nasce com
  forma, e é visível em Object?).

## §3 — Lições desta linha que valem para o Model

- ⛔ **«O módulo edita o 1.º objecto» nunca é UMA linha** — no Flip o briefing apontava `autokey.rs:54`
  e eram 14 leituras (painéis, pré-visualização, peek, assentar a origem) e 10 cópias da regra da
  camada. Faça o `git grep` de todas antes de desenhar, e junte-as num tipo só (o `FlipTarget`).
- ⛔ **O alvo não mora no documento se o documento entra no diff do undo** (`PartialEq`): entrar num
  modo viraria passo de histórico.
- ⛔ **Mudar o estado de FÁBRICA de um painel muda a população dos censos** do
  `ph2d-panel-registry-init` (corra-o com `--all-features`: sem elas 81 falhas falsas). Número que sobe
  porque a régua passou a ver mais é população — anote-o no sítio; uma régua que diz «sem coluna»
  não diz «fora da coluna».
- ⛔ **Apagar um ficheiro de testes pelo NOME apaga cada gate que mora nele** (Sculpt) — `grep -n '^fn '`.
- A foto apanha o que nenhum gate vê: fotografe cada passo (`PH2D_OBJECT_MODE_SMOKE=5` para o Model).
- O número da shell mede-se como a catraca o mede (todo `.rs` de `shells/desktop`, linha a linha):
  HEAD do Flip **196 914**, tecto **196 990** — **folga 76**. Código do modo vai para `mode_drive` /
  a crate da família; um gate novo em `shells/desktop/tests/it` também conta.

## §4 — Restrições (medidas)

- `screens/hero.rs` **699/700** · `interaction/state/mod.rs` **696/700** · `left_rail.rs`
  **693/700** · `hero/paint.rs` **700/700**: nada solto aí.
- A fundação é um DAG (`the_foundation_modules_form_a_dag`): `object_mode` não conhece o Hero.
- Um setter `set_*_mode` cai no censo `every_set_mode_setter_reconciles_or_is_benign`.
- O shell das ferramentas é zsh; o guarda de recursos lê o TEXTO do comando.
- Explorador com perguntas ESTREITAS (≤ 30 passos); confira o código antes de acreditar nele.

## §5 — Como a onda fecha

DIRETRIZ §1.5.9: gate batched 1× sobre o diff acumulado desde `1ad60a1ce`; mutação dos gates novos
(controlo VERDE antes, lado INDEPENDENTE — família falsa no editor-core, a real na crate dela); foto
do smoke (`fotografa_cena.sh`, um gatilho `PH2D_OBJECT_MODE_SMOKE=5`); handoff de integração NOVO que
cita os cinco anteriores; `rm -rf target/*/incremental`; e por último o build de smoke 2× com a 2.ª
saída colada. ⛔ Não integrar nem pushar.
