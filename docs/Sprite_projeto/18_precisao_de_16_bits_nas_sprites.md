# 18 — Precisão de 16 bits nas sprites (o par `Format` volta, agora COM modelo)

> **Estado:** **W1–W6 FEITAS.** Abertos: exportar PNG de 16 bits (espera pedido real) e a sprite
> emissiva (produto). As recusas medidas estão na tabela no fim — **leia-a antes de propor
> qualquer coisa aqui**.
> **Ordem:** Enio, 2026-08-20 — *"vamos corrigir a UI da sprite no painel com as duas opções e com os
> botões que a partir de agora devem converter a imagem."*
> **Antecedente obrigatório:** [`17_plano_render_source_e_hand_packed.md`](17_plano_render_source_e_hand_packed.md) §5,
> que **retirou** o par `RGBA8 / RGBA16` do Inspector por ele ser inerte. Este doc é o que o §5 chamou
> de *"wave própria"*.

---

## §1 — A objeção foi levantada, medida, e o Enio decidiu mesmo assim

O protocolo (`CLAUDE.md` §0.2/§6) manda **PARAR e reportar** quando o trabalho toca contrato
congelado. Foi feito. A objeção apresentada, com os números:

1. **Bloom / iluminação não ganham nada.** O `GameRt` já é `Rgba16Float` com folga acima de 1.0 e o
   `fx_stack` já mantém os intermediários em meio-float — *o brilho não vem do arquivo da sprite,
   vem da luz somada em cima dela, e essa conta já é 16 bits*.
2. **8 → 16 não cria informação.** Uma sprite que nasceu 8-bit convertida para 16 continua com 256
   degraus por canal; ganha prateleiras vazias.
3. **O defeito visual real** (anéis/faixas num degradê limpo) está na **descida final sem dither**.
   ⚠️ ~~`git grep -i dither` devolve **um** hit, e é um rótulo de painel de smoke.~~ **A evidência
   estava ERRADA, e foi corrigida ao construir a W6:** o grep devolve **dezenas**, e o Color
   Equalization tem um Floyd–Steinberg completo, com dois sliders e gate de brilho médio
   ([`posterize_quantize.rs`](../../crates/ph2d-tool-color-equalization/src/algorithm/posterize_quantize.rs)).
   ⛔ **Mas ele é OUTRA COISA:** aquele é um efeito **de estilo** — reduz a imagem a N níveis *de
   propósito* para dar o aspecto de arte por pixels. O que faltava é o **técnico e invisível**, de
   amplitude abaixo de meio passo, na quantização final. *Duas coisas com o mesmo nome, e só uma
   delas o utilizador quer ver.*
   A **afirmação** («não há dither na quantização final») era verdadeira; a **prova** citada não a
   provava. *Uma nota de diferido não é spec — confere-se e corrige-se no sítio.*

**Veredito do Enio: construir 16 bits de verdade.** A decisão está tomada e este doc a executa. O
que os três pontos acima passam a valer é **escopo**: eles dizem onde 16 bits *não* deve ser
vendido como ganho, e por isso o §6 (Fora de escopo) é parte do entregável, não uma ressalva.

---

## §2 — As cinco medições que decidem o desenho (2026-08-20)

Nenhuma delas era conhecida quando o §5 do doc 17 adiou isto. Duas delas **reduzem** o trabalho.

