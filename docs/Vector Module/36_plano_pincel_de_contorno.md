# Plano 36 — **o PINCEL de contorno** (*"a coisa precisa funcionar sem limitações"*)

> Ordem do Enio, 2026-08-28, depois de o [plano 35](35_plano_padrao_no_traco.md) fechar o padrão
> como TINTA: *"mas não posso usar o dash com pattern?"* → *"a coisa precisa funcionar sem
> limitações. Qual estado da arte?"*

---

## §1 — O ESTADO DA ARTE: são **dois modelos**, e todo aplicativo sério entrega os DOIS

### §1.1 — Modelo **A: o padrão é uma TINTA** (papel de parede)

A norma é explícita. SVG 2, *Painting*:

> *"When stroking is performed using a complex paint server, such as a gradient or a **pattern**,
> the stroke operation **must be identical** to the result that would have occurred if the geometric
> shape defined by the geometry of the current graphics element and its associated stroking
> properties were converted to an equivalent `path` element and then **filled using the given paint
> server**."*

⇒ um traço tracejado com padrão é, por definição, **a silhueta do tracejado preenchida com o papel
de parede**. Os traços são BURACOS; a arte não os conhece.

| Quem | Como |
|---|---|
| **SVG / navegadores** | `stroke="url(#pattern)"` + `stroke-dasharray` — normativo acima |
| **Figma** | *"use patterns as a fill or stroke"* — a mesma lista de tintas nos dois |
| **Illustrator** | um *pattern swatch* aplicado ao traço |
| **Inkscape** | *Stroke paint* → padrão |

⭐ **É EXACTAMENTE o que o plano 35 entregou**, e ele está **conforme à norma**. ⛔ O que o Enio
fotografou não era um defeito: era o modelo A a funcionar.

### §1.2 — Modelo **B: a arte PERCORRE o caminho** (pincel)

| Quem | Como | O que custou |
|---|---|---|
| **Illustrator — *Pattern Brush*** | até **CINCO** ladrilhos: *side · outer corner · inner corner · start · end*, mais geração automática de quinas (**Auto-Centered · Auto-Between · Auto-Sliced · Auto-Overlap**) | ⚠️ as quinas são *"a stumbling block"* declarado, só **parcialmente** resolvido pela geração automática — o *Auto-Between* deixa emenda visível. ⛔ E ele *"cannot leave out parts of the brush shape, it can only stretch or shrink it if it doesn't fit"* |
| **Illustrator — pincel + tracejado** | a arte **reinicia em CADA traço** | ⚠️ Fóruns pedem exactamente isto e recebem contornos (um 2.º traço com *knockout group*) — a combinação é reconhecidamente desajeitada |
| **Inkscape — *Pattern Along Path* (LPE)** | repetir **ou** esticar; *normal offset* e *tangential offset*; motivo e guia continuam editáveis | ⛔ **um caminho só** (sem grupos), e **não afina** (*taper*) |
| **Affinity — *Vector Brush*** | aceita grupos, sombreado, sobreposição | — |

### §1.3 — ⭐⭐ A síntese, e é ela que responde ao Enio

**Nenhum aplicativo trata isto como um botão.** São **duas ferramentas**: uma TINTA que o contorno
revela, e um PINCEL que corre ao longo dele. *"Sem limitações"* quer dizer **ter as duas, e poder
escolher qual**.

⛔ E é por isso que a resposta *"diminua o Width da estampa"* que eu dei estava errada: ela é um
contorno para a ausência do modelo B, não uma propriedade do modelo A.

---

## §2 — Onde NÓS estamos (medido, não estimado)

| Modelo | Estado nesta casa |
|---|---|
| **A — tinta** | ✅ **entregue** (plano 35): preenchimento e traço, `Tile/Mirror/Clamp`, reticulados, fase, ângulo, vão, e a arte a viajar no ficheiro |
| **B — pincel** | ⚠️ **o MOTOR existe e está medido; o PINCEL não** |

O motor do B é o [Pattern Along Path](23_plano_pattern_along_path.md) (plano 23): cópias **rígidas**
do motivo sobre o comprimento de arco, cada uma rodada para a tangente, sem *refit* — **0,597 ms
para 200 cópias × 40 vértices**, ~13× de folga sob o *kill* de 8 ms.

