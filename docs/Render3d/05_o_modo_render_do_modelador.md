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
- ~~**A radiância do céu é avaliada na direcção ESPELHADA**~~ — ✅ **CURADA em 14/09, ver §16.**
  O diagnóstico abaixo fica, porque é ele que explica a cura: para um lóbulo largo a média do céu
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
- ~~**Uma silhueta entre duas peças de cores diferentes**~~ — ⭐ **a fronteira INTERIOR foi curada em
  14/09 (§19)**; a da silhueta continua a ser a do centro do pixel, e está declarada no
  `shade_render`.
- ~~**O custo por quadro do `Owners`**~~ — ✅ **MEDIDO em 14/09, e não é o tecto.** Ver §14.
- ~~**Um material num GRUPO** não existe~~ — ✅ **EXISTE desde 14/09, e sem modelo novo: ver §18.**


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


---

## §15 — ⭐⭐⭐ O BRANCO CHAPADO: a pergunta do dono tinha uma METADE POR MEDIR (2026-09-14)

O §8 leva esta desde 13/09: *«a peça sai com `8,4 %` em BRANCO CHAPADO no olhar de omissão»*, com
duas saídas escritas — *a vista passa a `Neutral`* ou *a exposição desce um stop*. ⚠️ **As duas notas
descreviam só o que se GANHA.** O que se paga não tinha sido medido.

### §15.1 — ⛔ Primeiro, uma hipótese REFUTADA por leitura

O §8 nomeia, logo abaixo, que *«as TRÊS lâmpadas de preenchimento do rig são IDÊNTICAS»* — o que
sugeria que o estouro fosse o rig a somar três vezes a mesma luz. **Não é:** o
[`Light::FILL`](../../crates/ph2d-light/src/lib.rs) tem **`on: false`**. O rig de omissão é **uma
lâmpada**, e a nota das três continua verdadeira e continua a ser sobre o que acontece a quem as
ACENDE. *Uma hipótese barata, refutada a ler o ficheiro — antes de medir o que quer que seja.*

### §15.2 — ⚠️ A metade que faltava: o olhar governa TAMBÉM o matcap

O `Look` é **da cena** — e o doc do [`shade_with`](../../crates/ph2d-field-render/src/shade.rs)
escreve-o: *«o olhar vale também para o matcap, como no Blender»*. ⇒ **trocar a omissão não afina o
modo novo: repinta o que o modelador sempre mostrou.**

⚠️ E a razão de a omissão ser `Standard` está escrita no `ph2d-view-transform`: *«com exposição `0` e
luz dentro de `0..=1` ela devolve a entrada»* — isto é, **ela foi escolhida pelo matcap**, que é uma
fotografia em `0..=1`.

### §15.3 — A medição, no asset da casa (`749²`, `561 001` texels)

Sonda [`measure_what_neutral_would_do_to_the_matcap`](../../crates/ph2d-app-field3d/src/render_light_tests.rs):

| | texels que mudam | `|Δ|` p50 | p95 | pior |
|---|---:|---:|---:|---:|
| **`Neutral`** | `99,9 %` | **`13`** bytes | `29` | `−31` |
| **`−1` stop** | `100,0 %` | **`46`** bytes | `55` | `−56` |

⭐⭐ **E o achado que inverte a intuição: `0,0 %` dos texels estão acima do joelho (`0,76`) e ainda
assim `99,9 %` mudam.** A *Khronos PBR Neutral* **não é a identidade em lado nenhum** — ela subtrai um
offset a **todas** as cores antes de comprimir (a dessaturação do algoritmo), e não só às que passam
do joelho. ⚠️ *A leitura «abaixo do joelho ela não toca» é falsa, e era a premissa com que esta linha
teria escolhido.*

⇒ **o `−1` stop é `3,5×` pior para o matcap** (`46` contra `13` bytes de desvio típico) e ainda deixa
`0,2 %` a cortar. Entre as duas saídas escritas, a `Neutral` é a barata — e a nota do §8, que as dava
como equivalentes, estava a esconder um factor de três e meio.

### §15.4 — ⏳ E há uma TERCEIRA saída que o §8 não listava

**Não mexer na omissão.** O estouro é do modo **Render**, e o modo de omissão é o **Matcap** — onde a
`Standard` é a identidade por construção. O pulldown já oferece as duas vistas e os cinco stops.

| saída | o modo **Render** | o **Matcap** de hoje |
|---|---|---|
| **não mexer** | corta `8,4 %` | **intocado, ao bit** |
| **`Neutral`** | não corta em exposição nenhuma | muda `99,9 %` dos texels, `13` bytes típicos |
| **`−1` stop** | corta `0,2 %` | muda `100 %`, `46` bytes típicos |

⭐ **O precedente da indústria é a `Neutral`:** o Blender trocou a omissão de `Standard` para uma
vista com ombro (Filmic, depois AgX) exactamente por isto, e aplica a *Color Management* dele também
ao modo sólido.

### §15.5 — ✅ A DECISÃO: `Neutral`, por ordem do dono (14/09)

Posta com as três colunas acima, a escolha foi a **`Neutral`**. ⇒
[`crate::shading::OPENING_LOOK`](../../crates/ph2d-app-field3d/src/shading.rs), lido pela
`View::default` — que é a mesma porta que o `boot` usa.

⛔⛔ **E ela NÃO é o `Look::default()`, de propósito.** São duas leis:

| afirmação | de quem |
|---|---|
| `Look::default()` continua a ser a **identidade** (`Standard`, `0` stops) | do **tipo**, e vale para todo consumidor dele |
| o **módulo** abre em `OPENING_LOOK` | decisão de **produto**, e vive na `crate::shading` |

*Quem trocasse a primeira mudaria, à distância, o quadro de quem nunca pediu olhar nenhum.*

⚠️ **O modo de abertura NÃO se mexeu** — o matcap continua a ser o que um modelador vê primeiro (ele
lê **forma**). A decisão foi sobre o olhar.

⭐ **Gate `the_modeler_opens_with_the_look_the_owner_chose`**, com as duas metades e a ordem nomeada:
*é exactamente o tipo de linha que alguém «simplifica» de volta para `Look::default()` numa limpeza*,
e sem ele a decisão evaporaria em silêncio com o quadro a voltar ao branco chapado. 2 mutações, 2
sangrias.

### §15.6 — ⛔⛔ E a mudança de omissão APANHOU UM GATE ACABADO DE ESCREVER

O `changing_how_it_is_painted_drops_the_frame_that_was_already_traced` (§13) reprovou **sobre produto
correcto**: ele escrevia `SetLook { slot: 1 }`, que era *«a vista que não é a de omissão»* no dia em
que foi escrito. Com a omissão em `Neutral`, o `slot 1` passou a ser **o estado em vigor** — e o
`set_look` devolve cedo quando o olhar não muda, logo não larga pedido nenhum.

⚠️ *Uma fixtura que escolhe um slot LITERAL está a afirmar que ele difere do estado — e deixa de o
afirmar no dia em que a omissão se mexe, sem uma linha do gate mudar.* ⇒ os três slots passam a ser
**derivados** do estado lido no início do gate.

⭐ É a mesma família do `§0.0`: *quem move o número que tornava algo inalcançável tem de reconferir a
nota* — aqui, quem move a omissão tem de reconferir as fixturas que a pressupunham.


---

## §16 — ⭐⭐⭐ O CÉU PASSA A SER LIDO NA DIRECÇÃO MÉDIA DO LÓBULO (2026-09-14)

O §8 leva esta desde 13/09, e com a frase que a marcou para hoje: *«ela torna-se visível no dia em
que houver material por objecto»*. ⭐ **Esse dia foi ontem** (§12) — o metal passou a ser autorável.
*Quem move o número que tornava algo inalcançável tem de reconferir a nota* (`CLAUDE.md` §0.0).

### §16.1 — ⛔⛔ O defeito era um PARÂMETRO DEITADO FORA

```rust
fn radiance(&self, dir: [f32; 3], _alpha: f32) -> [f32; 3]
```

O `Environment::radiance` recebe a **largura do lóbulo** — a assinatura do MaterialX di-lo por
escrito (*«já pré-filtrada para um lóbulo GGX de rugosidade `alpha`»*) — e o céu de estúdio
**descartava-a**.

