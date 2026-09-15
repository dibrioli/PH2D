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
| ~~**o rig do DOCUMENTO**~~ | ⭐ **FECHOU em 14/09 por ordem do dono, e por outra porta**: o modelador não foi buscar o rig da escultura — a luz passou a ser um **objecto da cena** (§25) |
| **anisotropia, transmissão, subsuperfície, fuzz, película fina** | cada uma é uma closure com gate próprio; com peso zero a composição gerada dá-lhes contribuição **zero**, e os campos **não existem** na struct (um campo que a lei não lê é um controlo morto) |
| **AgX** | ⛔ **licença**: não há neste disco um AgX cuja licença permissiva se leia no artefacto (`04` §3). O oráculo dele corre-se à mesma |
| **exposição contínua** | os chips dão `±2` stops, que é o que o smoke do plano pede; um slider de vista pede um alcance medido e um sítio no painel |
| **sombras, oclusão, luz indirecta a sério** | são as `W4` e `W5` do [plano](03_o_plano.md) |
| ~~**o céu como fonte com forma**~~ | ⭐ **a `W3` abriu em 14/09** — ver §24; o céu continua analítico (sem mapa de ambiente) |

## §8 — ⏳ O que fica aberto

- ⛔⛔ **A peça sai com `8,4 %` em BRANCO CHAPADO no olhar de omissão** (§6) — não é um realce, é um
  planalto sem forma. ⚠️ **É decisão do dono**, e o cálculo está fechado: na `Standard` a peça só
  deixa de cortar a `−1` stop (`118` px, `0,2 %`); a `Neutral` não corta em exposição nenhuma. As
  duas saídas são *a vista de omissão passa a `Neutral`* ou *a exposição de omissão desce um stop*.
  ✅ **RESOLVIDO na §15 do mesmo dia** (13/09 → 14/09): o `OPENING_LOOK` do modelador é `Neutral`,
  com a tabela do que isso custa ao matcap ao lado. ⚠️⚠️ **Este parágrafo ficou por corrigir e
  custou caro:** em 14/09 a §24 leu-o como decisão em aberto e pediu ao dono que decidisse uma coisa
  que ele já tinha decidido nessa manhã. *O §8 é uma lista de trabalho pago — audite-a contra o
  CÓDIGO antes de pegar um item dela*, que é o aviso que este próprio ficheiro já dá sobre si mesmo.
  ⚠️ E a medição da §24.7 continua a valer, noutro papel: ela mostra **porque** é que aquela escolha
  era load-bearing — o estúdio leva o corte de `5 180` para `7 972` px na `Standard`, e a `Neutral`
  corta `0` nas vinte configurações medidas.
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

---

### §23.5 — ⛔⛔ E o smoke seguinte veio com FOTO: *«widgets sobrepostos embolados, mas espaçados»*

Três linhas do material colidiam com as de baixo, e duas coisas estavam erradas ao mesmo tempo.

**(a) O pintor do rótulo QUEBRAVA.** O doc do `paint_text_elided` nomeia o defeito à letra:

> *«`paint_text` trata `max_width` como orçamento de **quebra**, então um rótulo um pixel largo
> demais vira duas linhas em silêncio e transborda para a linha de baixo.»*

As linhas deste painel avançam um **passo fixo**, logo a segunda linha do rótulo caía **por cima** da
seguinte. ⚠️ **Ele dormiu três waves:** com `Radius` e `Round` nada quebrava; `Coat Roughness`,
`Specular Color` e `Emission Color` — os rótulos que o material completo trouxe — passaram a colidir
de uma vez. *Um pintor errado só se vê quando o conteúdo cresce, e o conteúdo cresce numa wave que
não olha para o pintor.*

**(b) A linha travada era a ÚNICA do painel ainda desenhada com o modelo de TRÊS COLUNAS.** A linha
viva perdeu a coluna externa de rótulo em 2026-09-02 — *«a caixa única: rótulo à esquerda DENTRO,
valor à direita DENTRO»* —, e o `label_w` daquele pintor é **ignorado de propósito** desde então. O
`LABEL_COL_W = 72` deste painel sobreviveu só nas linhas que eu escrevi: o facto, a escolha e as duas
amostras. ⇒ *a linha inactiva não era uma linha viva a cinzento; era outra coisa.*

⇒ **as quatro passam à mesma geometria:** rótulo à esquerda (**cortado**, nunca quebrado), valor ou
amostra na **coluna da direita**.

⚠️ **A amostra encolheu, e isso é visível:** ela ocupava a goteira inteira (`w − 72 ≈ 230 px`) e passa
a ocupar a coluna de `72`. *Uma amostra tão larga lê-se como um campo de texto, e a coluna dos valores
deixava de estar alinhada com a dos números* — ⏸️ mas é uma mudança de aparência sobre algo que o dono
já tinha aprovado, e fica à espera do veredito dele.

⚠️⚠️ **E o gate é TEXTUAL, com o motivo escrito:** a propriedade é *«a que altura o rótulo foi
pintado»*, e o arnês conta glifos — um rótulo quebrado tem **os mesmos glifos** do que cabe. Não há
régua de geometria que os separe, então o que resta é proibir o pintor **pelo nome**. ⛔ Ele salta os
comentários (este ficheiro cita o nome proibido em prosa), lê o ficheiro por `include_str!` (que falha
a **compilar** se ele mudar de sítio) e tem **controlo do próprio censo**: o ficheiro tem de conter o
pintor que corta, senão uma varredura partida leria `0` culpadas e passaria.

---

## §24 — ⭐⭐⭐ O CÉU GANHA UMA FONTE: o estúdio (2026-09-14)

O material fechou na §22 com as `15` entradas do OpenPBR autoráveis. A `W3` do
[plano](03_o_plano.md) diz que o passo seguinte é *«o céu como FONTE de luz … é o que faz o metal
existir»*. **Antes de construir, a premissa foi medida** — e ela veio mais afiada do que a frase do
plano.

---

### §24.1 — ⛔⛔ A premissa, em bytes: um ESPELHO e um pedaço de GIZ tinham o mesmo contraste

Sonda [`measure_how_much_of_the_material_this_sky_lets_through`](../../crates/ph2d-app-field3d/src/render_light_tests.rs),
esfera a `640×360`, `61 804` pixels de peça, **no olhar com que o modelador ABRE** (`Neutral`, §15),
tudo em bytes (logo independente do relógio e da carga):

| material | média | estrutura `\|∇²\|` do verde | quanto vem da LÂMPADA |
|---|---:|---:|---:|
| metal, rugosidade `0,05` (espelho) | `144,5` | **`1,077`** | **`1` de `144`** |
| metal, rugosidade `0,30` | `154,9` | `1,116` | `32` de `155` |
| metal, rugosidade `0,50` | `171,5` | `1,191` | — |
| metal, rugosidade `1,00` (baço) | `200,4` | **`0,930`** | `140` de `200` |
| dieléctrico, rugosidade `0,30` | `199,0` | `0,984` | `135` de `199` |

⭐⭐⭐ **A coluna do meio não diz de que material a peça é feita.** Ela vive toda numa banda de
`0,93`–`1,19`, e **o espelho está no fundo dela** — um metal *quase baço* tem MAIS contraste local do
que um cromado. A razão é dupla: a rampa `A + B·y` filtrada por um lóbulo largo **continua a ser uma
rampa** (não há nada para reflectir), e uma luz direccional é um **delta**, que um espelho reflecte
num conjunto de medida nula — **`1` byte de `144`**. *O cromado deste app era a peça com menos forma
de todas; o que dava contraste a um metal era a rugosidade dele deixar a lâmpada aparecer.*

⚠️ **E o controlo que o dono mais usa era o mais invisível.** Num dieléctrico, arrastar o `Roughness`
de `0,30` a `0,05` movia `1 048` de `61 804` pixels (**`1,70 %`**), com `\|Δ\|` médio de **`0,60`
bytes** sobre a peça inteira — e a §22 tinha acabado de lhe dar mais nove controlos para arrastar.

⚠️⚠️ **A régua da estrutura tem CONTROLO, e sem ele ela não afirma nada:** um `\|∇²\|` que lê `~1` em
tudo é indistinguível de uma régua **cega**. Sob um céu sintético com **aresta** ela lê `2,699` no
espelho (`2,5×`), `1,716` no metal baço e `1,248` no dieléctrico. *Ela vê, vê mais no espelho, e é
isso que prova que o `1,077` de cima é uma afirmação sobre o CÉU e não sobre a régua.*

---

### §24.2 — ⛔⛔ A recusa medida que existe sobre isto respondeu a OUTRA pergunta

O [`ph2d_light::ENV_SLOPE`](../../crates/ph2d-light/src/lib.rs) carrega, por extenso:

> *«um ambiente de três zonas (céu, horizonte claro, chão) — a forma de um HDRI de estúdio — dá
> contraste cima/baixo de **1,83×** contra os **2,20×** deste, precisa de um terceiro coeficiente e
> ainda deixa **0,0042** de resíduo. O ambiente mais rico mede pior justamente na coisa para a qual
> o termo existe.»*

⚠️ **Ela foi medida sobre a IRRADIÂNCIA** — quão bem o modelo reproduz a luz que uma **difusa**
recebe — e sobre isso continua a valer inteira. *Uma recusa medida responde UMA pergunta*
(`CLAUDE.md` §5.0): nada nela mede o que um **espelho** vê, e a rampa ganha aquela comparação por ser
lisa **exactamente pela razão** por que perde esta.

⇒ **a rampa fica, com a lei dela intacta e byte a byte** (gate `with_no_box_the_sky_is_the_one_it_replaces`,
sobre `3 000` direcções × `5` rugosidades), e o que se acrescenta é uma **forma** por cima.

---

### §24.3 — A lei, em duas decisões que não são números escolhidos

O estúdio é `rampa + caixa de luz`, em [`ph2d_app_field3d::studio`](../../crates/ph2d-app-field3d/src/studio.rs).

1. ⭐ **O EIXO da caixa é o eixo da PRÓPRIA RAMPA** (`+y` em espaço de vista). O céu já é claro em
   cima e escuro em baixo; a caixa **afia** esse eixo em vez de trazer uma direcção nova. ⇒ **zero**
   constantes de direcção. E ela é da **cor média do próprio céu** (`ENV_BASE`) ⇒ **zero** constantes
   de cor.
2. ⭐ **A ENERGIA sai do ambiente, não se soma a ele** — a caixa leva uma fracção `f` da energia total
   do céu e o termo constante desce `f`. A radiância média sobre a esfera fica **igual** (gate, sobre
   `150 000` direcções, nas duas perguntas do `Environment`), e **o chão escurece de graça**, porque a
   energia dele é que subiu.

⚠️ Nada disto toca a [`ph2d-light`](../../crates/ph2d-light/): **zero linhas** naquela crate, logo a
tinta e a escultura não podem ser afectadas por esta wave.

---

### §24.4 — ⭐⭐⭐ O pré-filtro é uma TABELA, e a forma fechada foi construída, medida e deitada fora

O `Environment` pede a radiância **já pré-filtrada** para um lóbulo GGX de rugosidade `α`. Para a
rampa isso é exacto e fechado (o `lobe_shrink` da §16); para uma calote, não é.

**O caminho elegante foi percorrido inteiro.** Uma gaussiana esférica (SG) convolvida com outra fecha
em álgebra (o produto de duas SG é uma SG), e o `λ` do núcleo pode ser escolhido para ter
**exactamente o primeiro momento** do pré-filtro GGX — porque `lobe_shrink` **é** esse momento
(`Σ(N·L)²/Σ(N·L) = E[ω·R]`) e uma SG tem o dela na função de **Langevin** `L(λ) = coth λ − 1/λ`. Com
`λ(α) = L⁻¹(lobe_shrink(α))` as duas metades do céu concordam em **toda função linear**.

| medição | resultado |
|---|---|
| a **álgebra** (produto de duas SG) contra a quadratura, `200 000` direcções | **`2,7e-6`** ✅ |
| o **MODELO** (núcleo SG) contra o pré-filtro GGX de verdade | **`5 %` a `28 %`** na faixa útil ⛔ |
| o **melhor `λ` possível**, por varredura de `400` pontos em `5` ordens de grandeza | ainda **`44,8 %`** a `α = 0,1` ⛔ |

⛔⛔ **Nenhum `λ` cura, e a razão é estrutural: a cauda do GGX é pesada e a de uma gaussiana não é.**
⚠️ E o pior pedaço é onde o material de omissão vive: **`α` é o QUADRADO da rugosidade**
(`isotropic_alpha`), logo a rugosidade de omissão `0,3` cai em `α = 0,09`.

⇒ **a tabela**, construída pela **definição** que o `mx_environment_prefilter` escreve (`N = V = R`,
amostragem por importância, peso `N·L`):

```text
m(α, ψ) = Σ (N·L) · caixa(ω_L) / Σ (N·L)
```

⭐ *Não há aproximação para declarar: a lei do produto e o oráculo dela são a mesma conta, e o gate
mede só a RESOLUÇÃO* (a tabela contra uma quadratura `390×` mais densa, em pontos que caem **entre**
as células).

⚠️ **Os dois eixos são deformados, cada um pela sua razão:** o da rugosidade é `√α` (a rugosidade que
o artista arrasta) e o do ângulo é `√(1 − cos ψ)`, que é proporcional a `ψ` junto do pico — o passo
sai `~0,22°` em toda a faixa. *Uma tabela uniforme em `cos ψ` teria `10°` de passo onde a caixa tem
`25°` de raio, e seria uma calote de seis degraus.* ⛔ E nenhum dos dois eixos precisa de um `acos` em
tempo de execução: só de dois `sqrt`.

---

### §24.5 — ⭐⭐⭐ E a FORMA certa não é a gaussiana: é o DISCO. A aresta é o efeito.

Com a tabela a fazer o pré-filtro, **a forma da caixa passou a ser livre** — e a medição escolheu
outra. Estrutura no espelho, sob a mesma energia:

| forma | estrutura `\|∇²\|` | ganho |
|---|---:|---:|
| rampa nua | `1,074` | — |
| **gaussiana esférica**, na melhor célula dela | `1,171` | `+9 %` |
| **disco**, na MESMA célula | `1,337` | `+24 %` |
| **disco**, na melhor célula dele | `1,478` | **`+38 %`** |

*(⚠️ estas três foram medidas na vista `Standard`, **antes** de a régua se mudar para o olhar do
produto — §24.7. A troca move a rampa de `1,074` para `1,077`, isto é, nada: o que ela move é o
extremo alto, onde o corte começa a comer a estrutura. A configuração escolhida lê `1,077 → 1,268`
no olhar do produto.)*

⭐⭐⭐ *Uma gaussiana não tem aresta nenhuma, e a aresta é o efeito.* Uma caixa de luz real é um
rectângulo de bordo nítido, e é isso que um cromado mostra. ⇒ o argumento inteiro a favor da SG — que
ela pré-filtra em forma fechada — tinha deixado de valer duas secções antes.

⚠️⚠️ **E a LARGURA do bordo não é um gosto: é a resolução da tabela.** Interpolar um `smoothstep` de
largura `W` com passo `h` erra `0,75·(h/W)²`; com `h ≈ 0,22°` e a barra do gate sai `W ≥ 5,8°`. A
primeira redacção pôs `3°` *«≈ 3 células»* e o gate **reprovou com `4,95e-2`** — vinte e cinco vezes a
barra, e sempre no bordo. *Uma aresta mais dura do que a tabela representa não é uma aresta: é o
degrau da tabela a passar por uma.* ⇒ `SOFTBOX_RIM_DEG = 6,0`.

---

### §24.6 — De onde saem os dois números da caixa

Varredura de `20` configurações contra as réguas do **produto**
([`measure_which_softbox_the_rulers_choose`](../../crates/ph2d-app-field3d/src/studio_tests.rs)),
medidas na vista que **não corta** (ver §24.7). `mérito` = quanto a caixa move um **espelho** a
dividir por quanto ela move **giz**:

| raio | `f` | estrut. espelho | moveu o espelho | moveu o giz | moveu a OMISSÃO | mérito | branco `Std 0` |
|---:|---:|---:|---:|---:|---:|---:|---:|
| — | — | `1,077` | — | — | — | — | `5 180` |
| `12°` | `0,20` | `1,240` | `19,5` B | `7,8` B | `8,4` B | `2,49` | `7 995` |
| `18°` | `0,20` | `1,263` | `20,1` B | `7,7` B | `8,2` B | `2,63` | `8 035` |
| **`25°`** | **`0,20`** | **`1,268`** | **`21,0` B** | **`7,4` B** | **`7,9` B** | `2,85` | `7 972` |
| `35°` | `0,20` | `1,248` | `22,4` B | `6,9` B | `7,4` B | `3,26` | `7 748` |
| `25°` | `0,35` | `1,285` | `37,0` B | `13,8` B | `14,6` B | `2,69` | `9 268` |
| `25°` | `0,50` | `1,361` | `55,0` B | `21,4` B | `22,4` B | `2,57` | `10 164` |
| `25°` | `0,65` | `1,408` | `74,4` B | `29,9` B | `30,6` B | `2,49` | `10 815` |

⭐ **A FRACÇÃO é decidida pela peça que o dono já aprovou:** ela tem de continuar a parecer-se com ela
própria. A `f = 0,20` o material de omissão move-se **`7,9` bytes** (`3 %`); a `0,35` move-se `14,6`
e a `0,65` move-se `30,6`. ⇒ `SOFTBOX_SHARE = 0,20`.

⭐ **O RAIO é decidido pela estrutura**, que é a razão de a caixa existir, e ela tem o máximo em `25°`
— a `12°` a caixa toca poucos pixels e a `50°` ela quase não tem bordo. ⇒ `SOFTBOX_RADIUS_DEG = 25`.

⭐ **Os dois juntos fazem a caixa `5,4×` mais clara do que o céu à volta dela** (`20 %` da energia em
`4,7 %` da esfera). ⚠️ Um estúdio real é muito mais contrastado do que isto; o que prende o número
aqui é que a energia **sai do ambiente**, logo uma caixa muito clara é uma sala muito escura — e a
sala é o que ilumina a difusa.

---

### §24.7 — ⛔⛔ DUAS afirmações minhas, derrubadas — e a segunda foi o dono que a derrubou

**(a) O branco chapado SOBE.** O cabeçalho do módulo dizia, quando foi escrito: *«a média fica igual,
logo o `8,4 %` de branco chapado do §8 não pode piorar por acumulação»*. **Falso.** Média constante é
uma afirmação sobre a **MÉDIA**; o corte é uma afirmação sobre o **PICO**, e concentrar energia é
precisamente subir o pico.

| vista | branco chapado, rampa nua | com a caixa |
|---|---:|---:|
| `Standard`, `0` stops | `5 180` | **`7 972`** |
| `Standard`, `−1` stop | `114` | `115` |
| **`Neutral`**, `0` stops | **`0`** | **`0`** |

⛔⛔⛔ **(b) E a linha que interessa é a última, porque é onde o modelador ABRE.** A primeira redacção
desta secção dizia *«a escolha da omissão continua a ser do dono, e esta wave torna-a urgente»*, e o
smoke que eu escrevi pedia-lhe que decidisse. **Ele já tinha decidido, nessa manhã, e a decisão já
estava shipada:** o [`shading::OPENING_LOOK`](../../crates/ph2d-app-field3d/src/shading.rs) é
`Neutral` desde a §15, com a tabela do preço ao lado dela. A resposta dele foi *«Neutral como padrão
(como já está)»*.

⚠️⚠️ **O mecanismo do erro é o do `CLAUDE.md` §5.0, e é reproduzível:** as sondas desta wave pintavam
com `ph2d_view_transform::Look::default()` — a omissão do **TIPO**, que é `Standard` porque ela é a
identidade para luz em `0..=1` — e eu li a coluna delas como sendo *o produto*. *Uma sonda que chama
o default do tipo mede outro programa que o pill.* ⇒ as sondas passam a pintar com o `OPENING_LOOK`,
e a coluna `Standard` fica como o **outro lado do A/B** (o que vê quem trocar a vista na fileira do
*Shading*), nunca como a omissão.

