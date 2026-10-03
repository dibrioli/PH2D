# Continuação — `line/UIUX`, 2026-10-03 (a linha segue; NÃO integrar agora)

> Leitor: a próxima janela desta linha. O bloco de entrada é o do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> `cd` + `pwd` + `git branch --show-current` ANTES de ler.
>
> **Ordem do dono (03/10):** *«vamos manter nessa linha mas vamos levar para outra janela. Não
> vamos integrar agora.»*
> ⇒ A onda nova (tipos de objecto e modos por objecto) é feita **nesta linha**, por cima da onda da
> escala. Esta já está fechada, mas **não foi integrada**.

## §1 — Estado da linha

- **Base (merge-base = `main`) `1ad60a1ce`.** O `main` não andou desde 02/10 e não houve rebase.
- **HEAD = o commit deste ficheiro.** `git cherry main HEAD` = **21 `+`**, todos desta linha e
  nenhum integrado. A árvore está limpa.
- **A onda da ESCALA está FECHADA e com o smoke do dono APROVADO (03/10).** O handoff de
  integração dela é
  [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md):
  §3 = superfície de colisão, §4 = gates e mutação, §9 = o que veio depois do fecho.
  ⚠️ Ele continua válido para os 18 commits que descreve. O fecho da onda NOVA escreve um handoff
  de integração NOVO, sobre o diff ACUMULADO desde a base, que cita esse.
- Depois do fecho vieram **3 commits só de docs**:
  - `78989c58f` — o plano e o oráculo;
  - `7ed3af514` — o modal;
  - este ficheiro.

## §2 — A TAREFA da próxima janela: o plano `spec/06`

Leia INTEIRO: [`../spec/06_tipos_e_modos_de_objeto.md`](../spec/06_tipos_e_modos_de_objeto.md).
É o plano, já com as escolhas do dono. Os oráculos estão em
[`../spec/oraculos/modos_e_tipos_2026-10-03.md`](../spec/oraculos/modos_e_tipos_2026-10-03.md),
com as sondas verbatim.

**O pedido do dono, nas palavras dele:**
- o `+` da Hierarchy cria TODOS os tipos de objecto da engine (hoje cria só o vazio);
- o Ctrl+N da imagem vai para lá;
- cada objecto tem os seus **modos de edição**, como o Blender. Exemplo dele: a malha de sculpt
  tem *Transformação*, *Sculpt* e *Painter*.

**As escolhas do dono (fechadas, não re-litigar):**

| | decisão |
|---|---|
| 1 | O `+` abre um **MODAL como o *Add shape…* do Modeling** — o `command_palette` genérico; o modelo copia `crates/ph2d-app-field3d/src/shape_palette.rs`. |
| 2 | **Pintura = MODO da imagem** (Image ▸ Paint), não um tipo próprio. |
| 3 | O mesmo modal no **Shift+A e no botão direito** do canvas, em modo Object. Dentro de um modo, o `A` é o Add do próprio modo. |
| 4 | **Cadeado ligado:** num modo de criação, clicar noutro objecto NÃO troca de objecto (o *Lock Object Modes* do Blender, de fábrica). |
| 5 | **Grupo «Jogo»** no modal: Câmara, Corpo de física, Som, HUD (um vazio com o componente já posto; só modo Object). |
| 6 | Sem pergunta, ficam na recomendação: o Ctrl+N fica como atalho; abrir uma aba de cima sem objecto compatível fica em Object e nunca cria um. |

E o que já estava decidido antes (30/08, [`../00_DECISOES_DO_ENIO.md`](../00_DECISOES_DO_ENIO.md)):
- **D3**: três eixos — layout (as abas de cima) · modo (do TIPO do objecto) · ferramenta;
- **D6**: a tabela de modos por tipo; o Flip tem `Draw` próprio e o vetor NÃO tem `Paint`.

**A ordem: comece pela F0, que é medir e não codar.** Para cada módulo — Flip, Vector, Painter,
Model3D (`ph2d-app-field3d`) e Sculpt3D — responda com `file:line`:
- *quem guarda hoje «o que estou a editar»;*
- *se ele consegue passar a ser «o documento DESTA entidade»;*
- *o que acontece hoje com um activo de outro tipo.*

O resultado decide a ordem da F3 (a proposta é Sculpt primeiro). ⚠️ Antes da F1, mostre ao dono
o resultado da F0 numa frase por módulo, sem jargão.

## §3 — O que já se sabe (e quanto se confia)

**Confirmado no código:**
- `ObjectKind` vive em `crates/ph2d-component-desc/src/lib.rs:336`:
  `Empty · Image · Vector · Flip · Painted · Model3D · Sculpt3D`.
  - É **DERIVADO** do componente-marcador (`Sprite`, `VecPathRef`, `FlipObjectRef`, `PaintedDoc`,
    `FieldObject`, `BakedForm`) e tem `ALL`, `label_key` e `marker`.
  - ⛔ A lei dele: nunca um campo escrito à mão.
