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

## §3-quinquies — A máquina a correr (W5)

⭐ **O «modo preview» que o Enio pediu é o RELÓGIO A ANDAR** — e ele já existia. Neste editor *"o
jogo a correr"* é o playhead, a mesma porta pela qual o dedo do jogador alcança a física; uma
terceira noção de runtime seria uma terceira coisa para o artista aprender.

⛔⛔ **E a guarda não é conservadorismo.** A condição de uma seta é uma **acção do Input Map**, isto
é, uma **tecla**. A escutar durante a edição, carregar em `Z` faria a forma mudar **e** o que quer
que o `Z` faça no editor — os dois, sem que nada na tela explicasse. É o argumento do `ui_preview`
(*"um hover que animasse a forma enquanto o artista trabalha tornaria o editor inutilizável"*) com
outro dispositivo de entrada, e a resposta é a mesma: **um modo**.

* ⭐⭐ **PARAR o relógio devolve a forma autorada, e isso não custa código:** é o que o ledger faz
  por construção. *Sair restaura o MUNDO, nunca «vá para o estado inicial»* — que moveria o desenho.
* ⛔⛔ **O `Driver::MorphT` sozinho NÃO bastava, e o plano já o tinha medido:** ele cobre o `t` e
  **só** o `t`. Sem o **`Driver::MorphPair`** novo, trocar de par durante a reprodução entrava no
  undo como se o artista tivesse re-ligado as fontes à mão. ⚠️ Dois motores sobre o mesmo
  componente é seguro **porque os campos são disjuntos** — a curva da timeline escreve o `t`, a
  máquina escreve o par; é a forma do par `SpriteAnim`/`SpriteAlpha` sobre a `Sprite`, e ali o doc
  avisa que *duas granularidades sobre o mesmo componente escreveriam por cima uma da outra*.
* ⚠️ **O ledger PRIMEIRO, a escrita depois** — ele precisa do valor ANTES para saber o que repor.
* ⚠️ **`just_pressed`, nunca `pressed`:** uma tecla segurada re-disparava a cada quadro e a máquina
  saltava a cadeia inteira num piscar de olhos.
* ⚠️ **Uma máquina cuja entidade morreu some junto** — senão sobreviveria ao objecto e o mapa
  cresceria para sempre (a varredura das `UiMachines`).

## §3-sexies — A cena (W6)

`PH2D_BUILD_SMOKE=75`. Ela arma o **material** — três formas bem diferentes (larga · alta · fina) e
um Morph entre as duas primeiras — e ⛔ **nada nasce ligado**: nenhuma seta, nenhuma condição.

⚠️ **É deliberado, e é a disciplina das cenas irmãs desta linha:** quem desenha as setas e escolhe o
que as dispara é o artista, e é **exactamente essa** a costura que a wave existe para provar. *Um
smoke que arma o gesto por baixo do pano pula a costura que devia testar.*

⚠️ **As acções são as de FÁBRICA** (`jump`, `dash`) — nenhuma nova é criada. A lista que o menu da
condição mostra é a do **projecto**, e usar a que já lá está prova que o vínculo com o Input Map é
real em vez de o simular. ⚠️ **E o roteiro nomeia a TECLA lida do mapa VIVO**, nunca de memória: se
o artista já remapeou, o texto acompanha.

⚠️ **A diferença entre as três formas é o instrumento**: um morph entre dois rectângulos parecidos é
indistinguível de *nada a acontecer*.

⭐ **Os passos 6 e 7 são os CONTROLOS**, e são o que separa esta feature de uma que parece funcionar:
parar o transporte tem de **devolver a forma desenhada** e deixar o Ctrl+Z **sem nada** para
desfazer; e com o transporte parado a tecla **não pode mover nada**.

## §4 — As waves

| | | estado |
|---|---|---|
| **W1** | **A LEI**, folha (`ph2d-morph-machine`): grafo · setas · condições · fila · mola/curva | ✅ **2026-08-25** — 13 gates, **8 mutações, 8 sangraram** |
| **W2** | O componente + a persistência | ✅ **2026-08-25** — 2 gates, 1 mutação, e ⛔ o `PROJECT_SCHEMA` **não** se mexeu (§3) |
| **W3a** | **O CANVAS, metade de VER**: as setas desenhadas entre as formas | ✅ **2026-08-25** — 8 gates, **6 mutações, 6 sangraram** |
| **W3b** | **O CANVAS, metade de AUTORAR**: `DrawMode::MorphLink` — arrastar de uma forma para outra cria a seta | ✅ **2026-08-25** — 5 gates, **4 mutações, 4 sangraram** |
| **W4a** | A secção **States**: a lista de setas + a **condição** (menu das acções do Input Map) + apagar | ✅ **2026-08-25** — 7 gates, **6 mutações, 6 sangraram** |
| **W4b** | O **ritmo** por seta (duração · curva · mola) — e o botão de **percorrer**, que precisa da máquina viva (W5) | ⏳ |
| **W5** | A **máquina VIVA** + o ledger de undo | ✅ **2026-08-25** — 5 gates, **5 mutações, 5 sangraram** |
| **W6** | A cena de smoke (`PH2D_BUILD_SMOKE=75`) | ✅ **2026-08-25** |

