# Bugs do módulo Motion Nodes — registro + soluções

> Log vivo dos bugs **não-triviais** do sistema de nós (sintoma → causa-raiz → hipóteses que
> caíram → medição → solução → lições). O objetivo não é listar todo fix (o git já faz isso),
> mas registrar os bugs cuja **causa enganava** — aqueles em que a aparência, ou o nó que o
> report acusou, levou o diagnóstico para a pista errada. Cada entrada termina em **lições
> generalizáveis**, para o próximo agente não repetir o erro.
>
> Estado por-wave: os handoffs em [`handoffs/`](handoffs/). O catálogo por família:
> [`89_conferencia/`](89_conferencia/).

| # | Bug | Área | Estado | Data |
|---|---|---|---|---|
| [1](#bug-1--o-nó-acusado-estava-inocente-o-campo-gateia-o-pulso-e-não-gateia-a-memória) | **"Box inconsistente"** — marcar Invert e desmarcar não devolve o quadro inicial | `field.box` (acusado, **inocente**) + `pulse.counter` (a memória) | ⚠️ Mecanismo FECHADO · cura é decisão de produto | 2026-08-10 |

---

## Bug #1 — O nó acusado estava INOCENTE: o campo gateia o pulso, e não gateia a MEMÓRIA

**Estado:** mecanismo fechado e medido em 2026-08-10, com gate executável
(`the_field_gates_the_pulse_but_not_the_counters_memory`). ⚠️ **A cura não foi construída** —
ela é uma escolha entre dois P1 abertos da folha 12, e é decisão do Enio.

### Sintoma

Report do Enio na cena `PH2D_GPU_COOK_DEMO=23` (O PORTÃO ESPACIAL):

> *"Nó Box inconsistente! Ao checar Invert e depois desmarcar, o resultado é diferente do
> inicial."*

Na tela: o losango pisca; marcar **Invert** e desmarcar devolve a máscara, mas a arte fica
**invertida** — a área de FORA acesa e parada, o miolo apagado. Lido do assento do artista, o
`field.box` "esquece" o estado dele.

### O que a medição disse (e ela derrubou o próprio report)

Sonda `probe_invert_round_trip`, no caminho REAL (o `set_param` que o checkbox emite + o `Cook`
que a cena usa), 262.144 linhas:

| medição | resultado |
|---|---|
| **M1 — o NÓ**: `invert` 0 → 1 → 0 | `invert` mudou **262.144** linhas · o ida-e-volta difere em **0** |
| **M2 — a CENA** no tique 120 | difere em **262.144 (100%)** |
| **M3 — o QUE difere** (sem toggle) | DENTRO **33.540/33.540** grandes · FORA **0**/228.604 |
| **M3 — o QUE difere** (pós ida-e-volta) | DENTRO **0**/33.540 · FORA **228.604/228.604** grandes |

⚠️ **O `field.box` é uma função PURA dos params** — o ida-e-volta re-deriva a máscara **byte a
byte**, em todas as linhas. E só existem dois tamanhos na cena (`0.0180` = repouso, `0.0480` =
crescido): é **paridade limpa**, não corrupção. O retrato pós-ida-e-volta é o **inverso EXATO**
do inicial.

### A causa-raiz

O que muda de lugar não é a máscara: é o **`count_tick` do `pulse.counter`**, que vive no `pre`
self-loop. Enquanto o campo está invertido, quem está **FORA** recebe as batidas do metrônomo e
avança a paridade; quem está **DENTRO** congela. Desmarcar devolve a máscara e **não** devolve a
memória.

A aritmética prevê a medição sem folga: a janela do toggle contém **uma** batida (`period` 0,5 s),
então cada lado fica exatamente **um** pulso fora de fase — e um contador `count_max = 2, Wrap` é
paridade, logo um pulso de diferença é a inversão completa.

⚠️ **E a informação que faltaria para consertar é DESTRUÍDA antes do contador:** o portão é um
`value.math(Multiply)`, e ele colapsa *"não há pulso agora"* e *"esta linha saiu do campo"* no
**mesmo zero**. O `pulse.counter` não tem como distinguir os dois — e não tem porta de **RESET**
(`inputs` = `pulse` + `state`, o self-loop). *O campo consegue gatear um EVENTO; ele não tem como
gatear ESTADO.*

### Por que nenhum gate viu (a pergunta 4 do protocolo)

`the_scene_blinks_only_inside_the_box` cozinha de `t = 0` com os params **FIXOS**, e o irmão
`the_gate_is_the_pulse_not_the_drives_own_mask` cozinha **um** quadro. ⚠️ **Nenhum gate deste
repositório cozinha uma cena ATRAVÉS de uma edição de param** — a classe inteira *"autorar sobre
um grafo vivo com estado"* estava sem cobertura. Os gates provam o comportamento **inicial** e
são estruturalmente cegos ao **gesto**.

### O que foi corrigido agora

1. **O doc do `field.box` MENTIA em duas frases** (e um doc que mente é parte do bug): ele abria
   com *"an **axis-aligned** rectangle"* e a lista de params **omitia o `rotation`** — num nó que
   tem o param, tem gate de rotação, e cuja rotação de 45° é o que faz o **losango** desta cena.
2. **A prosa da cena dizia *"Fora dela nada acontece, NUNCA"***. O "nunca" é falso depois de uma
   edição de campo — corrigido para nomear a condição.
3. **O gate executável** `the_field_gates_the_pulse_but_not_the_counters_memory`, com as duas
   metades: a que **inocenta** o nó (pura, e `invert` morde todas as linhas — sem essa segunda
   metade o gate ficaria verde sobre um memo que ignorasse o param) e a que **mede** a inversão.
   Prova de mutação: um `field.box` impuro (o flip guardado em vez de derivado) faz a metade da
   pureza sangrar em **262.144** linhas.

### O que segue ABERTO (decisão de produto)

Duas curas, as duas já na fila da folha 12 §3.2 — e este report é a **evidência de campo** que as
ordena:

- **`pulse.counter` ganha entrada de RESET.** O artista liga o complemento do campo ali, e sair
  do campo limpa o contador. Compõe, é explícito, e custa uma porta nova (default desconectado =
  `Empty` = o mundo de hoje). ⚠️ A cerca que a deferia tem **premissa falsa** (o `Graph::validate`
  itera ARESTAS e não recusa input faltante — dois nós que shipam já dependem disso).
- **`pulse.adsr`.** Um envelope **volta ao repouso sozinho**: uma linha que sai do campo decai e
  para, sem o artista fiar nada. É a cura auto-curável, e provavelmente a certa *para esta cena* —
  o `pulse.counter(2, Wrap)` foi escolhido por legibilidade (o toggle segura meio período), e o
  preço dessa escolha é justamente um estado que nunca volta.

⚠️ **O gate novo pina um defeito ABERTO, de propósito** — ele é o número dele. Quando a cura
landar, a metade da inversão fica vermelha: **reescreva-a para a lei nova, não a afrouxe.**

### Lições generalizáveis

1. **O nó que o report acusa é uma hipótese, não um diagnóstico.** A primeira medição tem de ser
   a que pode **inocentá-lo** — aqui, cozinhar o nó sozinho no ida-e-volta (0 de 262.144).
2. **Um gate de pureza precisa da metade irmã.** *"O ida-e-volta restaura"* é satisfeito por um
   nó que **não faz nada**; sem *"e o param MORDE todas as linhas"*, o gate passaria sobre um memo
   que ignorasse a edição.
3. **Compor um campo com um acumulador é compor duas coisas de tempos de vida diferentes.** A
   máscara é função do agora; o contador é função da história. Toda vez que um multiplicador
   gateia um evento a montante de um estado, *sair do gate* vira um evento que ninguém observa.
4. **Gate que cozinha de `t = 0` não testa AUTORIA.** Uma cena com estado tem dois
   comportamentos — o do boot e o do gesto — e o segundo custa um gate próprio.
