# Plano 35 — **padrão no TRAÇO** (o *"as a fill or stroke"* do Figma)

> Ordem do Enio, 2026-08-27. É o último item da lista que o [plano 33 §7](33_plano_texture_pattern.md)
> deixou em ⏸️, e a condição que ele escrevia — *"entra depois de o preenchimento fechar"* — foi
> cumprida: as dez waves do padrão de preenchimento estão fechadas.

---

## §0 — O que foi MEDIDO antes de desenhar (worktree `line/Vector`, 2026-08-27)

### §0.1 — ⭐⭐⭐ O tamanho decide a representação, e o número é decisivo

Sonda `measure_the_paint_sizes` ([`paint_pattern_tests.rs`](../../crates/ph2d-vec-scene/src/paint_pattern_tests.rs),
`#[ignore]`, corre com `-- --ignored --nocapture`):

| Tipo | bytes |
|---|---|
| `Rgba8` | 4 |
| `Paint` | **56** |
| `PatternFill` | **112** |
| `StrokeSpec` | **64** |
| `VecPath` | **208** |
| `Option<Box<PatternFill>>` | **8** |
| `Option<PatternFill>` | **112** |

⇒ **A escolha entre guardar o `PatternFill` em linha ou atrás de um `Box` não é estilo:**

| | `StrokeSpec` | `VecPath` | `StrokeSpec: Copy`? |
|---|---|---|---|
| **em linha** | 64 → **176** (×2,75) | 208 → ~320 (**+54 %**) | ✅ sim (o `PatternFill` é todo `Copy`-able) |
| **em `Box`** | 64 → **72** (+12 %) | 208 → ~216 (**+4 %**) | ⛔ **não** |

⚠️ **Todo `VecPath` entra em TODA fotografia de undo**, inclusive os que não têm padrão nenhum — é
exactamente a conta que o gate `the_paint_enum_does_not_grow_when_pattern_lands` (W3) já defende
para o `Paint`. ⇒ **`Box`**, e o preço é o `Copy`.

### §0.2 — ⭐ E o preço do `Copy` foi MEDIDO tirando-o e contando, não estimado

⚠️ **A nota do plano 33 §7 dizia *"287 menções em 13 crates, das quais 22 sítios copiam-no para fora
de um place e quebram"*. As 287 são menções — o número que interessa é outro, e o compilador é quem
o dá.** Removendo o `Copy` do `StrokeSpec` e iterando `cargo check --workspace --all-targets`:

| Onda | Erros | Onde |
|---|---|---|
| 1.ª | **3** | `ph2d-vec-scene`: `compound.rs:294`, `path_cut.rs:117`, `pattern_path.rs:316` — todos `stroke: X.stroke` num construtor ⇒ `.clone()` |
| 2.ª | **2** | `ph2d-vec-render`: `standalone.rs:47`, `lib.rs:469` — `if let Some(s) = path.stroke` ⇒ `.as_ref()` |
| 3.ª | **8** | `ph2d-vec-boolean`: `pathfinder.rs:207`, `cut.rs:133,210`, `expand_ribbon.rs:41`, `expand_ring.rs:98`, … |

⇒ **da ordem de 15–30 sítios mecânicos em ~6 crates**, e cada um é `.clone()` ou `.as_ref()`. Não é
uma jornada; é uma wave com cauda mecânica. ⛔ *A cifra que assustava media a palavra, não o dano.*

### §0.3 — E os leitores da COR do traço são ~15 ficheiros, não 72

`grep` de `stroke.color` / `s.color` / `spec.color` dá **72 acertos**, mas a maioria é de outro
assunto (`ph2d-painter-brush` 8, `ph2d-color` 3, `ph2d-mesh` 1 — `s` é outra coisa lá). Os do vector
são **~15 ficheiros**: `ph2d-vec-scene` 3 · `desktop` 5 · `ph2d-vec-render` 2 · `ph2d-vec-boolean` 2 ·
`ph2d-vec-edit` 1 · `ph2d-tool-vector` 1 · `ph2d-vector` 1.

