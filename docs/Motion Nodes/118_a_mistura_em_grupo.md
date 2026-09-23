# 118 — A MISTURA EM GRUPO: três alcances, imagens e formas, no tom das formas (2026-09-23)

> Ordem do dono, depois de aprovar o smoke da cena `PH2D_MOTION_OBJ_SMOKE=13`:
> *«deveria funcionar também para shape. E com 2 modos: 1 modo onde o blend atua sobre as
> próprias shapes instanciadas e 1 modo para como as shapes se comportam com os objetos do
> cenário»*. Perguntado sobre as duas escolhas que mudam o que se vê, ele respondeu:
> - **alcance:** *«todas as opções possíveis sem excluir nenhuma, com seletor de modo no nó»*;
> - **tom:** **igual nas imagens e nas formas, como nas formas** (o espaço do Vello / W3C / CSS).

## §1 — Os três alcances, definidos por forma fechada

Com `f` a lei do modo (W3C), `B` o cenário, `S₁` e `S₂` duas cópias opacas que se sobrepõem:

| alcance (`Blend With`) | só o cenário | uma cópia | onde duas se empilham |
|---|---|---|---|
| **`Everything`** — cada cópia com TUDO o que está por baixo | `B` | `f(B, S)` | `f(f(B, S₁), S₂)` |
| **`Copies`** — só entre as cópias; o grupo pousa em `Normal` | `B` | `S` | `f(S₁, S₂)` |
| **`Scene`** — as cópias juntam-se em `Normal`; o GRUPO mistura-se com o cenário | `B` | `f(B, S)` | `f(B, S₂)` |

⚠️ **`Everything` é o comportamento de hoje das imagens** (a `=13` aprovada) e é o valor de
fábrica: o param é APENDADO com omissão `0`, logo nenhum documento muda de alcance.

## §2 — O que a medição deu antes da primeira linha

1. **Uma forma ignora a mistura por CONSTRUÇÃO:** a `VectorInstance` não tem campo nenhum para
   ela. O defeito não é um descarte — é uma ausência.
2. **As duas médias vivem em passes diferentes:** imagens no passe de sprites (HDR, mistura de
   hardware em espaço LINEAR — `blend_mode_regression`), formas no Vello (mistura W3C no espaço
   CODIFICADO). ⇒ o mesmo `Add` dá tons diferentes; o dono escolheu o das formas.
3. **Uma camada Vello NÃO vê as imagens:** a cena Vello rasteriza para um intermediário à parte
   (`VelloPass::render_to_intermediate`) e só depois é pousada — o fundo de qualquer camada é só o
   outro vector. ⚠️ Vale também para a «mistura por objecto» do Vector (doc 44), cujo smoke usa
   um fundo VECTORIAL e nunca a mediu contra uma imagem.
4. **No caso comum o cenário nunca vira imagem:** sem faixas (ADR-0154) e sem o vidro do prefab,
   o compositor junta sprites + a cena Vello (documento **e** painéis) só no último passo. O único
   sítio que monta «o cenário sem os painéis» é o `WorldRt`, e quem o monta fora das faixas é o
   [`present_frost::glass`](../../shells/desktop/src/render_loop/present_frost.rs).
5. **O codificador das formas JÁ desenha imagens** — a «terceira média» (`motion_shape_gen::encode`,
   `VectorInstance::texture_id > 0`): um quad texturado NA MESMA cena, por ordem. ⇒ imagens e
   formas de um sink cabem numa só cena Vello sem código de desenho novo.
6. **O Vello aceita uma textura da placa como imagem** (`Renderer::override_image`, já usado pelas
   imagens de efeitos do vector) — exige `Rgba8Unorm` + `COPY_SRC`, e o `WorldRt` é `Bgra8Unorm`
   ⇒ uma cópia com conversão.

## §3 — O desenho

**Um sink que mistura é desenhado inteiro pelo Vello, por cima do cenário como IMAGEM de base.**
Os três alcances são três arrumações de camadas — o Vello faz a matemática W3C, no tom das
formas, com a opacidade do fundo tratada certa (a mistura de hardware não o faz para um grupo
isolado sobre transparente: o `Multiply` apagaria as cópias).

