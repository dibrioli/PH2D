# Plano — o GRAFO da booleana viva (a "máquina de estados" das booleanas)

> `line/Vector`, 2026-08-22. Nasce de um pedido do Enio:
> *"Avalie a possibilidade de um tipo de 'State Machine' para operações booleanas em tempo real
> (Live). Abre-se numa janela um círculo representante de cada forma e liga-se uma forma a outra
> com linhas e nessas linhas escolhemos o modo de interação de uma forma com a outra de modo que a
> mesma forma pode usar Union para uma forma e Subtract para outra forma."*
>
> **As duas etapas estão FECHADAS** — o modelo (§1-§6) e a janela (§7). Este doc é o roteador delas.

## 1. A ideia, e por que ela não é nova

O documento de inovações do módulo ([14 §14.2](14_inovacoes_extraordinarias.md)) já a descrevia
com nome — **Live Boolean Graph**, inovação nº 1 — e a justificava assim: o Illustrator faz a
operação e **descarta** os operandos; o Figma também; o Affinity só faz booleanas.

⚠️ Aquele plano é da versão **pré-cutover** do Vector (ADR-0108 descartou as 30 crates antigas). A
ideia sobrevive; o mecanismo que ele previa — *"nó vivo no `ph2d-nodegraph` domain `vector`"* —
**não**, e é bom que não: o [ADR-0132](../architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md)
mediu e **rejeitou** o grafo de nós para efeitos vetoriais.

⚠️ **E isto não o contradiz.** Aquele ADR fala de efeitos que agem sobre **UMA** forma (offset,
roughen, largura viva) — e separa explicitamente os **objetos relacionais**, que ligam várias
(Blend, Envelope, Booleana), como *"outra família, deliberadamente fora deste ADR"*. A booleana é
exatamente essa família.

## 2. O que o grafo acrescenta — e o que NÃO acrescenta

⚠️ **Não é capacidade nova.** *"Union com esta, Subtract daquela"* **já é exprimível hoje**,
aninhando grupos booleanos dentro de grupos: o `bool_live` resolve do mais interno para fora, e há
gate (`a_nested_boolean_cooks_from_the_inside_out`).

O que ele acrescenta é **EXPRESSÃO**: ver e reorganizar as relações num lugar, em vez de as inferir
de uma árvore de grupos aninhados. Isso é uma justificação **mais forte**, não mais fraca — o motor
já prova que a semântica funciona, e a janela passa a ser interface sobre algo que já roda.

## 3. A lei, em cinco frases

Fonte: [`crates/ph2d-vec-boolean/src/graph.rs`](../../crates/ph2d-vec-boolean/src/graph.rs).

1. **A seta É a ordem do fold** — `from` OPERA, `to` RECEBE. Ela resolve a assimetria do Subtract
   (`A−B ≠ B−A`): círculos ligados por linhas **sem** direção desenhariam duas coisas diferentes
   com a mesma aparência.
2. **Várias ligações a chegar no mesmo nó dobram na ordem de z do `from`** (fundo → topo). A régua
   é a MESMA do `apply_many`, e é o que faz a lista de camadas continuar a explicar o resultado.
3. **Só as quatro operações de CONJUNTO valem numa ligação.** ⛔ As quatro receitas
   (`MinusBack`/`Trim`/`Crop`/`Merge`) são afirmações sobre uma PILHA — *"cada forma menos a união
   do que está acima dela"* não é uma relação entre DOIS, e escrevê-la numa seta seria prometer o
   que o modelo não entrega. Elas continuam a ser do grupo.
4. **Ciclo é RECUSA inteira** — o grafo não cozinha e a arte fica como estava. Nunca uma recusa
   parcial: desenhar o pedaço acíclico mostraria arte que nenhuma leitura do diagrama explica.
5. **Nó consumido desenha VAZIO; sumidouro desenha no PRÓPRIO id.**

### 3.1 A distinção que o gesto exige

⚠️ **`VecBoolEdges` ausente ≠ `VecBoolEdges` com lista vazia.**

| Estado | Significado |
|---|---|
| componente **ausente** | grupo de sempre: os filhos combinam pela operação única do `VecBoolGroup` |
| componente **presente, lista vazia** | grafo **sem relação nenhuma**: cada forma desenha-se a si própria |

Se a lista vazia caísse de volta na operação única, **cortar o último elo no diagrama faria as
formas FUNDIREM-SE** — o oposto exato do gesto. Gate: `um_grafo_vazio_nao_reinstala_a_operacao_do_grupo`.

## 4. A ESTRELA — o que torna a etapa 2 segura

`derive_star(nodes, op)` escreve o grafo equivalente a um grupo de hoje: **todos apontam para a
base**, com a operação do grupo. O resultado é **o mesmo**, e não por sorte: o `apply_many_checked`
já É um fold binário da esquerda para a direita, e a estrela reproduz esse fold — mesma ordem, mesmo
doador de estilo (o último dobrado, que na estrela é o operando do topo).

