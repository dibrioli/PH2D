# Continuação — `line/UIUX`, 2026-10-03 — a F3 do Flip (spec/06), NESTA linha, sem integrar

> Leitor: a próxima janela desta linha. O bloco de entrada é o do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> `cd` + `pwd` + `git branch --show-current` ANTES de ler.

## §1 — Estado

- **Base (merge-base = `main`) `1ad60a1ce`**; o `main` não andou. `git cherry main HEAD` = **43 `+`**
  com este ficheiro, nenhum integrado (ordem do dono: não integrar agora). Árvore limpa.
- **F2 + F3 da Imagem + F3 do Sculpt FECHADAS, smoke do dono APROVADO nas duas (03/10).** O último
  handoff de integração é [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_SCULPT.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_SCULPT.md)
  (cita A_ESCALA, O_MENU_ADD e OS_MODOS). O fecho do Flip escreve um handoff NOVO sobre o diff
  ACUMULADO desde a base, que cita os quatro.

## §2 — A TAREFA: *Flip ▸ Object · Draw · Edit* (D6, correcção 1 do dono: o Flip tem modo PRÓPRIO)

Ordem medida da F3 (spec/06 §2.1): Image ✅ → Sculpt ✅ → **Flip** → Model → Vector.

**O que já existe e se REUSA (não reconstrua):**
- `ph2d_editor_core::object_mode` (vocabulário, `ModeState`, leis) e
  `screens::hero::mode_drive` — **`ModeFamily` é um TRAIT desde o Sculpt**: `modes`, `holds`,
  `enter`, `leave` (com a entidade), `follow(current)` (o módulo segue o modo que ficou) e `wants()`
  (um objecto que nasce num modo pede-o). A shell constrói as famílias por quadro com o que cada uma
  empresta (`render_loop/fase_object_mode.rs`). O molde vivo mais completo é
  `ph2d_app_sculpt3d::sculpt_mode` (leis puras `holds`/`follow` + a família + `smoke_step`).
- Um modo novo = a variante em `ObjectMode` (+ `label_key`, `row_id`, id em `ids/chrome/rail.rs`,
  chave em `ph2d-i18n/src/object_mode.rs`) + uma família na crate dela + UMA entrada no array de
  `fase_object_mode.rs`. `Draw` e `Edit` não existem ainda no vocabulário.
- `mode_drive::object_gizmo_shows` já esconde o gizmo do objecto em todo modo de criação.

**O que a F0 já mediu do Flip (spec/06 §2.1):** `FlipDoc` global; o traço cai SEMPRE no 1.º objecto
(`autokey.rs:54`); o `FlipEntityMap` já liga entidade↔objecto; um activo de outro tipo é descartado
em silêncio. Entrar em Draw tem de abrir SOBRE o desenho desta entidade.

**Comece por MEDIR, sem código de produto** (perguntas estreitas, uma por módulo):
- (a) o que o pill **FLIP** faz hoje (`chrome/flip_toggle.rs` → `ActivateTool { tool_id: "flip" }`;
  `menu_bar::MODULE_TRUTHS` tem `(TOPBAR_FLIP, Tool("flip"))`) e o que a ferramenta `flip` arma ao
  entrar (painéis, faixa de frames, o canvas);
- (b) o que é **Draw** e o que é **Edit** DENTRO da ferramenta hoje (os press do Flip:
  `input_dispatch/despacho_clique_flip.rs` — Draw, Pairs, Fill, Colorize, Trace, Erase, Reshape,
  Edit): quais são MODOS e quais são FERRAMENTAS do modo (a mesma confusão que a D3 mediu no
  `DrawMode` do vetor);
- (c) como o «objecto activo do Flip» se escolhe hoje e como passa a vir da entidade (o `autokey.rs:54`).
- Mostre o desenho ao Enio numa frase por peça, sem jargão, ANTES de codar; e pergunte, com a sua
  recomendação, se o pill FLIP sai (como o SCULPT) ou fica como atalho — e se o desenho Flip novo
  do menu Add nasce em Draw (como a peça nasce em Sculpt) ou em Object.

## §3 — Lições desta linha que valem para o Flip

- ⛔ **Apagar um ficheiro de testes pelo NOME apaga cada gate que mora nele** — o do pill SCULPT
  tinha 9 gates e só 4 eram dele (o gate batched apanhou-o). `grep -n '^fn '` antes de apagar.
- A foto apanhou o que nenhum gate via (o gizmo por cima do barro): fotografe cada passo.
- Um painel de outra crate só existe no registo da SHELL (features): um clique real numa aba desses
  painéis mora em `shells/desktop/tests/it/`, não no `ph2d-panel-registry-init`.

## §4 — Restrições (medidas)

- `screens/hero.rs` **699/700** · `interaction/state/mod.rs` **696/700** · `left_rail.rs`
  **693/700** · `hero/paint.rs` **700/700**: nada solto aí.
- A shell está SOB o tecto de `the_shell_only_shrinks` (acumulado da linha: líquido **−41**);
  código do modo vai para `mode_drive` / a crate da família.
- A fundação é um DAG (`the_foundation_modules_form_a_dag`): `object_mode` não conhece o Hero.
- Um setter `set_*_mode` cai no censo `every_set_mode_setter_reconciles_or_is_benign`.
- Explorador com perguntas ESTREITAS (≤ 30 passos); confira o código antes de acreditar nele.
- O shell das ferramentas é zsh; o guarda de recursos lê o TEXTO do comando.

## §5 — Como a onda fecha

DIRETRIZ §1.5.9: gate batched 1× sobre o diff acumulado desde `1ad60a1ce`; mutação dos gates novos
(controlo VERDE antes, lado INDEPENDENTE — família falsa no editor-core, a real na crate dela); foto
do smoke (`fotografa_cena.sh`, um gatilho `PH2D_OBJECT_MODE_SMOKE=4` para o Flip); handoff de
integração NOVO que cita os quatro anteriores; `rm -rf target/*/incremental`; e por último o build de
smoke 2× com a 2.ª saída colada. ⛔ Não integrar nem pushar.