⭐ **E quase todos ficam de graça** com um acessor: `StrokeSpec::color()` devolve a cor sólida (ou a
`fallback` do padrão), e `s.color` → `s.color()` é uma mudança de um caractere que **mantém o
sentido** — quem só quer uma cor para uma swatch continua a ter uma resposta honesta.

---

## §1 — Pesquisa: o estado da arte, e o que cada um ABANDONOU

| | Como o traço recebe uma tinta que não é uma cor | O que isso custou |
|---|---|---|
| **SVG** | `stroke` é um *paint*: `url(#pattern)` tal como o `fill`. **Um** conceito, dois sítios. | — (o modelo de que todos descendem) |
| **Figma** | *"use patterns **as a fill or stroke**"* — a mesma lista de tintas nos dois. | ⭐ nada a apontar: é o alvo. |
| **Illustrator** | O padrão é um *swatch*, e um swatch serve fill **e** stroke. ⚠️ E ele **não escala com o traço**: engrossar a linha não engrossa o motivo. | ⚠️ A queixa clássica dos fóruns — *"my pattern brush doesn't scale"* — é sobre **outra** feature (o *Pattern Brush*), e é o aviso de que **duas coisas com o mesmo nome confundem**. |
| **Inkscape** | *Stroke paint* tem o botão de **padrão** ao lado do de cor plana e do de gradiente. | ⚠️ O padrão dele é um objecto do `<defs>` e a UI dele é notoriamente difícil de alcançar. |

⭐⭐ **A síntese:** o traço tem uma **tinta**, e a tinta é uma **lista fechada**. Ninguém trata
"padrão no traço" como uma feature separada — é a mesma tinta noutro sítio.

⛔ **E o que NÃO se copia:** o *Pattern Brush* do Illustrator (um motivo que percorre o caminho e
roda com ele) é outra coisa, e nesta casa **já existe** — é o
[Pattern Along Path](23_plano_pattern_along_path.md), plano 23. Confundi-los seria dar dois nomes à
mesma coisa e uma palavra a duas.

---

## §2 — O DESENHO

### §2.1 — ⭐ `StrokePaint`, e **não** o `Paint` do preenchimento

```rust
pub enum StrokePaint { Solid(Rgba8), Pattern(Box<PatternFill>) }
// StrokeSpec.color: Rgba8   →   StrokeSpec.paint: StrokePaint
```

⛔ **Reusar o `Paint` está RECUSADO, e o motivo é uma armadilha concreta:** o `Paint` tem
`Linear`/`Radial`/`MultiPoint`, e o renderer de traço **não os desenha**. Um modelo que representa o
que o desenho não faz produz um documento que se grava, recarrega e pinta errado — *estado
inalcançável, gravado*. Quando um gradiente no traço for pedido, o `StrokePaint` **ganha uma
variante** (append-only, um degrau).

⭐ **Uma tinta, uma porta.** ⛔ A alternativa barata — manter `color` e apendar
`pattern: Option<..>` — dá ao traço **duas fontes de tinta** que podem discordar, e o sintoma é a
swatch a mostrar uma cor enquanto a linha desenha outra coisa. É o defeito de duas-portas contra o
qual o plano 33 inteiro foi escrito.

### §2.2 — As portas únicas

| Pergunta | Porta |
|---|---|
| *Com que tinta este traço desenha?* | `StrokeSpec::paint` (o enum) |
| *Que COR representa este traço?* (swatch, token, `StrokeStyle`) | `StrokeSpec::color()` — sólida, ou a `fallback` do padrão |
| *Como uma imagem preenche uma FAIXA?* | `VectorScene::stroke_path_image` — irmã do `fill_path_image` da W2, e pelo mesmo motivo: o `brush_transform` do peniko está morto na porta actual |
| *Que ladrilho este padrão tem hoje?* | o memo `texture_pattern_live`, que passa a varrer **fill e stroke** |

### §2.3 — ⚠️ A colocação do padrão do traço é a MESMA lei do preenchimento

O `brush_transform` é composto como `transform * brush_transform` — a mesma conta, no mesmo espaço
local. ⇒ o `PatternFill` do traço usa `placement_in` sem uma linha nova, e **roda e escala com a
forma** exactamente como o do preenchimento.