⚠️ É a segunda espécie de controlo morto do `CLAUDE.md` §5.0: *o consumidor que projecta o valor
fora*. O fio está inteiro, o valor chega, e quem o recebe ignora-o — **e nenhuma sonda de «quem lê
este campo?» o vê, porque ele *é* lido**: está na assinatura.

### §16.2 — ⭐⭐ A cura é UM ESCALAR, e é EXACTA

Este céu é **linear na altura** (`L(ω) = A + B·ω.y`), e a média de uma função linear sobre uma
distribuição é a função avaliada na **direcção média** dela. A distribuição do pré-filtro é simétrica
em torno da espelhada `R` ⇒ a componente perpendicular cancela e sobra `E[ω] = c(α)·R`:

```text
média(A + B·ω.y) = A + B · c(α) · R.y
```

⚠️ **É a convolução em harmónicos esféricos, não uma heurística:** uma função de grau `1` convolvida
com um núcleo simétrico é a mesma função de grau `1`, escalada pelo coeficiente de grau `1` do núcleo
— e `c(α)` **é** esse coeficiente.

A forma fechada, integrada da amostragem de importância do GGX com `N = V = R` (a suposição do
*split-sum*, que é a do `mx_environment_prefilter`), com `a = α²`, `k = a−1`, `m = a+1`, `L = ln(2a/m)`:

```text
c(α) = [ k(3a+1) − 4am·L ] / { k · [ 2a·L − 2a + m ] }
```

⭐ **Dois controlos que não são coincidência:** `c(0) = 1` (o lóbulo colapsa na espelhada) e
`c(1) = 2/3` — que é **exactamente** o `(2/3)·k` do lóbulo cosseno que o `ENV_SLOPE` da `ph2d-light`
já carrega, e que este ficheiro desfaz com o `RAW`. *A rugosidade máxima do GGX é o hemisfério
cosseno, e as duas metades da casa chegam ao mesmo número por caminhos diferentes.*

⚠️ **A vizinhança de `a = 1` é singularidade REMOVÍVEL e numericamente instável** (numerador e
denominador vão os dois a zero como `k³`): abaixo de `|k| = 1e-3` devolve-se o limite, com desvio
`< 1e-4`. ⛔ **E o gate varre essa vizinhança de propósito** — sem esses pontos ele ficaria verde
sobre a única região onde a forma fechada não se pode usar crua.

### §16.3 — ⛔⛔ E porque a cura NÃO mora na `ph2d-material`

A razão de sempre vale (*ela é o port fiel do GLSL, e «melhorar» uma fórmula ali deixa de ser a
referência*), mas há uma mais forte: **«avaliar na direcção média» só é EXACTO porque ESTE céu é
linear.** Sobre um céu com feições a lei é outra. Escrevê-la na crate do material seria prometer, a
quem trouxer o céu seguinte, uma exactidão que ela não tem.

### §16.4 — O que muda no pixel, por material

Sonda [`measure_what_the_lobe_cure_changes_in_the_pixel`](../../crates/ph2d-app-field3d/src/render_light_tests.rs)
— a mesma esfera, a mesma luz, os dois céus:

| material | pixels que mudam | `|Δ|` p50 | p95 | max |
|---|---:|---:|---:|---:|
| omissão (dieléctrico, `r 0,30`) | `13 %` | `0` | `1` | `1` |
| metal polido (`metal 1`, `r 0,10`) | `9 %` | `0` | `1` | `1` |
| metal escovado (`metal 1`, `r 0,50`) | `88 %` | `2` | `11` | `15` |
| **metal fosco** (`metal 1`, `r 1,00`) | `97 %` | `4` | **`22`** | **`32`** |

⭐⭐ **A previsão do §8 bate**: ele dizia *«no pixel, hoje, é ruído: `p50 = 0`, `p95 = 1`»* e *«num
METAL a rugosidade `1` custa `p95 = 23` e `max = 33`»*. Medido: `1` e `22`/`32`.

⇒ **a cura é cirúrgica**: o material de omissão fica onde estava (a peça que o dono já aprovou não se
mexe), e o que se endireita é exactamente o que passou a ser alcançável ontem.

### §16.5 — Os gates, e porque são DOIS

| gate | o que ele prende |
|---|---|
| `the_closed_form_of_the_lobe_agrees_with_the_quadrature` | a **lei**: a forma fechada contra a quadratura do ponto médio (`2¹⁶` intervalos), barra `2e-4`, mais os dois controlos e a **monotonia** |
| `the_sky_honours_the_lobe_width_it_is_handed` | a **costura**: que alguém a chama |

⛔ **O primeiro sozinho é cego ao defeito que esta wave curou** — a lei podia estar certa e o
parâmetro continuar a ser deitado fora, que era exactamente o estado anterior. *Um gate sobre a lei é
cego a um consumidor que a ignora.*

⚠️ **E o segundo mede o par**: num céu avaliado no **equador** o encolhimento é invisível por
construção (`c·0 = 0`), então ali ele afirma a **invariância** — sem essa metade, um `radiance` que
ignorasse o `dir` inteiro também passaria.

⛔⛔ **E o oráculo do céu (§2) continua cego a isto, por construção:** ele corre sobre um céu
**constante**, e num céu constante a espelhada e a média do lóbulo são iguais. *O gate desta lei tinha
de ser escrito de novo, e não estendido.*


---

## §17 — ⏱️ O que o MATERIAL POR OBJECTO custa ao sombreamento (2026-09-14)

⚠️ **Uma obrigação do `CLAUDE.md` §0.0 que estava por cumprir:** a §11 acrescentou uma resolução de
**dono por pixel** dentro do laço mais quente do quadro, e a única medição que existia era a da
resolução **isolada** (`1,6 ms` a 16 folhas sobre `26 100` px). *Quem acrescenta um custo ao caminho
quente mede-o no caminho quente, e não numa sonda ao lado.*

Sonda [`measure_what_material_per_object_costs_the_shading`](../../crates/ph2d-app-field3d/src/render_light_tests.rs)
— o `shade_render` inteiro, com e sem donos, sobre a mesma peça (`640×360`, mínimo de 5):

| folhas | peça px | sem donos | com donos | delta | % de um quadro |
|---:|---:|---:|---:|---:|---:|
| `2` | `12 024` | `1,558` | `1,469` | `≤ ruído` | `8,8 %` |
| `4` | `26 469` | `2,689` | `2,895` | `0,207` | `17,3 %` |
| `8` | `22 290` | `3,125` | `3,444` | `0,319` | `20,6 %` |
| `16` | `26 059` | `4,453` | `5,475` | **`1,021`** | `32,8 %` |

⇒ **`+1,0 ms` a 16 folhas** — `23 %` do sombreamento, `6 %` de um quadro. O sombreamento continua
**abaixo da marcha**, que o §8 mede em `80 %` do quadro de movimento. *Não é o tecto, e não há cura a
fazer.*

⚠️ **A linha de `2` folhas lê negativo** (`−0,089`): é ruído a `load 23`, e está escrita assim de
propósito — *um delta menor que o ruído reporta-se como ruído, não como ganho*.

### §17.1 — E o que a mesma corrida disse sobre a decisão do olhar

A [`measure_what_the_render_mode_costs_and_paints`], corrida depois da §15.5:

| exposição | `Standard` — pixels brancos | `Neutral` |
|---:|---:|---:|
| `−2` | `0` | `0` |
| `−1` | `114` | `0` |
| `0` | `5 180` (`8,4 %`) | **`0`** |
| `+1` | `37 177` | **`0`** |
| `+2` | `52 467` (`85 %`) | **`0`** |

⭐ A `Neutral` não corta em **exposição nenhuma** — é isso que ela compra, e o `+2` é onde a diferença
deixa de ser subtil: `85 %` da peça em branco chapado contra zero.


---

## §18 — ⭐⭐⭐ PINTAR VÁRIAS FORMAS DE UMA VEZ (2026-09-14)

A §12.7 dizia: *«um material num GRUPO não existe: quem o traçado sabe nomear por pixel é a folha.
Herdar do grupo é modelo novo.»* ⭐ **A primeira metade continua verdadeira e a segunda era falsa** —
não é preciso modelo novo nenhum, porque a resposta não é *herança*, é **alcance de escrita**.