- `Everything`: cada cópia na sua camada `f`, directamente sobre o cenário.
- `Copies`: uma camada `Normal` (o grupo isolado) e, dentro dela, cada cópia na sua camada `f`.
- `Scene`: uma camada `f` e, dentro dela, as cópias em `Normal`.

A ordem do quadro é a do vidro: cenário (sem o sink) → `WorldRt` → cópia `Rgba8` → cena Vello
[cenário como imagem + o grupo] → `WorldRt` → compositor com os painéis por cima.

## §4 — As waves

| wave | o quê | prova |
|---|---|---|
| **W1** | o param `Blend With` (3 opções, apendado) + a LEI das camadas no codificador | pixel contra a tabela do §1, com o Vello a desenhar |
| **W2** | a montagem do cenário no `WorldRt` com o sink RETIDO + o passe | foto com o controlo `Normal` ao lado |
| **W3** | as IMAGENS de um sink que mistura saem do passe de sprites e entram no Vello | a `=13` no tom novo, com formas e imagens lado a lado |
| **W4** | a rota da placa: um sink que mistura recusa-a, com o motivo dito | o relógio a 1k / 10k / 32k cópias |
| **W5** | a cena de smoke dos três alcances × as duas médias, fotografada | o smoke do dono |

⚠️ **O que muda para quem já tinha a `=13` aprovada:** o tom do `Add` nas imagens (o dono
escolheu-o) e a profundidade — um grupo que mistura é UMA camada por cima do cenário, e as cópias
deixam de se intercalar com as imagens do cenário por `z`.

## §5 — W1 FECHADA (2026-09-23): a lei das camadas, medida na placa

**O que existe.** O cartão do `Output` tem `Blend With` (`Everything` · `Copies` · `Scene`), pintado
LOGO A SEGUIR ao `Blend` e só quando o `Blend` é `Add`/`Multiply`/`Screen`. A bomba carimba em cada
`VectorInstance` o par `(mistura, alcance, sink)` (`MisturaDoSink`), e o codificador das formas
(`motion_shape_gen::encode` → `motion_shape_mistura`) parte a lista em CORRIDAS com a mesma chave e
dá a cada uma o arranjo do §3. As formas vão pela porta de lote nova
`ph2d_vec_render::draw_shared_instances_em_camadas` (o cache de tesselação por geometria continua a
valer: *uma camada à volta de cada chamada re-tesselaria toda cópia*); os quads de imagem levam a
camada recortada à caixa deles.

**A prova** (`motion_shape_mistura_gpu_tests`, `#[ignore]` de GPU): fundo + duas cópias
sobrepostas, cada modo × cada alcance × as duas médias, contra a tabela do §1 — **todas as células
a ≤ 1 byte**, formas e imagens com os MESMOS números. Controlo da fixtura primeiro (os três
alcances separam-se por mais de `4 ×` a barra em cada modo). Mutação **5 de 5** a sangrar (tirar a
camada por cópia · o grupo isolado · pôr o `Scene` por cópia · tirar a camada do quad · tirar o
`sink` da chave).

**Três coisas que a construção corrigiu:**

- ⛔ **A premissa das SECÇÕES era falsa.** Eu arrumei o cartão em quatro secções para o `Blend With`
  (apendado ao contrato) não ser pintado na última linha — e o cartão pinta as rows na ordem das
  DICAS, não na do contrato. As secções custavam `4` fileiras num cartão de `8` params, abaixo do
  piso MEDIDO do doc 108 W3 (`9`), e o censo `no_big_card_in_this_group_is_a_wall_of_sliders`
  acusou-o. Saíram; a dica mudou de sítio.
- ⚠️ **O `sink` entra na chave da corrida** — dois sinks VIZINHOS com a mesma mistura em `Copies`
  são DOIS grupos isolados (*um grupo é um NÓ, não um modo*). Nasceu de uma mutação SOBREVIVENTE:
  com um sink só, a fixtura não o distinguia.
