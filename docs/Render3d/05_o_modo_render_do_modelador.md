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

A luz que sai, com o rig e o céu de omissão — verde médio dos pixels da peça · quantos têm o verde
em `255` · quantos são **branco chapado** (os três canais em `255`):

| exposição | `Standard` | `Neutral` |
|---|---|---|
| `−2` | `105,6` · `0` · `0` | `88,6` · `0` · `0` |
| `−1` | `145,7` · `118` · `114` | `134,8` · `0` · `0` |
| `0` | `197,9` · `6 729` · **`5 197`** | `188,3` · `0` · `0` |
| `+1` | `235,5` · `37 965` · `37 172` | `224,7` · `0` · `0` |
| `+2` | `252,7` · `53 243` · `52 436` | `243,0` · `0` · `0` |

⇒ **é a tabela que justifica a vista existir**: a `Standard` corta `10,9 %` da peça já na exposição
neutra — **`8,4 %` dela em branco chapado, sem forma nenhuma** — e a `Neutral` não satura um único
pixel em nenhuma exposição.

### §6.1 — ⛔⛔ Esta tabela foi RE-MEDIDA em 13/09, e a primeira media outro programa

A redacção anterior dizia `108,1 · 0` … `254,8 · 61 492`, e o corte a `0` stops era `7 %`. Os
números eram reais e **não eram do produto**: a sonda que os produziu vivia na `ph2d-field-render`,
que **não alcança** o [`render_light`](../../crates/ph2d-app-field3d/src/render_light.rs), e por
isso escrevia a luz **à mão** — uma lâmpada com a direcção em literal (nem sequer unitária:
`1,000466` contra `1,000003` do rotor) e um céu **CONSTANTE**, onde o produto tem um gradiente
(`0,670` no zénite a `0,126` no nadir, no canal azul).

⇒ a sonda mudou-se para [`render_light_tests`](../../crates/ph2d-app-field3d/src/render_light_tests.rs),
onde **chama as portas** (`lamps(&LightRig::default())` · `StudioSky` · `OpenPbr::default()`), e a
tabela acima é a saída dela. *Uma segunda cópia escrita à mão do valor que uma porta produz é a
forma canónica de um número envelhecer sem ninguém ver* (`CLAUDE.md` §5.0).

⚠️ **E a diferença não é cosmética:** a primeira tabela errava **no sentido optimista** em toda a
linha que decide alguma coisa (`+54 %` de pixels cortados a `0` stops, `−13 %` a `+2`).

## §7 — ⛔ O que esta fatia NÃO faz, e porquê

| ausência | razão |
|---|---|
| ~~**material por objecto**~~ | ✅ **FECHOU em 13/09** — ver §11; e a cor passou a escolher-se **vendo-a** em 14/09, §12 |
| **o rig do DOCUMENTO** | o rig vive dentro da cena da escultura (`ph2d_app_sculpt3d::cena`), fora do alcance do modelador; esta fatia usa o rig de omissão da `ph2d-light` |
| **anisotropia, transmissão, subsuperfície, fuzz, película fina** | cada uma é uma closure com gate próprio; com peso zero a composição gerada dá-lhes contribuição **zero**, e os campos **não existem** na struct (um campo que a lei não lê é um controlo morto) |
| **AgX** | ⛔ **licença**: não há neste disco um AgX cuja licença permissiva se leia no artefacto (`04` §3). O oráculo dele corre-se à mesma |
| **exposição contínua** | os chips dão `±2` stops, que é o que o smoke do plano pede; um slider de vista pede um alcance medido e um sítio no painel |
| **sombras, oclusão, luz indirecta a sério** | são as `W4` e `W5` do [plano](03_o_plano.md) |

## §8 — ⏳ O que fica aberto

- ⛔⛔ **A peça sai com `8,4 %` em BRANCO CHAPADO no olhar de omissão** (§6) — não é um realce, é um
  planalto sem forma. ⚠️ **É decisão do dono**, e o cálculo está fechado: na `Standard` a peça só
  deixa de cortar a `−1` stop (`118` px, `0,2 %`); a `Neutral` não corta em exposição nenhuma. As
  duas saídas são *a vista de omissão passa a `Neutral`* ou *a exposição de omissão desce um stop*.
- ⏳ **A radiância do céu é avaliada na direcção ESPELHADA**, e para um lóbulo largo a média do céu
  linear está na direcção média do lóbulo, não na espelhada. ⚠️⚠️ **A redacção anterior dizia que o
  erro «não foi medido» e que ele «é da forma do céu, não do material» — as duas metades estavam
  erradas.** Medido em 13/09 contra a média GGX numérica (`65 536` amostras, peso `N·L`), o erro é
  **do PAR**, e quem manda na magnitude é o `alpha` do material:

  | rugosidade | `N` para CIMA | `N` para BAIXO | rasante (`V` a 80°) |
  |---|---:|---:|---:|
  | `0,224` | `0,71 %` | `3,59 %` | `6,65 %` |
  | **`0,300`** (a de omissão) | `1,64 %` | **`7,89 %`** | `14,30 %` |
  | `0,500` | `5,69 %` | `22,25 %` | `35,09 %` |
  | `1,000` | `15,65 %` | **`41,84 %`** | `59,57 %` |

  ⭐ **No pixel, hoje, é ruído:** `p50 = 0`, `p95 = 1` byte. Num **METAL** a rugosidade `1` custa
  `p95 = 23` e `max = 33` bytes ⇒ *ela torna-se visível no dia em que houver material por objecto*.
  ⛔⛔ **E o oráculo NÃO a pode apanhar:** o gate do céu (§2) corre sobre um céu **constante**, e num
  céu constante a espelhada e a média do lóbulo são iguais **por construção**. *O oráculo desta lei
  é cego exactamente à aproximação que esta linha diz estar aberta.*