### §18.1 — A lei: o material espalha, a dimensão não

| pedido | alcance |
|---|---|
| **material** (a cor, a rugosidade, o metal) | a **selecção inteira**, com cada grupo resolvido nas folhas debaixo dele |
| **dimensão** (largura, raio, posição, ângulo…) | só o nó que a pediu |

⚠️ **A distinção não é arbitrária, e é o que impede isto de ser um esmagamento:** largura, raio e
posição são **daquela forma**, e espalhá-los destrói o desenho das outras. ⭐ *Um material é a única
coisa que um artista atribui a MUITOS objectos de uma vez* — é o `assign material to selection` de
todo DCC, e é a continuação directa do gesto que o dono pediu ao pedir a caixa de cor: **pintar uma
peça, não uma face.**

⛔ **Um grupo continua a não TER material** — o que ele oferece é o das folhas da sub-árvore, e é lá
que a escrita cai. O traçado continua a saber nomear só a folha.

### §18.2 — ⛔⛔ Espalhar sem sinal é um ESMAGAMENTO silencioso

O controlo mostra o valor de **uma** forma. Sem um sinal, o artista lê *«estou a pintar esta»* e pinta
cinco — e quando as formas **discordam**, o valor mostrado é **falso** sobre as outras.

⇒ [`ParamRow::subject`](../../crates/ph2d-panel-model3d/src/state.rs): uma nota pintada **antes** da
linha, que diz sobre quantas formas ela escreve e, **só quando elas diferem**, que a amostra mostra a
primeira.

*É a lei que a caixa «Visible» do Inspector já pagou, à letra: «espalhar sem sinal troca um
sub-aplicar silencioso por um esmagamento silencioso».*

⚠️ **Três cercas, cada uma com o seu gate:**

- **uma forma só não leva nota** — ela é o sujeito óbvio, e uma nota permanente sobre o gesto mais
  comum do painel é ruído;
- **o aviso de divergência só aparece quando ela existe** — com as formas de acordo a amostra
  descreve todas, e um *«diferem»* ali seria mentira ao contrário;
- **a nota é um `String` composto e não uma chave** (HR-15): ela tem um NÚMERO dentro, e quem o sabe é
  o shell. É a mesma forma do `verb_subject`, que já compõe o nome da forma com um prefixo traduzido.

### §18.3 — ⭐⭐ E um pedido de um quadro atrás pinta só a forma DELE

O pedido traz o `entity` da **linha** que o produziu, e a selecção pode ter mudado entre o quadro que
a pintou e o que a drena (um clique no canvas, um desfazer). ⇒
[`material_reach`](../../crates/ph2d-app-field3d/src/scene_panel_material.rs) confere o sujeito do
pedido contra o presente: **fora do alcance de agora, a escrita cai só nele.**

⛔ Sem esta cerca, um pedido velho pintaria a selecção **de agora** — e o artista veria formas que
nunca escolheu mudar de cor. *É a mesma família do id da amostra (§12.3): um pedido carrega o sujeito
que o produziu, e quem o executa confere-o contra o presente.*

### §18.4 — Os gates (4) e as mutações (6/6)

| gate | o que ele prende |
|---|---|
| `a_group_offers_the_material_of_the_shapes_under_it_and_painting_it_paints_them_all` | as **duas** metades — oferecer sem escrever é o botão mudo; escrever sem oferecer é o gesto inalcançável |
| `a_dimension_never_spreads_across_the_selection` | o **controlo** que separa a lei nova de um esmagamento |
| `the_note_says_how_many_shapes_and_whether_they_differ` | §18.2, com as três cercas |
| `a_request_from_a_stale_selection_paints_only_its_own_shape` | §18.3 |

⭐ **6 mutações, 6 sangrias** — entre elas *apagar a guarda `param @ Material(_)`* (toda dimensão
passaria a espalhar) e *a nota avisar sempre que diferem*.

### §18.5 — ⚠️ E dois TETOS DE LOC caíram no caminho

O `scene_panel.rs` foi a `757` e o `render_light_tests.rs` a `765`, contra `700`. ⛔ *Corte por
responsabilidade, nunca uma entrada no `FILE_OVERAGE_OK`* — e os dois ficaram melhores:

- [`scene_panel_material.rs`](../../crates/ph2d-app-field3d/src/scene_panel_material.rs) — **um
  assunto**, e ele atravessa as duas pontas da costura (o retrato precisa do alcance para a nota, o
  dreno precisa dele para escrever);
- [`render_light_lobe_tests.rs`](../../crates/ph2d-app-field3d/src/render_light_lobe_tests.rs) — o
  oráculo, os dois gates e as duas sondas da §16, que se leem juntos e sem o resto.

⚠️ **E os testes foram CONFERIDOS depois de mudarem de ficheiro** (`nextest list`): mover código parte
gates em duas espécies e **só uma avisa** — um ficheiro que nenhum `mod` declara não é compilado, e as
suítes ficam verdes com os testes ausentes.


---

## §19 — ⭐⭐⭐ A FRONTEIRA ENTRE DUAS CORES DEIXA DE SER UMA ESCADA (2026-09-14)

A §11.6 declarava esta enquanto ela era invisível. A §12 (a caixa de cor) e a §18 (pintar várias
formas) tornaram-na o gesto normal: **duas formas, duas cores, uma peça**.

### §19.1 — ⛔⛔ Ela NÃO é a silhueta, e é por isso que ninguém a suavizava

O anti-serrilhado do traçado corre nas [`Gbuffer::edges`] — pixels em que **algumas** sub-amostras
acertam a peça e outras não. Uma fronteira **entre dois materiais** no meio da peça não é nenhuma
dessas: ali **todas** as sub-amostras acertam, **não há registo de borda nenhum**, e a cor muda de um
pixel para o outro a pique.

⇒ *a peça ficava com o contorno liso e uma escada por dentro.*

### §19.2 — ⭐⭐ A cura é um escalar, e a largura sai da geometria

A fronteira é onde `|f_dono| = |f_rival|`. Andando `s` perpendicular a ela, uma distância cresce `~s`
e a outra decresce `~s` ⇒ a diferença `d` varia `2s`. Logo:

```text
t = máx(0, ½ − d / (2 · largura))
```

⇒ [`Owners::mix_at`](../../crates/ph2d-field-eval/src/owners.rs), e o sombreamento **pinta duas vezes
e mistura o resultado**. ⚠️ O resultado, e **não os materiais**: um metal e um dieléctrico a meio
caminho não são um meio-metal, e o que uma super-amostragem convergiria a dar é a mistura das **luzes**.

⛔ **E só paga o dobro onde há fronteira** (`t > 0`): fora dela, e numa peça de um material só, é uma
chamada e mais nada.

### §19.3 — ⛔⛔ A margem da bola era a da MARCHA, e isso tornava a lei MUDA

O filtro da bola à frente usa a margem com que o **ponto** foi produzido (`~2e-4`). Mas a pergunta
aqui não é *«quem pode GANHAR?»* — é *«quem pode estar a menos de uma LARGURA de ganhar?»*, e uma
largura mede `4,4e-3`, **vinte vezes mais**.

⇒ medido, no pior pixel de uma união dura o rival vinha **filtrado** e a mistura lia `(0, 0, 0,0)` —
*o segundo melhor era o próprio dono*. A cura é uma linha, e sem ela a lei inteira não fazia nada
naquele caso.

### §19.4 — ⚠️ A RÉGUA CORRIGIU-SE TRÊS VEZES ANTES DO PRODUTO

| # | o que ela media de errado | a cura |
|---|---|---|
| 1 | **duas coisas**: numa união dura o maior salto é o **vinco**, onde a normal muda a pique e a luz dá um degrau **legítimo** — lia `236` bytes e atribuía-os à cor | a diferença contra um **controlo** (a mesma peça com as duas folhas do mesmo material) |
| 2 | **a população errada**: no pior pixel os dois pontos estão a **`16,3` px** um do outro em 3D — eles não são vizinhos **na superfície** | só vizinhos que o são também na superfície |
| 3 | e foi ela que mostrou o §19.3, ao não se mexer numa união dura | — |

*Uma régua que mede duas coisas não mede nenhuma.*

