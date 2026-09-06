# 46 — A forma tem N tintas (a pilha de aparência)

> **Estudo 42, item 4.** *"Cada camada de estilo exige HOJE duplicar o objeto. Mudança de data
> model; paga-se em todo o resto."*
> Wave de 2026-09-05, linha `line/Vector`.

---

## §1 — O que faltava, e o preço que ele cobrava

Uma forma tinha **um** `fill` e **um** `stroke`. Toda camada de estilo a mais — a etiqueta (um traço
branco largo por baixo de um preto fino), o carril de comboio, um segundo preenchimento em
`Multiply` — obrigava a **duplicar o objecto**.

⚠️ E duas cópias de uma forma são **duas geometrias**: o artista corrige um ponto numa e a outra fica
para trás. É a mesma classe de defeito que o `.svg` importado teria se cada camada virasse uma
forma, e é por isso que o manual deste repo chama a pilha de aparência de *"o fio que amarra tudo"*.

---

## §2 — ⭐ A metade que faltava de um mecanismo que já shipa

Esta struct **já tinha** a pilha da geometria: `effects: Vec<FxEntry>` (ADR-0132) — uma lista
ordenada de efeitos, cada um com o próprio interruptor. Isto é a pilha do **estilo**, com a mesma
forma e pelas mesmas razões.

> *O Inkscape separa geometria de estilo rigidamente, e o manual deste repo nomeia isso como a
> limitação a superar.*

`VecPath` ganha `paints: Vec<PaintEntry>` (v20), e o `PaintEntry` é o irmão do `FxEntry`: o que ela
pinta, se está ligada, e como se compõe.

---

## §3 — As três decisões, e de onde vieram

### §3.1 — INTERCALADA (uma lista, não duas)

Cada entrada é um preenchimento **ou** um contorno. ⚠️ É o que separa o modelo do **Illustrator** e
do **Rive** do do **Figma**: lá, as N tintas de contorno partilham **uma** geometria de traço, e dois
contornos de larguras diferentes — a etiqueta, o carril — são inexprimíveis sem duplicar a forma.
**É exactamente a lacuna que o estudo nomeia.**

### §3.2 — DE BAIXO PARA CIMA, com a inversão num sítio só

O índice `0` é a camada mais próxima da base e a primeira a desenhar. O painel mostra a pilha ao
contrário (o topo em cima, como o Illustrator), e essa inversão é **uma linha** (`n - 1 - k`) que não
se repete: com duas, o artista carrega em «subir» e a camada desce.

### §3.3 — A BASE É O CHÃO DA PILHA

O `fill` e o `stroke` continuam onde estavam e são as duas camadas mais baixas. ⛔ **Não há migração
escondida para a lista**, e não há duas fontes que possam discordar — a mesma relação que `verts` tem
com `subpaths`.

⚠️ **E a base não fica sem opacidade nem sem mistura: elas são as do OBJECTO** (v19, item 2 do mesmo
estudo). Isso não é assimetria — a camada de baixo compõe-se com o que está **atrás** da forma, e é
precisamente isso que a mistura do objecto quer dizer; as de cima compõem-se com o que está
**dentro**, e por isso as delas são da ENTRADA.

---

## §4 — O tecto, MEDIDO

| camadas | caminhos emitidos | recortes | encode (debug) |
|---|---|---|---|
| 4 | 14 | 8 | 20 µs |
| 16 | 50 | 32 | 72 µs |
| **32** | 98 | 64 | **167 µs** |
| 64 | 194 | 128 | 439 µs |

O custo é **linear** (~6,8 µs por camada em debug) ⇒ o encode **não é** o que trava: a `32`, UMA
forma custa `167 µs` de um quadro de `16 700` — **1,0 %**, sem optimização.

⇒ o recurso que manda é o **espaço de ids do painel**: cada linha tem cinco controlos, e resolver um
clique varre esse espaço (a lei que o `MAX_BLEND_MODES` já impõe). `MAX_PAINT_LAYERS = 32`, e as
artes de referência do Illustrator usam **2 a 5**.

---