Dois gates prendem a igualdade, um em cada camada:

- `a_estrela_derivada_desenha_o_que_o_grupo_de_hoje_desenha` (motor) — `assert_eq!` sobre a
  geometria inteira, para as quatro operações de conjunto.
- `a_estrela_materializada_no_componente_nao_move_a_arte` (shell) — o mapa inteiro, com e sem grafo.

⚠️ **É a licença da etapa 2.** Abrir a janela sobre um grupo existente materializa a estrela; sem
esta prova, a feature de VISUALIZAR alteraria o que se está a visualizar.

⚠️ O oráculo é geometria, **não área**: a promessa não é *"dá o mesmo tamanho"*, é *"não move um
pixel"*, e área é a medida que deixa passar uma forma trocada de sítio.

## 5. Custo MEDIDO

`cargo test -p ph2d-host-desktop --bins measure_a_live_boolean_frame --release -- --ignored --nocapture`
(o `recook` INTEIRO, com o memo invalidado a cada volta — o caso do arrasto, que é o único em que o
custo importa; máquina calma, release):

| operação | operandos | grupo | grafo (estrela) |
|---|---|---|---|
| Union | 2 | 0,055 ms | 0,058 ms |
| Union | 10 | 1,500 ms | **1,924 ms** |
| Subtract | 2 | 0,134 ms | 0,107 ms |
| Subtract | 10 | 0,555 ms | 0,569 ms |
| Intersect | 2 | 0,091 ms | 0,081 ms |
| Intersect | 10 | 0,259 ms | 0,308 ms |

Orçamento de um quadro a 60 fps: **16,6 ms**.

**Leitura:** o grafo empata com o grupo em cinco das seis células (as diferenças de ±0,03 ms são
ruído — em duas ele sai "mais rápido", o que só pode ser ruído). A célula que **não** é ruído é
`Union × 10`: **+28%** (+0,42 ms), e ela é 11,6% de um quadro. O custo mora no motor booleano, não
no grafo — Union cresce porque a forma acumulada vai ficando maior, e Subtract/Intersect quase não
crescem.

⚠️ **E o custo só é pago quando algo MUDA:** o `BoolLive.memo` compara a entrada (operação + grafo +
geometria em mundo) e reaproveita o resultado. Cena parada ⇒ o motor **não roda**.

⛔ **Não há teto que impeça a feature.** Uma rede real com 5–8 formas fica em 1–4% de um quadro.

## 6. Onde a etapa 1 mora

| Peça | Onde |
|---|---|
| O resolvedor + a lei | [`ph2d-vec-boolean/src/graph.rs`](../../crates/ph2d-vec-boolean/src/graph.rs) (`resolve_graph`, `derive_star`, `BoolEdge`, `GraphRefusal`) |
| O que o documento guarda | [`ph2d-ecs/src/vec_bool_edges.rs`](../../crates/ph2d-ecs/src/vec_bool_edges.rs) (`VecBoolEdges`, `VecBoolEdge`) |
| A costura por frame | [`shells/desktop/src/bool_live.rs`](../../shells/desktop/src/bool_live.rs) (`cook`) |
| O Apply | [`shells/desktop/src/bool_gesture.rs`](../../shells/desktop/src/bool_gesture.rs) (`bake`, agora com N sumidouros) |

Números que a etapa 1 moveu: `PROJECT_SCHEMA` **86 → 87** · registo de componentes **58 → 59**.

## 7. Etapa 2 — a janela (FEITA)

O diagrama existe e é operável. Três camadas, e o corte entre elas é o que as torna testáveis:

| camada | onde | o que faz |
|---|---|---|
| **geometria** | [`widget/bool_graph.rs`](../../crates/ph2d-editor-core/src/widget/bool_graph.rs) | onde cada círculo fica, por onde passa cada arco, o que está sob o dedo |
| **card** | [`chrome/bool_graph_modal.rs`](../../crates/ph2d-editor-core/src/screens/hero/chrome/bool_graph_modal.rs) | desenha e publica INTENÇÕES; não muta nada |
| **mundo** | [`bool_graph_ui.rs`](../../shells/desktop/src/bool_graph_ui.rs) + [`bool_graph_input.rs`](../../shells/desktop/src/bool_graph_input.rs) | a vista, a estrela, as intenções escritas, o gesto |

### 7.1 A disposição: uma COLUNA por z, arcos à direita

⚠️ **Não é um anel, e a razão é a lei.** Ligações que chegam ao mesmo nó dobram na ordem de **z** de
quem opera (§3.2), então o diagrama TEM de mostrar z. Um anel espalha os círculos bonito e apaga
exatamente o dado de que a lei depende. A coluna também não inventa convenção: é a leitura da lista
de camadas que o artista já tem, **o mais ao FUNDO em baixo**.

