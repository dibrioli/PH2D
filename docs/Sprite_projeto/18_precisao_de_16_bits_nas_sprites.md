# 18 — Precisão de 16 bits nas sprites (o par `Format` volta, agora COM modelo)

> **Estado:** plano aberto. W1 em construção.
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
3. **O defeito visual real** (anéis/faixas num degradê limpo) está na **descida final sem dither** —
   `git grep -i dither` sobre `crates/*/src` + `shells/desktop/src` devolve **um** hit, e é um
   rótulo de painel de smoke. Não há dither em lugar nenhum do produto.

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

### W2 — A GPU (⚠️ abre com o gate de M5-bis) · **a wave grande**
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

### W3 — Persistência
- `PROJECT_SCHEMA` +1 · `SHEET_DOC_VERSION` +1 (⚠️ ambos são números que **somam entre linhas**:
  contam-se lendo o código, `CLAUDE.md` §5.0 — e o `project_schema.rs` tem **três** sítios).
- Carregar um projeto antigo dá 8 bits, que é o que ele era.

### W4 — As ferramentas
- A regra do §3.4, num ponto de entrega só, com o aviso.

### W5 — A UI (o que o Enio pediu ver)
- O par `Format` volta ao Inspector — agora **com arm de dispatch**, com o aceso lido do **estado**,
  e mostrando o formato **real** (incluindo `GPU compressed` para cozidas, que é o que o §5 do doc 17
  corrigiu e **não** se perde aqui).
- Botão de converter, com o aviso do §3.3 (muda a estratégia) e do §3.2 (16→8 perde).
- **O dither na descida** — o defeito real do §1.3, que só aqui tem dono.

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