| # | Medição | Onde se confere | Consequência |
|---|---|---|---|
| M1 | **A representação de alta precisão JÁ EXISTE e já é congelada.** `DecodedImage::FlatHdr(ImageBuffer<LinearRgba>)`, `LinearRgba` = `f32`×4 linear | [`decoded.rs:39`](../../crates/ph2d-imageio/src/decoded.rs) · [`linear.rs:14`](../../crates/ph2d-color/src/linear.rs) | **Não é preciso emendar o cap de 5 variantes do ADR-0054.** O contrato já reservou o lugar |
| M2 | **O AVIF já PRODUZ `FlatHdr`**, com round-trip testado (10-bit PQ/Rec.2020) | [`avif/decode.rs:211`](../../crates/ph2d-imageio-avif/src/decode.rs) · [`decode_encode.rs:152`](../../crates/ph2d-imageio-avif/tests/decode_encode.rs) | O caminho de importação de alta precisão **não é hipotético** — ele funciona e morre adiante |
| M3 | **A parede é UMA linha, e ela é explícita.** `DecodedImage::FlatHdr(_) => Err(AssetError::Decode{ "HDR import not yet bridged to AssetDb" })` | [`loader.rs:290`](../../crates/ph2d-asset/src/loader.rs) | O trabalho não é «criar precisão»; é **parar de a deitar fora na porta** |
| M4 | **O armazenamento é 8-bit POR NOME, e é `#[non_exhaustive]` de propósito.** `Asset::ImageRgba8 { width, height, pixels: Arc<[u8]> }`; o cabeçalho diz *"adding variants doesn't break downstream matches"*. **56 referências em 20 ficheiros** | [`asset.rs:18`](../../crates/ph2d-asset/src/asset.rs) · `git grep -c ImageRgba8` | Variante irmã é **exatamente** o ponto de extensão append-only que o `CLAUDE.md` §0.2-B' exige de foundational novo |
| M5 | **`Rgba16Unorm` é feature OPCIONAL; `Rgba16Float` é baseline.** `TEXTURE_FORMAT_16BIT_NORM` é pedido **mascarado pelo adapter** em [`context.rs:73`](../../crates/ph2d-gpu/src/context.rs); o wgpu-types 27 exige-a para `Rgba16Unorm` (lib.rs:3341) e não para `Rgba16Float` | `wgpu-types-27.0.1/src/lib.rs` | **A textura é `Rgba16Float`.** Escolher `Rgba16Unorm` faria a feature nascer com uma máquina em que ela não liga |

### M5-bis — a armadilha que M5 esconde, e que custa um bug silencioso

⚠️ **Não existe variante sRGB de NENHUM formato de 16 bits.** Hoje toda textura de sprite é
`Rgba8UnormSrgb` ([`atlas/gpu_ops.rs:62`](../../crates/ph2d-render/src/atlas/gpu_ops.rs),
[`individual.rs:785`](../../crates/ph2d-render/src/individual.rs)) e o **hardware** faz a conversão
sRGB→linear na amostragem. Um `Rgba16Float` **não faz**.

Logo: *uma sprite de 16 bits guarda LINEAR, e o shader não pode converter outra vez*. Se este
parágrafo for ignorado, a sprite de 16 bits renderiza **visivelmente mais clara** que a gémea de 8
bits e o defeito parece «a conversão está errada» quando o errado é o espaço. **A W2 abre com o gate
de paridade que mede exatamente isto** (§4).

---

## §3 — O desenho

### §3.1 — Duas precisões, uma sprite

```
Rgba8   (default)  — sRGB-encoded, 4 B/px, textura `Rgba8UnormSrgb`, conversão em hardware
Rgba16  (alta)     — LINEAR, meio-float, 8 B/px, textura `Rgba16Float`, sem conversão
```

⛔ **A conversão de precisão NÃO mexe em premultiplicação** (decidido ao construir a W1.1, contra a
primeira redação deste parágrafo). A associação de alfa continua a ser dita pelo
`Sprite::premultiplied`, como sempre foi — juntar as duas leis faria uma troca de precisão mudar
**silenciosamente** a composição, que é o género de acoplamento que este projeto já pagou. Precisão e
espaço de cor entram; alfa sai como entrou, e há gate.

**Por que meio-float e não inteiro de 16 bits**, já que um PNG de 16 bits é unorm: M5. E o custo de
precisão é aceitável e mensurável — perto de 1.0 o meio-float dá passo `2⁻¹¹` (≈2048 níveis, 8× o de
8 bits) e nos escuros dá **muito** mais que unorm16, que é onde o olho está. Em troca vem folga
acima de 1.0 de graça, que é a única porta pela qual uma sprite pode um dia **ser** fonte de luz
(§6.1). É também a moeda que o resto da engine já usa (`GameRt`, `fx_stack`, `walk_gpu`,
`mesh-render`) — *dois motores, um estado* não se aplica porque não há segundo motor: há um formato
a mais no mesmo.

### §3.2 — A conversão, nos dois sentidos

| Sentido | Perde? | O que de facto acontece |
|---|---|---|
| **8 → 16** | não | `srgb_to_linear` por canal, exato. **Não cria informação** — o valor é preservar precisão nas edições SEGUINTES. O botão diz isto |
| **16 → 8** | **sim** | `linear_to_srgb_byte` + (W5) dither. Um caminho só, e é o mesmo que a descida final já usa |
| **8 → 16 → 8** | não | idempotente byte-a-byte se nada acontecer no meio. **É gate** (§4) |

### §3.3 — O que 16 bits IMPEDE, e que tem de aparecer na UI