- O `+` da Hierarchy: `shells/desktop/src/render_loop/hierarchy.rs:234` despacha para
  `hierarchy_add_root.rs::spawn_empty_root`.
- O Ctrl+N: `shells/desktop/src/render_loop/fase_new_image_modal.rs` (`spawn_blank_canvas`).
- O modal: `ph2d_editor_core::widget::command_palette` (`PaletteModel`/`PaletteGroup`/
  `PaletteSub`/`PaletteItem`).
  - Precedentes: `shape_palette.rs` (`build_from`; `item_id` = hash da chave i18n;
    `slot_of_pick`; a razão do bloqueado viaja no RÓTULO).
  - E o `component_palette` da shell (o `+` do Inspector): `shells/desktop/src/app_state_gfx.rs`,
    `init.rs` e `component_attach_seam_tests.rs`.
- As abas de cima são **layouts**: `crates/ph2d-editor-core/src/screens/task_layout.rs`
  (`TaskLayout`, `CanvasOwner`; labels `chrome.layout.*`).
- O sítio do selector «Object Mode ▾» são os pulldowns de área:
  - `AreaMenu`, com `faces` desde 02/10;
  - publicados por `WidgetStore::set_area_commands`;
  - postos na fila por `screens/hero/tool_bar.rs::bar_rail`;
  - o 3D Model já publica ali a vista e o sombreamento.

**NÃO confirmado** (dois exploradores voltaram fracos, a 40 passos; reconfira lendo o código):
- o Flip edita um `FlipDoc` global com objectos dentro (`FlipObjectRef(ObjectId)`; estado em
  `ph2d-app-flip/src/state.rs`, documento em `gfx.flip`);
- o Sculpt tem `Sculpt3dShellState` (`ph2d-app-sculpt3d/src/shell_state.rs:15`), com peças como
  entidades (`SculptRowsSeen`), e `BakedForm(u32)` é o resultado assado;
- o Model3D tem `FieldObject` por entidade (`ph2d-field-ecs`);
- o vetor tem `VecState`/`Pen::selected_paths()` (`shells/desktop/src/app_state.rs:360`) e o
  `DrawMode` com 14 variantes (2 modos + 12 ferramentas, D3);
- o Painter tem `PaintedDoc(DocId)` por entidade (`shells/desktop/src/project_painter.rs:68`), e
  ele não exige `Sprite`.

⇒ É exactamente isto que a F0 tem de medir. Para o explorador, dê **perguntas estreitas** (uma
por módulo, ≤ 30 passos). O pedido largo de 7 pontos estourou os 40 passos duas vezes.

## §4 — Restrições herdadas (medidas)

- **Tectos de LOC:**
  - `screens/hero.rs` **698/700**, `screens/hero/paint.rs` **700/700**, `hero/left_rail.rs`
    692. Um campo novo no hero exige corte no mesmo ficheiro (nunca subir o tecto).
  - ⇒ O estado do modo (`ModoActivo`) vive num ficheiro IRMÃO ou num sub-estado, não solto no
    `HeroScreen`.
- **A shell:** `+161 −91` acumulado, e a catraca `the_shell_only_shrinks` está verde. O código de
  família vai para `crates/ph2d-app-<família>` e a shell é composição.
- **Escala:** o chrome é LÓGICO. A shell pergunta pelas portas físicas (`chrome_hit`,
  `handle_pointer_fisico`… — o gate `a_shell_pergunta_ao_chrome_pelas_portas_fisicas`), e o texto
  arredonda por `ui_scale::ao_pixel`. Ver o §2 da
  [continuação de 02/10](HANDOFF_CONTINUACAO_line_UIUX_2026-10-02.md).
- **Fotos:** `docs/Components/ferramentas/fotografa_cena.sh` com HOME isolado. Para testar outra
  escala, `HOME=<dir com .ph2d/prefs.txt ui_scale=125>`; para HiDPI, `WINIT_X11_SCALE_FACTOR=2`.
  O XTest não chega: o clique prova-se num seam `ph2d-ui-testkit`.

## §5 — Como a onda fecha

É a DIRETRIZ §1.5.9:
1. o gate batched 1× sobre o diff acumulado desde `1ad60a1ce`;
2. a mutação dos gates novos (agente `mutacao`, controlo VERDE antes de cada uma);
3. o handoff de integração NOVO, que cita o da escala;
4. `rm -rf target/*/incremental`;
5. por último, o build de smoke 2× com a 2.ª saída no handoff.

⛔ **Não integrar nem pushar.** O dono decide quando.
