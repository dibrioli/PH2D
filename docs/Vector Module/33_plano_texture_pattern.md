# 33 — PLANO: *Texture Pattern* no preenchimento vetorial (item **3**, o último da fila)

> Pedido do Enio (2026-08-24, verbatim): *"Mais uma feature: Texture patttern para Formas Vetorias."*
> Reaberto em **2026-08-27** com a instrução *"buscamos o estado da arte"*.
>
> ⛔ **A fila [29](29_fila_morph_state_machine_e_texture_pattern.md) tinha CINCO perguntas por
> responder e nenhum plano.** Este documento responde às cinco **com medição**, e a §0 traz o que
> cada resposta custou de grep — porque três das cinco premissas da folha 29 mudaram de valor.

---

## §0 — O que foi MEDIDO antes de desenhar (2026-08-27, na `line/Vector` rebaseada em `330582deb`)

⚠️ *Quem move o número que tornava algo inalcançável tem de reconferir a nota* (`CLAUDE.md` §0.0).
A folha 29 é de 24/08; entre ela e hoje integraram nove linhas. Tudo abaixo foi conferido nesta árvore.

### §0.1 — ⭐⭐⭐ O Vello **LADRILHA NATIVAMENTE**, e isto está provado ao nível do BIT

> ⚠️⚠️ **ESTA MEDIÇÃO É DA `vello 0.8` / `peniko 0.6`, e a casa está hoje na `vello 0.10.0`**
> (subida de 2026-08-29, [ADR-0168](../architecture/decisions/0168-the-stack-rises-to-its-ceilings-and-four-dependencies-stay-behind-on-purpose.md)).
> Os endereços da tabela abaixo (`vello_encoding-0.8.0`, `vello_shaders-0.8.0`, `peniko-0.6.0`)
> continuam a ser o que se mediu **naquela versão** — não os apague, mas **não os cite como o estado
> de hoje**. O facto de fundo (o ladrilho é nativo, `Extend` no *sampler*, sem clip nem rasterização
> por quadro) é o que sustenta a arquitectura e é o menos provável de ter mudado; **as duas limitações
> do §0.1.1 abaixo é que quase de certeza mudaram** — leia o aviso lá.

Esta é a medição que decide a arquitectura inteira, e ela **contradiz o desenho que a folha 29
sugeria** (*"o `MultiPoint` já rasteriza num image-brush — a rota de imagem já existe"*). A rota que
existe é a **errada** para um padrão: ela rasteriza um buffer na CPU, empurra um `push_clip` e faz um
**blit único**. Um padrão não precisa de nada disso.

| Facto | Endereço | O que diz |
|---|---|---|
| O `peniko 0.6` tem `Extend` no *sampler* de imagem | `peniko-0.6.0/src/image.rs:95-152` | `ImageSampler { x_extend, y_extend, quality, alpha }` + `with_x_extend` / `with_y_extend` / `with_quality` / `with_alpha` |
| O Vello **empacota** os dois extends | `vello_encoding-0.8.0/src/encoding.rs:444-452` | `sample_alpha` = `format<<15 \| alpha_type<<14 \| quality<<12 \| x_extend<<10 \| y_extend<<8 \| alpha` |
| O shader **lê e honra** os dois | `vello_shaders-0.8.0/shader/fine.wgsl:819-826, 867-893` | `extend_mode_normalized`: `PAD = clamp` · `REPEAT = fract` · `REFLECT = abs(t − 2·round(t/2))` |
| E **não sangra o atlas** | `fine.wgsl` `case CMD_IMAGE` | o extend aplica-se **antes** de somar `atlas_offset`, e o resultado é `clamp(uv, atlas_offset, atlas_max)` — o repeat dá a volta **dentro do próprio ladrilho** |

⇒ **Ladrilhar é de graça: uma `fill()` com `Brush::Image`, zero camadas de clip, zero rasterização
por quadro.** É *mais barato* que o preenchimento sólido nunca ser, e estritamente mais barato que a
rota do `MultiPoint`, que hoje corre `rasterize_idw` (64×64×N pontos, CPU) **a cada quadro, sem memo**
([`gradient.rs:335,441`](../../crates/ph2d-vec-render/src/gradient.rs)).

#### §0.1.1 — As duas limitações da 0.8 — ⚠️⚠️ **PRECISAM DE RE-MEDIÇÃO NA 0.10**

> ⛔⛔ **NÃO TRATE AS DUAS LIMITAÇÕES ABAIXO COMO VERDADE DE HOJE.** Elas foram medidas na
> **`vello 0.8`**; a casa está na **`0.10.0`** desde 2026-08-29
> ([ADR-0168](../architecture/decisions/0168-the-stack-rises-to-its-ceilings-and-four-dependencies-stay-behind-on-purpose.md)),
> e **a `0.9` mexeu exactamente nestes dois pontos**:
> - a **`ImageQuality::High` passou a ser bicúbica a sério** (filtro de Mitchell) — ou seja, o ⛔ do
>   item 1 (*"não existe, renderiza como medium"*) tem toda a probabilidade de ter **dissolvido**;
> - o **deslocamento de meio pixel na amostragem foi corrigido** — o artefacto de *meio texel na
>   costura* do item 2 é o mesmo mecanismo, e pode já não existir.
>
> ⇒ **RE-MEÇA os dois contra o `fine.wgsl` da `0.10` antes de honrar, prometer ou recusar seja o que
> for a partir deles** (`CLAUDE.md` §0.0: *quem move o número que tornava algo inalcançável tem de
> reconferir a nota* — e quem moveu foi a subida do stack, não este plano). Em particular, a recusa
> **«⛔ Bicúbico»** do §7 deste documento depende inteiramente do item 1.

⚠️ **DUAS limitações medidas do Vello 0.8 que o plano tinha de honrar** *(medição de 2026-08-27, na
0.8 — ver o aviso acima)* (não são nossas, e nenhuma é bloqueante):

1. ⛔ **`ImageQuality::High` NÃO existe** — o shader diz-o em texto: *"We don't have an implementation
   for `IMAGE_QUALITY_HIGH` yet, just use the same as medium"*. ⇒ o `High` que o
   [`fill_multipoint`](../../crates/ph2d-vec-render/src/gradient.rs) pede hoje **já** renderiza como
   bilinear. Não prometemos bicúbico enquanto o Vello não o tiver.
2. ⚠️ **No `Repeat`, o filtro bilinear NÃO dá a volta na COSTURA — ele grampeia.** A conta é
   `uv − 0.5` seguida de `clamp` ao rectângulo do ladrilho: no último texel, `floor` e `ceil`
   colapsam no mesmo, então a última coluna não se mistura com a primeira. O artefacto é **meio
   texel** por fronteira de ladrilho. ⛔ **Um *gutter* NÃO cura** (ele entra no `extents` e alarga o
   padrão); quem cura é a **resolução do assado** — o ladrilho é assado no tamanho em que vai ser
   mostrado. E em `ImageQuality::Low` (o modo Pixel Art da casa) **não há artefacto nenhum**, porque
   não há interpolação.

### §0.2 — A costura de render é UM método, e ela tem uma armadilha JÁ DOCUMENTADA

[`VectorScene::fill_path`](../../crates/ph2d-vector/src/scene.rs) faz
`self.inner.fill(Fill::NonZero, transform, brush, None, path)` — e **os dois argumentos de que um
padrão precisa estão ali, mortos**:

