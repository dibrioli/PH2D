# Plano — a **máquina de estados do Morph**, autorada no canvas

> **Fila F1** ([doc 29](29_fila_morph_state_machine_e_texture_pattern.md)) · pesquisa em
> [doc 31](31_pesquisa_maquinas_de_estado.md) · o estado medido na reabertura está no §
> *RECONFERÊNCIA* do doc 29.
>
> Enio, 2026-08-24: *"um tipo de state machine específico para o tool Morph (…) entre múltiplas
> formas, de forma não destrutiva e funcional no runtime do game"*, com as setas *"no próprio canvas
> 2D (…) e nas setas colocaremos condições"*, e a configuração *"à seção states do módulo vector
> utilizando um modo preview"*. Em 2026-08-25, com as duas opções na mão: **"as setas devem ser
> desenhadas no canvas onde as formas foram desenhadas"**.

## §0 — O modelo, numa frase

⭐ **Um ESTADO é uma FORMA DESENHADA.** O artista desenha A, B e C no canvas e liga-as com setas; o
estado da máquina é *em qual delas ela está*, e a seta é *como se vai de uma para a outra*.

É a decisão que **apaga o caso especial**: nada de novo a nomear, nada de novo a gravar, e o que ele
vê **é** o modelo. Foi ela que a frase do Enio escolheu — *"setas de uma forma para outra"*.

### ⚠️ Isto NÃO é o `ph2d-ui-state`, e a correcção fica escrita

Na reabertura eu afirmei ao Enio que *"a máquina que contém a outra é a `ph2d-ui-state`"*. **Estava
errado**, e a medição que o mostra é sobre o produto, não sobre o código: aquela máquina interpola
**poses de N objectos** (translação, tinta, traço, geometria) entre papéis **fixos** de UI — hover,
pressed. Um estado dela é uma **gravação**, não uma forma no canvas. Aqui o assunto é **duas formas
e um `t`**.

> *Quem decide qual subsistema serve é o que o ARTISTA desenha, não o código que está por perto.*

O que de facto se reusa: o **motor** (`ph2d-spring`, `ph2d-anim::Easing` — os dois já folhas), o
**renderizador do par** (`VecMorph`, intocado), a **separação de undo** (`Driver::MorphT`, que já
existe) e o **vocabulário das condições** (as acções do Input Map, de ontem).

## §1 — A porta ÚNICA de cada pergunta

| Pergunta | A porta | ⚠️ |
|---|---|---|
| *que formas, que setas, que condições?* | `MorphGraph` (autorado, no documento) | as formas são **derivadas** das setas (`shapes()`) — uma lista à parte teria um estado que nenhuma seta alcança |
| *em que forma estou?* | `MorphMachine::current` | salta **no lançamento**, nunca na chegada nem na fila (§2) |
| *que par a cena mostra?* | `MorphMachine::pair` → `VecMorph::sources` | fica o par do ÚLTIMO voo com `t=1`; ⛔ **não** `(current, current)` |
| *onde no caminho?* | `MorphMachine::t` → `VecMorph::t` | e o undo não o vê — `Driver::MorphT` já existe |
| *a acção X aconteceu?* | `MorphMachine::fire(&str)` | ⛔ a crate **nunca** resolve o nome |
| *o que fazer daqui?* | `MorphMachine::live_actions` | a cura do medo do Animator (doc 31) |
| *ver a seta sem lhe dar nome* | `MorphMachine::travel(ix)` | a porta da pré-visualização |

## §2 — As leis, e por que cada uma é assim

1. ⭐ **Só as setas do estado CORRENTE disparam** (`MorphGraph::from`) — a correcção nº 1 da
   pesquisa (o *State Tree* do Unreal). Um varrimento global é a teia do Animator, construída por
   acidente.
2. ⛔ **Uma seta sem condição nunca dispara sozinha.** Ela nasce sem nome quando o artista a
   desenha; sem esta guarda, toda seta recém-desenhada responderia a uma acção de nome vazio.