⚠️ **E havia gate a mais e a menos ao mesmo tempo.** O `shading_tests` já proibia o `view.rs` de
voltar ao `Look::default()`, logo a DECISÃO estava presa — mas **nada ligava essa decisão ao que ela
protege**. *Uma catraca sobre um valor não diz porque é que o valor importa, e foi por isso que eu a
pude ler como aberta.* ⇒ gate novo, sobre a consequência:
`the_modeller_opens_in_a_view_the_softbox_does_not_blow_out`.

⚠️ **E é por isso que as constantes da §24.6 foram escolhidas no olhar do produto:** sob a `Standard`
um planalto saturado tem `∇² = 0`, logo o corte **esconde** o efeito que a régua devia medir. *A régua
estaria a ser lida através do defeito que ela acusa.*

---

### §24.8 — ⏱️ O preço

| | medido |
|---|---|
| construir a tabela (`49 × 513`, `512` amostras/célula) | **`27,6 ms`**, uma vez, no 1.º quadro de Render |
| o sombreamento, rampa nua → com a caixa | `1,634` → `1,737 ms` (**`+0,103`**, `+6,3 %`) |
| sobre um quadro de `16,7 ms` | **`+0,6 %`** |

*(`--release`, mínimo de `5`/`7` corridas, `load 2,2–3,0`, com o `/proc/loadavg` impresso ao lado —
`CLAUDE.md` §5.0. ⚠️ **As medianas do sombreamento sobrepõem-se** (`1,782` contra `1,751`): a `0,1 ms`
o efeito só é legível no mínimo.)*

⚠️⚠️ **A construção custava `110 ms` e a cura foi uma CONSTANTE DOBRADA DENTRO DO LAÇO:** a forma
resolvia `to_radians().cos()` **duas vezes** em cada uma das `12,9` milhões de avaliações. Hoje as
duas fronteiras resolvem-se uma vez por tabela (`Profile`) e a construção custa **`4×` menos**. *É a
mesma forma que a memória [`a_constant_folded_into_a_tree`](../../project-memory/feedback_a_constant_folded_into_a_tree_is_recomputed_wherever_the_tree_is.md)
já regista, noutro subsistema.*

⭐⭐ **E a contagem de amostras saiu da única régua que o dono vê: PIXELS.** Contra uma tabela de
`8 192` amostras, a de `512` devolve `\|Δ\|` médio de **`0,041` byte** e pior caso `3` bytes sobre oito
materiais. A régua intermédia dizia outra coisa — `7,4e-3` de desvio contra a definição, **vinte e
cinco vezes** o que a primeira redacção da barra pedia — e subir para `2 048` compra `0,034` de um
byte por `4×` o preço da construção. ⇒ a barra do gate é `1e-2`, **com um controlo ao lado** (uma
caixa `10°` maior sai a `0,1+`), senão uma barra larga não afirmaria nada.

---

### §24.9 — Os gates (6 + 1 partido ao meio) e as mutações (7/7)

| gate | o que ele prende |
|---|---|
| `the_table_is_the_prefilter_it_claims_to_be` | a tabela contra a **definição**, `390×` mais densa, entre células — **com controlo de forma** |
| `the_diffuse_table_is_the_cosine_convolution` | a outra pergunta do `Environment`, contra a própria definição (`9,8e-7`) |
| `a_box_as_wide_as_the_sky_is_the_sky` | ⭐ uma caixa de `180°` devolve o ambiente — é o que mantém o *furnace test* de pé sem uma linha nova |
| `with_no_box_the_sky_is_the_one_it_replaces` | **ao BIT**, nas duas perguntas, sobre `3 000` direcções |
| `the_box_redistributes_the_sky_it_does_not_add_to_it` | a média sobre a esfera não se mexe, na radiância **e** na irradiância |
| `the_box_brightens_the_zenith_and_darkens_the_floor` | o zénite sobe, o chão desce — a metade que diz *de onde* veio a energia |
| `the_modeller_opens_in_a_view_the_softbox_does_not_blow_out` | ⭐ a CONSEQUÊNCIA do `OPENING_LOOK`: `0` pixels saturados no caminho do produto, com o controlo na `Standard` |

⚠️⚠️ **E o `the_sky_honours_the_lobe_width_it_is_handed` PARTIU-SE, de propósito, e a forma como
partiu é o achado.** A redacção anterior media a lei do lóbulo **contra a fórmula da rampa** e
afirmava, no equador, a **invariância** em `α`. Com uma caixa em `+y`, um lóbulo largo avaliado no
equador **arrasta a caixa para dentro da média** ⇒ aquela igualdade passou a ser falsa — e passou a
sê-lo porque a lei ficou **mais forte**: hoje o `α` entra por **dois** caminhos independentes (o
encolhimento da rampa e a linha da tabela). ⇒ o gate parte-se em duas metades, **cada uma sobre o céu
de que ela fala**: a lei exacta sobre a rampa nua, a dependência sobre o céu do produto — e a metade
nova afirma o **contrário** da antiga no equador.

⚠️ **E o portão do fecho apanhou um TECTO DE LOC** (`studio_tests.rs` a `715` contra `700`), curado
por **corte por responsabilidade** e nunca por uma entrada no `FILE_OVERAGE_OK`. ⭐ O corte caiu no
sítio certo por si mesmo: um **gate** afirma uma lei e uma **sonda** imprime uma tabela e não afirma
nada — são dois leitores diferentes (o portão de fecho, e quem está a escolher uma constante). ⇒
`studio_probe_tests.rs`.

⚠️ **O oráculo da forma é RE-ESCRITO no gate, de propósito:** se ele chamasse a `Softbox` do produto,
uma mutação na forma passaria pelos dois lados ao mesmo tempo. É essa separação que mata a mutação
`[2]`.

**9/9 mutações** — a caixa a somar-se em vez de sair do ambiente · a linha da tabela fixa (o `α`
deitado fora, que mata em dois ficheiros diferentes) · o bordo a virar degrau · a irradiância a
ignorar a caixa · a tabela sem normalização · o eixo do ângulo sem deformação · o `up = dir[1]` · o
`OPENING_LOOK` a voltar a `Standard` · **o produto a perder a caixa**.
⚠️ Com **controlo do próprio filtro** antes de mutar (`6 passed` limpo), senão um filtro que casa zero
imprime o mesmo que um gate que morre.

⚠️⚠️ **E a última delas SOBREVIVEU à primeira redacção do gate novo, o que o corrigiu.** O controlo
pedia apenas *«na `Standard` isto satura»* — e a **rampa nua já satura `5 180` px sozinha**, logo
apagar a caixa do produto deixava o gate verde. *Um controlo que o sujeito da mutação não toca não é
um controlo: é uma segunda asserção sobre outra coisa.* ⇒ hoje o controlo é o A/B da própria caixa
(numa vista que corta, ela tem de **acrescentar** corte), e as duas metades dependem dela.

---

### §24.10 — ⏳ O que fica aberto

- ✅ ~~**A vista de omissão**~~ — **já estava decidida e shipada** (§15), e esta wave só descobriu que
  não sabia disso (§24.7). Hoje tem gate sobre a consequência.
- **A caixa é UMA, e no eixo da rampa.** Uma segunda caixa (o preenchimento, o `rim`) custa **uma
  segunda tabela** e uma direcção que já não sai da rampa — isto é, a primeira constante de direcção
  desta lei. *Não é trabalho a fazer: é uma decisão a tomar primeiro.*
- **O tamanho da caixa não é autorável**, e o preço está nomeado: a tabela é construída para UMA
  forma, logo um raio autorado é um **terceiro eixo** nela (ou uma reconstrução de `27,6 ms` por
  mexida no slider).
- ✅ ~~**O rig continua a ser o de omissão**~~ — **fechou no mesmo dia (§25)**: a luz virou objecto da
  cena, por ordem do dono. ⚠️ E abriu a incoerência que a §25.8 nomeia: o céu é de ecrã e as luzes são
  de mundo.
- ⏳ O que a `W3` do plano ainda não faz: o céu continua **analítico** (não há mapa de ambiente), e a
  `W4`/`W5` (sombras e luz indirecta) não começaram.

---

## §25 — ⭐⭐⭐ A LUZ É UM OBJECTO DA CENA (ordem do dono, 2026-09-14)

Enio, depois da §24: *«A luz deve virar objeto 3d como nos app 3d. Pode ser a luz. O Painel model
pode receber uma seção para a luz. Isso é provisório até finalizarmos os modos de arte do app.»*

---

### §25.1 — A lei, numa frase

Uma luz é uma **entidade** com um [`ph2d_field_ecs::FieldLight`](../../crates/ph2d-field-ecs/src/lib.rs)
e uma pose. ⇒ ela aparece na Hierarquia, escolhe-se, renomeia-se, apaga-se, esconde-se e **move-se com
o mesmo gizmo** que move uma forma — tudo isso **sem uma linha de código próprio**, porque é o que ser
uma entidade desta cena já significa (a W5: *a hierarquia da cena É o documento*).

⭐ **O componente guarda só o que NÃO é pose** — a força e a cor. *Um campo de posição ali seria a
segunda resposta a «onde está isto», e a que diverge no dia em que alguém arrasta o gizmo.*

⛔ **E ela NÃO leva `FieldNode`**, o que a mantém fora de tudo o resto **de graça**: o cozimento, o
`leaves` dos materiais, o `owners` e o clique no canvas filtram todos por ele. *Uma luz que fosse um
nó apareceria como uma forma invisível no meio da peça.*

---

### §25.2 — ⚠️ Ela é um PONTO, e não um sol

A escolha segue da ordem. Um sol não tem posição — só rotação —, logo **arrastá-lo pela cena não faria
nada**, e um objecto que se move sem efeito é o controlo morto do `CLAUDE.md` §5.0 na forma mais cara.
Um ponto cai com `1/r²`, e é isso que faz **aproximá-lo da peça ser um gesto que se vê**.

⭐ **A unidade da intensidade é DERIVADA, não escolhida:** o rig da casa promete que *«uma superfície
plana de frente para uma luz de intensidade `1` devolve `1`»* ⇒ aqui a intensidade é **a mesma, medida
a UMA unidade de distância**. *Não há uma segunda escala para aprender.*

---

### §25.3 — ⛔⛔ O que ela SUBSTITUI, e o que isso custa à imagem

**O rig ancorado no ecrã deixou de acender o modo Render.** Ele continua a acender o **matcap** (que é
sombreamento de vista, por definição), a tinta e a escultura — outros módulos. *As luzes desta cena são
as luzes desta cena; uma cena sem luz nenhuma sai acesa só pelo céu.*

⭐⭐ **E a cena nasce com uma**, como em todo aplicativo 3D — senão o modo Render perdia a lâmpada e não
ganhava nenhuma. *Uma feature que exige um gesto antes de a cena voltar a parecer-se com ela própria
não é uma feature: é uma regressão com um botão ao lado.*

**Onde ela nasce é derivado de duas coisas que já existiam:**

| | de onde vem |
|---|---|
| **a direcção** | a do `Light::KEY` do rig (`230°`/`30°`, afinada pelo Enio em 2026-07-12), lida pela porta e convertida pela base da **câmera** |
| **a distância** | `2 × half_extent` — duas meias-larguras do que a câmera enquadra: fora da peça, e a um passo de zoom de estar à vista |
| **a força** | `r²` — com a queda `1/r²`, é o que entrega **no centro da peça exactamente o que a lâmpada do rig entregava** |

⚠️⚠️ **A primeira redacção do sítio era um `[f32; 3]` const em MUNDO**, e isso é a mesma classe de erro
que o sinal de `y` desta casa já pagou: *a direcção do rig é de ECRÃ, e onde «superior-esquerda» cai no
mundo depende de para onde a câmera olha.* Hoje é uma função da câmera, com gate nas duas metades.

**O preço, medido** (esfera do §6, olhar do produto):

| | média | branco chapado | `\|Δ\|` médio | máx |
|---|---:|---:|---:|---:|
| a lâmpada de ECRÃ (o de antes) | `181,9` | `0` | — | — |
| **a luz de ABERTURA** | `163,3` | `0` | **`21,4`** | `71` |
| só o céu (cena sem luz) | `128,9` | `0` | `52,5` | `108` |

⭐ **E ela NÃO reproduz a imagem antiga, nem pode:** uma luz direccional entrega a mesma radiância em
todo o lado; um ponto entrega `1/r²`, e o lado de lá da peça está mais longe. Afastá-la e reforçá-la
aproxima (a `5` unidades com força `25` o desvio cai a `6,7` bytes) — e a `5` unidades ela está **fora
do enquadramento**, que é o oposto do que a ordem pediu. *A troca é o preço de a luz passar a estar em
algum sítio.*

---

### §25.4 — O painel, e o que ele NÃO oferece

Uma luz escolhida traz a secção **Light**: a **posição** (que é a pose, pelo `Param::Pos` que já
existia) e, sob o cabeçalho próprio, a **Intensity** e a **Color**.

⛔ **A rotação e a escala não são oferecidas**, e é a metade que interessa: um ponto não tem orientação
nem tamanho, e três sliders de ângulo que não movem um pixel são o controlo morto que a **W34** proíbe
por escrito.

⭐ **A caixa de cor é a MESMA máquina da §12** — âncora no canal `R`, seguidores dobrados na amostra.
⚠️⚠️ **E o `ModelIntent::SetColor` teve de deixar de carregar um `u8`:** com um índice cru, a cor de uma
lâmpada (`Light(1)`) e a cor base de um material (`Material(1)`) viajam como o **mesmo `1`**, e quem
drena escolhe a família por conta própria. *Um índice sem família é um sujeito por adivinhar, e a
adivinha é silenciosa.* ⇒ ele carrega o **`Param`**, e os três canais saem de uma porta só
([`Param::colour_channels`](../../crates/ph2d-field/src/dims.rs)) que o painel e o dreno partilham.

⚠️ **E a condição do ramo da amostra era a FAMÍLIA e passou a ser a AMOSTRA:** com o filtro por
`Param::Material`, a cor de uma lâmpada caía no slider e pintava o canal **vermelho** como um número.
*Quem decide que isto é uma cor é o `swatch`, e perguntar duas vezes deixa as duas respostas
divergirem.*

---

### §25.5 — ⛔ Acender uma luz NÃO entra na paleta de formas, e foi um gate que o disse

A primeira tentativa pôs a luz no catálogo, numa família `Lights`. O
`each_family_has_its_own_title_and_colour` reprovou: ele exige **uma tinta por família**, e há
exactamente **sete** `NodeCat*` — todas tomadas. Uma oitava família pedia um token novo, que é decisão
de design (§7) e **não desta linha**.

⭐ **E a recusa aponta para a leitura certa:** *aquela paleta é de FORMAS*, e uma lâmpada não é uma
forma — ela nem sequer entra na árvore da peça. ⇒ o gesto é um **chip próprio na fileira de criar**, ao
lado do *+ Add shape…*, e a fileira passou a decidir por **slot** em vez de todo clique abrir a paleta.
⛔ Um `_ =>` ali faria o chip novo parecer vivo e abrir a coisa errada — *o pior dos dois, porque parece
funcionar*.

---

### §25.6 — ⛔⛔ Duas coisas que quase shiparam invisíveis

**(a) A luz não tinha linha na Hierarquia.** A query das raízes da casa é
`With<Transform>, Without<ChildOf>`, e o `add_light` dava `Name` e `FieldPose` e mais nada. A luz
existia, iluminava a peça e **não se podia escolher, renomear, esconder nem apagar**. *Uma luz que não
aparece na Hierarquia não é um objecto 3D — é uma variável global com uma posição*, que é exactamente o
oposto da ordem. ⇒ `Transform` como **marcador** (a pose continua no `FieldPose`, que é a convenção que
o `spawn_doc` já usa na raiz da peça), e gate com **piso de população** — a peça tem de estar na lista
também, senão um dia em que a query deixe de casar seja o que for ele leria «zero raízes» e passaria
por vacuidade.

**(b) O gizmo não pegava nela.** O `anchor_for` perguntava *«é um nó?»* (`FieldNode`), e a resposta
certa é a mais simples das duas: *o gizmo agarra o que tem onde estar* ⇒ `FieldPose`.

---

### §25.7 — Os gates (9) e as mutações (7/7), com as DUAS que sobreviveram

| gate | o que ele prende |
|---|---|
| `a_light_is_a_row_of_the_hierarchy` | a query da casa, com piso de população e os nomes únicos |
| `the_first_light_is_born_where_the_rig_lamp_shone_from` | direcção, distância e força — e que o sítio **segue a câmera** |
| `the_panel_of_a_light_offers_no_angle_and_no_size` | a W34 sobre o objecto novo, e que mover é o mesmo gesto |
| `a_light_object_lights_the_side_it_is_on` | ⭐ acende **e** do lado certo, nos **dois** sentidos |
| `a_light_falls_off_with_the_square_of_the_distance` | a queda, pela imagem — e a singularidade |
| `the_hierarchy_eye_switches_a_light_off` | o olho, com o controlo do outro lado |
| `the_lights_come_out_in_a_stable_order` | a ordem contra a troca de arquétipo |
| `a_light_of_one_at_one_unit_is_the_lamp_the_rig_had` | a unidade da intensidade é a do rig |
| `a_negative_light_is_no_light_never_a_dark_one` | força e cor negativas são coadas |

⚠️⚠️ **A primeira redacção do gate da queda era uma TAUTOLOGIA, e uma mutação provou-o.** Ela chamava
`Surface::direct` com uma radiância que **o próprio teste dividia por `r²`** ⇒ apagar a divisão do
**produto** deixava-a verde. *Um gate que re-implementa a lei não mede o produto — mede-se a si mesmo.*
Hoje ela compara duas **imagens**, e o controlo dela mede a razão `4` sem compensação. ⚠️ E essa razão
teve de ser lida em **linear**: a saída é sRGB8, e `4` em linear lê-se **`1,88`** em bytes — o número
que a primeira leitura acusou é exactamente `4^(1/2,2)`.

⚠️⚠️ **E a segunda sobrevivente ensinou que o PISO da distância resolvia metade de um defeito.** Pô-lo a
`0` não sangrava: o byte satura, logo `1/0,0025` e `1/0` pintam os mesmos `255`. *O que era observável
era o defeito ao lado dele* — com a luz exactamente sobre o ponto, `d` é o vector **ZERO**, a direcção
normalizada sai `[0,0,0]`, o `N·L` dá `0` e o pixel fica **PRETO**, que é o mesmo sintoma da divisão
por zero por outro caminho. ⇒ abaixo do piso a direcção passa a ser a **NORMAL**, e o gate produz a
distância exactamente zero **lendo um ponto do próprio G-buffer**.

*Um piso que protege a aritmética e deixa a geometria degenerada resolve metade de um defeito, e a
metade que fica tem o mesmo sintoma.*

---

### §25.8 — ⏳ O que fica aberto (e o dono declarou esta fatia PROVISÓRIA)

- ⛔⛔ **O CÉU continua ancorado no ECRÃ e as luzes passaram a ser de MUNDO** — a sala não roda com a
  câmera e as lâmpadas rodam. É uma incoerência **declarada**: o céu de mundo é outra decisão (a `W3`
  do plano nomeia-a), e chega com o resto dos modos de arte.
- ✅ ~~**Uma luz não se escolhe no CANVAS**~~ — **FECHOU no mesmo dia (§26)**, por report do dono:
  *«a luz não tem seu próprio gizmo»*. Ela tem agora uma marca desenhada e clicável.
- ✅ ~~**Os verbos ROTAÇÃO e ESCALA do gizmo são inertes numa luz**~~ — **FECHOU (§28)**, e a cura
  escrita aqui estava ERRADA: ela dizia *«restringi-lo por-objecto é um campo novo no `Anchor`»*, e
  o que se restringe não é o gesto, é a **OFERTA**.
- ✅ ~~**Duplicar uma luz não faz nada**~~ — **ela sempre funcionou (§28)**. A nota dizia *«declarado
  e testado»* e o que estava testado era a porta de MODELAGEM, que uma luz **nunca visita**.
- **Uma luz não viaja no arquivo com o resto** — ela é uma entidade com componentes registados, logo
  entra no `WorldSnapshot`; o que **não** foi exercitado é um projecto gravado antes desta wave, que
  abre **sem luz nenhuma** e fica aceso só pelo céu.
- **Uma só luz e um só tipo.** Sem cone e sem área — ✅ a **sombra** fechou na `W4` (§27).

---