- o 1.º está **fixado em `NonZero`** ⇒ um padrão num *compound path* com `EvenOdd` **pintaria o
  buraco**. ⚠️ Não é hipótese: é exactamente por isto que o `fill_multipoint` teve de usar
  `push_clip_with_rule` em vez do `push_clip`, e o comentário dele diz-o.
- o 4.º é o **`brush_transform`, sempre `None`**. O Vello compõe-no como `transform * brush_transform`
  (`vello-0.8.0/src/scene.rs:329`) ⇒ **a colocação do padrão exprime-se no espaço LOCAL do caminho e
  cavalga o `Transform` da entidade de graça.**

⇒ A obra de render é **um método novo** na `ph2d-vector` (a única crate autorizada a importar
`vello::*`), não um caminho novo.

### §0.3 — ⭐ O contrato congelado **NÃO é tocado**, e aqui está a prova

O `Paint` vive na [`ph2d-vec-scene`](../../crates/ph2d-vec-scene/src/paint.rs). O gate
`architecture_vector_contract_surface`
([`ph2d-vector-doc/tests/`](../../crates/ph2d-vector-doc/tests/architecture_vector_contract_surface.rs))
varre `src/**/*.rs` **por NOME de crate**, e os nomes que ele varre são exactamente **dois**:

```
$ grep -o '"ph2d-[a-z-]*"' crates/ph2d-vector-doc/tests/architecture_vector_contract_surface.rs | sort -u
"ph2d-vector-doc"
"ph2d-vector-traits"

$ grep -c "vec-scene\|vec_scene" crates/ph2d-vector-doc/tests/architecture_vector_contract_surface.rs
0
```

⚠️ **E uma pista falsa a desarmar antes que alguém a siga:**
[`ph2d-vector-doc/src/style.rs:276`](../../crates/ph2d-vector-doc/src/style.rs) diz literalmente
*"room for the resource-bound fills (pattern/image) to grow"* — o `ProceduralFillKind` reserva 2 dos
4 lugares do tecto dele para isto. ⛔ **É o motor VELHO** (retirado pelo
[ADR-0108](../architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md));
aquela vaga foi guardada para um padrão que nunca chegou, e enchê-la hoje seria construir a feature
na crate que já não desenha nada.

### §0.4 — O schema, contado hoje (⛔ **não copie estes números — conte-os na integração**)

| Número | Valor **hoje** | Onde | O que esta wave lhe faz |
|---|---|---|---|
| `VEC_SCENE_SCHEMA_VERSION` | **14** | [`ph2d-vec-scene/src/lib.rs:438`](../../crates/ph2d-vec-scene/src/lib.rs) | **14 -> 15** (variante apendada ao `Paint`) |
| `PROJECT_SCHEMA` | **99** | [`project_schema.rs:305`](../../shells/desktop/src/project_schema.rs) | **99 -> 100**, por arrasto |
| Espelhos do registo de componentes | (a contar) | `ph2d-render`, `ph2d-script` | ⭐ **NENHUM — esta feature não cunha componente** |

⚠️ **Apendar uma variante ao `Paint` é aditivo num sentido e destrutivo no outro**, e a regra já está
escrita no doc-comment do `VEC_SCENE_SCHEMA_VERSION`: um save v14 lido por v15 lê **certo**; um save
v15 com um padrão, lido por um binário v14, encontra um índice de variante que não conhece. O bump é
o que transforma isso num erro de versão em vez de num postcard a falhar longe da causa.

⚠️ **O degrau da escada segue a decisão de 26/08 do Enio** (`project_schema.rs:286`): sem um tipo
`ProjectFileVn` congelado não há forma honesta de ler os bytes antigos, e um ficheiro de formato
anterior é **recusado em voz alta**. Este plano não muda essa política; só acrescenta o degrau
documentado.

### §0.5 — A `ph2d-vec-scene` é uma FOLHA PURA, e isso decide onde mora o id da imagem

O `Cargo.toml` dela declara-o: *"sem vello/kurbo/ph2d-color"*, e as dependências são todas folhas
zero-dep (`ph2d-arclen`, `ph2d-warp-style`, `ph2d-stroke-width`, `ph2d-symmetry`) + `serde` +
`postcard`.

⛔ **`ph2d-asset` NÃO pode entrar ali**: ela puxa `ph2d-imageio` + `ph2d-imageio-registry-init` +
`ph2d-color` — descodificadores de imagem dentro do modelo puro de documento.

⭐ **E a crate já resolveu exactamente este problema TRÊS vezes, com a mesma frase no Cargo.toml
dela:** *"a casa ÚNICA do vocabulário X mora nesta folha porque tem dois donos que não se veem"*
(`ph2d-warp-style`, `ph2d-stroke-width`, `ph2d-symmetry`) — e o `ph2d-arclen` diz textualmente
*"extraído para uma crate-folha ZERO-dep quando apareceu o segundo consumidor"*.

⇒ **W4 extrai o `AssetId` para `ph2d-asset-id`** (60 linhas, `blake3` + `serde`, o ficheiro
[`ph2d-asset/src/id.rs`](../../crates/ph2d-asset/src/id.rs) inteiro) e a `ph2d-asset` re-exporta-o.
**Nenhum sítio de chamada muda uma linha**, e não nascem dois ids para a mesma coisa — que é o modo de
falha que um newtype paralelo garantiria.

### §0.6 — Não há navegador de assets, e a porta de ficheiro é `rfd` **com gate**

- ⛔ Não existe navegador/índice de assets: o [ADR-0165](../architecture/decisions/0165-assets-are-born-inside-the-app-three-level-identity-index-before-browser.md)
  chama-se, literalmente, *"index **before** browser"*.
- ✅ A porta viva é o `rfd::FileDialog`, em **12 ficheiros** deste shell — e há um **gate de modal**
  ([`modal_tests.rs`](../../shells/desktop/src/modal_tests.rs)) que prova que a chamada bloqueante
  passa pela porta da casa. ⚠️ *A agulha dele é a CHAMADA, não o tipo.*
- ✅ E a lista de extensões tem **uma** fonte desde 23/08:
  [`import_router.rs`](../../shells/desktop/src/import_router.rs) — *uma lista escrita à mão ao lado
  de um predicado são duas respostas à mesma pergunta.*

### §0.7 — ⭐ Duas coisas que a composição JÁ exprime — e que este plano portanto **não constrói**

> *Antes de construir um item de lista aberta, MEÇA se a composição já o exprime* (`CLAUDE.md` §5.0).

| O que um plano ingénuo construiria | Porque **não** |
|---|---|
| **Repetir a arte N vezes** (o *Repeater*) | Já existe: `PathEffect::Repeat` ([`fx_repeat.rs`](../../crates/ph2d-vec-scene/src/fx_repeat.rs)) — N cópias com afim **cumulativo**, distâncias relativas por eixo. E `PathEffect::Hatch` já enche uma forma fechada com geometria por *scanline clip*. ⇒ **um padrão VECTORIAL ladrilhado é irmão do `Hatch` na pilha de efeitos, e não é esta feature.** Esta é a tinta. |
| **Arte repetida ao longo de um CAMINHO** | Já existe, inteiro: `VecPatternPath` + picker + duas alças de canvas ([plano 23](23_plano_pattern_along_path.md), W1-W4 **fechadas**, medido `0,597 ms` / 200 cópias). ⚠️ A folha 29 mandava "medir o que dele está construído" — está **tudo**. É outra ferramenta: ali o padrão cavalga uma guia; aqui ele preenche uma área. |