3. ⭐ **O `current` salta no LANÇAMENTO** — a meio de `A→B` as setas oferecidas são as de `B`.
4. ⛔⛔ **Mas ele NÃO salta num pedido em fila**, e o gate nasceu vermelho a provar porquê: com o
   salto na fila, o segundo pedido era lido a partir de um estado onde a máquina ainda não está —
   ou não casava com seta nenhuma (*o input do jogador desaparecia em silêncio*), ou casava com uma
   seta cujo `from` não é onde ela aterra, e o par **saltava um estado inteiro**.
5. **Um pedido a meio do voo ESPERA a chegada** (*input buffer* de UM). ⛔ As duas alternativas
   estão fechadas: **ignorar** perde o input; **saltar** não é exprimível, porque o `VecMorph`
   guarda **um par** e sair do meio de `(A,B)` para `(B,C)` precisaria de uma mistura de três.
6. **O mais novo ganha e a fila não cresce** — uma fila funda reproduziria teclas que o jogador já
   esqueceu. Seguro **por construção** graças à lei 4: todo candidato parte do mesmo sítio.
7. **A fila é RECONFERIDA na chegada** — o artista pode ter apagado a seta durante o voo.
8. ⭐ **Chegar não troca o par.** O cache de `Plan` do `morph_live` é chaveado pela geometria em
   **mundo** das duas fontes, e a busca de fase custa os **5,9 ms** que o `Plan` foi inventado para
   matar; `t=1` em `(A,B)` já **é** a forma B.

## §3 — Onde encosta