⚠️ **Uma sprite de 16 bits não pode viver no ATLAS.** O atlas é **uma** textura com **um** formato
([`atlas/mod.rs:197`](../../crates/ph2d-render/src/atlas/mod.rs) — um único `MipGenerator` para toda
ela, e o doc-comment diz porquê). Uma sprite de 16 bits é, portanto, obrigatoriamente `Individual`
ou `HandPacked` numa folha de 16 bits.

Isto **não é limitação a esconder**: é a mesma lei que a seção Render Source já exprime. Converter
para 16 bits **muda a estratégia** da sprite, e o Inspector tem de o dizer **antes**, não depois.

### §3.4 — As ferramentas de 8 bits

O contrato `RasterEditTool` é **congelado** (§6 do `CLAUDE.md`, `Tool=12`/`RasterEditTool=5`, gate
`architecture_tool_contract_surface`) e a sua `current_preview` devolve `(&[u8], u32, u32)`
([`tool.rs:186`](../../crates/ph2d-editor-core/src/tool.rs)).

⛔ **Este plano NÃO o toca.** O gate conta **métodos** (cap 5), então trocar a assinatura passaria
mudo — e passar mudo por um contrato congelado é pior do que reprovar. A regra é outra:

> **Uma ferramenta de 8 bits aplicada a uma sprite de 16 bits converte-a para 8 e AVISA antes.**

É honesto (o resultado da ferramenta é mesmo 8 bits), é uma linha de código no ponto de entrega, e
deixa a porta aberta para uma emenda por ADR **quando houver uma ferramenta que de facto queira 16
bits** — que hoje não existe. *Um contrato congelado emenda-se por necessidade medida, não por
simetria.*

---

## §4 — As waves

Cada wave é fechável e testável sozinha. A ordem é obrigatória: cada uma remove a parede da seguinte.

### W1 — A representação (foundational, append-only) ✅ **FEITA** (`80a9e9afa`, `b67b9679c`)
- ✅ [`precision.rs`](../../crates/ph2d-imageio/src/precision.rs) — `Precision` + as quatro
  conversões, meio-float à mão (o `f16` nativo é instável no toolchain fixado), e a lei exaustiva
  `8 → 16 → 8` sobre os 256 bytes.
- ✅ `Asset::ImageRgba16 { width, height, pixels: Arc<[u16]> }` (M4) + `Asset::image_rgba8` (porta
  única, `Cow`) + `Asset::precision`.

⚠️ **A prova de mutação mudou o desenho dos gates.** Trocar *round-to-nearest-even* por truncagem
deixou **verdes** a tabela IEEE (valores sem bits a largar) **e** a varredura exaustiva (a folga
entre a precisão do meio-float e o espaçamento sRGB absorve o erro). *O mutante sobrevivente era o
nome do gate em falta* — `the_rounding_is_nearest_even_and_not_truncation`.

### W1-bis — ⚠️ A ORDEM MUDOU, e a razão é a lei desta linha

A redação original mandava a W1 **também** virar os importadores de PNG/TIFF para preservarem 16
bits, e o `loader.rs` deixar de recusar `FlatHdr` (M3). **Está movido para o FIM da W2**, e não é
detalhe de agenda:

> Virar a importação antes de a GPU saber desenhar 16 bits faria uma imagem de 16 bits ser
> importada com sucesso e **não aparecer** — trocando um erro claro (*"HDR import not yet bridged"*)
> por um silêncio. É a mesma falha que esta linha acabou de curar duas vezes (o pill pintado e morto;
> os oito pills sem `InteractiveState`): **uma capacidade que nasce antes do seu consumidor nasce
> muda**, e o gate que a mede escolhe a ordem cómoda.

⛔ Não reverter esta ordem sem ler [`the_registry_is_installed_before_the_hero.rs`](../../shells/desktop/tests/the_registry_is_installed_before_the_hero.rs).

### W2 — A GPU (⚠️ abre com o gate de M5-bis) · **a wave grande** — 3 de 4 feitas

- ✅ **W2.1** (`d6276c490`) — `IndividualTextureStore::acquire_16` + `FORMAT_16` + um `MipGenerator`
  por formato. O gate `precision_parity_gpu` **reprovou sobre código certo**, e o diagnóstico é o
  achado: `esperado 0,701102 · gpu16 0,701172 · gpu8 0,699219` — quem se afasta da curva sRGB exacta
  é o caminho de **8 bits**, porque *a decodificação sRGB do hardware é aproximada*. A barra passou
  a ser **a imagem** (os dois voltam a byte sRGB e têm de dar o mesmo byte).
  ⚠️ *Quando um gate reprova, a primeira pergunta é qual dos dois lados é o oráculo.*