⚠️ Os arcos passam à **direita dos rótulos**, não da coluna: curvar a partir dos círculos riscaria
por cima do nome das formas que a ligação salta — e o nome é como o artista sabe qual círculo é
qual. Foi um **mutante sobrevivente** que expôs isto (o gate do card passava com a reserva de
largura REMOVIDA, porque a folga dos rótulos já continha o arco).

### 7.2 Os gestos

| gesto | o que acontece |
|---|---|
| *Down* num círculo → *Up* noutro | **liga** os dois (`from` opera, `to` recebe) |
| clique numa ligação | **gira** a operação entre as quatro de conjunto |
| **Shift**+clique numa ligação | **corta** a ligação |

⚠️ **A rotação NÃO inclui um estado *"sem ligação"*.** Cortar por sobre-rodar seria o engano mais
fácil do diagrama: quem quer ir de *Union* a *Subtract* e passa do ponto apagaria a ligação.

⚠️ **Uma ligação nova HERDA a operação das existentes** — é o que faz montar uma rede uniforme
custar um arrasto por ligação, em vez de um arrasto mais quatro cliques a girar de volta ao mesmo
verbo.

### 7.3 As leis da costura

- **O card publica o RECT QUE DESENHOU**, e é a porta única do acerto do clique. Recalculá-lo do
  canto pedido repetiria a prisão ao viewport — duas contas que divergem.
- **A vista vem do REGISTO do produtor** (`BoolLive::roster`), não de uma segunda triagem. E o
  registo sobrevive à RECUSA, que é onde ele mais importa: com um ciclo não há plano, e é aí que o
  diagrama tem de mostrar os círculos e dizer o que está errado.
- **O ciclo ganha voz** — no motor ele é recusa silenciosa; no card, uma frase em língua de artista
  (*"A shape ends up feeding itself — remove one link"*), sem a palavra *ciclo*.
- **Os oito botões da seção Boolean agem NO DIAGRAMA.** Com um grafo presente quem manda é a
  operação de cada ligação; mexer só no `VecBoolGroup` deixaria o artista a clicar *Subtract* e a
  ver a arte não mudar — o defeito *"parâmetro que não muda nada"* na sua forma mais pura. Uma das
  quatro de conjunto reescreve TODAS as ligações; uma das quatro **receitas remove o grafo** (ela é
  uma afirmação sobre a pilha inteira e não tem tradução em pares).
- **O painel mostra o verbo do diagrama, ou *Mixed***, para o artista SABER isso antes de clicar.
- **Apagar uma forma leva as ligações dela** (`prune_dead_edges`), e a varredura **só escreve quando
  de facto apaga** — o undo é por diff de bytes, e reescrever todo frame criaria um passo por frame.

### 7.4 O que a janela ainda NÃO faz

- ⏸️ **A linha elástica durante o arrasto** — o gesto decide-se no *Up*, e entre o *Down* e ele nada
  segue o cursor. É a única coisa que o gesto não mostra, e fica registada em vez de silenciada.
- ⏸️ **Pan/zoom do diagrama.** Com muitas formas a coluna cresce para além do card; ele prende-se ao
  viewport, então as linhas de baixo saem da vista. ⛔ Antes de construir, MEÇA quantas formas um
  grupo real tem — a composição pode já resolver (grupos aninhados).
- ⏸️ **Reordenar z pelo diagrama.** Arrastar um círculo para cima mudaria a pilha — é dizível, e não
  foi feito: o gesto já significa *ligar*.

---

## ⛔ Recusas MEDIDAS

| O quê | Por quê | Onde |
|---|---|---|
| Grafo de nós para efeitos de UMA forma | Medido e rejeitado; a pilha por-path venceu | [ADR-0132](../architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md) |
| Receita (`Trim`/`Crop`/`Merge`/`MinusBack`) numa ligação | Não é relação entre DOIS; é afirmação sobre a pilha inteira | §3.3, gate `uma_receita_de_pilha_nao_e_uma_ligacao` |
| Recusa PARCIAL num ciclo | Desenharia arte que nenhuma leitura do diagrama explica | §3.4, gate `um_ciclo_num_canto_recusa_o_grafo_inteiro` |
| Lista vazia == componente ausente | Cortar o último elo faria as formas fundirem-se | §3.1, gate `um_grafo_vazio_nao_reinstala_a_operacao_do_grupo` |
| Reusar o motor de nós do Motion | ~11 k linhas amarradas ao `ph2d-nodegraph`, para uma lista de três `u64` | §7.1 |
| Um ANEL de círculos | Apaga o z, e é o z que decide a ordem de dobra | §7.1 |
| Arco a curvar a partir da COLUNA | Risca por cima do nome das formas que salta | §7.1 |
| Rotação da operação a incluir *"cortar"* | Sobre-rodar apagaria a ligação por engano | §7.2 |
| Os oito botões a mexer só no `VecBoolGroup` | Ficariam MORTOS sobre um grupo com diagrama | §7.3 |
