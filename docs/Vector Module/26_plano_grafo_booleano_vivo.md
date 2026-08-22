# ⛔ RETIRADO — o GRAFO da booleana viva (a "máquina de estados" das booleanas)

> `line/Vector`, 2026-08-22. **Construído e RETIRADO no mesmo dia, por veredito de produto.**
>
> Este doc não é um plano: é o **registo da recusa**. Ele existe para que ninguém reconstrua isto
> sem saber o que já foi medido, o que funcionou, e — sobretudo — **o que exatamente foi rejeitado**.

## 1. O pedido, e o veredito

**Pedido** (Enio, 2026-08-22):

> *"Avalie a possibilidade de um tipo de 'State Machine' para operações booleanas em tempo real
> (Live). Abre-se numa janela um círculo representante de cada forma e liga-se uma forma a outra com
> linhas e nessas linhas escolhemos o modo de interação de uma forma com a outra de modo que a mesma
> forma pode usar Union para uma forma e Subtract para outra forma."*

**Veredito, depois de duas rodadas de smoke:**

> *"não ficou legal. confuso de usar. Vamos retirar a máquina de estados para live boolean."*

⚠️ **A recusa é de ERGONOMIA, não do modelo.** O motor fazia o que prometia (os gates provavam-no) e
a medição dizia que cabia folgado no orçamento de quadro. O que reprovou foi **usar**. É a distinção
que a memória `feedback_a_reverted_attempt_may_differ_only_in_lifetime_read_the_revert_reason`
nomeia: *leia o MOTIVO do revert, não o diff.*

✅ **E a capacidade CHEGOU por outro caminho, no mesmo dia:** o verbo passou a ser uma propriedade
de cada FORMA, lida na ordem que a hierarquia já mostra — sem janela, sem gesto e sem posições a
guardar ([27](27_um_verbo_por_forma.md)). *O que estava errado nunca foi o modelo; era a segunda
superfície inventada para o exprimir.*

⚠️ Corolário duro: **uma 2ª tentativa não começa por reconstruir isto.** Ela começa por perguntar
*o que era confuso* — e a resposta **não está neste doc, porque eu não a tenho**. A árvore inteira
sobrevive em `e0796e537` (e as etapas em `c781b77f8`, `fb37eb28f`, `25b8b7b64`, `783d1b8b7`).

### 1.1 As duas formas que ele viu, e o que cada uma pedia

Vale registar, porque a 2ª nasceu de uma correção pedida à 1ª:

1. **Coluna por z, ligações em arco.** O Enio: *"o modal ficou muito pequeno e não é possível
   organizar os ítens no espaço 2d… Liberdade para arrastar os círculos e criar conexões."*
2. **Plano livre**, círculos grandes arrastáveis, número de z no círculo, arrasto do aro para ligar,
   clique no miolo para selecionar. É esta que levou o *"confuso de usar"*.

## 2. O que ficou DE PÉ, apesar da retirada

- ⚠️ **A capacidade nunca dependeu do diagrama.** *"Union com esta, Subtract daquela"* **já era
  exprimível antes e continua a ser**, aninhando grupos booleanos dentro de grupos — o `bool_live`
  resolve do mais interno para fora, e há gate (`a_nested_boolean_cooks_from_the_inside_out`).
  O diagrama acrescentava **expressão**, não poder. Retirá-lo não tirou nada ao artista.
- **A correção do doc-comment do `ph2d-vec-boolean`** ficou: a frase *"é destrutivo por design"* era
  falsa desde que a booleana viva shipou.
- **A correção de acessibilidade do `paint_clip.rs`** ficou: era um defeito REAL da tarefa do recorte
  (2026-08-21), apanhado de passagem, e nada tem a ver com o grafo.

## 3. ✅ O DEFEITO que o diagrama expôs — CURADO, e sem diagrama nenhum

Durante o smoke, o Enio reportou:

> *"depois de configurar só é possível selecionar e mover no canvas uma shape."*

**Isto não era do grafo.** É uma lei pré-existente da booleana viva: um operando consumido desenha
**VAZIO** no mapa, e a regra do canvas é *"nada desenhado, nada pego"*
([`vec_gizmo_pick`](../../shells/desktop/src/vec_gizmo_pick.rs)) — ele ficava inalcançável **pelo
clique no canvas**. Estava assim desde que a booleana viva shipou; o diagrama só o tornou óbvio, e
depois passou a ser a porta que o contornava (clicar num círculo selecionava a forma).

