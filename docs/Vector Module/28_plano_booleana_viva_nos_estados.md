# A BOOLEANA VIVA nos ESTADOS de UI — e o verbo que muda no meio da animação

> `line/Vector`, 2026-08-23. **CONSTRUÍDO.** Este doc é o registo da wave, não um plano.
>
> Pedido do Enio: *"Sistema Live Boolean compatível plenamente com o sistema de animação States,
> inclusive com a possibilidade de mudar o tipo do boolean no meio da animação"* — e, no arranque
> da implementação: *"as formas além de mudar o modo do boolean também podem estar animadas em
> pos, scl e rot"*.
>
> Smoke: `PH2D_BUILD_SMOKE=74`. Schema: `PROJECT_SCHEMA` **89 → 90**.

## 1. Pesquisa — o que a indústria faz, e o que ela NÃO oferece

| referência | como anima um verbo booleano | o que isso custa |
|---|---|---|
| **Blender** — modificador Boolean | a `Operation` é um **enum**, e o Blender força interpolação **CONSTANT** em canais de enum: o valor **salta** no keyframe | previsível, e visualmente um POP |
| **After Effects** — *Merge Paths* numa shape layer | é um dropdown; desde o *Dropdown Menu Control* (2020) ele é dirigível por expressão, e **segura** o valor (step) | idem |
| **Figma** — Smart Animate | casa camadas e interpola transform/opacidade/cor/raio. Uma mudança de forma que ele **não sabe casar** vira **dissolve** (crossfade) | suave, mas *fantasma*: vê-se as duas formas sobrepostas |
| **Rive** — State Machine | mistura *animações*; entradas discretas são `trigger`/`bool` e são **stepped** | idem Blender |
| **Illustrator** | não tem linha de tempo | — |

⭐ **Conclusão:** **nenhuma das quatro oferece uma transição contínua de verbo booleano.** O padrão
da indústria é o **salto**, e o melhor recuo conhecido (Figma) é o **crossfade**, que mostra as duas
formas ao mesmo tempo.

⚠️ **E é aqui que este app tem uma carta que elas não têm:** ele já sabe **interpolar uma forma
noutra** (`ph2d-vec-blend::Plan`, o motor do Blend Object, do Morph e do Smart Animate), inclusive
**casando contornos** — o de fora com o de fora, o buraco com o buraco.

## 2. A MEDIÇÃO que decide o desenho

Sonda: [`probe_boolean_morph.rs`](../../crates/ph2d-vec-blend/tests/probe_boolean_morph.rs)
(`cargo test -p ph2d-vec-blend --release --test probe_boolean_morph -- --ignored --nocapture`).

### 2.1 A peça PARADA — o buraco nasce de um ponto

Duas fixtures, e a segunda existe por disciplina: **a primeira não contém o fenômeno**.

**Rig TRIO** (a cena `=48`): os quatro verbos dão **1 contorno**. Ele nunca exercita a mudança de
topologia — usá-lo sozinho teria "provado" o que ele não contém.

**Rig DONUT** (um retângulo com outro inteiramente dentro):

| verbo | contornos | área |
|---|---|---|
| Union | 1 | 400,0 |
| Subtract | **2** | 336,0 |
| Intersect | 1 | 64,0 |
| Exclude | **2** | 336,0 |

| par | Plan? | contornos em `t=0,5` | área 0,0 / 0,5 / 1,0 |
|---|---|---|---|
| Union → Subtract | **sim** | **2** | 400,0 / 384,0 / 336,0 |
| Union → Intersect | sim | 1 | 400,0 / 196,0 / 64,0 |
| Intersect → Subtract | sim | 2 | 64,0 / 180,0 / 336,0 |

⭐⭐ **Os 12 pares constroem plano, nos dois rigs. E o buraco não aparece de estalo: ele CRESCE de
um ponto.** A área do furo vai `0 → 16 → 64` num furo de 64 — exactamente `t²`, que é o furo a
crescer **linearmente na dimensão** a partir de um ponto.

