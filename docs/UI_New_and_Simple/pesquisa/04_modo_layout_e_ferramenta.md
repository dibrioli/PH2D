# Modo, Layout e Ferramenta — são TRÊS eixos, não um (2026-08-30)

> ⚠️ **Este documento existe por uma correcção do Enio.** Na primeira volta eu perguntei-lhe
> *"quando você troca de Modo, o que está trocando?"* e ofereci **tarefa** ou **módulo** — o que
> trata Modo e Layout como a mesma coisa. Ele corrigiu:
>
> > *"Aí como no Blender há duas coisas: Layout e Mode. Alguns objetos têm modo de edição próprio
> > como vector, cujas tools são completamente específicas e onde toda a tela vai mudar. Isso seria
> > como um mesh do Blender que tem modos Object, Edit, Sculpt, etc. Já Editor 2D, Editor de texto,
> > Runtime, são layouts."*
>
> Fui aos manuais confirmar. **Ele está certo, e a distinção é mais forte do que parece: os dois
> eixos têm DONOS diferentes.** E há um terceiro que os dois manuais separam e nós não.

---

## §1 — O que os manuais dizem

### Layout / Workspace — **o utilizador escolhe, e vive na barra de cima**

> **Blender, `manual/interface/window_system/workspaces.rst`:**
> *"Workspaces are essentially **predefined window layouts**. Each Workspace consists of a set of
> **Areas** containing **Editors**, and is geared towards a specific task such as modeling,
> animating, or scripting."*
> *"Workspaces are located at the **Topbar**."*

Os 10 do Blender: Layout · Modeling · Sculpting · UV Editing · Texture Paint · Shading ·
Animation · Rendering · Compositing · Geometry Nodes (+ 2D Animation, Masking, Motion Tracking,
Video Editing como extra).

Propriedades: **abas** na barra de cima · duplo-clique renomeia · adicionar a partir de
**modelo** · duplicar / apagar · `Ctrl-PageUp/PageDown` cicla · **gravado no ficheiro**.

> **Godot, `getting_started/introduction/first_look_at_the_editor.rst`:**
> *"along the window's top edge, it features **main menu** on the left, **workspace** switching
> buttons in the center (active workspace is highlighted)"*

Os 5 do Godot: 2D · 3D · Script · Game · AssetLib. ⚠️ Fixos — **o Godot não deixa o utilizador
criar workspaces**; o Blender deixa.

### Modo — **o OBJECTO decide quais existem, e vive no cabeçalho da ÁREA**

> **Blender, `manual/editors/3dview/modes.rst`:**
> *"Modes allow editing different aspects of objects."*
> ⭐ *"**Which modes are available depends on the object's type.**"*
> *"You can change the current mode using the *Mode* selector in the **3D Viewport header**."*

E o que um modo faz — as três consequências, textuais:

1. *"Each mode changes the **header and Toolbar** to show its own unique set of menus and tools.
   This also means it affects the **available keyboard shortcuts**."*
2. *"Modes can **completely change the look of the viewport**."* (ex.: Weight Paint sombreia o
   objecto para mostrar pesos que normalmente não se veem.)
3. ⭐⭐ *"Modes can **affect other editors**. For example, the **UV Editor can only be used if the
   3D Viewport is in Edit Mode**."*

⭐ **É a frase do Enio, palavra por palavra:** *"toda a tela vai mudar para exibir tudo."*

Os 13 modos, e quem os pode ter:

| modo | quem |
|---|---|
| **Object** | ⭐ **todos os tipos** — é o único universal |
| **Edit** | malhas, curvas, superfícies, Grease Pencil… |
| Sculpt · Vertex Paint · Weight Paint · Texture Paint · Particle Edit | **só malha** |
| Pose | **só armature** |
| Draw · Sculpt · Edit · Vertex Paint · Weight Paint (GP) | **só Grease Pencil** |

> **Godot:** *"This toolbar changes based on the **context and selected node**."*

⇒ O Godot tem o mesmo eixo, com outro nome: a barra da viewport muda com o **tipo do nó
seleccionado**. Não é um selector explícito, é implícito na selecção.

