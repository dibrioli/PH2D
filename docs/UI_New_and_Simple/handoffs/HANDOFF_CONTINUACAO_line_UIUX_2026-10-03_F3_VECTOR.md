# Continuação — `line/UIUX`, 2026-10-03 — a F3 do Vector (spec/06), NESTA linha, sem integrar

> Leitor: a próxima janela desta linha. O bloco de entrada é o do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> `cd` + `pwd` + `git branch --show-current` ANTES de ler.

## §1 — Estado

- **Base (merge-base = `main`) `1ad60a1ce`**; o `main` não andou. `git cherry main HEAD` = **54 `+`**
  com este ficheiro, nenhum integrado (ordem do dono: não integrar agora). Árvore limpa.
- **F2 + F3 da Imagem, do Sculpt, do Flip e do Model FECHADAS, smoke do dono APROVADO nas quatro
  (03/10).** O último handoff de integração é
  [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MODEL.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MODEL.md)
  (cita A_ESCALA, O_MENU_ADD, OS_MODOS, O_SCULPT e O_FLIP). O fecho do Vector escreve um handoff NOVO
  sobre o diff ACUMULADO desde a base, que cita os seis.

## §2 — A TAREFA: *Vector ▸ Object · Edit* (D6: forma vetorial → Object · Edit, nós e alças)

Ordem medida da F3 (spec/06 §2.1): Image ✅ → Sculpt ✅ → Flip ✅ → Model ✅ → **Vector**. É o item
difícil da D3: o `DrawMode` tem 14 variantes = 2 modos (`Select` = Object, `Node` = Edit) + 12
ferramentas, com gates vivos.

**O que já existe e se REUSA (não reconstrua):**
- `ph2d_editor_core::object_mode` — `ObjectMode::Edit` já existe (Flip e Model o declaram; o par do
  Vector é `(Vector, Edit)`, outra família). `mode_drive::ModeFamily`: `modes`, `holds`, `enter`,
  `leave`, `follow`, `wants(tools)` e, desde o Model, **`parts`** (o que se edita DENTRO da entidade:
  com `Some`, o cadeado aceita a selecção dentro dela — uma, várias, nenhuma) e **`owner_of`** (uma
  parte responde pelo dono). Meça se os nós/paths do vetor são PARTES no sentido da Hierarquia (são
  entidades?) antes de usar o `parts`; se não forem, a lei exacta serve.
- Os moldes vivos: `ph2d_app_sculpt3d::sculpt_mode` (módulo que não é `Tool`), `ph2d_app_flip::flip_mode`
  (o alvo num tipo só, `FlipTarget`; a ferramenta que chega por porta antiga é ADOPTADA por `wants`;
  a ferramenta só sai quando um modo da família ACABOU, `releases`) e `ph2d_app_field3d::model_mode`
  (o painel segue o modo; a porta antiga pede o modo UMA vez por abertura). O Vector É uma `Tool`
  (`"vector"`) — o molde mais perto é o Flip.
- A shell compõe as famílias em `render_loop/fase_object_mode.rs`; o censo
  `shells/desktop/tests/it/every_object_mode_has_a_composed_family.rs` (tabela `FAMILIAS`, e conta
  PARES (tipo, modo): o `(Vector, Edit)` não pode ser declarado por duas famílias).

**O que a F0 já mediu do Vector (spec/06 §2.1):** quem guarda «o que estou a editar» é
`PenTool::selected_paths` (`ph2d-vec-edit/src/lib.rs:142`); há N objectos; **a caneta cria um path
novo sem olhar o activo**.

**Comece por MEDIR, sem código de produto** (perguntas estreitas, um explorador por pergunta):
- (a) as portas que armam o vetor: o pill **VECTOR** (`MODULE_TRUTHS`: `ModuleTruth::Tool("vector")`),
  *Window*, a aba **Vector** de cima (`CanvasOwner::Tool("vector")`), o layout gravado, as cenas
  `PH2D_BUILD_SMOKE=<n>` / `PH2D_VEC_BONE_SMOKE=<n>` e quantas cenas fazem `set_active("vector")`;
- (b) as 14 variantes do `DrawMode` uma a uma: Object (`Select`), Edit (`Node`) ou ferramenta — e
  quais das 12 ferramentas CRIAM (caneta, formas: são do Object/Add ou do Edit?) e quais EDITAM o
  que existe; o painel do vetor por modo;