- ⚠️ **Uma mina para a `W3` (o rig de MUNDO): `l + v == 0` devolve `NaN`**, e o `sanitize` do olhar
  transforma-o em **PRETO em silêncio** — o pior modo de falha que existe, porque um pixel `NaN` e um
  pixel legitimamente preto leem-se iguais. Hoje é **inalcançável por geometria** (`to_light.z ≥
  sin(5°)` pelo `MIN_ELEV_DEG` do rig e `v.z > 0` em toda superfície visível ⇒ `(l+v).z ≥ 0,0872`;
  varredura de `31 417` normais: zero não-finitos). ⛔ **A cerca NÃO entra na
  [`ph2d-material`](../../crates/ph2d-material/)**: ela é o port fiel do GLSL, que tem a mesma
  propriedade, e «melhorar» uma fórmula ali deixa de ser a referência. *Quem der ao rig uma lâmpada
  atrás da câmera põe a cerca no CHAMADOR, e declara-a.*
- ⚠️ **O multi-scatter da difusa fica NEGATIVO acima de `base_color ≈ 6`** (polo em `5,981` com
  `base_diffuse_roughness = 1`; a `6,0` a indirecta devolve `[−676, −716, −813]`). Inalcançável nesta
  fatia (`base_color 0,8`, rugosidade difusa `0` ⇒ denominador `1`), e é a mesma propriedade do GLSL
  de referência. ⇒ **a porta que deixar autorar `base_color` coage a `0..1`.**
- ⚠️ **A `Surface::emission` é chamada por amostra e devolve zero** com o material de omissão — um
  `forward_facing`, um `dot`, um `clamp` e um `powf(5)` para somar `[0,0,0]`. Medido: `12,18 ns`
  contra `265,64 ns` do pixel completo ⇒ **`4,6 %`** do relógio de sombreamento. Cura: uma guarda
  `emission_luminance > 0` na porta.
- ⚠️ **As TRÊS lâmpadas de preenchimento do rig são IDÊNTICAS** (`[KEY, FILL, FILL, FILL]`, e as três
  `FILL` têm o mesmo azimute `50°` e a mesma elevação `25°`) — o doc do próprio `Light::FILL` escreve
  *«duas lâmpadas no mesmo lugar são uma lâmpada»*. **Pré-existente na
  [`ph2d-light`](../../crates/ph2d-light/), não desta fatia**, e o modo Render é só o primeiro
  consumidor que o torna visível em cor. Acendê-las multiplica a intensidade por `3` no mesmo sítio.
- ✅ **O MATERIAL POR OBJECTO foi MEDIDO (13/09) e a rota está escolhida** — esta linha dizia
  *«~`23 s` a `640×360`, e a via é a especialização por ladrilho»*, e as duas metades estavam erradas.
  O `23 s` saía de chamar o `surface_under` **por pixel**, que recompila a árvore a cada chamada (um
  JIT por raio); a pergunta real faz-se **uma vez, no ponto final**, e o ladrilho não é preciso.
  Medido em máquina calma (`load 1,8`), grelha de esferas a `640×360`, `~26 100` px de peça
  ([`pick_tests::measure_what_a_material_per_object_would_cost`](../../crates/ph2d-app-field3d/src/pick_tests.rs)):

  | folhas | donos, ingénuo | por pixel | donos, **com a bola à frente** | folhas visitadas |
  |---:|---:|---:|---:|---:|
  | `1` | `0,6 ms` | `24 ns` | `0,6 ms` | `1,0` |
  | `4` | `3,5 ms` | `132 ns` | `0,9 ms` | `1,0` |
  | `8` | `6,0 ms` | `258 ns` | `0,9 ms` | `1,0` |
  | `16` | `13,3 ms` | `507 ns` | **`1,6 ms`** | `1,0` |

  ⭐⭐ **A cura é a bola que a casa JÁ deriva** (`bounds::bounding_ball`): numa união o dono vale `~0`,
  logo o ponto está **sobre** a superfície da própria folha — quem não a contém não pode ganhar.
  `8,3×` com **a mesma resposta** (há assert no gate). O custo deixa de seguir a contagem de folhas.
  ⚠️⚠️ **E a MARGEM é obrigatória, não folga:** a 1.ª redacção da sonda testava `d² ≤ r²` e a rede
  disparou em **`26 216` de `26 216`** pixels — a marcha pára quando o campo desce abaixo de uma
  tolerância, isto é **ligeiramente fora** da superfície. *Um filtro exacto sobre um ponto que é
  aproximado por construção rejeita a resposta certa, em todo o lado.*
  ⚠️ **Duas coisas que a sonda NÃO mede:** formas **sobrepostas** (a grelha dela é separada, daí o
  `1,0`) e uma **mistura suave**, que empurra a superfície para fora das bolas das folhas — o
  `fold_children` escreve-o. Nos dois casos a rede existe e é contada, nunca escondida.
  ⭐ **E a 2.ª marcha (`pontos`, `5–6 ms`) é GRÁTIS:** o `march` já devolve o ponto de mundo e o
  `trace` **deita-o fora** (*«o ponto de mundo não interessa a um quadro inteiro»* — deixou de ser
  verdade no dia em que o material passou a ser por objecto). Guardá-lo custa `12 B` por pixel.
