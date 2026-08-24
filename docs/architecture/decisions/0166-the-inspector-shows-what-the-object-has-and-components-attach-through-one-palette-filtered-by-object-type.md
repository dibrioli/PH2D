# ADR-0166 — O Inspector mostra o que o objeto TEM; componente anexa-se por UMA porta, com categorias e filtro por TIPO DE OBJETO

- **Status:** Accepted (Enio, 2026-08-24 — instruções complementares à ordem de implementação da `line/components`)
- **Data:** 2026-08-24
- **Linha:** `line/components` (afina as fases **F0** e **F3** do [plano vivo](../../Components/05_plano_de_implementacao.md))
- **Depende de:** [ADR-0164](0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md) — o descritor de componente (F0) é o substrato desta decisão
- **Toca:** `ph2d-component-desc` (crate nova da F0: `category` · `applies_to` · `attach`) · `ph2d-ecs/scene/registry.rs` (`insert_default`) · `ph2d-panel-inspector` (a cascata deixa de ser literal) · `shells/desktop` (o modelo da paleta + o dreno do pick) · `ph2d-editor-core` (o id do `+`; **o widget da paleta NÃO se mexe**)
- **Não move:** [ADR-0074](0074-sprite-component-boundary.md) (a regra dos 3 lugares e o teto de 32 opcionais continuam de pé) · os três contratos congelados do CLAUDE.md §6 · o widget `command_palette` (é consumido como está)

## Contexto

O Enio, ao ordenar a implementação, nomeou o modelo: **Unity**. *"Os componentes não devem estar
todos no inspector desde o nascimento dos objetos, mas devem ser acrescentados ao objeto conforme a
necessidade do usuário"* — com um botão `+` no próprio Inspector, um modal **idêntico ao do módulo
Motion Nodes** (categorias), e **um filtro por tipo de objeto**, porque *"diferente de várias game
engines 2d, nesta temos uma variedade de tipos de objetos: imagens, vectors, Flip Objects, objetos
3d de vários tipos"* e *"9-slice provavelmente não se aplica a nada além de uma sprite de imagem"*.

**O que a medição do código acrescentou ao pedido** (2026-08-24, worktree da linha):

