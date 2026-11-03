# Handoff — linha `NovaUI`: fundação dos cartões + Widget Gallery em cartões (03/11/2026)

Worktree `Worktrees/line-NovaUI`, branch `line/NovaUI`, HEAD `6e402012fc`, merge-base com `main` = `3a3896a7ab`
(27 commits à frente). **Não integrada, sem push.** Leitor: a próxima LLM / o integrador.
Plano-mãe: [06](../06_COMO_PORTAR_PARA_O_MAIN.md) §3 · desenho da galeria: [08](../08_galeria_em_cartoes.md).
O diff contra o merge-base: 304 ficheiros, +21 985 −1 713 (inclui docs e fotos).

## 1 — O que a linha entrega

1. **Fundação (famílias 2 e 4 de 06 §3)**, portada de `line/TrelloUITeste` `45908badd2` (`049989b94a`):
   `ph2d-card-stack`, `ph2d-card-rope`, tokens `Card*`/`Kanban*` + `derive::card_tone`, `ph2d-i18n/src/cartoes.rs`
   (111 chaves), `verlet-rope::corda`, e o núcleo `ph2d-editor-core` (`widget/card_ink`, `card_field`, `panel/cards*`,
   `interaction/state/card_*_ops`, despacho, `property_row/{cartao,escolha,marca}`). `panel/cards.rs` e mais quatro
   acima do tecto foram partidos por responsabilidade (`5ee70929c1`, movimento puro).
2. **Widget Gallery em cartões**, no encaixe da direita como um Inspector (doc 08 §1): **17 cartões de ferramentas
   à Figma** (Align, Boolean, Transform, Layout, Corners, Constraints, Fill, Stroke, Effects, Text, Export…),
   **14 casos reais** (Image, Physics Body, Collider, Health, Platform Player, Camera, Animation, Brush, Bones,
   Constraint, Light 2D, Audio, Pixel Art, Script), **5 amostras do Rive** (Interpolation, State machine inputs,
   Trim path, Solo, Gradient; URLs em 08 §1.4) e **11 cartões de sempre** com título de ferramenta.
   - `PH2D_CARDS=0` = a janela flutuante antiga.
   - `PH2D_GALLERY_SMOKE=1` abre a galeria já visível; `PH2D_GALLERY_SMOKE_SCROLL=<px>` segura a rolagem;
     `PH2D_GALLERY_SMOKE_SIDE=1` abre a linha do efeito. Fotos `maquete/g1..g5`.
3. **A fundação continua um DAG** — curas de `7439627882` (o porte fechava três ciclos de módulos de topo e subia
   a catraca widget→interaction 43→44):
   - interaction→panel: a lista publica as linhas (`WidgetStore::set_color_style_rows`), o despacho só lê o store;
     o teste do ícone do menu foi para `panel/cards_menu.rs`.
   - widget→panel e property_row→panel: a tinta do corpo desce para `widget::card_ink`; o chip do dropdown para
     `widget/dropdown/card_chip.rs`; as três linhas do corpo (fields/choices/check) viram TABELA injectada
     (`property_row::cartao::CardRows`, publicada por `panel::cards_corpo::LINHAS`).
   - `CardInk::of_card` recebe o tema da secção; `CardFieldSpec` desce para `widget::card_field`.
   - `cards_estilos`: ids por literal (`hash_node_id`), o censo cobre-os.
   - `A11Y_OPT_OUT`: `card_ink.rs` e `dropdown/card_chip.rs`.
   - ⚠️ **PARA QUEM PORTAR O INSPECTOR (família 6):** `WidgetStore::publish_object_mode/object_mode` foi REMOVIDO.
     Só o «+» do topo da D25b do Inspector o lia, e ele fechava o ciclo interaction↔object_mode. O Inspector
     redesenha-o: p.ex. o `mode_drive` publica um bool simples (`at_top`) cujo campo é definido em `interaction`.
     Não repor o par antigo.
4. **Noto Sans de fábrica (D11)** veio com o porte. Curas do censo sob a Noto: `widget/showcase/encaixe.rs`;
   `property_row` (`que_quebra_o_nome`, `com_os_menus`: o nome que não cabe QUEBRA em duas linhas, o menu pede o
   chip da opção mais larga); `painter-layers` (secções); tokens.

## 2 — Foundational tocado e porquê