- ⛔⛔ **E isto reabre uma RECUSA MEDIDA, pela porta que o `CLAUDE.md` §0.0 nomeia.** O cabeçalho do
  [`pick`](../../crates/ph2d-app-field3d/src/pick.rs) recusa o *id-buffer* porque ele espalharia um
  segundo canal *«por cada pixel de cada quadro para responder a uma pergunta que só se faz num
  clique»*. ⇒ **material por objecto faz dela uma pergunta por pixel**, e a premissa dissolveu-se.
  ⚠️ A recusa continua a valer **contra o id-buffer**: a rota medida acima não arrasta canal nenhum
  pela marcha — ela resolve o dono **depois**, uma vez por pixel de peça.
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

## §10 — A auditoria de 13/09, e as duas curas que ela pagou

> Enio, 2026-09-13: *«funciona mas não totalmente. faça auditoria. e coloque a marca de seleção no
> render selecionado (ponto como nos menus do topo).»*

Duas lentes correram em paralelo — a **lei** (material · luz · cor) e a **costura** (ids → publicar
→ pintar → despachar → consumir). A lei está no §6.1 e no §8; a costura está aqui.

### §10.1 — ⛔⛔ As nove linhas abriam TODAS IGUAIS, e a verdade já estava publicada

O [`area_bar::entries`](../../crates/ph2d-panel-model3d/src/area_bar.rs) escreve `ButtonState::Pressed`
no chip aceso de cada fileira **desde que o pulldown existe**. Quem pintava o ponto era o
`id_is_currently_selected`, que responde por uma **tabela de ids que o `ph2d-editor-core` conhece** —
e as linhas de um pulldown de área são de outro módulo **por construção** (é a **D2**: aquela crate
não pode conhecer os ids de cada editor). A cadeia caía até `false`.

⇒ *o `Pressed` publicado não tinha LEITOR.* A cura é
[`menu_row_mark::contributed_row_is_current`](../../crates/ph2d-editor-core/src/screens/hero/menu_row_mark.rs):
uma linha **contribuída** acende pelo estado que o **módulo dono** publica.

⚠️ **A cerca *«esta linha veio do módulo?»* é obrigatória** — o `dispatch::pointer_down` escreve
`Pressed` num botão **enquanto o dedo está em baixo**, e sem ela a marca passaria a dizer *«é aqui
que estou a carregar»* em vez de *«é este o estado»*.

⭐ **E vale de graça para o pulldown *View***, que tinha exactamente o mesmo buraco: as seis vistas
nomeadas e os três gestos de câmera também abriam sem marca nenhuma.

### §10.2 — ⛔ O risco entre fileiras: o produtor não o emitia e o consumidor deitava-o fora

Três rádios independentes chegavam ao ecrã como **nove linhas numa coluna única**. O
`ToolRailEntry::Divider` **já existia** na casa — e o merge do menu fazia
`filter_map(|e| Some((e.node_id()?, e.label()?, None)))`, onde um divisor não tem nem id nem
rótulo ⇒ **todo divisor contribuído evaporava-se em silêncio**. *Um produtor sem consumidor e um
consumidor sem produtor dão o mesmo ecrã, e as curas são opostas.*

⇒ [`MenuRow`](../../crates/ph2d-editor-core/src/screens/hero/menu_row_mark.rs) (comando **ou** risco),
a altura do menu passa a **somar** em vez de multiplicar, e o `entries` emite o risco **só** entre
fileiras não-vazias (a lei *vazio ⇒ não é pintado*, nas duas pontas).

### §10.3 — Os gates, e porque são DOIS

| gate | onde | o que mata |
|---|---|---|
| `a_contributed_row_is_marked_by_the_state_its_module_publishes` | `ph2d-editor-core` | a **lei** — e a cerca do dedo em baixo é o 2.º `assert` |
| `the_mark_reaches_the_pixel_and_one_bullet_is_one_state` | `ph2d-panel-registry-init` | o **pixel** — conta segmentos de caminho com `0`, `1` e `3` fileiras acesas |
| `a_rule_between_the_rows_says_they_are_three_questions` | idem | o **risco** — a distância dentro da fileira contra a que atravessa a fronteira |

⭐⭐ **A prova de mutação mostrou que eles NÃO são redundantes:** apagar a chamada
`|| contributed_row_is_current(...)` no pintor deixa o gate da **lei VERDE** (a função está intacta)
e faz o gate do **pixel sangrar**. *Um gate que lê o `ButtonState` mede a publicação, não a pintura.*

### §10.4 — ⛔⛔ E TRÊS gates de ARQUITECTURA estavam VERMELHOS desde o commit da fatia