1. ⭐ **O modal já existe, já é genérico, e o doc-comment dele antecipa este uso.**
   [`ph2d-editor-core/src/widget/command_palette.rs`](../../../crates/ph2d-editor-core/src/widget/command_palette.rs)
   é *"a full-screen, centred modal that lists a large set of choices grouped by coloured category
   (Motion Nodes' 'Add Node', **and reusable by any future browse-everything picker**)"*, e diz-se
   **genérico**: *"it knows only `PaletteModel` … it does NOT know what an item MEANS; the shell that
   opened it maps the picked `id` back to a real action"*. Tem scrim, cascata de entrada, busca
   (`item_matches` — **um** predicado servindo o filtro pintado e o `Enter`), sub-clusters, e promoção
   a duas colunas por contagem. **Já tem dois construtores de modelo** — a biblioteca de nós
   ([`motion_bridge_library.rs`](../../../shells/desktop/src/render_loop/motion_bridge_library.rs)) e
   o `Ctrl+K` ([`global_palette.rs`](../../../crates/ph2d-editor-core/src/screens/hero/global_palette.rs)).
   O nosso é **o terceiro construtor**, não um modal novo.
2. ⚠️ **Já existem CINCO portas de "adicionar componente", cada uma escrita à mão, e nenhuma se
   chama assim:** `INSP_PLAYER_ADD` · `INSP_ANCHOR_ADD` · `INSP_ANIM_ADD` · `INSP_PHYS_ADD` + o botão
   de anexar da §5 9-Slice. Cinco respostas à mesma pergunta, em cinco sítios, descobríveis só por
   quem já sabe que a secção existe. *Esta decisão não acrescenta uma porta: substitui cinco por uma.*
3. ⭐ **A lei que se quer já está escrita no repo, em miniatura, e por outra linha.** O doc-comment de
   [`sections/slice_nine.rs`](../../../crates/ph2d-panel-inspector/src/sections/slice_nine.rs) declara:
   *"**Sem componente, a seção mostra UM botão e mais nada.** Não pinta bordas a zero: ausência de
   autoria não é «bordas a zero», e mostrar zeros seria afirmar um valor que não existe"* — e
   *"anexar é **inerte**: um botão que abre uma seção não pode ser uma edição destrutiva disfarçada"*.
   As duas frases são a decisão abaixo, generalizada.
4. **O registro não sabe criar um componente do zero** — a vtable tem `insert_from_bytes`,
   `serialize` e `remove`, e **nenhum** `insert_default`
   ([`registry.rs`](../../../crates/ph2d-ecs/src/scene/registry.rs)). É o elo que a F0 fornece, e é a
   razão de este ADR depender dela.
5. **107 componentes registados no boot** (69 `ph2d-ecs` + 1 `ph2d-render` + 32 `ph2d-physics-ecs` +
   5 `ph2d-field-ecs`), e ⚠️ **nem todos são autoráveis**: `VecPathRef`, `PaintedDoc`, `BakedForm` e
   `FlipObjectRef` são **pontes de identidade** — máquina, não escolha do artista. Uma paleta que
   liste "todo tipo registado" ofereceria quatro coisas que ninguém deve anexar à mão.
6. **O tipo de objeto lê-se por PRESENÇA de um marcador**, não por um campo: `Sprite` (imagem) ·
   `VecPathRef` (vetor) · `FlipObjectRef` (Flip) · `PaintedDoc` (Painter) · `FieldObject` (modelo 3D) ·
   `BakedForm`. Sem nenhum deles, o objeto é **vazio** — que é exatamente o que a F3 aprende a criar.

## Decisão

### 1. O Inspector pinta o que o objeto TEM — a base é `Transform` + `Name`, e mais nada

A cascata literal de secções (hoje `populate()` é uma lista de 19 funções e o paint é uma sequência
escrita à mão) passa a ser **derivada da presença do componente**. O objeto vazio nasce com
`Transform` + `Name` (+ `StableId`/`RootOrder`, que são máquina e não têm secção) e o Inspector dele
mostra **duas** secções. Toda a restante — ordenação, amostragem, blend, folha, 9-slice, âncoras,
animação, física, joint, roda, player — aparece **se, e só se**, o componente dela estiver na
entidade.

⚠️ **Isto vale também para os campos que hoje vivem DENTRO da `Sprite`.** O corte da F1
(`SpriteCornerTint` · `SpriteSheet` · `SpriteRegion`) passa a ter uma segunda razão além do tamanho
do tipo: enquanto o dado for campo de um componente que todo objeto-imagem tem, **não há como não o
mostrar**. Um campo só pode desaparecer da vista quando é um componente que pode estar ausente.

### 2. UMA porta: o `+` do Inspector abre a paleta que já existe

Um botão `+` no cabeçalho do Inspector (`INSP_ADD_COMPONENT`) abre o
`ph2d_editor_core::widget::command_palette` com um `PaletteModel` novo, construído do registro:
**grupos = categorias coloridas**, itens = componentes anexáveis, busca herdada do widget. O pick
volta pelo mesmo canal dos outros dois construtores (`take_command_pick_if` — cada dreno reconhece
os **seus** ids; ⚠️ o canal já tem dois consumidores e o dreno é condicional **de propósito**, senão
quem recebe o pick passa a ser a ordem dos drenos no quadro).

**As cinco portas por-secção são subsumidas**, não mantidas ao lado — duas respostas a *"como se
adiciona um componente?"* é a divergência que esta decisão existe para apagar. (O botão de anexar da
9-Slice pode sobreviver como atalho **da secção já visível**; o que não sobrevive é ser a única rota.)

