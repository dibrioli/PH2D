# As referências baixadas, e a triagem de licença (2026-08-30)

> Pergunta 2 do Enio: *"Encontre especificações concretas e bem documentadas sobre UI/UX de
> projetos opensource … e traga para nós, faça até mesmo o download."*
>
> ⛔ **A triagem de licença vem PRIMEIRO** (roteador do `CLAUDE.md` §1 → `SKILL_Cleanroom`).
> Ela decide, alvo a alvo, se podemos **ler**, **citar** ou **portar** — e é a diferença entre
> uma referência e um problema jurídico.

## §1 — A triagem

| Alvo | O que é | Licença | Podemos |
|---|---|---|---|
| **`godot-editor-src`** | motor + editor do Godot (subconjunto) | **MIT** | ⭐⭐ **ler o código E portá-lo.** Sem clean-room, sem intermediário. |
| **`godot-docs`** | manual + tutoriais de UI | CC-BY 4.0 | ler, citar |
| **`godot-contributing-docs`** | guia de contribuição, style guide do editor | CC-BY 4.0 | ler, citar |
| **`blender-developer-docs`** | **o HIG do Blender** (22 páginas) | **CC-BY-SA 4.0** | ⭐ ler e citar livremente — **é documentação, não código** |
| **`gnome-hig`** | HIG do GNOME (52 páginas, `hig/C/`) | CC-BY-SA 4.0 | ler, citar |
| **`spectrum-design-data`** | tokens + esquemas de componente da Adobe | **Apache-2.0** | ⭐⭐ ler **e usar os valores** |
| ~~`godot-proposals`~~ | ⚠️ **só o tracker** — LICENSE + README + TRIAGE. Zero conteúdo de design. | MIT | *baixado e inútil; fica só para não ser re-tentado* |

### ⛔ O que NÃO foi baixado, e porquê

- **Código do Blender (GPL).** ⛔ Não entra nesta árvore e não é lido por esta linha. O Blender
  contribui aqui **como documento** (o HIG, que é CC-BY-SA) e como **comportamento observável**.
  É a mesma disciplina do `SKILL_Cleanroom_Reimplementacao.md` já usada no módulo Sculpt.
  ⚠️ Reparar: o Enio usa o Blender como referência de *paradigma* (modos/layouts) — e para isso
  o HIG **é a fonte melhor**, porque diz a intenção, que o código não diz.
- **Unity.** Proprietário. Não há espec pública de UI do Editor comparável. O que existe de
  público é o *UI Toolkit* (a API para fazer UI **de jogo**), que não é a espec do editor deles.
  ⇒ **o Unity não entra como espec**; entra, quando muito, como observação de produto.
- **Apple HIG (iPadOS/Pencil).** Proprietário, legível online. ⛔ Nada é copiado para a árvore;
  o que ficar é resumo com link.

---

## §2 — Blender: o HIG (22 páginas, `referencias/blender-developer-docs/docs/features/interface/human_interface_guidelines/`)

`accessibility` · `animations` · `best_practices` · `color` · `components/` · `dialogs` ·
`editors` · `general_patterns/` · `glossary` · `icons` · `index` · **`layouts`** · `menus` ·
`modal_interfaces` · `navigation` · **`paradigms`** · `reports` · `selection` · `sidebar_tabs` ·
`tooltips` · `user_feedback/` · `writing_style`

⚠️ `modal_interfaces.md` é um **esboço** (um cabeçalho e nada). Não conte com ele.

### ⭐⭐ `paradigms.md` — os cinco paradigmas, e o primeiro é o diagnóstico da foto 1

> **Non Overlapping** — *"The UI should enable you to view all relevant options and tools at a
> glance, **without the need for pushing or dragging windows around**. For that reason we default
> to a subdivided window layout."*

Os três níveis do Blender:
1. **Screens** — a janela inteira, configurável em *workspaces* com múltiplos editores.
2. **Areas** — o contentor de um editor. *"Editors can each operate similar to a stand alone
   editor."* ⭐ É literalmente a frase do Enio: *"tools que são verdadeiros apps completos
   aninhados."*
3. **Regions** — subdivisão dentro do editor: header, toolbar, sidebar, channels.

Os outros quatro:
- **Non Blocking** — nada bloqueia o utilizador; sem diálogos que exigem preencher antes de
  executar. Quando bloqueia (render/sim) é **indicado e cancelável de imediato**.
- **Non Modal** — o input não muda debaixo da mão. Os modos que sobrevivem são poucos e
  **por-objecto**, não globais nem por-editor. Os restantes são **temporários** (acabam quando o
  utilizador pára).
- **Select → Operate** — escolhe-se o dado, depois a operação. *"There's no active tool mode you
  need to set first."*
- **Operate → Settings** — a ferramenta corre com os últimos valores e **ajusta-se depois**.
  *"This prevents annoying popups forcing you to decide settings before you even know how they'd
  look like."*

E um sexto, que é organização de painel: **Separated Data Properties from Tools** — janelas de
botões são **ou** propriedades **ou** ferramentas, nunca as duas.

### ⭐ `editors.md` — a anatomia, que é uma espec de layout pronta