## §26 — ⭐⭐⭐ A LUZ TEM GIZMO PRÓPRIO (report do dono, 2026-09-14)

Enio, depois de smokar a §25: *«Smoke OK. A luz não tem seu próprio gizmo»* — a metade que a §25.8 já
tinha nomeado como aberta.

---

### §26.1 — Porque ela é obrigatória, e não decoração

Uma luz **não tem campo**: o clique do canvas marcha a peça e não a encontra, logo a wave anterior
deixou-a alcançável só pela Hierarquia. ⇒ *um objecto 3D que não se pode apontar no sítio onde ele
está não é um objecto 3D*, e era essa a metade que faltava à ordem.

**A marca** é um disco do tamanho do punho do gizmo da casa, com oito raios à volta e um anel:

| o quê | de onde vem |
|---|---|
| o raio do disco | [`crate::gizmo::GRIP_HALF_PX`] — *uma marca de luz é uma alça como as outras, e duas escalas de chrome na mesma janela leem-se como duas ferramentas* |
| os raios (`1,4×`..`2,4×`) e a espessura | derivados do disco e da haste do gizmo |
| o **raio de agarre** | `disco + GRAB_PX/2` — a mesma folga que o vértice do gizmo declara por escrito: *«um alvo que agarra exactamente onde pinta obriga a mão a acertar no pixel»* |
| a cor do **miolo** | a cor da própria lâmpada, pela mesma porta sRGB↔linear da amostra do painel |
| o anel e os raios | `Text1`, ou **`Accent`** quando ela é a escolhida |

⭐ **Oito raios não é decoração:** é o que faz a marca ler-se como *luz* e não como um ponto de pivô.
⛔ Menos de seis lê-se como uma estrela de selecção; mais de doze vira um disco a esta escala.

⚠️ **E uma luz APAGADA continua a ter marca**, esmaecida — a ordem da §23 aplicada aqui. *Uma marca
que sumisse com o olho da Hierarquia deixaria a luz sem forma de voltar a acender senão pela
Hierarquia, que é de onde esta wave a tirou.*

---

### §26.2 — ⭐⭐ UMA função responde às duas perguntas

O pintor e o teste de acerto fazem a **mesma** pergunta — *onde é que esta luz cai no ecrã?* — e ela
vive uma vez só, em [`lights::marks`](../../crates/ph2d-app-field3d/src/lights.rs). Este módulo de
pintura **não projecta nada**: recebe as marcas prontas.

*Uma marca desenhada num sítio e apanhada noutro lê-se como «o clique não pega», e nenhum dos dois
lados o diagnostica sozinho* — é a mesma lei que o `smoke_draw` já escreve sobre a projecção do gizmo
(*«nunca uma segunda conta a partir do tamanho do traçado»*).

⚠️ **A marca GANHA da peça**, porque é um sobreposto — e o teste dela corre **antes** do `doc?`: uma
cena **sem peça nenhuma** não tem documento, e sem essa ordem as luzes de uma cena vazia seriam
inalcançáveis. *Que é exactamente a cena em que alguém está a montar a iluminação.*

⚠️ E a **recolha** mudou de forma para servir os dois consumidores: ela tem agora `bits`, `world`, a
lâmpada e o **olho**, e a lista do renderizador **deriva** dela ([`lights::lamps_of`]). ⛔ Filtrar o
olho na recolha tirava a luz apagada também do canvas.

---

### §26.3 — Os gates (7) e as mutações (7/7), com as DUAS fixturas que não distinguiam nada

| gate | o que ele prende |
|---|---|
| `the_click_finds_the_mark_where_it_is_drawn` | ⭐ a lei inteira, medida pelo **par** — e o agarre **acaba** |
| `the_nearest_mark_wins_and_not_the_first` | o desempate, com as duas ao alcance do mesmo clique |
| `a_light_that_is_off_still_has_a_mark` | a marca fica, não acende, **e pinta-se diferente** |
| `the_mark_puts_geometry_in_the_scene` | o pintor desenha — com o controlo de pintar sem marca, e a segunda luz a acrescentar |
| `the_selected_mark_is_painted_differently` | o realce compara **quem**, e uma entidade alheia não o acende |
| `the_frame_paints_the_mark_and_a_click_on_it_picks_the_light` | ⭐⭐ a costura das **duas** pontas, com uma luz de verdade no mundo |
| `const _: () = assert!(MARK_GRAB_PX > MARK_HALF_PX)` | ⭐ em tempo de **compilação** |

⚠️⚠️ **Duas mutações sobreviveram, e as duas acusaram a FIXTURA, não o gate:**

- *«a primeira da lista ganha em vez da mais próxima»* — as duas luzes estavam em lados opostos da
  peça, logo ao clicar numa a outra caía **fora do agarre** e o filtro já a tinha deitado fora.
  *Uma fixtura em que só um candidato sobrevive ao filtro não testa o critério de desempate.*
- *«uma luz apagada pinta-se igual a uma acesa»* — o gate afirmava que a marca **continua lá**, e isso
  é satisfeito por uma marca que **mente** sobre o estado da luz.

⚠️ E o `the_grab_is_wider_than_the_drawing` era um `#[test]` sobre dois `const`: o clippy chamou-lhe
*«esta asserção tem valor constante»*. ⇒ virou `const _: () = assert!(…)`, que o `cargo check` corre —
*um gate que só um `cargo test` corre é mais fraco do que um que o `check` corre.*

---

### §26.4 — ⏳ O que fica aberto

- **Rodar e escalar uma luz continuam inertes** (§25.8) — o verbo do gizmo é estado de **vista**.
- **A marca não tem realce de PASSAGEM** (hover): ela acende com a selecção e mais nada.
- **Duas luzes exactamente sobrepostas no ecrã** ficam por desempatar — a regra é a distância no ecrã,
  e ali qualquer regra é arbitrária. *A do ecrã é a única que o artista consegue prever.*

---

## §27 — ⭐⭐⭐ A PEÇA FAZ SOMBRA EM SI PRÓPRIA (W4, 2026-09-14)

O plano manda esta wave **começar por medir** (`03_o_plano.md` §W5: *«a wave começa por medir quanto
de GI cabe, e o resultado pode ser «cozida e não em tempo real» — que é uma resposta legítima»*).
Começou. Esta secção é o número, o que ele fechou, e o que ele deixou por decidir.

### §27.1 — ⛔ A régua da W4 pede um CHÃO, e a cena não tem nenhum

A W4 chama-se *«Sombras que POUSAM o objecto»* e a régua dela é **«um objecto a `0`, `1` e `10 cm`
do chão tem de dar três sombras diferentes»**. Auditado o produto: **não existe chão**. O *prato* é a
câmera a girar em torno do Y do mundo (`smoke_draw.rs`, `turn_world`), e o estúdio da §24 é um
**ambiente** — o `Studio::irradiance` escurece a direcção de baixo e não põe geometria nenhuma lá.

⇒ *a régua da wave não é corrível sem uma decisão de produto que não é da linha*: **o modelador passa
a ter um chão visível?** A pergunta vai ao dono no relatório, com o preço já medido ao lado.

⭐ O que **não** depende dessa decisão é a outra metade, e é ela que esta secção entrega: **a peça
tapa-se a si própria**. Numa peça de três cilindros cruzados isso é a diferença entre ler *três
cilindros* e ler *uma mancha*.

### §27.2 — ⭐ O preço, medido (`load 2,53`, mínimo de 5 corridas)

| px | pixels de peça | traçado | sombra | razão | do quadro |
|---|---:|---:|---:|---:|---:|
| `320×180` | `15 196` | `1,42 ms` | `1,51 ms` | `1,06×` | `9,1 %` |
| **`640×360`** — o quadro de MOVIMENTO (piso `D=3`) | `60 770` | `3,51 ms` | **`6,09 ms`** | `1,74×` | `36,4 %` |
| `1280×720` | `243 101` | `12,67 ms` | `24,67 ms` | `1,95×` | `147,7 %` |
| **`1920×1080`** — o quadro ASSENTE | `546 982` | `28,61 ms` | **`56,23 ms`** | `1,97×` | `336,7 %` |

**A unidade que explica a tabela: um raio de sombra custa `29,3` amostras de campo contra as `8,7`
de um raio de câmera — `3,4×`.** Com `44,7 %` dos pixels de peça a ver a luz, `0,447 × 3,4 = 1,52`,
e a razão medida é `1,74`. *O modelo fecha.*

### §27.3 — ⛔⛔ As alavancas óbvias foram medidas, e NENHUMA é barata

| alavanca | o que corta | relógio | veredito |
|---|---|---:|---|
| só os pixels que **vêem** a luz (`N·L > 0`) | `−55,3 %` da população | **`−6 %`** | fica, e **não é ela que paga** |
| cerca na **lâmpada** em vez de `2r` | mediana `3,200 → 1,291` | `−5 %` | fica: é a cerca honesta |
| cerca na **bola** da peça | mediana `→ 0,678` | `−10 %` | fica — ⚠️ **por outra razão**, §27.5 |
| viés de partida `4 → 256` | `29,3 → 13,5` amostras/raio | `−54 %` | ⛔ **RECUSADA** |

⭐⭐⭐ **O achado é a primeira linha, e ela inverte a intuição:** cortar **`55 %` dos raios** corta
**`6 %` do relógio**. *Os raios de costas para a luz acertam na própria peça ao primeiro passo — são
os BARATOS.* O caro é o raio que viaja; e apertar-lhe a cerca **`4,7×`** compra `11 %`, o que diz
que ele também não paga na viagem. **Ele paga a RASTEJAR à saída da superfície**, onde `d ≈ 0` e a
lei `t += d·passo` mal o move.

⛔ **O viés é recusado com número:** ele compra `54 %` de relógio **apagando sombra a sério** — os
pixels tapados caem de `27,3 %` para `20,0 %`. *Um viés maior não torna a sombra mais barata; torna-a
menos sombra.*

### §27.4 — ⭐⭐⭐ Ela viaja na bandeira que JÁ EXISTE, e o quadro de movimento fica BYTE-IDÊNTICO

O módulo tem desde a W73 uma lei escrita, com dois passageiros e um aviso: *grosso a mexer, nítido ao
assentar* — o contorno engrossado e o anti-serrilhado desligado saem da **mesma** bandeira (`coarse`),
e a nota diz que *«uma segunda pergunta para o mesmo facto podia divergir dela»*. **A sombra é o
terceiro passageiro.**

| | traçado | + sombra |
|---|---:|---:|
| movimento (`640×360`, piso `D=3`) | `3,51 ms` | `9,60 ms` de `16,7` |
| assente (`1920×1080`) | `28,61 ms` | `84,84 ms` |

⚠️ **Ela CABIA no quadro de movimento** — e fica fora dele mesmo assim. A razão é o piso: a
`preview.rs` declara por escrito que uma peça que não caiba a `D=3` *«fica presa no piso e a imagem
fica lenta»*, e dobrar o custo do quadro de movimento come metade da folga que o laço do divisor tem
para as peças pesadas. Pendurada no quadro que assenta, o de movimento fica **byte-idêntico** e os
`56 ms` correm **noutra thread**, onde já corriam `28,6` — a janela continua a `60 Hz`.

### §27.5 — ⛔⛔ Duas coisas que eu escrevi ERRADAS e a medição corrigiu

**(a) A acne de sombra, apanhada pelo gate da ESFERA.** A primeira versão partia o raio em `p` e
andava `hit·BIAS` **na direcção da luz**. Numa saída RASANTE (`N·L ≈ 0`) o raio viaja quase tangente
e, ao fim daquele troço, ainda está a menos de `hit` da superfície de onde saiu ⇒ o campo responde
*«acertaste»* sobre a peça em que o raio já estava. Medido: **`10,5 %` dos pixels de uma esfera**
vinham sombreados, e a sonda separa a espécie — **acerto DURO `3 009`, penumbra MOLE `0`**.

⇒ a cura é **erguer o ponto pela NORMAL** antes de partir, e não aumentar o viés (que já estava
recusado por apagar sombra). Um viés ao longo do raio teria de crescer com `1/(N·L)` e na rasante
diverge; erguer `ε` pela normal resolve-o **em qualquer ângulo com um `ε` só**. Depois da cura a
esfera lê **`0` — exactamente zero**, e é essa a barra do gate: não é um número escolhido, é o que a
geometria de um corpo convexo exige.

**(b) A cerca da bola não é uma poupança — é o DOMÍNIO da pergunta.** Eu documentei-a como
optimização; a prova de mutação desmentiu-me: apagá-la põe **`1 554` dos `28 640` pixels da esfera**
a vir sombreados. O mecanismo é o estimador de penumbra `vis = min(k·d/t)`: numa saída rasante de um
convexo, `d ≈ t²/2R`, logo `k·d/t ≈ k·t/2R` é **menor que `1`** enquanto `t < 2R/k` — *ele está a
ler a superfície de onde o raio saiu e a chamar-lhe oclusor*. Fora da bola não existe geometria, logo
nada que se leia ali é informação sobre oclusão.

⛔ **Uma cerca que corrige um artefacto e se documenta como poupança é a mais perigosa que há: o
primeiro que a apertar por relógio apaga a correcção sem saber que ela existia.**

⚠️ E as duas cercas dão a **mesma imagem** na peça de cilindros cruzados (`discordam 0`) — *é
precisamente por isso que só uma fixtura CONVEXA as separa*.

### §27.6 — Onde a sombra entra na conta, e porquê ali

Ela multiplica a **radiância que CHEGA** da lâmpada, dentro do `chega` — não o `N·L` e não o
resultado. *Uma peça tapada por outra continua a reflectir o ambiente, e continua a brilhar se for
ela própria uma luz.*

⚠️ **A primeira redacção do gate deixou a mutação oposta SOBREVIVER.** Ele escolhia um pixel com
`vis < 0,05` e afirmava *«não ficou preto»* — mas `0,03 × (céu + lâmpada)` ainda é maior que zero.
Só um pixel **inteiramente** tapado (`vis == 0,0`) separa as duas contas: ali uma delas dá preto.

⚠️ **E um pixel de costas para a luz recebe `1,0`, não `0,0`.** As duas pintam o mesmo hoje (o
`N·L ≤ 0` já anula a lâmpada), mas o canal diz *«quanto CHEGA»* e a resposta verdadeira é *«nada a
tapa»*. *Escrever `0` seria uma mentira que por acaso não se nota* — até alguém ler o canal para
outra coisa.

### §27.7 — Os gates, e as mutações

| gate | o que ele prende |
|---|---|
| `uma_peca_que_se_tapa_a_si_propria_tem_pixels_tapados` | a sombra existe — `10,4 %` da peça, com piso de população |
| `uma_esfera_nao_se_tapa_a_si_propria` | ⭐⭐⭐ **`== 0`** — apanha a acne **e** a cerca da bola |
| `um_pixel_tapado_continua_a_reflectir_o_ceu` | a sombra multiplica a luz que chega, e não o resultado |
| `sem_o_passe_a_imagem_e_byte_identica` | `shadows: None` e um passe VAZIO pintam os mesmos bytes |
| `probe_de_onde_vem_a_sombra_da_esfera` | (sonda) separa acerto DURO de penumbra MOLE |

| mutação | resultado |
|---|---|
| o raio parte do ponto, sem se erguer pela normal | 🔴 |
| a visibilidade multiplica o RESULTADO | 🔴 *(depois de o gate ser apertado — sobreviveu à 1.ª redacção)* |
| a cerca da bola desaparece | 🔴 *(e foi assim que a §27.5(b) se descobriu)* |
| a sombra nunca chega ao pintor | 🔴 |
| o filtro `N·L` deixa passar todos os pixels | 🔴 |

⚠️ **O denominador mordeu:** a primeira redacção do gate copiou o `27,3 %` da sonda do relógio, que
divide pela população que **vê** a luz (`44,8 %` da peça). Os mesmos `1 777` pixels leem `11,7 %`
com a peça no denominador. *Uma fracção sem o denominador escrito ao lado dela não é um número.*

### §27.8 — ⏳ O que fica aberto

- ⭐⭐⭐ **O CHÃO é decisão do dono** (§27.1) — sem ele a régua da W4 não tem sujeito, e o objecto
  continua a flutuar mesmo com a auto-sombra a funcionar.
- **As luzes ancoradas no ECRÃ não fazem sombra**, e é deliberado: elas giram com a câmera, e uma
  sombra que gira com o olhar não pousa nada — ensinaria o contrário do que a wave existe para dizer.
- **A `HARDNESS = 8` não foi varrida.** Ela é a constante da lei de penumbra (um `k` maior endurece),
  não um teto de recurso — mas o número é do idioma, não de uma medição nossa.
- **A borda anti-serrilhada usa a sombra do CENTRO do pixel** nas quatro amostras — a mesma
  aproximação que a direcção de vista e o material já fazem ali.
- **O passe é `O(lâmpadas × pixels)`** e não há cache: duas luzes custam o dobro.


---

## §28 — ⭐⭐ A LUZ COMPORTA-SE COMO UM OBJECTO (2026-09-14)

Três itens do §25.8, auditados contra o **código** antes de lhes tocar — que é a lei que este
repositório escreve sobre as próprias listas abertas. **Duas das três notas estavam erradas**, e o
erro das duas tem a mesma forma.

### §28.1 — ⛔⛔ *«Duplicar uma luz não faz nada»* era falso, e a nota dizia-se TESTADA

Ela lia: *«o `duplicate` exige um pai, e uma luz é raiz — **declarado e testado**, não acidental»*.
O que estava testado era [`ph2d_field_ecs::duplicate`], a porta de **modelagem** — e uma luz **nunca
a visita**: o `hierarchy_duplicate::duplicate_kind` pergunta por `FieldNode`, uma luz tem
`FieldLight`, logo ela cai no **braço genérico**, que é a cópia profunda do ADR-0164. Essa leva
**todo componente registado**, e os dois que fazem a luz (`FieldLight`, `FieldPose`) estão
registados.

⚠️ *Uma ausência afirmada pela porta ERRADA é um palpite com cara de medição* — e aqui ela veio
com a palavra «testado» ao lado.

⇒ o gate `duplicar_uma_luz_da_uma_segunda_luz` afirma o **roteamento** e o que a cópia **leva**:
cair no braço genérico só é a resposta certa enquanto os componentes forem registados, e um
`register_default` apagado devolveria exactamente o *«sósia que não desenha nada»* que o
`DuplicateKind` existe para evitar. (Mutação: apagar o registo do `FieldLight` ⇒ 🔴.)

### §28.2 — ⭐⭐⭐ Rodar e escalar uma luz: o que se restringe é a OFERTA, não o gesto

Uma lâmpada de ponto não tem orientação nem tamanho, e com ela escolhida aqueles dois verbos
desenhavam alças que não moviam número nenhum — *um botão pintado e morto*, que é como o próprio
`scene_gizmo` já descreve o que ele se recusa a desenhar num nó trancado.

⛔ **A cura que a nota prescrevia era um campo novo no [`crate::gizmo::Anchor`]**, raciocinada a
partir de *«o verbo é estado de VISTA, um por viewport»*. Está errada: **o gizmo não precisa de
saber que esta pergunta existe** — se o chip não é oferecido, o verbo nunca fica activo. *Uma cura
desenhada a partir de onde um valor MORA, em vez de a partir de quem o OFERECE, compra o refactor
errado.*

⭐ E o molde já estava no mesmo ficheiro, dez linhas abaixo: a fileira do **laço**, que só aparece
com duas ou mais peças e, ao sumir, **põe o modo de volta** — senão ele ficaria *«armado e
invisível»*.

⚠️ **A regra é sobre a selecção INTEIRA, não sobre a primária:** com uma luz e uma forma escolhidas
rodar tem sujeito (a forma), e tirar o verbo ali seria tirar um gesto legítimo por causa de um
acompanhante. (Mutação que lê só a primária ⇒ 🔴.)

### §28.3 — ⛔⛔ E o filtro ia introduzindo um defeito MUDO: a segunda contagem

O consumidor do clique fazia `Mode::ALL.get(slot)` e o painel publicava `Mode::ALL` inteiro — as
posições casavam **por construção**. Ao filtrar a oferta isso deixa de ser verdade, e ⚠️ **hoje ele
ainda acertaria por ACIDENTE**: o `Move` é o primeiro e é o único que sobra numa luz.

