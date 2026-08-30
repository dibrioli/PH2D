# O diagnóstico das três fotos, e os princípios que o nomeiam (2026-08-30)

> Perguntas 3 e 5 do Enio: pesquisa séria com vista a especificar *"UI/UX de extrema
> simplicidade, altíssima capacidade de ajustes, altíssima funcionalidade e versatilidade"* — e
> iniciar o diálogo.
>
> ⚠️ Este documento **não propõe a spec.** Ele faz três coisas: nomeia o mecanismo de cada
> defeito que o Enio fotografou, mostra que os três são **um só**, e fecha com as perguntas que
> só ele pode responder.

---

## §1 — O fundo teórico (Rogers · Sharp · Preece)

*Interaction Design: Beyond Human-Computer Interaction.* Os cinco princípios de design que o
livro isola — e que aqui não são decoração: cada defeito abaixo viola **um** deles, nomeadamente.

| princípio | pergunta que faz |
|---|---|
| **Visibility** | o que está disponível está **à vista**? |
| **Feedback** | o sistema diz o que fez, na hora? |
| **Constraints** | o design **impede** o gesto errado, em vez de o corrigir? |
| **Consistency** | a mesma coisa está sempre no mesmo sítio e comporta-se igual? |
| **Affordance** | a forma sugere o uso? |

E o par de Norman que o livro usa: **golfo de execução** (distância entre a intenção e a acção
disponível) e **golfo de avaliação** (entre o que o sistema fez e o que se percebe).

⭐ **A frase do Enio — *"não é tão fácil chegar ao resultado desejado"* — é a definição textual
de golfo de execução.** Não é uma queixa estética; é um diagnóstico, e tem endereço.

---

## §2 — Foto 1: os painéis flutuantes tapam as réguas

**Princípio violado: `Visibility`.** Um elemento de referência permanente (a régua) foi coberto
por um elemento temporário (o painel).

**Mecanismo:** o painel flutuante tem coordenada livre. Nada no sistema sabe que a faixa da
régua é território reservado, porque **não existe o conceito de território** — só há
`PanelLayout::Floating` e uma posição.

**O que a referência faz:**

> **Blender, `paradigms.md` — Non Overlapping:** *"The UI should enable you to view all relevant
> options and tools at a glance, **without the need for pushing or dragging windows around**.
> For that reason we default to a subdivided window layout."*

> **Godot, `editor_dock.h:53`:** existem **12 slots** e mais nenhum sítio. Não é possível pôr um
> dock em cima da régua porque **não há coordenada para isso**.

⭐⭐ **A diferença não é «eles ancoram e nós flutuamos».** É que nos dois a posição é
**enumerada**, não contínua — e um enum é um `Constraint` no sentido do Rogers/Sharp/Preece:
o design torna o erro **inexprimível**, em vez de o detectar e corrigir.

⚠️ **E nós já temos metade disto** e está preso ao sítio errado: `PanelLayout::Sidebar` existe,
funciona e tem teste — mas só é alcançável trocando de **tema** (`theme.rs:53`). Ver
[`medicoes/01_o_estado_medido.md §4.1`](../medicoes/01_o_estado_medido.md).

---

## §3 — Foto 2: painéis sobre as vistas 3D, e o gizmo foge para o centro

**Princípios violados: `Constraints` e `Consistency`.**

**Mecanismo, em dois tempos:**
1. O painel pode ficar por cima da vista (mesma causa do §2).
2. **O gizmo desloca-se para se manter acessível.**

⭐⭐ **O passo 2 é o achado, e não é um bug — é uma cura a tratar o sintoma.** Está documentado
no `CLAUDE.md` §5, módulo *3D Modeling*: o gizmo de navegação *"se **desloca para fugir à
moldura**"* (`panel_ops::panel_rects`), descrito ali como *"a fuga mais barata"*.

Isto é o `Constraint` ausente a ser pago em `Consistency`: em vez de impedir que o painel tape a
vista, ensinámos o gizmo a fugir — e **um controlo que muda de sítio é precisamente o que a
referência proíbe**:

> **Blender, `layouts.md` (regra dos pie menus, que generaliza):** *"**The same item should
> always appear at the same position.** Don't confuse users by having items be in different
> places based on context. Muscle memory is key."*

⭐ **Godot tem o campo que resolve isto**, e é uma linha de declaração:

```cpp
// editor_dock.h:91
BitField<DockLayout> available_layouts = DOCK_LAYOUT_VERTICAL | DOCK_LAYOUT_FLOATING;
```

Um dock **declara se pode flutuar**. Um painel de propriedades declara `VERTICAL` e nunca chega
perto de uma viewport. ⛔ Nós não temos esse campo: todo painel pode estar em todo o lado.