Nenhum deles vive nas crates que a fatia editou, e o fecho da wave correu só essas — é a cegueira
que o `CLAUDE.md` §5.1 já nomeia (*«um fecho que só corre as crates EDITADAS é cego aos gates de
arquitectura»*), agora pela quinta vez.

| vermelho | causa | cura |
|---|---|---|
| `as_citacoes_em_comentario_so_descem` | a `ph2d-material` trouxe **15** citações a `.glsl` do MaterialX, com a catraca em **zero** | triagem: o artefacto é **Apache-2.0** (`pacman -Qo` → `materialx 1.39.5-1.1`; o `LICENSE` abre com *«Apache License, Version 2.0»*) ⇒ os 11 ficheiros entram em `ALVO_PERMISSIVO`. *Atribuição é obrigação da licença, não dívida.* |
| `panel_functions_under_loc_cap` | `event.rs::apply_event` a `221` contra `200` | os **treze** braços copiados viraram a tabela `CHIP_ROWS` — a forma que este repo mediu como a única sem knob morto |
| `workspace_src_files_under_loc_cap` | `context_menu_overlay.rs` a `724` contra `700` | corte por responsabilidade: `menu_row_mark.rs` (*o que uma linha É, e quando está acesa*) sai do ficheiro que a DESENHA |

⛔ **Nenhuma isenção nova** — a catraca deste repo só desce.

### §10.5 — ⏳ O que a auditoria da costura deixou ABERTO (decisão de produto)

- **O menu FECHA a cada linha servida** ⇒ trocar modo + vista + exposição custa **abrir o pulldown
  três vezes**. A lei *«servir é fechar»* é genérica do store e está gateada — ⚠️ mas foi calibrada
  num pulldown de **uma** fileira (servir uma vista é um gesto terminal). Um pulldown de **três
  rádios** é composição nova, e ninguém reconferiu a nota.
- **A face fechada diz só o MODO** (`Shading / Matcap`): com o menu fechado não há confirmação da
  exposição nem da vista. O `AreaMenu` tem **um** campo `face`, e o chip mede `36 px` fixos.
- **A lei da W34 não cobre as três fileiras novas** — elas não dependem da selecção, logo caem fora
  do censo do [`reach_tests`](../../crates/ph2d-app-field3d/src/reach_tests.rs) **por construção, e
  não por decisão escrita**. O gate não reprova nem declara excepção.
- **Nenhuma das três tem tecla**, e o `Z` (o gesto do Blender para isto) está **livre** no roteador
  do modelador. ⚠️ O molde seguro existe (`input_pointer::mode_for_key` recusa `ctrl/alt/super`);
  sem ele, um `Z` come o `Ctrl+Z` do app inteiro.
- **O orçamento de chips da fila:** medido `3`, usam-se `2` ⇒ **cabe um terceiro**. ⚠️ A nota do
  `ids/chrome/rail.rs` ainda diz *«hoje usa-se 1»*, falsa desde esta fatia.

## §11 — **CADA OBJECTO COM O SEU MATERIAL** (13/09)

**O que o artista alcança:** escolhida uma forma, o painel `MODEL` ganha uma secção **Material** com
cinco números — *Base Color R/G/B*, *Roughness*, *Metalness*. Eles viajam no arquivo, sobrevivem ao
desfazer, e uma cópia sai com o material do original.

### §11.1 — ⭐⭐⭐ O painel não precisou de uma linha de código

As linhas deste painel são **derivadas** do [`params_of`](../../crates/ph2d-field-ecs/src/edit_params.rs):
a faixa, o despacho, a escrita e o cabeçalho de secção saem todos do [`ph2d_field::Param`]. ⇒ o
material entrou como uma variante nova (`Param::Material(u8)`) e a autoria apareceu inteira.

⛔ **A alternativa era uma superfície própria para o material** — uma segunda máquina de rows ao lado
de uma que já funciona. *A que apodrece é sempre a segunda.*

### §11.2 — As quatro peças, e onde cada uma vive

| peça | onde | o que ela decide |
|---|---|---|
| `Param::Material(u8)` | [`ph2d-field`](../../crates/ph2d-field/src/dims.rs) | que um material é um número autorado do nó, como uma largura |
| `FieldMaterial` | [`ph2d-field-ecs`](../../crates/ph2d-field-ecs/src/lib.rs) | componente **opcional** e registado: a ausência é *«o de omissão»*, e ele persiste |
| `owners::Owners` | [`ph2d-field-eval`](../../crates/ph2d-field-eval/src/owners.rs) | de quem é um ponto — a lei que o clique **já** usava |
| `materials::Table` | [`ph2d-app-field3d`](../../crates/ph2d-app-field3d/src/materials.rs) | a ponte: as folhas do mundo → uma superfície por folha, atravessando para a thread do traçado |

⚠️ **O `FIELD_DOC_VERSION` NÃO se mexe.** O documento é **geometria** — é ele que a marcha compila —
e uma cor não muda uma distância. ⇒ uma peça gravada antes desta wave abre, com o material de
omissão.

### §11.3 — ⭐⭐ A tabela tem DUAS metades, e o preço é a razão

| metade | depende de | refaz-se quando |
|---|---|---|
| a geometria (uma fita compilada por folha) | a forma | o documento muda |
| os números | o material | alguém lhes toca |