### §19.5 — O factor, com a tabela ao lado

A forma fechada supõe que se anda **perpendicular à fronteira**; quem percorre pixels anda **ao longo
da superfície visível**, e o ângulo entre as duas encolhe o passo. ⇒ o factor é a correcção desse
ângulo, e foi **varrido** (duas esferas, vermelha e azul, `640×360`; a coluna é quanto a **cor**
acrescenta ao degrau, já subtraído o controlo e só sobre vizinhos de superfície):

| factor | união dura | união suave |
|---:|---:|---:|
| `0` (sem a lei) | `+166` | `+191` |
| `1` (a forma fechada crua) | `+84` | `+148` |
| `1,5` | `+48` | `+109` |
| **`2`** ⬅ | **`+34`** | **`+95`** |
| `3` | `+16` | `+75` |
| `4` | `+2` | `+51` |

⚠️ **O joelho está em `2`**, e acima dele o que se compra é **desfoque**: a fronteira deixa de ser uma
aresta suavizada e passa a ser um degradê de N pixels. *Mais suave nem sempre é melhor — uma fronteira
de material tem de continuar a ler-se como fronteira.*

### §19.6 — ⏳ O que fica, e com que endereço

- **A derivada no ECRÃ** (`t = ½ − d / (2·|∂d/∂pixel|)`) dispensa o factor por medir o ângulo em cada
  pixel. Pede o **gradiente** dos dois campos — seis avaliações nos `~0,5 %` de pixels de fronteira.
  **Nomeada, não construída.**
- ⛔ **A cura de raiz é sub-amostrar o DONO**, e está **bloqueada por desenho**: o padrão `ROOK` já
  re-marcha quatro sub-amostras num pixel de silhueta, mas o `EdgePixel` guarda **normais, não
  pontos**, e a marcha **não conhece donos**. Dar-lhos é o *id-buffer*, que esta linha **mediu e
  recusou**.
- **Onde a superfície SALTA em profundidade** (um vinco visto de raspão) a diferença de cor entre
  vizinhos é tão legítima como a de dois pixels em lados opostos da silhueta. *Nenhuma mistura por
  ponto pode, ou deve, curar isso.*

### §19.7 — Os gates (4) e as mutações (6/6)

| gate | o que ele prende |
|---|---|
| `the_mix_is_half_on_the_boundary_and_zero_away_from_it` | empate ⇒ `½` · longe ⇒ `0` · monótona · **e do lado da OUTRA folha** |
| `the_width_drives_the_ramp_and_the_ball_filter_follows_it` | §19.3 |
| `the_ramp_reaches_zero_while_the_rival_is_still_in_play` | separa *«a mistura desce»* de *«o filtro deixou de ver o rival»* |
| `the_boundary_between_two_colours_is_not_a_step` | a costura, **no pixel**, com o controlo e o piso |

⛔⛔ **E DUAS mutações sobreviveram à primeira redacção, pela MESMA causa: os pontos da fixtura não
estavam SOBRE a superfície** — e fora dela o filtro deixa passar **uma** folha só, o `mix_at` devolve
*«não há rival»* **antes** da conta, e o gate mede um caminho que o produto nunca percorre. *É a lei
que o topo daquele ficheiro já escrevia, à letra, e a 1.ª redacção violou-a.*

⚠️ **E uma terceira asserção era INSATISFAZÍVEL numa esfera**: a bola envolvente de uma esfera **é** a
superfície dela, logo *«a rampa acabou»* e *«o filtro deixou de ver»* são a **mesma condição**. Ela
vive numa fixtura de **CAIXA**, cuja bola é folgada. *A forma da fixtura não é um detalhe — ela decide
que caminho do produto o gate percorre.*

⚠️ **E o `Surfaces::of` foi APAGADO**: o `mix_of` tomou-lhe os dois chamadores, e um método que
ninguém chama é **lixo**, não um morto a ligar (`CLAUDE.md` §5.0).

---

## §20 — ⭐⭐⭐ UMA PEÇA QUE DÁ LUZ: o brilho próprio (2026-09-14)

A `Surface::emission` do OpenPBR **já era paga por amostra** e somava `[0,0,0]`: a §8 mediu `12,18 ns`
de `265,64 ns`, **`4,6 %`** do relógio de sombreamento a produzir nada, e escreveu a cura — *«uma
guarda `emission_luminance > 0`»*.

⭐⭐ **Havia uma segunda saída, e ela não estava escrita:** a lei está paga e **ninguém lhe chegava**.
O `materials::surface_of` escrevia `3` dos `15` números do OpenPBR, e a emissão era um dos `12` que
ficavam no padrão da nodedef — *uma capacidade viva sem botão nenhum* (`CLAUDE.md` §5.1, o pincel de
tecido antes da fileira de chips).

⇒ **as duas foram feitas**: a guarda ficou, e o que ela protege passou a ser autorável.

---

### §20.1 — A medição que escolheu a faixa (`emission_tests`, sonda)

Esfera de omissão a `640×360`, olhar do produto (`Neutral`, `0` stops), verde médio na peça:

| luminância | verde médio | Δ desde o ponto anterior | pixels em branco chapado |
|---|---|---|---|
| `0,00` | `188,31` | — | `0` |
| `0,05` | `195,73` | `+7,42` | `0` |
| `0,25` | `218,25` | `+15,93` | `0` |
| `0,50` | `235,26` | `+17,00` | `0` |
| **`1,00`** | **`245,98`** | `+10,72` | `0` |
| `2,00` | `250,39` | `+4,42` | `0` |
| `8,00` | `253,77` | `+1,05` | `0` |
| `16,00` | `254,53` | `+0,76` | `26 380` |
| `32,00` | `254,77` | `+0,24` | `61 492` |
| `64,00` | `254,77` | **`0,00`** | `61 492` |

⭐ **A ponta do slider é `1`, e ela é MEDIDA:** `0 → 1` move `57,67` dos `66,46` bytes que a grandeza
tem para dar — **`86,8 %`** de toda a excursão possível. O `2` compra mais `6,6 %`, e **acima de `32`
a saída é bit a bit a mesma**: o olhar satura e o número deixa de ser observável.

⚠️ **O campo numérico continua aberto**, como em toda linha de material (`Span::SoftFromZero(1.0)`):
o que a medição fecha é o **curso do dedo**, não o que a peça pode afirmar sobre si.

⚠️ **E `0,05` já move `7` bytes** — isto é, o controlo é grosso perto do zero. *Um brilho não tem
ponto morto: ele começa a ver-se no primeiro centésimo.*

---

### §20.2 — ⛔ A guarda é `== 0.0`, e não `<= 0.0`

Com `luminance == 0` o `scale3` zera as três componentes e **tudo a jusante é uma multiplicação por
zero** — a guarda **observa** a álgebra, não a muda. Um `<= 0.0` **mudaria** a resposta para uma
luminância negativa, e a `ph2d-material` é o **port fiel** do GLSL de referência: uma entrada fora da
faixa da nodedef é assunto de quem autora, não desta lei.

⭐⭐ **E o que ela poupa foi MEDIDO no quadro inteiro**, com o A/B que a isola: o segundo ponto é
`f32::MIN_POSITIVE`, cuja luminância percorre o corpo **inteiro** da `emission` e acrescenta ao pixel
uma radiância de `1e-38`. *A imagem é a mesma; a diferença de relógio é exactamente o ramo que a
guarda salta.* ⛔ Comparar com um brilho de `1,0` mediria *«o que um material aceso custa»*, que é
outra pergunta — e um A/B de **build** não cabe numa corrida, enquanto um A/B de **entrada** cabe.

| corrida (mín. de 7, `load ~26`) | com guarda | sem ela | poupança |
|---|---|---|---|
| 1 | `1,78 ms` | `1,93 ms` | **`7,6 %`** |
| 2 | `1,82 ms` | `1,94 ms` | `5,8 %` |
| 3 | `1,81 ms` | `1,93 ms` | `6,2 %` |

⚠️ **E um brilho de verdade custa o mesmo que não ter guarda** (`1,89`–`1,92 ms`), que é a confirmação
de que o A/B mede o ramo e não a luz: *quem usa o brilho paga-o; quem não usa deixou de pagar.* O
número corrobora a estimativa de `4,6 %` da §8, medida por amostra noutra fixtura.