*Uma correspondência que só se mantém enquanto o primeiro elemento não muda não é uma lei — é uma
coincidência à espera de um verbo novo*, e o `ph2d_panel_model3d::area_bar` já proíbe esta forma por
escrito (*«uma tabela aqui seria uma segunda contagem: acrescentar um verbo lá dentro faria o
`SCALE` mandar rodar, e nada acusaria»*). ⇒ **uma lista só** (`scene::offered_verbs`), lida pelos
dois lados.

### §28.4 — ⛔⛔⛔ E o gate desse defeito SOBREVIVEU à mutação, porque media a porta e não o consumidor

A 1.ª redacção do `o_slot_do_verbo_le_a_lista_que_foi_oferecida` afirmava que `offered_verbs(luz)`
não tem posição `1`. Verdade, e **inútil**: repor `Mode::ALL.get(slot)` no dreno deixava-o **verde**.
*Um gate que só mede a porta não prova que alguém a usa* — a costura não-testada da
`DIRETIVA_IMPLEMENTACAO` §1, outra vez.

⇒ ele passa a **empurrar o intent e a correr o dreno do produto**, com o discriminador certo: o
`slot 1` com uma luz escolhida é um clique que a fileira **nunca pôde pintar**. Com a lista certa
não acontece nada; com a `Mode::ALL` ele arma o **Rotate**. ⭐ E leva o controlo ao lado — com uma
**forma** o mesmo `slot 1` *tem* de armar o Rotate, senão o gate passaria por o dreno estar morto.

⚠️ **E ele tem de ARMAR o módulo**: fora do pill o `with_smoke` devolve `None` e todo gancho é
inerte (W42). A 1.ª corrida leu `None` dos dois lados e teria passado por não medir nada.

### §28.5 — ⏳ O que fica aberto

- **Uma luz continua a não viajar num projecto gravado ANTES desta wave** — ele abre sem lâmpada e
  a peça fica acesa só pelo céu. A semente só corre quando a cena de demo nasce; o artista tem o
  `+ Light`, mas nada lhe diz que falta uma.
- **A marca da luz não tem realce de PASSAGEM** (§26.4), e duas exactamente sobrepostas continuam
  por desempatar.
- **Escalar uma luz terá sujeito quando houver luz de ÁREA**, e rodar quando houver **cone** — as
  duas são `W4` do plano e não estão construídas. *O verbo volta à fileira no dia em que o tipo de
  luz o justificar, e o `offered_verbs` é o sítio único onde isso se escreve.*

---

## §29 — ⭐⭐⭐ QUANTO DE GI CABE: a medição que a W5 exige ANTES de existir (2026-09-14)

O `03_o_plano.md` §W5 manda, por escrito: *«a wave começa por medir quanto de GI cabe, e o resultado
pode ser «cozida e não em tempo real» — que é uma resposta legítima»*. Esta secção é esse número.

### §29.1 — A medição (`load 2,06`, mínimo de 3 corridas, três cilindros cruzados)

`N` raios por pixel de peça, cosseno-distribuídos no hemisfério da normal, com a cerca curta da
oclusão (`0,35 × half_extent`) e a partida erguida pela normal (§27.5).

| px | raios/px | traçado | GI | GI/traçado | do quadro |
|---|---:|---:|---:|---:|---:|
| `640×360` | `1` | `3,70 ms` | **`7,21 ms`** | `1,95×` | `43,2 %` |
| `640×360` | `4` | `3,70 ms` | `29,82 ms` | `8,05×` | `178,6 %` |
| `640×360` | `16` | `3,70 ms` | `132,46 ms` | `35,8×` | `793,2 %` |
| `1920×1080` | `1` | `21,67 ms` | `70,34 ms` | `3,25×` | `421,2 %` |
| `1920×1080` | `4` | `21,67 ms` | `310,15 ms` | `14,3×` | `1 857 %` |
| **`1920×1080`** | **`16`** | `21,67 ms` | **`1 348 ms`** | `62,2×` | `8 073 %` |

⭐ **Um raio de GI é `1,9×` mais BARATO que um de sombra** (`119 ns` contra `224 ns`), e a razão é a
cerca: ele pergunta até `0,35 × half_extent` e o de sombra até à lâmpada. *A cerca é o que decide o
preço, outra vez.*

### §29.2 — ⛔⛔ O veredito: a força bruta por pixel NÃO cabe, por uma a duas ordens de grandeza

**`1,35 s` para um quadro assente a `1920×1080` com `16` raios por pixel** — contra os `85 ms` que o
quadro assente custa hoje (traçado + sombra). Nem como disparo único isso é um modelador.

⭐⭐⭐ **E isso CONFIRMA a escolha do plano em vez de a contrariar.** O `03` §W5 nomeia como candidato
principal as **cascatas de radiância com sondas esparsas**, e a razão escrita ali é *«sem ruído, logo
sem denoiser»*. A medição acrescenta a segunda razão, a de preço: **o que não escala é a contagem de
raios POR PIXEL**, e uma sonda esparsa é exactamente o que a desacopla da resolução.

### §29.3 — ⭐⭐ A segunda rota, que o módulo quase já tem de graça

`1` raio por pixel a `640×360` custa `7,21 ms`. Acumulado ao longo de passagens sucessivas do quadro
**assente**, `16` raios custam `115 ms` repartidos por `16` passagens de `7 ms` — a imagem **afina
enquanto a mão está parada**, que é o idioma de toda viewport de render.

⚠️ **Ela pede uma lei nova, e é honesto dizê-lo:** hoje o assentar faz **um** traçado e pára (o
`preview::next_trace` devolve `None` quando nada mudou, porque *«re-traçar seria queimar um núcleo
por nada»*). Acumular é precisamente re-traçar de propósito, com um critério de paragem. ⇒ não é uma
afinação do que existe; é uma decisão de desenho, e fica registada aqui com o preço ao lado.

### §29.4 — ⛔ E a PRIMEIRA redacção desta sonda mediu a coisa errada

A coluna do *tapado médio* lia **`5,3 %` a 1 raio e `17,3 %` a 16**, o que se lê como *«um raio
subestima a oclusão»* — uma conclusão sobre AO. Era um furo da **régua**: a elevação saía de
`(j + 0,5)/N`, logo com `N = 1` ela vale `0,5` em **todo** pixel — um só ângulo (45°) na imagem
inteira.

⇒ deslocamento por pixel (Cranley–Patterson), e a coluna passa a ler **`17,3 %` nos três N**, que é
o que um estimador imparcial tem de fazer. *Uma estratificação que muda com o N não compara os N* —
e o custo, que é o que a sonda existe para medir, subiu `20 %` ao ser medido em direcções honestas.

### §29.5 — ⏳ O que esta secção NÃO mede

- **Uma peça só, e uma câmera só.** O custo por raio segue o que a §27 mediu — ele paga a rastejar à
  saída da superfície —, logo uma peça com mais arestas paga mais.
- **Zero bounces de COR.** Isto é visibilidade (oclusão), não transporte de luz: a radiância que
  volta pelo raio não foi sombreada. *O preço de uma segunda quicada não está aqui.*
- **Nada na GPU.** Todos os números são da CPU, como o resto do traçado deste módulo.

---

## §30 — ⭐⭐⭐ A PEÇA GANHA PROFUNDIDADE: a oclusão refina com a mão parada (2026-09-14)

A §29 mediu que a luz indirecta por força bruta **não cabe** e nomeou duas rotas. Esta secção
constrói a segunda: **o quadro assente deixa de fazer um traçado e parar — ele continua a refinar**,
e o que ele refina é a **oclusão traçada contra o campo verdadeiro**.

### §30.1 — O que o artista vê

Enquanto a mão mexe, nada muda: o quadro de movimento é **byte-idêntico** ao de ontem. Quando ela
pára, a imagem fica nítida (como já ficava) e depois vai **ganhando profundidade** — as fendas entre
as partes escurecem, e a peça deixa de parecer recortada. Medido (`load 2,98`):

| px | traçado | 1.ª passagem | 4 passagens | 8 | **assentar (16)** |
|---|---:|---:|---:|---:|---:|
| `640×360` | `5,80 ms` | `10,87 ms` | `43 ms` | `88 ms` | **`207 ms`** |
| `1920×1080` | `29,18 ms` | `96 ms` | `390 ms` | `864 ms` | **`1 969 ms`** |

⚠️ **Estes números são os da §31**, que corrigiu o amostrador e acrescentou a suavização — a 1.ª
redacção desta tabela dizia `32` passagens e `3 474 ms`, sobre um estimador enviesado.

⚠️ **A imagem é utilizável desde a 1.ª passagem** e visivelmente assente por volta da 16.ª; o resto
é o último byte. O trabalho corre **noutra thread** e é **largado** assim que a mão volta a mexer.

### §30.2 — ⭐⭐ Onde ela entra na conta, e porquê ali

**A oclusão é a SOMBRA DO CÉU.** Ela vive no mesmo canal que as lâmpadas
([`ph2d_field_render::Shadows`]) porque é a mesma pergunta — *«quanto desta fonte chega a este
pixel?»* — e multiplica **só o termo do ambiente**:

- ⛔ **não toca nas lâmpadas** (elas têm sombra a sério, §27) — uma fenda continua a receber a luz
  directa que a alcança, e é isso que impede a peça de ficar acinzentada;
- ⛔ **não toca na emissão** — uma superfície que é ela própria uma luz não se apaga por ter vizinho.

⚠️⚠️ **Aproximação declarada:** o `indirect` devolve o difuso **e** o especular do ambiente numa
chamada só, e a oclusão exacta do especular não é a do difuso. Escalar os dois pelo mesmo número
escurece reflexos a mais numa superfície polida. Separá-los é mexer na fronteira do `ph2d-material`.

### §30.3 — ⛔⛔⛔ TRÊS coisas que eu escrevi erradas, e cada uma foi apanhada por uma régua diferente

**(a) O `hardness` — eu usei o estimador de PENUMBRA para responder a uma pergunta BINÁRIA.**
Escrevi `hardness = 1` a raciocinar *«`k = 1` é a fracção de céu que o vizinho deixa passar»*. O
gate da esfera apanhou-o: ela leu **`0,698` de céu contra `0,757` da cruz** — *um corpo CONVEXO a
ocluir-se mais que três cilindros cruzados*. É o mecanismo da §27.5(b): numa saída rasante
`d ≈ t²/2R`, logo `k·d/t` desce abaixo de `1` **sem haver oclusor nenhum**. Num raio de sombra isso
é penumbra e é desejável; aqui a pergunta é *«bate em alguma coisa?»*, que é sim ou não. Com
`INFINITY` a esfera lê **`1,000` exacto**.

**(b) O alcance — eu ESCOLHI `0,35` e o §0.0 manda medir.** Varrido, a resposta **satura em `1,0`**.
⚠️ E a coluna que o mostra é a **cauda**, nunca a média: numa cruz a fenda é `~5 %` dos pixels
visíveis, logo uma oclusão de `94 %` no fundo dela move a média da peça de `1,000` para `0,965`.
*Ler `0,965` é ler «quase nada»; ler `mín 0,062 · p05 0,750 · 5,8 % abaixo de 0,8` é ler o que o
olho vê.* É o «extremo global» deste repositório, pelo lado de dentro — e o **gate media a média**.

**(c) A API não sabia acumular, e eu só dei por isso ao tentar.** O 1.º desenho era
`(desde, quantos)`; ele ancora a estratificação na **fatia**, logo somar fatias dá uma distribuição
diferente da sequência inteira. São **três** números — `(primeiro, quantos, total)` — e o `total`
fixa as elevações que a sequência inteira vai cobrir. O gate que autoriza tudo o resto é
`somar_as_fatias_da_a_sequencia_inteira`, **ao bit**.

### §30.4 — ⛔⛔ E o PRATO GIRATÓRIO ia congelar

O prato só avança quando **não há trabalho em voo**. Um refinamento dura `3,5 s` a `1920×1080` ⇒
com ele em voo o prato passaria a dar **um passo a cada `3,6 s`**, e o artista leria isso como o app
travado. ⇒ *um prato a girar é MOVIMENTO*, e o refinamento é do assente
([`preview::refines_occlusion`]): tocar no canvas pára o prato, e é exactamente aí que ele começa.

### §30.5 — ⚠️ A fila passou a ser LIMITADA, e a razão é aritmética

Um trabalho manda `32` quadros em vez de um. Sem tecto, a fila guarda `32 × 8,3 MB = 265 MB` a
`1920×1080` sempre que ninguém a esvazie. ⇒ `sync_channel(2)` + `try_send`, e **perder uma passagem
intermédia é inofensivo**: cada quadro traz a média acumulada, não um incremento, logo o seguinte
mostra mais. *É a contrapressão que uma sequência idempotente autoriza.* E o dreno **esvazia até ao
mais novo** — colher um por frame faria a imagem andar atrás da acumulação.

### §30.6 — ⭐ A lei saiu da thread para uma PORTA, e foi por causa do gate

O laço vivia dentro do `std::thread::spawn` do `smoke_draw`, onde nenhum teste lhe chega: a
acumulação, a ordem das passagens e a paragem por cancelamento ficavam **inalcançáveis**. Hoje é
[`ph2d_field_render::refine_occlusion`], e a thread só pinta, manda e diz se vale a pena continuar.

| gate | o que ele prende |
|---|---|
| `somar_as_fatias_da_a_sequencia_inteira` | ⭐ a acumulação, **ao bit** |
| `a_oclusao_distingue_uma_esfera_de_uma_cruz` | a cauda, e a esfera a `1,000` exacto |
| `a_oclusao_escurece_o_ceu_e_deixa_a_lampada` | com o céu apagado ela não mexe um byte |
| `o_refinamento_entrega_32_passagens_e_acaba_na_sequencia_inteira` | ordem, contagem e o fim |
| `o_refinamento_para_quando_lhe_dizem_para_parar` | o cancelamento |
| `o_refinamento_nunca_abre_a_peca_preta` | a média é das passagens CORRIDAS, não do total |
| `um_prato_a_girar_nao_refina_a_oclusao` | §30.4 |

⚠️ **E uma fixtura mediu o TECTO DO BYTE:** o gate do céu escolhia um pixel com a lâmpada a `40`, e
ele **saturava** (`765` de `765`) — o gate lia *«não escureceu nada»* sobre um passe que funcionava.

### §30.7 — ⛔⛔ E os 8 vermelhos do `tests/it` têm diagnóstico: contadores GLOBAIS

O `ph2d-field-render --test it` reprova `7`–`8` gates de orçamento (fitas, ladrilhos, amostras) —
**e passa `24/24` com `--test-threads=1`, a `load 75`**. Eles leem estáticos de processo
(`TAPE_HITS`, `SPECIALISED`, `TILE_COSTS`) e envenenam-se uns aos outros dentro do mesmo binário.

⚠️ **O conjunto MUDA entre corridas** — três corridas, três conjuntos — que é a assinatura que esta
casa já conhece; e **a causa aqui não é a carga, é o fan-out DENTRO do binário**. Pré-existente:
verificado numa worktree em `HEAD` limpo, os mesmos gates. *Dívida nomeada, não desta wave.*

### §30.8 — ⏳ O que fica aberto

- **A oclusão do ESPECULAR** é a do difuso (§30.2), e não devia ser.
- **Zero bounces de COR**: isto é visibilidade, não transporte — a luz que volta pelo raio não é
  sombreada. *A cor sangrada de uma parede vermelha continua por construir.*
- **Uma peça só, e uma câmera só**, nos números: o custo por raio segue a §27 — ele paga a rastejar
  à saída da superfície, logo uma peça com mais arestas paga mais.
- **O prato giratório nunca refina** (§30.4). Numa cena de demonstração que ninguém toca, a oclusão
  não aparece — e isso é a lei, não um esquecimento.

---

## §31 — ⛔⛔⛔ «ARTEFATOS DE IMAGEM»: as listras, e a régua que era um ESPELHO (report do dono, 2026-09-14)

Foto do dono: **listras verticais finas**, a alternar coluna a coluna, sobre a peça de cilindros
cruzados. Eram **dois defeitos na mesma linha de código**, e a §30 shipou com os dois porque a régua
que os devia apanhar media o estimador **contra ele próprio**.

### §31.1 — Os dois defeitos, na mesma expressão

```rust
let u2 = ((i as u32).reverse_bits() >> 8) as f32 / 16_777_216.0;
```

**(a) O azimute não dependia do RAIO.** Ele saía só do pixel `i`, logo os `32` raios de um ponto
partilhavam o **mesmo `φ`**: *cada pixel amostrava um LEQUE PLANO, nunca o hemisfério.* A estimativa
ficava enviesada por pixel — e com um enviesamento **diferente em cada pixel**, que é o que produz
estrutura em vez de ruído.

**(b) O bit `0` de `i` virava o bit MAIS SIGNIFICATIVO do azimute.** `i.reverse_bits() >> 8` devolve
os bits `23..0` de `i` **ao contrário**, logo o bit menos significativo do índice — *a paridade da
coluna* — passa a mandar na metade do círculo. Medido: `i = 1000 → 0,093`, `i = 1001 → 0,593`.
**Colunas vizinhas a apontar para lados opostos**: a listra é de uma coluna porque a causa é de um
bit.

### §31.2 — ⛔⛔⛔ E a sonda da convergência era um ESPELHO

A `measure_how_many_passes_the_occlusion_needs` comparava `N` raios contra uma referência de `64` —
**do mesmo leque plano**. Ela via um `1/√N` impecável sobre um resultado enviesado, e por isso o
`OCCLUSION_PASSES` foi escolhido com base num erro que não era o erro.

⚠️ *Uma régua que partilha a lei do produto não acusa.* A lei está escrita no `CLAUDE.md`, tem
entrada própria na memória do repositório, e mordeu na mesma — porque o «espelho» aqui não era o
código, era **a sequência de amostragem**.

Com a referência honesta (`256` raios, oito vezes o maior `N` testado) os números são outros:

| passagens | erro no pixel (espelho) | erro no pixel (**honesto**) |
|---:|---:|---:|
| `8` | `2,85 bytes` | `4,07` |
| `16` | `1,33` | `2,34` |
| `32` | `0,63` | **`1,42`** |

### §31.3 — ⭐⭐⭐ A cura, e ela ficou MAIS RÁPIDA e MELHOR ao mesmo tempo

**A lei do amostrador** (`sample_uv`): a elevação continua estratificada sobre o total com rotação
por pixel; o **azimute** passa a ser uma sequência aditiva da **razão áurea** a partir de um arranque
por pixel — de baixa discrepância em **qualquer corte inicial**, que é o que uma acumulação exige (a
fatia `0..k` tem de ser boa, não só a sequência completa). E o arranque usa uma **mistura** de Knuth,
não `reverse_bits`: ela leva os bits baixos aos altos, e o acoplamento de paridade desaparece.

**E a suavização guiada** (`blur_occlusion`): a oclusão é de **baixa frequência dentro de uma
superfície** — o que ela tem de respeitar é a fronteira entre superfícies diferentes, e por isso a
média `3×3` é guardada pela **normal** (`cos ≥ 0,9`, `~26°`). ⛔ Não é batota, e há gate: um degrau
perfeito entre duas normais opostas sai **intacto**.

| passagens | erro no pixel | **com suavização** |
|---:|---:|---:|
| `4` | `7,22 bytes` | `2,19` |
| `8` | `4,07` | `1,35` |
| **`16`** | `2,34` | **`0,89`** |
| `32` | `1,42` | `0,67` |

⇒ **`OCCLUSION_PASSES` desce de `32` para `16`**: com a suavização, `16` ficam **abaixo de um byte**
onde `32` sem ela ficavam em `1,42`. *Metade da espera e melhor imagem.*

| | antes (§30) | **agora** |
|---|---:|---:|
| assentar a `640×360` | `385 ms` | **`207 ms`** |
| assentar a `1920×1080` | `3 474 ms` | **`1 969 ms`** |
| erro no pixel | `1,42 bytes` | **`0,89`** |

### §31.4 — ⚠️ E as BARRAS dos gates estavam calibradas sobre o defeito

Com o leque plano a fenda lia `mín 0,062` e `5,8 %` dos pixels abaixo de `0,8`. Com o hemisfério
varrido: **`mín 0,750`** e `12,2 %` abaixo de `0,9`. *O extremo antigo era o enviesamento, não a
geometria* — e o gate `a_oclusao_distingue_uma_esfera_de_uma_cruz` **exigia-o**, isto é, defendia o
defeito. ⛔ Uma barra calibrada sobre um estimador enviesado torna-se a razão para o não corrigir.