> *"In a way, editors are own modes with own shortcuts and tools — almost like different
> applications. It is crucial that these present themselves in a familiar way."*

**O header, da esquerda para a direita:**
1. selector de editor
2. **mode toggle** (ou opção de alto nível equivalente)
3. **pulldowns do editor** — *"All operators related to some editor should be available in the
   editor's pulldown section"*
4. *(centro)* **selector do dado principal** e/ou busca
5. *(direita)* **display options** — overlays, modos de desenho, X-Ray, ordenação; agrupáveis em
   popovers

**Regiões comuns:** Toolbar (esquerda) · Sidebar (direita) · Tool Settings · Adjust Last
Operation · Execution · Navigation · Channels.

⭐ **A secção «What Goes Where?» é uma tabela de decisão** para onde cada tipo de controlo vai —
é exactamente a espec que falta ao nosso Inspector de 33 secções.

### `layouts.md` — regras de composição de painel

- **Ordem de importância**: o mais usado em cima; o resto abaixo ou em sub-painéis.
- **Enums viram dropdown** se tiverem >2–3 itens **ou** se os rótulos não couberem. Excepção:
  quando alternar rápido é parte do fluxo, ou quando é só-ícone.
- **Um enum expande a largura toda no topo do painel** se (1) define a função do painel,
  (2) o nome infere-se do título, (3) o texto cabe. ⭐ É o *mode toggle*.
- **Headings** agrupam propriedades relacionadas e cortam repetição de texto.
- **Sub-painéis** são preferíveis a um rótulo solto acima de um bloco — ocupam quase o mesmo e
  **colapsam**.
- **Pie menus: máximo 8 itens**, sem menus/popovers/radios dentro, e **o mesmo item sempre na
  mesma posição** (memória muscular).
- ⛔ **«Do not use fancy spacial layouts to communicate meaning»** — layouts espaciais bonitos
  não integram com o resto e partem a busca de propriedades.

---

## §3 — Godot: o código é MIT, e é pequeno

### ⭐⭐ O gestor de docks inteiro: **1 183 LOC**

`referencias/godot-editor-src/editor/docks/editor_dock_manager.cpp`

O modelo, lido do header (`editor_dock.h:53`):

```
DOCK_SLOT_LEFT_UL   DOCK_SLOT_LEFT_BL   DOCK_SLOT_LEFT_UR   DOCK_SLOT_LEFT_BR
DOCK_SLOT_RIGHT_UL  DOCK_SLOT_RIGHT_BL  DOCK_SLOT_RIGHT_UR  DOCK_SLOT_RIGHT_BR
DOCK_SLOT_BOTTOM    DOCK_SLOT_BOTTOM_L  DOCK_SLOT_BOTTOM_R  DOCK_SLOT_MAIN_SCREEN
```

**12 slots fixos.** Não há posicionamento livre — e é *por isso* que funciona: o utilizador não
consegue pôr um painel em cima da régua, porque não há coordenada para isso.

E o que um dock **é** (`editor_dock.h:76-91`):

| campo | papel |
|---|---|
| `title`, `icon_name`, `dock_icon`, `title_color` | identidade |
| `layout_key` | ⭐ a chave com que o layout é **gravado/restaurado** |
| `shortcut` | atalho próprio |
| `default_slot` | onde nasce |
| `global` | segue o contexto do editor todo, ou é local |
| `transient` | some quando deixa de fazer sentido |
| `closable` | pode ser fechado |
| `allow_switch_screen` | pode ir para uma janela própria |
| `available_layouts` | `VERTICAL \| HORIZONTAL \| FLOATING` — **o que este dock aceita ser** |

⭐ **`available_layouts` é a peça que resolve a foto 2 do Enio:** um dock declara se pode
flutuar. Um painel que **não** pode flutuar nunca vai parar em cima de uma vista 3D.

### ⭐⭐ `EDSCALE` — a escala de UI do Godot inteira, em 12 linhas

`referencias/godot-editor-src/editor/themes/editor_scale.h`:

```cpp
class EditorScale {
    static float _scale;
public:
    static void set_scale(float p_scale);
    static float get_scale();
};
#define EDSCALE (EditorScale::get_scale())
#define EDSCALE_RND(m_value) (Math::round(m_value * EDSCALE))
```

**Um float global**, e toda dimensão do editor escreve-se `x * EDSCALE`. Godot ship-a HiDPI em
três SOs com isto. ⚠️ **Note o `EDSCALE_RND`**: o arredondamento é parte do contrato, não um
detalhe — é o que impede meias-linhas borradas.

### O sistema de tema: **7 527 LOC**, dos quais ~5 562 são *dois temas*

| ficheiro | LOC | o que é |
|---|---:|---|
| `theme_modern.cpp` | 2 960 | um tema completo |
| `theme_classic.cpp` | 2 602 | outro tema completo |
| `editor_theme_manager.cpp` | 760 | a máquina |
| `editor_fonts.cpp` | 567 | fontes |
| `editor_color_map.cpp` | 239 | mapa de cor |
| `editor_icons.cpp` | 227 | ícones |
| `editor_theme.cpp` | 131 | o tipo |

