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
| **material por objecto** | o material desta fatia é o **padrão da nodedef**. Por objecto precisa de saber *qual folha* cada pixel tocou, e o custo disso é uma medição por fazer (§8) |
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