⚠️ E a fixtura do gate do céu procurava um pixel com `oc < 0,6`: depois da cura **não existia
nenhum**, e o gate rebentava com *«nenhum pixel ocluído»*. *Uma fixtura calibrada sobre um defeito
deixa de ter sujeito quando ele é curado* — e essa é a forma barata de descobrir que ela o media.

### §31.5 — As réguas que passam a existir

| gate | o que ele prende |
|---|---|
| `os_raios_de_um_pixel_varrem_o_azimute_inteiro` | ⭐ **a CAUSA, sem traçar nada** — nenhum oitavo do círculo vazio, e vizinhos não arrancam a meia volta |
| `a_oclusao_nao_faz_listras_entre_colunas_vizinhas` | ⭐ **a CONSEQUÊNCIA** — a horizontal não pode destoar da vertical, e pares/ímpares leem o mesmo |
| `a_suavizacao_nao_atravessa_uma_quina` | o degrau entre duas normais opostas sai intacto |
| `a_suavizacao_apaga_o_ruido_dentro_de_uma_superficie` | e o controlo: ela não pode ser um no-op |

⭐ **A listra passou a ser uma MEDIÇÃO**, e não uma foto: a assinatura é *a diferença média entre
vizinhos horizontais a destoar da vertical*. Foi essa a régua que faltava — a que compara os dois
eixos em vez de olhar para um número só.

### §31.6 — ⏳ O que fica

- **A suavização é `3×3` e uniforme.** Um raio maior apagaria mais ruído e começaria a esbater a
  oclusão de contacto; o `3×3` foi o que a tabela mediu, e não foi varrido.
- **A guarda é a NORMAL, e não a profundidade.** Duas superfícies paralelas a distâncias diferentes
  (uma peça à frente de outra) misturam-se — declarado, e sem sujeito enquanto a cena é uma peça.

---

## §32 — ⛔⛔⛔ O DONO REPROVOU, E A MEDIÇÃO DIZ QUE ELE TEM RAZÃO POR UMA RAZÃO ESTRUTURAL (2026-09-14)

Dois reports no mesmo dia:

> *«funciona mas com aspecto ruim, muito demorado e em etapas estranhas. Bastante inferior a app
> como Unreal. Lembre-se: queremos superar Unreal»*
>
> *«mover os objetos ficou muito lento»*

### §32.1 — ⛔ O segundo é uma REGRESSÃO minha, e tem mecanismo

Cada passagem do refinamento **repinta a imagem inteira**, e o [`shade_render`] corre em **todos os
núcleos** (`par_chunks_mut` da `rayon`). A `1920×1080` isso são `~123 ms` de máquina inteira, `16`
vezes — e o quadro que a mão arrasta usa **a mesma pool**. *O refinamento roubava a máquina ao
gesto.* ⚠️ E o cancelamento era visto **depois** da pintura: largar depois de pagar não é largar.

### §32.2 — ⭐⭐⭐ O primeiro é o §0.0, e eu violei-o a olhos vistos

**Tudo o que as §27–§31 mediram é CPU.** Eu medi a CPU, vi que a luz indirecta não cabia, escrevi
*«não cabe, por uma a duas ordens de grandeza»* (§29.2) e deixei a **CPU desenhar o produto** — com
uma lei de acumulação, passagens visíveis e dois segundos de espera.

O `CLAUDE.md` §0.0 escreve o erro com estas palavras: **«nunca deixe o fallback definir o produto…
o caminho mais lento definiu o teto do mais rápido, no módulo cuja razão de existir é o mais
rápido»**. E esta máquina tem uma **RTX 5060 Ti** parada enquanto a CPU marcha.

### §32.3 — ⭐⭐⭐ O TECTO, medido (`ph2d-gpu/examples/field_march_ceiling.rs`)

A **mesma peça**, a **mesma lei de marcha** (`t += d·passo`), o **mesmo** passo seguro, a mesma
tolerância de acerto e o mesmo orçamento de `512` passos — escritos em WGSL:

| | CPU | **GPU** | ganho |
|---|---:|---:|---:|
| só o traçado, `640×360` | `5,80 ms` | `0,67 ms` | `8,7×` |
| só o traçado, `1920×1080` | `29,18 ms` | `0,67 ms` | `43,6×` |
| traçado + 1 passagem, `1920×1080` | `152 ms` | `0,62 ms` | `244×` |
| **traçado + oclusão INTEIRA (16 raios), `1920×1080`** | **`1 998 ms`** | **`5,00 ms`** | **`399,5×`** |

⇒ **o que aqui custa DOIS SEGUNDOS em dezasseis etapas visíveis cabe em `5 ms` de um quadro de
`16,7`.** Sem etapas, sem espera, sem lei de acumulação — e **sobra orçamento** para o que a §29.5
diz que falta (as quicadas de cor).

⚠️ **É um TECTO, não um produto.** O WGSL da sonda é aquela peça **escrita à mão**; um traçador a
sério compila a **árvore do documento** para o dispositivo, e é essa a wave. *Mas é exactamente para
isto que o §0.0 manda medir antes: o número diz que a wave vale a pena e diz quanto ela pode pedir.*

### §32.4 — ⭐⭐ E é aqui que *«superar o Unreal»* deixa de ser uma frase

O Lumen é uma **aproximação** — espaço de ecrã, *proxies*, sondas de distância —, e é assim porque
ele tem de aceitar qualquer malha de qualquer artista. **Nós temos o campo de distância verdadeiro**
(`02` §5.1): a peça *é* uma função, e marchar sobre ela dá a oclusão **exacta**, sem proxy e sem o
ruído que obriga a um *denoiser*. ⭐ O que faltava não era a ideia — era estar no dispositivo certo.

### §32.5 — O que muda hoje

- **A oclusão de CPU NASCE DESLIGADA** (`preview::refines_occlusion`). O quadro assente volta ao que
  o dono aprovou: traçado + sombra directa.
- **O código FICA** como **referência de CPU** que o traçador de GPU terá de reproduzir — o molde de
  *dois motores, uma lei* que este repositório já usa no Flip e no tecido. `PH2D_FIELD_AO=1` liga-o.
- **O cancelamento passa a ser visto ANTES da pintura**, mesmo com ele ligado.
- ⛔ **E há gate a afirmar a AUSÊNCIA** (`a_oclusao_de_cpu_nao_chega_ao_artista_por_omissao`): sem
  ele, alguém lê a condição que a desliga e «corrige»-a.

⚠️⚠️ **A lição, e ela já tinha ficheiro próprio na memória deste repositório:** *uma feature pode ser
PIOR do que não existir*. **Catorze gates verdes** defendiam um desenho que o dono reprovou em dois
segundos — porque nenhum deles perguntava *«isto é bom?»*, só *«isto está correcto?»*. A linha de
controlo que faltava era a mais simples de todas: **não fazer nada**.

---

## §33 — ⭐⭐⭐ O CAMPO VAI PARA O DISPOSITIVO: a fita vira WGSL (2026-09-14)

O §32 mediu o tecto (`399,5×`) e o dono disse *«siga»*. Esta secção é a primeira metade: **o campo
do documento avalia-se na GPU, e responde o mesmo que o da CPU nas dezoito cenas vivas.**

### §33.1 — ⛔ A rota que a medição REJEITOU: interpretar a fita

Interpretar a fita no dispositivo daria o catálogo de graça **e** zero compilação. Não cabe, e o
número é o **pico de valores vivos** (`Field::tape_shape`, medido nas cenas reais):

| cena | ops | **vivos** |
|---|---:|---:|
| a ponte | `49` | `11` |
| um verbo por forma | `126` | `15` |
| o lote da W103 | `451` | `34` |
| o polígono de `N` | `851` | `110` |
| **a pior** | **`2 969`** | **`464`** |

`464` valores vivos são `1 856 B` por thread e **`116 KB` por workgroup de 64** — acima da memória
partilhada de qualquer GPU. ⚠️ *A maioria das cenas caberia com folga; é a cauda que decide, e uma
arquitectura que só serve o caso médio não serve.*

### §33.2 — ⭐ A rota que fica: gerar WGSL — e ela custa `6`–`49 ms`, por ESTRUTURA

| ops | `naga` | pipeline | **total** | (a cena real com este tamanho) |
|---:|---:|---:|---:|---|
| `50` | `0,2 ms` | `6,2 ms` | `6,4 ms` | a ponte |
| `450` | `1,2 ms` | `7,3 ms` | `8,5 ms` | o lote da W103 |
| `900` | `5,1 ms` | `13,2 ms` | `18,3 ms` | o polígono de `N` |
| `3 000` | `22,2 ms` | `27,1 ms` | **`49,3 ms`** | a pior medida |

⭐⭐⭐ **E paga-se por ESTRUTURA, não por edição.** Arrastar um slider muda um **número**; a árvore
fica igual. ⇒ as constantes saem num **buffer** (`k[i]`) e a chave do cache é o **texto**, que não
muda: *um arrasto de slider reescreve um buffer e não recompila nada.* O que custa é acrescentar
uma forma — uma vez, e o módulo já sabe segurar o quadro anterior enquanto o novo não chega.

### §33.3 — ⭐⭐⭐ O achado: o catálogo inteiro por VINTE E OITO opcodes

O modelador tem **`62` primitivas** e uma pilha de modificadores. Escrever WGSL para cada uma seria
meses — e um **segundo sítio onde a forma vive**, que é o defeito que este repositório mede em toda
a parte.

⭐ Mas o documento **já** é compilado numa **fita**: uma lista SSA de operações **aritméticas**, onde
uma rosca, uma superfórmula e um filete já são `min`, `max`, `sqrt` e multiplicações. O gerador
([`ph2d_field_eval::wgsl`]) percorre a fita e emite **uma linha por instrução** — `17` unárias e
`11` binárias. *O catálogo inteiro, incluindo o que ainda não foi escrito, sai de graça.*

⚠️ **Quatro opcodes exigem cuidado e cada um tem a nota ao lado:**

- **`atan2(a, b)`** — a ordem é a da `fidget` (`a.atan2(b)`); trocá-la **espelha a peça** na diagonal;
- **`Mod` é `rem_euclid`**, não o `%` da WGSL (que leva o sinal do dividendo) — *uma repetição com o
  sinal trocado espelha metade da peça*;
- **`Compare`** devolve `-1/0/1` **e `NaN`** quando não são comparáveis;
- **`And`/`Or`** são a semântica da `fidget` (`a == 0` escolhe), não booleanos.

### §33.4 — ⭐⭐ A prova: dois motores, uma lei

`ph2d-field-gpu` tem o arnês ([`parity::compare`]) e o gate percorre **todas as cenas vivas**,
avaliando `1 331` pontos numa grelha que cai dentro, fora e sobre a superfície:

| pior desvio, sobre as 18 cenas | **`5,6e-7`** |
|---|---|
| a barra | `1e-3` |

É **erro de representação puro** (a fita da CPU é `f64`, o dispositivo é `f32`) e está `1 800×`
abaixo da barra. ⚠️ **A barra não é um número escolhido:** uma lei diferente — a ordem do `atan2`
trocada, o sinal do `Mod` — dá desvios de **unidades de mundo**, três ordens de grandeza acima.
⭐ E o gate exige também que **`NaN` de um lado seja `NaN` do outro**: um motor que responde `NaN`
onde o outro responde um número é uma lei diferente, e a subtração esconde-o.

### §33.5 — ⏳ O que falta para isto ser o traçador

- **A marcha, a normal, a oclusão e o sombreamento** ainda são da CPU; o que existe é o **campo**.
  A sonda `field_march_ceiling` já mostrou o que eles custam quando lá estiverem (`5,00 ms`).
- ⛔ **A ESCULTURA não atravessa** (`NodeKind::Sculpt`): ela não é uma expressão, é uma **grade** que
  o registo do avaliador resolve por nome. Na GPU ela seria uma textura 3D. *A paridade da cena 6
  lê `0,000` porque os dois motores concordam que ali não há expressão nenhuma — não porque a
  escultura funcione.*
- **A `f32` é a divergência declarada**, e ela é maior do que parece num sítio: a marcha acumula
  erro ao longo de centenas de passos. O gate mede o **campo**, não a marcha.
- **Nenhum pipeline é ainda partilhado com o produto** — o cache existe e ninguém o liga.

---

## §34 — ⭐⭐⭐ A MARCHA NO DISPOSITIVO: o G-buffer da GPU é o da CPU (2026-09-14)

O §33 pôs o **campo** no dispositivo. Esta põe a **marcha** — e a decisão que a governa saiu de um
número que eu tinha errado por quatro vezes.

### §34.1 — ⭐⭐ O quadro assente, repartido — e a PINTURA fica

| `1920×1080` | CPU | vai? |
|---|---:|---|
| traçado | `30,04 ms` | ⇒ **vai** |
| sombra directa | `61,73 ms` | ⇒ **vai** |
| **pintura** | **`10,87 ms`** | ⇒ **FICA** |
| total | `102,64 ms` | |

⭐⭐⭐ **Eu supunha que a pintura custava `~45 ms`** e, com esse palpite, teria escrito o material em
WGSL — uma **segunda implementação do OpenPBR**, com tudo o que este repositório já pagou por duas
respostas à mesma pergunta. Medida, ela custa `10,87 ms` e **cabe**. *A lei do material continua a
viver num sítio só, e foi a medição que a salvou.*

⇒ com a marcha no dispositivo, o quadro assente passa de `102,6 ms` para **`~18 ms`** — e traz a
oclusão junto, que hoje está desligada.

### §34.2 — ⚠️ A câmera é a ÚNICA coisa escrita duas vezes, e é deliberado

O `march.rs` avisa por escrito contra *«duas respostas para «que raio sai daqui?»»*. Mandar os raios
prontos seriam **`50 MB` por quadro**, logo o WGSL reconstrói o `ray_at_plane`. ⇒ ela nasce **com o
gate em cima**: uma divergência de câmera move o ponto de acerto em unidades de mundo e vira a
silhueta inteira — exactamente o que a primeira coluna mede.

### §34.3 — A prova, nas 18 cenas vivas (`192×108`)

| | resultado |
|---|---|
| pixels que discordam sobre **haver peça** | **`0,000 %` em todas** |
| `Δt` (p99) | `1,0`–`1,8e-4` — `f32` puro |
| `Δ` da normal contra **a variação da própria peça** | `0,00×`–**`0,17×`** |

⭐⭐⭐ **A régua da normal é a variação entre pixels VIZINHOS da CPU, e não um ângulo escolhido.**
Duas cenas — a **rosca** e as **curvas** — dão `p99` de `12,84°` e `9,90°` contra `≤ 0,50°` das
outras catorze, **e o campo concorda a `1e-7` nas três**. A diferença não é a lei, é o
**condicionamento**: numa ranhura de passo fino a normal roda dezenas de graus de um pixel para o
seguinte, logo um deslocamento de `1e-7` no ponto move-a muito. Medido, o vizinho da rosca varia
`75,92°` — *os dois motores concordam `6×` melhor do que um pixel concorda com o do lado*.

⛔ **Uma barra em graus absolutos ou isentava a rosca ou acusava as outras quinze.** É a mesma
família da §31, do lado oposto: ali a média escondia a cauda; aqui o extremo escondia a população.

⚠️ E a redacção anterior do gate usava o **máximo** — `94,1°` na rosca, um punhado de pixels sobre
um vinco. *O extremo e a população respondem a perguntas diferentes, e só uma delas é sobre a lei.*

### §34.4 — ⏳ O que falta para isto ser o quadro

- **A oclusão e a sombra ainda não estão no shader** — o que está é a marcha primária, a normal e o
  `t`. A sonda `field_march_ceiling` já mostrou o preço delas quando lá estiverem.
- ⛔ **O ANTI-SERRILHADO não atravessa.** O G-buffer da CPU traz `edges` com quatro amostras por
  pixel de borda, e o do dispositivo não — ligar isto ao produto hoje **perderia a suavização da
  silhueta**, que é uma regressão visível. *É por isso que nada disto está ligado.*
- **A escultura continua de fora** (§33.5): ela é uma grade, não uma expressão.
- **O dispositivo abre-se a cada chamada** — é a forma de sonda; o produto segura o contexto e o
  cache de pipelines, que já existe.

---

## §35 — ⭐⭐⭐ O QUADRO INTEIRO NO DISPOSITIVO: sombra, oclusão e bordas (2026-09-14)

O §34 pôs a marcha primária. Esta fecha o G-buffer: **sombra directa, oclusão e o anti-serrilhado**
— e as quatro coisas medem-se contra a CPU na mesma corrida.

### §35.1 — A prova, nas 18 cenas vivas

| | resultado |
|---|---|
| pixels que discordam sobre **haver peça** | `0,000 %` em todas |
| `Δt` (p99) | `1,0`–`1,8e-4` |
| `Δ` normal / variação da própria peça | `0,00×`–`0,17×` |
| **`Δ` sombra** (p99) | `0,000`–`0,027` |
| **`Δ` oclusão** (p99) | `0` ou `0,0625` — **exactamente UM raio de 16** |
| **bordas: sobreposição das listas** | **`98,9 %`–`99,9 %`** |

⭐⭐ **O amostrador da oclusão é o da CPU AO BIT** — a mesma mistura de Knuth, o mesmo deslocamento
por pixel, a mesma razão áurea. ⚠️ *Se ele fosse só «equivalente», a paridade teria de descer a uma
média — que é exactamente a régua que a §31 mostrou ser cega.*

### §35.2 — ⛔⛔ TRÊS coisas que o gate apanhou, e nenhuma era o algoritmo

**(a) A barra da oclusão ficou ABAIXO do quantum da medida.** Escrevi `0,05`; com `16` raios
binários a menor diferença possível é `1/16 = 0,0625`. O gate acusava a **granularidade**. ⇒ a barra
passa a ser **dois raios**, derivada da const: um raio a discordar é `f32` numa saída rasante.

**(b) O tecto da lista de bordas ESTOUROU, e o número vinha da grandeza errada.** Pus `6 %` citando
os *«`0,5`–`1,2 %` de silhueta»* do módulo — mas a borda deste passe é silhueta **mais VINCO**, e
uma rosca é quase toda vinco. Medido: `8,4 %`. A GPU devolvia exactamente `1 296` bordas — *o
tecto* — contra `1 745` da CPU, e a sobreposição caía a `72,6 %`. ⇒ `25 %`, três vezes o pior
medido. *Um tecto derivado da grandeza errada lê-se como generoso.*

**(c) O layout auto-derivado não é o layout do módulo.** As duas passagens usam bindings diferentes,
logo o layout da primeira declara `4` e o grupo de `6` é recusado — **em tempo de execução**. ⇒
layout explícito, partilhado.

### §35.3 — ⛔⛔⛔ E o RELÓGIO mediu a coisa errada, com o aviso escrito no próprio doc

A primeira medição do quadro leu **`130 ms` a `640×360`** — *mais lento que a CPU*. A porta que ela
usava **abre o adaptador, pede o dispositivo e compila o shader a cada chamada**, e o doc dela dizia
*«é a forma de sonda»*. Eu usei-a como relógio na mesma.

⇒ [`Tracer`], que segura o dispositivo e o cache entre quadros:

| | CPU | **dispositivo** | ganho |
|---|---:|---:|---:|
| `640×360` | `13,15 ms` | **`2,53 ms`** | `5,2×` |
| `1920×1080` | `102,64 ms` | **`24,17 ms`** | `4,2×` |

⭐ **E `11 ms` dos `35` iniciais eram a lista de bordas a ser lida INTEIRA.** O tecto é `25 %` dos
pixels e a ocupação real é `1`–`8 %`: copiar o tecto era quase metade dos `90 MB` de leitura por
quadro. ⇒ um segundo `submit` que lê só o que foi escrito. *Um buffer dimensionado para o pior caso
não se lê no pior caso.*

### §35.4 — ⏳ O que ainda separa os `24 ms` dos `5 ms` do tecto

O tecto do §32 (`5,00 ms`) é **compute puro, sem leitura**. O que sobra aqui:

- **`49 MB` de leitura por quadro** (centro `16 B` + luz `8 B` por pixel). O `t` e a normal cabem em
  `8 B` com meia-precisão e uma normal octaédrica — **por medir**.