⚠️ **E uma que a composição exprime PELA METADE:** a fileira **Fill Opacity** já existe
(`VECTOR_FILL_OPACITY`) — mas ela escreve a **alfa da cor do estilo da ferramenta**
([`vector_bridge_style.rs:276`](../../shells/desktop/src/render_loop/vector_bridge_style.rs)), que um
padrão não tem. A alfa de um padrão viaja no `ImageBrush::with_alpha`. ⇒ **é uma pergunta de costura,
não um knob novo**, e está nomeada na W5.

---

## §1 — Pesquisa: o estado da arte, e o que cada um deles ABANDONOU

### §1.1 — O quadro

| Ferramenta | O que a fonte do padrão é | A lei de ladrilho | A colocação | Anda com a forma? |
|---|---|---|---|---|
| **SVG** (`<pattern>`) | arte SVG viva | `viewBox` + `width`/`height` | `patternTransform` | ⭐ **é a bifurcação declarada:** `patternUnits = userSpaceOnUse` (rígido) **ou** `objectBoundingBox` (respira com a forma) |
| **Illustrator** (Pattern Options, CS6) | arte no documento, editada em *isolation mode* | **Grid · Brick by Row · Brick by Column · Hex by Column · Hex by Row** + H/V Spacing + Overlap + *Size Tile to Art* | origem da régua | ⛔ **por omissão NÃO** — só com *Transform Pattern Tiles* ligado |
| **Figma** (Patterns, 2025) | ⭐ **outro objecto da tela, no mesmo ficheiro** — e é **dinâmico**: editar a fonte actualiza toda camada que a usa | tile type · spacing · alignment | scale · opacity | sim (é a tinta da camada) |
| **Inkscape** (editor de padrão, 1.3) | arte SVG viva | `<pattern>` + gap | ⭐ **três alças na tela: `×` move · quadrado escala · círculo roda** | sim (`patternTransform`) |
| **Figma** (Image fill, *Tile*) | um bitmap | grade rectangular | **uma** percentagem de escala + *Rotate 90°* em passos | sim |

### §1.2 — O que foi TENTADO e virou queixa (é isto que o `/pd-feature` pede, e é onde se aprende)