⇒ **a máquina de tema são ~1 900 LOC**; o resto é conteúdo.

### Documentação Godot que interessa

`godot-docs/tutorials/ui/`: `gui_containers` · `size_and_anchors` · `gui_skinning` ·
`gui_theme_type_variations` · `custom_gui_controls` · `control_node_gallery` · `gui_using_fonts`
· `gui_navigation` · `gui_using_theme_editor` · `creating_applications`.

⚠️ **Cuidado com o enquadramento:** estes documentam o sistema de `Control` para fazer UI **de
jogo**. O editor usa o mesmo sistema, mas os documentos falam do ponto de vista de quem faz um
jogo. A espec do *editor* está no **código** (MIT) e no `godot-contributing-docs`.

---

## §4 — Adobe Spectrum (Apache-2.0): a resposta ao iPad/Wacom

`referencias/spectrum-design-data/packages/`: `tokens` (91 ficheiros JSON) ·
`component-schemas` · `design-data` · `design-data-spec` · `design-system-registry` ·
`token-names`.

⭐ **Porque é a referência certa para o alvo do Enio:** é o design system de uma empresa que
ship-a ferramentas criativas **com caneta, em tablet** (Fresco) e no desktop, e publica os
tokens com a licença permissiva. Não é um design system de website.

### ⭐⭐ O mecanismo `scale-set` — `desktop` e `mobile` no MESMO token

```json
"component-height-100": {
  "$schema": ".../scale-set.json",
  "sets": {
    "desktop": { "value": "32px" },
    "mobile":  { "value": "40px" }
  }
}
```

| token | desktop | mobile | razão |
|---|---:|---:|---:|
| `component-height-50` | 20 px | 26 px | 1,30× |
| `component-height-100` | 32 px | 40 px | 1,25× |
| `component-height-200` | 40 px | 50 px | 1,25× |
| `component-height-300` | 48 px | 60 px | 1,25× |
| `base-padding-horizontal-2x-large` | 18 px | **14 px** | **0,78×** |
| `base-padding-horizontal-extra-large` | 16 px | **12 px** | **0,75×** |

⭐⭐ **A lei: o ALVO cresce ~1,25×, o PADDING INTERNO encolhe ~0,77×.** Escalar tudo por 1,25 —
o que uma pessoa faria por instinto, e o que o `EDSCALE` do Godot faz — dá **o oposto** do que a
Adobe entrega para toque.

⚠️ E é **opt-in por token**: `base-padding-horizontal-extra-small` é `8px` nos dois, sem `sets`.
*A escala não é um multiplicador global; é uma propriedade de cada token que precisa dela.*

⚠️⚠️ **Godot e Spectrum discordam, e a discordância é informação.** `EDSCALE` é um multiplicador
global (simples, mantível, e é para **densidade de pixel**); o `scale-set` é por-token (mais
caro, e é para **modalidade de input**). São perguntas diferentes: *«o ecrã é fino?»* e *«o dedo
é gordo?»*. ⛔ Um app iPad/Wacom precisa das **duas** e não pode servi-las com o mesmo número.

---

## §5 — GNOME HIG (`referencias/gnome-hig/hig/C/`, 52 páginas)

`design-principles` · `visual-layout` · `patterns` · `pointer-and-touch-input` · `keyboard-input`
· `menu-bars` · `primary-menus` · `secondary-menus` · `popovers` · `header-bars` · `action-bars`
· `sidebar-lists` · `view-switchers` · `selection-mode` · `overlaid-controls` ·
`empty-placeholders` · `initial-state-placeholders` · `writing-style` · `typography` ·
`display-compatibility` · e ~30 páginas de componente.

⚠️ **É o HIG da era GTK3** (`gnome-devel-docs`), não o site novo `developer.gnome.org/hig`.
Continua útil pelo que tem e os outros não: **`pointer-and-touch-input`** e
**`overlaid-controls`** — que é literalmente o padrão da foto 1 (controlo sobreposto ao
conteúdo) com as regras de quando é aceitável.

---

## §6 — ⏳ Buracos nomeados desta recolha

1. **Falta uma referência pen-first com espec publicada.** O Spectrum é o mais próximo
   (Adobe/Fresco) mas os tokens não falam de **caneta** — falam de `desktop`/`mobile`. O Enio
   excluiu o Krita. ⏳ Não achei espec pública de app de iPad de qualidade; a mais autoritativa é
   a Apple HIG, que é proprietária e não pode entrar na árvore.
2. **`modal_interfaces.md` do Blender está vazio** — e modos são metade da pergunta do Enio. O
   que existe sobre modos está espalhado por `paradigms.md` («Non Modal») e `editors.md`
   («editors are own modes»). ⇒ para *Modos e Layouts* a fonte melhor é o **manual do
   utilizador** do Blender (`docs.blender.org/manual/en/latest/interface/`), não baixado aqui.
3. **Não foi baixado o código de docking de mais ninguém.** Se a decisão for docking, vale
   comparar com o *Qt Advanced Docking System* (LGPL — ⛔ ler com cuidado) e o *Dear ImGui*
   docking (MIT). ⏳ Não feito.