## §5 — Onde a lei tinha de entrar, e os dois sítios que só uma medição acharia

1. **O afim** (`bake_xform`). ⚠️ Este ficheiro carrega **três cicatrizes** da mesma classe — a
   largura do traço fora da lista de comprimentos, a estampa do contorno que não seguia a forma, a
   estampa que seguia a lei errada — e as três eram *código que diz `fill` e devia dizer **tinta***.
   Uma pilha de N tintas é a quarta oportunidade: a cura é a lei de cada espécie viver numa função
   (`transform_paint` / `transform_stroke`) que o chão e as camadas **partilham**.
2. **A CAIXA da forma** (`inflate_for_stroke`). Ela dimensiona o scratch do FX e o rectângulo da
   camada de mistura, e lia `path.stroke`. ⇒ um traço extra de `20` sobre um de `2` era **recortado**
   — o mesmo sintoma da ponta CEIFADA que aquele módulo já documenta, uma tinta adiante.
3. **O re-cozimento** (`replace_cooked`). A pilha é autoria do OBJECTO, não produto dos parâmetros:
   sem sobreviver, **reescrever o texto de uma forma com dois contornos devolvia-a com um só**. O
   destructuring exaustivo do módulo obrigou a decidir — é para isso que ele existe.

---

## §6 — O que a pilha atravessa

- **O desenho** — uma porta, N camadas, reusando a MESMA tesselação (⇒ N emissões, **zero**
  construções de caminho). Uma camada neutra **não abre camada de composição**.
- **O SVG** — um `<path>` por camada, com `opacity` e `mix-blend-mode`. ⭐ O SVG exprime as duas, e a
  ida-e-volta continua fechada. ⛔ Um modo sem nome em CSS sai **NOMEADO**, nunca como `normal`
  calado.
- **O painel** — a seção *Appearance* lista as camadas do topo para o chão: olho, rótulo, cor,
  subir/descer/apagar; e a camada ABERTA mostra largura, opacidade e mistura.
- **O save** — `VEC_SCENE_SCHEMA_VERSION` 19 → 20, `PROJECT_SCHEMA` 115 → 116. Vazio custa **1 byte**
  de comprimento zero: uma cena que nunca lhe toque desenha byte a byte o que desenhava.

---

## §7 — ⛔ O que uma camada NÃO carrega, e é declarado

**Padrão e pincel.** A arte de um ladrilho e a de um pincel são memoizadas pela forma ANFITRIÃ
(`VecPathId -> …`), e uma camada não é uma forma. Uma camada com padrão desenha a **cor de recurso**
dele — a mesma resposta honesta que o chão dá enquanto o ladrilho não resolve — e a rota está
declarada no censo `the_artless_draw_routes_are_declared`.

⚠️⚠️ **E esse censo tinha um PONTO CEGO que esta wave expôs:** a lista de ficheiros dele é escrita à
mão, então o módulo NOVO (`stack_draw.rs`) passou **verde sem se declarar**, e o filtro de contagem
nem conhecia o nome da porta nova. *Um censo que enumera FICHEIROS não vê o ficheiro que ainda não
existe.* Os dois foram corrigidos no mesmo commit.

---

## §8 — ⭐⭐ O clippy achou DOIS controlos mortos

Depois de tudo ligado e verde, o `clippy` acusou duas funções *never used*:

| acusação | o que ela significava de facto |
|---|---|
| `paint_blend_option_index` | a lista de mistura de uma camada **pintava, registava os hit-rects e consumia o clique** — e o valor não chegava a consumidor nenhum |
| `set_color` | a swatch de uma camada abria o picker, o artista escolhia uma cor e **nada acontecia** |

⚠️ **É a espécie que o `CLAUDE.md` §5.0 chama de *dreno de UM BRAÇO SÓ*, e nenhum gate de registo a
vê** — o controlo está pintado, registado e alcançável; o que falta é o braço no `match`. *Aqui foi
o compilador que fez o papel do censo que não existe*, e vale registá-lo: **uma função de resolução
que ninguém chama é a assinatura barata de um controlo morto.**