- ⛔ **`Subtract` não tem camada, DECLARADO:** o W3C (logo o Vello) não o tem, a mesma fronteira
  do `ph2d_vec_render::blend`. Ele continua a desenhar como sempre, o seletor não aparece para
  ele, e o gate PRENDE essa igualdade.

⚠️ **O que a W1 NÃO faz ainda:** o cenário de SPRITES não está debaixo desta cena — o grupo
mistura-se com o que a cena vectorial já pintou. Pôr o cenário por baixo é a W2.

## §6 — W2–W5 FECHADAS (2026-09-23): o mundo por baixo, as imagens na cena vectorial, a recusa da placa

**W2 — o MUNDO por baixo da cena do Vello** ([`vello_fundo.rs`](../../crates/ph2d-render/src/vello_fundo.rs)).
Uma camada de mistura do Vello mistura-se com o que a MESMA cena pintou, e as sprites vivem noutra
textura que o compositor junta só no fim por um `over` ⇒ um grupo em `Multiply` sobre uma imagem do
cenário multiplicava-se com o VAZIO. Quando uma corrida pede o cenário
(`VectorScene::pede_o_mundo_por_baixo`, só `Everything` e `Scene` — ⛔ `Copies` não o pede, por
definição), o passe copia a vista sRGB do mundo (a saída do tonemap, ou o acumulador `WorldRt` com
faixas/vidro) para uma textura `Rgba8Unorm` registada, marca-a suja **todo quadro** (o atlas é
persistente desde a `vello` 0.10), e compõe `mundo (afim identidade) + append(cena)`. O intermédio sai
opaco e o `over` do compositor devolve-o tal e qual.
⚠️ **A cópia existe porque os formatos não casam** (mundo `Bgra8`, o Vello só regista `Rgba8Unorm`);
lê-se pela vista sRGB da fonte e escreve-se pela vista sRGB do destino ⇒ os mesmos 8 bits.
Gate de GPU `o_mundo_chega_ao_byte_e_a_camada_mistura_com_ele`: a fila sem camada é o mundo **ao
byte** e a fila com a camada é `mundo·S` a `±1`, em dois quadros com o mundo INVERTIDO (sem a marca de
sujo o 2.º quadro leria o 1.º). ⚠️ O doc da `compoe` dizia que `Low` era *a* amostragem exacta: a
mutação `Low → Medium` **sobreviveu** — com o afim identidade qualquer filtro é exacto no centro do
pixel. `Low` fica por ser a mais barata, e a prosa diz isso.

**W3 — as IMAGENS de um sink que mistura em grupo vão à cena vectorial** (ordem do dono: *«igual,
como nas formas»*). A bomba estampa a mistura **antes** de rebaixar; com `tem_camada()` ela chama
`lower_group_onto`, que rebaixa **toda** a corrente (sprites incluídas) como `VectorInstance` na ordem
das linhas — uma imagem vira `VectorQuad` com a UV e o tamanho de omissão e a textura da coluna.
⚠️ **Só imagens compõem o `uv_cell`** (`uv_do_pedaco`, que devolve o rect ao bit para a identidade).
Gates em [`mistura_no_pump_tests.rs`](../../crates/ph2d-eval-motion/src/mistura_no_pump_tests.rs):
`Normal`/`Subtract` mantêm a rota de sempre (2 sprites + 1 forma); `1`/`3`/`4` dão 0 sprites, 3
instâncias vectoriais **na ordem**, as duas UVs certas (`[0.25,0,0.5,0.25]` e `[0.5,0.5,1,1]`) e a
estampa `Scene`. ⭐ E o LOD das formas **não** tira da cena vectorial uma cópia em grupo, a zoom
nenhum (`uma_copia_em_grupo_fica_no_vello_a_qualquer_zoom`): no rasterizador de sprites ela perderia a
camada.