⚠️ **O que a W5 vai encontrar, e está medido de antemão:** o ledger de pré-visualização
(`preview_drive.rs`) já tem `Driver::MorphT` / `Driven::MorphT(f32)` — construído em 23/08 para a
curva de `Morph` da timeline. Ele guarda **o `t` e só o `t`**. A máquina escreve **também o
`sources`**, e esse facto **não tem dono no ledger** ⇒ sem o acrescentar, mudar de par durante a
pré-visualização entra no undo.

> ⚠️ **Meça cada linha deste plano antes de a honrar.** Escrito em 2026-08-25.

---

## §5 — ⭐⭐ O PIVÔ de 2026-08-25: um botão, o grafo completo, e as setas deixam de se ver

> Enio, depois do smoke:
>
> > *"É impressão minha ou vc contaminou ou até mesmo estragou a feature states previamente
> > implementada? Os states de morph deveriam ter sessão exclusiva. Outra coisa: melhor criar um
> > modo automático de atribuição: 1) o usuário seleciona todas as peças que estarão envolvidas na
> > máquina de estados do morph. 2) Com o clique de um único botão um objeto novo surge na
> > hierarquia tendo como filhos as shapes escolhidas. Todas as setas são atribuídas automaticamente
> > cobrindo todas as morphs possíveis entre todas as formas (tanto de ida como de volta). As setas
> > são virtuais e ninguém jamais vê. No canvas uma única shape aparece (a shape do estado atual) e
> > as demais ficam ocultas. Na seção exclusiva dos estados no painel aparece a opção de atribuição
> > de inputs para cada uma das morphs possíveis. Restaure ao original o painel e funcionamento da
> > seção states para criação de animações."*

### §5.1 — A contaminação: o que a W4 fez, e por que o argumento dela era irrelevante

A W4 pôs as transições do Morph **dentro** da seção `States` — a das poses de UI e do Smart Animate.
O argumento escrito no código era o [ADR-0166](../architecture/decisions/) (*o Inspector mostra o que
o objecto **TEM***) mais *"um objecto raramente é as duas coisas"*.

⚠️ **Os dois são verdadeiros, e nenhum deles era a pergunta.** O efeito prático foi:

| o que a seção `States` fazia antes | o que passou a fazer |
|---|---|
| aparece com uma forma única na seleção | aparece **também** por causa de um Morph |
| o corpo é sempre poses | o corpo pode ser transições de outra feature |

⇒ *A lei do ADR-0166 diz **o que mostrar**, nunca **onde**.* Duas features com donos diferentes,
histórias diferentes e gates diferentes debaixo de um cabeçalho só é **uma porta a mais na seção de
quem chegou primeiro** — e quem chegou primeiro é quem paga a regressão.

⛔⛔ **A causa de fundo é a mesma da auditoria do Input Map, e é o achado que importa:** dos doze
gates da W4, **nenhum olhava para o que era PINTADO**. Todos mediam o mapa e o estado publicado.
É por isso que doze verdes conviveram com um cabeçalho alheio a aparecer. O gate novo
([`seam_morph_states.rs`](../../crates/ph2d-panel-vector/tests/seam_morph_states.rs)) mede a
**ausência nos dois sentidos** — um morph não pinta o cabeçalho `States`, e poses não pintam o
`Morph States` —, e a mutação que repõe a forma exacta da W4 sangra com essa mensagem.

**A restauração é literal:** `paint_states.rs` voltou por `git checkout main --` e o
`git diff main` desse ficheiro é **vazio**.

### §5.2 — O modo automático: `n` formas, `n(n-1)` transições, um passo de undo

O clique em **Make Morph States** faz quatro coisas **no mesmo quadro**:

1. nasce o objecto (um `VecPath` vazio + `VecMorph`, que é o que a cena **desenha**);
2. as formas escolhidas viram **filhos** dele (`ChildOf`), na ordem de z;
3. cada uma ganha `Visibility::hidden()` — *no canvas aparece uma forma só*;
4. o `VecMorphMachine` recebe o **grafo completo dirigido** sobre elas.

⚠️ **Um passo de undo, não quatro.** As quatro escritas caem no mesmo quadro e o `post_frame_undo`
regista por DIFF. Reparentar num quadro e esconder no seguinte daria dois passos, e o primeiro
deixaria o artista com `n` formas empilhadas.

⚠️ **`sources = [start, start]` e `t = 0.0`**, e **não** `VecMorph::new` (que nasce a `t = 0,5` de
propósito, para um morph autorado *se anunciar*). Aqui é o contrário: o conjunto tem de mostrar
**exactamente** o estado inicial, senão a primeira coisa na tela é uma forma que ninguém desenhou.

