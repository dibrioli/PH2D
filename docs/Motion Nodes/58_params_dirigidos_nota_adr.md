# 58 — Params dirigidos por fio (nota-ADR)

> **Status:** implementado (linha `line/motion-value`, 2026-07-13). FILA 2 do handoff.
> Companheiro do [doc 57](57_subgrafos_nota_adr.md) — mesma lei, segunda aplicação.

## 1. A decisão, numa frase

Um parâmetro pode ser **dirigido por um nó** (`value.lfo` → `force.wind.strength`), e isso
**não exigiu porta dinâmica**: exigiu uma **aresta que o manifesto não conhece**.

## 2. O plano dizia que era impossível — e estava lendo o problema errado

O [plano do módulo](01_plano_modulo_motion_nodes.md) deferiu isto duas vezes, com a mesma
justificativa: *"Promoção param→socket e 'olhinho': **deferidos** (exigem porta dinâmica no
modelo)"* e, na wave seguinte, *"promoção param→socket (agora **com porta dinâmica**)"*.

Porta dinâmica é de fato impossível: `NodeManifest.inputs` é `&'static [PortSpec]`
([ADR-0039](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md), contrato
**congelado**). Um nó não pode crescer uma porta. Mas **a porta nunca foi o requisito** — era
a forma que a solução tomaria num modelo onde tudo é porta. O requisito real é:

1. o **cook** tem que enxergar o driver como dependência (cozinhar antes, invalidar o memo);
2. o **nó** tem que ler o número (e são 86 tipos de nó — mexer em todos está fora de questão);
3. a **vista** tem que mostrar um lugar onde o fio pousa.

Nenhum dos três precisa de uma porta no manifesto. Precisam de **uma aresta**, e aresta é
**estado de DOCUMENTO** — que é exatamente onde o [canal de text param](32_expression_text_param_channel_nota_adr.md) já
mora (`Graph::set_text_param`, o precedente que resolveu o mesmo impasse pelo mesmo caminho).

> **É a terceira vez que o contrato congelado escolhe a arquitetura, e a terceira vez que o
> resultado é melhor do que o que teríamos feito sem ele.**

## 3. O modelo

```rust
Graph.param_sources: BTreeMap<NodeId, BTreeMap<String, (NodeId, u16)>>
Graph::drive_param(node, param, src)   -> Result<(), EdgeError>
Graph::undrive_param(node, param)      -> Option<Source>
```

Os verbos têm **a forma dos verbos de aresta** (`connect`/`disconnect`) porque é o que são.
E as invariantes são as mesmas:

- **ciclo**: `would_cycle` **anda pelo fio de param**. Uma dependência que o check não vê não
  é um connect recusado — é o `cook_node` recursando até estourar a pilha.
- **`remove_node`** limpa os **dois lados** (os params que o nó dirigia, e os que dirigiam
  ele). Fonte apontando pra nó deletado é socket ligado num fantasma: cozinharia `Empty` pra
  sempre em vez de falhar.
- **formato**: record `d` (`d <id> <param> <src> <port>`), semântico, re-validado no load como
  aresta. Header `v3` **só quando existe um** — um documento que nunca dirigiu param serializa
  **byte por byte** como sempre serializou.

## 4. Os 86 nós ficaram dirigíveis sem uma linha de mudança em nenhum deles

Porque todos leem param pelo **mesmo funil**:

```rust
EvalCtx::param(name)  =  fio  >  override  >  default do manifesto
```

Essa hierarquia (*"socket conectado > literal"*) estava **reservada no plano desde o dia 1** —
o que faltava não era o desenho, era perceber que ela cabia dentro do `param()`.

O `Cook` resolve as fontes **na mesma recursão** que resolve as portas, e a revisão do driver
entra **no mesmo fingerprint**. Mais um campo: o fingerprint da **FIAÇÃO** — re-apontar o param
pra **outra porta do MESMO nó** tem a mesma revisão e valor diferente; sem isso o memo devolve
o número velho pra sempre. (Meu primeiro gate desse campo **nasceu falso** — re-apontar pra
outro *nó* já muda a revisão sozinho, então ele passava com o campo deletado. Reescrito pro
caso que de fato morde.)