**W4 — a PLACA recusa, e o preço está medido.** O dispositivo não sabe o alcance (ele desenha pelo
passe de sprites) ⇒ `sink_mistura_em_grupo` derruba o sink para a CPU com o texto
`RECUSA_MISTURA_EM_GRUPO` (⚠️ o tag vive numa variável porque o censo
`every_gpu_cook_call_receives_the_style` conta o texto `blend,`). Gate de texto na shell a prender a
recusa e a leitura da marca no `present_chrome.rs`. **Custo medido** (`o_preco_da_mistura_em_grupo`,
`--release`, `load 16,6/29,4`, ⚠️ relógio de parede sob carga ⇒ ordens de grandeza, não barras):

| N | alcance | encode ms | palavras | placa ms |
|---:|---|---:|---:|---:|
| 10 000 | Normal / Everything / Copies / Scene | 0,25 / 0,52 / 0,56 / 0,27 | 10 000 / 20 000 / 20 001 / 10 001 | 1,12 / 1,51 / 1,59 / 1,40 |
| 32 000 | idem | 0,81 / 1,65 / 1,68 / 0,80 | 32 000 / 64 000 / 64 001 / 32 001 | 1,66 / 3,41 / 3,70 / 1,83 |
| 65 536 | idem | 1,80 / 3,54 / 3,63 / 1,86 | … / 131 072 / 131 073 / … | 2,96 / 3,93 / 3,91 / 3,38 |
| 131 072 | idem | 3,67 / 7,66 / 11,26 / 5,08 | … / **262 144** / 262 145 / … | 4,44 / 20,31 / 28,41 / 6,36 |
| 200 000 | idem | 7,52 / 13,34 / 11,44 / 6,32 | … / 400 000 / 400 001 / … | 15,07 / 16,49 / 16,44 / 7,45 |

⭐ `Everything` e `Copies` pagam **2 palavras** por cópia (abrir + fechar camada), `Scene` e `Normal`
**1**. O tecto do buffer é `262 144`, e a `200 000` cópias em camada (`400 000` palavras) o quadro
**continua pintado** (`2 034 540 px` nas quatro colunas) — o `Renderer` cresce o buffer; o custo sobe
em degrau acima de `~131 k` e fica dentro de um quadro a `32 k` (`~3,4 ms` de placa).

**W5 — a cena `=14`** ([`motion_object_smoke_grupo.rs`](../../crates/ph2d-app-motion/src/motion_object_smoke_grupo.rs)):
quatro colunas (controlo `Normal` · `Multiply` em `Everything` · `Copies` · `Scene`), cada uma com uma
fileira de IMAGENS e uma de FORMAS, 2×2 cópias SOBREPOSTAS (`PASSO < LADO_COPIA` é erro de
compilação) sobre um cenário que é uma SPRITE do mundo. Fotografada com o controlo: `Everything`
escurece a sobreposição outra vez, `Copies` deixa o cenário intacto e escurece só a sobreposição,
`Scene` escurece por igual, e imagens e formas seguem a mesma regra. A `=13` passou a abrir em
`Subtract` (tag 2), que é a mistura que continua híbrida na placa; `Add`/`Multiply`/`Screen` são da
`=14`, e o gate `a_cena_da_mistura_vai_a_placa` afirma que a `=13` **não** cai na recusa.

**Limites DECLARADOS:**

- ⛔ `Subtract` não tem camada (o W3C não o tem) — o seletor `Blend With` não aparece para ele.
- (O **tint** saiu desta lista na W6 — §7; a **amostragem** e o **ladrilhar** na W7 — §8; o **modo
  por linha** na W8 — §9.)
- ✅ **Ordem entre sprites do mundo e sprites do Motion:** curada na W10 (§10).
- ⭐ A **mistura por OBJECTO do vector** (doc 44 do módulo Vector) pode usar a mesma porta
  `pede_o_mundo_por_baixo` quando quiser misturar com sprites.