### 3. A aplicabilidade é DECLARADA no descritor, nunca inferida — e o filtro é a vista, não uma cerca muda

`ComponentDesc` ganha, ao lado do `field_id`/política/refs da F0:

```
category:   ComponentCategory      // o grupo colorido da paleta (o NodeUiCategory dos componentes)
attach:     Attach                 // Authored { applies_to } | Machinery (nunca oferecido, nunca secção)
applies_to: ObjectKinds            // bitset sobre {Empty, Image, Vector, Flip, Painted, Model3D, …}
```

- **`Machinery` é o que tira as quatro pontes de identidade da paleta** (facto 5) — e a ausência
  passa a ser **declarada**, não um esquecimento que ninguém nota.
- **A paleta abre filtrada pelo tipo do objeto selecionado.** O que não se aplica **não some sem
  explicação**: fica sob *Show all*, **esmaecido e com a razão nomeada** ("9-Slice needs an image").
  ⛔ Nem oferecer-e-não-fazer-nada (a DIRETIVA §2 proíbe o no-op silencioso), nem apagar da lista
  (um componente que existe e é invisível lê-se como defeito, e gera o report que esta linha quer
  evitar). *Esmaecido ainda despacha* — aqui, despacha a explicação.
- ⚠️ **`applies_to` tem de dizer de QUE DADO o componente depende**, senão vira uma segunda fonte de
  verdade que envelhece contra o código. A regra: um componente declara o tipo cujo **marcador ele
  lê** (a `SliceNine` calcula-se a partir da `Sprite` ⇒ `Image`). O gate é um **censo**: todo tipo
  registado tem descritor; nenhum `Authored` declara conjunto vazio; **a paleta e o gating de secção
  leem a MESMA declaração** — uma fonte, dois consumidores.

### 4. Anexar é INERTE e desfazível — a lei da 9-Slice, generalizada

O componente chega no **ponto neutro** (`insert_default`, F0): anexar abre uma secção, nunca muda o
que se vê. O desfazer vem de graça — a mudança de archetype altera os bytes e o diff pega
([doc 01 §6.2](../../Components/01_auditoria_modelo_de_objeto.md)); ⚠️ e é precisamente o caso que a
**F2** tem de ver, porque **remover um componente não carimba o tick de ninguém**
([ADR-0164](0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md) §2).

## Consequências

### ⚠️ A lei da FACE VAZIA não é revogada — ela MUDA DE SÍTIO, e isto tem de ficar escrito

O repo tem uma lei medida: *"a cura de «não há rota» é a **face vazia**, nunca o desaparecimento"*
([memória](../../../project-memory/feedback_the_three_ui_seam_questions_miss_the_fourth_the_sequence.md)),
e ela está viva no código — a §14 Player publica `Some` para todo corpo dinâmico **com ou sem o
componente**, com o comentário *"porque o botão dela é o que faz o comportamento existir"*
([`snapshots.rs`](../../../shells/desktop/src/render_loop/snapshots.rs)). Lida de fora, esta decisão
parece contradizê-la.

**Não contradiz, e a distinção é o que a torna segura:** a face vazia existe porque, sem ela, *a
secção era a única rota para a feature*. Com **uma** porta global (`+` → paleta), deixa de ser — a
feature continua alcançável, por um caminho que é o mesmo para todos os componentes e que se
**descobre sem saber que a secção existe**. O que a lei proíbe é tornar algo inalcançável; o que esta
decisão faz é trocar N rotas escondidas por uma rota única e nomeada. ⛔ **Corolário operacional: a
F3 não pode apagar uma face vazia antes de a porta nova estar viva e testada** — nessa janela a
feature fica de facto inalcançável, e é assim que a lei foi paga da primeira vez.

### O resto do preço, nomeado

