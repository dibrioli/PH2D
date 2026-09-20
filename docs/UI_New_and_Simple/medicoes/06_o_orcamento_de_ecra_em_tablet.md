# O orçamento de ecrã em TABLET (2026-08-31)

> Enio, 2026-08-31: *«Lembre-se que esse app tem tablets e iPad como alvo. Não podemos ir perdendo
> espaço. Desfaça isso. Veja nos planos se há mais motivos de perder espaço e ajuste.»*
>
> ⛔ **A resposta a *«quanto custa esta faixa?»* deixou de ser uma opinião.** Os números abaixo saem
> do **produto**, pelas mesmas funções que o `hero::frame_layout` usa, e vivem num gate:
> `crates/ph2d-editor-core/tests/it/the_chrome_never_eats_more_of_a_tablet_than_this.rs`.

---

## §1 — Os três alvos, e por que são três

O `tokens.json` declara **um**: `1366 × 1024` (iPad Pro 12,9"). Ele é o mais **generoso** dos
tablets que o Enio nomeia — e é por isso que medir só nele esconde o problema.

| alvo | pontos lógicos |
|---|---|
| iPad Pro 12,9" | 1366 × 1024 |
| iPad Pro 11" | 1194 × 834 |
| iPad mini | 1133 × 744 |

⚠️ **A largura do chrome NÃO escalava com o ecrã** — as duas colunas eram `308 + 304 = 612 px`
absolutos. ⇒ elas eram **44,8 %** da largura no 12,9" e **54,0 %** no mini. *A mesma decisão de
desenho custava 20 % mais no aparelho mais pequeno, e nenhum documento dizia isso.*

### ✅ CURADO em 2026-09-20 (ordem do dono: *«item 2»*)

A largura de **fábrica** de uma coluna passou a ser um **tecto em fracção da janela** —
[`ChromeBands::default_dock_w`](../../../crates/ph2d-editor-core/src/screens/dock_seam.rs). A
fracção é **derivada** e não escolhida: `HIERARCHY_W / HERO_VIEWPORT_W`, dois tokens que já
existiam, um a dividir pela janela para que foi autorado.

| alvo | as duas colunas, antes | depois |
|---|---:|---:|
| iPad 12,9" | `44,8 %` | `44,8 %` — **a referência, intocada ao bit** |
| iPad 11" | `51,3 %` | **`44,8 %`** |
| iPad mini | `54,0 %` | **`44,8 %`** |

⛔⛔ **É um TECTO e nunca uma ESCALA, e a diferença tem número:** escalar nos dois sentidos poria as
colunas em `1930 × 308/1366 = 435 px` cada na janela do dono — **`870`** contra `612`. *A cura
tornaria o app pior exactamente onde ele é usado todos os dias.* ⇒ acima da referência a lei é
inerte, com gate a exigi-lo.

⚠️ **Ela não toca na largura que o ARTISTA arrastou** — apertar uma escolha explícita é o *«aceita
e mente»* do §0.0. ⏳ Fica declarado: uma escolha gravada num ecrã largo continua a valer o que vale
num estreito.

---

## §2 — A medição

Área de **desenho** como percentagem da janela, com a barra de menus e a fila de ferramentas
presentes (o chrome de produção):

| alvo | colunas abertas | colunas abertas **a pintar** | colunas fechadas |
|---|---:|---:|---:|
| iPad 12.9 | ~~50,8 %~~ → **50,6 %** | idem | **91,8 %** |
| iPad 11 | ~~44,0 %~~ → **49,6 %** | idem | 90,0 % |
| iPad mini | ~~40,9 %~~ → **48,9 %** | idem | 88,8 % |

⭐ **Os números de 2026-09-20 estão na coluna da direita de cada célula**, e quem os mandou
actualizar foi a **metade da obsolescência** do gate, não eu: `+5,6` pontos no iPad 11 e **`+8,0`**
no mini. ⚠️ **E o `50,6` do 12,9" NÃO é desta wave** — o piso desceu `0,2` pontos em 2026-09-07 (a
divisória de `4 px`) e esta tabela ficou para trás; *quando um doc imprime duas medidas da mesma
grandeza e elas discordam, isso É o achado.*

⇒ **no iPad mini, a pintar, com os dois painéis abertos, o artista desenhava em 37,6 % do ecrã.**

⭐⭐ **CURADO em 2026-08-31 (entrega 32):** a coluna «a pintar» deixou de ser pior — `+3,2` pontos
no iPad 11 e `+3,3` no mini. Ver o §3.

---

## §3 — ⛔⛔ O achado: a fila de ferramentas DOBRA nos dois tablets menores

| alvo | fila sem pincel | fila **com** pincel |
|---|---:|---:|
| iPad 12.9 | 54 px (1 linha) | 54 px (1 linha) |
| iPad 11 | 54 px | **108 px (2 linhas)** |
| iPad mini | 54 px | **108 px (2 linhas)** |