**Mutações** da fiação desta metade: **7 de 7** a sangrar, mais a de amostragem (`Low → Medium`)
**sobrevivente e registada** acima como não-lei. Portão:
`nextest-impacted` **20 150/20 150**, clippy `-D warnings` limpo nas sete crates tocadas, GPU
`vello_fundo` **1/1**, `mistura` **5/5**, e o gate de pixel da mistura no dispositivo **2/2**.

## §7 — W6 FECHADA (2026-09-23): a tinta de uma imagem é a conta da sprite

**O defeito:** o `draw_quad` desenhava a imagem tal e qual e deitava fora o `tint` da linha. Uma cópia
tintada que fosse ao grupo perdia a cor — e isto já valia **antes** do doc 118 para toda imagem da
terceira mídia (as folhas `VectorQuad` da rota vectorial). ⚠️ **A fixtura da tabela da W1 só passava
porque a tinta era ignorada:** pintava as imagens cinzentas com arte cinzenta **e** tinta cinzenta, e
a conta certa dava o cinzento ao quadrado. Hoje as imagens levam tinta branca e o cinzento vem da arte.

**A conta a igualar** é a do `sprite.wgsl`: `rgb = tex.rgb·tint.rgb`, `a = tex.a·tint.a`,
pré-multiplicado. **A ordem ingénua vaza:** a imagem primeiro e a cor da tinta por cima numa camada
`Multiply`+`SrcAtop` dá, pela lei W3C `cs' = mix(cs, B(cb,cs), αb)`, um resto `(1−αb)·T` em cada
texel **meio transparente** — a borda anti-serrilhada de toda arte. ⇒ **a ordem exacta põe a COR por
baixo, opaca e recortada ao quad, e a imagem por cima numa camada `Multiply`+`SrcIn`**: com o fundo
opaco a mistura não tem resto, e o `SrcIn` guarda só a alfa da imagem. A alfa da tinta é a da camada
de fora. Porta nova `VectorScene::push_layer_shape` (camada recortada a uma forma sob um afim).
`Tinta { Neutra · SoAlfa · Cor · Apagada }` — a **branca opaca não abre camada nenhuma** (byte-idêntico
ao que shipava), a de rgb branco abre UMA, e a alfa zero não desenha.

**Gate de GPU** `a_tinta_da_imagem_e_a_conta_da_sprite`: arte `2×1` com um texel opaco e um a **meia
alfa** (a fixtura tem de conter o fenómeno da ordem ingénua), três tintas (branca · `[1,0,5,0,25,0,8]`
· só-alfa `0,6`), dois texels cada, contra a conta fechada à mão, barra `≤ 3` bytes (medido `≤ 2`).
**Dois controlos dentro:** a tinta move os píxeis mais de `4×` a barra, e a ordem `SrcAtop` difere da
exacta mais de `4×` a barra. **Mutação 4 de 4** (ignorar a tinta · `SrcIn → SrcAtop` ·
`Multiply → Normal` · largar a alfa do `SoAlfa`).

### §7.1 — O que continua ABERTO neste doc, em ordem, cada um a MEDIR antes de construir

- ✅ **W7 — FECHADA (§8).**
- ✅ **W8 — FECHADA (§9).**
- ✅ **W9 — FECHADA (§8).**
- ✅ **W10 — FECHADA (§10).**

## §8 — W7 + W9 FECHADAS (2026-09-23): o filtro da imagem, a tile do LOD, e a terceira rota declarada

**A medição antes da 1.ª linha partiu a W7 em duas, e só UMA tinha trabalho.**

- ⛔ **O LADRILHAR não tem escritor.** Censo dos escritores de `uv_cell`: só o `motion.sub_uv`, e ele
  escreve **recortes** (`1/colunas × 1/linhas`, deslocamento em `[0, 1)`). Os escritores genéricos não
  a alcançam: o `motion.drive` recusa uma coluna não-escalar que já existe (`drive_named`), a
  `value.expression` emite um escalar, e o `value.attribute` esconde-a com motivo. E o `repeat` do
  `sampling` é **sempre** `Inherit` (`sink_style` crava `pack_sampling(filtro, 0)`). ⇒ na rota
  vectorial não há ladrilho a honrar, e o recorte já era composto exactamente (`uv_do_pedaco`).
  ⭐ Em vez de construir o que nada produz, um **censo** (`so_o_sub_uv_escreve_a_celula_de_uv`, piso
  de `100` crates de nó) reprova no dia em que um nó novo a escrever — e a porta para ladrilhar
  nesse dia está nomeada no doc dele (`fill_path_image` com `Extend`).
