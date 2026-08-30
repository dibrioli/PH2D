# As decisões do Enio — FECHADAS (2026-08-30)

> ⛔ **Não re-litigar.** Estas três foram postas ao Enio com as alternativas, os preços e o
> mecanismo de cada uma, e ele decidiu. A spec descende daqui.
>
> Quem quiser reabrir uma delas precisa de um **facto novo medido**, não de uma preferência.

---

## D1 — Painéis: **ancorados, com flutuação DECLARADA**

> *"Cada painel diz se PODE flutuar."*

O modelo do Godot (`editor_dock.h:91`):

```cpp
BitField<DockLayout> available_layouts = DOCK_LAYOUT_VERTICAL | DOCK_LAYOUT_FLOATING;
```

**O que isto fixa:**
- Painéis de propriedade declaram que **não flutuam** ⇒ nunca chegam perto de uma viewport nem
  de uma régua. É a cura da foto 2, e é um `Constraint` (o gesto errado torna-se inexprimível),
  não uma verificação.
- O que faz sentido solto (paleta de cor, selector, popover) continua solto — **por declaração**.
- A posição passa a ser **enumerada**, não contínua.

**O que já temos e encaixa:**
- `PanelLayout::Sidebar` existe, funciona e tem teste (`theme.rs:54`).
- ⛔ **Mas está preso ao TEMA.** A primeira obra desta decisão é **separar layout de paleta** —
  hoje só se têm painéis ancorados aceitando o azul claro do `blueprint`.

⚠️ **Consequência obrigatória, no MESMO trabalho:** a **fuga do gizmo de navegação**
(`panel_ops::panel_rects`, o gizmo que se desloca para escapar à moldura — `CLAUDE.md` §5,
3D Modeling) **tem de ser removida**. Ela é o remédio do sintoma; com os painéis fora da vista
ela passaria a fugir de uma moldura que já não a alcança — remédio duplo.

⚠️ **E ancorar NÃO reduz por si só os 51 % de canvas coberto**
([`medicoes/02`](medicoes/02_a_area_tapada.md)): um dock ocupa o mesmo que um flutuante. O que
reduz é colapsar, empilhar por abas, ou ter menos conteúdo lá dentro — que é a **D2**.

---

## D2 — Comandos: **os dois** — barra global **e** cabeçalho por área

> Barra global para o que é do aplicativo inteiro (Arquivo, Editar, Ajuda);
> cabeçalho por área para o que é da ferramenta.

É o que o Godot faz na prática, e é o desenho mais completo dos três oferecidos.

**O que isto fixa:**
- Existe **um** sítio canónico para cada comando ⇒ o painel deixa de ser o depósito por omissão.
  É a cura da foto 3.
- O corte é **por âmbito**: se o comando vale em todo o app, vai à barra; se vale só naquele
  editor, vai ao cabeçalho dele.

**A tabela de destino, aplicada ao painel medido (`3D Model`, 74 entradas):**

| hoje no painel | nº | vai para |
|---|---:|---|
| `export.*` (Export Draft/Fine/Max) | 3 | **barra global → Arquivo** |
| `view.*` + `camera.*` + `frame.*` | 11 | **cabeçalho da área do canvas 3D** |
| `add.*` (paleta de formas) | 20 | **cabeçalho da área → menu Adicionar** |
| `kind/act/verb/op/mode` | 17 | **cabeçalho da área → pulldowns** |
| `mod.*` (modificadores) | 8 | ✅ **fica no painel** — é propriedade do objecto |
| leituras/estado/título | 15 | fica |

⇒ **8 de 74 entradas pertencem a um painel de propriedades.** As outras 66 têm outro dono.

**O que já temos e encaixa:** **148 itens `CTX_MENU_*`** e **40 handlers de chrome**, incluindo
um ficheiro cujo doc-comment é *"os itens do menu Ficheiro"* (`chrome/io_menu.rs`).
⭐ **O trabalho é de realojamento, não de construção.**

⚠️ **O preço que o Enio aceitou:** a barra global come uma faixa de altura permanente. No alvo
iPad (1024 pontos de altura) isso é caro, e soma-se aos 51 % de largura já medidos.

---

## D3 — **LAYOUTS por tarefa** · e MODOS são per-objecto (⚠️ **CORRIGIDA**)

> **Correcção do Enio, 2026-08-30:** *"Aí como no Blender há duas coisas: Layout e Mode. Alguns
> objetos têm modo de edição próprio como vector, cujas tools são completamente específicas e onde
> toda a tela vai mudar. Já Editor 2D, Editor de texto, Runtime, são layouts."*

⛔ **A minha pergunta original tratava Modo e Layout como a mesma coisa, e estava errada.** Os
manuais confirmam o Enio, e há um **terceiro** eixo que os dois motores separam e nós não. O
estudo completo, com as citações, está em
[`pesquisa/04_modo_layout_e_ferramenta.md`](pesquisa/04_modo_layout_e_ferramenta.md).

### Os três eixos

| eixo | quem decide | onde vive | o que muda |
|---|---|---|---|
| **Layout** | o **utilizador** | barra de cima (abas) | que **áreas** existem e que editor está em cada |
| **Modo** | ⭐ o **TIPO DO OBJECTO** seleccionado | cabeçalho da **área** | ferramentas, atalhos, aspecto da vista, e **que outros editores funcionam** |
| **Ferramenta** | o utilizador, dentro do modo | **toolbar** da área | o **gesto** do ponteiro |

**O que a decisão fixa:**
- **A escolha «por tarefa» vale, e é sobre LAYOUTS** — poucos e largos, escolhidos pelo
  utilizador, abas na barra de cima. *Editor 2D · Editor de Texto · Runtime · …* (exemplos dele).