⛔⛔ **E o PREÇO da guarda não tem gate, por medição.** Um gate sobre a resposta é cego ao preço
(`CLAUDE.md` §5.0), e a cura que a wave anterior usou — um contador (`POINT_TAPES`) — **não serve
aqui**: aquele conta uma **compilação de fita**, que é rara; este contaria uma **amostra**, e um
`fetch_add` partilhado por 32 threads sobre `26 100` pixels custa mais do que a comparação que a
guarda poupa. *Um instrumento mais caro que o defeito que mede é um defeito novo.* ⇒ a régua do preço
é a sonda, e está declarada como tal.

---

### §20.3 — ⭐⭐ O painel: uma linha nova, e uma que fica TRAVADA

> ⛔⛔ **CORRIGIDO PELA §23 (14/09, ordem do dono): a linha não desaparece — ela fica VISÍVEL e
> INACTIVA.** O que segue descreve a primeira versão, em que ela era escondida.

| linha | quando aparece |
|---|---|
| **Emission** (`Material(5)`) | sempre, numa folha |
| **Emission Color** (`Material(6)`, amostra) | **só com a luminância acima de zero** |

⭐⭐⭐ **A cor do brilho MULTIPLICA a luminância**, logo a zero varrer o selector não muda um bit do
quadro. Publicá-la ali seria **o knob morto na espécie mais cara** do `CLAUDE.md` §5.0 — *o consumidor
que projecta o valor fora*: o fio está inteiro, o valor chega, e a matemática descarta-o.

⚠️ **Quem esconde é a APRESENTAÇÃO, e a porta de escrita NÃO se estreita:** o `set_param` continua a
aceitar `Material(6..=8)`. Sem isso, um pedido guardado de um quadro atrás — o selector ainda aberto
quando a luminância vai a zero — cairia em silêncio, que é a mesma família do §13.2.

⚠️ **`MATERIAL_FIELDS` deixou de ser «quantas linhas»**: ele é **quantas posições a escrita aceita**
(`9`), e o `params_of` publica **até** isso. Um gate que os confundia foi corrigido nesta wave
(`scene_gesture_tests`) — ele exigia exactamente `MATERIAL_FIELDS` linhas, que é precisamente o que a
lei da W34 proíbe.

---

### §20.4 — ⛔⛔ O selector é UM, e agora há DUAS amostras na mesma forma

O id da amostra passou a ser o par `(entidade, campo)`. Sem o campo, abrir o selector na cor base e
carregar na amostra do brilho deixaria as **duas** a responder *«aberto em mim»*: a segunda leria a
cor escolhida para a primeira e escrevê-la-ia por cima, **em silêncio**. É o mecanismo do §12.3 um
nível abaixo — *o sujeito de um id flutuante é o par `(quem, qual)`, e não um dos dois*.

⚠️ **E a lei «o selector segue o sujeito» passou a lembrar um CONJUNTO.** Ela guardava *a* amostra da
última pintura; com duas, uma memória de uma só responderia *«a aberta já não é pintada»* sempre que a
outra fosse desenhada depois dela — **fechando o selector no quadro seguinte a abri-lo**, sem nada ter
mudado. Hoje guarda a lista, e o conjunto é contado **na mesma faixa que foi pintada** (`take(MAX_ROWS)`).

⚠️ **E a porta da travessia sRGB↔linear mudou de nome** (`base_color_srgb8` → `colour_srgb8`): com duas
cores autoradas, uma porta chamada `base_color_*` convida a segunda a escrever a conversão outra vez ao
lado. *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é.*

---

### §20.5 — ⛔⛔ O TETO DE LINHAS NÃO SUBIU, e a razão é uma RÉGUA CORRIGIDA

O gate que dimensiona a família de widgets do painel (`polygon_rows_tests`) contava **`params_of`** — e
isso deixou de ser o número de linhas em **14/09**, quando a amostra de cor dobrou três params numa
linha. Ele lia `2N + 19` onde o painel pinta `2N + 15`.

⚠️ **Ele errava a FAVOR**, o que o torna invisível: *uma régua conservadora não avisa no dia em que
deixa de descrever o que mede.* Corrigida para contar no **produtor das linhas** (`param_rows`), e no
**pior estado** (brilho aceso, que publica a amostra extra):

| vértices | linhas, brilho apagado | linhas, brilho aceso |
|---|---|---|
| `3` | `20` | `21` |
| `16` | `46` | `47` |
| **`27`** (o teto) | `68` | **`69`** |

⭐ **A família estava sobre-provisionada em exactamente `2`** — os dois canais dobrados que a régua
velha contava —, e o brilho consumiu essa folga. O `MAX_ROWS` fica em **`69`**, com **zero** de folga
medida, e o `MAX_POLYGON_VERTICES` não perde um vértice.

⚠️ **E o literal `15` do gate continua `15` por DUAS correcções de sinal oposto** (`−4` canais
dobrados, `+1` linha de brilho). *Um número que não se mexe enquanto a grandeza muda é a forma mais
silenciosa de um gate deixar de descrever o que mede* — o que o prende é a nota, não a coincidência.

---

### §20.6 — O arquivo: `PROJECT_SCHEMA` **128 → 129**

O `FieldMaterial` passou de `5` para `9` números. O postcard é **posicional e sem comprimento**: um
blob v128 tem `20` bytes e este binário pede `36`, então um material gravado antes desta wave sairia
com *«Hit the end of buffer»* **no meio da travessia dos componentes**, porque o `ComponentBlob` é
opaco ao parse do `ProjectFile`. O degrau transforma isso num **erro de versão**.

⚠️ **O `FIELD_DOC_VERSION` NÃO se mexe**, o que parece estranho num degrau que fala de material: o
documento do campo é **geometria** — é ele que a marcha compila —, e uma cor não muda uma distância.

⛔ **Conte o DELTA (`+1`) contra a árvore em que a linha aterrar**, nunca o literal (`CLAUDE.md` §5.0).

---

### §20.7 — Os gates (5) e as mutações (8/8)

| gate | o que ele prende |
|---|---|
| `a_glow_carries_the_authored_colour_into_the_law` | a travessia `FieldMaterial → OpenPbr`, com a **cor** e não só a luminância |
| `no_glow_is_exactly_black_and_a_coat_still_filters_one` | a guarda é legítima **e** não come o caminho aceso — com o ramo do verniz dentro |
| `the_colour_of_the_glow_only_exists_while_the_glow_does` | a lei da W34 sobre o par de amostras, e as duas cores distintas |
| `each_colour_of_a_shape_has_its_own_picker` | o id é o par `(quem, qual)` |
| `a_glow_colour_reaches_the_document_without_touching_the_base` | o dreno escreve em `field + k`, e não em `k` |

⚠️ **A mutação mais perigosa é a do dreno** (`field + k` → `k`): a cor do brilho aterraria na **cor
base** — a peça mudava de cor e o brilho ficava branco, **sem erro nenhum**. É a redacção mais natural,
porque era o que lá estava antes.

⚠️ **E a prova de mutação do teto de linhas precisou de CONTROLO no próprio filtro:** a primeira
corrida passou `--exact one_more_vertex_would_not_fit` sem o caminho do módulo, casou **zero** testes,
e o `0 passed` leu-se como *«sangrou»*. *Um filtro que casa zero imprime a mesma coisa que um gate que
morre* — é a lição que a `project-memory` já registava, paga outra vez.

---

### §20.8 — ⏳ O que fica aberto

- **O material continua sem ALFA** (§12.7) — e agora com uma pergunta a mais: um brilho é aditivo e a
  transparência é multiplicativa, então as duas não se resolvem na mesma wave.
- **As outras `10` entradas do OpenPBR** que a lei honra e nenhum controlo alcança — o **verniz** (5),
  o `specular_weight`/`specular_color`/`specular_ior`, o `base_weight` e a `base_diffuse_roughness`. ⚠️ **Nenhum é
  como a emissão:** eles não estão a ser **pagos a zero**, e a pergunta deles é de produto (*quantos
  knobs um modelador quer ver?*), não de dívida.
- **O brilho não ILUMINA a vizinhança** — ele acende a própria superfície e mais nada. Um emissor que
  ilumina é *global illumination*, que este traçador não faz e cujo preço não foi medido.