- ✅ **O FILTRO tinha trabalho, e um defeito que ninguém tinha visto.** A rota desenhava `Medium` para
  toda imagem; a sprite, com o sink em `Inherit`, amostra pelo filtro do **projecto** — logo num
  projecto em `PixelArt` uma folha era **nítida como sprite e borrada na cena vectorial**, e o
  comentário da W3 (*«mudar uma sprite do passe HDR para a camada LDR não muda um pixel»*) só era
  verdade para a cor. Hoje `qualidade_da_imagem(sampling, projecto)`: a tag `0` herda o projecto
  (`PixelArt → Low`, `Smooth → Medium`), as outras perguntam o `filter_tag_magnifies_by_point` — a
  MESMA função que monta o sampler da sprite. ⭐ O projecto de fábrica é `Smooth` ⇒ `Medium`, que é o
  que já se desenhava: **o caminho de omissão é byte-idêntico**. ⛔ **Declarado:** o pincel de imagem
  do Vello não tem mips nem anisotropia — as tags `3..=6` honram a metade de AMPLIAÇÃO e reduzem sem
  mips; o `High` (bicúbico) não entra porque nenhuma tag o pede.
- ⛔⛔ **E a TILE do LOD largava o PIVÔ e o FILTRO** (achado a ler o `vector_instance_as_tile` para
  saber quem mais lê a amostragem): os dois estavam cravados na identidade, contra o que o
  `StyleReach::VECTOR` prometia por escrito (*«acima de `LOD_COUNT` a linha passa a ser uma sprite,
  que honra os quatro»*). Com um `Pivot` no sink a forma **saltava de sítio** no quadro em que as
  cópias passavam o tecto do LOD. Curado com `anchor: vi.anchor` e `sampling: vi.sampling`; o gate
  mede a **posição de um canto** pelas duas rotas com a base rodada, não os campos.
- ⭐ **W9: a terceira rota está na lista.** `StyleReach::IMAGE_ON_VECTOR` desenhava desde 30/08 **fora**
  do censo que existe para que nenhuma rota desenhe sem declarar — honra os quatro, cada um pela
  mesma função da sprite, e o gate afirma-o.

**Gates:** `a_lei_do_filtro_da_imagem_e_a_do_sampler_da_sprite` (a tabela `7 tags × 4 repeats × 2
projectos`, com o controlo de que ela exercita as duas leis e de que o projecto de fábrica é `Smooth`)
· `o_filtro_do_sink_chega_ao_pixel_da_imagem` (GPU; uma imagem `PRETO | BRANCO` ampliada `24×` —
`20` píxeis intermédios com a lei bilinear, `0` por ponto, nas quatro células `Inherit×2` e
`explícita×2`) · `a_tile_do_lod_leva_o_pivo_e_o_filtro_do_sink` · a asserção nova em
`a_vector_row_gets_the_geometric_half…` (a linha leva o `sampling` do sink) ·
`the_frame_hands_the_project_filter_to_the_motion_encoder` (shell: o QUADRO passa o filtro do
projecto e não uma constante — ⚠️ sem ele a suíte da família fica verde com o `Smooth` cravado,
porque todo gate dela entra pelo `encode` com o filtro que o teste escolhe) ·
`so_o_sub_uv_escreve_a_celula_de_uv` · e o `every_draw_route_answers_the_sink_style` a exigir a rota.