⚠️ **E ele NÃO escala com a LARGURA do traço** — que é a queixa que o Illustrator colhe há anos. A
largura decide a *faixa*; o padrão decide o que a preenche. São duas grandezas, e juntá-las faria
engrossar a linha mudar o motivo.

### §2.4 — A UI: **o alvo é um chip, não uma secção nova**

A secção *Pattern* hoje edita o padrão do **preenchimento**. Com um padrão no traço ela passa a ter
**dois sujeitos possíveis** ⇒ ganha uma fileira `Fill | Stroke` no topo, e edita o que estiver aceso.

⛔ **Duplicar a secção está recusado:** onze fileiras a dobrar, e as duas divergiriam no primeiro
knob novo. ⚠️ O chip do alvo **só aparece quando os dois existem** — com um só, não há escolha a
oferecer (a lei do `Option<bool>` que a caixa *Stroke* e o *Resize Box* já obedecem).

E a secção *Stroke* ganha a fileira de TIPO (`Solid | Pattern`), irmã do *Fill Type* — clicar em
`Pattern` sem ter um **abre a porta da arte**, que é a 4.ª condição que o plano 33 §4 já resolveu
para o preenchimento.

### §2.5 — Schema

`StrokeSpec.color: Rgba8` → `paint: StrokePaint` **muda o formato do fio** (o postcard é posicional).
⇒ `VEC_SCENE_SCHEMA_VERSION` **+1**, `PROJECT_SCHEMA` **+1** por arrasto, a **tripla**, e o degrau da
escada. ⚠️ **Conte os três contra o `main` do dia** — esta linha já os moveu uma vez (99→100) e a
`line/components` mexeu no mesmo degrau em 26/08.

---

## §3 — As waves

| Wave | Entrega | Onde | Schema? |
|---|---|---|---|
| **A** | `StrokePaint` + `StrokeSpec.paint` + `color()`, e a cauda mecânica do `Copy` | `ph2d-vec-scene` + `-render` + `-boolean` + `-edit` | **sim** |
| **B** | `stroke_path_image` + o traço a desenhar com o ladrilho | `ph2d-vector` + `ph2d-vec-render` | não |
| **C** | O memo varre o traço; o ladrilho do traço assa como o do preenchimento | shell | não |
| **D** ✅ | A UI: tipo do traço (`Solid \| Pattern`) + o alvo (`Fill \| Stroke`) na secção *Pattern* | `ph2d-panel-vector` + shell | não |
| **E** ✅ | Persistência + smoke + gates + mutações | shell | não |

**Kill-criterion (DIRETIVA §5):** o desenho promete que **um traço com padrão custa o que um traço
sólido custa** — uma chamada de `stroke()`, zero camadas de clip. Se a wave B medir mais do que isso
no `Encoding`, o desenho está errado e o passo seguinte é achar porquê. ⛔ Não subir a barra.

---

## §4 — Os gates, red-first

| # | Gate | O defeito que ele mata |
|---|---|---|
| 1 | `a_stroke_can_carry_a_pattern` | o buraco inteiro |
| 2 | `the_stroke_pattern_survives_the_save` | o formato |
| 3 | `the_stroke_pattern_costs_one_draw_call` | uma camada de clip a entrar sem ninguém ver |
| 4 | `the_stroke_pattern_does_not_scale_with_the_stroke_width` | a queixa do Illustrator, do lado certo |
| 5 | `the_stroke_colour_still_answers_for_a_patterned_stroke` | a swatch/token a ficar sem resposta |
| 6 | `the_pattern_section_edits_the_target_that_is_lit` | a secção a editar o preenchimento com o traço aceso |
| 7 | `the_target_chip_only_shows_when_both_exist` | um chip que não tem escolha a oferecer |

⚠️ **A fixtura tem de conter os DOIS**: uma forma com padrão só no preenchimento, e uma com padrão
nos dois. Um gate só sobre a primeira passa com a wave inteira por construir.

---

## §5 — O que este plano NÃO faz

- ⛔ **Gradiente no traço** — o `StrokePaint` deixa a porta aberta (uma variante), mas representar o
  que o renderer não desenha é estado inalcançável gravado.