---

## §21 — ⭐⭐⭐ O VERNIZ: a última coisa do material que a lei fazia e ninguém alcançava (2026-09-14)

Depois do brilho próprio, o `materials::surface_of` escrevia `5` das `15` entradas do OpenPBR. **Cinco**
das dez que faltavam eram o verniz — `coat_weight`, `coat_color`, `coat_roughness`, `coat_ior`,
`coat_darkening` —, e a lei honra-os todos: o `prepare` deriva o `coat_alpha`, o `coat_f0`, o
escurecimento da base e a atenuação a partir deles.

⚠️ **Mas «a lei honra-o» não é «ele move o quadro»**, e a §20.8 tinha escrito que a pergunta do verniz
era *de produto*. A medição respondeu antes: **os cinco movem**, e por isso nenhum tem recusa por
inércia.

---

### §21.1 — A medição (`coat_tests`, sonda)

Esfera a `640×360`, **base FOSCA** (`specular_roughness 0,6` — é sobre uma superfície baça que um
verniz aparece como o que é: um segundo realce nítido por cima de um primeiro que não é), olhar do
produto.

| o quê | valor | pior Δ por pixel | pixels que mudam ≥ 2 |
|---|---|---|---|
| **peso** | `0,10` | `7` | `50 182` |
| | `0,50` | `32` | `61 612` |
| | `1,00` | **`63`** | `61 647` |
| rugosidade | `0,10` | `44` | `1 662` |
| | `1,00` | `50` | `53 268` |
| IOR | `1,00` | `32` | `61 796` |
| | `2,50` | `49` | `61 802` |
| **cor** | `0,6` | **`120`** | `61 804` |
| escurecimento | `0,00` | `29` | `61 804` |

⚠️⚠️ **A régua é o MÁXIMO por pixel, e não a média** — e isso é uma correcção sobre a wave anterior:
um verniz é um realce **local** (ele acende uma mancha e deixa o resto onde estava), enquanto a
emissão é **global**. *Uma régua emprestada da wave anterior mede a grandeza da wave anterior.*

⭐ **A rugosidade mostra a diferença numa coluna só:** o pior pixel mal se move (`44 → 50`) e a
**população** salta de `1 662` para `53 268` — ela não muda *quanto*, muda *onde*.

---

### §21.2 — ⛔⛔ O IOR nunca satura, e por isso a faixa dele é FÍSICA

A régua que fixou a ponta do brilho — *«onde o número deixa de ser observável»* — **não responde
aqui**. Varrido contra o ponto anterior:

| IOR | pior Δ contra o anterior |
|---|---|
| `1,4` | `12` |
| `2,0` | `24` |
| `2,5` | `26` |
| `4,0` | `20` |
| `10,0` | `17` |
| `20,0` | `17` |

⇒ ele **nunca pára**. O recurso é outro: **o material de que uma película transparente pode ser
feita**. O piso é `1` (a luz não atravessa nada mais depressa do que o vácuo) e o tecto é `2,5`, logo
acima do diamante (`2,42`), que é o mais alto dos transparentes conhecidos.

⛔ **`Span::Range` e não `SoftFromZero`**: as duas pontas são **duras** e digitar fora delas clampa.
*Um IOR abaixo de `1` não é um valor raro — é um valor que o modelo não admite*, e um slider a
começar em `0` daria metade do curso do dedo a sítios inexistentes.

⚠️ **É o primeiro número deste material que não é uma fracção**, e por isso o `material_span` deixou
de ser uma constante e passou a ser uma **tabela de um caso**. *Quinze dos dezasseis continuam
`0..1`, e o gate afirma os dois lados: o IOR duro, e o vizinho macio.*

---

### §21.3 — O painel: uma linha viva, quatro TRAVADAS sem verniz

> ⛔⛔ **CORRIGIDO PELA §23** — as quatro não desaparecem: ficam visíveis e inactivas.

| linha | quando aparece |
|---|---|
| **Coat** (`Material(9)`) | sempre, numa folha |
| **Coat Roughness** · **Coat Color** (amostra) · **Coat IOR** · **Coat Darkening** | **só com o peso acima de zero** |

⭐ **É a terceira vez que esta lei é aplicada neste painel** (o raio de junção, a cor do brilho, e
agora isto), e aqui ela tem a prova ao lado: o gate `every_coat_number_reaches_the_law_and_moves_the_answer`
mede que cada um move a resposta **com o peso ligado**, e o `prepare` mistura-os todos por
`coat_weight` — logo com ele a zero nenhum move um bit.

⚠️ **E os dois pesos são INDEPENDENTES:** acender o verniz não pode acender a cor do brilho. Um
`visivel` escrito com um `||` a mais fá-lo-ia, e há gate.

⚠️ **O `coat_darkening` fica, e o Blender não o expõe.** Medido, ele move `29` bytes no pior pixel, e
a lei desta casa é que um número vivo tem controlo. *Uma referência é um oráculo do que a LEI faz, não
um censo do que um painel deve ter.*

---

### §21.4 — ⭐⭐ O TETO DE LINHAS SUBIU, `69 → 74`, com a medição ao lado

Medido no produtor das linhas e no **pior estado** — brilho *e* verniz acesos, que é quando mais
linhas coexistem:

| vértices | tudo apagado | tudo aceso |
|---|---|---|
| `3` | `21` | `26` |
| `16` | `47` | `52` |
| **`27`** (o teto do polígono) | `69` | **`74`** |

⭐ **Preço MEDIDO:** cada linha regista `6` widgets, logo `69 → 74` custa **`30` widgets e 5
`String`** no store, uma vez, no arranque — o mesmo que a subida de `64 → 69`.

⛔⛔ **A alternativa era baixar o `MAX_POLYGON_VERTICES` de `27` para `24`:** tirar três vértices ao
artista porque uma peça passou a poder ser envernizada. *Um teto de registo cujo recurso é memória a
mandar num teto de FORMA é o caminho lento a definir o rápido.*

⚠️ **E o «pior caso» é um ESTADO, não uma propriedade da forma** — ele tem de ser reconferido a cada
número novo do material. A sonda acende **pelos pesos** (`5` e `9`), que são os únicos que decidem
visibilidade, em vez de guardar uma lista escrita à mão.

---

### §21.5 — ⛔⛔ Uma asserção minha de ONTEM reprovou hoje, e não havia defeito nenhum

O gate da ordem (`scene_gesture_tests`) exigia que os números do material saíssem **contíguos desde o
zero**, com a razão escrita: *«um buraco no meio faria a tabela de chaves e a porta de escrita
indexarem coisas diferentes»*.

**A razão era falsa.** As duas são indexadas pelo **`k`**, não pela posição na lista — e com o verniz
o material de omissão publica `0..5` e `9`, um buraco onde mora a cor do brilho. *Uma asserção escrita
sobre o estado em que a fixtura calhou de estar reprova a wave seguinte sem nomear defeito nenhum.*

⇒ o que é lei ali é a **ORDEM** (as linhas saem pela ordem dos campos), e é isso que o gate afirma
agora.

---

### §21.6 — O arquivo: `PROJECT_SCHEMA` **129 → 130**

O mesmo `FieldMaterial`, de `9` para `16` números — `36` bytes contra `64`, e o mesmo mecanismo do
degrau de ontem. ⛔ **Conte o DELTA (`+2` desde o `main`)**, nunca o literal.

---

### §21.7 — Os gates (3) e as mutações (10/10)

| gate | o que ele prende |
|---|---|
| `every_coat_number_reaches_the_law_and_moves_the_answer` | os cinco atravessam o `surface_of` **e** movem a radiância |
| `the_coat_numbers_only_exist_while_the_coat_does` | a lei da W34, e a independência dos dois pesos |
| `the_coat_colour_is_a_swatch_and_the_ior_is_not_a_fraction` | a terceira amostra, os três selectores, e a faixa dura do IOR ao lado da macia do vizinho |
| ⭐ `every_number_a_material_has_reaches_the_law` | **o censo, derivado do `MATERIAL_FIELDS`**: as `16` posições, uma a uma |