⚠️ **A ordem das arestas é determinística** (`from` externo, `to` interno, ambos na ordem dos
membros). A lista do painel indexa por **posição**: uma ordem que dependesse de iteração de mapa
faria o menu de uma linha escrever a condição noutra depois de um undo.

⚠️ **Toda transição nasce SEM condição.** É a metade que torna o grafo completo seguro — se cada
aresta nascesse com uma acção, um conjunto de 9 formas nasceria com 72 regras a disparar todas na
primeira tecla.

### §5.3 — O TETO, medido: 9 formas

O recurso é o **relógio de pintura do painel**, e o número que manda é o de **formas** (o artista
escolhe formas; as setas são derivadas). Medido com o `MockPanelHost` a pintar o painel Vector
inteiro, release, 2026-08-25 — o painel **sem** esta seção custa `0,746 ms`:

| formas | setas `n(n-1)` | painel | delta da seção | % de um quadro de 16,7 ms |
|---:|---:|---:|---:|---:|
| 7 | 42 | `1,181 ms` | `0,435 ms` | 7,07 % |
| 8 | 56 | `1,330 ms` | `0,584 ms` | 7,97 % |
| **9** | **72** | **`1,497 ms`** | **`0,752 ms`** | **8,97 %** |
| 10 | 90 | `1,699 ms` | `0,954 ms` | 10,18 % |
| 11 | 110 | `1,899 ms` | `1,154 ms` | 11,37 % |

Slope linear: **`0,0104 ms` por linha**. ⇒ **9 é o último `n` em que esta seção sozinha não custa
mais do que TODO o resto do painel junto** (`0,752` contra `0,746`). Em 10 ela passa a custar mais
que todas as outras seções somadas, e o painel existe para responder a mais perguntas do que esta.

⛔ **O pool de ids NÃO é o recurso** — foi a primeira hipótese e a medição refutou-a: registar 132
linhas × 25 widgets custa `0,293 ms` **uma vez**, no `populate`, nunca por quadro.

⛔ **A régua não ficou como gate.** Ela divide dois relógios, que é exactamente a família de flakes
sob fan-out do `CLAUDE.md` §5.0 — a tabela acima é o registo, e re-medir é rodar a sonda.

### §5.4 — ⛔ RECUSAS MEDIDAS desta wave (não reconstrua sem ler)

| o que foi retirado | por quê |
|---|---|
| **as setas desenhadas no canvas** (W3a, `morph_arrow_overlay.rs` + gates) | *"as setas são virtuais e ninguém jamais vê"* — decisão directa do dono. E desenhar `n(n-1)` setas entre formas que **já estão escondidas** é ruído sobre uma resposta que ninguém precisa de ler no canvas. |
| **o modo de arrasto forma→forma** (W3b, `DrawMode::MorphLink` + `morph_link_gesture.rs`, dez sítios) | o grafo passou a ser **completo por construção** ⇒ o arrasto criaria uma aresta **que já existe**. Um gesto cujo produto já está lá é um gesto que não faz nada, e o pill dele competiria com treze irmãos pela fileira. |
| **a lixeira por linha** (`ArrowCmd::Delete`) | o conjunto de arestas é uma **função pura** das formas. Apagar uma linha seria apagar uma passagem que a próxima derivação repõe. *Desligar uma transição é tirar-lhe a condição* — o «—» do menu —, e uma seta sem condição existe e nunca acontece. |

⚠️ **A razão de fundo é uma só:** *duas portas para a mesma pergunta divergem em silêncio*. Com o
botão a gerar `n(n-1)` e um arrasto a acrescentar à mão, a lista deixaria de ser derivável e a
próxima derivação apagaria o trabalho do arrasto.

### §5.5 — As provas de mutação (11, todas sangraram)

Arnês: verde-antes · a mutação **compila** · o gate **correu** (`running 1 test`) · restore por
escrita.

| mutação | gate que sangrou |
|---|---|
| `from != to` passa a aceitar o laço | `the_graph_covers_every_ordered_pair_in_both_directions` |
| `start` = a **última** forma | `the_first_shape_chosen_is_the_start` |
| os membros **não** se escondem | `the_set_owns_the_shapes_hides_them_and_shows_the_start` |
| o conjunto nasce a `t = 0,5` | `the_set_owns_the_shapes_hides_them_and_shows_the_start` |
| um Morph vira estado de outro conjunto | `a_morph_is_never_a_state_of_another_set` |
| o teto de formas deixa de valer | `one_shape_or_too_many_refuses_without_littering_the_scene` |
| ⭐ **a forma EXACTA da W4 de volta** (`ui_states_section` consulta o morph) | `a_morph_machine_never_makes_the_ui_states_section_appear` |
| a seção do Morph pintada **sempre** | `ui_poses_never_make_the_morph_states_section_appear` |
| o botão aparece com **uma** forma | `one_shape_offers_no_button_at_all` |
| o botão sai da allowlist do painel | `the_make_button_is_alive_and_reaches_the_bus` |
| o botão sai do `populate` (morto sob o ponteiro) | `the_make_button_is_alive_and_reaches_the_bus` |