### ⭐⭐ E a costura entre os dois é **UM CAMPO OPCIONAL**

> **Blender, `workspaces.rst`, secção *Workspace Settings*:**
> **`Mode`** — *"Switch to this Mode when activating the workspace."*

⇒ **um Workspace pode PEDIR um modo ao ser activado, e mais nada.** Não são acoplados: são
ortogonais com um atalho declarado. É por isso que o workspace *Sculpting* te põe em Sculpt Mode
sem que Workspace e Mode sejam a mesma coisa.

*(Há outros dois campos no mesmo sítio, que mostram como o Workspace é «configuração de sessão» e
não «estado do objecto»: **Pin Scene** e **Filter Add-ons** — que add-ons estão ligados neste
workspace.)*

### Ferramenta — o terceiro eixo

Blender tem uma **Toolbar** (tecla `T`) com ferramentas *dentro* do modo: Tweak, Select Box,
Cursor, Move/Rotate/Scale, Annotate, Measure, e — em Edit Mode — Extrude, Inset, Bevel, Loop Cut,
Knife… ⇒ **a ferramenta muda o GESTO; o modo muda o que se pode editar.**

---

## §2 — Os três eixos, resumidos

| eixo | quem decide | onde vive | o que muda |
|---|---|---|---|
| **Layout** | o **utilizador** | barra de cima (abas) | que **áreas** existem e que **editor** está em cada uma |
| **Modo** | o **tipo do objecto** seleccionado | cabeçalho da **área** | que **ferramentas** existem, que **atalhos** valem, o **aspecto** da vista, e que outros editores **funcionam** |
| **Ferramenta** | o utilizador, dentro do modo | **toolbar** da área | o **gesto** do ponteiro |

⚠️ **Os três são ortogonais.** *Layout `Animação` + objecto malha em modo `Sculpt` + ferramenta
`Draw`* é um estado legítimo, e nenhum dos três sabe dos outros — **excepto** pelo campo opcional
`Mode` do Workspace.

---

## §3 — ⛔ Onde o PH2D confunde os três, medido

### 3.1 — O `DrawMode` do vetor são **2 modos + 12 ferramentas**, num enum só

`crates/ph2d-tool-vector/src/params_mode.rs` — 14 variantes. Lidos os doc-comments, dois deles
são modos no sentido do Blender e os outros doze são ferramentas:

| variante | doc-comment (nosso) | é |
|---|---|---|
| **`Select`** | *"seleciona e TRANSFORMA a forma pelo gizmo. **Não toca a geometria**"* | ⭐ **Object Mode** |
| **`Node`** | *"edita âncoras e handles do path selecionado. **Nunca cria um path**"* | ⭐ **Edit Mode** |
| `Pen` · `Pencil` · `Shape` · `Text` · `Build` · `Connect` · `PickBlend` · `Fillet` · `Chamfer` · `Width` · `Cut` · `Frame` | cada um justifica-se por **o gesto ser outro** | **ferramentas** |

⭐⭐ **A prova está nos nossos próprios comentários.** Cada uma das doze explica-se assim:
*"é um modo e não uma variante da caneta **porque o gesto é o oposto**"* (Pencil), *"no Node estas
alças **competiriam com as âncoras**"* (Width), *"o gesto é escolher formas no canvas, **não editar
a selecionada**"* (PickBlend). **Todas dizem «gesto». Nenhuma diz «que aspecto do objecto se
edita»** — que é a definição de modo.

⚠️ **A consequência prática:** achatados num enum, **não se pode exprimir «estou em Edit com a
ferramenta Fillet»**. Ou se está em `Fillet`, ou em `Node` — e sair do Fillet devolve-te a um modo
que o app tem de adivinhar.

### 3.2 — Os 29 pills do rail são quase todos **ferramentas**, e 3 são uma preferência