* **Contrato congelado (§6):** nenhum. A crate é nova e o `VecMorph` **não se mexe**.
* ⛔ **Schema: NÃO sobe — esta linha do plano estava errada e a W2 mediu-a.** O `ComponentBlob` é
  chaveado por `blake3(nome canónico)`, **não** por posição: um ficheiro gravado antes desta wave
  simplesmente **não tem** o blob, e a entidade volta **sem máquina** — que é a leitura correcta de
  *"ninguém desenhou seta nenhuma"*. É a mesma lição que a `line/3DModeling` registou na W35 (*"a
  peça atravessa o arquivo e o `PROJECT_SCHEMA` não se mexe; a nota que dizia o contrário era
  velha"*), e há gate: `a_morph_without_arrows_comes_back_without_them`.
* ⚠️ **O que SOBE é a contagem do REGISTO, nos TRÊS sítios** — `ph2d-ecs` **70 → 71** e os dois
  espelhos (`ph2d-render`, `ph2d-script`) **71 → 72**. Número que **soma entre linhas**; ⛔ conte-o
  contra o `main` do dia, nunca o copie daqui.

## §3-bis — As duas leis de LEGIBILIDADE da seta (W3a), e as três tentativas que morreram

1. ⭐ **`A→B` e `B→A` não se sobrepõem.** Toda máquina útil tem pares de ida e volta, e duas rectas
   entre os mesmos dois centros são **uma** recta na tela. As setas curvam, e o lado sai de
   `perp(u)` — a normal da **direcção de viagem**, que inverte com ela **por construção**.
   ⛔⛔ **Duas tentativas anteriores morreram, e a segunda passou no gate pelo motivo errado:**
   a 1ª fazia `perp(u) · sign(ids)` e os dois fatores **cancelavam-se** (em `B→A` o versor inverte
   *e* o sinal inverte) ⇒ as duas setas dobravam para o mesmo lado; a 2ª construiu um "versor
   canónico" `u·sign` e tirou-lhe a normal com o mesmo `sign` — que é **algebricamente `perp(u)`**.
   O gate ficou verde, mas a mutação `sign = 1.0` **não o matava**, porque o `sign` não decidia
   nada. *Um gate verde sobre código morto continua verde — quem o apanhou foi a prova de mutação.*
2. **A ponta encosta na BORDA da forma, não no centro.** Uma seta que morre no meio de um
   rectângulo grande fica escondida por baixo dele.
   ⚠️ **E ela aponta na tangente de CHEGADA, não na recta centro-a-centro** — numa seta curva as
   duas divergem. ⛔ **A régua desta lei também teve de se corrigir:** o teste do **sinal** do
   produto interno **sobreviveu à mutação** (com 22 px de curvatura sobre um segmento de centenas,
   as duas direcções passam num teste de sinal). A régua é o **ÂNGULO**: o bissector das duas abas
   é *exactamente* o oposto da tangente de chegada.
3. ⚠️ **A curvatura é de ECRÃ.** Uma curvatura em mundo desapareceria ao afastar o zoom — e é
   exactamente com a máquina inteira à vista que a ida e a volta precisam de se distinguir.

⚠️ **A seta é CHROME, não desenho:** não se selecciona como forma, não exporta, não imprime. É por
isso que ela vive no overlay e **não** como um `VecPath` derivado, que é o que o **conector** faz.
*O conector é uma linha que o artista quer no produto final; a seta é a explicação de uma regra.*

## §3-ter — O gesto (W3b): o mesmo movimento do conector, outro produto

⭐ **`DrawMode::MorphLink`** — pressiona numa forma, arrasta, solta noutra. É **um modo próprio** e
não uma variante do `Connect`, e a razão é o **produto**: o conector faz uma **linha no documento**
(que exporta, imprime e se selecciona); esta faz uma **aresta num grafo**, que é chrome. *Dois
produtos atrás do mesmo movimento da mão precisam de dois modos, senão o artista não tem como dizer
qual deles quer.*

⚠️ **O hit-test é o MESMO** (`App::shape_under_cursor`), não uma cópia — a pergunta é literalmente
a mesma, e duas respostas divergiriam no dia em que uma anotação nova nascesse.

* ⛔ **Sem um Morph SELECIONADO o gesto é inerte**, e é a resposta honesta: uma seta é uma aresta no
  grafo de alguém. Criar um `VecMorph` do nada a meio de um arrasto poria no documento um objecto
  que o artista não pediu.
* ⭐ **A PRIMEIRA seta faz nascer a máquina**, com `start` na forma de onde ela parte — é por isso
  que o `VecMorphMachine` não tem `Default`: o `start` é um facto do **gesto**.
* ⛔ **Uma forma não se liga a si própria.** ⚠️ E isto **não** é a decisão do conector, que aceita o
  laço de propósito: lá o laço é um **desenho** legítimo; aqui seria uma regra vazia.
* ⚠️ **Uma seta repetida não se duplica** — duas arestas iguais seriam duas linhas idênticas no
  painel, uma impossível de distinguir da outra ao apagar (a lei que a ligação de tecla do Input
  Map já paga).
* A seta **em voo** é **recta**: a curvatura existe para separar a ida da volta, e uma seta sem
  destino não tem par de quem se separar.

⚠️ **A lei mora FORA do `impl App` (`link_shapes`), e é isso que a torna gateável:** o gesto precisa
de um `AppGfx` — janela real e superfície de GPU —, que um teste não alcança (a mesma parede que o
undo do filtro do sculpt3d registou). ⇒ a **lei** tem gate de comportamento; a **costura** tem gate
de texto, e sem essa metade os outros quatro ficariam verdes sobre uma feature que **gesto nenhum
alcança**.

⛔⛔ **DEZ sítios para um modo novo, e o décimo só apareceu porque um gate o disse.** Eu editei
nove — o `enum` · o `NodeId` · o censo de ids · o re-export do painel · o `populate` (sem ele o
pill nasce **morto sob o ponteiro**) · a fileira pintada · o clique→modo · o gate de costura
id→modo · o rótulo i18n — e o portão reprovou com **`Ignored` em vez de `Consumed`**: havia uma
**décima** lista, uma allowlist de `VECTOR_MODE_*` em `event_clicks.rs`, e sem ela o clique era
**engolido** e o modo nunca trocava.

⚠️ **O pill teria ficado pintado, registado, e inerte** — o terceiro membro daquela família nesta
linha, e o único que nenhuma das minhas nove edições apanharia. *Uma feature espalhada por dez
listas escritas à mão só é alcançável se um gate percorrer as dez* — e a mensagem daquele gate
**nomeia o ficheiro que falta**, que é o que o torna útil em vez de só vermelho.

## §3-quater — A secção (W4a)

⭐ **Ela vive na MESMA seção *States*** que as poses de UI — Enio, 24/08 — e a lei da casa concorda:
**o Inspector mostra o que o objecto TEM** (ADR-0166). Um objecto raramente é as duas coisas, então
não é preciso aba nenhuma.

* **Duas linhas por seta**, o corte do `paint_signals`: `de -> para` + a lixeira em cima; a
  **condição** em baixo. Espremer as duas numa daria um chip de meia dúzia de caracteres.
  ⛔ **`->` em ASCII**: a fonte da casa não cobre o bloco de setas do Unicode e o glifo sairia caixa
  vazia (gate `no_tofu_glyphs`, que já mordeu três vezes neste repo).
* ⚠️ **A condição é um MENU das acções do Input Map, nunca um campo de texto.** Um nome digitado
  pode não existir, e uma seta que espera uma acção inexistente **nunca dispara** — sem uma palavra
  na tela. *Um modelo que aceita o que o painel não mostra produz estado inalcançável.*
* ⚠️ **A opção `0` é o «—»**: tirar a condição tem de ser um gesto, senão o artista só poderia
  apagar a seta inteira para se arrepender.
* ⭐⭐ **Um Morph SEM máquina publica a face VAZIA, e nunca `None`.** `None` = *"a seleção não é um
  Morph"* (a seção nem fala de setas); vazio = *"é um Morph e ainda não tem setas"* — e é essa face
  que **diz o gesto** (nomeia o pill e o movimento da mão). Sem ela o artista vê um cabeçalho e nada
  por baixo, e *"não há setas"* e *"isto está partido"* leem-se igual.
* ⚠️ **O Morph é achado na seleção INTEIRA**, nunca em `sel.first()`: tocar num morph traz o grupo,
  e a seção mostraria as setas de um objecto enquanto o clique escrevia noutro (a lição do
  `host_of_selection`).
* ⚠️ **As acções são PUBLICADAS pela shell**, e o índice escolhido resolve-se contra **essa mesma**
  lista — uma segunda leitura poria o nome escolhido a apontar para outro no quadro em que o artista
  criasse uma acção.
* ⛔ **A seta NÃO tem botão de «percorrer» nesta wave, e a ausência é deliberada:** o que ele faria é
  pôr a máquina VIVA a andar, e ela nasce na W5. Um botão pintado antes disso é um clique que não
  faz nada — *é assim que o artista aprende a não confiar nos botões desta seção*, e é a lei que a
  própria seção de poses já escreve (*"Show e Clear só existem depois do Rec"*).

## §4 — As waves

| | | estado |
|---|---|---|
| **W1** | **A LEI**, folha (`ph2d-morph-machine`): grafo · setas · condições · fila · mola/curva | ✅ **2026-08-25** — 13 gates, **8 mutações, 8 sangraram** |
| **W2** | O componente + a persistência | ✅ **2026-08-25** — 2 gates, 1 mutação, e ⛔ o `PROJECT_SCHEMA` **não** se mexeu (§3) |
| **W3a** | **O CANVAS, metade de VER**: as setas desenhadas entre as formas | ✅ **2026-08-25** — 8 gates, **6 mutações, 6 sangraram** |
| **W3b** | **O CANVAS, metade de AUTORAR**: `DrawMode::MorphLink` — arrastar de uma forma para outra cria a seta | ✅ **2026-08-25** — 5 gates, **4 mutações, 4 sangraram** |
| **W4a** | A secção **States**: a lista de setas + a **condição** (menu das acções do Input Map) + apagar | ✅ **2026-08-25** — 7 gates, **6 mutações, 6 sangraram** |
| **W4b** | O **ritmo** por seta (duração · curva · mola) — e o botão de **percorrer**, que precisa da máquina viva (W5) | ⏳ |
| **W5** | O **modo preview** + o ledger de undo (⚠️ o `Driven::MorphT` cobre o `t`, **não** o `sources`) | ⏳ |
| **W6** | A cena de smoke, com números MEDIDOS | ⏳ |

⚠️ **O que a W5 vai encontrar, e está medido de antemão:** o ledger de pré-visualização
(`preview_drive.rs`) já tem `Driver::MorphT` / `Driven::MorphT(f32)` — construído em 23/08 para a
curva de `Morph` da timeline. Ele guarda **o `t` e só o `t`**. A máquina escreve **também o
`sources`**, e esse facto **não tem dono no ledger** ⇒ sem o acrescentar, mudar de par durante a
pré-visualização entra no undo.

> ⚠️ **Meça cada linha deste plano antes de a honrar.** Escrito em 2026-08-25.