Retirado o diagrama, o defeito voltou ao que era — e foi **curado à parte, em 2026-08-22**.

### 3.1 A lei, numa frase

> **A tinta do GRUPO é a porta dos operandos dele.** Onde a booleana desenha, cada operando
> absorvido é alcançável; onde ela não desenha, nada é pego.

⚠️ **A cura não fura a lei do pick — dá ao operando a porta que ele de facto tem.** Um operando
absorvido não desapareceu: ele continua no documento, continua a contribuir, e o grupo desenha *por*
ele. O que faltava era distinguir isso da **ANIQUILAÇÃO** (o offset que come a forma), e no mapa as
duas são o mesmo `Some(vec![])`.

### 3.2 O mecanismo

O `bool_live` publica os pares *(operando, base que carrega a tinta)* no `VecViewState.absorbed` —
a mesma prateleira dos `clips` e das `poses`, e pela mesma razão: é um fato que só o DESENHO sabe, e
o hit-test monta o estado dele do zero a cada evento de ponteiro. Zero assinaturas de pick mudaram.

Três consequências, cada uma com gate:

1. **A porta é a tinta do grupo, nunca o footprint do operando.** ⛔ A variante ingênua — alcançar o
   operando onde ele está — passa no gate do alcance e **falha** no do `Subtract`: o cortador ocupa
   exactamente o BURACO, então clicar em tela limpa selecionaria uma forma invisível e roubaria o
   clique de quem está por baixo.
2. **Quem está sob o dedo vem primeiro.** Dentro da tinta, TODOS os operandos respondem em qualquer
   ponto — sem uma partição, clicar no lobo esquerdo de uma união nomearia o círculo do topo, que
   pode ser o da direita. O resto da lista fica ao alcance do clique seguinte, pelo ciclo que o
   canvas já tinha.
3. **O aninhamento resolve-se na publicação, não no pick.** A base de um grupo interno é ela própria
   operando do externo, logo o mapa dela também acaba vazio: um par direto apontaria para uma porta
   **sem tinta**, e o defeito voltaria inteiro — só nos documentos aninhados, que são os que ninguém
   smoka.

E o gizmo não precisou de nada: a caixa dele **já** vinha da FONTE (decisão do `vec_gizmo_view`,
não esquecimento), então o operando alcançado nasce com caixa e alças no sítio certo. O marquee é
que precisou da mesma tabela — sem ela apanhava só a base, e arrastar a seleção partia a booleana
ao meio.

### 3.3 A prova

Oito gates em [`vec_bool_pick_tests.rs`](../../shells/desktop/src/vec_bool_pick_tests.rs), em pares
alcance/regressão, e **sete mutantes mortos com sangramento diferenciado** — a mutação canônica (o
`absorbed_door` a devolver sempre `None`, que é o produto de antes) sangra os seis de alcance e
deixa **verdes** os dois que defendem a tela limpa.

⚠️ **A primeira rodada de mutação MENTIU, e no sentido perigoso:** o harness restaurava os arquivos
com `shutil.copy2`, que repõe o **mtime original** — mais antigo que o artefacto recém-compilado. O
cargo deixou de reconstruir a crate, e os seis mutantes seguintes correram **com o primeiro ainda
ligado**, todos a sangrar exactamente os mesmos seis gates. Sete mortos, zero provados. *O sinal de
que era harness e não prova foi o sangramento ser IDÊNTICO — um mutante que só toca o aninhamento
não pode derrubar o gate de duas formas.* Restaure por `write_text`, e ponha um controlo que exige
o verde de volta antes do mutante seguinte.

## 4. O que foi MEDIDO (os números sobrevivem à recusa)

Custo de UM frame de booleana viva — o `recook` inteiro, com o memo invalidado a cada volta (o caso
do arrasto, o único em que o custo importa). Máquina calma, release. Orçamento de um quadro a
60 fps: **16,6 ms**.

| operação | operandos | grupo (a booleana de hoje) | grafo |
|---|---|---|---|
| Union | 2 | 0,055 ms | 0,058 ms |
| Union | 10 | 1,500 ms | **1,924 ms** |
| Subtract | 2 | 0,134 ms | 0,107 ms |
| Subtract | 10 | 0,555 ms | 0,569 ms |
| Intersect | 2 | 0,091 ms | 0,081 ms |
| Intersect | 10 | 0,259 ms | 0,308 ms |

**Leitura:** empate em cinco das seis células (±0,03 ms é ruído). A única real é `Union × 10`:
**+28%**, e ainda assim 11,6% de um quadro. ⛔ **Custo nunca foi o motivo da recusa.**

