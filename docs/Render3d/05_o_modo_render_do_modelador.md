# 05 — A 1.ª fatia: **o modo RENDER do modelador** (13/09)

> A `W1` (gestão de cor) e a metade mínima da `W2`/`W3` (material e céu), **juntas** — porque
> medidas contra a árvore ([`04`](04_a_remedicao_contra_a_arvore.md) §1.1) elas não se mostram
> separadas: o modelador pinta **matcap**, que é uma fotografia de luz, e uma exposição sobre uma
> fotografia é cosmética.

**O que o artista alcança:** no pulldown **Shading** da área, o viewport passa de `Matcap` a
`Render` — a peça com material, as lâmpadas do rig e o céu —, e a cena ganha **exposição**
(`−2`…`+2` stops) e **vista** (`Standard` · `Neutral`). ⚠️ **A omissão é o quadro de sempre**, byte a
byte: matcap, `0` stops, `Standard`.

## §1 — As três peças novas, e por que são três

| crate | o que é | por que nasce fora do que já existia |
|---|---|---|
| [`ph2d-view-transform`](../../crates/ph2d-view-transform/) | exposição (`2^stops`) + vista (`Standard`, `Neutral`) → luz de ecrã em `0..=1` | o tonemap do `ph2d-render` corre sobre o `game_rt` **partilhado com a arte 2D** e tem de a deixar passar byte a byte; a vista é da **cena 3D** (o modelo do Godot) |
| [`ph2d-material`](../../crates/ph2d-material/) | o **OpenPBR** como lei de referência em CPU — port Apache-2.0 do GLSL do MaterialX 1.39.5 | o modelador traça e sombreia na **CPU**; o WGSL gerado (`04` §2) é o gémeo de GPU, e nasce com gate de paridade contra esta |
| `ph2d-field-render::shade_render` | a costura: G-buffer + câmera + material + luz + olhar → pixels | o traçador é o único que sabe o que é uma cor, e já o dizia por escrito |

⛔ **Zero dependências de produção** nas duas crates novas: é o que permite ao traçador de CPU
puxá-las sem arrastar o `wgpu`, e ao futuro passe de GPU responder a MESMA lei.

## §2 — O oráculo, e a barra de cada gate

Tudo corrido **sem interface**, sobre entradas nossas (`CLAUDE.md` §0.9). Os instrumentos vivem em
[`ferramentas/`](ferramentas/).

| gate | oráculo | amostras | pior medido | barra |
|---|---|---:|---:|---:|
| a vista `Neutral` | OpenColorIO 2.5.1 sobre o `config.ocio` do Blender 5.2, **nos nós do LUT** | 457 | `8,9e-6` | `2e-5` |
| a luz **directa** do material | `GlslRenderer` do MaterialX a desenhar a `sphere.obj` dele | 945 (7 materiais × 3 luzes) | `9,8e-6` | `1e-4` |
| a luz do **céu** | o mesmo, método `PREFILTER` sobre um céu constante | 315 | `1,6e-6` | `1e-4` |
| o *furnace test* | a propriedade (branco sob céu branco devolve branco) | 440 células | `1,19e-7` | `1e-5` |

⚠️ **A barra da `Neutral` mora nos NÓS**: fora deles o próprio oráculo erra até `0,030` (a
interpolação do LUT dele), contra `8,9e-6` nos nós — *quando o oráculo é um LUT, o sítio onde ele é
exacto é o sítio onde se mede*.

⚠️ **As duas metades**: em cada gate de oráculo a lei ERRADA sobre a MESMA fixture tem de ficar muito
acima da barra (a `Standard` contra a `Neutral`; o material de omissão contra o material certo, que
dá `595`).

### §2.1 — ⚠️ Uma perda de precisão que só o oráculo apanhou

A NDF do GGX usa **`|H × N|²`** e não `1 − (H·N)²`. São iguais em `f64` e não em `f32`: a subtracção
cancela onde o lóbulo é estreito, e no **espelho** da fixture (rugosidade `0,02`) ela afastava a luz
directa `1,12e-4` do oráculo — acima da barra — enquanto a réplica em `f64` ficava a `7,7e-6`. O
GLSL projecta `H` no plano da tangente; o produto vectorial **é** essa projecção.

## §3 — As duas traduções da luz, e o preço de cada uma

O rig continua a ser o da `ph2d-light` (⛔ nenhum rig novo — *«a mesma lâmpada acende a tinta ao
lado»*), e o céu continua a ser o `env_ambient` dela. O [`render_light`](../../crates/ph2d-app-field3d/src/render_light.rs) traduz:

1. **o sinal de `y`** — o rig é autorado no canvas (`y` para baixo) e o G-buffer está em vista (`y`
   para cima). ⚠️ A casa já pagou este sinal uma vez entre a tinta e a escultura; aqui há gate, e ele
   afirma o que o rig **promete por escrito** (a chave de omissão ilumina de cima e da esquerda);
2. **o `π`** — o rig promete *«um plano de frente devolve `1`»* e o MaterialX recebe **radiância**,
   logo `radiância = π × intensidade × cor`.

⭐ **E a ponte entre as duas perguntas do céu tem gate por INTEGRAÇÃO**: a irradiância é a média
cosseno da radiância sobre o hemisfério (`1e-3` em quatro normais). É ela que impede a difusa e o
especular do mesmo céu de acenderem a peça com duas luzes diferentes.

## §4 — Onde o estado vive (e por que em dois sítios)

| o quê | dono | como no Blender |
|---|---|---|
| `Matcap`/`Render` | **o viewport** | *Viewport Shading* — numa divisão, as vistas nomeadas medem em matcap e a do artista vê o material |
| exposição e vista | **a cena** | *Color Management* — duas vistas com exposições diferentes seriam duas cenas |