⚠️ **Compilar a fita de uma folha é um JIT**, e arrastar um slider de cor não muda geometria nenhuma.
Uma tabela só, refeita sempre que qualquer das duas mudasse, pagaria **um JIT por folha a cada
quadro** de um arrasto de cor.

⚠️ **E as duas largam o pedido guardado** (`Smoke::forget_requests`, agora com dois chamadores — o
olhar e o material): as duas mudam o que o traçado **pinta** sem mudar nada do que o cache compara.

### §11.4 — ⛔⛔ Os cinco vermelhos que os censos desta casa apanharam

Nenhum deles era previsível a partir do diff — os cinco foram **encontrados por gates que já
existiam**:

| censo | o que ele apanhou |
|---|---|
| `a_duplicate_carries_every_optional_component_of_a_node` | **defeito real**: duplicar uma peça de metal vermelho devolvia uma de omissão. O `copy_optional` não conhecia o componente novo, e a diferença só apareceria no modo *Render* — longe do gesto que a causou |
| `every_row_of_the_biggest_polygon_fits_the_registered_family` | o teto de linhas do painel — ver §11.5 |
| `one_more_vertex_would_not_fit` | a outra ponta do mesmo teto |
| `every_modifier_gets_its_own_section_in_the_panel` | a secção nova tinha de ser **declarada**, não tolerada |
| `every_node_shows_position_then_rotation_then_what_it_measures` | a **ordem**: o material vem depois das dimensões, e um material no meio delas partiria a secção da forma em duas |

### §11.5 — ⛔⛔ E um TETO EMPRESTADO caiu, como o da graduação da ponta antes dele

O `MAX_ROWS` do painel estava em **`64`**, e a nota dele dizia-o por escrito: *«o número escrito na
primeira vez … quando um documento real passar disto, o número muda com uma medição atrás»*. O
`MAX_POLYGON_VERTICES` (**27**) era **derivado** dele: `2 × 27 + 10 = 64`, exactamente no teto.

⇒ as cinco linhas do material empurravam o polígono para **24 vértices**. *Um teto de REGISTO cujo
recurso é memória a mandar num teto de FORMA é o caminho lento a definir o rápido* (`CLAUDE.md` §0.0).

⭐ **Medido:** cada linha regista `6` widgets (um slider, um campo numérico e os `MAX_CHOICES` botões
de escolha) ⇒ `64 → 69` custa **`30` widgets e 5 `String`**, uma vez, no arranque. O polígono fica
onde estava, e o `MAX_ROWS` passa a ser **derivado da maior forma**: `2 × 27 + 15`.

### §11.6 — ⏳ O que fica aberto

- ~~**A cor escolhe-se por TRÊS NÚMEROS**~~ — ✅ **PAGA em 14/09**, ver §12.
- **Uma silhueta entre duas peças de cores diferentes** recebe a cor de quem o centro do pixel
  apanhou (as quatro sub-amostras da borda não guardam ponto). É a mesma aproximação que a direcção
  de vista já faz, e está declarada no `shade_render`.
- ~~**O custo por quadro do `Owners`**~~ — ✅ **MEDIDO em 14/09, e não é o tecto.** Ver §14.
- **Um material num GRUPO** não existe: quem o traçado sabe nomear por pixel é a folha. Herdar do
  grupo é modelo novo.


---

## §12 — ⭐⭐⭐ A COR ESCOLHE-SE VENDO-A (2026-09-14)

> Enio: *«Em vez de 3 sliders de RGB, deveríamos ter uma caixa seletora de cor»*.

A §11.6 nomeava esta dívida por escrito. Está paga: os três canais dobram-se numa **amostra** que
abre o selector de cor da casa — roda OKLCH, canais RGB/HSV/OKLCH, hexadecimal, paletas e
conta-gotas.

### §12.1 — ⭐⭐ O selector NÃO foi construído: ele já existia, e entra-se nele por DUAS linhas

A casa tem **um** selector de cor (`BlenderColorPicker`, flutuante sobre o canvas) e um caminho
genérico de entrada no `pointer_down`: quem regista o id da sua amostra em
`WidgetStore::register_picker_swatch` e mantém `widget_color(id)` em dia ganha-o inteiro. É o mesmo
caminho do traço do Flip, do preenchimento do vetor e da tinta do Painter.

⛔ **Um segundo selector neste painel seria uma segunda resposta à mesma pergunta, e a que
envelhece.** O que esta wave escreveu foi a **linha-amostra** (`ParamRow::swatch`) e a travessia de
espaço de cor — não colorimetria, não widget, não janela.

### §12.2 — ⚠️ A travessia sRGB ↔ linear é UMA porta, e o ida-e-volta tem de fechar AO BIT

| lado | espaço | porquê |
|---|---|---|
| o documento (`FieldMaterial::base_color`) | **linear** | é o que o OpenPBR integra |
| o selector | **sRGB8** | é o que um humano escolhe |

A conversão é precisa em **dois** sítios distantes — a construção da linha (`scene_panel::param_rows`)
e o dreno do pedido (`scene_intents`). Escrita duas vezes seriam **duas curvas**, e o dia em que uma
ganhasse um `clamp` diferente a cor **derivaria a cada abertura do selector**. ⇒ uma porta:
`materials::base_color_srgb8` e o par dela.

