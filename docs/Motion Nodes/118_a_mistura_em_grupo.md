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