Os dois são **VISTA** (`04` §4): sobrevivem a fechar o painel ([`View`](../../crates/ph2d-app-field3d/src/view.rs)) e não entram no undo nem no arquivo.

⚠️ **E mudá-los larga o pedido guardado** (`Smoke::set_shading` · `set_look`): sem isso o laço do
preview responde *«nada mudou»* — a câmera, o tamanho e o documento são os mesmos — e o quadro fica
no modo antigo até alguém tocar na peça. *É o congelador que o doc do `requested` já descrevia.*

## §5 — A costura da UI, e a lei que ela honra

O **2.º pulldown da área** (*Shading*), com três fileiras: o modo · as vistas · as exposições. As
sete pontas foram fiadas juntas (ids · `populate` · `paint`/índice de acerto · `event` · intent ·
retrato · consumidor), e o gate do painel que conta as famílias (`CHIP_FAMILY_COUNT`) passou a
conhecer as três novas.

⚠️ **Ele só é publicado quando tem o que oferecer** — a mesma lei das fileiras do painel (*vazio ⇒
não é pintada*) —, e é isso que deixa os três gates do orçamento da fila intactos sobre a fixtura
deles. O pulldown novo tem gate próprio, **com o dedo** (`Down`+`Up` pelo despacho real), e ele mede
também a fila: **`2` chips de `3` do orçamento, e uma linha só nos três tablets**.

## §6 — As medições

Esfera a `640×360`, `61 804` pixels de peça, `--release`. ⚠️ **Com a máquina a `load ~9`**, logo são
ordens de grandeza e não um relógio (`CLAUDE.md` §5.0) — a medição fina vai com o relógio adiado.

| | ms |
|---|---:|
| traçado | `3,3`–`7,8` |
| sombrear matcap | `2,2`–`3,0` |
| sombrear render | `1,7`–`2,9` |

⭐ **O render custa o mesmo que o matcap, e o achado é do outro lado:** ele corre as linhas em
**paralelo** e o matcap corria em série desde que existe. O matcap passou a correr igual — bytes
idênticos, porque a lei é por pixel (há gate).

A luz que sai, com o rig e o céu de omissão (verde médio dos pixels da peça · quantos saturam):

| exposição | `Standard` | `Neutral` |
|---|---|---|
| `−2` | `108,1` · `0` | `92,1` · `0` |
| `−1` | `148,9` · `115` | `138,9` · `0` |
| `0` | `202,7` · `4 364` | `194,0` · `0` |
| `+1` | `242,0` · `39 890` | `231,2` · `0` |
| `+2` | `254,8` · `61 492` | `243,9` · `0` |

⇒ **é a tabela que justifica a vista existir**: a `Standard` corta `7 %` da peça já na exposição
neutra (o realce especular), e a `Neutral` não satura um único pixel em nenhuma exposição.

## §7 — ⛔ O que esta fatia NÃO faz, e porquê

| ausência | razão |
|---|---|
| **material por objecto** | o material desta fatia é o **padrão da nodedef**. Por objecto precisa de saber *qual folha* cada pixel tocou, e o custo disso é uma medição por fazer (§8) |
| **o rig do DOCUMENTO** | o rig vive dentro da cena da escultura (`ph2d_app_sculpt3d::cena`), fora do alcance do modelador; esta fatia usa o rig de omissão da `ph2d-light` |
| **anisotropia, transmissão, subsuperfície, fuzz, película fina** | cada uma é uma closure com gate próprio; com peso zero a composição gerada dá-lhes contribuição **zero**, e os campos **não existem** na struct (um campo que a lei não lê é um controlo morto) |
| **AgX** | ⛔ **licença**: não há neste disco um AgX cuja licença permissiva se leia no artefacto (`04` §3). O oráculo dele corre-se à mesma |
| **exposição contínua** | os chips dão `±2` stops, que é o que o smoke do plano pede; um slider de vista pede um alcance medido e um sítio no painel |
| **sombras, oclusão, luz indirecta a sério** | são as `W4` e `W5` do [plano](03_o_plano.md) |

## §8 — ⏳ O que fica aberto

- ⏳ **A radiância do céu é avaliada na direcção ESPELHADA**, e para um lóbulo largo a média do céu
  linear está na direcção média do lóbulo, não na espelhada. O erro **não foi medido**; ele é da
  forma do céu (`ENV_SLOPE`), não do material.
- ⏳ **O material por objecto** pede o nó por pixel. O `surface_under` custa `0,10 ms` por raio
  (é uma folha de menor módulo), o que a `640×360` seriam ~`23 s` — ⇒ a via é a especialização por
  ladrilho que o traçador já tem, e ela é uma medição por fazer.
- ⏳ **O relógio fino** do custo do sombreamento (a máquina esteve a `load ~9`).
- ⏳ **O gémeo em WGSL** do material (o gerado pelo MaterialX) com gate de paridade contra a
  `ph2d-material` — é o que leva esta lei à escultura e ao runtime.

## §9 — ⚠️ Uma armadilha do CORREDOR, medida aqui

Os gates de contagem do `ph2d-field-render` (fitas, amostras, acertos de cache) leem **contadores
globais**. Sob `cargo test` — threads no MESMO processo — eles veem-se uns aos outros: **8
reprovações**, uma delas um **pânico** no produto (`spent() - before` a ficar negativo). Sob
`nextest`, que dá **um processo por teste**, os mesmos 63 passam.

⇒ a subtracção passou a **saturar** (um contador que outro pode repor não é uma diferença), e a
regra fica escrita: *os gates de contagem desta crate correm por `nextest`* — o
`scripts/cargo-test-narrow.sh` é o corredor errado para eles.