- ⛔ **Modos NÃO são uma lista global e não se escolhem.** Cada **tipo de objecto** declara os
  seus. Blender: *"Which modes are available depends on the object's type."* Só o modo **Object**
  é universal.
- ⭐⭐ **A costura entre os dois é UM CAMPO OPCIONAL** — o Workspace do Blender tem
  `Mode: "switch to this Mode when activating the workspace"`. Ortogonais, com um atalho
  declarado; **não acoplados**.

⚠️ **E a mesma confusão está no nosso código, medida:** o `DrawMode` do vetor tem 14 variantes que
são, lidas pelos nossos próprios doc-comments, **2 modos (`Select`=Object, `Node`=Edit) + 12
ferramentas** — todas as doze justificam-se por *"o gesto é outro"*, que é a definição de
ferramenta, não de modo. ⇒ hoje **não se consegue exprimir «Edit + ferramenta Fillet»**.

⚠️ Os **29 pills** ordenam-se assim: ~19 **ferramentas** · 2 **layout** (mostrar Hierarquia /
Inspetor → menu *Ver*) · 3 ⛔ **uma preferência** (tamanho do botão → Preferências).

⚠️ E os **9 toggles de módulo** são a coisa mais próxima de Layouts que temos — mas são
interruptores **independentes** (2⁹ = 512 combinações), não um selector *um-de-N*.

⏳ **Fica por decidir:** a lista de Layouts · que modos declara cada tipo de objecto nosso · se
adoptamos o campo `Mode` do Workspace · e **como partir o `DrawMode` nos dois eixos sem partir o
que funciona** (14 variantes vivas, com gates — não desenhado).

---

## D4 — Áreas: **encaixes FIXOS**, como o Godot

> *"Lugares pré-definidos. O artista escolhe QUAL painel vai em cada lugar, e arrasta a
> divisória — mas não inventa lugares novos."*

**A alternativa recusada:** a divisão livre do Blender (qualquer área corta-se em duas, sem
limite). ⛔ **Recusada com motivo, não por preguiça:** é muito mais código, o artista consegue
produzir uma tela que não sabe desfazer, e no iPad arrastar divisórias finas com o dedo é mau.

**O que isto fixa:**
- A posição de um painel é **um valor de um conjunto finito**. É a forma mais forte do
  `Constraint`: o erro não é detectado, é **inexprimível**.
- Vários painéis no mesmo encaixe viram **abas** — que é como um encaixe absorve crescimento sem
  crescer.
- ⭐ E torna o layout **serializável de forma trivial**: um layout é `{encaixe → [painéis], posição
  das divisórias}`. O Godot chama à chave `layout_key` (`editor_dock.h:77`).

⚠️ **Isto resolve a tensão que o §6 do diagnóstico nomeava** («extrema simplicidade» contra
«altíssima capacidade de ajustes»): o ajuste que fica é **o que**, não **onde**.

---

## D5 — A régua entra na **ÁREA DE DESENHO**

> *"A régua deixa de ser da janela e passa a ser da área do canvas — começa depois do trilho, não
> por baixo dele."*

É o modelo do Blender: a régua é uma **region** do editor, não do ecrã.

**O que isto cura, e é medido** ([`medicoes/02`](medicoes/02_a_area_tapada.md)): hoje
`left_band = (canvas.x, canvas.y, 20, canvas.h)` com `canvas = (0,0,w,h)`, e o rail também
começa em `x = 0` ⇒ **87,8 % da régua da esquerda por baixo do rail**. Com a régua dentro da
área, ela começa **depois** do rail e a sobreposição é **estruturalmente zero**.

**As alternativas recusadas:**
- *Empurrar o rail 20 px* — ⛔ custa mais 20 px de largura numa tela onde 51 % já é moldura, e
  **não cura a régua de cima**, que continua sob a barra superior.
- *Régua ligável/desligável* — ⛔ quem desenha com medida deixa-a ligada sempre, e para essa
  pessoa nada muda.

⭐⭐ **E D5 generaliza para o RAIL:** se a régua é uma região da área, o trilho de ferramentas
também é (Blender: *Toolbar*, região esquerda do editor; Godot: barra da própria viewport). ⇒ os
dois passam a ser **irmãos numa fila**, não **camadas empilhadas** — e irmãos não se tapam. *A
ordem de pintura deixa de ser a resposta, porque deixa de haver sobreposição.*

---

## O que estas cinco decisões implicam, junto

⭐⭐ **As três convergem na MESMA peça em falta: um modelo de REGIÃO / ÁREA.**

- **D1** precisa de slots onde um dock possa estar (Godot: 12 slots enumerados).
- **D2** precisa de um cabeçalho **por área** — logo, precisa que áreas existam.
- **D3** precisa que um modo possa dizer *"esta área tem este editor"*.

⇒ **A primeira obra não é nenhuma das três: é o modelo de áreas.** Sem ele, D1 vira painéis
ancorados sem sítio, D2 vira cabeçalhos sem dono e D3 vira um selector que não sabe o que
arrumar.

⭐ **E a D4 + D5 dizem-lhe a forma:** áreas com **encaixes enumerados** (D4), e dentro de cada
área **regiões em fila** — cabeçalho, ferramentas, régua, conteúdo (D5). O rascunho está em
[`spec/01_modelo_de_areas.md`](spec/01_modelo_de_areas.md).

⛔ **E a ordem tem uma trava dura**
([`medicoes/03 §5`](medicoes/03_o_censo_de_cor.md)): reduzir os temas de 4 para 2 **antes** de
separar layout de paleta **apaga o único modo ancorado que o app tem** (o `blueprint`). A paleta
mexe-se **depois**.