- ⛔ **Padrão que escala com a LARGURA** (§2.3).
- ⛔ **Confundir com o *Pattern Along Path*** ([plano 23](23_plano_pattern_along_path.md)) — o
  motivo que percorre a guia já existe e é outra feature.
- ⏸️ **Um padrão partilhado entre fill e stroke** (editar um muda o outro). Hoje são dois
  `PatternFill` independentes; partilhar exige um id de recurso, que é o *navegador de assets* que o
  ADR-0165 adia.

---

## §6 — O que as waves D e E MEDIRAM (2026-08-28)

### §6.1 — ⛔⛔ Um defeito de PERSISTÊNCIA que o plano não previa

A colheita que embute a arte de um padrão no `.ph2dproj`
([`project_texture_pattern.rs`](../../shells/desktop/src/project_texture_pattern.rs)) **lia só o
`path.fill`**. ⇒ uma forma cujo **traço** tinha padrão gravava o `AssetId` no documento e **nunca os
pixels**: reabrir o projecto noutra sessão dava uma linha pintada a **cor de recurso**, para sempre e
**sem erro nenhum a que agarrar** — o defeito exacto que aquele ficheiro existe para curar, com o
sujeito trocado.

⚠️ **Ele não estava na lista de gates do §4**, e nenhum dos seis primeiros o alcançava: a wave A
gateou o round-trip da `VecScene` (que carrega o `StrokePaint` **inteiro**, incluindo o `AssetId`), e
os pixels viajam **noutro blob**. *Um round-trip verde sobre o documento não diz nada sobre os
recursos que o documento NOMEIA.*

⭐ A cura veio com a metade **PURA** extraída (`art_ids_named_by`), porque a colheita precisa do
`AssetDb`, que vive num `AppGfx` que segura uma surface de janela real — sem a extracção, a única
defesa possível seria um gate de FONTE a procurar a palavra `stroke`.

### §6.2 — ⚠️ Um gate reprovou PRODUTO CORRECTO, e o defeito era a agulha dele

`every_shape_in_the_pattern_smoke_is_born_with_a_stroke` (plano 34) contava
`stroke: Some(contorno())` — a **grafia** de uma porta, não a lei. A wave D acrescentou um segundo
construtor (`contorno_com_padrao`) e o gate reprovou duas formas que **nascem vestidas**, só que por
outra porta.

⇒ a agulha passou a ser `stroke: Some(`, mais um controlo `stroke: None == 0`. *Um gate que fixa o
nome de quem obedece à lei reprova a segunda maneira de a obedecer.*

### §6.3 — O ALVO é uma preferência COAGIDA, nunca um estado

`texpat_target` vive na sessão (como o cadeado), e **toda** leitura passa por
`texture_pattern_edit::lit_target`, que a coage ao que a forma de facto tem. Sem a coerção, escolher
*Stroke* numa forma e clicar noutra faria a secção **desaparecer** por se lembrar de uma escolha
feita algures — e a escrita cairia num slot vazio, um **no-op silencioso** com a secção a mostrar o
outro sujeito.

⭐ **Uma função responde às DUAS perguntas** (*quem é o sujeito?* e *mostro o chip?*): separá-las é
como as duas respostas passam a discordar, e o sintoma seria o chip a aparecer com um alvo só.

### §6.4 — As nove provas de mutação

| Mutação | Gate que morreu |
|---|---|
| `lit_target`: `(true, false) => want` | `a_sticky_target_is_coerced_to_what_the_shape_actually_has` |
| `lit_target`: `fill \|\| stroke` | `the_target_chip_only_shows_when_both_exist` |
| `write_pattern`: o braço `Stroke` nunca escreve | `the_pattern_section_edits_the_target_that_is_lit` |
| `set_kind`: `Solid` com uma cor fixa | `going_back_to_solid_keeps_the_colour_the_line_was_already_showing` |
| a fileira *Type* pintada sem tinta publicada | `the_stroke_type_row_is_absent_without_a_stroke` |
| o chip do alvo pintado com um sujeito só | `the_target_row_is_absent_when_there_is_no_choice_to_offer` |
| `stroke_draw`: o pincel a escalar com `s.width` | `the_stroke_pattern_does_not_scale_with_the_stroke_width` |
| a colheita a ignorar o traço | `the_art_of_a_patterned_stroke_travels_in_the_file_too` |