**Um número, não N.** Um param é um número e um stream são muitos: o param lê o **primeiro**.
Não é meio-termo — é a convenção que este grafo já fala na direção contrária: um nó de valor
sem input de geometria emite um stream de **comprimento 1** (`value.lfo`: *"cardinality follows
the geometry, else the length-1 global oscillation"*), que o `motion.drive` re-espalha pra N.
Dirigir param **é** o caso length-1. Param **por-instância** é outra feature, e já existe: é
uma porta de entrada de verdade (o `value` do `motion.drive`), declarada pelos nós que querem.

Um driver que emite **nada** deixa o param **no valor que ele tinha** (override/default), não
em zero: um fio que ainda não produziu número não disse que o número é zero.

## 5. Não existe estado "promovido" — e isso é deliberado

Cavalry e Houdini fazem você **promover** o parâmetro e **depois** ligar o fio. Aqui **o fio É
a promoção**: solta o fio no **corpo do nó**, escolhe o param no menu, e o socket **aparece
porque o fio existe**. Puxa o fio fora e o socket vai junto.

É a **mesma lei do doc 57 §3** (a fronteira do card é derivada das arestas que cruzam), e a
mesma máquina: o menu que o [doc 57 §6.1](57_subgrafos_nota_adr.md) construiu pro card já
perguntava *"onde dentro esse fio vai?"*. Um card esconde **portas**; um nó esconde
**parâmetros**; a pergunta é uma só.

Um socket que existe só pra ser preenchido é um estado que o artista tem que manter.

## 6. As armadilhas que o desenho desarmou

1. **A saída antecipada do fold matava a feature nos grafos SEM grupo** (que são a maioria):
   `fold` retornava cedo quando não havia subgrafo, e publicava o mapa de alvos **vazio**. O
   gate e2e nasceu vermelho por isso.
2. **`port: 0` como sentinela pra "isto é um param" colidiu com a porta 0 de verdade** e
   derrubou um gate do doc 57 que estava verde há um dia. Virou `ChoiceTarget::{Port, Param}`
   — o tipo não finge que as duas coisas são a mesma.
3. **O `card_ports` do subgrafo tinha que enxergar o fio de param.** Sem isso, agrupar um nó
   cujo param é dirigido de fora fazia **o fio sumir da tela** — o cook continuava lendo, e a
   tela mentia sobre o que a cena computa.
4. **Três lugares desligam um fio** (Disconnect, faca, ponta arrastada). Nenhum deles aprendeu
   o que é um param: todos passam por **um funil** (`subgraph::unplug`), a mesma disciplina que
   impede o `reconcile` de ser esquecido num dos seus sete call-sites.
5. **Um knob dirigido não pode continuar girável.** A linha vira **read-only** e mostra o
   número vivo que o fio está pondo (lido do MEMO do cook, nunca reavaliando) — e não registra
   widget nenhum, porque *dim é cosmético e widget dimmed ainda despacha*.

## 7. Superfície (para o integrador)

- **Foundational tocado:** `ph2d-nodegraph` — módulo novo `param_source.rs` (isolado), campo
  novo no `Graph`, 4 acessores, `would_cycle`/`remove_node` estendidos, `EvalCtx.driven`,
  `attr::VALUE_COLUMN` (a coluna `"v"`, que era um const privado redeclarado em cada crate de
  valor). **Contrato congelado intacto:** `architecture_contract_surface` = 3 verdes (8/2/1).
- **Formato:** header `v3` + record `d` — **aditivo**, ausente quando não há param dirigido.
- **Painel:** `ScalarRow.driven` (campo novo — toda construção precisa dele).
- **Aberto:** dirigir param **de dentro** de um card por um fio que vem de fora do card já
  funciona (o card cresce o socket); o inverso — um param **da própria caixa** promovido pra
  interface do grupo (o *Group Input* do Blender) — não existe, e nem deveria: é reuso, e reuso
  é o §7 do doc 57.