| grupo | ids | é |
|---|---|---|
| `RAIL_BRUSH` `ERASER` `FILL` `SMEAR` `BLUR` `CLONE` `LIQUIFY` `INPAINT` `MASK` `SELECTION` `SHAPES` (+5 formas) `EYEDROPPER` `TRANSFORM` | ~19 | **ferramentas** |
| `RAIL_SHOW_HIERARCHY` · `RAIL_SHOW_INSPECTOR` | 2 | ⚠️ **layout** (que painéis estão abertos) → menu *Ver* |
| `RAIL_SIZE_SMALL` · `MEDIUM` · `LARGE` | 3 | ⛔ **uma preferência** (tamanho do botão do rail) → *Preferências* |
| `RAIL_BACKDROP` · `RAIL_*_IDS` | resto | chrome interno |

### 3.3 — Os 9 toggles de módulo são a coisa mais próxima de **Layouts** que temos

`vector_toggle` · `motion_toggle` · `flip_toggle` · `physics_toggle` · `model3d_toggle` ·
`sculpt3d_toggle` · `image_tools_toggle` · `tokens_toggle` · `authored_toggle`

⚠️ **Mas são interruptores independentes, não um selector exclusivo.** Um Layout é *um de N*; nove
toggles são *2⁹ = 512 combinações*, das quais quase nenhuma foi desenhada.

---

## §4 — O que isto corrige na D3

⛔ **A minha pergunta original era mal posta.** *"Modos por tarefa ou por módulo?"* trata Modo
como sinónimo de Layout. A resposta certa é **os dois eixos existem** e a pergunta *"por tarefa"*
aplica-se só ao **Layout**.

⭐ **A escolha do Enio («por tarefa») continua válida — mas é sobre LAYOUTS.** O eixo dos Modos
não é escolhido: **é derivado do tipo do objecto**, exactamente como no Blender.

⇒ A D3 fica lida como:

- **Layouts** — poucos, por tarefa, escolhidos pelo utilizador, abas na barra de cima. *Editor 2D,
  Editor de Texto, Runtime, Animação…* (os exemplos são do Enio).
- **Modos** — ⛔ **não são uma lista global.** Cada **tipo de objecto** declara os seus, e o
  selector vive no **cabeçalho da área**, não na barra de cima.
- **Ferramentas** — dentro do modo, na toolbar da área.

---

## §5 — ⏳ O que falta decidir (e agora as perguntas são as certas)

1. **Que tipos de objecto temos, e que modos declara cada um?** Rascunho a confirmar:

   | tipo | modos plausíveis |
   |---|---|
   | todos | **Object** (mover/rodar/escalar — o nosso `Select`) |
   | `VecPath` | Edit (o nosso `Node`) · Text? |
   | malha 3D | Edit · Sculpt · Paint |
   | sprite / raster | Paint · Mask |
   | objecto Flip | Draw · Edit |
   | peça SDF (Model) | Edit |
   | corpo de física | Object apenas? |

   ⚠️ **É uma pergunta de produto, não de código** — e é a que substitui a que eu fiz mal.

2. **A lista de Layouts.** Continua por dar; o Enio deu três exemplos (*Editor 2D, Editor de
   texto, Runtime*).

3. **Adoptamos o campo `Mode` do Workspace?** (*"switch to this mode when activating"*.) É um
   campo opcional e resolve *"o layout Escultura põe-me em modo Sculpt"* sem acoplar os eixos.

4. ⏳ **Como se parte o `DrawMode`** nos dois eixos sem partir o que já funciona. **Não desenhado.**
   ⚠️ Ele tem **14 variantes vivas, com gates**, e é a superfície de uma ferramenta inteira.

---

## §6 — Fontes

- `referencias/blender-manual/manual/interface/window_system/workspaces.rst` (CC-BY-SA 4.0)
- `referencias/blender-manual/manual/editors/3dview/modes.rst` (CC-BY-SA 4.0)
- `referencias/blender-developer-docs/…/human_interface_guidelines/paradigms.md` — «Non Modal»:
  *"Editing modes … **these now are per-object states**, and not editor dependent or global."*
  ⚠️ **Esta frase estava citada no meu `pesquisa/03` desde o início e eu não a segui.**
- `referencias/godot-docs/getting_started/introduction/first_look_at_the_editor.rst` (CC-BY 4.0)