A fila tem 10 entradas em repouso e **18** com o Painter em mãos. Nos dois tablets menores a área
entre as colunas (`582` e `521 px`) não as segura numa linha, e a faixa **cresce**.

⚠️ Custo: `−3,2` pontos percentuais no iPad 11 e `−3,3` no mini — **enquanto se pinta**, que é
precisamente quando o ecrã faz falta.

⭐ **A cura ficou DECIDIDA pela restrição, e não era antes.** O handoff §7 registava duas saídas —
*«quebrar em duas linhas (a faixa cresce) ou um menu de transbordo»* — sem critério para escolher.
O alvo tablet escolheu: **a faixa não cresce; o excesso vai para um controlo de transbordo.**
⛔ Encolher o chip ficou fora: ele mente sobre o preset de tamanho que o artista escolheu.

### ✅ FEITO (entrega 32)

A faixa é **sempre uma linha**; o que não cabe vive atrás do `⋯` (`tool_bar::bar_split`, a porta
única que o pintor, o registo de hit e o menu leem).

| alvo | a pintar: cabem | atrás do `⋯` |
|---|---:|---:|
| iPad 12.9 | 18 | 0 |
| iPad 11 | 13 | **5** |
| iPad mini | 12 | **7** |

⇒ `+3,2` pontos de área no iPad 11 e `+3,3` no mini, **enquanto se pinta**.

---

## §4 — ⭐⭐ A maior alavanca não é cortar chrome: é RECOLHER

Fechar as duas colunas devolve **89 a 92 %** do ecrã em qualquer dos três. É mais do que todas as
faixas de chrome somadas valem.

### ✅ FEITO (2026-09-07) — e o gesto é o do Blender

⚠️ **Até aqui isso custava dois gestos de menu** (*View → Hierarchy*, *View → Inspector*), um por
coluna, num aparelho sem teclado.

⭐ **Hoje é um gesto: arrastar a borda da coluna para dentro, para além do mínimo, fecha-a**; uma
alça pintada na margem trá-la de volta. O dono delegou a decisão (*«faça o que achar melhor
buscando o estado da arte»*), e a escolha tem três razões medidas:

1. o artista **já arrasta esta borda** — a costura shipou em 2026-08-30, e até aqui apenas travava
   no mínimo;
2. funciona **sem teclado**, que é a condição no alvo. O `Ctrl+Space` do Blender e o modo sem
   distracções do Godot (`Ctrl+Shift+F11`) resolvem o mesmo problema **com uma tecla**, e uma tecla
   não existe num tablet;
3. **não custa chrome permanente**: a alça só existe enquanto a coluna está fechada, e nesse estado
   troca `304–308 px` por `6`.

⚠️ **A alça é PINTADA, ao contrário da costura** — e essa assimetria é a wave inteira num detalhe: a
costura vive do **cursor**, e num ecrã de toque não há cursor; o que a torna descobrível é a borda
visível da coluna. Fechada a coluna, essa borda desaparece, e sem alça o caminho de volta seria
outra vez o menu.

⚠️ O degrau de fecho está **uma linha abaixo do mínimo** (`ROW_H_PX`): chegar ao mínimo é um
objectivo legítimo, logo tocar-lhe não pode fechar nada — e `22 px` é dez vezes o tremor de um
toque. As duas leis do degrau são **erro de compilação**, não teste.

⭐ E o fecho **sobrevive ao reinício**: a visibilidade de um painel já entrava na arrumação gravada.

---

## §5 — ⛔ O que esta medição RECUSOU

**O cabeçalho por área** (D2, metade 2) foi construído e **revertido no mesmo dia** — entrega 30,
revertida na 31, a pedido do Enio. Ele custava `ROW_H_PX + 2·Xxs` = **28 px** de altura permanente:

| alvo | com o cabeçalho | sem ele |
|---|---:|---:|
| iPad 12.9 | 49,3 % | 50,8 % |
| iPad mini | ~36,3 % | 37,6 % |

⇒ **−1,5 ponto no alvo declarado**, para dar casa a dois interruptores. *Uma faixa permanente tem
de devolver mais do que consome, e esta não devolvia.*

⚠️ **A D2 continua certa sobre o ÂMBITO** (um comando do editor não pertence a um menu do app) — o
que a medição recusa é a **faixa própria**. Se a metade 2 voltar, ela tem de caber onde já se paga
altura: a fila de ferramentas, ou um popover do lado direito dela.

---

## §6 — A catraca, e o censo dela

O gate mede as seis células do §2 (colunas abertas, com e sem pincel) e reprova quando a área
**desce**. ⚠️ Ele traz também o **tecto**: uma célula que suba mais de `2` pontos acima do piso
reprova como **obsoleta**, porque nesse dia a barra deixou de defender o que mede.
*Uma catraca sem censo de obsolescência não desce: ela vira licença.*

⛔ **Provado por mutação:** repor uma faixa permanente de 28 px derruba-o (`49,3 %` contra o piso
`50,8 %`).