Cada uma com os **três controlos**: verde antes · a agulha casa exactamente uma vez · verde depois do
restore (com `utime`, senão o cargo fica *stale*).

### §6.5 — Os 7 gates do §4, e onde eles estão

| # | Gate | Onde |
|---|---|---|
| 1 | `a_stroke_can_carry_a_pattern` | `ph2d-vec-scene/src/paint_pattern_tests.rs` |
| 2 | `the_stroke_pattern_survives_the_save` + `the_art_of_a_patterned_stroke_travels_in_the_file_too` | idem + `shells/desktop/src/project_texture_pattern_tests.rs` |
| 3 | `a_patterned_stroke_pushes_no_clip_layer` | `ph2d-vec-render/src/pattern_tests.rs` |
| 4 | `the_stroke_pattern_does_not_scale_with_the_stroke_width` | idem |
| 5 | `the_stroke_colour_still_answers_for_a_patterned_stroke` | `ph2d-vec-scene/src/paint_pattern_tests.rs` |
| 6 | `the_pattern_section_edits_the_target_that_is_lit` | `shells/desktop/src/texture_pattern_edit_tests.rs` |
| 7 | `the_target_chip_only_shows_when_both_exist` | idem |

Mais a costura das duas fileiras novas
([`seam_stroke_paint.rs`](../../crates/ph2d-panel-vector/tests/seam_stroke_paint.rs), gesto REAL) e os
quatro sítios da shell
([`the_stroke_paint_row_is_wired.rs`](../../shells/desktop/tests/the_stroke_paint_row_is_wired.rs)).

---

## §7 — ⛔⛔ O REPORT DE 2026-08-28: *"se ajustar width sai de pattern e vai para solid"*

### §7.1 — A causa: uma INVARIANTE que envelheceu quando o modelo cresceu

`StrokeStyle::onto` ([`vector_bridge_style.rs`](../../shells/desktop/src/render_loop/vector_bridge_style.rs))
reconstruía o `StrokeSpec` inteiro e escrevia `paint: StrokePaint::Solid(self.color)` — **por desenho
explícito**, com a razão escrita ao lado:

> *"a ficha + a largura determinam o traço INTEIRO — nada do spec antigo sobrevive, e é isso que
> garante que os dois lados (detectar / gravar) não possam divergir num campo esquecido."*

⚠️⚠️ **Era verdade enquanto o traço tinha UMA tinta possível.** A wave A deu-lhe duas, e a mesma
frase passou a significar *"toda edição de geometria do traço destrói a tinta autorada"*.
*Uma invariante é uma afirmação sobre o modelo do dia em que foi escrita — quem alarga o modelo tem
de a reconferir* (§0.0 do roteador, aplicado a uma invariante em vez de a um teto).

⭐ **E o report nomeou o Width, mas o defeito era a PORTA:** cap, join, alinhamento, tracejado e as
duas pontas passam pela mesma `onto` e destruíam o padrão cada um sozinho. Gate:
`no_stroke_geometry_control_destroys_the_pattern`.

### §7.2 — ⭐ A lei certa já existia nesta casa, e também veio de um report

No **preenchimento**, escrita no [`vector_bridge.rs`](../../shells/desktop/src/render_loop/vector_bridge.rs)
desde 2026-07-08:

> *"a fill pick only replaces a Solid / None fill; it must NEVER clobber a gradient (use Fill Type ->
> Solid for that). Guards the linear/radial->solid regression (Enio 2026-07-08)."*

O traço nunca a aprendeu porque, até 26/08, ele não tinha o que houvesse a preservar. ⇒ **a ficha da
ferramenta possui uma COR, nunca a TINTA**: ela escreve a cor *dentro* da tinta que já lá está, e a
espécie da tinta só muda pela porta explícita — a fileira *Type*, da wave D.

### §7.3 — ⚠️ *"Nunca esmagar"* não pode virar *"não fazer nada"*

A swatch **mostra** a cor de recurso de um padrão (`StrokeSpec::color()` responde-a desde a wave A).
Uma swatch que mostra um valor e não o muda é o controlo morto que esta linha caça há três waves — e
faria o `differs_from` disparar **todo quadro**, com cada um a virar um passo de undo. ⇒ a cor
escreve a `fallback`.