---

## §8-bis — ⛔⛔ E o SMOKE achou o que o clippy não podia achar: a pilha INTEIRA estava muda

O 1.º smoke reprovou (Enio, 2026-09-05): *"O olho não funciona. Clicar no nome não exibe a largura,
a opacidade e a mistura. Demais botões não funcionam"*. Mecanismo completo no
[BUGS #29](BUGS_vector.md); o que interessa aqui é **por que a §8 não bastou**.

O `clippy` acha uma função que **ninguém chama**. As três rotas que faltavam não eram funções — eram
**uma linha ausente em três allowlists**:

| rota | a linha que faltava |
|---|---|
| clique | a família em `event_clicks::forwards_plain_click` |
| campo numérico | `VECTOR_PAINT_WIDTH` em `is_shell_owned_number` |
| slider | `VECTOR_PAINT_OPACITY` na lista de tracks |

Nada ficava *never used*: `stack_verb_for_id` e `apply` eram chamados pelo `render_loop`, com os
gates de unidade verdes. **A shell estava inteira; o painel é que não falava.**

⚠️ **E o `hit_indexed_ids_are_registered` estava verde COM RAZÃO** — os ids estavam todos
registados (a §5 tinha-o curado), logo os controlos eram focáveis, **acendiam sob o rato** e
consumiam o gesto. Entre esse gate e os testes de unidade da shell há um passo que **nenhum dos
dois** atravessa: *o clique sai do painel?*

⇒ Duas mudanças estruturais, e as duas são a mesma lei:

1. **As três rotas da família vivem num módulo** ([`event_paint_stack.rs`](../../crates/ph2d-panel-vector/src/event_paint_stack.rs)),
   à vista uma da outra. A pilha de EFEITOS tinha a mesma forma espalhada por dois ficheiros e foi
   cortada igual (`event_fx_stack.rs`).
2. **A família ganha um `seam_*` com gesto REAL** ([`seam_paint_stack.rs`](../../crates/ph2d-panel-vector/tests/seam_paint_stack.rs),
   7 gates), cujo oráculo é o `EditorAction` e nunca o `WidgetEvent`. ⚠️ Esta crate tinha **40**
   ficheiros `seam_*` e a pilha não tinha nenhum.

E a caça devolveu **mais dois** defeitos que o report não nomeia — o campo de largura mostrava o que
a última edição deixou no store (o `PaintRow::width` publicado **não tinha leitor**), e a swatch de
uma camada abria o picker no **cinzento de fábrica**, porque ninguém sincronizava o `widget_color`
dela. Mais um verbo **inalcançável por construção** (`StackVerb::Swatch`: uma *picker swatch* nunca
emite `Click`), removido.

---

## §8-ter — ⭐⭐⭐ ONDE cada camada desenha (v21), e por que ele é GRÁTIS

Pergunta do Enio, 2026-09-05, logo depois do smoke: *"o fill não deveria ter um offset? não seria
mais útil?"*

**Ele tem razão, e a medição diz porquê.** Sem deslocamento, dois preenchimentos ocupam os
**mesmos pixels**, e o de cima só se distingue do de baixo por cor, opacidade e mistura. As duas
coisas que um artista de facto faz com um segundo preenchimento — **a sombra dura** (o *offset
print*, o *long shadow*) e a **profundidade** de um rótulo — pedem que ele esteja noutro sítio, e
sem isso voltam a exigir **duplicar a forma**: exactamente a doença que o item 4 curou.

### A medição parte o pedido em DUAS coisas com preços muito diferentes

| | o que faz | custo por quadro |
|---|---|---|
| **deslocar** (v21, feito) | a mesma forma, noutro lugar | **zero** — a pilha inteira de `32` camadas encoda em `26,5 µs` de `16 700` (**0,16 %**), em release, e mover não acrescenta nada |
| **engordar** (⏳ aberto) | a silhueta cresce para fora — o adesivo | **`0,085`–`0,44 ms` por camada** (`ph2d-vec-boolean`, `probe_offset_cost_per_call`, release) ⇒ até **17×** a pilha inteira, por UMA camada |

⭐ O deslocamento é grátis porque **a geometria não muda**: a camada reusa a mesma tesselação e o
renderer só lhe passa outro afim — que ele já passava, em toda camada. O custo somado é uma
multiplicação de afins.

⚠️⚠️ **E esta tabela estava PELA METADE — a §8-quater corrige-a.** O motor tem DOIS caminhos, e o
barato (`offset_ring`, `~0,5 µs`) cobre exactamente *dilatar*; só *contrair* paga os `85–440 µs`.
A recusa acima adiou uma feature com o preço do caminho caro sobre um caso que usa o barato.

### ⚠️ A receita da sombra é INVERTIDA, e é uma consequência do §3.3

Uma camada extra desenha sempre **por cima** da base — a base é o chão da pilha, por desenho. ⇒ para
uma sombra **atrás**, o que se desloca é a *forma*, não a sombra: a BASE leva a cor da sombra e a
camada extra leva a cor viva, deslocada no sentido contrário. O olho lê um cartão claro com sombra
dura, e é o que a peça **A SOMBRA** do smoke ensina.

### As três leis que só a construção revelou

1. **O deslocamento é um VECTOR, não um ponto.** Ele passa pelo `bake_xform` (é LOCAL — a regra-mãe
   desta casa), e ali sofre só a parte **LINEAR**: apanhar a translação fá-lo-ia andar **duas**
   vezes, uma porque a geometria se moveu e outra porque o número se somou. ⚠️ **O caso de omissão
   não vê o defeito** — com uma pose que só escala as duas leis dão o mesmo número; o que as separa
   é uma pose com translação, que é o gesto mais comum do app. ⭐ A porta já existia com a lei
   escrita nela (`Xform::apply_vec`: *"transladar um delta o transformaria em ponto — o erro
   clássico"*).
2. **A ordem do afim é `transform ∘ translate`, e o neutro também não a testa.** Com `offset =
   [0,0]` as duas ordens dão o mesmo afim; o que as separa é uma forma **rodada** — na ordem errada
   a sombra anda no eixo do ECRÃ enquanto a forma roda por baixo dela.
3. **A CAIXA da forma tem de cobrir a camada deslocada**, senão ela é **recortada** — a terceira vez
   que o `inflate_for_stroke` paga a mesma conta (a ponta ceifada do miter, o traço mais gordo da
   pilha, e agora isto). A inflação é simétrica de propósito: um deslocamento tem direcção e uma
   caixa não, e a troca só tem um lado (uma caixa apertada CORTA; uma folgada custa memória).

⛔⛔ **E o exportador SVG quase pagou a lei do eixo pela QUINTA vez.** A 1.ª redacção multiplicava o
deslocamento por `EXPORT_PIXELS_PER_UNIT` e invertia o `y` — sobre um valor que **já** tinha sofrido
as duas coisas, porque ele viaja com a geometria pelos dois `bake_xform`. Ele sai **cru**, e isso é a
prova de que o sítio dele é o assador. *Uma lei escrita em dois sítios ainda não é uma lei — só uma
PORTA é*, que é o que o cabeçalho daquele ficheiro já dizia.

`VEC_SCENE_SCHEMA_VERSION` **20 → 21** · `PROJECT_SCHEMA` **116 → 117** (a tripla dos gates junto).

---

## §8-quater — ⭐⭐⭐ O OFFSET DE CAD: a silhueta de uma camada CONTRAI e DILATA (v22)

*"não era esse tipo de offset, mas sim o offset do cad, contraindo e dilatando"* — Enio,
2026-09-05, sobre o §8-ter.

Ele estava a pedir o **Offset Path** aplicado a UM atributo, e eu construí o deslocamento. As duas
coexistem porque fazem coisas diferentes, e o painel chama-lhes `X`/`Y` e **Offset** — os nomes que
o artista já conhece.

### ⚠️ A tabela de custos que eu dei estava PELA METADE

A 1.ª resposta disse *«engordar custa 0,085–0,44 ms por camada»* e adiou a feature com isso. **O
motor tem DOIS caminhos**, e o barato cobre exactamente a direcção que ele nomeou primeiro:

| direcção | motor | custo por camada | domínio |
|---|---|---|---|
| **dilatar** | `offset_ring` — a dilatação de Minkowski, sem booleana | **~0,5 µs** | contorno único, `d > 0` |
| **contrair** | `offset_path` — o sweep booleano | **85–440 µs** | tudo o resto |

Tesselar uma forma custa **~0,13 µs** ⇒ dilatar é da ordem do desenho, e contrair é **~1 000×**
isso. *Uma recusa medida responde UMA pergunta, e a minha respondeu à do caminho caro sobre uma
feature que usa o barato na metade dos casos.*

⇒ O memo existe pelo **contrair**, e serve os dois (uma porta).

### O desenho, e as cinco decisões

1. **A distância é LOCAL, e não MUNDO.** O `contour_live` coze em mundo e por isso **recoze a cada
   quadro** enquanto a forma é arrastada — o próprio ficheiro dele declara sofrer disso. Local dá
   duas coisas: o contorno **engrossa com a forma** (a lei que o traço já segue, bug #27) e a chave
   do memo **sobrevive ao arrasto**. No `bake_xform` ele escala por `√|det|`, a **mesma** conta da
   largura do traço.
2. **O cozimento passa pela porta do Contour** (`contour_live::cook_piece`), que já escolhe entre o
   anel e a booleana com o domínio de cada um medido. ⛔ Uma segunda escolha divergiria dela no dia
   em que o domínio do anel crescesse.
3. **A shell entrega o CAMINHO, o renderer tessela.** O caro fica memoizado; os `~0,13 µs` de
   tesselar correm no quadro, como em qualquer forma. ⭐ É essa divisão que faz um **contorno**
   dilatado passar pela mesma porta do de base, com o tracejado re-ajustado ao comprimento novo de
   graça.
4. **Um offset pode PARTIR a forma** — encolher um haltere para além do pescoço dá duas ilhas, e a
   sonda `probe_offset_as_effect` mediu **8** no pior caso. As peças fundem-se num composto
   `EvenOdd`, que é legítimo porque tudo o que sai do motor está regularizado.
5. **A QUINA só é pintada com o offset armado** — com `dilate = 0` não há esquina a formar, e três
   chips que não mudam nada são um controlo morto sob o dedo.

### ⛔⛔ E a 1.ª versão ARREDONDAVA as quinas — a lei de um motor aplicada à saída do outro

Report do Enio, 2026-09-06: *"o offset não obedece as quinas (arredonda as quinas)"*. Mecanismo
completo no [BUGS #30](BUGS_vector.md); o que interessa aqui é a forma do erro.

O `merge` — que junta as peças quando um offset PARTE a forma — carimbava `EvenOdd` em **toda**
peça. Mas essa lei é do **sweep** (a sonda `probe_offset_as_effect` validou-a sobre a saída dele,
*«tudo o que sai do sweep está regularizado»*), e o **anel** devolve `NonZero` **de propósito**: o
laço dele pode ser auto-cruzado, e é o NonZero que preenche a auto-interseção da ponta de uma
`Miter`. Sob `EvenOdd` essa ponta **cancela-se contra si mesma**.

⚠️ **Os dois motores vivem atrás da MESMA porta** (`cook_piece`), e é isso que torna a troca
invisível a quem lê o chamador — *a decisão «qual motor» é dela; a decisão «qual regra de
preenchimento» também tinha de ser.*

⚠️ **E o smoke não o mostrava:** a peça da cena **encolhe**, e encolher sai pelo caminho booleano,
onde o `EvenOdd` estava certo. O defeito vive só no **crescer**.

⛔ **O gate que já existia passava**, e a razão é a mesma família de sempre: ele mede o ALCANCE dos
vértices (geometria), e o que se perdia era o **preenchimento**. *Duas grandezas estavam a ser
lidas como uma.*

### ⚠️ A receita do adesivo também é INVERTIDA

Uma camada extra desenha sempre **por cima** da base (§3.3), então uma que CRESCE tapa a forma
inteira. O adesivo faz-se ao contrário: a **base** é a borda branca e a camada de cima é a cor viva
**encolhida**. É a mesma inversão da SOMBRA, e a peça **O ADESIVO** do smoke ensina-a.

### Três coisas que a construção corrigiu em mim

- **O piso do offset.** Escrevi `MIN_DILATE = 1e-6` sem olhar; o gate
  `the_dilate_floor_matches_the_engine` reprovou **na primeira corrida** e o número certo é `1e-9`
  (o piso da tolerância de achatamento do motor). ⚠️ Ele vive **do lado do motor** porque o
  documento é uma crate-FOLHA — depender do booleano puxaria o `linesweeper` para dentro do tipo que
  o save serializa.
- **A faixa do campo.** A do deslocamento nasceu emprestada da **largura do traço**; a lei da casa
  para uma coordenada de mundo (*"world coords span any magnitude"*) estava num **comentário dentro
  de um laço**, e hoje é a porta `world_number_field`.
- **O 8.º argumento.** Threading a geometria pela cadeia cruzou o tecto do `clippy`. ⛔ Silenciar
  seria armengo; os quatro artefactos derivados passaram a viajar num pacote (`Derived`), o que
  **reduz** a assinatura de 8 para 5 e torna impossível trocar dois `None` de posição.

`VEC_SCENE_SCHEMA_VERSION` **21 → 22** · `PROJECT_SCHEMA` **117 → 118**.

---

## §9 — Smoke

```
env PH2D_VEC_STACK_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

| na cena | o que prova |
|---|---|
| **A ETIQUETA** — uma estrela com contorno branco largo por baixo de um preto fino | a largura é POR CAMADA (o que o Figma não faz) |
| **O CARRIL** — uma linha com três contornos de larguras decrescentes | a pilha ordena-se, e o de cima desenha por último |
| **A MISTURA** — um disco com um 2.º preenchimento em `Multiply` a 60 % | opacidade e mistura são de CADA camada |
| **A SOMBRA** — um cartão com o 2.º preenchimento DESLOCADO (v21) | ONDE cada camada desenha é da CAMADA; ⚠️ a receita é invertida (§8-ter) |
| **O ADESIVO** — uma estrela branca cujo 2.º preenchimento ENCOLHE (v22) | o *offset de CAD*: a silhueta de UMA camada contrai/dilata; ⚠️ a receita também é invertida (§8-quater) |
| o par à direita | a mesma etiqueta à moda antiga: **duas** formas empilhadas |

---

## §10 — O que fica ABERTO

1. **Padrão e pincel numa camada** (§7) — a memoização por forma anfitriã é a wave que o censo já
   nomeia para as outras três rotas.
2. **Arrastar para reordenar** — hoje são dois botões (▲▼). O arrasto pede um gesto de lista que
   este painel ainda não tem em lado nenhum.
3. **Um gradiente numa camada** existe no documento e no desenho, e o painel só edita **cor sólida**
   ali (a swatch mostra uma cor, e escrever uma cor é o que ela promete).
4. **Efeitos por atributo** (o *Appearance panel* do Illustrator põe um efeito em UMA camada) — a
   pilha de efeitos é do caminho inteiro, e cruzá-las é modelo novo.
5. ✅ **O OFFSET DE CAD FECHOU** (v22, §8-quater) — a silhueta de uma camada contrai e dilata.
   ⏳ Fica aberto o **miter limit**: a quina `Miter` usa o limite de fábrica do motor, e o
   Illustrator expõe-no no diálogo dele. Nenhum report pediu, e o número não foi medido.
6. ⏳ **Escalar e rodar uma camada.** O afim por camada já é o mecanismo (o `camada_xf` compõe um
   `Affine`), então isto é painel, não motor. ⛔ Não foi feito porque **não foi pedido**, e escalar
   uma forma comprida não a engorda por igual — quem quer o adesivo quer o item 5, e oferecer a
   escala com esse nome ensinaria a coisa errada.
