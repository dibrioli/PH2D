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

## §9 — Smoke

```
env PH2D_VEC_STACK_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

| na cena | o que prova |
|---|---|
| **A ETIQUETA** — uma estrela com contorno branco largo por baixo de um preto fino | a largura é POR CAMADA (o que o Figma não faz) |
| **O CARRIL** — uma linha com três contornos de larguras decrescentes | a pilha ordena-se, e o de cima desenha por último |
| **A MISTURA** — um disco com um 2.º preenchimento em `Multiply` a 60 % | opacidade e mistura são de CADA camada |
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