⛔⛔ **E o ida-e-volta ser exacto não é uma curiosidade de colorimetria:** o painel pergunta *«a cor
mudou?»* comparando **bytes**. Um único byte que não voltasse ao mesmo valor faria essa comparação
ser sempre verdadeira naquela cor ⇒ **um pedido de edição por quadro** enquanto o selector estivesse
aberto, e um passo de desfazer por cada largar de botão. Gate sobre os **256** bytes, pelo caminho do
produto (`f32` do componente incluído): `the_round_trip_through_the_document_is_exact`.

⚠️ **E a comparação é em sRGB8 e não em linear**, pela razão que o `motion_bridge_color` já tinha
escrito: comparar os lineares faria **abrir** o selector sobre uma cor que não é um ida-e-volta
exacto de 8 bits **gravar uma quantização** — uma edição que o artista não fez, embrulhada num passo
de undo.

### §12.3 — ⛔⛔ O id da amostra é o ÚNICO deste painel cunhado pela ENTIDADE, e a excepção é a lei

Todos os outros ids do painel vêm da **posição da linha** (o `populate` cunha-os às cegas, antes de a
peça existir). Este não precisa de registo nenhum — quem o reconhece é o `is_picker_swatch`,
preenchido ao pintar — e por isso **pode** carregar o sujeito. E tem de carregar:

O selector sobrevive a uma mudança de selecção, e **escolher outra forma no canvas não o fecha** (o
módulo 3D toma aquele clique antes de o `pointer_down` do chrome correr). Com um id da posição, a
amostra do objecto novo teria o **mesmo** id, o painel leria o selector como *«aberto em mim»* e
escreveria a cor do objecto **anterior** na forma acabada de escolher — em silêncio. Com o id da
entidade isso é **inexprimível**.

*É o mesmo defeito que o `card_swatch_id` do Motion documenta — ali vinte cartões a partilhar o id do
param; aqui duas selecções a partilhar o id da linha.*

### §12.4 — ⭐ E o selector ÓRFÃO fecha-se

`paint::close_a_stranded_picker`, com **dois** leitores (o caminho normal e a saída antecipada do
painel fechado). Sem ela, trocar de selecção — ou fechar o painel — deixaria o selector aberto sobre
uma amostra que **já não é pintada por ninguém**: a roda move-se, a cor muda no selector, e nada no
documento a recebe. *É a espécie de controlo morto que o `CLAUDE.md` §5.0 chama de «o consumidor que
projecta o valor fora», e nenhuma sonda de registo a vê — porque ele **é** registado.*

⚠️ **Ela só fecha o que é NOSSO**: a comparação é com a amostra que este painel pintou da última vez
(`state::remember_swatch`), nunca *«há um selector aberto»* — fechar o do Inspector ou o do Painter
seria este painel a mandar na superfície partilhada de outro.

### §12.5 — Os gates, e as sete mutações que sangraram

| gate | o que ele prende |
|---|---|
| `a_colour_is_one_row_and_it_carries_the_swatch` | **uma** linha, com a cor do documento em sRGB8, e ela **segue** o documento |
| `the_colour_intent_reaches_the_document` | os três canais escritos, no mesmo quadro ⇒ **um** passo de desfazer |
| `the_round_trip_through_the_document_is_exact` | os 256 bytes fecham — §12.2 |
| `clicking_the_swatch_opens_the_house_colour_picker` | o gesto REAL abre o selector, **semeado com a cor da peça** |
| `what_the_picker_writes_becomes_one_edit_and_only_when_it_changed` | a leitura chega ao documento **e pára** quando ele a apanha |
| `choosing_another_shape_does_not_paint_it_with_the_previous_colour` | §12.3 e §12.4, nas duas metades |
| `a_colour_row_paints_no_slider_to_grab` | a amostra **substitui** o controlo de número; ele escreveria só o vermelho |

⚠️ **Clicar numa amostra não emite `WidgetEvent` nenhum** — o `pointer_down` genérico abre o selector
e devolve. ⇒ um gate que contasse eventos leria a amostra como **muda**, e um que só medisse o rect
leria como **viva** uma que ninguém registou. *As duas leituras erram, e em sentidos opostos*; o que
as separa é o estado que o clique deixa (`picker_target`).

⭐ **Sete mutações, sete sangrias** — entre elas *semear também com o selector aberto* (a cor deixa de
sair do selector) e *cunhar o id pela posição* (§12.3).

### §12.6 — ⚠️ O que MUDOU de sítio, e o que NÃO mudou

- ⛔ **O `params_of` continua a oferecer os CINCO números do material.** A dobra é da
  **apresentação**: a porta de escrita `Param::Material(0..2)` continua a existir e continua gateada,
  e é por ela que esta cura escreve. *Colapsar na fonte tornaria a cor inalcançável por toda a
  maquinaria que já a alcança* — e há **dois** censos a medir os dois lados (o do painel lê `3`, o do
  `params_of` lê `5`; se os dois lerem `5`, alguém desfez a dobra).
- ⚠️ **As chaves `field.dim.base_r/g/b` FICAM.** Elas rotulam os params, que continuam a existir.
  *Apagar a etiqueta de um número porque um painel deixou de o pintar é deixá-lo sem nome no dia em
  que outra vista o mostrar.*