- ✅ **W2.2** (`9c745088f`) — `SpriteRenderer::acquire_individual_16` + `individual_format`.
- ✅ **W2.3** (`71902be05`) — a auditoria. Doze sítios de falha **silenciosa**, dez encaminhados pela
  porta `Asset::image_rgba8` / `image_dimensions`, e o gate
  [`reading_pixels_goes_through_the_precision_door`](../../shells/desktop/tests/reading_pixels_goes_through_the_precision_door.rs)
  a impedir o décimo-terceiro — **sem allowlist central**: o bypass declara-se no sítio com
  `PRECISION-BYPASS:`.
- ⏳ **W2.4** — virar a importação. ⚠️ **BLOQUEADA PELA W3**, ver W2-ter.

### W2-ter — ⚠️ A ORDEM MUDA OUTRA VEZ, e desta vez foi a auditoria que a mudou

A W1-bis já tinha movido a viragem da importação para o fim da W2 (uma capacidade sem consumidor
nasce muda). A auditoria da W2.3 mostrou que **isso ainda é cedo**:

> `project_sprite_pixels.rs` e `project_assets.rs` são caminhos de **ESCRITA**. Enquanto eles não
> souberem gravar 16 bits, um asset de 16 bits que exista é **silenciosamente rebaixado a 8 no
> save** — ou pior, descartado. Importar 16 bits antes disso não dá ao artista precisão: dá-lhe uma
> precisão que **desaparece ao gravar**, que é pior que não a ter.

⛔ **A viragem da importação é o ÚLTIMO passo da W3, não o último da W2.**

*As duas correcções de ordem desta wave têm a mesma forma: o que decide a sequência não é a
dependência de compilação, é onde o dado morre em silêncio.*

### W2 (referência) — o que a wave continha
- Store `Rgba16Float` irmão do `individual.rs`.
- **Gate de paridade primeiro**: a mesma imagem pelos dois caminhos tem de renderizar dentro da barra
  derivada do formato. É este gate que apanha a dupla-conversão de sRGB, e ele é escrito **antes** do
  código que ele mede.
- Mip generator do formato novo.
- **A auditoria dos 56 sítios de `ImageRgba8`** (`git grep -n ImageRgba8 -- crates shells`, 20
  ficheiros). ⚠️ **O `cargo check` NÃO a entrega**: `Asset` é `#[non_exhaustive]`, então todo `match`
  de fora da crate tem braço `_` e **aceita a variante nova em silêncio**. Ou o sítio passa pela
  porta `Asset::image_rgba8`, ou trata o ramo — e quem decide é a leitura, não o compilador.
- **Por fim**, e só aqui: virar a importação (W1-bis) — `loader.rs` deixa de recusar `FlatHdr` (M3)
  e os importadores de PNG/TIFF param de esmagar 16 bits → 8
  ([`imageio-png/src/lib.rs:16`](../../crates/ph2d-imageio-png/src/lib.rs) documenta a perda hoje, e
  há teste a fixá-la: `import_quantizes_16bit_to_8bit_with_documented_loss` — **esse teste inverte-se**).

### W3 — Persistência · **feita, e ela custou MENOS do que o plano previa**

- ✅ `SpritePixelDoc.rgba: Vec<u8>` → `pixels: PixelPayload` (`Rgba8` | `Rgba16`), `SHEET_DOC_VERSION`
  3 → 4. **A variante É a precisão**: um campo `precision` ao lado do payload permitiria um
  documento que diz 16 e carrega 8, e alguém teria de o validar em toda leitura. *A representação
  apaga o caso especial.*
- ✅ **Migração de v3**, com `postcard::take_from_bytes` a ler só o cabeçalho e uma cópia congelada
  da forma antiga. ⚠️ **Os bumps v1→v2→v3 recusavam, e podiam** — aconteceram na jornada em que o
  formato nasceu. *Um formato só precisa de migração a partir do dia em que alguém guardou alguma
  coisa nele*, e a v4 é o primeiro depois disso. O gate constrói bytes com a forma v3 **verdadeira**,
  não serializando o tipo de hoje — senão provaria apenas que hoje concorda consigo próprio.
- ✅ `AssetDb::insert_image_rgba16` + o restore a honrar a precisão do documento nos **dois** passos
  (asset e textura).

**Duas correcções ao plano, ambas descobertas a construir:**