⚠️ **Consequência para a spec:** se o docking entrar, **a fuga do gizmo tem de ser removida no
mesmo trabalho.** Deixar as duas é o caso da memória
[`feedback_a_new_remedy_makes_the_old_one_double_counting`](../../../project-memory/feedback_a_new_remedy_makes_the_old_one_double_counting.md)
— o gizmo passaria a fugir de uma moldura que já não o alcança.

---

## §4 — Foto 3 / a queixa dos menus: ⭐⭐ o defeito RAIZ

O Enio: *"Na ausência de Menus na barra superior, os painéis se tornaram extremamente grandes e
mal organizados."*

**Princípio violado: o `conceptual model`** — antes de qualquer um dos cinco. Um painel deixou de
ser *"onde vejo e ajusto as propriedades disto"* e passou a ser *"onde está tudo o que este
módulo sabe fazer"*.

### A medição, no painel exacto da foto

`crates/ph2d-i18n/src/model3d.rs` — **74 entradas** no painel *3D Model*, por família de chave:

| família | nº | o que é | onde a referência põe isto |
|---|---:|---|---|
| `add` | 20 | paleta de formas | popover/menu **Add** |
| `mod` | 8 | modificadores (Hollow, Offset, Mirror X/Y/Z, Array, Radial, Taper) | ✅ propriedade do objecto — **é o único grupo que pertence a um painel** |
| `view` + `camera` + `frame` | **11** | Front/Back/Right/Left/Top/Bottom, Ortho, Frame, Quad View | ⭐ **header da viewport** (Blender) / barra da viewport (Godot) |
| `kind`+`act`+`verb`+`op`+`mode` | 17 | operadores sobre o objecto | pulldowns do editor |
| `export` | **3** | Export Draft / Fine / Max | ⭐⭐ **menu Ficheiro** |
| resto | 15 | título, leituras, estado | — |

⭐⭐⭐ **Um painel de propriedades contém três comandos de exportação de ficheiro e onze de
navegação de câmera.** É isto que o torna gigante — e o Blender tem um paradigma com nome
exactamente contra:

> **`paradigms.md` — Separated Data Properties from Tools:** *"Button windows are now separated
> in either Property lists/bars **or** Tool lists/bars."*

E a tabela «What Goes Where?» de `editors.md` diz, item a item, onde cada um vai:
- **Display options** → direita do header, agrupáveis em popover.
- **Editor-global options** → direita do header ou Tool Settings.
- **Main data selectors** → centro do header.
- **Operadores do editor** → pulldowns do header.

### ⭐ E o mais importante: **os menus já estão escritos**

Medido: **148 itens `CTX_MENU_*`** e **40 handlers de chrome** — `io_menu.rs` é literalmente
*"os itens do menu Ficheiro"*, e há `settings_text`, `settings_filter`, `settings_motion`,
`settings_ppm`, `settings_present`, `settings_unit`, `theme`, `view_toggles`, `transport`, e 8
toggles de módulo.

⇒ **Não falta construir menus. Falta dar-lhes uma barra.** O trabalho é de **realojamento**, não
de construção — o que muda radicalmente o preço da spec.

---

## §5 — Os três são um só defeito

| foto | sintoma | princípio | causa |
|---|---|---|---|
| 1 | painel tapa a régua | Visibility | posição é **contínua**, não enumerada |
| 2 | painel tapa a vista; gizmo foge | Constraints + Consistency | painel não **declara** onde pode estar |
| 3 | painéis gigantes | conceptual model | painel virou **contentor de comandos** |

⭐⭐ **A causa comum: não existe um modelo de REGIÃO.** Não há nada no sistema que diga *"esta
faixa é a régua"*, *"esta área é a viewport"*, *"esta coluna é para propriedades"*. Sem regiões,
a posição é livre (fotos 1 e 2) e não há sítio canónico para um comando, logo tudo cai no painel
(foto 3).

**Blender chama-lhes `Areas` e `Regions`. Godot chama-lhes `DockSlot`.** É a mesma peça, e é a
peça que falta.

⚠️ **E ela é também a resposta à outra exigência do Enio** — *"teremos uma enorme quantidade de
tools que são verdadeiros apps completos aninhados. Logo precisaremos de um sistema de troca de
Modos e Layouts como no Blender."* O Blender responde com a frase que abre `editors.md`:

> *"In a way, **editors are own modes** with own shortcuts and tools — almost like different
> applications."*

⇒ **Modo e Layout não são duas features: um Modo É um Layout de Áreas.** Trocar de modo é trocar
que editor ocupa cada área. Isso não se constrói por cima de painéis flutuantes — precisa das
regiões primeiro.

---

## §6 — A tensão que a spec vai ter de resolver, declarada agora

O Enio pediu **quatro** coisas que não são todas compatíveis por omissão:

1. *"extrema simplicidade"*
2. *"altíssima capacidade de ajustes"*
3. *"altíssima funcionalidade e versatilidade"*
4. e alvo **iPad/Wacom** *e* desktop

⚠️ **(1) contra (2)+(3)** é a tensão clássica. As referências resolvem-na todas da mesma maneira,
e vale nomeá-la porque é uma decisão, não um truque: **profundidade progressiva** — o caminho
comum é curto e visível; o resto está a **um gesto** de distância (sub-painel colapsado, popover,
pulldown), nunca ausente e nunca à vista.

É o `layouts.md`: *"the most important and most commonly used widgets should be exposed more
accessibly. Lesser used widgets should be placed below, or in sub-panels."*

⚠️ **(4) contra tudo:** Godot e Spectrum discordam sobre escala, e a discordância é real —
`EDSCALE` é **um multiplicador global** para densidade de pixel; o `scale-set` do Spectrum é
**por-token** para modalidade de input, e nele o alvo cresce 1,25× enquanto o padding **encolhe**
0,77×. ⛔ **São duas perguntas diferentes** (*«o ecrã é fino?»* / *«o dedo é gordo?»*) e um app
iPad/Wacom precisa das duas. Ver
[`02_referencias_e_licenca.md §4`](02_referencias_e_licenca.md).

---

## §7 — As perguntas que são do Enio (o diálogo começa aqui)

⛔ Nenhuma destas é técnica. Todas mudam o que se constrói.

### Q1 — Ancorado por omissão, ou flutuante por omissão?

- **(a) Ancorado, como Godot/Blender.** Painéis vivem em slots; nada tapa régua nem viewport.
  Preço: perde-se o «feel» de app de iPad que motivou o desenho actual.
- **(b) Ancorado com flutuação declarada** — o modelo do Godot: cada painel **declara** se pode
  flutuar (`available_layouts`). É o mais próximo do que temos e resolve as fotos 1 e 2.
- **(c) Flutuante com territórios reservados** — mantém-se o flutuante, mas régua e viewport
  ficam interditas. Mais barato, ⚠️ e **não resolve a foto 3**.

*Recomendação da linha: **(b)**. É o único que resolve as três fotos, e metade dele já existe
(`PanelLayout::Sidebar` + os 148 itens de menu).*

### Q2 — Uma barra de menus no topo: sim?

Os itens existem (148). A pergunta é se aceita **uma barra de menus clássica** (Ficheiro /
Editar / Ver / …) no topo — que é o que Godot e Blender têm, e o que descongestiona os painéis.

⚠️ Ela custa altura vertical permanente, que num iPad é caro. Blender resolve pondo os pulldowns
**no header de cada editor** em vez de uma barra global. São dois desenhos diferentes e a escolha
é sua.

### Q3 — Quantos temas, e quantas cores?

Hoje: **4 temas**, **83 slots de cor**, **273 dos 355 tokens são cor (77 %)**.
Cortar para 2 temas (um escuro, um claro) e ~40 slots tira ~150 tokens.
⏳ **Antes de cortar tem de se contar quantos slots cada tema usa de forma distinta** — não foi
medido, e cortar sem esse censo é escolher em vez de contar.

### Q4 — Modos: quantos, e quem os define?

O Blender tem *Workspaces* (Layout, Modeling, Sculpting, UV Editing, Texture Paint, Shading,
Animation, Rendering, Compositing, Geometry Nodes) — **na foto que você mandou**. Nós temos
**29 pills** no rail.

A pergunta: os nossos modos são **os módulos** (Vector, Motion, Flip, Sculpt, Model, Audio,
Physics…) ou são **tarefas** (Desenhar, Modelar, Animar, Programar)? ⭐ O Blender escolheu
**tarefa**, e por isso o mesmo editor aparece em vários workspaces.

### Q5 — O editor de código: agora ou depois?

Você nomeou-o (*"editor de texto de codificação"*). ⭐ A subida do stack trouxe um **editor de
texto completo** (`parley::PlainEditor` + `cursor` + `editing` + hit-test exacto) que **não está
ligado a nada**. É a peça mais barata da lista — mas só faz sentido depois de haver uma **área**
onde ela viva.

---

## §8 — O que a linha recomenda como próximo passo

⛔ **Não escrever spec ainda.** Faltam duas medições que são pré-requisito, e as duas são
baratas:

1. **O censo de uso de cor** (Q3) — quantos dos 83 slots cada tema usa distintamente.
2. **A área tapada** — quantos px e que % do canvas os painéis flutuantes cobrem hoje, nas
   configurações que o Enio usa. É a foto 1 virada em número, e é o que transforma *"os painéis
   tapam"* num gate.

⚠️ Sem (2), qualquer decisão de docking é gosto contra gosto. Com (2), é um número antes e um
número depois.
