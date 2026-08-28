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
| **D** | A UI: tipo do traço (`Solid \| Pattern`) + o alvo (`Fill \| Stroke`) na secção *Pattern* | `ph2d-panel-vector` + shell | não |
| **E** | Persistência + smoke + gates + mutações | shell | não |

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