⚠️ E o custo só é pago quando algo MUDA: o `BoolLive.memo` compara a entrada e reaproveita.

## 5. As leis que foram DERIVADAS (para não se re-derivarem do zero)

Se um dia isto voltar noutra forma, estas continuam verdadeiras:

1. **A seta é a ordem do fold** — `A−B ≠ B−A`, e círculos ligados por linhas sem direção desenham
   duas coisas diferentes com a mesma aparência.
2. **Ligações que chegam ao mesmo nó dobram na ordem de z de quem opera.** ⚠️ Isto obriga o diagrama
   a MOSTRAR z de alguma forma — a 1ª versão usou uma coluna, a 2ª um número no círculo. Um diagrama
   que não mostre z esconde o que decide o resultado.
3. ⛔ **Só as quatro operações de CONJUNTO cabem numa ligação.** `MinusBack`/`Trim`/`Crop`/`Merge`
   são afirmações sobre uma PILHA inteira — *"cada forma menos a união do que está acima dela"* não é
   uma relação entre DOIS.
4. **A estrela derivada** (todos apontam para a base, com a operação do grupo) desenha **exatamente**
   o que o grupo desenha — e não por sorte: o `apply_many_checked` já É um fold binário da esquerda
   para a direita. Era isso que tornava a migração um no-op visível.
5. ⛔ **Componente ausente ≠ componente com lista vazia.** Se a lista vazia caísse de volta na
   operação única, cortar o último elo faria as formas **fundirem-se** — o oposto do gesto.
6. ⛔ **Um botão de operação que só mexe no `VecBoolGroup` fica MORTO** quando existe um grafo: quem
   manda passa a ser a operação de cada ligação. É o defeito *"parâmetro que não muda nada"*, e ele
   volta a valer para qualquer segunda tentativa.

## 6. O que a retirada mexeu

Removidos: `ph2d-vec-boolean::graph` · `ph2d_ecs::VecBoolEdges` · `ph2d_ecs::VecBoolGraphPos` ·
`widget::bool_graph` · `chrome::bool_graph_modal` · `state::bool_graph_ops` · `bool_graph_ui` ·
`bool_graph_input` · o botão e a linha *Links* do painel · nove chaves de i18n · os quatro ids.

Revertidos ao estado pré-grafo: `bool_live` (o `Cooked` volta a `{base, operands, out}`, sem
`rosters` nem `cook`) · `bool_gesture` (sem `retarget_graph`) · o `render_loop` · o `input_dispatch`
· os dois blocos de codegen (`widget-sync`, `chrome-sync`).

`PROJECT_SCHEMA` **88 → 86**, e os degraus v87/v88 saíram da escada. ⚠️ Eles descreviam componentes
que deixaram de existir; deixá-los seria uma mentira na escada **e** faria a próxima linha contar o
degrau errado. Registo de componentes **60 → 58**.

---

## ⛔ Recusas MEDIDAS

| O quê | Por quê | Onde |
|---|---|---|
| **O diagrama inteiro** | *"não ficou legal. confuso de usar"* — veredito de produto, **não** do modelo nem do custo | §1 |
| Alcançar o operando absorvido pelo **próprio footprint** | O cortador de um `Subtract` ocupa o BURACO: seria um clique em tela limpa a selecionar forma invisível | §3.2 |
| Ordenar a lista do clique só por **z** | Todos os operandos respondem em qualquer ponto da tinta ⇒ clicar num lobo nomearia o círculo do outro | §3.2 |
| Mapear o absorvido para a **base imediata** (sem cadeia) | Num grupo aninhado essa base também tem o mapa vazio: porta sem tinta | §3.2 |
| Grafo de nós para efeitos de UMA forma | Medido e rejeitado antes; a pilha por-path venceu | [ADR-0132](../architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md) |
| Receita (`Trim`/`Crop`/`Merge`/`MinusBack`) numa ligação | Não é relação entre DOIS; é afirmação sobre a pilha inteira | §5.3 |
| Um ANEL/plano SEM mostrar z | Apaga o dado que decide a ordem de dobra | §5.2 |
| Lista vazia == componente ausente | Cortar o último elo faria as formas fundirem-se | §5.5 |
| Os oito botões a mexer só no `VecBoolGroup` | Ficariam MORTOS sobre um grupo com diagrama | §5.6 |
| Reusar o motor de nós do Motion | ~11 k linhas amarradas ao `ph2d-nodegraph`, para uma lista de três `u64` | — |