⭐⭐ **E a OPACIDADE é a metade que se VÊ, medida:** com um ladrilho, o desenho lê
`PatternFill::alpha` e a alfa da `fallback` só aparece enquanto a arte não resolve
([`stroke_draw.rs`](../../crates/ph2d-vec-render/src/stroke_draw.rs)) ⇒ escrever só a cor deixaria a
barra *Opacity* a andar **sem mudar um pixel**. A lei já estava escrita palavra por palavra no
`paint_bind::fade` (*"um padrão não tem cor para escalar — tem OPACIDADE, e as duas descem
juntas"*), ali para a sobreposição derivada; agora vale para a **autoria**.

⚠️ **Uma opacidade, uma casa** — e **inclusive no nascimento**: um traço a 50% que vira padrão
nasceria com o `alpha = 1,0` do construtor e **saltaria** para opaco no clique
(`vec_stroke_paint::set_kind`).

### §7.4 — ⏳ O IRMÃO SIMÉTRICO, medido e NÃO curado: a opacidade do PREENCHIMENTO

`fill_differs` exclui um `Paint::Pattern` pelo `matches!(p.fill, None | Some(Paint::Solid(_)))`
⇒ **a barra *Opacity* do preenchimento também está morta sobre um padrão**, e isto é
**pré-existente** (plano 33), não uma regressão desta wave.

⛔ **Não foi curado, e a razão é que o sujeito não está decidido:** para o traço, a ferramenta
**adopta** a cor da forma seleccionada (`adopt_stroke` lê `stroke.color()`), então escrever a
`fallback` é a ida-e-volta de um valor que o painel já mostra. Para o preenchimento,
`seed_style_from_selection` faz `Some(_) => {}` numa tinta não-sólida — a swatch mostra a **última
cor autorada**, não a do padrão. Escrever essa cor no padrão faria uma cor velha saltar para dentro
dele na primeira mexida do slider.

⇒ curar exige primeiro decidir **o que a swatch do *Fill* significa sobre um padrão**, que é
decisão de produto. As duas saídas: (a) a swatch adopta a `fallback` do padrão, como o traço faz, e
a lei fica idêntica nos dois; (b) só a ALFA atravessa, e a cor do padrão só se muda pelo *Source…*.

### §7.5 — As cinco provas de mutação

| Mutação | Gate que morreu |
|---|---|
| `onto`: o braço `Pattern` volta a `Solid(self.color)` | `adjusting_the_stroke_width_never_destroys_the_pattern` |
| idem | `no_stroke_geometry_control_destroys_the_pattern` |
| `onto`: a `fallback` não recebe a cor | `the_colour_still_reaches_a_patterned_stroke_through_its_fallback` |
| `onto`: o `alpha` não recebe a opacidade | `the_stroke_opacity_reaches_a_patterned_stroke` |
| `set_kind`: o padrão nasce com o `alpha` do construtor | `the_opacity_survives_the_paint_switch_in_both_directions` |

---

## §8 — ⛔⛔ O REPORT DE 2026-08-28 (2.º, com foto): *"ao mudar a posição dos nós ... o stroke inverte"*

### §8.1 — ⭐ A MEDIÇÃO ILIBOU a colocação, e foi ela que apontou a causa

Sonda `measure_what_a_node_move_does_to_the_stroke_pattern`
([`pattern_tests.rs`](../../crates/ph2d-vec-render/src/pattern_tests.rs), `#[ignore]`), com os
números **exactos** da cena de smoke:

| | bbox | `placement` | peças |
|---|---|---|---|
| nós originais | `(0,0)..(2.2,2.2)` | `[0.3667, 0, 0, 0.3667, 0, 0]` | `["Line(emprestada)"]` |
| **nó movido** | `(0,0)..(3.52,2.2)` | `[0.3667, 0, 0, 0.3667, 0, 0]` | `["Line(emprestada)"]` |

⇒ **byte-idêntico.** O `placement_in` devolve a colocação AUTORADA em todo modo que não seja
`Clamp`, então mover um nó não move o reticulado. *Uma hipótese cara (o traçado, o memo, o
enquadramento) morreu numa sonda de dez linhas.*

### §8.2 — ⛔ A causa 1: a CENA ancorava os padrões na origem do MUNDO

`PatternFill::new` nasce com `origin: [0,0]`, e a cena **nunca o escrevia** — enquanto o produto
ancora no canto da forma (`texture_pattern_pick::default_placement` devolve o `lo` da caixa).

⚠️ **É a MESMA lei que um report do Enio já pagou em 27/08** (*"clamp deixa tudo em branco"*), e ela
volta aqui com outro sintoma: num **preenchimento** a fase do reticulado é invisível, porque a forma
mostra o reticulado inteiro; numa **faixa de 20 % da forma** vê-se uma *fatia* dele, e a fase é a
aparência inteira. ⇒ *o mesmo defeito muda de gravidade com o tamanho do sujeito.*

### §8.3 — ⛔ A causa 2: `Mirror` numa faixa fina

A 2.ª forma nova usava `PatternMode::Mirror` no traço — escolhido para *"ser diferente do
preenchimento"*. O espelho troca a paridade a cada célula: com a fase solta (§8.2), mover um nó
fazia a arte sob a banda **virar do avesso**. É literalmente *"inverte"*.

⇒ hoje as duas formas usam `Tile`, e a diferença entre as duas tintas passou a ser o **reticulado**
(grade no miolo, tijolo no contorno) — visível e **estável** sob edição. ⚠️ *Uma cena de smoke
escolhe o modo que deixa a FEATURE visível, não o que exercita mais código.*

### §8.4 — ⛔ A causa 3 (minha, da wave B): um marcador ABERTO era PREENCHIDO

O ramo do padrão tratava `StrokePiece::Symbol` e `StrokePiece::Fill` como a mesma coisa. O
`stroke_plan` distingue-os por `Marker::is_filled`: um marcador aberto **traça-se**. ⇒ uma seta
aberta saía **maciça** assim que o traço ganhava padrão.

⭐ **A régua é a IGUALDADE com o ramo sólido** (`the_pattern_branch_draws_the_same_pieces_as_the_solid_one`),
e não uma contagem à mão: *o que* desenhar é decisão do `stroke_plan`, e quem pinta só obedece.

### §8.5 — ⚠️⚠️ O gate que eu escrevi era uma TAUTOLOGIA, e a mutação disse-o

A 1.ª cura fez o `rect` **derivar** do `canto` (*"uma porta, dois consumidores"*) e o gate comparava
os dois. A mutação `canto = [cx - half, cy]` **SOBREVIVEU**: a forma seguia o canto para onde ele
fosse.

⇒ o `rect` voltou a ser uma conta independente, e a régua passou a ser uma **propriedade da
geometria** (*o canto é ≤ todo vértice e É um deles*). *Concordância só se mede entre duas
afirmações independentes; derivar as duas de uma porta anula o instrumento* —
[[feedback_a_claim_no_mutation_can_kill_is_a_claim_about_nothing]].

### §8.6 — ⏳ O que FICA, e é a lei do módulo (não um defeito)

Com a âncora na forma e sem espelho, mover um nó **não inverte** nada — mas a arte **desliza** por
baixo da faixa, porque um padrão é *papel de parede no espaço da forma que a banda revela*. É a lei
declarada do módulo para preenchimentos, aplicada a uma banda fina.

⛔ **O motivo que percorre a linha e roda com ela é OUTRA feature, e já existe**: o
[Pattern Along Path](23_plano_pattern_along_path.md) (plano 23). Confundi-los seria dar dois nomes à
mesma coisa e uma palavra a duas (§1). Se o Enio quiser esse comportamento no contorno, a pergunta
é de produto e a maquinaria está construída.

### §8.7 — As três provas de mutação

| Mutação | Gate que morreu |
|---|---|
| `Symbol` traçado com metade da largura | `the_pattern_branch_draws_the_same_pieces_as_the_solid_one` |
| `lei`: `f.origin = [0.0, 0.0]` | `a_pattern_is_born_where_its_shape_starts` |
| `canto`: `[cx - half, cy]` | idem (⚠️ **sobreviveu** à 1.ª redacção do gate — §8.5) |