1. ⛔ **O `PROJECT_SCHEMA` NÃO se move.** O `sprite_pixels` é um blob auto-versionado dentro do
   `ProjectFile` — o mesmo desenho que fez o hand-packed custar zero recusa. Era isso que o campo
   sempre prometeu; a linha do plano que mandava bumpar não tinha lido a promessa.
2. ⛔ **O `SavedAsset` NÃO ganha ramo de 16 bits**, e a nota da W2.3 que dizia que ganharia estava
   errada. Ele percorre o `atlas_asset_map`, e **o atlas é de 8 bits por construção**; uma sprite de
   16 bits é `Individual`, e os pixels dela gravam-se pelo `SpritePixelDoc`, que já sabe.
   ⚠️ A regra *16 bits ⇒ `Individual`* é o que torna aquela linha verdadeira, e impõe-se **onde a
   precisão nasce** (a conversão da W5 e a importação da W2.4) — nunca acrescentando um ramo ao
   caminho de gravação do atlas, que só gravaria o rebaixamento com mais passos.

**Uma prova de mutação que corrigiu a MINHA justificação:** eu escrevi que o discriminante no hash
do `insert_image_rgba16` era o que separava as duas precisões. Removê-lo deixou o gate **verde** — o
payload de 16 bits tem o dobro dos elementos, logo os `hash_input` nunca têm o mesmo comprimento.
*Uma justificação que uma mutação não confirma é uma hipótese com cara de facto.* O gate foi
reapontado para a falha **realista** (implementar o de 16 derivando o id do caminho de 8), e essa
reprova.

### W2.4 — virar a importação · ✅ **FEITA**
- ✅ `loader.rs` deixa de recusar `FlatHdr` e produz `Asset::ImageRgba16` (`f32` linear →
  meio-float; ⚠️ **sem curva** — o `FlatHdr` já é linear, e aplicar `linear_to_srgb` ali, que é o
  reflexo de quem vê *"cor a entrar"*, escureceria a imagem sem erro nenhum).
- ✅ O importador de PNG preserva 16 bits. O teste
  `import_quantizes_16bit_to_8bit_with_documented_loss` **inverteu-se**: ⚠️ *quando a premissa de um
  teste morre, ele não se silencia — inverte-se, e o commit diz porquê.* A fixture nova usa `0x8080`
  e `0x8081`, que quantizam para o **mesmo** byte de 8 bits: se a importação ainda esmagasse, eles
  sairiam idênticos.
- ✅ `srgb_to_linear_unit` em `ph2d-color`, e o `srgb_to_linear_byte` passa a delegar nela — **uma
  curva, não duas** que um dia divirjam na terceira casa. Uma fonte de 16 bits não passa por `u8`.
- ✅ **A regra *16 bits ⇒ `Individual`* impõe-se aqui**, com `PackedSource` a substituir o
  `cell_idx` cru: uma imagem de alta precisão não entra no atlas **nem no `atlas_asset_map`** — e é
  isso que mantém verdadeira a linha do `project_assets.rs` que grava as células em 8 bits.
  ⛔ **A alternativa recusada** era converter para 8 e meter no atlas na mesma: importaria um
  ficheiro de alta precisão **rebaixando-o em silêncio**, que é o que esta wave existe para acabar.
- ✅ ⚠️ **O carimbo `SpritePixels`, que é a metade que se esquece.** Uma `Individual` sem ele abre
  perfeita e grava **vazia** — o `texture_id` é uma alocação de GPU e morre com o processo. A
  estratégia errada dá uma sprite visivelmente partida; o carimbo em falta dá uma que só falha no
  save, e **nada no ecrã distingue as duas**. Gate + mutação.
- ✅ O contador de células do atlas só avança quando uma célula foi mesmo consumida — senão cada
  import de 16 bits abriria um buraco no atlas.

### W4 — As ferramentas · ✅ **FEITA**
- ✅ O aviso mora no **funil** (`commit_edited_texture`), num sítio só: toda ferramenta de imagem
  trabalha em 8 bits (o `SpriteImage` é `Vec<u8>`) e todas escrevem de volta por ali.
- ⚠️ **O aviso é DEPOIS, não antes**, e é decisão: *antes* exigiria interceptar a activação de cada
  uma das nove ferramentas para dizer o que este único sítio sabe de facto.
- ⛔ **`RasterEditTool` NÃO foi tocado** (§3.4) — nenhuma ferramenta pede 16 bits hoje, e emendar um
  contrato congelado por **simetria** em vez de por necessidade medida é pagá-lo duas vezes.