- (c) todas as leituras de «o path/objecto em edição» (`git grep` — no Flip eram 14, não 1) e se o
  alvo cabe num tipo só, fora do documento que entra no undo.
- Mostre o desenho ao Enio numa frase por peça, sem jargão, ANTES de codar; e pergunte, com a sua
  recomendação, se o pill **VECTOR** sai (como SCULPT, FLIP e MODEL) e em que modo nasce uma forma
  nova do menu Add (o Flip nasceu em Draw porque nasce vazio; o Model em Edit porque fora dele não
  se desenha — uma forma vetorial é VISÍVEL em Object).

## §3 — Lições desta linha que valem para o Vector

- ⛔ **«O módulo edita o 1.º objecto» nunca é UMA linha** (Flip: 14 leituras; Model: 4 `.next()`).
  `git grep` de todas antes de desenhar, e uma porta só (`FlipTarget`, `scene::root_in_hand`).
- ⛔ **O alvo não mora no documento se o documento entra no diff do undo** (`PartialEq`).
- ⛔ **O cadeado exacto derruba um modo que edita PARTES no 1.º clique** — foi o desvio do Model
  (`ModeFamily::parts`). Pergunte cedo se o Edit do vetor selecciona entidades da Hierarquia.
- ⛔ **Uma aba de cima que abre um painel que ARMA um módulo pode criar objecto** (a aba Model
  plantava a demo). Os layouts pedem MODOS (`CanvasOwner::Mode`); `Open(Edit)` com um desenho Flip
  activo entra no Edit do Flip — é a lei do Workspace do Blender, e é por omissão.
- ⛔ **Um gate de «todo modo tem família» que só conta MODOS não vê uma família sumir** se outra
  declara o mesmo modo — o censo agora conta pares.
- ⛔ **Mudar o estado de FÁBRICA de um painel muda a população dos censos** do
  `ph2d-panel-registry-init` (corra-o com `--all-features`: sem elas 81 falhas falsas).
- ⛔ **Apagar um ficheiro de testes pelo NOME apaga cada gate que mora nele** — `grep -n '^fn '`.
- A mutação acha o que o gate não pergunta (M14 do Model: o seletor com NADA seleccionado). Uma
  edição que falha por o ficheiro ter sido formatado deixa o gate como estava — confirme o VERMELHO
  sob a mutação depois de cada cura.
- A foto apanha o que nenhum gate vê: `PH2D_OBJECT_MODE_SMOKE=6` para o Vector. A placa pode estar
  com outra linha (o guarda diz quem): espere, nunca force.
- O número da shell mede-se como a catraca o mede (todo `.rs` de `shells/desktop`): HEAD do Model
  **196 841**, tecto **196 990** — **folga 149**. Código do modo vai para `mode_drive` / a crate da
  família.

## §4 — Restrições (medidas)

- `screens/hero.rs` **699/700** · `interaction/state/mod.rs` **696/700** · `left_rail.rs`
  **693/700** · `hero/paint.rs` **700/700**: nada solto aí.
- Contrato congelado do vetor ANTIGO (`architecture_vector_contract_surface`, CLAUDE.md §6): o
  `DrawMode` vive em `ph2d-vec-*` (motor novo, não congelado) — confirme antes de mexer.
- A fundação é um DAG (`the_foundation_modules_form_a_dag`): `object_mode` não conhece o Hero.
- Um setter `set_*_mode` cai no censo `every_set_mode_setter_reconciles_or_is_benign`.
- O shell das ferramentas é zsh; o guarda de recursos lê o TEXTO do comando.
- Explorador com perguntas ESTREITAS (≤ 30 passos); confira o código antes de acreditar nele.

## §5 — Como a onda fecha

DIRETRIZ §1.5.9: gate batched 1× sobre o diff acumulado desde `1ad60a1ce` (o
`ph2d-panel-registry-init` com `--all-features`); mutação dos gates novos (controlo VERDE antes, lado
INDEPENDENTE — família falsa no editor-core, a real na crate dela); a família nova na tabela
`FAMILIAS`; foto do smoke (`fotografa_cena.sh`, `PH2D_OBJECT_MODE_SMOKE=6`); handoff de integração NOVO
que cita os seis anteriores; `rm -rf target/*/incremental`; e por último o build de smoke 2× com a 2.ª
saída colada. ⛔ Não integrar nem pushar.
