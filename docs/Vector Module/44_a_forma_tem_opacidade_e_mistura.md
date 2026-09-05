# 44 — A forma tem OPACIDADE e MISTURA próprias

> **Wave de 2026-09-05** · `line/Vector` · item **2** do [estudo 42](42_o_que_falta_ao_vetor.md)
> (*"Blend modes + opacidade por forma — a correção visual mais barata, e a mais visível numa cena
> montada"*), e a metade que faltava ao 2.º report do Enio de 04/09
> ([doc 43 §7.4](43_a_timeline_alcanca_o_vetor.md)).

---

## §0 — O que existia, medido

| Pergunta | Antes | Onde se lia |
|---|---|---|
| A forma tem opacidade própria? | **Não** | o painel tem duas rows `Opacity` e as duas são o **alfa da tinta da ferramenta** — elas semeiam a forma seguinte e re-vestem a selecção |
| A forma tem modo de mistura? | **Não** | `grep blend_mode` nas crates `ph2d-vec-*` = **vazio** (estudo 42 §2, linha 90) |
| Havia um vocabulário de mistura no app? | **Sim, dois** | `ph2d_painter_effects::BlendMode` (22 modos do W3C, camada do Painter) e `ph2d_ecs::BlendMode` (6, **estado de pipeline** de sprite, empacotado nos bits do `flip_uv`) |

⚠️ **A opacidade que existia era de VISTA, e escalava a TINTA** (`BoundStyle::alpha`, plano UI/UX
W8b.3): o único desvanecimento possível era multiplicar o alfa de cada cor da forma.

---

## §1 — O estado da arte, e o que a palavra significa

**Illustrator** (painel *Transparency*), **Figma** (fileira de baixo do *Fill*), **Rive** e o
**SVG 2 / CSS** (`opacity` + `mix-blend-mode`) têm todos a MESMA distinção, e ela não é cosmética:

- **opacidade da TINTA** — o alfa de um preenchimento ou de um traço. Descreve **uma marca**.
- **opacidade do OBJECTO** — a forma inteira compõe-se **uma vez** e o resultado desvanece.

⭐ **Onde a diferença se vê:** uma forma com preenchimento **e** traço, a 50 %. Com o alfa na tinta,
o traço transparece **sobre o próprio preenchimento** (duas marcas, cada uma a meio); com a
opacidade do objecto, a forma compõe-se e só então desvanece. *É por isso que os dois controlos
convivem em todo editor sério, e não são o mesmo número com dois nomes.*

⇒ A opacidade do objecto é uma **CAMADA** no desenho, nunca uma multiplicação nas cores. O mesmo
mecanismo entrega o modo de mistura de graça: `push_layer(blend, alpha)`.

---

## §2 — O desenho

### §2.1 — O documento (v19 do `VEC_SCENE_SCHEMA`)

```rust
pub struct VecPath {
    …
    pub opacity: Opacity,                    // v19
    pub blend: ph2d_blend_mode::BlendMode,   // v19
}
```

⚠️⚠️ **`Opacity` é um NEWTYPE por causa do `Default`.** A `VecPath` deriva-o, `..VecPath::default()`
é o idioma de centenas de sítios deste repo, e um `f32` cru nasceria a **`0.0`** — *toda forma do
app invisível, sem uma linha de erro*. O tipo carrega o neutro **e** a cerca (`0..=1`), e ⚠️ um
`NaN` vira **opaco** em vez de `NaN` (um `f32::clamp` com `NaN` devolve `NaN`, e um `NaN` no
`push_layer` do Vello é uma camada com alfa indefinido — o modo de falha silencioso que o tipo
existe para não ter).

⭐ **O destructuring exaustivo do `replace_cooked` fez a pergunta certa no commit certo:** a wave
não compilou até alguém responder *isto é produto do re-cozimento, ou sobrevive a ele?*. Resposta:
**sobrevive** — um cozimento nasce de um `VecPath::default()`, e lê-las dali repunha o neutro. O
sintoma seria **reescrever o texto de uma forma a 40 % devolvê-la opaca**.

### §2.2 — A composição tem UMA porta

```rust
pub fn object_alpha(path: &VecPath, bound: Option<&BoundStyle>) -> f32
```

⭐⭐⭐ **O `BoundStyle::alpha` passou a ser um OVERRIDE, e isso mudou uma lei.** Até aqui ele
*escalava* a tinta (era a única forma de desvanecer que existia). Com o objecto a ter a sua, os
**quatro** campos daquela struct passam a ser a mesma lei — *`None` = o literal do documento vale;
`Some` = o valor vivo cobre-o* —, que é o que o doc dela sempre disse sobre `fill`/`stroke`/`width`.

⛔ **Multiplicar em vez de sobrepor foi recusado:** uma forma autorada a `0,5` com um estado de UI a
pedir `1,0` ficaria a meio — *o controlo no topo da barra deixaria de significar «opaca»*.

### §2.3 — O desenho

`dispatch` abre uma camada por objecto **quando ela é observável** e não a abre quando não é:

| Estado | Camada? |
|---|---|
| opaco + `Normal` | **não** — byte-idêntico a todo documento que nunca abriu a seção |
| `alpha < 1` ou modo ≠ `Normal` | sim, `push_layer(blend, alpha, rect)` |

⚠️ **A camada envolve os TRÊS braços** (o desenho vectorial, a imagem de FX e a pele de widget): os
dois últimos SUBSTITUEM o desenho mas continuam a ser **este objecto**, e desvanecer só o braço do
meio seria a opacidade a funcionar até alguém ligar um filtro.

⚠️ **O rectângulo da camada é o da ARTE, e sai da mesma porta que dimensiona o scratch do FX**
(`path_screen_bounds`, com o transbordo do traço e o miter dentro). Com FX é a caixa da **imagem**,
que é maior (a pilha tem alcance: um halo, uma sombra) — medir a forma **cortaria o halo**.

⭐ **E isto tornou a cura de 04/09 mais barata:** com a opacidade fora da tinta, a chave do memo de
FX deixa de mudar durante um fade ⇒ uma forma filtrada a desvanecer **acerta** o memo, em vez de
re-cozinhar 60 vezes por segundo para produzir os mesmos pixels.

---

## §3 — O vocabulário: uma folha nova, e TRÊS recusas medidas

`ph2d-blend-mode` (crate nova, só-serde) recebe os 22 modos do W3C + a matemática, e a
`ph2d-painter-effects` **re-exporta os três nomes** — nenhum dos 8 consumidores mudou uma linha.

⚠️ **A extracção existe por uma FRONTEIRA DECLARADA:** a `ph2d-vec-scene` diz de si, na primeira
linha do `Cargo.toml`, *"modelo puro de documento — sem vello/kurbo/**ph2d-color**"*, e a
`ph2d-painter-effects` depende da `ph2d-color`. As alternativas eram quebrar aquela fronteira ou
inventar um **terceiro** enum de mistura; o corte foi limpo porque a matemática já era só `f32`.

**A tradução para o Vello** (`ph2d_vec_render::blend`) é `match` **exaustivo** — um modo novo no
vocabulário **não compila** até alguém responder *o Vello faz isto?*:

| Nossos 22 | Vello |
|---|---|
| os 16 do W3C (Normal…Luminosity) | `Mix::X` + `Compose::SrcOver` |
| `Add` (*Linear Dodge*) | `Mix::Normal` + **`Compose::Plus`** |
| `Behind` | `Mix::Normal` + **`Compose::DestOver`** |
| `Clear` | `Mix::Normal` + **`Compose::DestOut`** |
| `LinearBurn` · `VividLight` · `LinearLight` | ⛔ **sem tradução** |

⛔ **As três recusas são do FORMATO, não nossas:** elas são do Photoshop e não estão no conjunto do
W3C *Compositing and Blending Level 1* — logo não estão no shader do Vello. Fazê-las exigiria
**forkar o `.wgsl` de terceiros** que desenha o app inteiro. Elas continuam a existir no Painter,
onde a mistura é a nossa `apply` em CPU/compute.

⚠️ **`Clear` é `DestOut` e não `Compose::Clear`**, e os dois compilam: o `Clear` de Porter-Duff zera
a região **inteira** da camada (um rectângulo de buraco no desenho, incluindo onde a forma não
pinta), e o que o nosso vocabulário promete é *"reduz o alfa do fundo pelo alfa da FONTE"*.

⭐ **A lista que o painel oferece é DERIVADA da tradução** (`blend::offered`), nunca escrita à mão:
um modo oferecido sem tradução gravaria no documento e desenharia `Normal` — *o controlo morto mais
silencioso que há*.

---

## §4 — O painel

Seção **Appearance**, logo abaixo do *Transform* (as duas descrevem a forma selecionada como um
todo): um slider **Opacity** (0..100 %) e um chip **Blend** com o popover derivado.

⚠️ **Seção PRÓPRIA, e não uma row da *Fill***: aquela já tem uma `Opacity` que é outra coisa (§1).

⚠️ **A lei da selecção múltipla: MOSTRA o primário, ESCREVE em todos** — a mesma da fileira de
verbos booleanos, e ela veio de um report (*tocar num filho selecciona o GRUPO*, então o sujeito de
um readout é a forma que o artista apontou). ⛔ Mostrar *«misto»* foi recusado: obriga um terceiro
estado em cada controlo e o artista fica sem saber o que o arrasto vai fazer.

⚠️ **O que viaja no bus é o CÓDIGO do modo**, não a linha do popover — reconstruir a lista do outro
lado para traduzir um índice seria a segunda cópia dela, a que discorda no primeiro modo novo.

⭐ **O passo de undo sai de graça:** o registo é por DIFF e um gesto em curso suprime a captura, então
um arrasto de slider inteiro colapsa em **um** passo ao soltar.

### §4.1 — O teto de LOC cobrou uma seção

O `paint_sections.rs` estava **exactamente** em 600. *A cura de um teto é o corte para um irmão,
nunca uma isenção* — e a escolha de qual cortar não foi o tamanho, foi a **responsabilidade**: a
`boolean_section` era a única SEÇÃO a viver no ficheiro das FERRAMENTAS do corpo. Ela mudou-se para
`paint_boolean.rs` (o idioma que este directório já pratica trinta vezes) e o ficheiro voltou a ser
o que o nome dele diz.

⭐⭐ **E o gate da paridade dos chips booleanos apanhou a mudança pelo CONTROLO dele:** o scanner
apontava ao ficheiro velho, leu **zero** ids e disparou o `!painted.is_empty()` — *a régua quebrou,
não o produto*, exactamente como o doc dele promete. Sem essa metade, ele teria ficado **verde sobre
nada**.

---

## §5 — ⭐⭐⭐ O defeito que a v19 EXPÔS na wave anterior

A ponte da linha do tempo **escreve** o `VecDrivenStyle` e **nunca o apaga**: apagar uma track de
opacidade deixava a forma congelada no último valor da curva, **para sempre e sem gesto de volta**.

⚠️ **Enquanto não havia opacidade autorada isso era invisível** — não havia outro valor a discordar.
Com a v19 há, e o silêncio virou contradição (o documento diz `1,0`, a tela mostra `0,4`).

⇒ `vec_driven_style::settle_to_authored` repõe o componente **depois** de a projecção do quadro ser
lida. ⚠️ **Depois, e não antes:** a lista de estilo já foi construída a partir dele e é ela que
desenha — repor antes apagaria a curva **deste** quadro.

⚠️ **E só para quem JÁ TEM o componente.** Semeá-lo em toda forma poria uma entrada de estilo por
forma na projecção, e o `bound_style` é uma varredura linear — **`O(N²)` por quadro** numa cena de
dez mil formas. *Uma cura que custa um quadrado não é uma cura.*

---

## §6 — Os gates

| Gate | Onde | Afirma |
|---|---|---|
| `a_recook_preserves_the_objects_opacity_and_blend` | `ph2d-vec-scene/recook_tests` | a autoria sobrevive ao re-cozimento (a fixtura está **fora do neutro**, senão o gate não teria dentes) |
| `the_live_opacity_overrides_the_authored_one` | `paint_bind_tests` | a lei dos quatro campos do `BoundStyle` |
| `a_live_opacity_never_touches_the_paint_and_costs_nothing` | idem | o `painted` não desvanece — e **não clona** em toda a faixa do slider |
| `the_fade_dims_every_species_of_paint` · `the_fade_rounds_to_nearest…` | idem | a lei antiga, **re-apontada ao `fade`** (que ficou com um consumidor: a arte de um pincel) |
| `the_offered_list_is_derived_and_the_three_refusals_are_named` | `ph2d-vec-render/blend_tests` | as recusas pelo NOME, sem contagem literal |
| `normal_is_the_fast_path_and_no_other_offered_mode_is` | idem | o censo do controlo morto **e** o caminho byte-idêntico |
| `the_three_composition_operators_carry_a_neutral_mix` | idem | `Add`/`Behind`/`Clear`, com o `DestOut` do §3 |
| `sixteen_modes_translate_as_pure_mix` | idem | o conjunto do W3C, contado pela tradução |
| `only_a_non_neutral_object_pushes_a_layer` | idem | o oráculo é o **`n_clips` da cena do Vello**, e a mistura **não toca na tinta** (`ends_with`) |
| `a_filtered_object_layers_over_the_images_box` | idem | com FX a camada cobre o HALO |
| `the_panel_reads_the_primary_and_the_gesture_writes_them_all` | `vec_appearance_tests` | a lei da selecção múltipla |
| `settling_returns_the_component_to_the_authored_value` · `settling_never_creates_the_component` | `vec_driven_style_tests` | o §5, e o tecto de custo dele |
| `every_painted_section_header_is_collapsible` · `every_section_header_is_registered_as_collapsible` | gates que **já existiam** | ⭐ os dois apanharam a seção nova antes de qualquer smoke: um chevron que não dobra e a contagem da lista |

### §6.1 — Prova de mutação: **5 corridas, 5 morreram**

Cada uma com o controlo verde e o filtro a casar um número **não-zero** de testes (*um filtro que
casa ZERO imprime «SOBREVIVEU»* — a armadilha que a memória deste repo nomeia):

| Mutação | Gate que sangrou |
|---|---|
| `object_alpha` **multiplica** em vez de sobrepor | `the_live_opacity_overrides_the_authored_one` |
| `is_neutral` sempre `true` (nunca empurra camada) | `only_a_non_neutral_object_pushes_a_layer` |
| `layer_rect` ignora a caixa da IMAGEM de FX | `a_filtered_object_layers_over_the_images_box` |
| o re-cozimento **leva** a opacidade do cozido | `a_recook_preserves_the_objects_opacity_and_blend` |
| o `settle` repõe `1,0` em vez do documento | `settling_returns_the_component_to_the_authored_value` |

---

## §7 — O que esta wave NÃO faz

- ⏳ **A tecla K sobre uma forma NUNCA conduzida grava `1,0`**, e não a opacidade autorada: as duas
  leitoras da crate (`read_prop_kind`, que semeia o `rest`) **não alcançam o documento vectorial**.
  Fechá-lo é passar a cena por três assinaturas. *A forma que já foi conduzida uma vez está certa*
  (o §5 repõe o autorado no componente).
- ⛔ **Uma INSTÂNCIA de Motion de uma forma com opacidade desenha-a opaca**, e é a mesma fronteira
  declarada do padrão logo ao lado (`instance.rs`): aquela rota é alimentada pelo cozimento do
  Motion, e o braço com preenchimento pinta a tinta do documento sem passar por uma camada.
- ⛔ **A arte de uma ESTAMPA continua a assar o autorado** — a fronteira é do MEMO daquele assador
  (doc 43 §7.1), não do alcance.
- ⏳ A **timeline** não anima o modo de mistura (um canal de enum pede o caminho de valor não-`Float`
  que a cor também espera — doc 43 §5).

---

## §8 — O smoke

`PH2D_VEC_APPEARANCE_SMOKE=1` (`shells/desktop/src/vec_appearance_smoke.rs`) — uma faixa de fundo em
três cores, cinco discos (um por modo de mistura) e dois quadrados iguais com opacidades
diferentes. ⚠️ **O fundo é load-bearing:** um modo de mistura só existe contra o que está por baixo,
e sobre o fundo da tela todos desenhariam a mesma coisa — *a cena ensinaria que a feature não
funciona*.