### 2.2 A peça a MOVER-SE — a pergunta que o Enio acrescentou

A régua **não pode ser a área**: ela é um escalar global, e um escalar global é cego a um salto de
correspondência (a lição das réguas do quad remesh). A régua é **quanta TINTA muda de sítio entre
dois quadros consecutivos** (a área da diferença simétrica) — a única pergunta que o olho faz, e a
única cega à parametrização.

Três travessias, 60 quadros cada, `Union → Subtract`:

| travessia | CONTROLE (só o movimento) | SALTO (Blender/AE) | **MORPH (o nosso)** | PERSEGUIÇÃO |
|---|---|---|---|---|
| o de dentro anda e fica DENTRO | 1,1 | **64,0** | **3,1** | 6,0 |
| o de dentro ATRAVESSA a parede | 3,7 | **62,1** | **38,3** (1 quadro) | 6,0 |
| a PEÇA INTEIRA viaja 100 unidades | 93,3 | 130,7 | **94,0** | **379,7** |

⭐ **O morph acompanha o movimento a 0,6 de tinta do controlo** (94,0 contra 93,3) na travessia mais
dura — ou seja, o desenho segue a animação de pos/scl/rot como se o verbo não estivesse a mudar.

⛔ **O que fica por curar, medido e nomeado:** quando o MOVIMENTO dos operandos muda a topologia de
uma das duas pontas a meio da transição (um operando a atravessar a parede da peça), o desenho dá
**um** passo de 38,3 de tinta nesse quadro — ainda **38% menor** que o salto da indústria, que
acontece em **todos** os casos, inclusive com a peça parada.

**Custo medido:** **0,73–0,95 ms** por quadro (dois cozimentos + `Plan::new` + `at`) de um quadro de
**16,67 ms**, e **só durante a transição**.

## 3. O desenho — dois canais, porque são dois FATOS

### 3.1 O que uma pose carrega

`ObjectPose` ganhou **dois** campos:

```rust
/// O verbo PRÓPRIO desta forma. `None` = ela herda o do grupo — a MESMA lei do `VecBoolOp`.
pub bool_op: Option<u8>,
/// A operação do GRUPO booleano acima dela. `None` = ela não está metida em booleana nenhuma.
pub bool_group_op: Option<u8>,
```

⚠️ **Dois e não um, e a razão é o `Trim`.** O primeiro é *"que verbo ESTA forma manda"*; o segundo é
*"em que operação ela está metida"* — e é o segundo que faz a receita **inteira** do grupo mudar
entre dois estados, inclusive as quatro **receitas** (`Trim`/`Crop`/`Merge`/`MinusBack`), que não têm
decomposição por forma nenhuma. Um campo só teria de escolher qual dos dois carregar, e a escolha
calada é como um `Trim` autorado no Hover não anima nada.

⚠️ **O `bool_group_op` repete-se em cada operando do mesmo grupo**, e a redundância é deliberada: o
grupo é uma entidade **sem `VecPathId`** e a pose é chaveada por caminho, então ele não tem slot
próprio. Quem o governa é a única chave que já existe — e como a captura lê os operandos todos do
mesmo sítio, os valores não podem divergir.

⚠️ **Ausência NUNCA desfaz o grupo.** `None` no segundo campo é *"não sei de grupo nenhum"*, e a
escrita simplesmente não acontece; lê-lo como *"remova o `VecBoolGroup`"* faria uma pose gravada
antes da booleana **destruir** a booleana no primeiro Show. Há gate, e ele mora ao lado do gate do
`None` do **primeiro** campo, que significa o OPOSTO (ali `None` REMOVE o override) — é o tipo de par
que alguém uniformiza por simetria.

### 3.2 Como o verbo atravessa a transição

`Transition::at` **segura** os dois canais na ponta de partida — não há meio caminho entre `Union` e
`Subtract`, e um número interpolado entre dois códigos dá a operação **errada** (o `2` entre `Union`
e `Exclude` é `Intersect`, que não está em nenhuma das duas pontas).