**Mutação 10 de 10:** o `Medium` cravado · a herança a ignorar o projecto · a tile sem filtro · a tile
sem pivô · o lowering sem filtro · o quadro a cravar `Smooth` · a linha a ignorar o próprio
`sampling` · a rota fora da lista · a rota a mentir sobre a amostragem · um nó novo a escrever
`uv_cell`.

## §9 — W8 FECHADA (2026-09-23): o modo de UMA linha, nas formas, e os dois LOD a respeitá-lo

**A medição achou o defeito maior do que a nota dizia.** A nota falava de *«o modo por linha dentro
de um grupo»*; medido, a coluna `blend` (`0` = o do sink, `m + 1` = o modo `m`) tem **três**
escritores — o *Echo Operator* do `motion.trail`, o *Flash Operator* do `motion.strobe` e o modo da
sombra do `fx.drop_shadow` — e numa FORMA os três desenhavam em `Normal`, **com ou sem grupo**: a
`VectorInstance` não tinha onde levar o degrau (a forma de §2.1, uma ausência e não um descarte).

- ✅ `VectorInstance::blend_linha` (o DEGRAU, nunca o tag cru — guardar o tag faria o `0` de uma linha
  rebaixar o modo do sink), lido por `degrau_de_mistura`, que é agora a leitura ÚNICA da coluna:
  o `blend_at` das sprites passou a empacotá-lo. *As duas médias de uma linha não leem a coluna de
  duas maneiras.* (As leis de uma linha saíram para `lower_linha.rs` no tecto de LOC, e no corte a
  doc de `lower_to_instances_into` — que um corte anterior tinha deixado colada ao `blend_at` —
  voltou para a função dela.)
- ✅ **A linha SUBSTITUI a camada por cópia do grupo, e não a soma** — é o que a sprite faz (a linha
  ganha do sink). Sem grupo ela ganha a sua camada sobre o que está por baixo, e a cena **pede o
  mundo por baixo** como o `Everything` pede; num `Copies` ela troca o modo por cópia dentro do
  grupo isolado; num `Scene` ganha a sua camada dentro do grupo. ⚠️ O lote de formas parte-se onde
  a camada muda — sem linha nenhuma com modo, o lote de sempre, **byte a byte**.
- ⛔⛔ **E os DOIS LOD têm de a deixar no Vello — e o dos OBJETOS nem a cerca do GRUPO tinha.** A
  partição das formas já recusava trocar por tile uma cópia com `mistura.tem_camada()`; a dos objetos
  (`apply_object_lod`, acima de `LOD_COUNT`) **não**, logo um grupo que mistura com mais de 16 000
  cópias perdia o modo no quadro em que passava o tecto. Hoje as duas perguntam a MESMA porta,
  `precisa_do_vello` (grupo **ou** linha) — *uma cerca escrita em cada partição é como a segunda
  ficou sem ela*.
- ⛔ **Declarado:** um degrau sem camada Vello (`Subtract`) desenha em `Normal` na forma, como o sink
  `Subtract` já desenhava (o W3C não o tem) — e por isso **pode** virar tile. E numa corrente SEM grupo
  as imagens dela continuam no passe de sprites, que honra o modo por hardware em espaço LINEAR: o eco
  de uma imagem e o de uma forma no mesmo rasto não têm o mesmo tom (a fronteira de §2.2; o dono
  escolheu o tom das formas para os GRUPOS, e uma corrente sem grupo não passa pelo Vello).

**Gates:** `o_modo_da_linha_chega_a_cena_vectorial` (GPU; o CONTROLO sem degrau · `Multiply` em
formas **e** em imagens sem grupo, com o pedido do mundo · um `Copies` em `Screen` onde a linha
`Multiply` ganha, com o controlo de que a fixtura separa `f(Multiply)` de `f(Screen)`) ·
`o_degrau_da_linha_e_o_mesmo_nas_duas_medias` · `a_linha_vectorial_leva_o_proprio_modo` ·
`uma_copia_com_modo_proprio_fica_no_vello` (com o `Subtract` a mover-se como controlo) ·
`um_objecto_que_mistura_nao_vira_tile` (grupo, linha e o controlo na mesma lista).

