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
- ⚠️ Uma imagem na rota vectorial ignora o **tint**, o **modo por linha**, a **amostragem** e o
  **ladrilhar** do `uv_cell` — o quad desenha a UV, o tamanho e a textura. Só o recorte da célula é
  composto.
- ⚠️ **Ordem entre sprites do mundo e sprites do Motion, MEDIDA e não investigada:** com o cenário por
  baixo da coluna de controlo, as quatro imagens `Normal` (desenhadas pelo passe) ficavam TAPADAS por
  ele, e um `ZIndexOverride(-1)` no cenário não as trouxe à frente. É anterior a esta wave; a cena põe
  o controlo fora do cenário e o porquê está no doc da constante.
- ⭐ A **mistura por OBJECTO do vector** (doc 44 do módulo Vector) pode usar a mesma porta
  `pede_o_mundo_por_baixo` quando quiser misturar com sprites.

**Mutações** da fiação desta metade: **7 de 7** a sangrar, mais a de amostragem (`Low → Medium`)
**sobrevivente e registada** acima como não-lei. Portão:
`nextest-impacted` **20 150/20 150**, clippy `-D warnings` limpo nas sete crates tocadas, GPU
`vello_fundo` **1/1**, `mistura` **5/5**, e o gate de pixel da mistura no dispositivo **2/2**.
