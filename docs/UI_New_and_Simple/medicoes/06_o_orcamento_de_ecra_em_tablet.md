# O orçamento de ecrã em TABLET (2026-08-31)

> Enio, 2026-08-31: *«Lembre-se que esse app tem tablets e iPad como alvo. Não podemos ir perdendo
> espaço. Desfaça isso. Veja nos planos se há mais motivos de perder espaço e ajuste.»*
>
> ⛔ **A resposta a *«quanto custa esta faixa?»* deixou de ser uma opinião.** Os números abaixo saem
> do **produto**, pelas mesmas funções que o `hero::frame_layout` usa, e vivem num gate:
> `crates/ph2d-editor-core/tests/the_chrome_never_eats_more_of_a_tablet_than_this.rs`.

---

## §1 — Os três alvos, e por que são três

O `tokens.json` declara **um**: `1366 × 1024` (iPad Pro 12,9"). Ele é o mais **generoso** dos
tablets que o Enio nomeia — e é por isso que medir só nele esconde o problema.

| alvo | pontos lógicos |
|---|---|
| iPad Pro 12,9" | 1366 × 1024 |
| iPad Pro 11" | 1194 × 834 |
| iPad mini | 1133 × 744 |

⚠️ **A largura do chrome NÃO escala com o ecrã** — as duas colunas são `308 + 304 = 612 px`
absolutos. ⇒ elas são **44,8 %** da largura no 12,9" e **54,0 %** no mini. *A mesma decisão de
desenho custa 20 % mais no aparelho mais pequeno, e nenhum documento dizia isso.*

---

## §2 — A medição

Área de **desenho** como percentagem da janela, com a barra de menus e a fila de ferramentas
presentes (o chrome de produção):

| alvo | colunas abertas | colunas abertas **a pintar** | colunas fechadas |
|---|---:|---:|---:|
| iPad 12.9 | 50,8 % | 50,8 % | **92,0 %** |
| iPad 11 | 44,0 % | ~~40,8 %~~ → **44,0 %** | 90,2 % |
| iPad mini | 40,9 % | ~~37,6 %~~ → **40,9 %** | 89,0 % |

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

⚠️ **E hoje isso custa dois gestos de menu** (*View → Hierarchy*, *View → Inspector*), um por
coluna. Não há um gesto de *recolher*. ⏳ É a alavanca com melhor razão custo/benefício que esta
medição encontra, e é **decisão de produto** (que gesto, que tecla, se as colunas voltam sozinhas).

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