1. ⛔⛔ **A ancoragem do padrão à ORIGEM DA RÉGUA (Illustrator) é o erro clássico da categoria.**
   Por omissão o padrão fica preso ao *artboard*, não à forma: mover a forma faz o padrão **deslizar
   por baixo dela**, e escalar a forma **não** escala o padrão. A cura da Adobe foi uma
   **preferência** (*Preferences > General > Transform Pattern Tiles*) mais caixas separadas dentro
   do efeito *Transform*, e há pedidos abertos no UserVoice para **desatar** as duas
   ([Untie the Transform Patterns checkbox](https://illustrator.uservoice.com/forums/333657-illustrator-desktop-feature-requests/suggestions/42759833-untie-the-transform-patterns-checkbox-in-general-o),
   [Pattern origin](https://illustrator.uservoice.com/forums/601447-illustrator-desktop-bugs/suggestions/34484974-pattern-origin)).
   ⭐ **Nós não temos essa escolha a fazer, e é uma vantagem:** o cabeçalho do
   [`paint.rs`](../../crates/ph2d-vec-scene/src/paint.rs) já a fez para os gradientes — *world-space,
   transforma junto com o path* — e a razão foi um **bug**: o gradiente relativo à bbox *respirava*
   a cada edição. Esta feature herda a lei inteira e **não ganha preferência nenhuma**.
2. ⛔ **A escala única do *Tile* do Figma.** O modo *Tile* de um preenchimento de imagem tem **uma**
   percentagem e nenhuma rotação livre (só *Rotate 90°* em passos) — pedido recorrente no fórum
   ([Image fill rotation by angle](https://forum.figma.com/ask-the-community-7/image-fill-rotation-by-angle-21643)).
   É a razão de existirem plugins de comunidade só para isso.
   ⇒ **rotação livre entra desde o desenho** (é um ângulo no `brush_transform`, custo zero).
3. ⭐ **O que o Figma acertou e é o estado da arte de 2025:** a fonte é *"another object on the canvas
   in the same file"* e *"pattern fills are dynamic — if you update the pattern's source object, the
   pattern will automatically update on each layer where it's used"*
   ([Figma Help](https://help.figma.com/hc/en-us/articles/31616030150167-Use-patterns-as-a-fill-or-stroke)).
   ⇒ **é a W7 deste plano**, e é o que separa *um preenchimento de imagem* de *um sistema de padrões*.
4. ⭐ **O que o Inkscape acertou:** o padrão edita-se **na tela**, com três alças
   ([Inkscape Beginners' Guide](https://inkscape-manuals.readthedocs.io/en/latest/creating-custom-patterns.html)) —
   e não num formulário de números. ⇒ foi a **W6**, e a máquina de alças **já existia** nesta crate
   (`GradHandle` + `hit_gradient_handle` / `drag_gradient_handle` / `draw_gradient_handles`,
   [`gradient.rs`](../../crates/ph2d-vec-render/src/gradient.rs)).
   ⛔⛔ **E o Enio RECUSOU-A depois de a ver** (2026-08-27: *"não ficou legal"*) — este item da
   pesquisa continua a descrever o Inkscape com fidelidade, e **deixou de descrever este produto**.
   O que ficou no lugar está no **§6-quater**. *Uma leitura do estado da arte não é um veredito do
   dono; e quando os dois discordam, quem manda é o segundo.*

### §1.3 — ⭐⭐ A síntese, e a ideia que faz este desenho ser MAIS simples que o dos outros

Os cinco tipos de ladrilho do Illustrator (grid · brick-por-linha · brick-por-coluna · hex-por-linha ·
hex-por-coluna), o *spacing*, o *overlap* e o *alignment* do Figma são todos **decisões sobre um
reticulado**. E um reticulado de *brick* com meio passo de desfasamento **é** um reticulado
rectangular de duas linhas; um *half-drop* é um de duas colunas; um *hex* é um de 2×2.

⇒ **A lei de ladrilho resolve-se ao ASSAR, não ao desenhar.** O assador compõe a arte no reticulado
pedido dentro de **um** rectângulo, e a GPU faz o único `Extend::Repeat` que sempre soube fazer.

**Consequências, todas medidas:**

- O custo por quadro é o de um preenchimento sólido — **uma** `fill()`, sem camada de clip, sem
  rasterização. O trabalho todo é do assador, **uma vez**, memoizado.
- *Spacing* e *overlap* também são o assador (assar num rectângulo maior/menor que a arte).
- Não há caso especial: `Grid` é o reticulado 1×1, e por isso é **byte-idêntico** à fonte — o ponto
  neutro tem gate, como todo efeito da casa.
- E é **exacto**: nenhum destes é uma aproximação do que o Illustrator faz; é a mesma grelha, noutra
  ordem de operações.

---

## §2 — O DESENHO, com a porta ÚNICA de cada pergunta

> *Duas portas divergem em silêncio.* Cada linha desta tabela é uma pergunta que o produto faz, e a
> **única** função que a responde.

| Pergunta | A porta ÚNICA | Onde |
|---|---|---|
| *Que pixels tem o ladrilho?* | `pattern::bake(source_rgba, w, h, &TileLaw) -> (Vec<u8>, u32, u32)` | `ph2d-vec-pattern` (folha nova) |
| *Onde ele fica?* | `pattern::placement(&PatternFill, tile_px) -> [f64; 6]` (afim px-da-imagem -> espaço das âncoras) | idem |
| *Como se repete?* | `PatternMode -> (x_extend, y_extend)` | idem |
| *Como se pinta?* | `VectorScene::fill_path_image(path, rule, transform, &StableImage, brush_xform, x_ext, y_ext, quality, alpha)` | `ph2d-vector` (a única crate com `vello::*`) |
| *De onde vêm os pixels da FONTE?* | `pattern_tile::resolve(app, &PatternSource) -> Option<StableImage>` (memo) | shell |
| *Quem é a fonte?* | `PatternSource::{ Image(AssetId), Shape(VecPathId) }` | `ph2d-vec-scene` |

### §2.1 — O dado

```rust
// ph2d-vec-scene/src/paint.rs — a 5ª variante
pub enum Paint {
    Solid(Rgba8),
    Linear  { .. },
    Radial  { .. },
    MultiPoint { .. },
    /// ⚠️ BOXED de propósito — ver §2.2.
    Pattern(Box<PatternFill>),
}

pub struct PatternFill {
    pub source: PatternSource,
    pub tile:   TileLaw,      // Grid | BrickRow | BrickCol | HalfDrop | Hex  (+ shift)
    /// Tamanho do ladrilho no espaço das ÂNCORAS (as mesmas unidades dos stops de gradiente).
    pub size:   [f64; 2],
    /// Vão entre ladrilhos, nas mesmas unidades. `[0,0]` = encostados.
    pub gap:    [f64; 2],
    /// Colocação: canto do ladrilho no espaço das âncoras + rotação em radianos.
    pub origin: [f64; 2],
    pub angle:  f64,
    pub mode:   PatternMode,  // Tile -> Repeat | Mirror -> Reflect | Clamp -> Pad
    pub alpha:  f32,
}
```

### §2.2 — As decisões, cada uma com o mecanismo

1. ⭐ **A colocação vive no espaço das ÂNCORAS, e por isso o padrão é RÍGIDO com a forma.**
   Mecanismo: o Vello compõe `transform * brush_transform`, e o `transform` que a
   `ph2d-vec-render` já passa é `camera * Transform_da_entidade`
   ([`path_to_screen`](../../crates/ph2d-vec-render/src/lib.rs)). ⇒ pôr a colocação em espaço de
   âncoras faz o padrão cavalgar a pose **sem uma linha de código de acompanhamento** — a mesma
   escolha que os gradientes fizeram, pelo mesmo motivo, com a mesma prova.

2. ⚠️⚠️ **Sob escala NÃO-uniforme o padrão ESMAGA — e é a decisão certa, ao contrário do traço.**
   Esta é a pergunta 3 da folha 29, e ela aponta o [bug #27](BUGS_vector.md) (a caneta virava elipse,
   curado com `√|det|`). ⛔ **A analogia não se aplica, e confundi-las seria o erro:**
   - o **traço** é uma FERRAMENTA que desenha a forma — a caneta do artista não muda de feitio
     porque a forma foi esticada, e o Enio decidiu-o em 23/08 (*"quando engrossa, engrossa por igual
     nos dois eixos"*);
   - o **preenchimento** está COLADO à forma. Um gradiente radial já vira elipse sob escala
     não-uniforme, hoje, e ninguém chamou a isso um bug — porque é o que uma tinta colada faz.
   ⇒ **O padrão segue o preenchimento.** Isto entra num gate com o nome inteiro
   (`the_pattern_shears_with_the_shape_unlike_the_pen`) para que a próxima pessoa que ler o bug #27
   encontre a diferença escrita, e não a redescubra.

3. ⚠️ **`Box<PatternFill>`, e o número decide.** O `Paint` mora dentro de **todo** `VecPath`, e todo
   `VecPath` entra em **toda** fotografia de undo. O tamanho do `enum` é o da maior variante — hoje
   um `Vec` (24 B) + tag. Um `PatternFill` inline levá-lo-ia a ~112 B. ⇒ gate
   `the_paint_enum_does_not_grow_when_pattern_lands`, com o `size_of` medido **antes** e depois. O
   postcard não vê o `Box`.

4. ⛔ **Zero componente ECS novo, zero espelho de registo a contar.** O padrão é *com que tinta a
   forma aparece*, e essa pergunta já tem dono: o `Paint`. ⚠️ Isto é o **oposto** do que a folha 23
   decidiu para o *pattern along path* — e a diferença é a certa: ali o vínculo é uma **relação entre
   dois objectos** (motivo + guia), aqui é uma **propriedade de um**.

5. ⚠️ **A regra de preenchimento viaja, e há um precedente a dizer que sim.** O método novo recebe a
   `Fill` do caminho. Sem isso, um padrão numa forma composta com `EvenOdd` pinta o buraco — e o
   `fill_multipoint` já tropeçou nesta pedra e deixou o comentário.

6. ⭐ **Uma FONTE, dois produtores, um consumidor** — o molde do `FxImage`, que esta crate já usa:
   o `PatternSource` é `Image(AssetId)` **ou** `Shape(VecPathId)`, e os dois desaguam no **mesmo**
   `StableImage` memoizado. ⛔ O consumidor nunca ramifica por origem.

---

## §3 — Onde encosta (§6 / schema) — **com a prova, não com a suposição**

| Superfície | Encosta? | Prova |
|---|---|---|
| `NodeOp` / `OpResolver` / `NodeManifest` (nós) | **não** | outra árvore |
| `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` | **não** | nenhum `impl Tool` novo; a feature é uma variante de dado + uma fileira de painel |
| `VectorOp` / `Vertex` / `Segment` / `AnimValue` (**congelado**, ADR-0056..0068) | **não** | §0.3 — o gate varre `ph2d-vector-doc` + `ph2d-vector-traits` e **zero** menções a `ph2d-vec-scene` |
| `VEC_SCENE_SCHEMA_VERSION` | **SIM**, 14 -> 15 | variante apendada |
| `PROJECT_SCHEMA` | **SIM**, 99 -> 100 | por arrasto; ⛔ **recontar na integração** |
| Espelhos do registo de componentes | **não** | zero componentes novos |

⚠️ **`ph2d-asset-id` é crate NOVA** — e nasce desenhada para isolamento, como o §0.2 do `CLAUDE.md`
manda: é um módulo irmão zero-dep e um `pub use` **append-only** na `ph2d-asset`. Não há um símbolo
existente a mover de sítio, então não há colisão de mesmo-símbolo para outra linha.

---

## §4 — A UI: as QUATRO condições, cada uma independente

> *O componente EXISTE · é pintado e registado · o clique chega ao barramento · e a SEQUÊNCIA leva a
> algum lugar.* As três primeiras têm gate de costura nesta casa; a quarta é a que se esquece.

**1. O componente existe** — o selector de tipo de preenchimento já tem 4 chips
([`paint_arrange.rs:267-270`](../../crates/ph2d-panel-vector/src/paint_arrange.rs)); o padrão é o 5.º.

**2. Pintado e registado** — os 7 sítios que uma fileira de `fill_kind` atravessa, medidos:

| # | Sítio | Ficheiro |
|---|---|---|
| 1 | o id | `ph2d-editor-core/src/ids/chrome/vector.rs:65-69` |
| 2 | o re-export | `ph2d-panel-vector/src/ids.rs:86` |
| 3 | o registo (`button(store, id)`) | `populate_ops.rs:185-188` |
| 4 | a pintura | `paint_arrange.rs:267` |
| 5 | o reconhecimento do clique | `event_clicks.rs:306-309` |
| 6 | o despacho id -> `VecFillKind` | `input_dispatch.rs:524-531` |
| 7 | a construção do `Paint` | `input_dispatch.rs:646-672` |
| + | a publicação `Paint` -> `FillKind` | `vector_bridge_publish.rs:220-230` |

**3. O clique chega ao barramento** — gate `seam_pattern.rs` que **CLICA** (o precedente é o
`seam_bool.rs`: ⚠️ *um controlo nunca pintado e um morto sob o dedo dão o MESMO report*, e só o gesto
real mede a segunda costura).

**4. ⚠️ A SEQUÊNCIA leva a algum lugar** — e aqui está a armadilha desta feature: **escolher
"Pattern" sem fonte não pode não fazer nada.** A regra:
- se a forma **já** tem um padrão, o chip só volta a ele;
- se não tem, o chip **abre a porta de escolher a fonte** — que é a mesma porta do botão *Source…*
  da secção. ⛔ Um chip que muda o tipo de preenchimento para algo invisível é o defeito que o Enio
  reportou três vezes nesta linha (o item mudo).

**A secção *Pattern*** (só sobe quando o preenchimento é padrão — a regra da secção do
`paint_patternpath.rs`): `Source…` (nomeia a imagem/forma) · `Tile` (5 opções) · `Size X/Y` ·
`Gap X/Y` · `Angle` · `Mode` (Tile/Mirror/Clamp) · `Alpha`. Rótulos por **i18n**
([`ph2d-i18n/src/vector.rs`](../../crates/ph2d-i18n/src/vector.rs)), espaçamento por **tokens**,
zero hex e zero `f32` literal (HR-15).

⚠️ **`MAX_ENUM_OPTIONS` e o tecto de linhas do painel** foram medidos pela conferência do Motion (o
`motion.bezier_warp` levou o tecto a 24 e desenha 1083 px num dock de 880). Esta secção tem **~10
fileiras** — cabe, mas a W5 mede-a em vez de o assumir.

---

## §5 — Os gates, **red-first**, e a fixtura que CONTÉM o fenómeno

> ⛔⛔ **A lição que esta linha pagou seis vezes em 26/08 e que este plano herda inteira:** *uma
> fixtura que não contém o fenómeno aprova a cura errada.* Quatro waves seguidas ficaram verdes e
> cada uma curou só a metade que o arnês sabia produzir. A fixtura desta feature é, por isso,
> **declarada antes dos gates**.

### §5.1 — A fixtura tem de conter os CINCO fenómenos

| # | Fenómeno | Porque é que a sua ausência aprovaria a cura errada |
|---|---|---|
| 1 | **escala não-uniforme** (`sx != sy`) | é a pergunta 3 da folha 29 e a família do bug #27 |
| 2 | **rotação** | separa "roda com a forma" de "roda o padrão" — duas leis que num quadrado parado leem igual |
| 3 | **caminho composto com buraco + `EvenOdd`** | o `Fill::NonZero` fixado no `fill_path` pinta o buraco, e **nenhuma forma simples o mostra** |
| 4 | **ladrilho com ALFA parcial** | ⚠️ a família do [Bug #4 do Motion](../Motion%20Nodes/BUGS_motion_nodes.md): uma fonte pré-multiplicada codifica *"não contribui"* como zero, e em `α = 1` **nada se move** — era só `α = 1` que aquele gate media |
| 5 | **um ladrilho pequeno muito ampliado** | é onde a costura do bilinear (§0.1) se vê, e onde a resolução do assado se decide |

### §5.2 — Os gates

| Wave | Gate | O que morre se ele mentir |
|---|---|---|
| W1 | `a_grid_tile_is_byte_identical_to_its_source` | o ponto neutro — a invariante da casa |
| W1 | `brick_by_row_is_the_transpose_of_brick_by_column` | a lei do reticulado |
| W1 | `a_gap_of_zero_leaves_the_art_untouched` | *spacing* como caso especial em vez de composição |
| W1 | `the_hex_lattice_closes_on_itself` | o ladrilho hex tem de ser periódico, ou aparece emenda |
| W2 | `a_pattern_on_an_evenodd_compound_does_not_fill_the_hole` | o `Fill::NonZero` fixado (§0.2) |
| W2 | `the_pattern_fill_encodes_no_clip_layer` | a regressão para a rota do `MultiPoint`, medida pelo arnês que **já existe** ([`encode_cost_tests.rs`](../../crates/ph2d-vec-render/src/encode_cost_tests.rs), 272 linhas) |
| W2 | `the_pattern_shears_with_the_shape_unlike_the_pen` | a confusão com o bug #27 |
| W3 | `the_paint_enum_does_not_grow_when_pattern_lands` | o `Box` (§2.2.3) |
| W3 | a tripla `(PROJECT_SCHEMA, FLIP, VEC_SCENE)` | **três** sítios, nunca um |
| W4 | `a_saved_pattern_reopens_with_the_same_pixels` | o defeito exacto que o [`project_sprite_pixels.rs`](../../shells/desktop/src/project_sprite_pixels.rs) curou para as sprites |
| W5 | `seam_pattern.rs` (CLICA) | chip pintado e morto sob o dedo |
| W5 | `the_pattern_chip_opens_the_source_door_when_there_is_none` | a 4.ª condição |
| ~~W6~~ | ~~`the_handles_write_through_the_same_door_as_the_sliders`~~ ⛔ as alças foram retiradas (§6-quater) | ~~alça e número a divergirem~~ |
| W9 | `the_pattern_has_no_canvas_handles_anymore` | a próxima janela reconstruir as alças de boa-fé |
| W9 | `writing_the_phase_that_is_already_there_writes_nothing` | cada quadro do arrasto virar um passo de undo |
| W7 | `editing_the_source_shape_rebakes_the_tile` | o *dynamic* do Figma |

⚠️ **Prova de mutação em cada wave, com os três controlos no arnês** — e a regra que esta linha
aprendeu cinco vezes na jornada passada: *o dano vive um passo à frente do que o gate da feature
olha*. Um gate sobre o assador não prova a porta de render.

### §5.3 — ⚠️ Os gates de relógio desta feature são candidatos à FAMÍLIA de flakes

Qualquer barra que compare **duas medianas de um recurso** reprova sob fan-out (`CLAUDE.md` §5.0). O
gate de custo da W2 é, por isso, escrito sobre **a contagem de comandos codificados** (que é
determinística) e **não** sobre um relógio. *O `encode_cost_tests.rs` já foi escrito assim — é por
isso que ele é o instrumento certo.*

---

## §6 — As waves

| Wave | Entrega | Onde | Custa schema? |
|---|---|---|---|
| **W1** | **O assador** — `TileLaw`, `bake`, `placement`, `mode -> extend`. CPU puro, zero GPU, zero deps | `ph2d-vec-pattern` (folha nova) | não |
| **W2** | **A porta de render** — `fill_path_image` + `fill_pattern`; a regra de preenchimento e o `brush_transform` deixam de estar mortos | `ph2d-vector` + `ph2d-vec-render` | não |
| **W3** | **O dado** — `Paint::Pattern(Box<..>)`, tripla de schema, degrau da escada | `ph2d-vec-scene` + shell | **sim** (14->15, 99->100) |
| **W4** | **Fonte 1: uma IMAGEM** — `ph2d-asset-id` (extracção) · `rfd` pela porta da casa · `AssetDb` · persistência espelhando o `collect_sprite_pixels` | crate nova + shell | não |
| **W5** | **O painel** — 5.º chip + secção *Pattern* (8 sítios de costura + i18n + tokens), e a 4.ª condição | `ph2d-panel-vector` + `ph2d-editor-core` + `ph2d-i18n` + shell | não |
| ~~**W6**~~ | ~~**As alças na tela** — mover · escalar · rodar, espelhando o `GradHandle` (a lei do Inkscape)~~ ⛔ **CONSTRUÍDA E RETIRADA** por decisão do Enio (§6-quater) | ~~`ph2d-vec-render` + shell~~ | não |
| **W7** | ⭐ **Fonte 2: uma FORMA do documento**, viva — o modelo do Figma: editar a fonte re-assa o ladrilho em toda forma que a usa | shell | não |
| **W8** | **Cena de smoke** auto-verificável + os números medidos | `build_smoke.rs` | não |

**Kill-criterion (antes do build, DIRETIVA §5):** o desenho promete **custo de quadro igual ao de um
preenchimento sólido**. Se a W2 medir mais de **um** comando de desenho por forma com padrão, ou
qualquer camada de clip, **o desenho está errado** e o passo seguinte é achar porquê — ⛔ não subir a
barra. O assado (W1) tem orçamento próprio: **8 ms** para um ladrilho de 512×512 num reticulado hex
(o mesmo tecto que o [plano 23](23_plano_pattern_along_path.md) usou, e que ele bateu por 13×).

---

## §6-bis — O ESTADO, em 2026-08-27 (⚠️ audite esta tabela antes de pegar um item)

> *O §5 do `CLAUDE.md` só se edita na integração, então ele acumula trabalho já pago.* Esta tabela é
> o que a linha `line/Vector` de facto fechou, com o commit ao lado.

| Wave | Estado | O que ficou de fora |
|---|---|---|
| **W1** assador | ✅ | — (10/10 mutações mortas; assado medido em `1,047 ms` contra o kill de 8) |
| **W2** porta de render | ✅ | — (6/6 mutações mortas depois de uma corrigida) |
| **W3** o dado | ✅ | `VEC_SCENE` **14→15**, `PROJECT_SCHEMA` **99→100**; ⚠️ **RECONTE na integração** |
| **W4** fonte 1 (imagem) | ✅ | ⛔ instância de Motion pinta a `fallback` (fronteira declarada, com gate) |
| **W4b** persistência | ✅ | — |
| **W5** painel | ✅ | o 5.º chip + a fileira que reflui + a secção **Pattern** inteira (Source… · Tile · Offset · Size · Gap · Angle · Repeat), com 7/7 mutações mortas |
| ~~**W6** alças na tela~~ | ⛔ **RETIRADA** | construída (mover · escalar · rodar, 6/6 mutações mortas) e **apagada** no mesmo dia por decisão do Enio — **§6-quater** |
| **W7** fonte 2 (forma) | ✅ | o modelo do Figma, **viva** (editar a fonte re-assa), com o botão *Use Shape…* e a recusa do ciclo |
| **W8** smoke | ✅ | `PH2D_BUILD_SMOKE=76` |
| **W9** *Shift X/Y* no painel | ✅ | a POSIÇÃO passou para duas fileiras (§6-quater); a lei é uma fase `0..100 %` de UMA repetição, nos eixos do padrão |
| **W10** *Width/Height* + cadeado | ✅ | o TAMANHO passa a ser **dois** números, e a arte achata de propósito (§6-sexies). ⛔ **Schema: zero** |

⭐⭐ **AS OITO WAVES FECHARAM** (2026-08-27) — e a **W6 foi depois retirada**, com a posição a mudar
de sítio na W9. O que sobra é o que os reports do Enio abriram e o que ele decidir a seguir — a
lista está no §6-ter.

⚠️ **E a W5 deixou duas coisas nomeadas:**

- ✅ **O Size era UM número** (o lado maior, com o aspecto preservado) — **e deixou de ser** em
  2026-08-27, quando o Enio pediu para poder achatar a arte de propósito. O desenho é o que esta
  nota previa (**um cadeado de aspecto**, não dois campos soltos), com uma correcção que ela não
  antecipava: **o cadeado preserva a razão ACTUAL, não a natural da arte** — ver §6-sexies.
- **O Gap é UM número** para os dois eixos (`gap: [v, v]`). O dado guarda os dois; a UI oferece um.

⚠️ **Dois achados que mudaram o desenho a meio, e que a próxima janela herda:**

1. **A fileira de chips reimplementava a aritmética de largura à mão.** Com 4 ninguém notava; o 5.º
   fez morder. Hoje ela passa pelo `paint_segmented_group_adaptive`, que **reflui** — os quatro
   chips antigos voltaram aos `60,0 px` **de antes** e o quinto ganhou uma linha inteira de `252`.
   ⇒ a restrição que ia obrigar o rótulo `"Tile"` dissolveu-se, e o nome de produto cabe.
2. **A persistência mudou a AUTORIA.** O `insert_image_bytes` cunha o id do **ficheiro**; só o
   `insert_image_rgba8` (o id dos **pixels**) volta igual depois de um save. Com o primeiro, reabrir
   o projecto daria um id novo e a fonte apontaria para o nada — sem erro nenhum.

## §6-ter — O que os SMOKES do Enio abriram (2026-08-27)

| | estado |
|---|---|
| *"clamp deixa tudo em branco"* | ✅ o padrão nascia na **origem do mundo**; nasce no canto da forma, e o `Clamp` **enquadra** (derivado, nunca escrito) |
| *"volta para tile e o aspecto fica de clamp"* | ✅ o enquadramento deixou de ser gravado — trocar de modo escreve **um** campo |
| *"os parâmetros que um modo não usa não devem aparecer"* | ✅ no `Clamp` somem Tile · Offset · Size · Gap, e **voltam** ao sair; as três alças de canvas também |
| *"filters anula pattern"* | ✅ a rasterização isolada do FX não levava o ladrilho, e a imagem de FX **substitui** o desenho |
| *"em column o pattern some"* | ✅ o **vão** era assado na resolução da arte; o ladrilho é **reduzido até caber**, nunca recusado |
| *"não ficou legal [as alças]. vamos retirar e deixar os ajustes apenas no painel"* | ⛔ as três alças de canvas foram **apagadas**; a posição virou *Shift X/Y* no painel — **§6-quater** |
| ✅ *"pattern anula stroke"* / *"o contorno não volta ao trocar pattern por solid"* | ⭐⭐ **ERA A CENA DO SMOKE, e o produto nunca esteve errado** — fechado pelo terceiro report do Enio (*"o contorno funciona com as shapes que eu desejo, mas não funcionam com os teus desenhos"*), que é o que **nomeou a população**. Mecanismo no **§6-quinquies**. |

## §6-quater — ⛔ **AS ALÇAS DE CANVAS FORAM RETIRADAS** (Enio, 2026-08-27), e a posição mudou de sítio

> *"não ficou legal. vamos retirar e deixar os ajustes apenas no painel"*

**Veredito de produto, sem mecanismo nomeado.** Isto é uma decisão do dono, não uma medição — e é
por isso que ela fica escrita aqui **com o que existia**: uma 2.ª tentativa começa perguntando *o
que ficou pior*, nunca reconstruindo. (É a mesma disciplina da faixa de barras do `value.pattern` do
Motion, e das setas de canvas do plano 32, retiradas por ele em 25/08.)

### O que foi apagado

`ph2d-vec-render::pattern_handle` inteiro (o módulo + os gates), `PatternFill::handle_points`, os
campos `App::vec_pattern_drag` / `vec_pattern_selected`, os quatro sítios de fiação
(`vec_pattern_hit` · o move · o press com `history.begin` · o release com `commit_if_changed`) e o
desenho no `render_loop`. Ele existiu em `001b8ba43`; a árvore de antes deste corte é recuperável
daí.

### ⚠️ Retirar as três alças NÃO é retirar três ajustes — é retirar UM

O *Size* e o *Angle* já eram fileiras do painel: a alça de escalar e a de rodar tinham **irmã**, e
apagá-las não tira nada ao artista. A de **mover** não tinha: o `origin` só era escrito no
nascimento (`default_placement`) e por ela. ⇒ *"deixar os ajustes apenas no painel"* obrigava a que
a posição **passasse a existir no painel** — senão o corte teria silenciosamente **encolhido o
produto**, que é o oposto do que o pedido diz.

### A W9: *Shift X* e *Shift Y*, uma FASE de uma repetição

| | |
|---|---|
| **A faixa** | `0..100 %`, unipolar. ⭐ Ela **não é um palpite**: deslocar o padrão por um período inteiro é a **identidade**, então `100 %` é o mesmo que `0 %` e a faixa exprime **toda** a aparência distinta. O recurso que a fecha é a periodicidade do reticulado. |
| **⛔ A alternativa recusada** | uma coordenada de MUNDO crua. Ela é **ilimitada** (a forma vive onde quiser), quase todo o curso dela é repetição, e um slider precisa de duas pontas. |
| **Os eixos** | os do **PADRÃO**, não os do mundo — é ao longo deles que a repetição é periódica. Rodar o padrão roda o que o *Shift* desliza. |
| **A base** | o canto da **caixa da forma** — o MESMO canto em que a colocação nasce. Sem uma referência ligada à forma, a fase dependeria de onde a forma está no mundo. |
| **A porta única** | `PatternFill::shift` / `set_shift_axis` (`ph2d-vec-scene`) — a leitura que o painel mostra e a escrita que ele faz atravessam a **mesma** base ortonormal (`along_axes`), escrita uma vez. |
| **No `Clamp`** | some, com as outras quatro: ali a colocação é **derivada** e `origin` não tem quem o leia. |

⚠️⚠️ **Duas armadilhas que a implementação pagou, e que uma reescrita reencontra:**

1. **Só a parte FRACCIONÁRIA se mexe.** Reescrever a posição inteira teleportaria a origem para
   junto da base; no `Tile` isso é invisível (um período é a identidade), mas no `Mirror` a
   identidade são **dois** períodos e o reflexo trocaria de fase sozinho. *Uma escrita que só está
   certa num dos modos não é a lei.*
2. **A escrita tem de ser IDEMPOTENTE**, com tolerância **relativa ao período**. A ida e volta
   `origin -> fase -> origin` corre a **cada quadro** em que o slider está agarrado, e sem isso o
   último bit mudava sempre ⇒ **cada quadro um passo de undo** — o defeito que o `canonicalize` do
   editor curou para o mundo inteiro. O gate é
   `writing_the_phase_that_is_already_there_writes_nothing` (64 re-escritas, igualdade **exacta**).

### A cerca executável

`the_pattern_has_no_canvas_handles_anymore`
([`shells/desktop/tests/the_shape_art_picker_is_wired.rs`](../../shells/desktop/tests/the_shape_art_picker_is_wired.rs))
recusa os quatro nomes e **exige** que a porta que ficou no lugar exista — senão ele ficaria verde
num produto que **perdeu** a posição em vez de a ter mudado de sítio. ⚠️ *Uma decisão de produto que
vive só num documento é uma decisão que a próxima janela reconstrói de boa-fé.*

## §6-quinquies — ⭐⭐ O *"pattern anula stroke"* era a CENA DO SMOKE, não o produto

Três mensagens do Enio, e a terceira é a que resolveu — porque **nomeou a população**:

> *"o contorno funciona com as shapes que eu desejo, mas não funcionam com os teus desenhos"*

### O mecanismo, em três factos que só juntos explicam o report

1. A ferramenta de forma escreve `path.stroke = Some(self.style.stroke_spec(w))`
   **incondicionalmente** ([`shape.rs`](../../crates/ph2d-vec-edit/src/shape.rs)) ⇒ **toda** forma
   que o artista desenha nasce com contorno.
2. A cena do smoke construía os `VecPath` com `..VecPath::default()`, que é **`stroke: None`**.
3. O `restyle_selected_strokes` **recusa por desenho** quem não tem um — *"ganhar um traço do nada
   seria a UI inventando geometria"*
   ([`vector_bridge_style.rs`](../../shells/desktop/src/render_loop/vector_bridge_style.rs)).

⇒ a secção *Stroke* estava **pintada e inerte**, e **só nesta cena**. Isso lê-se exactamente como
*"o padrão anulou o contorno"* — e as duas buscas anteriores foram no sítio errado porque a
pergunta *"o padrão apaga o traço?"* tem resposta **não** em todo lado que se olhe.

### ⚠️ As três lições, e nenhuma é sobre padrões

- **Uma cena de smoke montada por CÓDIGO não herda o que a ferramenta de autoria garante.** Ela tem
  de nascer **no estado em que o artista a encontraria**; senão o smoke mede um objecto que o
  produto nunca produz, e o report que ele gera manda a próxima janela caçar um defeito que não
  existe. Gate: `every_shape_in_the_pattern_smoke_is_born_with_a_stroke`.
- **Um gate «não reproduzi» é uma afirmação sobre a POPULAÇÃO que ele mediu.** Os gates da wave
  construíam a forma como o *produto* a constrói (com traço) e estavam certos; a cena construía-a
  de outra maneira. *Reproduzir com a fixtura errada prova o quê?*
- **A 3.ª pergunta ao Enio não devia ter sido «rode o log».** A pergunta que resolveu foi *"em que
  formas?"* — e ela custava uma linha. O `PH2D_PATTERN_LOG=1` teria mostrado `SEM traco` e dito o
  mesmo, mas exigindo dele uma corrida com variável de ambiente. *Pergunte a população antes de
  entregar um instrumento.*

### ⏳ O que isto DESTAPOU, e que é decisão do Enio (não construído)

**Uma forma que nasceu sem contorno não pode ganhar um.** A recusa do `restyle_selected_strokes` é
deliberada e está comentada, mas o efeito é que a secção *Stroke* é oferecida e é **inerte** para
essa forma — e não há *"Add Stroke"* em lado nenhum. Hoje isso só se alcança pelo caminho que esta
cena usava, porque a autoria sempre veste; mas o Illustrator, o Figma e o Inkscape **todos** deixam
acrescentar um traço a qualquer forma. ⇒ Ou o verbo existe, ou a secção **some** para quem não tem
traço (a lei que a secção *Pattern* já obedece). ⛔ Não construído: é outra secção, e o Enio disse
que para as formas dele funciona.

## §7 — O que este plano NÃO faz, de propósito

- ⛔ **Padrão VECTORIAL ladrilhado** (resolução infinita ao ampliar). É irmão do `PathEffect::Hatch`
  na pilha de efeitos, não do `Paint`. A W7 dá o **resultado** (uma forma como fonte) pelo caminho
  raster; a versão vectorial é outra wave, com outro dono, e o §0.7 explica porquê.
- ⛔ **Preferência "o padrão anda com a forma?"** — a casa já decidiu (§2.2.1), e a Adobe mostra o
  preço de a transformar numa opção.
- ⛔ **Bicúbico.** O Vello 0.8 não o tinha (§0.1); prometê-lo seria prometer o que o substrato não faz.
  ⚠️⚠️ **ESTA RECUSA PRECISA DE SER RECONFERIDA:** a casa está na **`vello 0.10.0`** desde 2026-08-29
  ([ADR-0168](../architecture/decisions/0168-the-stack-rises-to-its-ceilings-and-four-dependencies-stay-behind-on-purpose.md)),
  e a **`0.9` tornou a `ImageQuality::High` bicúbica a sério** (Mitchell). ⇒ *o substrato passou
  provavelmente a fazê-lo*, e a recusa que dependia dele **pode ter dissolvido** — meça o `fine.wgsl`
  da 0.10 antes de repetir esta linha (§0.1.1).
- ⏸️ **Padrão no TRAÇO** (o Figma tem: *"as a fill or stroke"*) — **a última lacuna de paridade que
  este plano nomeia**, e o preenchimento fechou. ⚠️ **O preço foi MEDIDO em 2026-08-27, e não é o
  que a frase anterior sugeria** (*"o `StrokeSpec` é outra casa"*):
  - o modelo certo é o do SVG e o do Figma — **um traço tem um `Paint`, como um preenchimento**
    (`StrokeSpec.color: Rgba8` → `StrokeSpec.paint: Paint`);
  - mas o `Paint` carrega um `Vec<GradientStop>` e um `Box<PatternFill>` ⇒ **o `StrokeSpec` deixa de
    ser `Copy`**, e ele é `Copy` hoje (`#[derive(Copy, ..)]`,
    [`stroke_style.rs`](../../crates/ph2d-vec-scene/src/stroke_style.rs));
  - raio de explosão medido: **287 menções a `StrokeSpec` em 13 crates**, das quais **22 sítios
    copiam-no para fora de um *place*** (`if let Some(s) = path.stroke`, `let Some(old) = …`) e
    **quebram** — incluindo o `draw_stroke_with` e o `restyle_selected_strokes`;
  - mais **schema**: `VEC_SCENE_SCHEMA_VERSION` + a tripla + um degrau na escada.
  - ⛔ **A saída barata está RECUSADA por desenho**: um `stroke_pattern` à parte no `VecPath`
    manteria o `Copy` e daria ao traço **duas fontes de tinta** (`stroke.color` e o campo novo), que
    é exactamente o defeito de duas-portas contra o qual este plano inteiro foi escrito.
  ⇒ é **uma wave própria, com dono e ordem do Enio**, não um apêndice desta.
- ⏸️ **Navegador de assets.** O ADR-0165 chama-se *index before browser*; a W4 usa a porta de
  ficheiro que a casa já tem e que tem gate.

---

## §8 — Como a próxima janela começa

1. `cd` na worktree + `pwd` + `git branch --show-current` (a janela abre no primário).
2. **W1 primeiro, e ela não precisa de GPU nenhuma** — é uma folha pura com gates red-first.
3. ⚠️ **Reconte os três números do §0.4 contra o `main` do dia.** Eles somam entre linhas, e este
   ficheiro já viu duas recontagens (`96 -> 97`, `98 -> 99`).

---

## §6-sexies — ⭐ **W10: o tamanho passa a ter DOIS eixos, e a arte achata de propósito**

> Enio, 2026-08-27. O §6-bis já previa a forma da cura (*"um cadeado de aspecto, não dois campos
> soltos"*) — e errou num ponto que decide o produto.

### ⭐⭐ O cadeado preserva a razão **ACTUAL**, não a natural da arte

É a lei do *constrain proportions* do **Photoshop** e do **Figma**, e a diferença não é cosmética:
um cadeado que voltasse ao aspecto da imagem **desfaria o achatamento** que o artista acabou de
autorar, no instante em que ele mexesse no outro número. ⇒ com o cadeado, mexer num eixo escala
**os dois pelo mesmo factor**; sem ele, cada eixo é independente.

⚠️⚠️ **E é isso que faz o cadeado NÃO precisar de viajar no ficheiro.** Ele descreve o **gesto**
(*"estou a escalar proporcionalmente"*), não o padrão — o que se grava é o `size`, e ele já guarda
os dois eixos desde que a `Paint::Pattern` existe.

⇒ **`PROJECT_SCHEMA` e `VEC_SCENE` NÃO se mexem nesta wave.** ⛔ Um `lock_aspect: bool` no documento
seria estado persistente cujo único trabalho é **proibir o gesto que o artista pediu**, e ainda
custaria um degrau da escada.

| | |
|---|---|
| **A porta única** | `PatternFill::set_axis(axis, v, lock)` — o slider passa por aqui, e é ele que impede a lei de existir em dois sítios |
| **Onde o cadeado mora** | `App::texpat_lock_aspect` — sessão da shell, publicado ao painel na `TexturePatternRow`, e viaja no `TexPatCmd::Axis` |
| **A faixa** | a **MESMA** nos dois eixos. ⚠️ Um eixo com faixa própria faria o cadeado saturar num deles e continuar no outro, e a razão que ele promete preservar quebrava sozinha na ponta do curso |
| **O default** | **ligado** — o comportamento exacto que a secção tinha antes de os dois eixos existirem |
| **Gates** | 4 no modelo + 3 no shell + o seam (o cadeado clicado, e escondido no `Clamp` com o resto) |

### ⚠️ O que uma leitura rápida do diff entende ao CONTRÁRIO

1. **`set_longer_side` não foi «removido por limpeza»** — ele *era* a protecção, e a protecção
   mudou de **lei imposta** para **gesto escolhido**. Ela continua lá, com o cadeado ligado.
2. **Um `size` degenerado sob o cadeado vira quadrado**, e não é um caso esquecido: sem razão
   anterior não há razão a preservar, e `[v, v]` é o único par que satisfaz *"a razão de antes"*.
3. **O `VECTOR_TEXPAT_SIZE` continua definido no `editor-core`** (o bloco de ids é append-only) e
   deixou de ser re-exportado pelo painel — é assim que um id morre sem partir o hash de ninguém.
