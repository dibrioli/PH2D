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

## D3 — Modos: **por TAREFA**, como o Blender

> Desenhar, Modelar, Esculpir, Animar, Programar — não Vetor/Movimento/Flip/Escultura.

**O que isto fixa:**
- Um Modo **é um layout de áreas**, não um interruptor de módulo. É a leitura literal do HIG do
  Blender: *"editors are own modes with own shortcuts and tools — almost like different
  applications"* (`editors.md`).
- A **mesma ferramenta aparece em várias tarefas** — é isso que impede a explosão que nos deu
  **29 pills**.
- ⭐ Casa com a frase do próprio Enio na abertura: *"teremos uma enorme quantidade de tools que
  são verdadeiros apps completos aninhados."*

⚠️ **Consequência:** o rail de 29 pills deixa de ser o selector de modo. O que lá fica (se ficar)
é outra coisa — ferramentas **dentro** da tarefa activa.

⚠️ **E há um custo escondido, nomeado agora:** um módulo hoje é ligado/desligado por um toggle
próprio (8 `*_toggle.rs` em `chrome/`). Se o modo passa a ser tarefa, **quem decide que módulos
estão vivos é o layout da tarefa** — os 8 toggles mudam de dono ou desaparecem. Não medido.

---

## O que estas três decisões implicam, junto

⭐⭐ **As três convergem na MESMA peça em falta: um modelo de REGIÃO / ÁREA.**

- **D1** precisa de slots onde um dock possa estar (Godot: 12 slots enumerados).
- **D2** precisa de um cabeçalho **por área** — logo, precisa que áreas existam.
- **D3** precisa que um modo possa dizer *"esta área tem este editor"*.

⇒ **A primeira obra não é nenhuma das três: é o modelo de áreas.** Sem ele, D1 vira painéis
ancorados sem sítio, D2 vira cabeçalhos sem dono e D3 vira um selector que não sabe o que
arrumar.

⛔ **E a ordem tem uma trava dura**
([`medicoes/03 §5`](medicoes/03_o_censo_de_cor.md)): reduzir os temas de 4 para 2 **antes** de
separar layout de paleta **apaga o único modo ancorado que o app tem** (o `blueprint`). A paleta
mexe-se **depois**.