### W5 — A UI · ✅ **FEITA** (o que o Enio pediu ver)
- ✅ A linha `Format` diz a precisão **MEDIDA**. ⚠️ Ela já mentiu **duas** vezes: primeiro como par
  sem dispatch com o aceso literal; depois como facto **derivado da estratégia**, que dizia "RGBA8"
  para toda a gente. *Uma linha de proveniência que deriva o que devia medir mente na primeira
  exceção.* Sem medição ela diz `—`, nunca um palpite.
- ✅ O par `RGBA8 / RGBA16` volta — **pintado, registado E com braço no `event.rs`**, e o gate
  [`seam_precision`](../../crates/ph2d-panel-inspector/tests/seam_precision.rs) afirma as três
  juntas, porque cada uma falha em silêncio de maneira diferente. ⛔ Ressuscitar aqueles ids só é
  legítimo porque agora existe **modelo** por trás deles.
- ✅ A conversão vive no shell ([`precision_convert.rs`](../../shells/desktop/src/precision_convert.rs)),
  lê do **`AssetDb`** e nunca da GPU (a textura pode ter sido mexida por uma prévia), e sai pela
  **mesma cauda** que toda ferramenta atravessa — `rebind_to_individual`, extraída para que as cinco
  invariantes de *"esta sprite passou a ter pixels próprios"* não tenham duas cópias. Duas delas só
  falham **depois de fechar e reabrir o projeto**.
- ✅ A nota de custo aparece **antes** do clique (*"RGBA16 doubles memory and forces Individual"*) —
  *uma consequência que só aparece depois do clique lê-se como um bug*.

### W6 — O dither da descida para 8 bits · ✅ **FEITA** (metade), ⛔ **RECUSADA** (a outra)

> Enio, 2026-08-21: *"siga implementando"*, sobre o item que a lista de abertos chamava de *"o
> defeito visual **real** desta conversa"*.

⚠️ **A wave partiu-se em duas ao ser medida, e as duas metades chegaram a respostas opostas.** Há
**duas** descidas para 8 bits, e elas não se parecem nada:

| | onde | quem manda | dither? |
|---|---|---|---|
| **W6.1** | o botão `RGBA8` do Inspector | **o autor**, e o resultado fica gravado | ✅ **ship*ou*** |
| **W6.2** | o passe de tonemap → ecrã | ninguém; acontece a cada quadro | ⛔ **recusado, com número** |

#### W6.1 — a descida que o autor COMANDA ✅

[`ph2d-color/src/dither.rs`](../../crates/ph2d-color/src/dither.rs). Bayer 8×8 ordenado, **não**
Floyd–Steinberg: o padrão é função **só da posição do pixel**, e numa engine em que a mesma imagem é
recortada e ladrilhada um dither que dependesse do varrimento faria o mesmo pixel sair diferente
conforme o recorte em que calhou (HR-5 também proíbe RNG por quadro). A matriz é **derivada** da
recorrência num `const fn` — 64 números à mão continuariam a *parecer* um dither se um estivesse
trocado.

⚠️ **A amplitude é MEDIDA, e é ela que torna a ida-e-volta exacta.** Meio passo inteiro estragaria
arte por pixels: o valor que chega já não está na grelha, porque a mantissa de 11 bits do meio-float
o devolve ao lado. Medido sobre os 256 bytes:

| canal | deriva máxima | onde |
|---|---|---|
| cor (atravessa a curva sRGB) | 0,037231 LSB | byte 192 |
| **alfa** (escala directa) | **0,062012 LSB** | byte 239 |

⚠️ **É o alfa que manda, e é contra-intuitivo:** sem curva nenhuma, o erro relativo do meio-float
(2⁻¹²) aparece **inteiro** em LSB no topo da faixa, enquanto na cor a derivada da sRGB o encolhe. *O
canal sem curva é o que tem menos folga.* Daí `DITHER_SPAN_LSB = 1 − 2 × 0,062012`.

Gate exaustivo: 256 bytes × 64 células = **16 384 casos**, nenhum byte já na grelha se mexe. **Prova
de mutação:** com a deriva a `0.0` (o meio passo «de manual») o gate morre alto, 96 valores movidos.

**Duas portas, de propósito:** `rgba16_to_rgba8` é **fiel** e serve leituras (*um read que devolvesse
valores diferentes dos guardados não é um read*); `rgba16_to_rgba8_dithered` serve o botão. Há gate
de costura contra o refactor que as colapsa — ele não daria erro nenhum, e as faixas voltavam em
silêncio.