- **DUAS paragens de sincronização** (a conta, depois as bordas). A segunda pode desaparecer com um
  `map_async` assíncrono em vez de `poll(Wait)`.
- ⚠️ **E nada disto está ligado ao produto.** O que falta é a costura: o `Tracer` vive no viewport,
  o resultado entra como `Gbuffer` + `Shadows`, e a pintura corre onde já corre.

---

## §36 — ⭐⭐⭐ LIGADO: o quadro assente do modelador passa pelo dispositivo (2026-09-14)

### §36.1 — As três condições, e nenhuma é opcional

| condição | porquê |
|---|---|
| **há adaptador** | sem GPU o módulo corre como sempre |
| **o quadro é o ASSENTE** | o de movimento fica **byte-idêntico** — a cerca que impede a regressão do §32 de voltar por outra porta |
| ⛔ **a peça não tem ESCULTURA** | ver §36.2 |

`PH2D_FIELD_GPU=0` devolve tudo à CPU — a porta de bissecção que responde a *«piorou»* sem ninguém
adivinhar qual metade.

### §36.2 — ⛔⛔⛔ A ESCULTURA: o defeito que o gate da paridade NÃO podia ver

Uma escultura (`NodeKind::Sampled`) não é uma expressão — é uma **grade** que o registo resolve por
nome. O compilador da fita traduz o nó para **`Tree::constant(ABSENT)`**, isto é, *espaço vazio*.

⚠️⚠️ **Sem a recusa, ligar a GPU faria a escultura DESAPARECER — em silêncio, e com o resto da peça
perfeito.** E o gate da paridade nunca o veria: ele compara a fita com a fita, e as duas concordam
que ali não há nada. *Foi a cena da ponte a ler `Δ = 0,000` que mostrou o buraco — um zero de
«igual» e um de «nenhum dos dois sabe» são o mesmo byte.*

⇒ [`ph2d_field_gpu::supports`], com gate que afirma a recusa **e** o controlo (as outras dezassete
cenas são aceites, senão a recusa seria um `false` constante).

⚠️ E o mesmo raciocínio vale para **duas lâmpadas**: o shader tem um canal de sombra, logo a segunda
ficaria sem sombra em silêncio. ⇒ o chamador cai na CPU em vez de a ignorar.

### §36.3 — ⛔⛔ E o dispositivo NÃO pode viver no `Smoke`

A primeira versão guardava o traçador no [`crate::smoke::Smoke`], que é um **`thread_local`**. Um
`wgpu::Device` ali é destruído **na saída da thread**, onde outros `thread_local` já morreram:

```text
panicked at std/src/thread/local.rs:429:
cannot access a Local Storage value during or after destruction
```

E o relatório da suíte dizia **`0 falharam · 0 passaram · 0 ignorados`**.
⚠️ *Um binário que não corre teste nenhum lê-se, num relatório, quase como um que passou.*

⭐ O sítio certo não é uma correcção de conveniência: **o dispositivo é da MÁQUINA**. Quatro vistas
da mesma peça partilham a estrutura, logo partilham o pipeline — e nada nele é estado que o artista
tenha pousado. ⇒ um `OnceLock` de processo.

### §36.4 — O que o artista passa a ver

| `1920×1080`, quadro assente | antes | **agora** |
|---|---:|---:|
| encontrar a peça + sombra | `92 ms` | *incluído* |
| o dispositivo | — | `24 ms` |
| pintar (OpenPBR, na CPU) | `11 ms` | `11 ms` |
| **total** | **`103 ms`** | **`~35 ms`** |
| oclusão de contacto | **desligada** | **incluída** |

⭐ E a pintura **continua a ser a da CPU**: a lei do material vive num sítio só (§34.1).

### §36.5 — ⏳ O que fica

- **O quadro de MOVIMENTO continua na CPU** — de propósito nesta fatia. Ele custa `2,53 ms` no
  dispositivo contra `13,15` aqui, e é a próxima costura.
- **Os `24 ms` ainda não são os `5 ms` do tecto** (§35.4): `49 MB` de leitura por quadro e duas
  paragens de sincronização.
- **Uma peça com escultura ou com duas luzes fica na CPU** — declarado, com gate.

## §37 — ⛔⛔⛔ «BAIXÍSSIMA QUALIDADE»: o grão era MONTE CARLO, e a cura é um CONE (report do dono, 2026-09-15)

> *«baixíssima qualidade. a anos luz da unreal»* — foto do interior de um furo passante, com uma seta
> vermelha a apontar para a parede dele.

### §37.1 — O que a foto mostra, medido

A seta aponta para a **oclusão**. Até este dia ela eram `OCCLUSION_PASSES` **raios binários sorteados
por pixel**, e um estimador binário de `N` amostras tem desvio-padrão `√(p(1−p)/N)`: a `N = 16` e
`p = ½` isso é **`12,5 %` da faixa, por pixel**. O borrão `3×3` desce-o a `~4 %`, e num furo ESCURO,
depois da curva sRGB, `4 %` de erro relativo é a dezena de níveis que a foto tem.

⚠️ **A primeira régua que corri disse que não era isso, e ela é que estava errada.** Medindo o erro
no BYTE sobre remendos lisos, o `p99` lia `2` níveis — um número que se lê como *«invisível»*. O que
ele não media era a **estrutura**: despejando o canal de oclusão numa imagem
(`cargo run --release -p ph2d-field-gpu --example ao_grain`, com `PH2D_AO_DUMP`), ele sai com uma
**textura tecida** visível a olho. *Um `p99` de amplitude não vê um padrão — o olho vê padrões antes
de ver amplitudes, e é por isso que a sonda desta wave despeja uma IMAGEM ao lado da tabela.*

### §37.2 — ⛔ A §31 curou um ENVIESAMENTO e deixou a VARIÂNCIA de pé

A wave anterior (§31) apanhou duas coisas na mesma linha — o azimute que não dependia do raio, e o
bit `0` do pixel a virar o bit mais significativo do azimute — e curou-as. As listras foram-se.

⚠️ **Mas um amostrador sem enviesamento continua a ser um amostrador com ruído**, e as duas coisas
leem-se igual num report: *«a imagem melhorou»*. A §31 mediu convergência (`1/√N` sobre uma
referência de 64 raios) e não mediu **o vizinho**, que é o que o olho faz.

### §37.3 — ⛔⛔ E havia um segundo defeito, PIOR, que régua nenhuma media

O sorteio era semeado no **índice do pixel** (`sample_uv(i, …)`). ⇒ o mesmo ponto da peça recebia
outra estimativa conforme onde aterrava, logo **rodar a câmera repintava o sombreado de contacto**.
Não é grão: é a imagem a mudar ao reenquadrar.

Os dois gates que nascem com esta wave são exactamente esses, e **nenhum deles podia passar com a lei
antiga**:

| gate | o que afirma | a mutação que o mata |
|---|---|---|
| `a_oclusao_de_um_ponto_nao_depende_do_pixel_em_que_ele_cai` | o mesmo `(ponto, normal)` em oito índices dá o mesmo byte, **ao bit** | semear o conjunto no pixel |
| `a_oclusao_de_um_ponto_nao_depende_da_camera` | quatro orientações de câmera, o mesmo ponto, `< 1e-5` | ancorar o conjunto na VISTA |

⭐ A segunda mutação é **discriminante**: ela mata só o gate da câmera e deixa o do pixel verde.

### §37.4 — ⭐⭐⭐ A lei que fica: um CONE por direcção fixa de MUNDO

- as direcções são um **reticulado de Fibonacci esférico** sobre a esfera inteira (`cone_dir`),
  **iguais em todo pixel e em toda câmera**;
- cada uma é traçada como **cone**, com o estimador `min(d / (t·cos))` que a marcha da sombra já
  tinha — o que muda é a **dureza**, que passa a ser **por raio**;
- o peso é `n·d`, e a média é `Σ w·vis / Σ w`.

⭐⭐⭐ **A dureza é `1/(n·d)`, e é ela que faz a lei ser exacta num corpo convexo.** Esse é o cone que
**roça o plano tangente**: num plano a distância ao longo do raio é `t·(n·d)` por identidade, logo o
quociente é `1` exacto; numa esfera é maior que `1`. ⇒ `vis` fica exactamente `1,0` e a média fica
`w/w = 1,0` — **ao bit**, e o gate da esfera (que já existia) afirma-o com `assert_eq!`.

⛔ **A primeira tentativa de cone, na W4, usou `hardness = 1` e foi REJEITADA** porque a esfera lia
`0,698` contra `0,757` da cruz — um corpo convexo a ocluir-se mais que três cilindros cruzados. *A
recusa estava certa sobre aquele número e errada sobre a família: o que não servia era a dureza
CONSTANTE, não o cone.*

⛔ **Um referencial TANGENTE foi considerado e recusado.** Ele daria `N` direcções úteis em vez de
`N/2`, mas toda base construída a partir de `n` tem uma descontinuidade (a que vivia aqui saltava
quando `|n.x|` cruzava `0,9`; a de Duff salta em `n.z = 0`). Com direcções SORTEADAS isso é
invisível — o sorteio já embaralha o azimute. Com um conjunto FIXO vira uma **costura desenhada na
peça**. *Uma base que só era aceitável porque o ruído a escondia deixa de ser aceitável quando se
apaga o ruído.*

### §37.5 — ⭐ O que mudou, medido (sonda `ao_grain`, a peça da foto)

**A `960×540`, com as MESMAS 16 amostras dos dois lados** — `grão` é
`|ao(i) − média dos 8 vizinhos|` sobre remendos lisos:

| | `ms` | grão p50 | p99 | máx |
|---|---:|---:|---:|---:|
| raios sorteados (até 2026-09-14) | `8,2` | `0,0043` | `0,0208` | `0,0486` |
| **cones** | **`6,0`** | **`0,0002`** | `0,0053` | `0,0222` |

⇒ **`21×` no `p50` e mais barato.** ⭐ E o que resta **não é ruído**: é determinístico, e os dois
gates acima afirmam-no.

O número de direcções e o joelho estão na tabela do `OCCLUSION_PASSES` (`16 → 48`, medida a
`1920×1080`): o que mais direcções compram deixou de ser variância e passou a ser **resolução
angular** — o que se evita é **banda**, terraços de nível onde a resposta devia variar devagar.

### §37.6 — ⛔⛔ A paridade apanhou uma LEI escrita só num motor

Com cones, o gate `o_gbuffer_do_dispositivo_e_o_da_cpu` acusou **`0,1450`** de divergência. A causa:
a **cerca da bola** — o domínio da pergunta, documentado na §27.5 — vivia **só no WGSL**. Com raios
binários ela quase nunca vincula; com um estimador que é um **mínimo ao longo do raio**, tudo o que
ele lê para lá da peça entra na resposta. Dando-a à CPU: `0,1450 → 0,0201` na pior cena.

⚠️ *Uma cerca escrita num motor só é uma lei diferente nos dois, e a régua que a não vê é a que corre
com o estimador errado.*

E a barra daquele gate teve de ser **re-derivada**: ela era `2 / OCCLUSION_PASSES`, *dois raios do
quantum binário*. Com cones não há quantum — a fórmula passou a devolver `0,0417` sem nomear recurso
nenhum. Hoje o recurso é a `f32` da fita propagada pelo estimador, com a conta e a varredura do
filtro escritas no gate.

⭐⭐ **E a oclusão passou a comparar-se só onde a GEOMETRIA concorda** (normal `< 1°`, `Δt < 3e-5`),
com piso de população. A razão é a lei nova: a oclusão é **função de `(ponto, normal)`** — onde os
dois motores entregam normais a `9,9°` uma da outra (cena 30, num vinco), eles TÊM de entregar
oclusões diferentes, e essa divergência **já tem barra própria duas linhas acima**. *Gatear a mesma
divergência duas vezes não a mede melhor — mede o acoplamento e chama-lhe defeito do segundo passe.*

### §37.7 — ⛔ Duas recusas medidas

| tentativa | medição | veredito |
|---|---|---|
| sair do cone quando `vis < 0,02` | `6,0 → 6,4 ms` · `11,6 → 13,4` | ⛔ não compra: o que custa é o cone ABERTO, que viaja até à cerca |
| piso no cosseno (`0,05`) para cortar a amplificação da dureza | pior `p99` **exactamente** onde estava (`0,0201`) | ⛔ o amplificador não é a dureza — é o `Δt` do ponto de partida |

### §37.8 — ⏳ O que fica aberto, com o preço

- **O quadro assente passa de `22,0` para `36,5 ms` no dispositivo** (`+11 ms` no total de ponta a
  ponta). ⛔ O quadro de MOVIMENTO não paga nada: o dispositivo só toma o assente.
- ⏳ **A alavanca medida e não construída:** a oclusão é de baixa frequência — é a mesma premissa que
  legitima o `blur_occlusion` —, logo cabe em **meia resolução** com reconstrução guiada pela normal.
  `4×` mais barata, o que poria `96` cones abaixo do preço dos `16` raios de ontem. Traz uma classe
  de artefacto própria (halo na descontinuidade de profundidade) e é wave com espec própria.
- ⏳ **O `blur_occlusion` mudou de razão, e o doc dele já o diz.** Ele existia para apagar ruído de
  amostragem; hoje não há ruído, e o que ele faz é suavizar as **estrias** do conjunto discreto. A
  premissa que o legitima é a mesma; o que **não** foi re-medido é quanto ele vale contra a lei nova
  — a tabela que está lá é da lei antiga e está marcada como história.

## §38 — ⛔⛔⛔ «O AO APAGA AO ROTACIONAR» e «performance aquém de tempo real» ERAM O MESMO (report do dono, 2026-09-15)

> *«smoke OK. Boa qualidade. Mas com performance bem aquém que Unreal e outros renders de tempo real.
> E apagar o AO ao rotacionar a tela»*

### §38.1 — A cerca que eu pus era a causa

A §36 ligou o dispositivo sob **três** condições, e a segunda era *«só o quadro ASSENTE»* — escrita
como cerca contra a regressão do §32 (o gesto lento). ⛔ **O sombreado de contacto só existe no
caminho do dispositivo.** ⇒ enquanto essa cerca existiu, ele **desaparecia a cada gesto** e voltava
ao largar. Não é um defeito de oclusão: é a cerca a fazer exactamente o que diz.

⚠️ *Uma cerca que protege um gesto pode ser o que apaga uma feature nesse gesto, e as duas frases
leem-se igual num doc.*

### §38.2 — ⛔⛔ A medição desmentiu o que este doc dizia sobre o custo

A sonda nova (`cargo run --release -p ph2d-field-gpu --example frame_budget`) separa as fases por
**diferenças** — o compute e a cópia de leitura vivem no mesmo `submit`, logo um relógio à volta
dele mede os dois juntos — e mede a leitura **à parte**, no mesmo volume de bytes.

O item aberto da §36 dizia que o tecto eram os **`49 MB` de leitura por quadro**. Medido: **`1,86 ms`**,
`4 %` do quadro. O tecto era o `DeviceGbuffer::to_cpu` — **`47,44 ms` de `87,58`**, mais do que a
placa (`24,65`) e o pintor (`13,31`) somados.

⇒ *um item aberto que nomeia um recurso sem o ter medido manda optimizar a coisa errada.*

### §38.3 — ⛔⛔ E duas hipóteses minhas caíram, as duas medidas a ZERO

| hipótese | medição | veredito |
|---|---|---|
| a base de **quaternião** por pixel (`ray_at_plane` chama `basis()`) | `47,44 → 48,20 ms` | ⛔ zero — são ~30 operações **sem divisão**, e o compilador já a tirava do laço |
| a **divisão inteira** `i % w` / `i / w` por valor de *runtime*, duas por pixel | `→ 47,96 ms` | ⛔ zero |
| **a normalização**: `o + (v/\|v\|)·t` são **três** divisões `f32` | `37,87 → 5,75 ms` | ⭐ **`6,6×`** |

A cura é algébrica: `o + (v/|v|)·t = o + v·(t/|v|)`, que tem **uma** divisão. Uma divisão em `f32`
tem ~15 ciclos de latência e péssimo débito, e três seguidas não emparelham. Encher os mesmos
`24 MB` de saída custa `0,61 ms` ⇒ o laço curado está a `9×` do custo da memória.

⚠️ Isso é uma **segunda porta** para a mesma pergunta (o `ray_at_plane` declara-se *«a porta
única»*), e ela **não é bit-a-bit** — daí o gate `o_ponto_de_acerto_e_o_mesmo_com_e_sem_normalizar`,
nas duas lentes, com barra **relativa** (a `t = T_MAX` um `ulp` escala com a distância).

### §38.4 — ⭐⭐⭐ O quadro de movimento passa a ser do dispositivo

A cerca sai; o que a substitui **não é uma cerca**, é a lei da W73 a viajar com o pedido: o
`antialias` chega ao `MarchSetup` e o dispositivo **salta o segundo despacho** (a borda
re-amostrada) quando ele é falso — a mesma lei que a CPU já seguia. *Duas metades de uma lei, uma em
cada motor, é a forma como ela morre num deles.* Gate:
`sem_anti_serrilhado_o_dispositivo_nao_reamostra_borda_nenhuma`, com o **controlo primeiro** (sem
ele, `0 == 0` passaria com o passe inteiro partido).

| | antes | agora |
|---|---:|---:|
| quadro **assente** `1920×1080` | `87,6 ms` | **`60,9 ms`** |
| quadro de **MOVIMENTO** `640×360` | `~27 ms` na CPU, **sem oclusão** | **`5,0 ms`** no dispositivo, **com** oclusão |

⇒ `200` quadros por segundo a rodar, com o sombreado de contacto que antes desaparecia.

### §38.5 — ⏳ O que fica

- **`60,9 ms` do quadro assente ainda não é o tecto.** A repartição é: placa `25,1` (oclusão `11,1` ·
  leitura `1,9` · marcha+sombra+bordas `13,5`) · reconstruir+suavizar `21,9` · pintar `14,1`.
- ⏳ **A CPU ainda faz `36 ms` de trabalho por quadro assente** sobre dados que a placa já tem. O
  caminho de um render de tempo real é **sombrear no dispositivo** e devolver a imagem (`8,3 MB` em
  vez de `49,8`), e não devolver o G-buffer inteiro. Isso apaga o `to_cpu` e o pintor de uma vez;
  é wave com espec própria.
- ⏳ **O `to_cpu` curado ainda lê `17,5 ms` onde o laço isolado lê `5,7`** — os `~12 ms` de
  diferença não foram atribuídos.

---

## §39 — ⭐⭐⭐ O PINTOR VAI PARA O DISPOSITIVO: a quarta lei era a do DONO (2026-09-15)

O §38.5 deixou o item aberto com o nome certo: *«o caminho de um render de tempo real é sombrear no
dispositivo e devolver a imagem»*. Esta secção fecha-o.

### §39.1 — ⚠️ O achado que mudou o preço: `owners: None` é o caso de UMA folha

A wave foi orçada supondo que o material atravessava sozinho. Ao ler o `materials.rs`:

```rust
/// De quem é cada ponto. `None` numa peça de uma folha só — ver [`Table::surfaces_for`].
pub owners: Option<ph2d_field_eval::owners::Owners>,
```

⇒ **toda peça real é multi-material**, e a sua cruz de cilindros tem quatro folhas. O `{FIELD}` do
§33 leva **uma** fita — a da peça inteira, que é a união de tudo — e o sombreamento não pergunta
*«onde está a superfície?»*: pergunta ***«de quem é este ponto?»***. Essa resposta precisa de **uma
fita por folha**, da bola de cada uma, e do mesmo desempate que a CPU corre.

⛔ **E a recusa do *id-buffer* continua de pé:** ela arrastaria um segundo canal por cada passo da
marcha. Aqui o dono resolve-se **uma vez por ponto**, no passe que pinta, como na CPU.

### §39.2 — ⛔⛔ Porque a reescrita de TEXTO não serve, e a cura é uma linha

A forma óbvia de ter `N` fitas é gerar `N` vezes com o emissor que já existe e **reescrever o
texto** (`fn field` → `fn folha_3`, `k[7]` → `k[62]`). Ela falha de duas maneiras **mudas**:

| | modo de falha |
|---|---|
| o nome | `field` aparece dentro de **todo identificador que o contenha** |
| a origem | uma expressão regular sobre `k[i]` não distingue `k[7]` de `k[70]` sem varrer da direita para a esquerda |

⇒ `PointTape::to_wgsl_named(nome, const_base)`: o nome e a origem entram **onde o texto é escrito**,
que é o único sítio que sabe o que cada coisa é. *Uma reescrita de texto é uma segunda análise do
que o gerador já sabia.*

⚠️ **E quem escolhe o `const_base` é quem CONCATENA os vectores** — o `marcha_com`, num sítio só. A
origem que o texto indexa e a ordem da concatenação são a **mesma decisão**, e duas respostas pintam
cada folha com os números da vizinha **sem erro nenhum**.

### §39.3 — A paridade, lei a lei

| lei | barra | medido |
|---|---:|---:|
| material (OpenPBR completo, §33-bis) | `1e-4` relativo | `1,05e-6` |
| céu do estúdio | `1e-4` relativo | `2,42e-7` |
| olhar (exposição + vista) | `1e-4` relativo | `7,80e-6` |
| **dono** (`a`, `b`, `t`) | `2e-3` em `t`, `100 %` nos índices | **`2,41e-6`** · **`0` trocas** em `1 041` |
| **a IMAGEM inteira** | `≤1` nível em `99,5 %`, pior `2` | **`100,000 %`** · pior **`1`** |

⚠️ **O gate da imagem ISOLA o sombreamento:** os dois lados correm sobre a **MESMA marcha do
dispositivo** — o mesmo `t`, a mesma normal, a mesma sombra, a mesma oclusão. O que sobra entre eles
é só o pintor. Um gate que traçasse cada lado no seu motor mediria as duas coisas somadas.

⚠️ **E a lei do dono precisou de arnês PRÓPRIO porque a imagem é cega a ela:** numa peça de duas
folhas da mesma cor o dono errado pinta exactamente o mesmo pixel. *Um zero de «igual» e um de
«nenhum dos dois olhou» são o mesmo byte* — a mesma armadilha que a `supports` tapa um nível acima.

### §39.4 — ⚠️ Uma mutação SOBREVIVEU, e ela nomeou o corpus

Trocar `cru > piso` por `cru > 0` no piso da luz-objecto passou os gates: na cena de medição a
lâmpada está a duas meias-extensões da peça e **nenhum pixel chega a `0,05` dela**. *Um corpus no
ponto NEUTRO de um knob não testa esse knob.*

⇒ gate próprio (`a_lampada_encostada_a_peca_concorda_nos_dois_motores`) com a lâmpada **sobre a
superfície**, e um piso de população que conta quantos pixels estão de facto abaixo do piso (`18`).
Com ele, a mesma mutação lê **`166` níveis** de divergência.

### §39.5 — ⚠️ E o portão de LOC estava VERMELHO havia waves

Seis ficheiros acima do tecto de `700`, e os seis são crescimento **desta linha** — nenhum estava
acima no merge-base. Curados por **corte por responsabilidade**, nunca por isenção:

| ficheiro | antes | depois | a fronteira |
|---|---:|---:|---|
| `app-field3d/smoke.rs` | `1 161` | `618` | o roteador das cenas ≠ os gates que as medem |
| `field-render/shadow.rs` | `766` | `290` | *«esta LÂMPADA vê?»* ≠ *«quanto do HEMISFÉRIO vê?»* |
| `app-field3d/smoke_draw.rs` | `759` | `520` | quem ARMA o pedido ≠ quem o RESPONDE, fora da thread |
| `app-field3d/scene_edit_tests.rs` | `736` | `384` | ⚠️ por TETO e não por assunto — e dizê-lo é mais honesto |
| `app-field3d/scene_panel.rs` | `735` | `509` | os NÚMEROS do escolhido ≠ os CHIPS |
| `field-gpu/trace.rs` | `781` | `654` | o texto do shader ≠ quem o despacha |

⚠️ **E o `#[allow(clippy::…)]` do `marcha_com` colou-se ao VIZINHO** durante o corte — a mesma forma
do `#[cfg]` órfão que a memória do repo já regista, uma família de atributos ao lado. A isenção
mudou-se em silêncio para uma função de doze linhas.

### §39.5-bis — O relógio, e porque UMA das colunas não se pode ler

⚠️⚠️ **A máquina esteve entre `load 24` e `load 63` a jornada inteira** (outras linhas a compilar), e
nenhuma leitura de relógio deste repositório vale nada acima de `~5`. ⇒ a tabela traz o **mínimo de
7 corridas** com a carga ao lado, e as duas colunas **não** se leem da mesma maneira:

| `1920×1080`, a mesma peça | `load 58` |
|---|---:|
| marcha + G-buffer de volta (`49,8 MB`) | `108,2 ms` |
| … mais o pintor na CPU | `136,2 ms` |
| **marcha + PINTOR no dispositivo** (`8,3 MB`) | **`4,37 ms`** |

⭐⭐⭐ **A coluna do dispositivo é INSENSÍVEL à carga e a da CPU não é** — e isso não é uma desculpa,
é a medição: `4,37`, `4,76`, `5,16`, `5,22`, `5,29 ms` em cinco corridas entre `load 24` e `load 63`
(`±10 %`), contra `136`, `156`, `170`, `222 ms` do outro lado (`±63 %`) sobre a MESMA árvore. *Uma
régua cujo valor depende de quem mais está a usar a máquina está a medir a máquina, não o código.*

⇒ o que se pode afirmar com a máquina neste estado é o **piso** do ganho, e ele já é o que interessa:
o quadro assente do modelador passou a caber num quadro de `60 Hz` com folga, e o barramento por
quadro caiu de `49,8` para `8,3 MB` — esse número é exacto e não tem relógio nenhum dentro.

⏳ **O número da CPU tem de ser re-medido com a máquina calma** antes de entrar em qualquer tabela
comparativa. O da placa não.

### §39.6 — ⏳ O que fica

- ⏳ **Uma lâmpada só.** O shader tem um canal de sombra; com duas, a segunda ficaria sem sombra em
  silêncio, e é por isso que o chamador **cai na CPU** em vez de a ignorar.
- ⏳ **O refinamento de CPU mantém o caminho antigo.** Com `PH2D_FIELD_CPU_OCCLUSION` ligado o quadro
  precisa do G-buffer para refinar sobre ele — é a única razão que sobra para o trazer de volta.
- ⏳ **Uma peça com ESCULTURA continua na CPU** (`ph2d_field_gpu::supports`).
- ⚠️ **O `hits` do caminho pintado conta PIXELS COM TINTA**, e não acertos de centro: o G-buffer
  ficou no dispositivo. É número de diagnóstico, e dizê-lo é mais honesto do que trazer `49,8 MB`
  para o calcular.

---

## §40 — ⭐⭐⭐ VÁRIAS LÂMPADAS, e o tecto que sai da PLACA (ordem do dono, 2026-09-15)

O §39.6 deixou *«uma lâmpada só»* como item aberto, com o mecanismo escrito: o canal de luz era um
`vec2` por pixel — o céu e **UMA** sombra — e com duas a segunda ficava sem sombra **em silêncio**.
*Um formato que não tem onde pôr a segunda resposta é um tecto escrito em bytes.*

### §40.1 — O que mudou

- o canal passa a ter passo **`1 + n_lamps`**: o céu no slot `0`, uma visibilidade por lâmpada a
  seguir; a marcha faz uma sombra por lâmpada e o pintor soma as radiâncias, cada uma com a direcção
  e a distância do **ponto** daquele pixel;
- o `DeviceGbuffer::shadow` sai por **bloco de lâmpada** (`l · pixels ..`), que é exactamente a forma
  que o `Shadows::set_lamp` recebe — *o formato que o consumidor pede é o formato que se escreve*;
- o literal `8` do `array<vec4, N>` do WGSL passa a **derivar** do `MAX_LAMPS`.

### §40.2 — ⛔⛔ O tecto que eu escrevi estava errado nos DOIS sentidos

Escrevi `MAX_LAMPS = 8` e disse que o recurso era o **relógio**, com a conta *«o passe cresce ~1
traçado por lâmpada»*. Medido a `1920×1080`, `load 3,0`, mínimo de sete:

| lâmpadas | 1 | 2 | 4 | 8 | 12 |
|---|---:|---:|---:|---:|---:|
| quadro | `4,12` | `4,34` | `4,74` | `7,15` | `7,70 ms` |

⇒ `12` lâmpadas custam **`1,8×`** uma, não `12×`: a marcha de sombra só corre nos pixels que acertam
**e** que vêem aquela luz. *Um tecto derivado de uma estimativa em vez de uma medição erra para o
lado de dentro.*

⭐⭐⭐ **E o recurso de verdade apareceu quando a varredura passou de `12`: foi a PLACA que o disse.**

```text
Buffer binding 3 range 141004800 exceeds `max_*_buffer_binding_size` limit 134217728
```

O canal de luz é `pixels · (1 + n_lamps) · 4 B`, logo o tecto é `limite / (pixels · 4) − 1` — e ele
**depende da resolução**. ⇒ `lamps_that_fit(limite, w, h)`, com o limite **perguntado à placa**.

### §40.3 — ⚠️ E o «limite da placa» era o piso da especificação

Ao medir o passe da escultura, a `wgpu` recusou outra coisa:

```text
Too many bindings of type StorageBuffers in Stage COMPUTE, limit is 8, count was 9
```

⛔⛔ **Eu pedia `wgpu::Limits::default()`, que é o PISO GARANTIDO da especificação** (pensado para a
Web), e não o que esta placa tem. Trocado por `adapter.limits()`:

| | `Limits::default()` | esta placa |
|---|---:|---:|
| maior armazém ligável | `134 217 728 B` | **`2 147 483 644 B`** (`16×`) |
| armazéns por shader | `8` | o que o passe precisa (`9`) |

⇒ o tecto de lâmpadas a `1920×1080` passou de `15` para `32` (o do uniforme) **sem uma linha de
algoritmo mudar**, e o passe que pinta com escultura passou a caber. *Nunca deixe o caminho mais
lento definir o tecto do mais rápido* (`CLAUDE.md` §0.0) — e o «caminho mais lento» aqui era uma
constante da biblioteca que eu nunca tinha lido.

### §40.4 — ⚠️ E a primeira redacção do controlo reprovou por causa do CORPUS

Passar de `1` para `2` lâmpadas moveu `811` canais, abaixo da barra. A leitura fácil — *«o
dispositivo ignora a segunda»* — estava **refutada na linha de cima**: a paridade com a CPU lia
`100 %` com `pior 0`, e a CPU usa as duas. O que estava errado era a **constelação**, que punha a
segunda **atrás** da peça. Hoje ela é um prefixo de uma lista fixa no hemisfério que o olho vê.

---

## §41 — ⭐⭐⭐ A ESCULTURA ATRAVESSA: a folha que não é uma expressão (ordem do dono, 2026-09-15)

O §39.6 deixou *«uma peça com ESCULTURA continua na CPU»*, e a `supports` dizia porquê: a álgebra da
`fidget` é **fechada** (`Input` · `Const` · `Binary` · `Unary` · remapeamentos, **sem consulta a
dados**), logo o compilador traduzia uma escultura para `Tree::constant(ABSENT)` — **espaço vazio**.

### §41.1 — ⭐⭐⭐ A cura: a escultura entra como uma VARIÁVEL

A `fidget` tem variáveis genéricas (`Var::V`), e um remapeamento de eixos **não lhes toca**. ⇒ cada
escultura vira uma variável, a árvore continua a ser **uma só** — com todos os filetes, chanfros e
booleanas onde sempre estiveram — e o gerador de WGSL traduz a variável numa **chamada**
(`escultura_k(p)`), cujo corpo o `ph2d-field-gpu::sculpt` escreve.

⚠️⚠️ **É isto que evita uma TERCEIRA cópia da lei das booleanas.** Ela já existe duas vezes (a árvore
e o `hybrid_law`, com o `the_numeric_law_is_the_same_law_as_the_tree` a segurá-las); portá-la para
WGSL seria uma terceira. *Aqui o dispositivo corre a MESMA árvore que a CPU analítica corre — a
escultura é que é uma folha.*

⚠️ **A pose e a pilha de um `Combine` MISTO continuam a não correr, e isso é PARIDADE.** O `hybrid`
declara-o por escrito; reproduzir o limite é o que faz as duas imagens baterem.

### §41.2 — A grade, e porque ela tem CACHE

Uma grade de `128³` são `8 MB`. Reenviá-la por quadro custaria **mais barramento do que a imagem
inteira** que o §39 veio poupar (`8,3 MB`) — isto é, a escultura teria desfeito aquela wave sem mover
um número que alguém estivesse a olhar. ⇒ ela sobe **uma vez** e fica, com a identidade a ser o
ponteiro do `Arc`.

⛔⛔ **E o cache guarda referências FORTES.** Sem elas a escultura podia morrer, o alocador devolver o
mesmo endereço a outra, e o cache servir a grade errada **sem erro nenhum**. *Um cache que compara
endereços tem de impedir que eles sejam reciclados.*

⚠️ **O custo é invisível na imagem** (os dois caminhos desenham o mesmo pixel), e por isso ele tem
gate próprio: `um_arrasto_nao_reenvia_a_escultura` lê `[1, 1, 1, 1, 1, 1]` em seis quadros de
arrasto, e `[1, 2, 3, 4, 5, 6]` com o cache desligado.

### §41.3 — ⛔⛔ O gate do G-buffer comparava a cena da PONTE VAZIA — desde que existe

A linha da cena `6` lia `0,000 %` em **todas** as colunas, incluindo *«a variação da PRÓPRIA peça»*.
A causa: o registo de esculturas do gate era um `Registry::new()` do topo do ficheiro — **vazio** —, e
quem o enche é o construtor da cena. ⇒ os dois motores desenhavam espaço vazio e concordavam sobre um
buraco.

⚠️ *Um zero de «igual» e um de «nenhum dos dois desenhou nada» são o mesmo byte* — a mesma frase que
a `supports` já tinha escrita, uma wave antes, sobre este mesmo assunto. ⇒ o registo lê-se **depois**
de a cena nascer, e o gate ganhou um **piso de população** (`1 %` da tela) que teria apanhado isto.

### §41.4 — ⚠️ Duas mutações SOBREVIVERAM, e nomearam o corpus

Apagar a translação/escala da pose (`q = (p − t)/s` → `q = p`) e apagar o `· s` do retorno passaram
o gate da cena `6`: ali a escultura está na **identidade**. *Um corpus no ponto NEUTRO de um knob não
testa esse knob* — a terceira vez que esta linha paga a mesma forma.

⇒ gate próprio (`a_escultura_posta_e_a_mesma_nos_dois_motores`) com a MESMA grade posta com
translação, rotação de `40°` em torno de `(1,1,1)` e três escalas. Com ele, **7 de 7** mutações
morrem (as duas acima, mais a parede, a trilinear, a ordem da grade e a rotação transposta).

### §41.5 — O que se mede

| cena da PONTE, `1920×1080` | |
|---|---:|
| CPU inteira | `515,6 ms` |
| **dispositivo** | **`15,4 ms`** |

⚠️ `load 14`, mínimo de sete corridas: a coluna da CPU está inflada pela carga e a do dispositivo não
(ver §39.5-bis). O que se afirma é o **piso** do ganho.

| paridade | |
|---|---:|
| silhueta (cena `6`) | `0,000 %` |
| Δt p99 | `9,94e-5` |
| Δnormal p99 | `0,24°` (a peça varia `27,90°` entre vizinhos) |
| a escultura **pintada** | `100,000 %` dos canais a `≤1` nível, pior `0` |

### §41.6 — ⏳ O que fica

- ⏳ **Uma folha amostrada sem grade cai na CPU** — o default do trait é `None`, que é o lado seguro;
- ⏳ **Uma placa com menos de `9` armazéns por shader** não pinta no dispositivo (recusa em voz alta);
- ⏳ o `Combine` misto continua a ignorar a pose própria, **nos dois motores** — item do `hybrid`, não
  desta wave.

---

## §42 — ⛔⛔⛔ UMA REGRESSÃO QUE ESTA LINHA IA DEIXAR PASSAR: a peça DESENHADA na placa (2026-09-15)

Com o quadro inteiro no dispositivo, a lista de abertos do módulo passou a descrever outro programa
— ela foi escrita quando a marcha era `80 %` de um quadro de CPU. Antes de escolher o passo seguinte,
medi onde o tecto está agora. O que apareceu não foi um item de melhoria: foi **uma regressão**.

### §42.1 — As cenas lentas têm causas DIFERENTES, e três colunas separam-nas

| cena | fita | vivos | passos/acerto | o que a torna lenta |
|---|---:|---:|---:|---|
| `2` · cubo | `31` | `8` | `16` | — (a mais barata) |
| `4` · perfil DESENHADO extrudado | **`2 899`** | `296` | `79` | a **fita** |
| `5` · o mesmo perfil TORNEADO | **`2 972`** | **`464`** | `42` | a fita **e** a ocupação |
| `27` | `854` | `110` | `55` | — (cabe) |
| `28` · superfórmula | `766` | `34` | **`410`** | o **minorante**: o campo não é uma distância honesta |

⚠️⚠️ **Três hipóteses minhas caíram antes desta tabela existir**, e cada uma media uma **procuração**:
*«é o tamanho da fita»* (a `28` tem `766` e custa como a `4`, que tem `2 899`), *«são as
transcendentais»* (a `25` tem `40` e custa `13 ms`), *«é o passo da marcha»* (a `5` e a `27` têm o
mesmo passo e a mesma cerca, e custam `130` contra `17 ms`). ⇒ *a grandeza que decide é a que se
conta DIRECTO*, e ela só apareceu quando parei de a inferir.

### §42.2 — ⛔⛔ E a curva do perfil não é linear: ela cai de um DEGRAU

Um polígono extrudado de `N` arestas — a família que o `+ Extrude` do artista produz —, a
`1920×1080`. A coluna que interessa é o custo **por aresta**:

| arestas | instruções | vivos | quadro | ms/aresta |
|---:|---:|---:|---:|---:|
| `32` | `1 043` | `163` | `20,6 ms` | `0,642` |
| `64` | `2 063` | `320` | `42,1 ms` | `0,657` |
| `128` | `4 083` | `623` | `130,2 ms` | `1,017` |
| `144` | `4 589` | `700` | `158,0 ms` | `1,097` |
| **`152`** | `4 846` | **`743`** | `186,6 ms` | **`1,227`** ⬅ o último deste lado |
| `160` | `5 100` | `779` | `270,4 ms` | **`1,690`** ⬅ `+36 %` por `+5 %` de arestas |
| `256` | `8 124` | `1 230` | `1 224,5 ms` | `4,783` |

⭐ **O salto é DISCRETO e não gradual** — `+36 %` de custo por `+5 %` de trabalho —, que é a
assinatura de a **ocupação** cair um degrau, e não de mais aritmética. O recurso é o **ficheiro de
registos**: o `vivos` é o scratch por thread, e ele decide quantos fios cabem num multiprocessador.

⛔⛔ **E acima do degrau o dispositivo é MAIS LENTO que a CPU que ele substituiu:** a `256` arestas
mediu-se **`0,20×`** — cinco vezes pior. *Esta linha pôs o quadro na placa e ia deixar a peça
desenhada do artista ficar mais lenta do que era, no topo da faixa que o slider dele alcança.*

### §42.3 — A cerca, e ⛔⛔ o CRITÉRIO que eu usei primeiro estava errado

`ph2d_field_gpu::MAX_VIVOS = 358`: acima dele o quadro fica na CPU.

> ⛔⛔⛔ **ESTE NÚMERO, ESTE EIXO E A CERCA INTEIRA MORRERAM EM 2026-09-15 — ver a §43.5 e a §43.8.**
> Duas coisas de uma vez: o `vivos` era uma propriedade da **ordem de emissão** da fita (com o
> escalonador ele lê `33` e `35` nos dois lados da suposta travessia), e a régua que mediu a
> travessia **pedia trabalhos diferentes aos dois motores** — o quadro pintado à placa, só o traçado
> à CPU, com o traçador a `opt-0`. Medido com a régua inteira, **não há travessia**: a placa ganha
> `1,3×`–`7,8×` e as cenas do produto `2,6×`–`98×`. ⇒ a cerca saiu e no lugar dela ficou um gate
> sobre a propriedade.
>
> ⚠️ **A §42 fica como está de propósito**: ela é o diagnóstico que separou as causas, e a
> regressão que ela nomeia é REAL — a fita **crua** atravessa mesmo, a `96` arestas, e cai a
> `0,29×`. *O que estava errado era o ponto e a magnitude, não a existência.*