- ⚠️ **Um corte de LOC pago no caminho:** a linha nova pôs o `paint` do painel em `210` de `200` ⇒ as
  fileiras de chips do corpo saíram para `paint_chip_rows`. ⛔ *Corte, nunca uma entrada no
  `FN_OVERAGE_OK`.*
- ⭐ **Um método novo no arnês partilhado** (`MockPanelHost::pick_colour_in_the_open_picker`): ele
  encena o espelho que o `hero::paint` corre, e **entra em pânico sem um selector aberto** — nunca um
  no-op silencioso. É isso que o impede de ser o `store_mut()` que aquele ficheiro recusa por escrito.

### §12.7 — ⏳ O que fica aberto

- **O material não tem ALFA**, e a amostra pinta-se opaca. Carregar um alfa que o documento não
  guarda seria prometer uma transparência que nada honra.
- **Um material num GRUPO** continua a não existir (§11.6) — quem o traçado sabe nomear por pixel é a
  folha.
- **O selector abre no canto do canvas**, e não junto da amostra. É onde a casa o põe para todos os
  painéis; movê-lo é decisão da UI, não deste módulo.


---

## §13 — ⭐⭐ A LEI DO ALCANCE CHEGA ÀS TRÊS FILEIRAS NOVAS (2026-09-14)

O §8 nomeava este buraco: a lei-mãe da **W34** (*«o painel oferece exactamente o que o gesto faz»*)
não cobria o **sombreamento**, o **olhar** e a **exposição**. Cobre agora —
[`reach_shading_tests.rs`](../../crates/ph2d-app-field3d/src/reach_shading_tests.rs).

### §13.1 — ⚠️ Porque não foi uma linha na tabela que já existia

A W34 mede *«a intenção muda o DOCUMENTO»*, e só varre fileiras que **dependem da selecção**. Estas
três não são nem uma coisa nem outra: elas são **estado de VISTA** — não entram no undo, não viajam
no ficheiro, e não precisam de nada escolhido. ⇒ medi-las por aquela régua daria as três **mudas**, e
a conclusão errada seria **apagar os botões**.

⭐ É a mesma partição que o irmão [`reach_camera_tests.rs`](../../crates/ph2d-app-field3d/src/reach_camera_tests.rs)
já tinha pago, com a frase dele: *uma lei de alcançabilidade tem uma régua por espécie de gesto*. A
régua aqui é o **estado do viewport e da cena**.

### §13.2 — ⛔⛔ E a metade que quase ninguém escreve: o PEDIDO GUARDADO

O traçado é caro, então o módulo guarda o pedido que serviu e reaproveita-o enquanto nada mudar. O
laço compara **a câmera, o tamanho e o documento** — e **um olhar novo não move nenhum dos três**. Sem
um `forget_requests` explícito, o quadro seguinte devolve o desenho antigo: o chip acende, o estado
muda, e **a peça não muda de aparência**.

⚠️ O artista lê isso como *«o botão não faz nada»*, e **as três leis de estado ficam todas verdes por
cima**. É a espécie de morto que o `CLAUDE.md` §5.0 chama de *«o consumidor que projecta o valor
fora»*: o fio está inteiro, o valor chega, e quem o recebe descarta-o — e **nenhuma sonda de registo
o vê**, porque o chip *é* registado.

⇒ o gate `changing_how_it_is_painted_drops_the_frame_that_was_already_traced` **encena um quadro já
servido** antes de cada chip. ⚠️ Sem essa encenação ele mediria um campo que já era `None` — *e
passaria a afirmar nada*, que é a forma de gate vazio que este repo já pagou várias vezes.

### §13.3 — ⭐⭐ A CRUZ: as duas fileiras escrevem o MESMO `Look`

A vista e a exposição são **campos da mesma struct**, e cada fileira escreve um só. Um chip de
exposição que reescrevesse a vista (ou o contrário) apagaria a escolha do vizinho **em silêncio**.

⇒ o gate **cruza** as duas: escolhe uma vista fora da omissão e só depois varre as cinco exposições,
afirmando a cada passo que a vista continua onde ficou. A mutação `with_exposure → with_view` sangra
exactamente aí.

### §13.4 — ⚠️ E a ordem de varredura tem uma razão

As fileiras são varridas com **o modo de omissão por ÚLTIMO**. Um despacho com um `if != actual { … }`
a mais deixaria o chip do estado em vigor mudo, e um gate que só pedisse *o modo que não é o de
partida* passaria por cima disso.

### §13.5 — Os gates, e as quatro mutações que sangraram

| gate | o que ele prende |
|---|---|
| `every_shading_chip_paints_the_piece_the_way_it_says` | o chip põe o viewport no modo que nomeia, **e o aceso segue** |
| `every_look_chip_changes_the_scene_look_and_leaves_its_neighbour_alone` | §13.3, com a cruz |
| `changing_how_it_is_painted_drops_the_frame_that_was_already_traced` | §13.2, nos **três** chips e em **todos** os viewports |

⭐ **4/4 sangraram:** `SetShading` a ignorar o slot · `SetExposure` a escrever a vista · `set_shading`
sem largar o pedido · `set_look` sem largar os pedidos.

⚠️ **O pedido é largado em TODOS os viewports** no caso do olhar, e só no activo no caso do
sombreamento — porque o olhar é **da cena** e o sombreamento é **do viewport** (§12 da tabela deste
doc, e o doc do `crate::shading`). O gate afirma a versão forte (nenhum viewport fica com pedido),
que é a que vale para os três.