**Smoke:** `PH2D_DITHER_SMOKE=1` — **uma** sprite partida ao meio: cima a descida fiel (faixas),
baixo a com dither (liso). A fixture atravessa **seis** códigos sRGB ao longo de 512 px (~85 px por
faixa) porque a banda é um defeito de degradês **lentos**; uma fixture rápida esconderia o fenómeno
que ela devia conter, e há três gates sobre ela a dizê-lo.

⚠️ **A primeira versão do smoke saía lisa dos DOIS lados, e a causa não estava no dither.** Enio,
2026-08-21: *"com o zoom máximo ainda estão lisas"*. O filtro de imagem do projeto é **`Smooth` por
omissão** (bilinear + anisotropia 16×) e em **ampliação** o bilinear interpola entre texels vizinhos
— um degrau de um código vira uma rampa. *Aproximar o zoom não mostrava o defeito: apagava-o.* A
sprite passa a trazer o seu próprio `TextureFilter(Nearest)`, e a cena deixou de ser duas imagens
vizinhas para ser **uma partida ao meio**: em cada coluna as duas metades partem do mesmo valor, e as
arestas de cima **param na costura**. *O olho compara através de uma fronteira partilhada muito
melhor do que entre dois quadrados separados por um vão* — e aqui a diferença é de **um** código, o
passo mais pequeno que existe.

⚠️ **Uma propriedade do Bayer que só apareceu ao escrever esses gates:** o espalhamento dos limiares
**por coluna** é desigual — a coluna `x%8 == 0` cobre a faixa toda (0…63), a `x%8 == 3` só o meio
(20…43, ou seja ±0,21 em vez de ±0,43). Metade das colunas dither*a* com meia amplitude, e é daí que
vem o xadrez característico do dither ordenado. Medido: **56%** das colunas se misturam, não os ~86%
que a previsão analítica dava. ⛔ Não é defeito a curar — o olho integra ao longo de x.

#### W6.2 — a descida do ECRÃ ⛔ recusada, e o que ficou no lugar

Construída, medida ([`tonemap_descent_gpu`](../../crates/ph2d-render/tests/tonemap_descent_gpu.rs),
RTX + wgpu 28) e **revertida** — o código executável do shader não mudou uma linha:

| | |
|---|---|
| folga máxima que não move byte nenhum | **~0,0283 LSB** (de 0,5 possíveis) |
| com o pico que a W6.1 usa (0,4311 LSB) | **5,98%** dos pixels movidos |

Um dither ali teria de caber em **7%** da amplitude do caminho de software, e a 7% não espalha nada.
A alternativa é 6% de uma cor chapada virar mosquito — numa ferramenta de arte por pixels, o defeito
pior de todos.

⚠️ **O mecanismo, que é o que impede reconstruir isto:** o valor que chega ao tonemap é
`hw_decode(byte)`, e a **tabela sRGB do hardware não é a curva ideal** (a W2 já a tinha medido a
`0,00195` em linear, ~0,34 de um código). `hw_encode(hw_decode(N)) == N` é garantido pelas
especificações — mas só enquanto ninguém empurra o valor pelo meio. Um shader que re-codifique com a
curva *ideal*, some o viés e volte a descodificar mede a distância à fronteira com uma régua que não
é a do hardware, e a folga que sobra é **propriedade da placa**. ⛔ Encolher a amplitude até passar
trocaria um defeito visível por um número ajustado a uma placa só.

**No lugar do dither ficaram duas coisas que não existiam:**

1. ✅ **`a_flat_eight_bit_colour_survives_the_descent`** — o gate de que uma cor chapada atravessa o
   passe final **byte-exacta**, os 256 bytes nas 64 posições. É a promessa central de uma ferramenta
   2D e atravessa três traduções que ninguém escolheu; ⚠️ ela é verdadeira **por cancelamento, não
   por exactidão**, e nunca tinha sido medida.
2. ✅ **A sonda**, que mede a folga em **qualquer** máquina. Quem quiser reabrir isto começa por a
   correr, não por escrever um shader.

⚠️ **O erro que a sonda cometeu primeiro vale mais que o resultado.** A primeira versão somava o viés
**antes** do meio-float — outra cadeia: o `f16` é uma quantização, e um viés infinitesimal que mude o
lado para que ele arredonda vira um salto de 0,037 LSB. Ela respondia *«folga = 0,0000»*, que é a
resposta certa à pergunta errada. O **controle negativo** (viés zero tem de dar exactamente o que o
passe de produção dá) é o que agora o impede, e não existia. *Um aparelho que mede uma cadeia
diferente da que se vai construir dá um número verdadeiro sobre nada.*

#### E uma nota que esta wave invalidou