| Crate | Porquê |
|---|---|
| `ph2d-editor-core` (127 ficheiros) | núcleo dos cartões (porte), partição de `cards.rs`, curas de DAG, `showcase/` |
| `ph2d-tokens`, `ph2d-panel-tokens`, `docs/design/tokens.json`, `tools/ph2d-widget-sync` | tokens `Card*`/`Kanban*`, `derive::card_tone`, bloco gerado do `widget/` |
| `ph2d-i18n` | `cartoes.rs`, `galeria.rs` (180 chaves), `galeria_reais.rs` (91), `galeria_rive.rs` (36); só inglês |
| `ph2d-node-motion-verlet-rope` | `corda.rs` (+72 linhas, para o `card-rope`) |
| `ph2d-panel-painter-layers`, `ph2d-panel-registry-init` | só testes e a secção que quebra o nome (ver §4) |
| `Cargo.lock` | crates novas (regenerar, nunca resolver à mão) |

Contratos congelados de CLAUDE.md §6 (`NodeOp`/`OpResolver`/`NodeManifest`, `Tool`, `Vector`): **nenhum tocado**
(`git diff 3a3896a7ab HEAD --name-only` não lista `ph2d-nodegraph`, `ph2d-tool-*`, `ph2d-vec-*` nem os gates
`*_contract_surface`).

## 3 — Ids, chaves e ícones novos

- `crates/ph2d-panel-widget-gallery/src/`: `ids.rs` 145 · `ids_reais.rs` 156 · `ids_rive.rs` 33 constantes `pub`.
- `ph2d-i18n`: 180 + 91 + 36 chaves `panel.widget_gallery.*` (+ `cartoes.rs` 111, do porte).
- `docs/design/icons/card-*.svg`: **40** (28 no porte, 12 acrescentados depois, p.ex. interp-linear/cubic/hold e capsule). Nenhum glifo emprestado: o campo mostra a letra da grandeza (08 §1.3).

## 4 — Listas e catracas BAIXADAS (medidas, nunca subidas)

- `cada_nome_deste_painel_cabe_na_coluna_da_seccao` (painter-layers): coluna 220: 12 → 1; coluna 245: 1 → 0;
  a régua passou a perguntar à secção do PRODUTO (`seccoes::seccao`).
- Degrau estreito (`1f9360db07`): `inspector` 86 → 84 cortes (letras 84 → 82); `tokens` 2 → 1; `color_equalization` 2 → 1.
- Nenhuma lista cresceu. Os testes de `a_docked_panel_never_reaches_the_drawing_area` e `a_panel_paints_where_its_tab_says`
  passaram a ler a declaração D24 (lista que FLUTUA) com controlo (`listas >= 1`).

## 5 — Lacunas conhecidas

- Paragens do Gradient: clicáveis, **não arrastáveis** (a posição muda-se no `Position (%)`).
- Camera: o alvo por arrastar da Hierarchy é só uma **dica** na galeria.
- Números de Brush, Image, Audio e Script são **fictícios** (os do código real estão no 08 §1.3).
- `CAN_FLOAT = false` faz o `PH2D_CARDS=0` mostrar também uma aba.
- **Mola do Kanban NÃO extraída** (D19): `ph2d-card-stack` existe, mas o `ph2d-app-kanban` ainda tem a sua
  própria — família 3 de 06 §3 pendente.
- `card-rope` portado **sem consumidor** até ao Inspector/Bones.
- Ainda por portar (linhas seguintes): famílias 5 (Bones) e 6–10 (Inspector, estilos de cor do Vector, cola da shell,
  cenas de smoke, testes `o_*`/`os_*`) — prompts em [`../PROMPT_janela_Bones.md`](../PROMPT_janela_Bones.md) e
  [`../PROMPT_janela_InspectorVector.md`](../PROMPT_janela_InspectorVector.md).

## 6 — O que só a integração apanha

**Impactados e suíte completa ficam para a integração.** Nada disto foi corrido na linha além dos testes por nome
e dos portões tocados. O schema não mudou nesta linha (nada a recontar).

## 7 — Saída dos portões

> (A preencher a partir do relatório do verificador.)

## 8 — Smoke para o dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-NovaUI && PH2D_GALLERY_SMOKE=1 cargo run --profile smoke -p ph2d-host-desktop`.
2. A galeria abre na coluna da direita, como o Inspector. Role: Align, Transform, Fill, Effects…, depois os casos
   reais, as amostras do Rive e os widgets de sempre.
3. Deve acontecer: clicar «+»/«−» do Fill, arrastar a pega da curva em Interpolation, escolher uma paragem do
   Gradient. Tudo responde.
4. Deu errado se: nomes cortados com «...», uma letra emprestada num campo, ou um cartão que não responde ao clique.