⭐⭐ **E o quarto é o que faltava às três waves anteriores.** A cor, o brilho e o verniz trouxeram cada
um o seu gate, e cada um cobria **os números daquela wave** — *um número novo escrito no
`FieldMaterial` e esquecido no `surface_of` não acordaria nenhum deles*: ele viajaria até ao
documento, seria gravado no arquivo, apareceria no painel, e não faria nada. Provado por mutação
sobre três campos de waves **anteriores** (o metal, a rugosidade, a cor base), os três a sangrar.

⚠️ **A régua do primeiro é a RADIÂNCIA, e não a `Surface`:** ela guarda o `OpenPbr` inteiro lá dentro,
logo duas superfícies com params diferentes são **sempre** diferentes por `PartialEq` — um gate
escrito assim passaria com a tradução a deitar o número fora, porque ele continuaria a viajar no campo
cru. *Compara-se o que a lei RESPONDE, não o que ela guarda.*

⛔⛔ **E o arnês das mutações mentiu duas vezes, as duas na mesma direcção:**
1. apagar uma entrada da tabela `CORES` é **erro de compilação** (o comprimento do array é literal), e
   o harness leu *«não compilou»* como *«sobreviveu»*. ⇒ *uma mutação que não compila não afirma
   nada*, e o harness passou a distinguir os dois.
2. um filtro escrito com o texto de antes do `cargo fmt` casou **zero** — o `0 passed` outra vez a
   ler-se como morte. O controlo do filtro, escrito na wave de ontem, apanhou-o à primeira.

---

### §21.8 — ⏳ O que fica aberto

- **CINCO entradas do OpenPBR continuam sem controlo:** `base_weight`, `base_diffuse_roughness`,
  `specular_weight`, `specular_color` e `specular_ior` — o `surface_of` escreve **`10` de `15`**.
  ⚠️⚠️ **Esta linha esteve errada duas vezes antes de ser contada** (*«sete»*, *«doze»*): eu somei de
  cabeça uma lista que a própria wave estava a alterar. *Um número que descreve o código lê-se do
  código* — aqui, com um `grep` ao `OpenPbr` e ao corpo do `surface_of`.
- **O material continua sem ALFA** (§20.8), e o verniz não muda isso: ele é uma película **opaca à
  transparência** — ela é a `transmission_*`, que a fatia do port deixou de fora com motivo escrito.
- **O `coat_roughness_anisotropy`** não existe na crate (o traçador não tem tangentes), e por isso não
  é um knob por construir — é uma ausência declarada.

---

## §22 — ⭐⭐⭐ O MATERIAL FECHA: as últimas cinco entradas, e a ordem que passa a ser permanente (2026-09-14)

Depois do verniz, o `surface_of` escrevia `10` das `15` entradas do OpenPBR. As cinco que faltavam —
`base_weight`, `base_diffuse_roughness`, `specular_weight`, `specular_color`, `specular_ior` — eram as
únicas que a lei honrava e nenhum controlo alcançava.

⚠️ **E elas não são como o verniz:** nenhuma tem um peso que as desligue, logo cada linha que ganhem é
uma linha que o painel mostra **sempre**. *O preço de as autorar não era o mesmo, e por isso a medição
também não podia ser.*

---

### §22.1 — A medição, e o que ela partiu em dois

> ⛔⛔ **CORRIGIDO PELA §23** — as duas só-dieléctricas não desaparecem num metal: ficam **travadas**.

Esfera a `640×360`, olhar do produto, contra o material de omissão de cada lado:

| entrada | valor | **dieléctrico**: pior Δ · px ≥2 | **metal**: pior Δ · px ≥2 |
|---|---|---|---|
| `base_weight` | `0` | **`224`** · `61 804` | **`255`** · `61 804` |
| `base_diffuse_roughness` | `1` | `39` · `53 914` | **`0` · `0`** |
| `specular_weight` | `0` | `17` · `22 381` | **`255`** · `61 804` |
| `specular_color` | `[1; 0,6; 0,2]` | `51` · `51 787` | `5` · `6 410` |
| `specular_ior` | `1,0` | `17` · `22 381` | **`0` · `0`** |

⭐⭐⭐ **DUAS delas são EXACTAMENTE inertes num metal** — `0` bytes em `61 804` pixels, não «pouco».
A rugosidade da difusa e o IOR alimentam o lóbulo **dieléctrico**, que o `base_metalness` mistura para
fora: `mix3(dieléctrico, metal, 1)` é `dieléctrico × 0 + metal × 1`, e `x × 0` é zero por construção.

⇒ **a lei da W34 aplica-se pela terceira vez neste painel, e pela primeira com um predicado que não é
um peso a zero:** a cor do brilho morre com a luminância, os quatro do verniz morrem com o peso dele,
e estes dois morrem com o **metal a um**. *O que decide não é a forma do predicado — é o efeito ser
sempre nulo.*

⚠️ **E o `specular_weight` muda de NATUREZA com o vizinho de cima:** num dieléctrico mal se vê (`4`
bytes a meio curso), num metal ele **é** o brilho da peça. *Um gate posto no lado fraco de um número
que muda de natureza mede o lado que não importa* — o gate dele corre no metal.

---

### §22.2 — ⭐⭐ A ORDEM passa a ser a da NODEDEF, e é agora que ela se arruma

Até aqui a ordem das posições era a **ordem de chegada das waves** (a cor, o brilho, o verniz). Com
estas cinco, o `base_weight` teria aterrado **depois** do escurecimento do verniz.

⇒ as `23` posições foram **re-numeradas** para a ordem em que o `ND_open_pbr_surface_surfaceshader`
declara as entradas — a mesma do `pub struct OpenPbr` e da linha `D` da fixture do oráculo:

| faixa | o quê |
|---|---|
| `0`–`5` | a **base**: peso, cor (âncora `1`), rugosidade da difusa, metal |
| `6`–`11` | o **realce**: peso, cor (âncora `7`), rugosidade, IOR |
| `12`–`18` | o **verniz**: peso, cor (âncora `13`), rugosidade, IOR, escurecimento |
| `19`–`22` | o **brilho próprio**: luminância, cor (âncora `20`) |

⭐ **É agora ou nunca:** *uma ordem de chegada é permanente no dia em que a lista fecha* — e esta
fechou. São as `15` entradas do OpenPBR, e não há mais nenhuma para apender.

⛔⛔ **E a re-numeração tem um modo de falha MUDO que mordeu TRÊS vezes:** uma lista de índices
escrita à mão (`for peso in [5u8, 9]`, `for k in [3u8, 4]`) sobrevive a uma re-numeração **sem erro de
compilação** e passa a medir outra coisa. No `polygon_rows_tests` ela punha o **metal** a `1` — que
*esconde* duas linhas em vez de abrir quatro —, e quem a apanhou foi o gate do outro lado, a ler `18`
onde esperava `25`. *A cura é contar do produto, nunca de uma lista ao lado dele.*

---

### §22.3 — ⭐⭐⭐ E o `surface_of` deixou de poder esquecer um campo

O literal do `OpenPbr` passou a ser **exaustivo** — sem `..Default::default()`. ⇒ apagar uma linha
dele é **erro de compilação**, e as cinco mutações da wave tiveram de mudar de forma: em vez de
apagar o campo, elas **congelam-no no neutro**, que é a forma que o defeito teria de tomar para
chegar ao main.

⭐ *Uma lei que o compilador prende não precisa de gate* — e o gate que fica
(`every_number_a_material_has_reaches_the_law`) guarda a direcção **oposta**: um número novo no
`FieldMaterial` que ninguém ligue.

---

### §22.4 — ✅ A CERCA que a §8 pediu existe, e o mecanismo dela era outro

A §8 mediu que o multi-scatter da difusa tem um **polo** (`base_color ≈ 5,981` com
`base_diffuse_roughness = 1`; a `6,0` a indirecta devolve `[−676, −716, −813]`) e escreveu a cerca:
*«a porta que deixar autorar `base_color` coage a `0..1`»*. Com a rugosidade da difusa a tornar-se
autorável, **este é o dia** — e o gate existe agora
(`no_material_a_gesture_can_produce_returns_negative_light`): `20 000` materiais de um LCG de semente
fixa, cada número na faixa que o slider oferece, e nenhum devolve luz negativa ou não-finita.