Quem desenha o meio é o **cozimento**, e ele recebe as duas pontas por uma pergunta separada:

```rust
Transition::bool_morphs(t) -> Vec<BoolMorph>   // { id, op, group_op, t }
```

⚠️ **Separada do `at` de propósito:** uma pose descreve *um objeto*, e o que muda quando o verbo
troca é **o que um GRUPO desenha**. Enfiar as duas pontas dentro da pose poria um transitório dentro
de um tipo que vai para o arquivo. E `t = 0` / `t = 1` devolvem **vazio** — nelas o desenho é uma das
pontas, que é o que um cozimento só já produz.

A `Machine` guarda o recado ao lado da pose viva e **apaga-o na chegada**; a ponte
(`ui_state_bridge::dispatch`) limpa-o em toda entrada e enche-o no mesmo `if` que instala a pose.

### 3.3 Quem cozinha, e o que ele faz

`BoolLive::recook` recebe os recados. Um grupo tocado por algum deles cozinha **as duas pontas** —
com as formas onde elas estão **agora** — e entrega o par ao `Plan`. Os outros correm byte-idênticos
ao que sempre correram.

- ⛔ **O memo é DESVIADO** durante o morph: a chave dele é *(op, verbos, entrada)* e nenhuma das três
  contém o `t`. Durante uma transição a entrada muda a cada quadro de qualquer maneira.
- **Sem plano, fica-se na PARTIDA** — a mesma lei do par degenerado do `Transition::at`.
- **Uma chegada que desenha o mesmo não paga segundo cozimento**, e há instrumento a medi-lo
  (`BoolLive::morphed()`), porque a igualdade do desenho é **cega** a este defeito: morfar duas
  pontas iguais devolve a mesma forma **ao bit**.

## 4. Contrato congelado e schema

**Não encosta em contrato congelado** — o §6 nomeia `ph2d-vector-doc` · `-traits` · `ph2d-painter-*`
· `ph2d-imageio`; `ph2d-ui-state` não aparece, e o `architecture_vector_contract_surface` *"escaneia
só `ph2d-vector-doc`+`-traits`"*.

**`PROJECT_SCHEMA` 89 → 90**, nos três sítios. Dois campos apendados ao `ObjectPose`, que viaja no
`HostStates` dentro do `ProjectFile`; postcard é posicional. ⚠️ **Nenhum registro novo** no
`ComponentRegistry` — os dois componentes que estes campos espelham já lá estavam.

## 5. A UI — as quatro condições, e a que faltava

| condição | estado |
|---|---|
| o componente **existe** | ✅ a fileira *This Shape* + os oito botões do grupo |
| é **pintado e registrado** | ✅ (com gate de gesto real + paridade de registro, 22/08) |
| o clique **chega ao barramento** | ✅ |
| a **sequência leva a algum lugar** | ⭐ **é isto que esta wave fechou** |

⇒ **Nenhum widget novo.** O trabalho foi a **captura** e a **aplicação**.

⚠️ **Como o artista alcança isto, e é uma restrição real:** a seção *States* exige um hospedeiro
**ÚNICO**, e clicar num operando acende o **grupo inteiro**. O hospedeiro tem de ser uma FORMA cuja
sub-árvore contenha o grupo booleano — um chip/fundo com a booleana pendurada nele. A lei que o
permite já existia (`selection_root` **para** quando o pai é uma forma), e a cena `=74` monta
exactamente essa disposição.

## 6. Os gates, e a prova de mutação

**11 gates novos**, e **10 mutantes mortos** (o arnês exige controle positivo por corrida e restaura
por `write_text`; ⚠️ `copy2` restaura o mtime e a mutação sobrevive ao restauro).

`ph2d-ui-state` — [`bool_morph_tests.rs`](../../crates/ph2d-ui-state/src/bool_morph_tests.rs):
o verbo segura e a troca viaja ao lado · as pontas são exatas e não publicam recado · a operação do
GRUPO viaja mesmo sem verbo de forma nenhum a mover-se · **o controle** (uma pose que só se move não
publica nada) · a máquina publica enquanto anda e apaga na chegada.

