# Plano — o GRAFO da booleana viva (a "máquina de estados" das booleanas)

> `line/Vector`, 2026-08-22. Nasce de um pedido do Enio:
> *"Avalie a possibilidade de um tipo de 'State Machine' para operações booleanas em tempo real
> (Live). Abre-se numa janela um círculo representante de cada forma e liga-se uma forma a outra
> com linhas e nessas linhas escolhemos o modo de interação de uma forma com a outra de modo que a
> mesma forma pode usar Union para uma forma e Subtract para outra forma."*
>
> **Etapa 1 (o MODELO) está FECHADA.** Este doc é o roteador dela e a spec da etapa 2 (a janela).

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

## 7. Etapa 2 — a janela (POR FAZER)

O modelo está pronto e provado; falta a interface. O que ela tem de resolver:

1. **Um círculo por forma, uma linha por ligação.** A referência de gesto é o modal *Add Nodes* do
   Motion Nodes — ⚠️ mas **não se reusa o motor daquele módulo**: são ~11 k linhas amarradas ao
   `ph2d-nodegraph`, e o que aqui se precisa é um diagrama sobre `VecBoolEdges`, que é uma lista de
   três `u64`.
2. **Materializar a estrela ao abrir** sobre um grupo que ainda não tem grafo (§4 garante que não
   move a arte).
3. **A seta tem de ser VISÍVEL** — sem ela, o diagrama não diz `A−B` de `B−A`.
4. **O ciclo tem de ser dito na tela.** Hoje ele é uma recusa silenciosa: a arte fica correta e
   nada explica por quê. É aceitável para o modelo, ⛔ não para a janela.
5. **Apagar uma forma tem de chamar `VecBoolEdges::forget`.** Hoje a ligação órfã é **filtrada** no
   `recook` (gate `uma_ligacao_orfa_nao_apaga_a_booleana_do_grupo`), então ela nunca é fatal — mas
   fica no documento. A limpeza é da porta de autoria, que é a etapa 2.
6. **A operação do painel e o grafo têm de dizer a mesma coisa.** ⚠️ Um dropdown que não mexe em
   nada porque há grafo é o defeito *"parâmetro que não muda NADA"*. Duas saídas legítimas: o
   dropdown reescreve TODAS as ligações (e mostra *Mixed* quando elas discordam), ou ele fica
   desligado com o motivo à vista. É decisão de produto.

---

## ⛔ Recusas MEDIDAS

| O quê | Por quê | Onde |
|---|---|---|
| Grafo de nós para efeitos de UMA forma | Medido e rejeitado; a pilha por-path venceu | [ADR-0132](../architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md) |
| Receita (`Trim`/`Crop`/`Merge`/`MinusBack`) numa ligação | Não é relação entre DOIS; é afirmação sobre a pilha inteira | §3.3, gate `uma_receita_de_pilha_nao_e_uma_ligacao` |
| Recusa PARCIAL num ciclo | Desenharia arte que nenhuma leitura do diagrama explica | §3.4, gate `um_ciclo_num_canto_recusa_o_grafo_inteiro` |
| Lista vazia == componente ausente | Cortar o último elo faria as formas fundirem-se | §3.1, gate `um_grafo_vazio_nao_reinstala_a_operacao_do_grupo` |
| Reusar o motor de nós do Motion | ~11 k linhas amarradas ao `ph2d-nodegraph`, para uma lista de três `u64` | §7.1 |