⛔ **Mas ele é endereçado como uma RELAÇÃO ENTRE DOIS OBJECTOS** (`VecPatternPath { path }`: *"este
motivo cavalga aquela guia"*), e não como uma **propriedade do traço**. ⇒ o artista **não consegue
dizer *"desenha este contorno com esta arte"***, que é a frase inteira da feature.

⇒ **O buraco, numa frase:** falta o **pincel**. O motor está pago.

---

## §3 — O DESENHO

### §3.1 — A fileira *Type* do traço ganha a terceira opção

```
Solid | Pattern | Brush
```

- **`Solid`** — uma cor (hoje).
- **`Pattern`** — a TINTA que o contorno revela (hoje; modelo A, conforme à norma).
- **`Brush`** — a arte **percorre** o contorno (modelo B), pelo motor do plano 23.

⭐ **A escolha da arte é a MESMA porta**: `Source…` (ficheiro) e `Use Shape…` (uma forma do
documento). *Um pincel novo não é um fluxo novo — é a mesma arte noutra lei.*

### §3.2 — ⭐⭐ O pincel corre no PRÓPRIO contorno, sem segundo objecto

O plano 23 exige uma **guia** (outro `VecPath`). Aqui a guia é o **próprio caminho da forma** — o
mesmo que o traço já percorre. ⇒ nada de vínculo, nada de segunda entidade, nada de `stable_name_id`.

⚠️ **E a `LiveGeometry` é o sujeito**, não a fonte autorada: o pincel corre sobre o que a forma É
depois dos efeitos vivos (cantos vivos, largura viva, booleana), pela mesma razão que o traço.

### §3.3 — As três perguntas que o modelo B tem e o A não

| Pergunta | A resposta que o estado da arte dá | A nossa |
|---|---|---|
| **O tracejado?** | Illustrator **reinicia a arte em cada traço** | idem — e é o que o Enio pediu ao juntar as duas coisas |
| **As quinas?** | Illustrator: **ladrilho de quina** (interno/externo) + 4 modos automáticos, e ainda assim *"a stumbling block"* | ⏳ **wave própria**, com os 4 modos MEDIDOS antes de escolher. ⛔ O plano 23 hoje **pula a cópia** numa cúspide — é o buraco conhecido |
| **A emenda de um contorno FECHADO?** | Illustrator estica/encolhe para fechar; Inkscape deixa sobrar | ⭐ **a maquinaria já existe**: a `dash_fit` faz exactamente isto para o tracejado (ajusta ao comprimento para a emenda fechar). *Uma porta, dois consumidores.* |

### §3.4 — ⛔ O que este plano NÃO faz

- **Não deforma** a arte para dobrar na curva (*bending*) — é o **Envelope** (ADR-0129), que já
  existe e **compõe**; o plano 23 §0 já nomeou esta bifurcação.
- **Não substitui o modelo A.** Os dois ficam, e a fileira *Type* é onde se escolhe. *Um aplicativo
  que só tem um dos dois é o que tem limitação.*
- **Não faz os 5 ladrilhos do Illustrator na v1** — *side* primeiro; *start/end/corner* são waves com
  a medição ao lado.

---

## §4 — As waves

| Wave | Entrega | Onde |
|---|---|---|
| **W1** ✅ | `StrokePaint::Brush(Box<BrushStroke>)` + o schema (`VEC_SCENE` **16→17**, `PROJECT` **101→102**, a tripla) | `ph2d-vec-scene` |
| **W2** ✅ | O **motor**: correr o `pattern_along` sobre o próprio contorno, com **fit** de emenda pela porta da `dash_fit` — **0,423 ms / 200 cópias**, 19× sob o *kill* | `ph2d-vec-scene` |
| **W3** ✅ | O **desenho**: as cópias emitidas, com o guarda de ciclo estrutural e a queda para a cor de recurso | `ph2d-vec-render` + shell |
| **W3-bis** ✅ | O **tracejado**: a arte reinicia em cada traço (a lei do Illustrator) + a cena de smoke `=77` — teto de fatias medido em **4096** | `ph2d-vec-scene` + shell |
| **W4** ✅ | A UI: a 3.ª opção da fileira *Type* + a secção **Brush** (Size · Spacing · Rotation · Offset · Flip) + o gesto de duas mãos | `ph2d-panel-vector` + shell |
| **W5** | As **QUINAS**: os 4 modos do Illustrator medidos lado a lado, e o nosso escolhido **com a tabela** | `ph2d-vec-scene` |
| **W6** | Persistência + smoke + gates + mutações | shell |

**Kill-criterion (DIRETIVA §5):** o plano 23 mede **0,597 ms / 200 cópias**. Um contorno de forma
típica pede a mesma ordem. ⇒ **se o re-cook de uma tecla passar de 8 ms, a feature não existe nesta
forma** e o passo seguinte é cache por-params — ⛔ **não** subir o teto.

---

## §5 — ⚠️ A lição que trouxe este plano

Eu respondi ao report do Enio com *"diminua o Width da estampa até caberem várias cópias por
traço"*. Ele devolveu: *"a coisa precisa funcionar sem limitações"*.

⭐ **Ele estava certo, e o defeito era meu:** eu ofereci um **contorno** para a ausência de uma
feature, em vez de ir ver o que o estado da arte faz. A pesquisa levou quatro buscas e mostrou que
*todo* aplicativo sério tem os dois modelos — e que nós já tínhamos o motor do segundo, pago e
medido, há um mês.

*Um workaround oferecido no lugar de uma pesquisa é uma limitação transformada em política.*

---

## ⛔ Recusas MEDIDAS

| O que | Porquê | Onde |
|---|---|---|
| Deformar a arte para dobrar na curva dentro deste plano | é o **Envelope** (ADR-0129), que já existe e compõe | §3.4 |
| Substituir o modelo A pelo B | a norma SVG exige o A, e o Figma entrega-o como *fill or stroke* | §1.1 |
| Os 5 ladrilhos do Illustrator na v1 | as quinas são *"a stumbling block"* declarado até para eles; entram com medição | §3.3 |
| *"Diminua o Width da estampa"* como resposta ao artista | é um contorno para a ausência do modelo B | §5 |

---

## §6 — W1 fechada: **o modelo** (2026-08-28)

### §6.1 — ⛔ A arte de um pincel é uma FORMA, e o TIPO impede o contrário

`BrushStroke::art` é um `VecPathId`, **não** um `PatternSource`. O motor (`pattern_along`, plano 23)
copia **geometria**; um `PatternSource` também aceita imagem, e `Brush(Image(..))` seria estado
**gravável e indesenhável**. É a mesma lei que recusou reusar o `Paint` como tinta de traço
(plano 35 §2.1), e aqui ela está no **tipo** — a forma mais forte de invariante, e a mais fácil de
apagar num refactor sem dar por isso. ⇒ gate `a_brush_can_only_name_a_shape_never_an_image`.

### §6.2 — ⭐⭐ O pincel ESCALA com a largura; o padrão NÃO

O plano 35 §2.3 fixou o contrário para a TINTA (*"a largura decide a faixa; o padrão decide o que a
preenche"*) — a queixa clássica do Illustrator, do lado certo. Um pincel **é** a faixa, e o *Pattern
Brush* escala com o peso do traço.

⇒ o pincel guarda `scale` **relativo** (multiplica a altura derivada da largura) e o padrão guarda
`size` **absoluto** em unidades de mundo. *Se os dois guardassem a mesma grandeza, uma das duas leis
estaria escrita no sítio errado.*

### §6.3 — ⭐ O enum FECHADO trouxe-me a cinco decisões que eu não teria visto

O compilador parou em cinco `match` exaustivos, e **cada um era uma pergunta de produto**:

| Sítio | A decisão |
|---|---|
| `paint_bind::fade` | ⏳ um pincel desvanece a **cor de recurso**, e as CÓPIAS ainda não — declarado, não calado com um `_ => {}` |
| `StrokeSpec::pattern()` | um pincel **não** responde como padrão |
| `StrokeStyle::onto` | a cor escreve a `fallback`, **nunca** substitui a tinta — e aqui **não** há opacidade a escrever, ao contrário do padrão |
| `log_shape` | o diagnóstico **nomeia** o pincel inteiro |
| `vec_stroke_paint::set_kind` | recusa em voz baixa (a criação é a W4) — ⛔ e não um `todo!()`, que derrubaria o app se alguém publicasse o chip antes da hora |

*Um `_ =>` genérico em qualquer um deles teria calado uma pergunta.* ⚠️ **Uma metade nomeada é uma
dívida; uma metade silenciosa é um defeito.**

### §6.4 — ⚠️ A escada do `VEC_SCENE` estava um degrau atrás

Ela parava no **v15**: a wave A do plano 35 subiu para 16 e **não escreveu o degrau** (documentou-o
só no `project_schema.rs`). Escritos os dois — v16 (destrutivo: um campo mudou de tipo no meio da
estrutura) e v17 (aditivo: variante apendada). *Uma escada com um degrau a menos manda a próxima
janela procurar o que mudou no diff.*

### §6.5 — As quatro provas de mutação

| Mutação | Gate que morreu |
|---|---|
| `pattern()` a devolver `Some` para um pincel | `a_stroke_can_carry_a_brush` |
| `color()` a devolver preto em vez da `fallback` | `the_stroke_colour_still_answers_for_a_brush` |
| `#[serde(skip)]` no `scale` | `the_brush_survives_the_save` |
| o neutro do `scale` a virar `2.0` | `the_brush_scale_is_relative_and_the_pattern_size_is_absolute` |

⚠️ E o **round-trip nomeia os SETE campos**, em vez de um `assert_eq!` da struct: uma igualdade
verde diz que os bytes voltam, **não quais**. Com um campo apendado e não escrito, os dois lados
teriam o default e a igualdade passaria.

---

## §7 — W2 fechada: **o motor** (2026-08-28)

### §7.1 — ⭐⭐ O ENCAIXE é a porta do tracejado, com o avanço no lugar do período

O `dash_fit::fit` é uma **lei pura**: escala um `[traço, vão]` para caber um número **inteiro** de
vezes num contorno. Um avanço é o mesmo problema com o vão a zero — e a resposta é a primeira
componente. ⇒ `PatternSpec::fit_to_guide`, e a lei **não se reescreve**.

⚠️ **O plano 23 §3 deixou isto nomeado como *"refinamento com dono próprio"*** — e o dono é este
plano. *Um adiamento com dono nomeado é uma dívida; sem dono, é uma limitação.*

⛔ **E fica OPT-IN:** o default é `false`, e o *Pattern on Path* sai **byte a byte** como saía —
mudar uma feature entregue por causa de outra seria o oposto do que este plano faz.

### §7.2 — ⚠️ O `ArcPath` estava a DEITAR FORA o que lhe diziam

`from_contour(verts, closed)` recebia `closed` para contar os segmentos e **esquecia-o**. Um
consumidor que precisasse dele teria de o carregar em paralelo — e um dado paralelo ao objecto que o
descreve dessincroniza no primeiro sítio que esquecer de o passar. ⇒ o `ArcPath` guarda-o.
*Se o construtor já sabe, o objecto guarda.*

### §7.3 — ⭐ Cada CONTORNO fecha, e a limitação herdada era inventada

O `dash_fit` escolhe o contorno **mais longo** porque o traçador recebe **um** par `[traço, vão]`
para o caminho inteiro (a nota dele diz isso, com o preço). O pincel **não tem essa restrição**: cada
contorno recebe as suas cópias e fecha exactamente. *Herdar uma limitação sem perguntar se ela ainda
existe é inventá-la.*

### §7.4 — ⭐⭐ O KILL-CRITERION, MEDIDO

```
[plano 36 W2] re-cook do pincel: 0,423 ms  (200 copias)  — kill = 8 ms
```

**19× de folga**, e mais rápido que os `0,597 ms` que o plano 23 mediu para o mesmo trabalho: a
escala da arte é **um passe sobre os vértices dela, uma vez** — não por cópia. Gate `#[ignore]`,
`--release`.

### §7.5 — ⚠️⚠️ DUAS lições de processo, e as duas custaram

1. **O `| grep Summary` da varredura ENGOLIU um erro de compilação.** A saída veio vazia e eu quase a
   li como *"sem falhas"*. É o modo de falha que o roteador §2 documenta com número
   (**4.414 corridas que nunca chegaram a rodar um teste**) e a razão de os scripts preservarem o
   *exit code*. ⇒ *uma corrida que devolve NADA nunca é uma corrida verde.*
2. **O que apanhou o erro foi um LITERAL DE STRUCT.** O `spec_to_motor` do *Pattern on Path*
   constrói o `PatternSpec` campo a campo, sem `..Default::default()` — então o campo novo **obrigou
   a uma decisão**. Com o `..Default` a decisão teria sido tomada em silêncio, pelo default.
   ⭐ *Um construtor exaustivo é um gate que não se apaga.*

### §7.6 — As quatro provas de mutação

| Mutação | Gate que morreu |
|---|---|
| a altura deixa de multiplicar a largura | `the_brush_art_scales_with_the_stroke_width` |
| `fit_to_guide: false` no pincel | `on_a_closed_contour_the_copies_close_exactly` |
| só o contorno principal recebe cópias | `every_contour_of_a_compound_gets_its_own_copies` |
| o encaixe passa a ser o default | `the_fit_is_opt_in_and_the_old_consumer_is_untouched` |

⚠️ E o gate do encaixe traz o **controlo de que a fixtura contém o fenómeno**: a arte foi escolhida
com uma largura que **não** divide o perímetro, senão ele ficaria verde sobre um encaixe por acidente.

---

## §8 — W3 fechada: **o desenho** (2026-08-28)

### §8.1 — ⭐ A arte entra pela porta do LADRILHO, e pela mesma razão

`BrushArts = BTreeMap<VecPathId, VecPath>` — a arte **cozida** de cada pincel, pela forma
HOSPEDEIRA, resolvida pela **shell** (`brush_live`). A `ph2d-vec-render` não alcança a cena, e ir
buscar a forma-fonte lá dentro poria o guarda de ciclo, a geometria viva e o cozimento num sítio que
não os pode medir. *É a decisão que o ladrilho do padrão já tinha tomado.*

⚠️ **A chave é quem PINTA, não quem é pintado.** Trocá-las faria o desenho procurar pelo id errado e
cair sempre na cor de recurso — gate com o controlo de que o mapa tem **uma entrada por hospedeira**.

### §8.2 — ⛔⛔ O guarda de ciclo é ESTRUTURAL, não uma bandeira

As cópias desenham com os **três mapas a `None`**. Uma arte que tivesse ela própria um pincel
entraria em recursão infinita, e ⚠️ **o sintoma não seria um erro: seria o app a parar**. ⇒ a recusa
vive **na chamada**, e não numa bandeira que alguém se lembre de passar.

⚠️ E há uma segunda metade, PURA, no `brush_live`: uma forma **não pode ser o próprio pincel** — a
mesma recusa que o padrão-forma já tem, e pelo mesmo mecanismo.

### §8.3 — ⭐⭐ As DUAS portas de rasterização isolada, e a assimetria entre elas

| Porta | Padrão | Pincel |
|---|---|---|
| **FX raster** (`fx_live`) | ✅ já resolvia — e **esqueceu-o uma vez**: o report do Enio de 27/08 (*"filters anula pattern"*) | ✅ resolve |
| **Assado de objecto Motion** (`motion_object_bake`) | ⛔ leva a `fallback`, **declarado**: o ladrilho precisa do assado, que vive fora | ✅ **resolve** |

⭐ **A assimetria é real, não um descuido:** o ladrilho de um padrão precisa da arte descodificada e
do reticulado composto; a arte de um pincel é **geometria da mesma cena** que o assado já recebe.
*Herdar a limitação do vizinho por simetria seria inventá-la* — a mesma lição que o encaixe por
contorno deu na W2.

⚠️ **A porta do FX é a que já esqueceu a tinta nova uma vez.** Uma segunda porta de desenho esquece a
tinta seguinte — e desta vez ela entrou na mesma wave.

### §8.4 — As quatro provas de mutação

| Mutação | Gate que morreu |
|---|---|
| o ramo do pincel nunca dispara | `a_brushed_stroke_draws_the_copies_and_falls_back_to_the_colour` |
| o guarda de ciclo desligado | `a_shape_can_never_be_its_own_brush` |
| a arte entra AUTORADA em vez de cozida | `the_art_enters_cooked_not_as_authored` |
| o mapa chaveado pela ARTE em vez da hospedeira | `the_brush_art_resolves_keyed_by_its_host` |

### §8.5 — ⚠️ Um membro NOVO da família de flakes de recurso

`the_cost_of_a_player_is_linear_in_their_number` (`ph2d-physics-ecs::measure_player_budget`)
reprovou na varredura e passou **3/3 sozinho**, com **zero** ficheiros de física no diff. Ele compara
**duas medianas de relógio**, que é a forma que o `CLAUDE.md` §5.0 declara ser a família inteira —
*"todo gate que compara duas medianas de um RECURSO é candidato, e a lista nunca estará completa"*.
Fica registado aqui para a próxima janela não o caçar.

---

## §9 — W4 fechada: **a UI** (2026-08-28)

### §9.1 — ⭐⭐ Um gate achou um defeito de MODELO: `art` tem de ser um `Option`

`VecPathId::default()` é um id **VÁLIDO** — a primeira forma de uma cena pode tê-lo. Com um id cru,
*"sem arte"* e *"a arte é aquela forma"* seriam **os mesmos bytes**, e a porta que escreve a arte
recusava-a **em silêncio** por «já é esse valor».

⇒ `BrushStroke::art: Option<VecPathId>`. É a lei que esta casa já pagou noutro sítio:
*um zero de «não medido» e um de «perfeito» são o mesmo byte.*

⚠️ **E `Some(id)` não garante que a forma existe** — ela pode ter sido apagada. *"Tem arte?"* é uma
pergunta à **CENA**; o campo só diz o que foi **autorado**. É por isso que o rótulo do botão
(*Pick Shape…* × *Change Shape…*) sai de `vec_scene.path(a).is_some()`, e não do campo.

⚠️ **O `PROJECT_SCHEMA` fica em 102**, e a razão é escrita: a variante `Brush` **nasceu nesta wave**
e nenhum ficheiro a pode carregar — a forma interna dela assentou antes de existir um byte gravado.
*A escada mede «um ficheiro antigo passa a ser lido errado», e aqui não há ficheiro antigo.*

### §9.2 — ⛔ Sem diálogo de ficheiro: a arte é uma FORMA

Clicar `Brush` **arma o gesto de duas mãos** (o `PathPick::BrushArt`), e não abre um `rfd`. O motor
copia GEOMETRIA, e o tipo torna a alternativa inexprimível — a mesma decisão da W1, agora visível na
UI. ⭐ *A porta do produto segue o tipo, em vez de o tipo seguir a porta.*

### §9.3 — ⚠️ Duas leis da casa apanharam o ficheiro novo, e as duas curas são reais

| Gate | O que ele pediu |
|---|---|
| `no_magic_numeric_in_widget_or_screens` | os três `0.05` de passo ganharam **nome** e o marcador `LITERAL-PX-OK`, como o irmão do padrão já fazia |
| `every_widget_file_wires_a11y` | a secção delega em `slider_row`/`checkbox_row`/`action_button` do próprio painel ⇒ entrou no `PANEL_A11Y_DELEGATE_OK` **com a justificação**. ⚠️ E a nota regista que a **irmã `paint_texture_pattern.rs` passa por COINCIDÊNCIA** (nomeia `ph2d_a11y::NodeId` numa assinatura de helper), não por fiar a11y — *a delegação das duas é a mesma; só uma estava declarada* |

### §9.4 — As três provas de mutação

| Mutação | Gate que morreu |
|---|---|
| o guarda «uma forma não é o próprio pincel» desligado | `a_shape_can_never_author_itself_as_its_own_brush` |
| `Spacing` a escrever no campo do `Scale` | `every_brush_knob_writes_its_own_field_and_only_its_own` |
| o `if` de igualdade removido (passo espúrio de undo) | idem |

⚠️ O gate dos knobs traz a metade que importa: **os outros quatro campos ficam INTACTOS**. Sem ela,
ele ficaria verde sobre uma porta que reconstrói o pincel do zero — e o artista veria os outros
knobs saltarem para o default.

### §9.5 — ⚠️ Três flakes de recurso numa corrida, e o CONJUNTO mudou entre corridas

`the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` · `only_the_lower_row_breathes_…` ·
`a_wet_move_costs_what_the_footprint_costs_…` — os três **nomeados** no `CLAUDE.md` §5.0, todos
**5/5 verdes sozinhos**, e o diff sem uma linha de Flip, áudio ou Painter. ⭐ E o sinal que o §5.0
descreve apareceu inteiro: *num grupo, o conjunto de reprovadas MUDA entre corridas do mesmo
binário* — a corrida anterior reprovou outros dois.

---

## §10 — W3-bis fechada: **o TRACEJADO** (2026-08-29)

> *"mas não posso usar o dash com pattern?"* — a pergunta que abriu o plano fecha aqui.

### §10.1 — ⭐⭐ A arte reinicia em cada traço, e os vãos ficam VAZIOS

A lei é a do *Pattern Brush* do Illustrator, e ela cabe inteira em duas frases:

1. **as FATIAS** que o contorno oferece são os **traços** do traçador
   ([`brush_spans`](../../crates/ph2d-vec-scene/src/brush_stroke.rs));
2. o **avanço encaixa no TRAÇO**, não no contorno — é isso que faz cada traço começar e acabar com
   uma cópia inteira.

⚠️ **O `[traço, vão]` chega já ENCAIXADO, pela porta do traçador**
([`dash_fit::dash_lengths_for`](../../crates/ph2d-vec-scene/src/dash_fit.rs)) — o mesmo par que
desenharia a linha. *Uma segunda medição poria a arte numa cadência e a linha noutra, e o artista
veria dois tracejados sobre a mesma forma.*

⚠️ **A última fatia é TRUNCADA no fim do contorno**, como um traçador faz: um composto carrega **um**
par de tracejado, fitado ao contorno mais longo, então os outros anéis acabam a meio de um traço.

### §10.2 — ⭐ O alvo do encaixe é UM número para todas as fatias

⛔ A alternativa — encaixar cada fatia em si mesma — daria à fatia truncada uma cadência própria,
**à vista**. Com um alvo só, todo traço leva o mesmo ritmo e a fatia truncada simplesmente leva
menos cópias.

### §10.3 — ⚠️ `fit_to_guide: bool` virou `fit_span: Option<f64>`, e a W2 estava a decidir no sítio errado

Enquanto era um `bool` (*"encaixa na guia"*), o alvo era **sempre** a guia inteira. Num traço
tracejado quem tem de fechar é o **TRAÇO**, e só o chamador sabe qual dos dois é o sujeito. *Um
parâmetro booleano é uma decisão tomada por quem não tem os dois números na mão.*

### §10.4 — ⛔⛔ Uma guarda que eu escrevi, gateei, e a mutação DESFEZ

O `dash_fit` tem duas leis (fechada: `n` períodos · aberta: `n` períodos **mais um traço**), e eu
concluí — com razão aparente — que *"uma fatia de traço é aberta mesmo numa guia fechada"*, escrevi
a derivação e um gate a defendê-la pela saída.

⭐⭐ **A mutação que trocava a derivação de volta por `guide.closed()` SOBREVIVEU.** Com o vão a
zero as duas leis dizem a mesma frase — *`round(T/p)` cópias de comprimento `p`* — e só divergem
nos **empates do `round` em `f64`**, que nenhuma fixtura honesta atinge. A derivação saiu; ficou a
linha simples com a medição ao lado.

⚠️ **Três redacções minhas erraram antes disso, cada uma medida e refutada pela seguinte:**

| # | O que afirmei | O que a medição deu |
|---|---|---|
| 1 | as duas leis dão o **mesmo bit** | reprovou na 2.ª amostra — **2 ULP** |
| 2 | *"2 ULP, é só uma questão de nome"* | reprovou a **`5,84e-4`** relativo (3 ordens acima) |
| 3 | *"⇒ a bandeira é load-bearing, derive-a"* | a **mutação sobreviveu** — é knife-edge, e nada honesto lá cai |

*Uma afirmação sobre `f64` que não foi varrida é uma conjectura com cara de teorema; e uma guarda
que mutação nenhuma mata é código sem sujeito.*

### §10.5 — ⭐⭐ O teto de fatias, MEDIDO (`MAX_DASHES = 4096`)

⚠️ **O que o tracejado acrescenta ao custo NÃO são as cópias**: a soma delas sobre os traços é no
máximo `total/avanço`, o mesmo do contorno inteiro. O que cresce é o custo **FIXO por fatia** (uma
medida do bbox da arte, uma divisão).

| traços | cópias | re-cook |
|---|---|---|
| 1 (sem tracejado) | 200 | 0,27 ms |
| 100 | 100 | 0,15 ms |
| 400 | 400 | 0,62 ms |
| 1 026 | 1 025 | 1,69 ms |
| 2 051 | 2 051 | 2,92 ms |
| **4 103** | 4 103 | **6,32 ms** |
| 8 205 | 8 205 | ⛔ **12,08 ms** |

⇒ o joelho está entre `4 103` e `8 205` contra o *kill* de **8 ms**; o teto fica em **4096**, com o
teto real medido a `5,8`–`6,0 ms`. ⭐ É o mesmo número do `MAX_COPIES` **por medição, não por
simetria** — os dois limitam o trabalho de um re-cook. ⚠️ Ele morde quando
`comprimento(contorno) > 4096 × período` (com largura `0,03` e tracejado `(2,2)`: um perímetro de
**~490** unidades), e o sintoma é a arte **parar a meio do contorno, sem aviso** — a mesma nota que
o `MAX_COPIES` já carrega, e a mesma saída: o cache por-params do plano 23 §0.

### §10.6 — ⭐ A cena de smoke: `PH2D_BUILD_SMOKE=77`

Irmã da `=76` (a estampa) de propósito: as duas existem lado a lado porque são os **dois modelos**.
Cinco formas + a arte visível: o caso base · **o tracejado** · uma onda ABERTA · um anel COMPOSTO ·
a mesma arte com `Rotation = 90` e `Flip`.

⚠️ **As curvas são suaves de propósito** — as QUINAS são a W5, e uma cena que as exibisse mostraria
um buraco **conhecido** como se fosse um defeito.

⚠️ **A fixtura precisou de duas correcções, e as duas foram apanhadas por gate:**

- o tracejado `(3, 2)` dava **exactamente uma cópia por traço** — e aí *"a arte reinicia em cada
  traço"* e *"há uma bolha em cada traço"* **desenham a mesma coisa**. O número saiu da conta
  (arte de `0,875` escalada, volta de `6,91`) e é `(5, 2½)`: 3 traços de 2 cópias;
- a régua da assimetria da arte era um **proxy** (centro do bbox contra a média dos vértices) e
  reprovou produto correto a `0,0125`. A régua a sério **espelha a arte e mede o quanto ela deixou
  de coincidir consigo mesma** — `0,19` em `x` e `0,37` em `y`. *A média de quatro pontos não sabe
  nada sobre a forma entre eles.*

### §10.7 — As cinco provas de mutação

| Mutação | Gate que morreu |
|---|---|
| o alvo do encaixe deixa de ser o traço (`alvo = total`) | `every_dash_carries_the_same_rhythm` |
| o tracejado não chega ao motor (`dash = None`) | `the_art_lives_inside_the_dashes_and_the_gaps_stay_empty` |
| o encaixe do avanço desliga (`fit_span: None`) | `on_a_closed_contour_the_copies_close_exactly` |
| as fatias deixam de ser limitadas (`MAX_DASHES` fora do laço) | `the_spans_are_one_without_a_dash_and_one_per_dash_with_one` |
| a cena volta ao tracejado de uma cópia por traço | `the_smoke_dash_carries_more_than_one_copy_per_dash` |

⚠️ E a **sexta** foi a que sobreviveu (§10.4) — ela não está nesta tabela porque não há gate: o que
ela mediu foi que a guarda não tinha sujeito, e a resposta foi apagar a guarda.

---

## §11 — W5: **AS QUINAS** — o plano (2026-08-30)

> Ordem do Enio: *"vamos lá, comece"*. ⚠️ Etapa **complexa** ⇒ ela é **auditada antes** de eu
> oferecer o smoke (regra nova do Enio, 30/08).

### §11.1 — ⭐⭐⭐ A MEDIÇÃO veio primeiro, e ela REESCREVEU o problema

Eu ia desenhar «ladrilhos de quina à Illustrator». A sonda
(`measure_how_far_a_corner_throws_the_copies_off_the_guide`, `ph2d-vec-scene`) diz outra coisa.

| arte (largura) | círculo de igual perímetro | quadrado 7×7 | **buracos** | desvio pior no quadrado |
|---:|---:|---:|---:|---:|
| 1,0 | 28 cópias | 28 | **0** | 1,41× a meia-altura |
| 1,3 | 22 | 20 | **2** | 1,00× |
| 1,7 | 16 | 16 | **0** | 1,00× |
| 2,0 | 14 | 12 | **2** | 1,00× |

E a varredura por **razão lado/arte** — o regime da queixa nº 1 dos fóruns do Illustrator
(*"apliquei o pincel a um rectângulo pequeno e os lados sobrepõem-se nas quinas"*):

| lado | lado/arte | cópias | **buracos** | desvio pior |
|---:|---:|---:|---:|---:|
| 2,0 | 1,5 | 4 | **2** | 1,00× |
| 3,0 | 2,3 | 8 | **1** | 1,00× |
| 5,0 | 3,8 | 14 | **1** | 1,00× |
| 7,0 | 5,4 | 20 | **2** | 1,00× |
| 12,0 | 9,2 | 36 | **1** | 1,64× |
| 20,0 | 15,4 | 60 | **2** | 1,63× |

⇒ **O defeito dominante é a AUSÊNCIA, não o excesso.** Há `1`–`2` cópias em falta em **todo**
tamanho testado, e o desvio das que ficam mal se mexe (`1,00×`–`1,64×` de meia-altura). *O pincel
não «salta» a quina: ele não a desenha.*

⚠️⚠️ **E a 1.ª RÉGUA era cega a isso.** Ela percorria as cópias **emitidas** e media o desvio de
cada uma — e o que uma quina faz hoje é **não emitir**. *Uma régua que percorre o que existe não vê
o que faltou* — a mesma família do balde que ninguém enche e se lê como perfeito.

⚠️ **E a 1.ª FIXTURA dizia que estava tudo bem** (arte `1,0` ⇒ avanço `1,0` ⇒ as quinas em `7·14·21·28`
caem exactamente ENTRE duas cópias, `0` buracos). *A fixtura mais azarada possível é a que aprova.*

### §11.2 — ⭐⭐⭐ O MECANISMO, e ele NÃO é «a quina não tem tangente»

A cadeia, medida ponta a ponta:

```
ArcPath::frame_at(s)  ->  tangent_at(seg, t)  ->  None   (cúspide)
GlyphFrame::on_path   ->  None
pattern_along         ->  `continue`  ⇒  a cópia é PULADA
```

E a causa da cúspide é **uma degenerescência de parametrização, não uma quina**:

- `deriv(c, 0) = 3·(P1 − P0)`;
- `VecVertex::corner(p)` põe `in_handle = out_handle = anchor` ⇒ num segmento entre duas quinas
  **`P1 = P0` e `P2 = P3`**;
- ⇒ `B'(0) = B'(1) = 0`, e `tangent_at` devolve `None` **nas duas pontas de todo segmento recto**.

⭐⭐ **A direcção EXISTE ali** — é uma recta. O que não existe é a *velocidade*. ⇒ uma parte do buraco
de hoje é um **falso positivo de cúspide**, e a cura é a padrão: quando `B'(t) = 0`, cair para a
**derivada seguinte** (`B''`, depois `B'''`), que para `P1=P0, P2=P3` devolve exactamente a direcção
da corda.

⚠️⚠️ **Isto é `ph2d-arclen`, uma crate-folha com CINCO consumidores** (Trim · Repeater ·
Pattern Along Path · texto em caminho · Zig Zag). Curar ali cura os cinco — e por isso a mudança é
**foundational** e leva gate próprio + prova de mutação, não um remendo local no pincel.

⇒ **O trabalho parte em DUAS metades que não se confundem:**

| | o quê | onde |
|---|---|---|
| **A — a cúspide FALSA** | `B'(t) = 0` numa recta: a direcção existe e é a corda | `ph2d-arclen` (foundational) |
| **B — a quina VERDADEIRA** | a tangente **salta** pelo ângulo de viragem; nenhuma cópia rígida cobre os dois lados | `ph2d-vec-scene` (o pincel) |

⛔ **A ordem é A → B, e não é preferência:** enquanto o A não fecha, toda medição do B mistura
buracos de degenerescência com buracos de viragem, e a tabela do B seria sobre as duas coisas.

### §11.3 — O DESENHO da metade B, e a palavra já existe nesta casa

⭐⭐ **A quina de um pincel é uma JUNÇÃO** — e o `StrokeSpec` já carrega
`join: LineJoin { Miter, Round, Bevel }`, que o pincel **ignora hoje** (zero ocorrências de `join`
em `brush_stroke.rs` / `pattern_path.rs`). *O desenho pedido já é lei na outra metade do app.*

E os quatro modos automáticos do Illustrator mapeiam-se quase um a um
([Peachpit](https://www.peachpit.com/articles/article.aspx?p=2979069&seqNum=16) ·
[Tiny Tutorials](https://tinytutorials.wordpress.com/2014/06/02/illustrator-cc-automatic-corner-generation/)):

| Illustrator | o que faz à arte | a nossa palavra |
|---|---|---|
| **Auto-Sliced** | fatia o ladrilho na diagonal, as metades juntam-se **como um miter** | `Miter` |
| **Auto-Centered** | estica o ladrilho **à volta** da quina, centrado nela | `Round` |
| **Auto-Between** | uma cópia de cada lado **entra até** à quina (deixa emenda visível) | `Bevel` |
| **Auto-Overlap** | copia e **sobrepõe** na quina | é o que fazemos hoje, e é o que se vê mal |

⚠️ **A escolha ainda NÃO está feita** — a tabela acima é o mapa, não a decisão. Os quatro entram
**medidos lado a lado** com a régua do §11.1 (buracos · desvio · sobreposição) antes de um ser o
default, e é isso que a W5 vai produzir.

⛔ **O que a W5 NÃO faz:** os **cinco ladrilhos autorados** do Illustrator (side · outer · inner ·
start · end). A referência declara-os *"a stumbling block"* e os fóruns dela dizem porquê — *"é
impossível criar ladrilhos de quina à mão que casem com os laterais"*, *"os ladrilhos de quina são
grandes demais para objectos pequenos"*, e a receita que os utilizadores experientes dão é
**abandonar o pattern brush** e refazer o efeito com *Outline Stroke* + *Roughen*
([Adobe Community](https://community.adobe.com/t5/illustrator/pattern-brush-side-and-top-overlapping-at-corners-how-to-fix/td-p/9739376)).
⇒ *a barra a bater é baixa, e o caminho deles não é o que se copia.*

### §11.4 — Onde encosta

| | |
|---|---|
| **Contrato congelado (§6)** | **nenhum** — `ph2d-arclen` e `ph2d-vec-scene` não são gateadas por `architecture_*_contract_surface` |
| **Schema** | **nenhum** se a quina for lida do `join` que já existe e já persiste; ⚠️ um enum NOVO de modo de quina no `BrushStroke` custaria `VEC_SCENE` + `PROJECT` + a tripla |
| **UI** | ⚠️ a decidir depois da medição — se o `join` responder, são **zero** controlos novos |

### §11.5 — Os gates, red-first

1. `a_straight_segment_has_a_direction_at_its_endpoints` (`ph2d-arclen`) — a metade A, com o
   controlo de que uma cúspide **verdadeira** continua a devolver `None`.
2. `a_square_gets_the_same_number_of_copies_as_a_circle_of_equal_perimeter` — a régua do §11.1
   promovida a gate: **zero buracos**.
3. `the_corner_law_is_the_join_the_stroke_already_declares` — se a medição escolher o `join`.
4. A fixtura contém o fenómeno: um quadrado **e** o círculo de igual perímetro, com a arte cuja
   largura **não** divide o lado.

### §11.6 — O smoke

Cena **`=78`** (⛔ o número **conta-se** no `build_smoke_router.rs` na altura, nunca desta nota):
um **quadrado**, uma **estrela** e uma forma de **bicos** desenhados com pincel, ao lado das curvas
suaves da `=77` — e a régua do Enio é a mais simples que há: *a linha dá a volta inteira sem
buracos nas quinas*.

### §11.7 — ✅ **METADE A FECHADA** (2026-08-30): a reta volta a ter direção

`ph2d-arclen::tangent_at` deixa de ler *velocidade zero* como *direção ausente*. Quando `B'` se
anula, a direção sai do **polígono de controlo** — mas **só se aquela ponta for degenerada**, que é
a cerca que impede a cura de invadir a cúspide interior.

**Medido, na mesma sonda, antes e depois:**

| lado do quadrado | esperadas (a lei do encaixe) | emitidas ANTES | emitidas DEPOIS |
|---:|---:|---:|---:|
| 2 | 6 | 4 | **6** |
| 3 | 9 | 8 | **9** |
| 5 | 15 | 14 | **15** |
| 7 | 22 | 20 | **22** |
| 12 | 37 | 36 | **37** |
| 20 | 62 | 60 | **62** |

E as posições de arco com tangente nula numa volta: **5 → 0** (4001 amostras), e **0 de 4** quinas
em cada um dos três tamanhos sondados directamente.

⇒ **Zero buracos.** O gate que o fixa é `a_square_gets_every_copy_the_fit_asks_for`, e ele carrega
a metade que prova que a fixtura contém o fenómeno: *o quadrado é autorado com vértices de quina*.

#### §11.7.1 — ⛔⛔ A 1.ª cura passava nos testes de mesa e FALHAVA NA ÁRVORE

Ela testava `t <= 0.0` / `t >= 1.0`. **O `t` nunca chega à ponta:** o prefixo de arco é somado por
quadratura, e o comprimento de um segmento reto de `2` sai `2,000000000000000_4` — quem pergunta
pelo arco `2,0` cai no segmento **anterior**, em `t = 0,999999999999999_8`, onde `|B'| ≈ 2,6e-15`:
**abaixo** do piso do versor e **acima** de zero.

⚠️ **E o sintoma era selectivo, que é o pior:** com a 1.ª cura o quadrado de lado `7` ficava a zero
buracos e os de lado `2` e `12` mantinham **4 de 4** quinas sem tangente. *Duas fixturas do mesmo
desenho, e só uma via o defeito* — se a varredura tivesse um tamanho só, a cura teria sido dada por
boa.

⇒ a condição passou a ser **«o polígono é degenerado NAQUELA ponta»**, que é um facto do segmento e
não de um `t` derivado de uma soma.

#### §11.7.2 — ⚠️ E uma MUTAÇÃO sobreviveu porque a fixtura era simétrica

`t < 0.5` → `t <= 0.0` **sobreviveu** ao gate do caso numérico: a fixtura era uma **reta**, e numa
reta as duas cascatas (`P₂−P₀` e `P₃−P₁`) devolvem a **mesma** direção — cair no ramo errado é
invisível. ⇒ a fixtura passou a ser **assimétrica** (degenerada só no começo), onde o ramo errado
devolve `None`.

*As duas lições são a mesma: **uma fixtura simétrica não distingue os dois lados de uma lei que tem
dois lados.***

**Quatro provas de mutação, todas mortas:** a cascata inteira removida · a ponta final a usar a
cascata do começo · a cascata a abrir só exactamente na ponta · a cerca da cúspide interior caída.

#### §11.7.3 — O que a metade A **não** resolveu, e é a metade B

O desvio das cópias que existem **não se mexeu** (`1,30×`–`1,64×` da meia-altura), e há `1`–`3`
cópias por quadrado acima de `1,2×`. Elas são as que **atravessam** a quina: rígidas, colocadas por
um referencial só, a cortar o canto. ⇒ é aí que entram os quatro modos do §11.3, medidos lado a
lado.

⚠️ **E a régua da metade B ainda não existe:** o `desvio` mede a distância de cada vértice da cópia
ao ponto MAIS PRÓXIMO da guia — e numa quina o lado perpendicular está logo ali, então uma cópia que
saltou para o outro lado do canto lê **`1,00×`**. *A terceira régua desta wave a nascer torta.*

### §11.8 — ⭐⭐⭐ A cura tinha DOIS consumidores a mais, e um deles era uma CERCA COM O PREÇO ESCRITO

`ph2d-arclen` é folha de cinco consumidores. A varredura impactada acusou **dois**:

#### O Zig Zag — a crista estava ACHATADA, e o emissor já o declarava

O emissor escreve `âncora = ponto + normal·lift`, e `normal` é a tangente rodada. Com tangente
**nula** a crista era deslocada por **nada**: ela colapsava sobre o caminho. ⚠️ E o comentário do
emissor **já nomeava a compensação** — *"numa cúspide não há direção: o ponto entra sem
deslocamento, em vez de ser DESCARTADO — descartar tirava uma crista da conta em silêncio"*. Ela
estava certa para uma cúspide **verdadeira** e disparava numa **reta**.

**A/B medido** (`open_corner`, amplitude `15`, caminho de `120`): **4 de 20** vértices, até
`5,07` unidades — *1/3 da amplitude* — e exactamente nos quatro sítios onde `B'` se anulava (as
duas pontas do caminho aberto e as duas quinas). As outras três fixturas são curvas suaves e saem
**byte a byte** iguais.

⇒ *fingerprint* re-pinado com a tabela ao lado, e dois gates novos:
`the_ends_of_an_open_cornered_path_lift` e `only_the_cornered_case_moved_when_the_tangent_cure_landed`.

⚠️ **A 1.ª régua deste lado também nasceu torta** — ela proibia *qualquer* vértice de tocar o
caminho, e acusou **2 de 18** pontos correctos: a onda tem **zeros por construção**. *Uma régua que
proíbe o zero de uma onda proíbe a onda.* O sujeito são as **duas pontas**.

#### O texto em caminho — ⛔⛔ uma CERCA DE CHESTERTON, e ela pedia exactamente este número

O gate `a_stationary_parameterisation_has_no_direction_and_the_text_skips_it` **media o defeito e
defendia-o de propósito**, com o motivo escrito:

> *A cura é geometricamente óbvia […] mas ela mora no `ArcPath::frame_at`, cujo outro consumidor é
> o Zig Zag: ele amostra EXATAMENTE nas âncoras, que são precisamente os pontos estacionários.
> Aplicar a cura faz sangrar o fingerprint […] ou seja, **muda o desenho de um efeito que o Enio já
> aprovou em smoke**. Isso é decisão de produto, não carona de uma wave de texto.*
>
> *Este gate existe para o defeito não ser re-descoberto do zero, e **para a cura não ser aplicada
> sem que alguém veja o que ela custa**.*

⭐⭐ **A cerca foi lida, e o que ela pedia agora existe:** o preço está medido (a tabela acima), e o
que se compra está medido (o pincel deixa de perder `1`–`2` cópias por quadrado, em todo tamanho, e
o texto deixa de saltar a primeira letra de uma reta). O gate foi **invertido**, com a história e o
número dentro dele.

⚠️ **É uma mudança visível num desenho aprovado, e por isso vai NOMEADA na resposta ao Enio** — não
enterrada num diff. *Uma cerca lida e atravessada com o preço na mesa é diferente de uma cerca
derrubada porque estorvava.*

### §11.9 — E o teto de LOC: o ficheiro de gates partiu-se em TRÊS, por responsabilidade

`brush_stroke_engine_tests.rs` passou de `736` para `472` LOC. O corte não foi por tamanho, foi por
pergunta: **`brush_stroke_fixtures.rs`** (o que se CONSTRÓI, `pub(super)`) · o ficheiro do motor (o
que se AFIRMA sobre a lei e o tracejado) · **`brush_corner_tests.rs`** (a W5).

### §11.10 — ✅ **METADE B FECHADA** (2026-08-30): a lei é o **encaixe na PEÇA**, e as quinas partem o contorno

As quatro políticas foram medidas lado a lado com o **desvio de cobertura** — *o quanto a guia que
uma cópia recebeu se afasta da espinha RÍGIDA que ela deita sobre ela*:

| forma | **A** (hoje: um avanço para a volta) | **B** (quebrar nas quinas, encaixe por trecho) | **B'** (quebrar, avanço global) |
|---|---:|---:|---:|
| quadrado 7×7 | `1,273` · 2 fatias más | **`0,000`** | `0,000`, mas **vão de `1,273` na quina** |
| retângulo 12×4 | `1,280` · 3 más | **`0,000`** | `0,000`, **vão `0,960`** |
| estrela 5 pontas | `0,919` · 8 más | **`0,000`** | `0,000`, **vão `0,517`** |

*(em múltiplos da meia-altura da arte; o CONTROLO é um círculo de igual perímetro, que paga `0,091`
— a flecha da corda, o preço inevitável de uma cópia rígida.)*

⇒ **B vence, e não é perto.** O B′ leva a cobertura a zero mas deixa **uma cópia inteira em falta**
em cada quina, que é visualmente a mesma classe de defeito de hoje.

⭐⭐ **A lei generaliza a W3-bis com a palavra certa:** era *«o avanço encaixa no TRAÇO»*, passa a
ser **«o avanço encaixa na PEÇA»** — e uma peça é um trecho que a arte tem de preencher de ponta a
ponta, delimitado por um **vão de tracejado** ou por uma **quina**.

⛔ **E os quatro modos do Illustrator não se medem mais fundo que isto**, porque as diferenças
entre eles são sobre **o ornamento da quina** (que ladrilho pôr lá), e isso exige um **ladrilho de
canto autorado** — recusado no §11.3 com o motivo dos próprios fóruns deles. Na régua que nomeia o
defeito, o B dá **zero**, e nada bate zero.

#### §11.10.1 — O limiar de quina, MEDIDO

`CORNER_MIN_TURN = 1°`, e a vala tem **treze ordens de grandeza**: a viragem máxima de uma âncora
num círculo e numa elipse `3:1` é `≤ 1e-12°` (zero de máquina — as alças são colineares por
construção), e num quadrado é `~90°`, numa estrela `~122°`. ⛔ Não é um botão: um limiar exposto
convidaria a afinar o que a geometria já responde.

⚠️ **Isto só é mensurável desde a metade A**: a viragem sai das **tangentes exactas dos dois
segmentos** (`tangent_at(anterior, 1)` contra `tangent_at(seguinte, 0)`), e até 30/08 as duas eram
**nulas** num contorno de quinas — precisamente onde há quina para medir.

#### §11.10.2 — ⛔⛔⛔ TRÊS RÉGUAS SEGUIDAS NASCERAM TORTAS, e a lição é uma só

| # | a régua | o que ela acusou | por quê |
|---|---|---|---|
| 1 | desvio das cópias **EMITIDAS** | nada | o defeito era uma cópia **não emitida** |
| 2 | distância ao ponto **mais próximo da guia inteira** | `1,00×` numa cópia que saltou o canto | numa quina o lado perpendicular está logo ali |
| 3 | (na cena) distância à **espinha** = dois vértices extremos | `0,96` sobre produto correcto | naquela arte os extremos **não** estão na linha central |
| 4 | (na cena) *«a guia está coberta por arte?»* | `0,73` sobre produto correcto | a arte é uma **folha**: à altura da linha central mede `0,60` de `1,00` de caixa ⇒ há vão **por desenho**, também numa curva suave |

⭐ **A que funciona pergunta outra coisa:** não *«a linha está coberta»*, mas ***«a arte SEGUE a
linha?»*** — a guia da fatia contra a espinha rígida daquela cópia. E na cena, a pergunta
equivalente e barata: ***nenhuma cópia fica centrada em cima de uma quina.***

⚠️ **E a 1.ª versão do gate do produto era CEGA à mutação que importa:** ela reconstruía as fatias a
partir do `corner_arcs` e media a **sua própria** divisão — desligar o corte no produto **não a
movia**. *Um gate que compara duas construções é cego à mutação partilhada; a lei pede a SAÍDA.*

#### §11.10.3 — Os gates e as provas

`the_guide_is_covered_by_the_copies_the_product_emitted` (sobre as cópias emitidas) ·
`the_art_never_cuts_a_corner` · `a_square_gets_every_copy_the_fit_asks_for` (agora **por trecho**) ·
`a_smooth_vertex_is_not_a_corner` · e na cena `every_shape_in_the_corner_smoke_actually_has_corners`
+ `no_copy_in_the_corner_smoke_sits_on_a_corner`.

**Três provas de mutação, todas mortas:** o corte nas quinas desligado · o encaixe de volta a
GLOBAL · um vértice suave a contar como quina.

### §11.11 — O smoke: `PH2D_BUILD_SMOKE=78`

Quatro formas de **bicos** — quadrado (90°) · estrela (dez quinas, agudas e reflexas) · triângulo ·
**retângulo achatado**. ⚠️ **A quarta é a que separa as duas leis**: num quadrado todos os lados
medem o mesmo e um encaixe global e um por-lado dão quase a mesma coisa; num `12 × 4` divergem, e é
ali que se vê o ritmo **por lado**.

⭐ Ela é a **irmã da `=77`** e existe por causa dela: aquela é feita só de curvas suaves, e a
mensagem dela dizia porquê.

---

## §12 — W6: **A OPACIDADE** — a barra que duas das três tintas não obedeciam (2026-08-30)

### §12.1 — O buraco, medido antes de desenhar

A barra *Opacity* (uma no *Fill*, outra no *Stroke*) tem uma **casa** diferente em cada espécie de
tinta. Medido no início da wave:

| tinta | a casa | escrita? | LIDA? |
|---|---|---|---|
| `Solid` (traço e preenchimento) | a alfa da cor | sim | sim |
| `Pattern` no **traço** | `PatternFill::alpha` | sim (plano 35) | sim |
| `Pattern` no **preenchimento** | idem | ⛔ **não** | sim |
| `Brush` no traço | — | sim, na `fallback` | ⛔ **não** |

⇒ **dois buracos, e de espécies DIFERENTES** — que é exactamente a distinção que o `CLAUDE.md` §5
nomeia:

- **O padrão do preenchimento não era ESCRITO.** A guarda que o protegia era certa e incompleta:
  `matches!(p.fill, None | Some(Paint::Solid(_)))` impedia um pick de esmagar um gradiente (report do
  Enio, 2026-07-08) e, **na mesma linha**, deixava o padrão sem nenhum caminho de escrita.
  *Proteger uma tinta de ser destruída não é a mesma coisa que ela ser editável.*
- **O pincel era escrito e NÃO era lido.** O `StrokeStyle::onto` já punha a alfa na `fallback` desde
  a W1, e a `fallback` só se vê **enquanto a arte não resolve** ⇒ com arte, o artista arrastava a
  barra de ponta a ponta sem mudar um pixel. É *o consumidor que projecta o valor fora*, e ⛔ nenhuma
  sonda de *«quem lê este campo?»* o vê: ele **é** lido.

**Red-first medido:** `255` onde se pediu `128`, nos quatro gates do motor.

### §12.2 — A lei: *uma opacidade, uma casa*

A frase já estava escrita no `paint_bind::fade` e no `vector_bridge_style` para o padrão; a W6
completa a tabela.

| tinta | a casa | porquê |
|---|---|---|
| `Solid` | a alfa da cor | não há outro sítio |
| `Pattern` | `PatternFill::alpha` (`f32`) + `fallback.a` em sincronia | o amostrador do Vello quer um `f32` |
| **`Brush`** | **`fallback.a`, e mais nada** | as cópias são geometria `Rgba8`; não há amostrador |

⛔ **O pincel NÃO ganhou um campo `alpha` próprio, e a assimetria é a resposta certa.** Um campo novo
seria uma **segunda casa** para o mesmo número, e pagaria um degrau do `VEC_SCENE_SCHEMA_VERSION` por
nada. ⭐ E há um sinal de que o sítio está certo: **a dívida declarada no `paint_bind` fechou sem uma
linha lá.** Ela dizia *«um pincel desvanece a cor de recurso, e as CÓPIAS ainda não»*; com o motor a
ler `fallback.a`, escalar a `fallback` passou a **ser** escalar as cópias.

### §12.3 — Onde a cura entrou (quatro sítios, três deles uma PORTA)

1. **`ph2d_vec_scene::brush_copies`** — desvanece a arte escalada **uma vez** (`O(1)`, não
   `O(cópias)`) por `crate::paint_bind::fade`, que passou a `pub(crate)`. ⭐ *Desvanecer toda a tinta
   de um `VecPath` é uma pergunta só*, e uma 2.ª cópia dela divergiria na primeira espécie de tinta
   nova. ⚠️ **Sem guarda de `< 255`:** `(255·255 + 127)/255` é `255`, logo o topo da barra é a
   identidade **pela própria conta** — um `if` a proteger o que a aritmética já protege é uma segunda
   lei a manter em sincronia.
2. **`style::recolour_pattern`** — a cor + a opacidade num padrão, **uma porta com dois consumidores**
   (o traço pelo `onto`, o preenchimento pelo `apply_fill_colour`). A prova de que ela serve os dois:
   a mutação que lhe tira o `alpha` mata **3** gates, um deles pré-existente.
3. **`style::apply_fill_colour`** — *o que um pick faz a esta tinta*, e **detector e escritor de uma
   vez** (chamada num clone, responde *«difere?»*). ⚠️ A versão anterior tinha a decisão escrita
   **duas** vezes — a condição do `differs` e o `else if` do escritor — e a do padrão faltava numa
   delas. ⭐ A extracção absorveu a decisão inteira: o `new_fill: Option<Rgba8>` pré-digerido
   (`alfa 0 => None`) ficou **morto**, e era a convenção do *sólido* aplicada **antes de saber a
   espécie da tinta**.
4. **`style::seed_fill_from_paint`** — o espelho: o que a ferramenta ADOTA ao seleccionar.

⛔⛔ **As duas metades do preenchimento de padrão são independentes, e a de cima sozinha é PIOR que o
buraco.** Sem a semente, o `seed_style_from_selection` deixava um padrão passar (`Some(_) => {}`) e a
ferramenta ficava com a cor e a alfa da forma **anterior**; com a porta de escrita aberta, a primeira
mexida em qualquer controlo jogaria essa alfa alheia no padrão. ⚠️ E a semente lê o **`alpha`**, não a
`fallback.a`: um ficheiro gravado antes da W6 tem a `fallback` opaca e o `alpha` autorado noutro
sítio, e o que se DESENHA é o `alpha`. *Semear pelo campo que não se vê poria a barra a mentir sobre
o ficheiro do artista.*

### §12.4 — As duas fronteiras que a wave declarou

- ⭐ **Alfa zero num padrão ESCONDE-O; não o apaga.** Num sólido, alfa zero significa *«sem
  preenchimento»* e a tinta desaparece — a convenção que a ponte já usa nos dois sentidos, e não é
  desta wave mudá-la. Num padrão a mesma leitura destruiria a grade, o ladrilho, a rotação e a fonte
  por um arrasto acidental até ao fundo. *Uma convenção herdada aplica-se onde não custa nada; onde
  custa, ela é a pergunta.*
- ⛔ **Um gradiente continua intocável.** A W6 abriu uma porta de escrita; escrita como *«tudo o que
  não é sólido recebe a cor»* teria reaberto a regressão de 2026-07-08. Há gate.
- ⭐ **A opacidade MULTIPLICA a da arte, nunca a substitui** (arte a `200` sob barra a `128` sai a
  `100`). Substituir daria `128` e **subiria** a opacidade de uma arte que o artista fez translúcida.

### §12.5 — Os gates e as provas

**Motor** (`brush_opacity_tests.rs`, 5): `the_opacity_slider_reaches_the_brush_copies` ·
`a_fully_opaque_brush_leaves_the_art_byte_identical` (o controlo) ·
`the_opacity_scales_the_arts_own_alpha_it_does_not_replace_it` · `the_arts_own_stroke_fades_with_its_fill` ·
`the_live_fade_reaches_the_copies_through_the_fallback`.

**Ponte** (`vector_bridge_opacity_tests.rs`, 8): as duas do pincel (a escrita + *o detector concorda
com o escritor*), `the_fill_opacity_reaches_a_patterned_fill`,
`dragging_a_patterns_opacity_to_zero_hides_it_instead_of_deleting_it`,
`a_gradient_fill_is_still_never_clobbered_by_a_pick`, `an_open_path_takes_no_fill`,
`selecting_a_patterned_fill_seeds_the_opacity_from_what_is_drawn`,
`seeding_a_pattern_and_writing_it_back_changes_nothing`.

**Cena** (`paint_opacity_smoke_tests.rs`, 7).

⚠️ **Red-first não era estruturalmente possível para três das quatro curas** — as funções não
existiam, logo o gate não compilava contra o código velho. ⇒ **oito provas de mutação, oito mortes**,
com o controlo verde no fim de cada uma:

| mutação | matou |
|---|---|
| as cópias não desvanecem | 4 |
| o padrão do preenchimento não recebe a opacidade | 2 |
| `recolour_pattern` escreve a cor e **não** o `alpha` | **3** (um pré-existente) |
| a semente lê a `fallback` em vez do que se DESENHA | 1 |
| um gradiente deixa de estar protegido | 1 |
| as três colunas da cena nascem todas opacas | 2 |
| as faixas claras vão para cima das formas | 1 |
| o herói passa a ser o índice literal | 1 |

### §12.6 — O smoke: `PH2D_BUILD_SMOKE=79`

Duas fileiras de três — em cima a **estampa**, em baixo o **pincel** —, a mesma tinta em `100 %`,
`50 %` e `15 %`.

⚠️⚠️ **A FAIXA CLARA por trás de cada fileira é load-bearing.** Sobre um fundo neutro, *mais
transparente* e *mais escuro* desenham-se quase igual, e a cena provaria o defeito tão bem quanto a
cura. Há gate a exigir que a faixa esteja empilhada **antes** de toda forma que ela torna legível —
e a régua é a **relação** (a faixa que cobre o centro da forma vem antes dela), ⛔ nunca o índice `0`.

⭐ **A coluna opaca é o CONTROLO**: uma cura que escurecesse tudo por sistema daria três valores
distintos na mesma ordem e passaria sem essa linha.

⭐ **O herói é DERIVADO** (*a primeira forma com estampa*), nunca um índice — as faixas e a arte do
pincel nascem antes, e um literal poria o painel na secção errada no dia em que a cena ganhasse mais
uma forma, **sem erro nenhum**.

⚠️ **A arte da estampa é opaca por inteiro**, ao contrário da da `=76`: ali o alfa da arte é o
sujeito; aqui seria uma **segunda** fonte de transparência a somar-se à que a cena mede.

⚠️ **E o enquadramento é uma cerca com gate** — não uma medição do viewport, e está declarado como
tal: a barra é a extensão da `=76`, que o Enio já smokou. Ela apanhou a 1.ª redacção dos três `ROW_*`
(a faixa clara é `1,2 ×` a altura das formas e saía `0,58` acima do topo provado visível).

⛔ **Recusas MEDIDAS desta wave**

| recusa | mecanismo |
|---|---|
| `BrushStroke::alpha` como campo próprio | segunda casa para o mesmo número + degrau de `VEC_SCENE_SCHEMA_VERSION` por nada; a `fallback.a` já recebia a barra (§12.2) |
| guarda `if a < 255` antes do `fade` | o topo já é a identidade **pela conta**; o `if` seria uma 2.ª lei a manter em sincronia (§12.3) |
| alfa zero apagar um padrão, como faz num sólido | destruiria grade, ladrilho, rotação e fonte por um arrasto acidental (§12.4) |
| a porta de escrita como *«tudo o que não é sólido recebe a cor»* | reabriria a regressão do gradiente de 2026-07-08 (§12.4) |
| semear a barra pela `fallback.a` | um ficheiro pré-W6 tem a `fallback` opaca e o `alpha` autorado — a barra mentiria sobre o ficheiro (§12.3) |

### §12.7 — A lente que procurou o QUARTO buraco, e não o achou

A wave curou dois consumidores. A pergunta de auditoria é *«há um terceiro sítio onde a opacidade
atravessa e se perde?»* — e o candidato óbvio é o **nascimento** de uma tinta: a fileira *Type*
(`vec_stroke_paint.rs`), que troca `Solid ⟷ Pattern ⟷ Brush`.

⭐ **Não há buraco: os três braços já carregam a alfa.** O braço `Pattern` escreve
`f.alpha = cur.color().a / 255` com a lei nomeada (*"uma opacidade, uma casa — inclusive no
nascimento"*), e o braço `Brush` escreve `fallback: cur.color()`, **que inclui a alfa**, com o
comentário a explicar que sem ela a linha *saltaria para preto*.

⚠️ **Mas a razão pela qual o braço `Brush` estava certo era OUTRA** — ele preservava a **cor**, e a
alfa vinha junto por acidente de tipo. Até esta wave essa alfa não tinha consumidor nenhum, então a
correcção era invisível: ir e voltar preservava um número que não desenhava nada. *Uma linha pode
estar certa por um motivo e passar a estar certa por dois.*

### §12.8 — O que a W6 NÃO tocou, declarado

- **`VEC_SCENE_SCHEMA_VERSION` não se mexe** (fica em `17`): nenhum campo novo, nenhuma variante
  nova. ⇒ o `PROJECT_SCHEMA` também não.
- **O caminho opaco é byte-idêntico**: `fade(art, 255)` é a identidade pela conta, e há gate
  (`a_fully_opaque_brush_leaves_the_art_byte_identical`).
- **O gradiente** e o **caminho aberto** continuam a recusar um pick, cada um com gate.

### §12.9 — A mudança de comportamento que a semente traz, nomeada

⚠️ **Escolher uma forma com estampa passa a SEMEAR a ferramenta** (cor de recurso + a opacidade que
ela desenha). Consequência visível: desenhar uma forma **nova** logo a seguir herda essa cor, em vez
da última que o artista escolheu à mão.

⭐ **É a lei que o preenchimento SÓLIDO já obedece** — *"a ponte LÊ na selecção o que ESCREVE no
apply"* —, e é ela que faz a barra *Opacity* mostrar a verdade. A alternativa (não semear) é
precisamente o defeito que esta wave cura: a barra ficava a mostrar o valor da forma **anterior**.

⛔ **O gradiente continua a não semear** (`None`): ele tem alça própria, e o `apply` também não lhe
toca — as duas metades continuam a concordar.

⚠️ **Caso de borda declarado:** uma estampa a opacidade `0` semeia alfa `0`, e a forma seguinte nasce
**sem preenchimento**. É o mesmo que um sólido a `0` já fazia, e a convenção não muda aqui.

### §12.10 — As quatro condições de alcance, conferidas

| condição | veredito |
|---|---|
| o componente EXISTE | `ids::VECTOR_FILL_OPACITY` / `VECTOR_STROKE_OPACITY` |
| é PINTADO e REGISTADO | `paint_sections.rs` pinta a fileira **incondicionalmente** (não depende da espécie da tinta) e o `populate_style.rs` regista o par slider+caixa |
| o clique CHEGA ao barramento | `event.rs` roteia os dois ids |
| a SEQUÊNCIA leva a algum lugar | ⛔ **era aqui que falhava**, nas duas tintas — e é o que a W6 fecha |

*O knob estava pintado, registado e alcançável pelo dedo. O que faltava era o último passo, que
nenhuma sonda deste repo pergunta.*

### §12.11 — O portão de fecho, e a família de flake outra vez

`20 191` testes, **`20 187` verdes**. As `4` vermelhas são **flakes de recurso sob fan-out**, e a
evidência é a que o `CLAUDE.md` §5.0 nomeia, inteira:

| sinal | leitura |
|---|---|
| o **CONJUNTO de reprovadas MUDOU entre duas corridas do mesmo binário** | a 1.ª varredura reprovou `a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs`, que na 2.ª passou; estas quatro passaram na 1.ª | 
| as quatro passam **sozinhas** (`--test-threads=1`) | `4/4` |
| o diff não tem **uma linha** nas crates delas | `ph2d-mesh` · `ph2d-timeline` · `ph2d-vec-boolean` · `ph2d-wet-paint` |
| três das quatro já estão **nomeadas** na lista do §5.0 | `the_region_refresh_…` · `the_cost_of_depth_…` · `a_round_live_offset_…` (o caso canónico) |

⭐ **A quarta — `glaze_layering_costs_a_ratio_not_an_order_of_magnitude` (`ph2d-wet-paint`) — é um
membro NOVO, e traz a prova mais limpa que esta linha já viu:** o **irmão** dela,
`pigment_mixing_costs_a_ratio_not_an_order_of_magnitude`, corre no mesmo ficheiro, mede a mesma
espécie de razão e **passou na MESMA corrida**. *Dois gates gémeos a discordar dentro do mesmo
binário não podem ambos estar a medir o código.*

⚠️ **E o §5.0 diz para parar de as contar uma a uma** — a forma que se lê é o MECANISMO (um gate que
divide dois relógios), não o nome. Esta fica registada aqui porque a corrida a produziu, não porque a
lista precise de crescer.

⚠️⚠️ **NOTA DE PROCESSO, e é a 4.ª vez nesta linha:** a 1.ª re-corrida destas quatro compilou em
`/home/enio/Documentos/Projetos/PH2D/crates/…` — **a árvore primária**. A `cwd` do shell escorrega
para o primário entre chamadas, e o mesmo caminho relativo existe nas duas árvores, então o comando
**corre sem erro nenhum** e mede a árvore errada. ⇒ *todo comando começa com o `cd` absoluto da
worktree*, mesmo quando o anterior já lá estava.