`shells/desktop` — [`vec_ui_state_edit_bool_tests.rs`](../../shells/desktop/src/vec_ui_state_edit_bool_tests.rs):
a costura captura⟷instala dos dois canais · um operando que HERDA grava `None` **e ainda assim grava
o grupo** · `None` no verbo próprio REMOVE o override · ⛔ uma pose que não conhece grupo **não
desfaz** o grupo · o controle (forma fora de booleana grava os dois vazios).

`shells/desktop` — [`bool_live_morph_tests.rs`](../../shells/desktop/src/bool_live_morph_tests.rs),
fixture **DONUT**: o meio não é nenhuma das pontas (buraco `16` de `64`) · ⭐ **o desenho segue os
operandos** (a peça viaja 30 e o desenho viaja 30; o de dentro encolhe e o buraco encolhe) · o grupo
troca de operação a meio, inclusive para uma receita · inércia · uma chegada que desenha o mesmo não
paga segundo cozimento · ⭐⭐⭐ **a composição do quadro** (a ponte publica, a booleana consome).

## 7. A cena de smoke — `PH2D_BUILD_SMOKE=74`

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
  env PH2D_BUILD_SMOKE=74 cargo run -p ph2d-host-desktop --release
```

Dois chips idênticos. O da **esquerda** nasce com as duas poses gravadas (pela porta do produto):
`Default` = `Union`, `Hover` = `Subtract` **e** o operando de dentro deslocado e maior. O da
**direita** é o CONTROLE, sem pose nenhuma.

⚠️ **Ela nasce PRONTA**, ao contrário da `=48`: o que ela prova é a ANIMAÇÃO, e uma animação que
exige quinze cliques antes de aparecer não é smokável (*feature nova = auto-play*). A cena imprime
**quantas poses gravou** e **quantas formas trocam de operação** — se o segundo número for zero,
PARE.

---

## ⛔ Recusas MEDIDAS

| O quê | Por quê | Onde |
|---|---|---|
| **Saltar o verbo** (Blender / AE / Rive) | move **64,0** de tinta num quadro com a peça PARADA, contra 3,1 do morph | §2.2 |
| **Crossfade** das duas formas (o recuo do Figma) | mostra as duas ao mesmo tempo; o morph não tem fantasma | §1 |
| **Perseguir a partir do vivo** (morfar do que está na tela para a chegada, pela fração do que falta) | cura o único quadro que salta (38,3 → 6,0) e paga com o desenho a **ficar para trás do movimento**: numa peça que viaja 5× a própria largura ele afasta-se **793,0** de tinta do par fresco e salta **379,7** | §2.2 |
| **Um segundo motor de morph** (campo implícito + marching squares) | seria contínuo por construção, e é *dois motores e um estado*: perde as curvas de Bézier e diverge do `Plan` numa screenshot | §3.3 |
| **Slot de pose para o verbo do GRUPO** com chave própria | o grupo não tem `VecPathId`; a chave nova divergiria da que já existe | §3.1 |
| **Um campo só** (o verbo efetivo) | não exprime as quatro RECEITAS, que são verbo da pilha inteira | §3.1 |
| **`None` na pose significar *"não mexe"*** | divergiria do componente, onde `None` é *herda* — dois vocabulários | §3.1 |
| **Validar o morph só com a fixture TRIO** | ela dá 1 contorno em todos os verbos: não contém a mudança de topologia | §2.1 |
| **Medir a continuidade pela ÁREA** | é um escalar global e é **cego** a um salto de correspondência | §2.2 |
| **Medir o custo pela igualdade do desenho** | morfar duas pontas iguais devolve a mesma forma **ao bit**: o dobro do trabalho fica invisível | §3.3 |
| **Medir *"segue o operando"* pelo centro do BURACO** | a meio caminho o buraco está entre o ponto em que nasce e o buraco real, então mover o operando 4 move o centro 2 — *foi a régua, não o motor* | §6 |