---

## §14 — ⏱️ O CUSTO DE CONSTRUIR A TABELA, MEDIDO (2026-09-14)

A §11.6 deixou isto escrito como *«a primeira medição da wave seguinte»*: a sonda de 13/09 mediu a
**RESOLUÇÃO** (*«de quem é este pixel?»*, `1,6 ms` a 16 folhas) com as fitas **já compiladas**, e o
doc dela dizia-o por escrito. O que faltava medir era a **CONSTRUÇÃO** — um JIT por folha, que corre
a cada mudança de geometria, isto é, **a cada quadro de um arrasto do gizmo**.

### §14.1 — A tabela

Sonda [`measure_what_building_the_table_costs_per_frame`](../../crates/ph2d-app-field3d/src/materials_tests.rs),
grelha de esferas em união, **mínimo de 7 corridas com a mediana ao lado**:

| folhas | `Table::build` MIN | p50 | `Owners::new` | `leaves()` | % de um quadro de 16,7 ms |
|---:|---:|---:|---:|---:|---:|
| `1` | `0,000` | `0,000` | `0,008` | `0,000` | `0,00 %` |
| `2` | `0,017` | `0,017` | `0,015` | `0,000` | `0,10 %` |
| `4` | `0,032` | `0,032` | `0,030` | `0,001` | `0,19 %` |
| `8` | `0,061` | `0,062` | `0,061` | `0,001` | `0,37 %` |
| `16` | `0,122` | `0,125` | `0,120` | `0,003` | `0,73 %` |
| `32` | `0,243` | `0,248` | `0,240` | `0,005` | `1,46 %` |
| `64` | **`0,479`** | `0,486` | `0,472` | `0,010` | **`2,87 %`** |

⭐ **Linear, a `7,5 µs` por folha.** ⇒ o custo chega a `10 %` de um quadro por volta das **`223`
folhas**, e a um quadro inteiro por volta das **`2 230`**. Nenhuma peça deste módulo se aproxima
disso — e o quadro de **movimento** já custa `26,7 ms` (§8), de que a marcha é `80 %`.

⇒ **a construção não é o tecto, e não há cura a fazer.** *Medir antes de limitar, e não optimizar o
que não é o tecto* (`CLAUDE.md` §0.0).

⚠️ **`98 %` do custo é o `Owners::new`** (a compilação das fitas); o `leaves()` — a travessia do
mundo mais um `FieldDoc` por folha — é `2 %`. Quem quiser mexer neste número mexe no JIT, não na
travessia.

⚠️⚠️ **A medição foi feita com a máquina a `load 45`, e vale mesmo assim** — o mínimo e a mediana de
7 corridas concordam a menos de `2 %`. *A carga de fundo desta workstation nunca desce abaixo de
`~7`, e «esperar pela calma» não chega: mede-se o MÍNIMO de N, com a mediana ao lado.*

### §14.2 — ⭐⭐ A metade que a medição mostrou já estar certa: os dois ritmos

O `sync` chama `Table::build` **só quando o documento muda**, e mudar uma COR **não muda o
documento** (o material é um componente à parte; o `FieldDoc` cozido é a geometria). ⇒

| gesto | o que corre | custo a 64 folhas |
|---|---|---|
| arrastar o **gizmo** | `Table::build` (o JIT) | `0,479 ms` |
| arrastar a **cor** | `refresh_authored` | `0,010 ms` |

**`48×` mais barato**, e é essa a razão de a tabela ter duas metades.

### §14.3 — ⛔⛔ E o gate que defendia isso media a RESPOSTA

O `changing_a_number_refreshes_the_surfaces_and_not_the_geometry` afirma que o dono **responde o
mesmo** depois de um número mudar. Isso sai certo **mesmo que alguém troque o `refresh_authored` por
um `Table::build` inteiro** — a geometria é a mesma, logo o dono responde o mesmo, e o quadro passa a
pagar um JIT por folha a cada pixel de arrasto do selector de cor.

*É a mesma lei que o `Owners::at_counting` já existia para servir:* **um gate sobre a RESPOSTA é cego
ao PREÇO**. ⇒ `dragging_a_colour_compiles_no_tape_at_all`, sobre o **trabalho**.

⭐⭐ **E a régua não existia — foi escrita nesta wave.** O contador da casa (`hybrid::FLOAT_TAPES`,
W70) conta a fita do **traçado**; a que a `Owners` compila é a do **PONTO** (`Field::new →
PointTape::build`), e não tinha contador nenhum. A 1.ª redacção do gate leu o contador errado e mediu
**zero de zero** — ⚠️ **quem a apanhou foi o PISO**, que afirma que o lado caro compila pelo menos uma
fita por folha. *Uma régua que lê zero nos dois lados é verde e não afirma nada.*

⇒ [`ph2d_field_eval::POINT_TAPES`], o gémeo, com o mesmo `#[doc(hidden)]` e a mesma frase do irmão:
*um custo que nenhuma sonda conta é um custo que nenhuma mutação mata*.

⚠️ **Ele corre por `nextest`** — o §9 deste doc mede o que acontece a um contador global sob `cargo
test`, em que as threads se vêem umas às outras.