**Mutação 9 de 9:** a linha sem camada · a imagem a ignorar a linha · o lote a ignorar a linha · o
grupo a ganhar da linha · o mundo por baixo não pedido · a partição dos objetos sem cerca · a cerca
sem o termo da linha · o lowering sem degrau · a cerca do degrau em `0` em vez de `0,5`.

## §10 — W10 FECHADA (2026-09-23): o Motion desenha por cima do mundo pelas DUAS rotas

**A nota dizia «medida e não investigada»; investigada, ela era um desacordo ENTRE ROTAS.** Um sink
não tem lugar na Hierarquia, e o lowering de CPU escrevia `z_order = 0` — o rank do objecto **mais
ao fundo** do cenário —, logo a ordenação (`sort_render_order`) punha o sink inteiro **por baixo de
toda sprite do mundo**, e o empate no rank `0` era o que enterrava as imagens mesmo com um
`ZIndexOverride(-1)` no cenário. Mas a rota da PLACA, que é a de omissão, desenha o buffer dela
**depois** das corridas da cena (`renderer_draw`), e as FORMAS do mesmo grafo vão ao Vello, que
pousa por cima de tudo. ⇒ *o mesmo grafo mudava de profundidade conforme a rota que o cozinhava* —
e a W5 viu-o porque um sink que mistura recusa a placa.

- ✅ `RenderInstance::Z_ORDER_OVER_THE_WORLD` (`u32::MAX`) nas TRÊS escritas: o lowering de CPU, a
  geradora de WGSL do device (que o `gpu_cpu_parity` compara campo a campo) e a tile do LOD. Entre
  si as linhas de um sink ordenam-se como antes (todas empatam no `z`, logo decidem o `sub_order` e a
  textura). ⚠️ As partículas dos objectos (TOP-20 #18) escrevem o rank do DONO e não mudam.
- ⭐ **A rota de omissão não muda um pixel** — ela já desenhava por cima. Muda a de CPU, que passa a
  concordar com ela.

**Gates:** `as_imagens_do_motion_desenham_por_cima_do_mundo` (pela porta do renderer, com um objecto
no rank `0` e outro alto, a ordem das linhas mantida, e o CONTROLO de que o `0` antigo punha o mundo
por cima) · `o_device_escreve_o_z_do_motion_por_cima_do_mundo` · a asserção nova na tile do LOD ·
e o `gpu_cpu_parity` (`184/185`; a vermelha é a `value_slope_kernel_matches_the_cpu_on_the_device`,
**pré-existente e documentada no próprio teste** com o mesmo número, `1,05023384e-4`).

## §11 — A cena `=15`: o modo de uma linha, as duas médias, fotografada

`PH2D_MOTION_OBJ_SMOKE=15` — um cenário azul-claro com uma FAIXA escura por baixo de cada sombra, e
duas colunas (imagem em cima, forma em baixo, as duas com um `fx.drop_shadow` cinzento e opaco): à
esquerda o controlo em `Sink`, que TAPA a faixa; à direita o `Multiply`, através do qual a faixa se
vê. A fileira da FORMA é a W8; a fileira da IMAGEM é a W10 (sem ela a imagem, que vai à rota de CPU
porque o `fx.drop_shadow` só tem lowering de CPU, desenhava por baixo do cenário).

⚠️ **Três fotos, três correcções que nenhum gate via:** o `size` de um círculo é o RAIO (o disco
saiu com o dobro da imagem e passou a borda do cenário) · uma sombra laranja confundia-se com o
ladrilho laranja da imagem · e sobre um cenário LISO o `Multiply` só muda um pouco a cor da sombra,
que ninguém lê como «ver através» — ⇒ a faixa, com uma cerca `const` que a obriga a cair dentro da
sombra e fora da peça. Gate `a_sombra_da_forma_leva_o_modo_da_coluna_ate_ao_vello` (a publicação REAL
da forma, o cozimento do grafo da cena e o lowering vectorial, com o controlo de que as duas colunas
existem).

