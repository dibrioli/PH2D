# 95 — Estudo: ramificação CONTÍNUA e instâncias por letra/fase no `source.lsystem`

> História verbatim: [arquivo](../archive/motion-nodes-2026-10-12/95_estudo_ramificacao_continua_e_instancias.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| Emitir a fita como quads no próprio interpretador | Separa-se esqueleto de superfície nas quatro referências; o varrimento serve também os cinco `rig.*` | vivo, tabela «Recusas MEDIDAS (deste estudo)» |
| Portas de geometria `J`/`K`/`M` à la Houdini | O nó tem `inputs: &[]` e a casa nomeia objectos por nome; portas seriam um segundo idioma | vivo, tabela «deste estudo» |
| Terminar a recursão para não acumular | Medido: muda a planta — `64` elementos em vez de `256` a `g = 8` | vivo, tabela «deste estudo» |
| Usar o `falloff` para livrar as folhas do tint | Partiu a planta: o `falloff` é a máscara de todos os modificadores (`P' = P + (dx,dy)·falloff`) | vivo §8 (história no arquivo) e tabela «deste estudo» |
| Desenhar o vector na passagem HDR das sprites | Re-medido no `vello` 0.10: `render_to_texture` exige `Rgba8Unorm`; o alvo HDR está fora do alcance | vivo, tabela «deste estudo» |
| «Curar» a oscilação do Dragon | Medido: a motricidade é contínua (o degrau encolhe ~4× quando a amostragem afina 4×) | vivo §12.2 |
| Recusa viva sobre trabalho já feito (âncora no `DEFAULT_RULES`; esconder `Tropism Angle`) | Revogadas: a premissa estava errada no dia em que foram escritas | vivo §14.3 e §15 |

> **Dois reports do Enio (2026-08-30):**
> 1. *"as formas crescem sempre separadas e não crescem como um objeto só. Exemplo: o tronco de
>    árvore. O tronco deve ter uma estrutura única e não vários retângulos soltos sobrepostos."*
> 2. *"não há modo de escolher objetos para as fases de crescimento. Exemplo: as folhas.
>    Deveríamos ter um modo de escolher o objeto que será exposto (instanciado) em cada fase."*
>
> **Estudo, não plano.** O que se constrói e em que ordem é decisão dele. Aqui está o que o
> nosso código faz hoje (medido), o que as referências fazem (pesquisado, com fonte), onde
> exactamente está o buraco de cada report, e o que tem de ser MEDIDO antes de desenhar.

---

## §1 — O que o nosso nó já faz (MEDIDO, 2026-08-30)

⭐ **O `source.lsystem` já emite um ESQUELETO, não uma nuvem de pontos.** Por elemento:

| coluna | o quê |
|---|---|
| `P` | posição |
| **`parent`** | o índice do elemento anterior no ramo (`-1` = raiz) |
| `len` · `rot` · `wrot` | comprimento, ângulo local, ângulo de mundo |
| **`size`** | a ESPESSURA da tartaruga (grossa no tronco, fina no galho — o `!` da gramática) |
| `depth` | quão fundo no aninhamento de `[ ]` |
| **`gen`** | **em que geração este elemento nasceu** |
| **`sym`** | **que letra o desenhou** (`F` tronco · `J`/`K`/`M` folha/flor/instância) |
| `Index` · `Count` | os de sempre |

⚠️ **É exactamente o contrato de colunas do `rig.skeleton`** — o doc do `turtle.rs` diz-se assim,
e há gate a provar que `source.lsystem → rig.fk` é a identidade ao bit.

E as letras de instância **já existem**: o alfabeto declara `J K M` = *"pousa um elemento **sem**
segmento (folha, flor, instância)"*, e o interpretador emite-as com o `sym` preenchido.

⛔ **O que NÃO existe, e é o buraco dos dois reports:**

| medição | resultado |
|---|---|
| Há um passo que transforma a cadeia `parent` numa forma contínua? | ⛔ **Não.** Cada elemento é carimbado como um quad próprio ⇒ **N retângulos sobrepostos** por ramo. É o report nº 1, literalmente |
| O `source.lsystem` tem portas de entrada? | ⛔ **`inputs: &[]`** — zero. Não há onde ligar a folha |
| O `motion.cull` filtra por um atributo qualquer? | ⛔ **Não** — só *Fraction* e *Falloff*. Separar `sym == 'J'` pede `value.attribute` + limiar + `motion.falloff` + `cull` + `duplicator` + `source.object` + `combine`, **e saber que `J` é 74 em ASCII** |

⇒ O report nº 2 é o caso canónico de ***exprimível não é alcançável*** (a mesma frase que abriu o
`value.number`, citada no [doc 92](92_o_que_o_mini_cavalry_tem_e_nos_nao.md)).

---

## §2 — O que as referências fazem: **a resposta é unânime**

> **Um ramo não é uma sequência de segmentos desenhados. Um ramo é uma CURVA (o eixo) com uma
> FUNÇÃO DE RAIO ao longo dela, e a geometria é esse perfil VARRIDO pelo eixo — uma superfície
> contínua.** Quatro referências independentes, quatro vocabulários, uma só lei.

| referência | o esqueleto | a superfície | a lei do raio | a JUNÇÃO |
|---|---|---|---|---|
| **cpfg** — o interpretador dos autores do ABOP, que é a **nossa** referência | pontos de controlo postos pela tartaruga: `@Gs` (primeiro) · `@Gc(n)` (do meio) · `@Ge(n)` (último). O eixo é uma **curva de Hermite**, e a **tangente de cada ponto é o heading da tartaruga** | *generalized cylinder* | **`@Gr(ângulo1, comprimento1, ângulo2, comprimento2)`** — o perfil de raio entre dois pontos de controlo, dado pela inclinação e comprimento das tangentes ⇒ afinamento **não-linear** | `[` guarda *"o último ponto de controlo antes do ramo"*, e `]` repõe-no: **o filho LIGA-SE a esse ponto** |
| **Houdini** *L-System SOP* | emite **polilinha** (skeleton) com atributo de largura (`width` / `pscale`) | **um nó SEPARADO**: *PolyWire* (rápido) ou *Sweep* (com curva de secção) | o atributo por-ponto | herdada do esqueleto |
| **SpeedTree** *Spine Generator* | **spine** = spline, com segmentação **adaptativa pela curvatura** | secção radial varrida; **tampas** nas pontas abertas | *radius profile*, com o número de segmentos da secção configurável | ⭐ **o raio do filho nunca excede o do pai no ponto onde nasceu** (restrição explícita) |
| **Blender** *Sapling Tree Gen* | curvas de Bézier | **bevel** (a secção) + **taper** | `Branch Radius Ratio` · `Split Radius Ratio` · `Auto Taper` · `Minimum Radius`, **por nível** | a razão de raio por nível |

### ⭐⭐ As duas coisas que TODAS têm e nós não

1. **O VARRIMENTO é uma etapa própria.** Nenhuma delas desenha no interpretador. Houdini é o
   caso mais explícito: o L-System SOP devolve *linhas*, e quem faz o tubo é outro nó.
   *O interpretador produz o esqueleto; a superfície é uma segunda pergunta.*
2. **Uma LEI DE RAIO NA JUNÇÃO.** SpeedTree crava-a (o filho nunca mais grosso que o pai ali),
   Blender dá-lhe um knob por nível, cpfg fá-la contínua por construção (o filho começa no
   ponto de controlo do pai). Sem ela, um galho pode nascer mais grosso que o tronco que o
   segura — que é meio caminho para *"retângulos soltos"* mesmo com varrimento.

### ⚠️ E a resposta ao report nº 2 é a MESMA em todas: **a letra escolhe o objeto**

O *L-System SOP* do Houdini tem **três entradas de geometria** — e elas chamam-se, literalmente,
**`J`, `K` e `M`**: *"any geometry connected to these inputs is created in the sequence by these
letters"*. O exemplo da documentação é exactamente o do Enio: uma regra de tronco que chama um
`J` para a folha.

⭐⭐⭐ **É o nosso alfabeto, letra por letra** — o `J K M` já está implementado aqui, com o mesmo
significado, e vindo da mesma fonte (o ABOP). *A metade que falta é só a porta que diz **qual**
objecto cada letra pousa.*

⚠️ **E «por fase de crescimento» resolve-se pela GRAMÁTICA, não por uma tabela de fases.** É por
isso que nenhuma das quatro tem um controlo *«objeto da fase N»*: a regra que dispara na geração
`N` é que emite a letra, e a letra escolhe o objecto. O Houdini expõe a geração como `@gens`
para quem quiser filtrar — e **nós já emitimos `gen` por elemento**. Uma folha *«só na geração
mais nova»* é um filtro sobre `gen`; uma folha *«onde a regra mandou»* é a letra. As duas
perguntas têm resposta, e são perguntas diferentes.

---

## §3 — Os dois buracos, nomeados

### Buraco A — falta o VARRIMENTO (report nº 1)

O esqueleto está certo e é o mesmo das referências. Falta a etapa que o torna superfície.

⚠️ **É o MESMO buraco que o [doc 92](92_o_que_o_mini_cavalry_tem_e_nos_nao.md) já tinha achado
pelo outro lado** — o item 8 da §2, `skeletonRender`: *"sem ele um esqueleto é invisível, o que
torna os cinco `rig.*` difíceis de autorar"*. O L-System e o rig partilham o contrato de colunas,
logo **partilham a cura**: um varrimento que leia `parent`/`P`/`size` serve os dois.

⭐ **Em 2D o varrimento é MUITO mais barato que em 3D:** uma fita afinada é um **traço com perfil
de largura**, e o módulo vetorial já tem *live width profile*
([ADR-0148](../architecture/decisions/0148-vector-live-width-profile-is-an-ecs-component-and-one-baker-serves-preview-and-apply.md))
e o motor de traço com junções e pontas.

⏳ **O que tem de ser MEDIDO antes de desenhar** (⛔ nenhuma destas se escolhe):
1. **Onde vive a fita** — um `VecPath` (ganha junções, pontas e o perfil de largura de graça,
   mas atravessa a fronteira Motion↔Vector) ou uma tira de triângulos no próprio stream (barato
   e residente, mas as junções passam a ser nossas). *A resposta muda o tamanho da wave.*
2. **Quantos ramos** uma planta típica tem (a cadeia `parent` parte-se em quantos caminhos?) — é
   o número que decide se uma fita por ramo é barata.
3. **A lei do raio na junção**: adoptar a de SpeedTree (o filho nunca excede o pai ali) muda o
   desenho de plantas já autoradas? Se mudar, é decisão de produto.
4. **A curvatura**: cpfg interpola o eixo com **Hermite** e o heading da tartaruga como tangente
   — um tronco fica CURVO em vez de poligonal. Medir o custo disso contra a polilinha crua.

### Buraco B — falta dizer QUAL objecto cada letra pousa (report nº 2)

⭐ **A forma da cura já está escolhida pela referência**, e a nossa casa tem o idioma:
o `fx.glow` guarda o **NOME** de um objecto da cena num text param e é a **shell** que o resolve
(a mesma porta do `motion.path`), e o painel já sabe desenhar essa linha — a `SourceRow`, com
chips dos nomes que a app publicou.

⇒ Três text params (*Leaf J* · *Leaf K* · *Leaf M*), cada um nomeando um objecto.
⛔ **Não** portas de entrada: `inputs: &[]` hoje, e o Houdini usa portas porque é um grafo de
geometria; aqui o nome já é a porta de toda a casa, e ela **é alcançável** (os chips existem).

⏳ **O que tem de ser MEDIDO antes de desenhar:**
1. **Quem escreve a aparência por elemento.** O `source.object` emite as colunas de aparência
   (`texture_id`, `uv_rect`, `size`, `tint`) para UM elemento. Aqui é preciso que **elementos
   diferentes do mesmo stream** tenham aparências diferentes — medir se o lowering suporta isso
   ou se hoje a aparência é uniforme por stream. *É esta medição que decide se o buraco B é uma
   wave pequena ou uma mudança de substrato.*
2. Se não suportar, a saída conhecida é a que a composição já faz: **separar por letra e voltar a
   juntar** (`cull` → `duplicator` → `combine`), com o nó a fazê-lo por dentro em vez de o
   artista o montar. Medir o custo dessa separação numa planta cheia.

---

## §4 — O que NÃO fazer

- ⛔ **Não desenhar a fita dentro do interpretador.** As quatro referências separam esqueleto de
  superfície, e a nossa também tem de o fazer: o mesmo varrimento serve o `rig.*`, e um
  interpretador que desenha não é reutilizável.
- ⛔ **Não inventar uma tabela «objeto por fase».** Nenhuma referência tem uma, e a razão é
  estrutural: a fase é a gramática, e o objecto é a letra. Uma tabela paralela daria duas
  respostas à mesma pergunta e a segunda envelheceria.
- ⛔ **Não resolver o nº 2 por composição de sete nós.** É exprimível hoje e ninguém lá chega —
  incluindo saber que `J` vale 74.
- ⛔ **Não escolher a lei do raio na junção.** Ela muda plantas já autoradas; é medição + decisão
  do dono.

---

## §8 — O 4.º smoke: *"as folhas não aparecem"* — e o `falloff` era o canal ERRADO

> *"Keep own color não funciona, as folhas não aparecem"* · *"Leaves in front não funciona, nada
> muda"*

⛔⛔ **A minha cura do tint estava errada de ESPÉCIE.** Eu tinha escrito `falloff = 0` nas linhas
de folha porque o `motion.tint` faz `lerp(existente, alvo, falloff)`. Mas o `falloff` é a máscara
de **todos** os modificadores desta casa — e o `motion.move` faz `P' = P + (dx, dy) · falloff`.

⇒ na cena `=108`, que move cada coluna com um `motion.move`, **as folhas ficavam paradas
enquanto a árvore andava**. Elas não desapareciam: ficavam noutro sítio, longe da planta a que
pertenciam. *O canal que escolhi era muito mais largo do que a pergunta que fiz.*

⭐ A cura é uma coluna **própria** — `attr::TINT_MASK_COLUMN`, multiplicativa com o `falloff` no
`motion.tint`, ausente ⇒ `1` ⇒ byte-idêntica. É o molde do `falloff_y` (que só o
`motion.falloff(Mask Channel)` escreve e só o `motion.scale(Separate Y Mask)` lê), mas declarada
**numa porta só**: dois lados a escrever a mesma string à mão são duas leis à espera de divergir.

⚠️⚠️ **E o gate que eu tinha NÃO O VIU, porque media a COLUNA e não a CONSEQUÊNCIA.** Ele
afirmava *«a máscara nasce com `0` nas folhas»* — o que era verdade — e nada sobre o que um nó a
jusante faz com ela. O gate novo coze um **`motion.move` a sério** e mede que as folhas andam
com a planta; a mutação que repõe o `falloff` mata-o.

### 8.1 — E o `First Level` esvaziava um molde inteiro

Medido: as `10` marcas do `Sprig` estão **todas na profundidade `1`** — ali o `J` vive num ramo
lateral de primeiro nível (`[+F(s*0.35)J]`) enquanto no `Tree` ele vive no eixo. Com o default
único de `3`, aquele molde ficava com **zero** folhas.

⇒ **o molde carrega o SEU número** (`Preset::leaf_first_level`), como já carregava ângulo,
gerações, passo e espessura. ⚠️ *A profundidade de encaixe significa coisas diferentes em
gramáticas diferentes, então um número só não a atravessa.*

⛔⛔ **E a mutação que repunha o `3` no `Sprig` SOBREVIVEU a toda a suíte** — não havia nada a
dizer que um molde com marcas tem de mostrar pelo menos uma. É o gate
`no_preset_silences_its_own_leaves`, com o controlo que impede o laço de passar vazio.

### 8.2 — O silêncio era metade da causa dos DOIS reports

Uma gramática cujas marcas vivem todas abaixo do nível mínimo fica sem folha nenhuma, e nada na
tela o diz — e sem folhas, o `Leaves In Front` também *"não muda nada"*. ⇒ o app **diz**
(`say_if_the_level_hid_every_leaf`), e cala nos três casos vizinhos para não virar ruído por
quadro.

---

## §10 — *"algumas aparecem já grandes"*: a IDENTIDADE de uma folha não é o índice dela

> Enio, 2026-08-30: *"ficou muito bom, mas nem todas as folhas crescem, algumas aparecem já
> grandes"*.

⭐ **A lei do crescimento estava certa** — medida: varrendo `g` de `1` a `6`, **zero** folhas
recebem um primeiro peso acima de `0,5`. Todas nascem pequenas.

⛔⛔ **O defeito era o SORTEIO, e a causa é a identidade.** O tamanho (e o lado frente/trás)
saíam do **índice da âncora na lista** — e ao crescer, a planta **insere marcas no MEIO** (a
travessia é em profundidade). ⇒ o índice de uma folha que já existia MUDA, ela recebe outro
número, e **salta de tamanho**. O mesmo valia para o `Leaves In Front`: as folhas trocavam de
lado sozinhas enquanto a planta crescia.

⭐ A cura é uma identidade **estável**: o par `(geração, ordinal dentro dela)`. E ele é estável
pela razão que faz a planta crescer — *as gerações velhas não se reescrevem, logo a ordem
relativa das marcas de uma geração não muda quando outra nasce*.

⚠️⚠️ **E a primeira sonda desta caça inventou o defeito ao medi-lo:** ela identificava a folha
pela POSIÇÃO e acusou **209 saltos de 420** — mas a marca da geração mais nova **move-se**
enquanto o ramo dela estica, então cada passo do varrimento contava «uma folha nova». Com a
identidade certa, `0` de `42`. *A régua que se usa para procurar um defeito pode fabricá-lo* —
e é a quarta vez nesta jornada que a régua mordeu antes do produto.

⚠️ O gate compara os retratos de `g = 4` e `g = 5` **só nas folhas cujas gerações já pararam de
crescer** (as que estão no mesmo sítio nos dois), com o controlo que impede a comparação de
passar vazia.

---

## §11 — *"vários dos presets não produzem folhas"*: a minha recusa estava errada

Enio, 2026-08-30. Eram **quatro de oito** — `Bush`, `Weed`, `Koch`, `Dragon` — por uma decisão
minha, registada neste doc como recusa medida: *«num refinador toda a silhueta renasce e as
folhas espalham-se pelo tronco; numa curva não há ponta»*.

⛔ **A medição desmentiu-a**, e o que a dissolveu foi trabalho meu de duas horas antes: o
`First Level` e a lei monótona da idade não existiam quando escrevi aquela recusa.

| molde | com o `[J]` no sítio certo |
|---|---|
| `Bush` · `Weed` | **121** marcas, profundidades `1..5`; a `First Level = 3` sobram `96` e nenhuma no caule |
| `Koch` | `156` marcas, **todas na profundidade 1** |
| `Dragon` | `512` marcas, todas na profundidade 1 |

⇒ os oito moldes passam a emitir. ⚠️ Nas curvas o `First Level` **não tem por onde
discriminar** (uma curva não tem tronco), então elas trazem `1` e mostram todas: *quem escreve
um nome numa curva quer decoração, e a resposta honesta a «não produz folhas» não é explicar
porquê — é produzir.*

### 11.1 — ⚠️ E a régua do empilhamento acusava a FIGURA, não a colocação

A curva do **Dragão toca-se a si própria** (é o que uma curva que ladrilha o plano faz): `2048`
marcas em `1324` sítios, sem nada empilhado por culpa da colocação. O gate reprovou.

⇒ a chave passa a incluir o **PAI**: o que ele acusa é o empilhamento que a COLOCAÇÃO faz —
duas marcas do mesmo pai no mesmo sítio, que foi o defeito de `F(s)![+A J][-A J]` — e não dois
ramos diferentes que a figura leva ao mesmo ponto. *Uma régua que não separa a geometria da
figura da colocação acusa a figura.* Provado: com o `J` antigo de volta, ela ainda diz
`62 marcas em 30 sítios`.

⚠️ **E não havia nada a afirmar que um molde produz folhas** — `no_preset_silences_its_own_leaves`
media só que o molde não se apagava a si próprio, e um molde SEM marcas passava por `continue`.
Hoje ele exige que os oito emitam, e o controlo conta os oito.

---

## §12 — *"em custom não funciona"* e *"falta fluidez no Dragon"*: um é defeito, o outro é a figura

### 12.1 — ⛔ Um nó tirado da PALETA não tinha onde plantar folha

O `Custom` é *"nenhum destes"* e **não escreve texto nenhum** — logo ele herda o que lá estava. E
o que lá está num nó recém-dropado é a gramática do modo **GUIADO**, derivada dos sliders… **sem
`J` nenhum**. A gramática de FÁBRICA (`DEFAULT_RULES`) também não o tinha.

⇒ escrever um nome em *Leaf (J)* naquele estado não plantava nada. *A feature era inalcançável
do estado em que o nó nasce* — e eu tinha deixado o `DEFAULT_RULES` de fora **de propósito**, em
2026-08-30 de manhã, para não pagar o preço em toda planta sem folha.

⚠️ **As duas mudam JUNTAS** — o gate `the_guided_plant_draws_exactly_what_the_factory_grammar_draws`
compara-as ao bit, e é essa comparação que prova que os sliders não são uma segunda planta
parecida.

⚠️ **O preço, medido:** `32 → 63` elementos a `g = 5` e `256 → 511` a `g = 8` (**~2×**), com o
relógio no ruído. A alternativa mediu-se no report.

⚠️ **E três gates da forma cravavam o texto antigo** — foram corrigidos **pela intenção**, não
pelo literal: *«um ramo não leva parênteses»* passa a proibir `[+`/`[-` (uma âncora fecha no
mesmo módulo e não empurra estado), e *«um par por rebento»* passa a descontar os `[J]`. *Um
gate que conta delimitadores tem de saber quais são os DELE.*

### 12.2 — ⭐ O Dragon: a motricidade é CONTÍNUA; o que oscila é a figura

Enio: *"ainda temos falta de total fluidez no crescimento de algumas plantas como dragon"*. Ele
tem razão sobre o que vê, e a medição diz o que é.

| medida | resultado |
|---|---|
| altura nas gerações INTEIRAS `1..10` | **monótona**, `0,02 → 0,59`, zero recuos ⇒ a figura cresce |
| maior degrau com `40` amostras | `0,2237` (13% da subida) — contra 3–5% em todos os outros moldes |
| com `160` amostras | `0,0655` |
| com `640` amostras | `0,0165` |

⭐⭐ **O degrau encolhe ~4× quando a amostragem afina 4×** — essa é a assinatura de **movimento
contínuo**. Uma descontinuidade **não encolhe**. ⇒ não há salto a curar: o que se vê é a curva a
**dobrar-se**, e a extensão dela oscilar enquanto as juntas abrem de `0°` a `90°` é o que um
dragão faz. Aos `640` pontos há `57` «recuos» — uma oscilação fina, não quatro trancos.

⛔ **Curá-la seria mentir sobre a figura.** O que a distingue dos outros moldes é a VELOCIDADE no
instante mais rápido (~3×), e isso é a mesma geometria.

---

## §13 — *"descubra para cada Preset quais os parâmetros que não são usados e esconda do painel"*

Report do Enio, 2026-08-31: *"Não quero parâmetros mortos"*.

### 13.1 — A régua é o PRODUTO, e um scanner de símbolos responde por 2 knobs de 29

O único gate por-molde deste género comparava a lista de visibilidade com `Reads::of`, que procura
`!` e `"` no **texto**. Os três que esta caça achou **não têm símbolo nenhum a denunciá-los** —
eles morrem por *como o `build` os lê*:

| knob | por que morre | onde vive |
|---|---|---|
| `Step Scale` | entra por `step · step_scale^g`, que alimenta o `Setup::step` — lido **só** por um `F` SEM parâmetro. Numa gramática paramétrica (`A(s)`) o comprimento viaja no módulo e sai da expressão, que lê o `step` **cru** | `lib.rs` |
| `Grow Length` | o braço do refinamento **nunca lê** o `want_len` | `lib.rs`, `grows_by_refining()` |
| `Grow Angle` | o braço da ponta faz `ang_frac = frac` **sempre** — a viragem contínua ali não é desligável, é lei | idem |

⇒ os dois interruptores do crescimento suave são **complementares por construção**, e o painel
mostrava sempre os dois. *Metade de um par exclusivo é um knob morto em todo molde, o tempo todo.*

A porta é `probe_param_prints` — quantas saídas **distintas ao bit** o nó emite ao varrer o param
pela faixa do próprio `ParamUiHint`. A bancada (`examples/dead_params_report.rs`) e o portão
chamam-na: *uma lei escrita em dois sítios ainda não é uma lei.*

### 13.2 — ⚠️⚠️ UMA leitura só FABRICA uma lista de dívida: 12 acusados, 9 reais

Um param cujo sujeito é criado por **outro** mede-se morto com os defaults. Cada célula passa a ser
medida **duas** vezes — o enquadramento do molde, e o contexto **aceso** (vizinhos ligados + geração
**fraccionária**, que é o estado em que o `Generations` está a ser animado):

| param | dorme em | acorda com | veredito |
|---|---|---|---|
| `tropism_angle` | **9 de 9** | `tropism` | morto de FÁBRICA ⇒ `ParamGateAbove` |
| `seed` | 8 de 9 | `leaf_spread` | vivo no `Wild` (gramática estocástica) ⇒ **isento** |
| `tropism` | 1 de 9 | `root_angle` | no `Sprig` a cadeia persistente é exactamente vertical, logo **anti-paralela** ao puxão ⇒ isento |

⇒ dos três que caíram, **esconder qualquer um teria apagado um controlo vivo.**

⛔ **E a cura do `seed` NÃO é um limiar sobre o `Leaf Spread`** — o único despertador que esta régua
vê. Ele é também semeado pelo `Leaf Size Jitter` e pelo `Leaf Pos Jitter`, que a **shell** lê e o
`build` não: um limiar sobre um dos três apagaria o knob para quem usasse os outros dois.
*Uma isenção medida é mais barata que um gate errado.*

### 13.3 — A terceira metade do portão, e por que a isenção sozinha era licença

1. toda lista de visibilidade bate com a medição (com o controlo do próprio filtro);
2. todo par (param, molde) inerte está **explicado**, e a explicação não é um nome numa lista — é
   uma afirmação que se volta a medir (o dependente tem de **acordar** na 2.ª leitura);
3. ⭐ **nenhum knob dorme no corpus INTEIRO sem que algo o esconda.** Sem esta, a isenção do
   dependente era uma licença — e era ela que deixava o `Tropism Direction` morto nos nove moldes.

⚠️ **O `Custom` nunca esconde nada, e agora é LEI com gate:** ali a gramática é a que o artista
escreveu, e esconder é adivinhar.

⚠️ `PRESETS_READING_STEP_SCALE` e `PRESETS_GROWING_BY_REFINEMENT` têm a **mesma tabela hoje** e são
duas constantes de propósito: as perguntas são outras (*«há um `F` sem parâmetro?»* contra *«refina
ou cresce pela ponta?»*), e um alias faria a segunda resposta seguir a primeira em silêncio.

**Medido:** o painel passa de **28 para 20** controlos em todo molde (`Custom`, 23).

---

## §14 — *"o feedback ao vivo de uma regra malformada"* (aberto desde 2026-08-29)

### 14.1 — A política de erro não muda; o silêncio é que sai

O que não se entende continua a descartar **aquela** regra e a deixar as outras vivas — recusar a
gramática inteira apagaria a planta enquanto o artista escreve a segunda regra, que é o estado
normal de quem autora.

⚠️ **A mais cara em silêncio é a CONDIÇÃO:** ela é o travão da recursão, e um `n <= 6` que não
compila dá **16 384** módulos onde `n < 6` dá **32**. A planta muda de forma e nada diz porquê.

### 14.2 — UM percurso, DUAS saídas — e por que não podiam ser duas funções

`parse_rules_reporting → (Vec<Rule>, Vec<Complaint>)`, com `parse_rules` a ser o `.0`. Cada
`return Err` está exactamente onde estava um `continue`.

⚠️⚠️ Um contador de queixas escrito **ao lado** do parser seria um segundo leitor do mesmo texto, e
este nó já pagou esse defeito: o gate dos pesos lia-os com um `str::parse::<f32>()` próprio e ficava
verde sobre uma soma que o motor nunca calculava.

⛔ **E não há `ParamRow::Note`:** uma variante nova custaria **104 sítios em 19 ficheiros**, incluindo
o painel de outro módulo, para servir um caso que vive onde há **texto livre** — e é a `TextRow` que
o tem. `TextRow.problem` custou **6** sítios de construção.

⚠️ A queixa **não entra no `HitIndex`** (um aviso não se clica), e é **uma** — a primeira, não a
lista: uma linha por regra empurraria o painel para fora do dock precisamente quando o artista tem
várias regras a meio.

⚠️ **Três gates, porque são três defeitos diferentes** — a queixa nascer certa, chegar à row, e
chegar a **pixel**. Sem o terceiro, um `match` que caísse no braço das irmãs deixava os outros dois
verdes e o artista sem aviso: *um controlo nunca pintado e um morto sob o dedo dão o mesmo report.*
A régua dele é a **altura publicada**, porque um aviso que não se clica não deixa marca no
hit-index.

### 14.3 — ⚠️⚠️ Uma recusa minha tinha a premissa errada no dia em que a escrevi

A tabela abaixo dizia, em **2026-08-30**, que esconder o `Tropism Angle` era inexprimível: *«o
`ParamGate` da casa compara com uma lista de INTEIROS e a condição é "diferente de zero" num slider
contínuo»*.

⛔ **O `ParamGateAbove` — que existe exactamente para essa pergunta — shipou em 2026-08-21**, nove
dias ANTES. A recusa não dissolveu com o tempo: ela **nunca foi verdade**, e eu escrevi-a sem
procurar o irmão do gate que estava a usar.

A segunda razão dela (*«a linha desapareceria no estado de fábrica»*) era uma opinião de produto, e
o dono revogou-a por escrito: *"Não quero parâmetros mortos"*. A terceira cláusula (*«a cura é o app
DIZER, e só quando há fio»*) **continua válida e não foi tocada** — ela é sobre um param **conduzido
por fio** que está inerte, que é outro defeito, com outro diagnóstico.

⇒ *Antes de declarar que uma pergunta é inexprimível nesta casa, procure o irmão do mecanismo que
está a usar* — o `ParamGate` tem três, e o doc-comment de cada um nomeia a pergunta que os outros
não sabem fazer.

---

## §15 — Uma recusa VIVA sobre trabalho já feito custa uma janela inteira

Auditoria de seis lentes, doc 96 §5.4.

O `CLAUDE.md` §5.0 manda **consultar a tabela de recusas antes de propor qualquer mudança de
desenho**. É a regra que impede refazer trabalho já pago — e ela tem um custo simétrico que
ninguém tinha nomeado: **uma recusa que descreve trabalho já FEITO faz a próxima janela
recusar-se a fazer o que já está no `main`.**

A entrada *«Âncora no `DEFAULT_RULES`»* estava viva, sem rasura, e a revogação dela estava **duas
linhas abaixo, na mesma tabela** (*«Deixar o `DEFAULT_RULES` sem âncora… ⛔ Revertido no mesmo
dia»*). As duas descreviam o mesmo facto em sentidos opostos — e **discordavam também sobre o
preço** (`~3×` contra `~2×`; a medição diz **2×**).

⚠️ **A mesma recusa estava escrita uma TERCEIRA vez, também viva**, no §5.1 deste documento.

⇒ **quem reverte uma recusa rasura-a no mesmo commit.** O precedente de formatação já existia
nesta tabela — a do `Tropism Angle`, rasurada em [§14.3](#143--uma-recusa-minha-tinha-a-premissa-errada-no-dia-em-que-a-escrevi) —, e foi por isso que
esta se lê como um esquecimento e não como uma decisão.

---

## ⛔ Recusas MEDIDAS (deste estudo)

| Item | Motivo |
|---|---|
| Emitir a fita como quads no próprio interpretador | Separa-se esqueleto de superfície nas quatro referências; e o varrimento serve também os cinco `rig.*` (doc 92 §2 item 8) |
| Portas de geometria `J`/`K`/`M` à la Houdini | O nó tem `inputs: &[]` e a casa nomeia objectos por **nome** (`fx.glow`, `motion.path`), com chips já alcançáveis no painel. Portas seriam um segundo idioma |
| Tabela de «objeto por fase de crescimento» | A fase é a regra que dispara; a letra é o objecto. Nenhuma das quatro referências tem tabela de fase — e nós já emitimos `gen` para quem quiser filtrar |
| ~~Âncora no `DEFAULT_RULES`~~ | ⛔⛔ **REVOGADA — ela descreve trabalho JÁ FEITO**, e a revogação estava duas linhas abaixo, por rasurar. Ver [§15](#15--uma-recusa-viva-sobre-trabalho-já-feito-custa-uma-janela-inteira) |
| Terminar a recursão para não acumular (`A(s) : s <= k -> F(s)J`) | Medido: **muda a planta** — `64` elementos em vez de `256` a `g = 8`. Não é a mesma planta com folhas |
| O cruza-fade entre duas gerações (peso `f` / `1 − f`) | Comprava «só as pontas» numa planta parada e fazia **cada folha encolher até sumir** durante o crescimento — *«só as pontas» e «uma folha não encolhe» não cabem na mesma lei, porque uma ponta vira interior*. Veredito do Enio: *«bem bizarro»* |
| ~~Esconder o `Tropism Angle` quando o `Tropism` é `0`~~ | ⛔⛔ **REVOGADA em 2026-08-31 — a premissa técnica estava ERRADA quando foi escrita.** Ver [§14.3](#143--uma-recusa-minha-tinha-a-premissa-errada-no-dia-em-que-a-escrevi) |
| Usar o `falloff` para livrar as folhas do tint | ⛔ **PARTIU a planta**: o `falloff` é a máscara de TODOS os modificadores, e o `motion.move` faz `P' = P + (dx, dy) · falloff` ⇒ as folhas ficavam paradas enquanto a árvore andava. A cura é uma coluna própria (`attr::TINT_MASK_COLUMN`) |
| Um `First Level` único para todos os moldes | ⛔ **Esvaziava o `Sprig`** (10 marcas, todas na profundidade 1). O molde carrega o seu, como os outros quatro números de enquadramento |
| Folhas da frente como imagem na cena Vello | ⛔ Poria metade da copa DEPOIS do tonemap e metade antes — as duas metades da mesma folhagem com cores diferentes |
| Desenhar o vector do documento na passagem HDR das sprites | ⛔ **Re-medido no `vello` 0.10** (a nota citava o 0.8): o `render_to_texture` exige `Rgba8Unorm`; o alvo HDR está fora do alcance da biblioteca |
| «Curar» a oscilação do Dragon | ⛔ Medido: a motricidade é CONTÍNUA (o degrau encolhe ~4× quando a amostragem afina 4×, que é a assinatura de movimento e não de salto), e as gerações inteiras são monótonas. O que oscila é a EXTENSÃO da figura enquanto as juntas abrem — curá-la seria mentir sobre a curva |
| Deixar o `DEFAULT_RULES` sem âncora para poupar ~2× elementos | ⛔ Revertido no mesmo dia: a poupança tornava a feature **inalcançável do estado em que o nó nasce** (*"em custom não funciona"*). ⚠️ É esta que manda, e o preço certo é **~2×** — não os `~3×` da linha revogada acima |
| Filtrar `sym` com `motion.cull` | Ele só faz *Fraction* e *Falloff*; a rota por atributo pede 6-7 nós e o código ASCII da letra |

---

## Fontes

- [L-System geometry node — SideFX Houdini docs](https://www.sidefx.com/docs/houdini/nodes/sop/lsystem.html)
- [PolyWire geometry node — SideFX](https://www.sidefx.com/docs/houdini/nodes/sop/polywire.html)
- [Generalized Cylinders — cpfg 3.0 tutorial, Algorithmic Botany](https://algorithmicbotany.org/cpfg3.0-tutorial/cylinders.html)
- [Spine generator — SpeedTree documentation](https://docs.speedtree.com/doku.php?id=spine_generator)
- [Sapling Tree Gen — Blender manual](https://docs.blender.org/manual/en/4.1/addons/add_curve/sapling.html)
- [How to create L-Systems — Houdini Kitchen](https://www.houdinikitchen.net/2019/12/21/how-to-create-l-systems/)