⭐ **Medido ao escrevê-lo: o polo continua INALCANÇÁVEL, e não pela razão que eu esperava.** O
suspeito era o `base_weight`, que multiplica a cor base e **tem campo numérico aberto** — mas ele
escala a indirecta **linearmente** (`1 → 8` dá `0,36 → 2,80`, sempre positiva). O polo exige a **cor**
acima de `~6`, e a única porta que a escreve é o selector, que fala `sRGB8`.

⛔ **E a prova de que a varredura não é fraca é uma mutação na própria CERCA:** levantar a coerção a
`0..1` põe o gate vermelho com `[15,2, −1,0, 3,6]`. *Um gate de ausência tem de mostrar que alcança a
presença.*

⚠️ **Achado de lado:** a `base_diffuse_roughness` **não toca a luz do céu** — as três amostras
(`0`, `0,5`, `1`) dão a indirecta bit a bit igual. Ela é um termo da luz **directa**, e um gate que só
a medisse na indirecta estaria a olhar para o lado.

---

### §22.5 — O teto de linhas e o arquivo

- **`MAX_ROWS` `74 → 79`** (`2 × 27 + 25`), medido no produtor das linhas e no pior estado — brilho e
  verniz acesos, **com o metal abaixo de `1`**, que é quando as duas linhas só-dieléctricas aparecem.
  ⭐ E o material **fechou**: este teto deixa de crescer por aí.
- **`PROJECT_SCHEMA` `130 → 131`** — ⚠️ e aqui não é só apendar: os campos foram **re-ordenados**, o
  que põe a rugosidade onde morava o peso da base. ⛔ Conte o DELTA (`+3` desde o `main`).

---

### §22.6 — Os gates (4) e as mutações (10/10)

| gate | o que ele prende |
|---|---|
| `the_last_five_openpbr_inputs_reach_the_law_and_move_the_answer` | as cinco atravessam **e** movem a radiância — o peso do realce medido no **metal** |
| `the_two_dielectric_only_numbers_are_exactly_inert_on_a_metal` | os **mesmos bits** num metal, **e** a linha a sumir — só ela, e só ali |
| `the_surface_ior_has_the_same_physical_range_as_the_coats` | os dois IOR têm a mesma cerca física |
| `no_material_a_gesture_can_produce_returns_negative_light` | o polo do multi-scatter, sobre `20 000` materiais |

⚠️⚠️ **Uma mutação SOBREVIVEU à primeira redacção, e o buraco era do gate:** mover a âncora da cor do
realce de `7` para `6` deixava a linha do `specular_weight` com o **rótulo** da cor, e o gate — que
comparava só as chaves — continuava verde. *Um gate que lê só o rótulo não sabe sobre que número ele
está escrito.* Hoje ele compara o par `(param, chave)`.

---

### §22.7 — ⏳ O que fica aberto

- ⭐ **Do material, NADA.** As `15` entradas do OpenPBR são autoráveis, e o que a fatia do port deixou
  de fora (`transmission_*`, `subsurface_*`, `fuzz_*`, `thin_film_*`, `geometry_opacity`, as
  anisotropias) **não existe na crate** — são ausências declaradas com motivo, e não knobs por
  construir.
- **A ALFA** continua a ser a única coisa que o §12.7 pedia e ninguém entregou: ela é a
  `geometry_opacity`, está fora do port, e num traçador exige que a marcha continue **através** da
  peça. *É uma wave de motor, não de painel.*
- **O painel tem `10` linhas de material com tudo apagado**, e `15` com tudo aceso. ⏸️ **Se isso for
  denso demais é veredito do dono** — a cura conhecida são **secções recolhíveis**, que este painel
  não tem e que o Motion já nomeou como a resposta ao mesmo problema.

---

## §23 — ⛔⛔ UM CONTROLO INERTE NÃO DESAPARECE: ELE FICA TRAVADO (ordem do dono, 2026-09-14)

Enio, depois do smoke da §22: *«os slideres que só aparecem sob uma condição específica não devem
desaparecer, mas apenas serem inativados, mas sempre visíveis»*.

---

### §23.1 — ⛔⛔ A lei já estava escrita nesta casa, e eu apliquei a OUTRA

As três waves do material esconderam linhas inertes invocando a **W34** — *o painel oferece
exactamente o que o gesto faz*. Ela proíbe **pintar um controlo** que não pode ser honrado. ⚠️ **Ela
não manda apagar a linha** — e a lei que responde a *o que fazer então* estava escrita, por extenso,
no `ph2d_field::Span::Locked`, desde a trava de cardan:

> *«É diferente de "não aparece". O valor continua a ser um facto que o artista precisa de ler — e
> esconder a linha faria o painel saltar de tamanho a cada travessia. O que ela perde é o
> **controle**: quem a recebe pinta um facto, não um slider.»*

⇒ **eu tinha o mecanismo certo à mão, com o motivo certo escrito ao lado, e usei o outro.** *Duas
leis que se leem parecidas, e a diferença entre elas é o painel a saltar debaixo do dedo.*

---

### §23.2 — O que muda

| estado | antes | agora |
|---|---|---|
| a cor do brilho, sem brilho | some | **visível, travada** |
| os quatro do verniz, sem verniz | somem | **visíveis, travados** |
| a rugosidade da difusa e o IOR, num metal | somem | **visíveis, travados** |

⭐⭐⭐ **E a contagem de linhas de um nó deixou de depender do ESTADO da peça:** um polígono no teto
pede `79` linhas com tudo apagado e `79` com tudo aceso. *O painel não muda de altura quando o
artista acende o verniz*, que é literalmente o que a nota do `Span::Locked` previa.

⚠️ **O `MAX_ROWS` não se mexe** (`79`): ele já estava dimensionado pelo pior caso, e o pior caso
passou a ser **o único** caso.

---

### §23.3 — ⭐⭐ Uma AMOSTRA travada continua a ser uma amostra

O despacho de uma linha testava `live` **antes** do ramo da cor, logo uma amostra travada caía no
`paint_fact` e o artista via um **número** (`0,8`) onde estava uma cor. ⇒ *a linha deixava de saltar
de sítio e passava a saltar de ESPÉCIE, que é a mesma queixa noutra escala.*

Hoje o ramo da cor vem primeiro, e uma amostra travada pinta-se **apagada**
(`SwatchState::Disabled`). ⛔ **As três metades saem juntas, e cada uma sozinha é um defeito
diferente:** sem o `register_picker_swatch` o selector não a reconhece · sem o `hit_index` ela não é
clicável · sem o `Disabled` ela **parece** clicável. *Uma amostra que parece viva e não responde é o
controlo morto na forma que o artista mais depressa lê como avaria.*

---

### §23.4 — Os gates, e as duas coisas que o arnês ensinou

| gate | o que ele prende |
|---|---|
| `the_coat_numbers_are_locked_while_the_coat_is_off` | `15` linhas em qualquer estado, e **quais** estão vivas |
| `the_colour_of_the_glow_is_locked_while_the_glow_is_off` | as quatro amostras, e o `live` de cada uma |
| `the_two_dielectric_only_numbers_are_exactly_inert_on_a_metal` | a lista **não muda de tamanho**; muda o `live` |
| `every_row_of_the_biggest_polygon_fits_the_registered_family` | ⭐ **a contagem é a mesma nos dois estados** |
| `a_locked_swatch_is_painted_and_unreachable` | a costura: sem rect no índice de acerto, **e** desenhada |

⭐ **O `tudo_aceso` da sonda do teto mudou de papel:** era o *pior caso*, e é hoje o **controlo** da
lei. *Sem essa asserção ele seria um knob morto dentro da própria bancada que caça knobs mortos.*

⚠️⚠️ **E a primeira redacção do gate da costura tinha uma TAUTOLOGIA:** ela repetia a asserção
anterior com o sinal trocado e chamava-lhe *«continua a ser desenhada»*. *Uma asserção que reafirma a
anterior mede zero e lê-se como cobertura* — é o mesmo defeito que o gate do L-System pagou ao medir
a linha **reservada** em vez da pintada. A régua certa é a **geometria da picture**, contra o controlo
de pintar sem a linha.

**7/7 mutações** — entre elas a que faz uma linha travada voltar a **esconder-se**, que é a que
guarda a ordem do dono.