⚠️⚠️ **A primeira redacção escreveu `743`**, tirado do joelho da curva **do próprio dispositivo**
(onde o custo por aresta dobra) — porque a coluna da CPU não era medível na altura. Quando a máquina
acalmou (`load 2,6`), ela ficou:

| arestas | vivos | dispositivo | CPU | razão |
|---:|---:|---:|---:|---:|
| `48` | `239` | `30,2 ms` | `47,0 ms` | `1,56×` |
| `64` | `320` | `41,0 ms` | `54,2 ms` | `1,32×` |
| **`72`** | **`358`** | `49,6 ms` | `54,7 ms` | **`1,10×`** ⬅ o último que a placa ganha |
| `80` | `395` | `57,3 ms` | `57,3 ms` | **`1,00×`** ⬅ a travessia |
| `128` | `623` | `127,9 ms` | `71,3 ms` | `0,56×` |
| `256` | `1 230` | `1 226,7 ms` | `92,0 ms` | `0,07×` |

⛔ **A `743` o dispositivo já estava a `0,41×` da CPU** — a cerca deixava passar peças **duas vezes e
meia mais lentas** do que o caminho que ela existe para proteger.

⭐⭐ *O joelho de uma curva e a TRAVESSIA de duas curvas não são o mesmo ponto, e eu usei o primeiro
por não conseguir medir o segundo.* ⇒ quando a medição que falta é a que decide, o número que se
escreve entretanto é **provisório e tem de ser marcado como tal** — este não foi, e shipou.

⭐ **E a coluna da CPU explica a travessia ser tão cedo:** ela é quase PLANA (`47 → 92 ms` para `8×`
as arestas), porque ali o contorno já é uma **consulta** (`profile_index`: BVH mais grelha de
enrolamento) e não uma cadeia desenrolada. *A placa não perde por ser lenta — ela perde contra uma
estrutura de dados que a fita dela não tem.*

⚠️ **Sobre `vivos` e não sobre instruções**: a cena da superfórmula tem `766` instruções e apenas
`34` vivos, e é lenta por outra razão. *Um tecto sobre as instruções mandaria essa peça para a CPU
sem curar nada* — e deixaria as `128` arestas, que a placa ainda ganha por `1,6×`, também lá.

⚠️ **E o `tecto` é um PARÂMETRO da porta** (`gpu_frame::paint_com_tecto`), porque a sonda que o
calibra tem de o poder atravessar: *um número que ninguém consegue voltar a medir é um palpite com
data.*

### §42.4 — O que a régua do relógio conseguiu, e o que só conseguiu depois

⭐ **A curva do dispositivo repetiu-se a `load 43`, `91` e `111`** — as três corridas dão os mesmos
números até ao segundo decimal. ⇒ ela é uma propriedade do shader, e a cerca que sai dela é medida.

⛔ **A travessia com a CPU não era** — ali o mesmo traçado leu `71` e `482 ms` na mesma corrida
(outra linha a correr a suíte dela). *Uma régua que varia `7×` entre corridas do mesmo código não
mede código.*

⭐ **Ela ficou medível quando a outra linha acabou a suíte** (`load 2,6`), e foi ela que corrigiu o
tecto de `743` para `358` — ver a §42.3. ⚠️ *Esperar pela calma «nunca chega» é verdade como regra e
falso como lei: aqui chegou, e o que ela trouxe foi o número que decidia.*

### §42.5 — ⏳ O que fica

- ✅ **O `MAX_VIVOS` SAIU em 2026-09-15, e não foi substituído por outro número** — ver a §43.8. A
  sonda que o derivava fica, com as duas metades da régua curadas;
- ⛔⛔ ~~**A cura de fundo é a que a CPU já usa**: o contorno passa a ser uma consulta … só uma peça
  **sem modificadores no contorno** a poderia usar~~ — **REFUTADO em 2026-09-15 (§43).** Nem a cura
  nem o preço estavam certos: o que segurava a peça desenhada era a **ordem da fita**, e curá-la
  custou **zero** ao produto. *A nota media uma procuração, e o preço dela também.*
- ⏳ **O minorante da superfórmula** (`410` passos por acerto) é a outra alavanca, e ela **não se
  substitui** à primeira: encurtar a fita do perfil não tira um passo à superfórmula.

---

## §43 — ⭐⭐⭐ A CURA DA PEÇA DESENHADA NÃO ERA UMA ESTRUTURA DE DADOS: ERA A **ORDEM** DA FITA (2026-09-15)

A §42.5 prescreveu a cura e escreveu o preço dela ao lado: *«o contorno passa a ser uma consulta …
uma folha de dados não passa pela pilha de modificadores, logo só uma peça **sem modificadores no
contorno** a poderia usar»*. ⛔ **Esse preço não era necessário, e a nota media uma PROCURAÇÃO.**

### §43.1 — O mecanismo, contado e não cronometrado

O `vivos` de uma fita é o pico de valores vivos **na ordem em que ela é emitida**, e essa ordem é a
da travessia da `fidget`: uma DFS que empilha os filhos e emite o de cima primeiro. Sobre a cadeia
esquerda `min(min(min(s₀, s₁), s₂), s₃)` que o `profile::sd_profile_inner` constrói, isso dá

```text
ordem da DFS :  s₃  s₂  s₁  s₀  min  min  min      ⇒ os N segmentos VIVOS ao mesmo tempo
ordem óptima :  s₀  s₁  min  s₂  min  s₃  min      ⇒ um acumulador e um segmento: DOIS
```

⇒ o `vivos` **não era uma propriedade do grafo**; era uma propriedade da ordem de visita. E como
`vivos` é o scratch por thread, era ele que decidia a ocupação — logo o degrau da §42.2 tinha uma
causa que não estava no algoritmo nenhum.

⭐ A cura é o [`ph2d_field_eval::tape_schedule`]: um **escalonamento de lista** com duas chaves —
`delta` (quantos operandos MORREM ao emitir este nó, que é a variação exacta do número de vivos) e,
no empate, o **caminho crítico** até à raiz. ⚠️ A segunda chave não é decoração: no primeiro passo
os prontos são `x`, `y`, `z` e **todas** as constantes, e todos têm `delta = +1`; é o caminho crítico
que sabe que `s₀` é o segmento que a cadeia quer primeiro. ⛔ **A ordem original da fita seria um
desempate mudo e ERRADO** — ela põe `s_{N-1}` primeiro, exactamente ao contrário do que a cadeia
consome. *Um desempate que depende da ordem de iteração de outra crate não é uma lei, é um acidente.*

### §43.2 — E metade do pico não eram registos: eram CONSTANTES

A primeira medição do escalonador leu `93` vivos a 256 arestas, e o diagnóstico por espécie dizia
**`Const 51` · `Sub 32`**. ⇒ mais de metade do «scratch por thread» eram `k[i]`, que no shader é uma
**leitura de buffer** e não um valor guardado — o emissor é que lhes dava um `let`.

⭐ Duas metades de uma lei só ([`Instr::ocupa_registo`]): o `wgsl` escreve `p.x` e `k[7]` **onde são
usados**, e o `TapeShape::vivos` e o escalonador **contam o mesmo**. *Uma régua que conta o que o
código emitido não guarda mede outro programa.* ⚠️ A escultura fica **de fora** da isenção, e não por
simetria: `escultura_k(p)` é uma consulta de oito amostras a uma grade, e reescrevê-la em cada uso
duplicaria o trabalho. ⭐ De borla, a fita em WGSL encolheu `~15 %` (`1 040 → 882` linhas a 32
arestas).

### §43.3 — A tabela que decide, e ela é uma CONTAGEM (vale com a máquina cheia)

| arestas | instruções | vivos CRU | vivos ESCALONADO | razão |
|---:|---:|---:|---:|---:|
| `32` | `1 040` | `68` | `28` | `2,4×` |
| `64` | `2 060` | `130` | `29` | `4,5×` |
| `128` | `4 080` | `251` | `33` | `7,6×` |
| `256` | `8 121` | `492` | **`48`** | **`10,2×`** |

⭐⭐ **A ordem crua é LINEAR nas arestas** (`68 → 492`, `7,2×` para `8×` o trabalho) e a escalonada é
quase **plana** (`28 → 48`). ⇒ o `vivos` deixou de ser uma propriedade do *número de arestas* e
voltou a ser uma propriedade da *peça*, que é o que a placa precisa.

⚠️ **A barra do gate é no MAIOR contorno, e não «em todos»**: a `32` arestas a ordem crua já tem
pouco para desperdiçar (`68` vivos), e exigir `8×` ali seria uma barra que a aritmética não pode dar.

### §43.4 — E nas CENAS REAIS: a pior passou de `464` para `95` vivos

O `measure_tape_shape_of_the_real_scenes` sobre as 17 cenas do smoke: pior `2 969` instruções,
**`2 663`** guardados e **`95`** vivos (era `464`), com as duas peças desenhadas — a cantoneira
(`2 896` ops, `2 663` guardados) e o torno (`2 969` / `2 504`) — a medir **`69`** e **`33`** vivos.

⭐⭐ **E é o TORNO que troca de lado:** a `464` vivos ele estava **acima** do tecto antigo, logo o
quadro dele caía na CPU; hoje mede `2 504` guardados contra um tecto de `3 463`. *A peça que a §42
usou para nomear a regressão é a primeira que a cura devolve à placa.*

⭐⭐⭐ **Isso DESMENTE uma recusa medida que estava escrita no emissor:** *«interpretar a fita no
dispositivo não cabe — `464` vivos são `1 856 B` por thread e `116 KB` por workgroup de 64, acima da
memória partilhada de qualquer GPU»*. Com `95` são `380 B` e **`23 KB`**, que **cabe**. ⇒ §0.0:
*quem move o número que tornava algo inalcançável tem de reconferir a nota.* A recusa **fica**, e o
motivo que sobra é outro (um interpretador paga descodificação por amostra e perde a fusão que o
`naga` faz) — *o que mudou é que a rota deixou de ser impossível e passou a ser uma medição por
fazer.*

### §43.5 — ⛔⛔⛔ A RÉGUA QUE DECIDIU TUDO ISTO ESTAVA PARTIDA EM DUAS METADES

Antes de qualquer relógio desta wave valer, dois defeitos **na sonda**. Nenhuma das três discussões
que corrigiram o tecto (`743 → 358 → 3 463`) foi sobre eles — foram todas sobre **quando** medir
(carga, calma), e nenhuma sobre **o que** estava em cada coluna.

1. ⛔ Ela media, do lado da placa, o **quadro pintado inteiro** (G-buffer + sombra + sombreamento +
   bordas) e, do lado da CPU, **só o traçado**. O quadro de CPU do produto são **três** passos
   (`smoke_draw_thread`): `trace` + `shadow_pass` + `shade_render`. ⇒ *pedia-se três à placa e um à
   referência.*
2. ⛔ A `ph2d-field-render` e a `ph2d-material` **não estavam** na lista de `opt-level = 2` do
   `Cargo.toml` da raiz: o traçador e a lei OpenPBR corriam a `opt-0` — a própria nota daquela lista
   mede **`11,4×`–`25×`** para aritmética por-amostra — contra um WGSL optimizado pelo driver.

⇒ o caminho de referência lia-se **lento na estrutura** da medição e **rápido no conteúdo** dela, e
a «travessia» era o saldo dos dois erros.

⚠️ **A junção ao `opt-level` estava RECUSADA desde 10/09** (*«parte um teste: `quadro MORNO … pagou
4`»*) — e esse teste foi **curado em 13/09**: ele contava um memo atrás de estado **por thread**, e
hoje corre numa pool de UMA thread. Medido agora: `24/24` verde. *§0.0 — quem move o número que
tornava algo inalcançável tem de reconferir a nota.*

### §43.6 — ⭐⭐⭐ E COM AS DUAS METADES CURADAS, O TECTO FICA SEM SUJEITO

A `1920×1080`, CPU a `95`–`99 %` ociosa, duas rondas consistentes:

| arestas | guardados | CRUA | **escalonada** | ganho | CPU (quadro inteiro) | placa vs CPU |
|---:|---:|---:|---:|---:|---:|---:|
| `32` | `879` | `28,9 ms` | **`23,7`** | `1,2×` | `181,4` | **`7,65×`** |
| `64` | `1 741` | `189,8` | **`49,1`** | `3,9×` | `345,2` | **`7,03×`** |
| `96` | `2 602` | `520,2` | **`87,1`** | `6,0×` | `463,6` | **`5,32×`** |
| `128` | `3 462` | `379,1` | **`181,0`** | `2,1×` | `609,7` | **`3,37×`** |
| `192` | `5 189` | `1 594,4` | **`919,2`** | `1,7×` | `884,1` | `0,96×` ⬅ o penhasco |
| `256` | `6 903` | `4 145,0` | **`631,0`** | `6,6×` | `1 243,9` | **`1,97×`** |
| `384` | `10 351` | `5 851,4` | **`1 306,4`** | `4,5×` | `1 703,1` | **`1,30×`** |

⭐⭐⭐ **Não há travessia: o pior ponto é um EMPATE.** E nas 17 cenas do produto a placa ganha
`2,58×` a `98×` — as duas desenhadas a `2,58×` (a cantoneira) e `4,98×` (o torno).

⭐⭐ **E a regressão que a §42 nomeou era REAL — o que estava errado era o PONTO.** Contra o quadro
inteiro e optimizado, a fita na ordem **CRUA** atravessa a `96` arestas e cai a **`0,29×`** a `384`:

| arestas | `32` | `64` | **`96`** | `128` | `192` | `256` | `384` |
|---|---:|---:|---:|---:|---:|---:|---:|
| crua vs CPU | `6,3×` | `1,8×` | **`0,89×`** | `1,6×` | `0,55×` | `0,30×` | `0,29×` |

⇒ *é o escalonador que remove o sujeito da cerca, não a correcção da régua.*

⚠️ **O penhasco de `192` é real e reproduz-se** (`919,2` e `888,1` em duas rondas, contra `181,0` a
`128` — `5,1×` de relógio por `1,5×` de trabalho, e depois **recupera** para `631` a `256`). Ele
**não é monótono** e move-se com o código emitido: no A/B do inlining ele apareceu a `144` sem o
inlining e a `192` com ele. *A assinatura é o tamanho do shader, não a ocupação* — o `vivos` ali lê
`35`, menos que a `176`.

### §43.7 — ⛔⛔ E o ESCALONADOR tem um preço que corre A CADA QUADRO

O `pedido` reconstrói a fita **por quadro** (é um JIT: o contador é o `POINT_TAPES`). A 1.ª redacção
do escalonador varria a lista de prontos a cada passo, e o doc dela **defendia-se por escrito** —
*«uma varredura linear e não um monte: a chave muda quando um vizinho é emitido, e um monte com
chaves obsoletas é um defeito mudo»*. ⚠️ Verdade para uma chave qualquer, **falso para uma
monótona**: o `delta` só **desce** (o `restam` de um operando nunca sobe), logo um balde com a chave
de entrada e uma re-conferência à saída é **exacto**.

O preço dessa prudência estava medido e ninguém o tinha contado:

| arestas | montagem CRUA | varredura linear | **baldes** |
|---:|---:|---:|---:|
| `32` | `0,328 ms` | `0,556` (`+68 %`) | `0,386` (`+17,6 %`) |
| `64` | `0,661` | `1,271` (`+92 %`) | `0,802` (`+21,3 %`) |
| `128` | `1,368` | `3,464` (`+156 %`) | `1,716` (`+25,4 %`) |
| `256` | `2,854` | `11,224` (**`+301 %`**) | `3,472` (**`+21,6 %`**) |

⭐ O acréscimo passa a ser **plano na dimensão** — o escalonador entra na mesma classe de custo que
a montagem que ele reordena. ⚠️ *Um custo que nenhuma sonda conta é um custo que nenhuma mutação
mata*, e este só apareceu porque a sonda foi escrita **de propósito** para a pergunta *«o que isto
custa a quem o paga?»*.

⭐⭐ **E esta corrida deu a terceira confirmação da régua da calma:** ela correu a `loadavg 82` com a
CPU a `88 %` ociosa, e as colunas CRUAS reproduzem a corrida de `loadavg 3,24` a menos de `2 %`.
*A grandeza que decide se um relógio vale é a ociosidade, não a média de carga.*

### §43.8 — ⛔⛔⛔ O TECTO SAIU, E NO LUGAR DELE FICA UMA PROPRIEDADE

`MAX_VIVOS = 743` → `358` → `MAX_GUARDADOS = 3 463` → **nada**. Uma cerca que, medida com a régua
inteira, nunca pode disparar **não é uma cerca** — é um palpite que sobreviveu a três correcções
porque ninguém releu o que estava em cada coluna.

⭐ O que fica é o gate `a_placa_ganha_em_toda_a_faixa_medivel`, que afirma a **propriedade** em vez
de a codificar num número que só uma máquina mediu: *a placa não perde de forma significativa, e
ganha com margem no contorno mais largo.* ⚠️ Ele é **deliberadamente frouxo no pior ponto** (o
penhasco lê `0,96×`, um empate, e a barra é `0,85×`) e exigente no topo (`1,2×` contra `1,30×`
medido) — *um gate que exigisse vitória em todo ponto reprovaria sobre produto correto*.

⚠️ **E nasce uma segunda sonda, a `mede_as_cenas_reais_nos_dois_motores`**, porque *um tecto
calibrado numa família e aplicado a outra é uma procuração*: uma peça de `2 663` valores guardados
ganha `2,58×` onde o polígono de `2 602` ganha `5,32×`. A razão depende da FORMA — quantos passos de
marcha por acerto, quanto a especialização por ladrilho da CPU corta —, não só da largura da fita.

### §43.9 — ⏳ O que fica

- ⛔⛔ **ESTE DOC TEM `236 KB` E O JOELHO DO `CLAUDE.md` §5.0 ESTÁ ENTRE `80` E `110`** — *«acima
  disso o `Read` desaparece e o acesso vira raspagem por shell»*. E a jornada de hoje confirmou-o na
  prática: **nenhuma** leitura dele nesta wave foi um `Read`; foram todas `sed -n` e `grep`. ⚠️ Eu
  acrescentei-lhe duzentas linhas e piorei o problema. ⇒ o corte é devido
  (`python3 scripts/doc-split.py`, que aborta se as duas metades não remontarem byte-a-byte), e
  **não foi feito aqui de propósito**: um movimento verbatim de 200 KB no fim de uma wave de medição
  tornaria o diff dela irrevisível, e escolher o que continua a ser LEI é uma decisão que merece a
  sua própria passagem;
- ⏳ **O PENHASCO de `192` arestas não tem mecanismo.** Ele reproduz-se, move-se com o código
  emitido e não é a ocupação (`vivos 35`, menos que a `176`). A hipótese por medir é o **tamanho do
  shader contra a cache de instruções** da placa — e ela nomeia um recurso, que é o mínimo para
  valer uma wave;
- ⏳⏳ **A consulta por ladrilho** (o que a CPU faz por `RegionCompiler`, e o estado da arte de
  Keeter 2020) continua a ser a alavanca de fundo para o contorno grande. ⚠️ **O preço que a §42.5
  lhe atribuía já não é o preço:** ele só se paga se a folha entrar como `Var` (que é o que apaga os
  modificadores), e desde que a fita é **escalonável** ela também é extensível — as arestas podiam
  viajar no vector `k[]` com índice por ladrilho, e `u`/`v` continuariam a ser expressões que a
  pilha remapeia. ⛔ Isso pede uma instrução nova na fita (`k[índice dinâmico]`), que a álgebra da
  `fidget` não exprime ⇒ é wave de substrato, e **ninguém mediu o que ela compra**;
- ⏳ **A recusa do interpretador de fita** ficou sem a premissa (`95` vivos ⇒ `23 KB` por workgroup,
  que cabe) e **fica de pé por outro motivo**, esse por medir: um interpretador paga descodificação
  por amostra e perde a fusão de operações que o `naga` faz.