- **Secções que desaparecem mudam o que toda sonda de UI mede.** Testes de seam, golden e smokes que
  hoje assumem "o Inspector de uma sprite tem N secções" passam a ver menos — e a mudança **é a cura**.
- **`ObjectKinds` é vocabulário novo e transversal.** Ele nomeia os tipos de objeto do app inteiro; se
  divergir do que os módulos entendem por "um objeto Flip", vira a segunda fonte de verdade que o item
  3 proíbe. Nasce **derivado do marcador**, e o censo é o que o prende.
- **A `Sprite` congelada em 20 campos (ADR-0074) fica sob pressão de produto, não de tamanho.** Todo
  campo dela é, por construção, uma coisa que o artista não pode remover da vista. O corte da F1 tira
  três; os outros ficam declarados como "base do objeto-imagem" — e isso é agora uma **afirmação de
  produto**, que o ADR-0074 não fazia.
- **`LuauScript` continua fora do registro do boot** (a ambiguidade §8.1 do doc 01 sobrevive): um
  componente não registado não pode aparecer na paleta nem ser salvo. Fica **nomeado**, não resolvido.

## Alternativas medidas e recusadas

| alternativa | por que não |
|---|---|
| **Manter as cinco portas por-secção** (o que existe) | cinco respostas à mesma pergunta, cada uma escrita à mão, nenhuma descobrível como *"adicionar componente"*; e a sexta secção pagaria a sexta |
| **Construir um modal novo para componentes** | o widget genérico existe, o doc-comment dele **antecipa este uso**, e tem scrim/cascata/busca/sub-clusters resolvidos. Um segundo modal seria uma segunda lei de teclado, foco e fecho |
| **Inferir a aplicabilidade do código** (*"a secção tem dados?"*) | não responde por um componente que **ainda não foi anexado** — que é exatamente a pergunta da paleta; e um predicado que enumera os seus consumidores apodrece ([memória](../../../project-memory/feedback_a_condition_that_enumerates_its_readers_rots.md)) |
| **Esconder o inaplicável sem escape** | um componente que existe e é invisível lê-se como defeito — é o report que se quer evitar, e custa mais caro do que a linha esmaecida |
| **Oferecer tudo e deixar anexar o que não se aplica** | no-op silencioso, proibido pela DIRETIVA §2; e uma secção que não faz nada ensina que o painel mente |
| **Listar todo tipo REGISTADO na paleta** | ofereceria as quatro pontes de identidade (`VecPathRef`/`PaintedDoc`/`BakedForm`/`FlipObjectRef`), que são máquina; daí `Attach::Machinery` ser **declarado** |
| **Apagar as faces vazias antes da porta nova** | abre uma janela em que a feature fica inalcançável — a forma exata do defeito que a lei da face vazia foi escrita para curar |

## Referências

- **Instrução do dono:** Enio, 2026-08-24 (complemento à ordem de implementação da `line/components`)
- **O plano que esta decisão afina:** [`05_plano_de_implementacao.md`](../../Components/05_plano_de_implementacao.md) §F0 e §F3
- **O widget reusado:** [`command_palette.rs`](../../../crates/ph2d-editor-core/src/widget/command_palette.rs) · construtores-precedente: [`motion_bridge_library.rs`](../../../shells/desktop/src/render_loop/motion_bridge_library.rs), [`global_palette.rs`](../../../crates/ph2d-editor-core/src/screens/hero/global_palette.rs)
- **A lei em miniatura:** [`sections/slice_nine.rs`](../../../crates/ph2d-panel-inspector/src/sections/slice_nine.rs)
- **O estado medido do Inspector:** [doc 01 §4](../../Components/01_auditoria_modelo_de_objeto.md)
- ADRs que este honra: [0025](0025-gameobject-model.md) (GameObject = Entity + Components) · [0029](0029-trait-driven-panel-host.md) (painel é crate tipada) · [0074](0074-sprite-component-boundary.md) (regra dos 3 lugares) · [0164](0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md)