O gatilho de migração do AgX em [`tonemap.wgsl`](../../crates/ph2d-render/src/shaders/tonemap.wgsl)
dizia «quando a importação HDR (EXR / HDR / **PNG de 16 bits**) ship*ar*». **Ship*ou* — na W2.4
desta mesma wave.** A nota está corrigida no sítio (`CLAUDE.md` §0.0: *quem move o número que tornava
algo inalcançável tem de reconferir a nota*). ⛔ E mesmo assim o LUT **não** se acende: o que falta
não é o gatilho, é o **bake**, que continua TBD — e o LUT identidade aplica a curva log que produz o
*«dull look»* que o Enio já recusou na ronda 7 do M14.5.

### ⏳ O que fica ABERTO, e por quê

1. **Exportar PNG de 16 bits** — a importação preserva; o `PngExporter` continua a emitir 8 bits e a
   recusar `FlatHdr`. Espera um pedido real.
2. ⛔ **Sprite emissiva** (§6.1) — o `Rgba16Float` dá a folga acima de 1.0 de graça, e não há
   conceito de emissivo em sprite nenhuma. É produto, não formato.

---

## §5 — Custo, medido e não estimado

| Grandeza | 8 bits | 16 bits |
|---|---|---|
| Bytes/px | 4 | 8 |
| Folha 4096² | 64 MiB | 128 MiB |
| Pode ir ao atlas | sim | **não** (§3.3) |
| Ganho em bloom/luz | — | **nenhum** (§1.1) |
| Ganho em importar 16-bit | — | **real**: hoje perde-se na porta (M3) |
| Ganho em edição empilhada | — | **real**: evita requantizar entre passos |

---

## §6 — Fora de escopo (com o motivo, para não ser reconstruído)

1. ⛔ **Sprite emissiva / fonte de luz.** O `Rgba16Float` dá a folga acima de 1.0 *de graça*, mas
   **não existe conceito de emissivo em sprite nenhuma** (`git grep -i emissive` sobre `crates/*/src`
   + `shells/desktop/src` = **zero**). Ligar isto é produto, não formato.
2. ⛔ **Mudar `RasterEditTool`.** §3.4 — por necessidade medida, não por simetria.
3. ⛔ **32 bits.** `LinearRgba` é `f32` no decode porque é a moeda do importador; **armazenar** f32
   seria 16 B/px (4×) sem consumidor que o peça.
4. ⛔ **Vender isto como melhoria de bloom.** §1.1. A UI não pode sugeri-lo.
5. ⛔ **Dither no passe de tonemap** (W6.2). Folga medida: **0,0283 LSB** de 0,5 — 7% do que o
   caminho de software usa. A tabela sRGB do hardware não é a curva ideal, e a folga é propriedade
   **da placa**. Sonda: `tonemap_descent_gpu`.

---

## ⛔ Recusas MEDIDAS

| O que foi tentado | O que a medição disse | Onde |
|---|---|---|
| Dither no passe de tonemap (ecrã) | folga **0,0283 LSB** de 0,5; ao pico da CPU, **5,98%** dos pixels movem | [`tonemap.wgsl`](../../crates/ph2d-render/src/shaders/tonemap.wgsl) · [sonda](../../crates/ph2d-render/tests/tonemap_descent_gpu.rs) |
| Dither com meio passo inteiro (o «de manual») | move 96 valores que já estavam na grelha | [`dither.rs`](../../crates/ph2d-color/src/dither.rs) |
| Acender o LUT AgX agora que o gatilho disparou | falta o **bake**; o LUT identidade aplica curva log (o *dull look* recusado no M14.5 r7) | [`tonemap.wgsl`](../../crates/ph2d-render/src/shaders/tonemap.wgsl) |
| Preservar 16 bits no Upscale filtrado / Rasterize / Equalize / Color-Eq / Painter | eles **calculam** o pixel de saída; sem resampler de 16 bits o rótulo mentiria | [doc 19 §4](19_auditoria_precisao_por_ferramenta.md) |
| Promover 8→16 em `Asset::image_rgba16()` | não cria informação; deixaria o chamador a crer em degraus que não existem | [`asset.rs`](../../crates/ph2d-asset/src/asset.rs) |
| Bloom/iluminação como argumento para 16 bits | o `GameRt` e o `fx_stack` já são meio-float; ganho **nenhum** | §1.1 |
| `Rgba16Unorm` em vez de `Rgba16Float` | exige `TEXTURE_FORMAT_16BIT_NORM`, que é mascarada pelo adapter | §2 (M5) |
