# 41 — A reserva do pincel é um NÍVEL disputado, lido macio (Watercolor, Charge < 1) — 2026-09-20

> **Report do Enio (com foto), análise no [doc 40](40_a_costura_dura_do_retorno_sobre_o_proprio_traco.md):**
> com `Charge < 1`, voltar sobre o próprio traço sem pen-up *«lava parte da borda escura … mas deixa por
> baixo uma borda plana dura pixelada»*, e *«mesmo se aumento ao máximo Rewet e Smudge, não consigo me
> livrar dessa borda»*. Encomendas: **(1)** a costura menos dura com Rewet e Smudge em `0`; **(2)** Rewet e
> Smudge a funcionarem sobre ela. Depois: *«pode implementar sua solução. Busque o padrão ouro, o estado
> da arte.»*
>
> Linha: `line/PainterWatercolor`. Código: [`watercolor_reserve.rs`](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_reserve.rs).
> Régua e gates: [`tests/watercolor_selfseam.rs`](../../crates/ph2d-tool-painter/src/tool/paint/tests/watercolor_selfseam.rs)
> · [`watercolor_reserve_tests.rs`](../../crates/ph2d-tool-painter/src/tool/paint/watercolor_reserve_tests.rs).

## §0 — O que o artista passa a ter

1. A costura entre a perna escura e a perna pálida do MESMO traço é um degradê molhado-sobre-molhado, à
   escala do pincel, em vez de um degrau pixelado — com Rewet e Smudge em `0`.
2. **Rewet** alarga-a (a água re-molha e o pigmento difunde mais), em qualquer tamanho de pincel.
3. **Smudge** arrasta o pigmento do traço VIVO: esfregar por cima da costura, sem levantar a caneta,
   desfá-la.
4. A borda EXTERNA do traço (o aro escuro que ele aprovou) e todo traço a `Charge = 1` ficam como eram.

## §1 — Estado da arte (o que os outros fazem, e o que abandonaram)

| quem | o que faz na costura de um traço que se cruza | proveniência |
|---|---|---|
| **libmypaint 1.6.1** (ISC — porta ABERTA) | dab macio ADITIVO: cada dab mistura com o peso do seu perfil ⇒ a costura interna tem a largura do perfil do próprio dab — **razão `~1,00`** contra a borda externa, em quatro durezas | **CORRIDO** sem interface sobre um U nosso: [`ferramentas/oraculo_costura/`](ferramentas/oraculo_costura/README.md) |
| Procreate *Wet Mix* (Charge / Dilution / Attack / Pull / Grade) | a carga esgota com a distância e o pincel re-capta do que está por baixo; a mistura é por dab, macia | handbook público (doc 11/12) |
| Krita *Color Smudge* · Photoshop *Mixer Brush* | acumulador de cor corrente (*dulling*) — macio por construção, mas perde a textura do que arrasta | doc 11 §2 |
| Rebelle · Curtis et al. (simulação de fluido) | a difusão do pigmento na película molhada é FÍSICA (Navier–Stokes raso + camadas de pigmento) | ⛔ **fora por decisão**: ADR-0096 retirou a física; este motor é óptico (Tier 2, doc 11) |

**O que foi tentado e abandonado, e por quê nos importa:** o *dulling* (acumulador corrente) é o que o
Smudge deste app **recusou** em favor do *smear* verdadeiro (`smear.rs`: arrasta os pixels reais e guarda a
textura); e a regra de domínio desta casa é o `max` (*re-entintar um rasto pálido restaura-o; um pincel
esgotado não clareia tinta escura* — MIX-1, doc 12), que o dab aditivo do oráculo não tem. ⇒ a lei do
oráculo **transpõe-se**, não se copia: *a costura tem a escala do perfil do dab*, dentro da regra do `max`.

## §2 — O desenho, com a porta ÚNICA de cada pergunta

| pergunta | porta (uma) |
|---|---|
| *que nível de pigmento este pixel tem?* | `splat_level` — cada dab disputa com o peso do que DEPOSITA (`feather` normalizado pelo planalto): `L = max_d(v_d·q_d) / max_d(q_d)`. Inteiros, idempotente. |
| *onde a reserva afila na beira?* | `T = rampa(proximidade)` — geometria, à parte do nível. Numa passada só `T·v` é o mapa de sempre. |
| *que reserva o composite multiplica aqui?* | `ReserveFields::sample(raio do DONO, sx, sy)` — `T·S`, bilinear, nas coordenadas deformadas. |
| *quão longe o nível difunde?* | `reserve_radius` (capturado no `WetStrokeStyle::reserve_r` — um wash assado re-renderiza com o DELE): a base arredonda o joelho (`0,10·r ≤ core_r`); o **Rewet** difunde `Rewet × max(Bleed, 0,25·r)`. |
| *a janela do composite cobre esse raio?* | `WetSessionStyles::reserve_reach` entra no `reach` de `wash_window` **só** com o mapa vivo. |
| *o Smudge arrasta neste traço?* | `wet_smudge_live()` — lida pelo arrasto da BASE (`stamp_route`) e pelo dos NÍVEIS (`accumulate_wet_coverage`). |

Dois planos `u8` por traço (só com o mixer ligado): `stroke_deplete` = o **nível** (já não leva a rampa) e
`stroke_deplete_prox` = a **proximidade** do dab mais próximo (o peso da disputa e o afilamento derivam dela
por tabela). `+1 byte/px` enquanto o mapa vive (`+16,8 MB` a 4096²).

**As três propriedades que o campo tem por CONSTRUÇÃO, e o gate de cada uma:**

- **Não atravessa papel seco** — as somas em caixa correm por TROÇOS contíguos da lavagem, na horizontal e
  depois na vertical (difusão sem fluxo na silhueta). Dois washes que não se tocam não se influenciam,
  qualquer que seja o raio — mais forte que a cerca `WET_FIELD_BLUR_PX < 10 px` dos campos vizinhos.
- **É função do MAPA, não da janela** — `Σ L·q` e `Σ q` são `u64` exactos e a divisão é uma só. O
  `box_blur` da casa soma `f32` desde a origem da janela; este não herda esse resíduo.
- **Charge = 1 é byte-idêntico por AUSÊNCIA** — sem mixer os planos não existem, o campo é `None` e a
  janela é a de sempre.

## §3 — Contratos congelados e schema: não encosta (prova)

`git diff --stat main -- . ':!docs'` lista **só** `crates/ph2d-tool-painter/src/tool/paint/**`. Nenhum
ficheiro de `ph2d-editor-core/src/tool*` (o `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/
`PanelEvent=4`), nenhum `BrushSpec` (o `ph2d-painter-brush` não foi tocado — **zero campo novo
serializado**), nenhum `PROJECT_SCHEMA`. O `WetStrokeStyle` é `Clone + Copy` de runtime, nunca gravado.

## §4 — UI: nenhum controlo novo

Os quatro knobs que o artista já tem — **Charge · Rewet · Smudge · Bleed** — passam a alcançar a costura.
Nada a pintar, registar ou ligar; as quatro condições de UI ficam como estavam. ⛔⛔ O **item 4** do
doc 40 (o pincel esgotado RE-CAPTAR a própria tinta ao voltar) foi construído como o knob `Self Pickup`
em 2026-09-20 e **RETIRADO no mesmo dia por veredito do dono** — *«não consegue distribuir corretamente
a carga da tinta»*. ⇒ este cartão volta a ter **três** linhas, e a recusa medida (com o que funcionava
e o que ele julgou) vive no [doc 40 §S2-C](40_a_costura_dura_do_retorno_sobre_o_proprio_traco.md).

## §5 — Medido (preset de aquarela do PRODUTO, Ragged Edge `6 px`; `load 2,85`)

A régua é a **largura efectiva** da transição (`contraste / maior inclinação`, linha alisada por 5 px —
o chão dela é `5 px`). CONTROLO = a lei de antes, corrida dentro do mesmo arnês (`LEI_ANTIGA`, só em teste).

| raio | lei antiga | **nova, seca** | em raios | **Rewet 1** | em raios | borda externa |
|---:|---:|---:|---:|---:|---:|---:|
| 20 | 5,40 px | **6,76 px** | 0,34 r | **10,00 px** | 0,50 r | 5,80 px |
| 32 | 5,86 px | **9,32 px** | 0,29 r | **12,00 px** | 0,38 r | 5,94 px |
| 45 | 6,52 px | **11,30 px** | 0,25 r | **15,26 px** | 0,34 r | 6,16 px |
| 96 | 10,22 px | **22,00 px** | 0,23 r | **30,00 px** | 0,31 r | 7,99 px |

- A `r = 8` a costura inteira é menor que o chão da régua — ela não decide ali (declarado).
- **Smudge**, esfregando de lado a lado por cima da costura sem pen-up: `10,4 → 33,1 px`. ⚠️ Antes da cura
  o traço com Smudge `1` era **byte-idêntico** ao de Smudge `0` (a tabela de `antes` tem as duas linhas
  iguais nos cinco raios) — o relato do dono, ao número.
- **Uma passada só:** pior byte `Δ11` em `17 341 px` contra a lei antiga — a casca da beira, bilinear contra
  vizinho-mais-próximo. É a anatomia aprovada, intacta.
- Suíte do Painter: `1252` verdes · clippy `-D warnings` limpo.

## §6 — ⛔ Recusas MEDIDAS (não as reconstrua)

| o que se tentou | o que mediu | mecanismo |
|---|---|---|
| **Rewet alarga a CAUDA da disputa, dos dois lados** (`0,38 → 0,75`) | `9,3 → 10,8 px` a `r = 32`, quando a conta pedia o dobro | com as duas caudas largas o dab pálido deixa de estar SATURADO sobre a cauda da escura; a transição passa a ser a razão `q₁/q₂`, que muda depressa — a costura **desloca-se**, não abre |
| **Rewet alarga só a cauda de quem JÁ ESTÁ no papel** (assimétrica, `0,90`) | na lei nua: `7 → 0 px` | o mesmo: na geometria do U (pernas a `1,2 r`) o planalto da pálida só cobre `dn₁ ≥ 0,58` |
| **Rewet alarga o raio em PIXELS do Bleed** (S2-A do doc 40) | `22,0 → 22,1 px` a `r = 96` | `7 px` de difusão não se vêem numa costura de `22` — a difusão tem de ter a escala do pincel |
| **`max \|ΔI\| / contraste` como régua** | a costura curada lia `0,15` e nada mais | piso de RUÍDO: o dente do papel mexe ~5 bytes/px sobre ~38 de contraste |
| **«costura não mais dura que a borda externa» como barra** (doc 40 §7) | a lei ANTIGA já lia razão `0,9`–`1,5` | a nossa borda externa é nítida DE PROPÓSITO (o aro); é um piso, não a barra |
| **gate de polling a nível de EVENTOS** (4 px contra 11 px) | `Δ226` a **Charge 1**, sem mixer | o preset tem dinâmica de velocidade — a fixtura mede outra coisa; a forma testável é o corte dos LOTES |

⇒ *a disputa é local e a largura dela é a da sobreposição; quem alarga para lá disso é a DIFUSÃO, que lê
vizinhança* — e é por isso que o Rewet entra pelo raio, com a janela a saber do alcance.

## §7 — Gates e provas de mutação

Lei nua (`watercolor_reserve_tests.rs`): uma passada lê o próprio `v` · **o afilamento da beira afila na casca
de `RESERVE_RIM_RAMP`, é monótono e morre na beira** (com o controlo de que a casca não começa cedo) · a regra
do `max` fica no miolo · a pálida cede ao longo do feather da escura (**com o controlo da lei antiga dentro**)
· nunca atravessa papel seco (com o controlo «tapada a coluna, difunde») · função do mapa e não da janela (AO
BIT) · **e o complemento: uma margem ABAIXO de `R` lê outro campo** (com a margem larga como controlo) ·
**a janela do composite consulta o raio do campo** (gate de TEXTO) · idempotente e ausente sem mixer · a cauda
a seco É o feather · o Rewet difunde à escala do pincel · leitura bilinear.

Produto (`tests/watercolor_selfseam.rs`): a costura tem a escala do pincel (controlo = lei antiga) · o Rewet
alarga-a em todo raio · o Smudge alcança o traço vivo (controlo: fora do esfregão a régua lê o mesmo) · os
planos não dependem do corte dos lotes · incremental ≡ full com o mixer ligado · uma passada só fica com a
cara que tinha.

⛔⛔ **E uma correcção que a prova de mutação impôs a este doc.** A redacção anterior desta secção dizia que o
`incremental ≡ full` testemunhava o `reserve_reach` da janela — *«e a `r = 80`, onde o Rewet difunde o triplo
do Bleed»*. **É falso, e a mutação mediu-o:** apagar aquele termo deixa esse gate verde a `r = 80` **e** a
`r = 120`. A conta certa é `pad = reach + ceil(warp) + 2`, aplicada **duas vezes**, e o que o campo pede é
`R`; sem o termo o `pad` vale `22` contra `R = 20` (`r = 80`) e `R = 30` (`r = 120`) — ou seja, **no segundo
a margem é mesmo deficiente e a imagem ainda concorda a dois níveis**, porque o erro do campo mora a `22 px`
de qualquer dab daquele quadro e o quadro seguinte reescreve por cima. ⇒ aquela metade partiu-se em **duas**:
a PREMISSA (`a_caixa_truncada_le_outro_campo`, medida — pior `|Δ|` no campo `0,000094` com margem `38 ≥ R`,
`0,028832` com `18`, `0,138007` com `8`) e a FIAÇÃO (gate de TEXTO). *Um gate cujo nome promete o que a régua
dele não alcança é a forma canónica do verde que não afirma nada.*

**Provas de mutação — `8` de `8` sangram**, com CONTROLO do arnês antes e depois (`18` verdes, `0` vermelhos
nas duas pontas):

| mutação | o que ela apaga | sangra em |
|---|---|---|
| M1 | a leitura bilinear → vizinho-mais-próximo | `the_field_is_read_bilinear_not_nearest` · `…incremental_equal_to_full` |
| M2 | a caixa respeitar o troço (atravessa papel seco) | **7** gates, entre eles `a_caixa_truncada_le_outro_campo` |
| M3 | a disputa (volta ao `max` cru) | **5** gates, entre eles `a_pale_pass_over_a_dark_edge_yields_along_the_dark_feather` |
| M4 | o afilamento da beira | `the_rim_tapers_over_the_outer_shell_and_dies_at_the_edge` |
| M5 | o Smudge arrastar os níveis | `smudge_reaches_the_seam_of_the_live_stroke` |
| M6 | a janela reservar o raio do campo | `a_janela_do_composite_reserva_o_raio_do_campo` |
| M7 | o Rewet difundir | `rewet_diffuses_at_the_scale_of_the_brush` · `rewet_widens_the_return_seam_at_every_brush_size` |
| M8 | a cadeia do Smudge sobreviver ao lote | `the_level_planes_do_not_depend_on_how_the_dabs_are_batched` |

⛔⛔ **E a 1.ª ronda deu `5` de `8`. As três que faltaram eram defeito do AUTOR, não do produto, e cada uma
nomeou uma coisa que vale para a próxima wave:**

1. **M2 lia «NÃO COMPILA» e na verdade PENDURAVA.** A agulha apagava o único `x += 1` do ramo seco, e o
   varredor de troços passou a girar para sempre ⇒ nunca houve linha `test result:`, e o arnês leu a ausência
   dela como *«não compilou»*. ⚠️ *Uma mutação que não TERMINA e uma que não compila são o mesmo silêncio para
   quem faz o parse da saída.* A agulha passou a ser a cerca do troço (`.max(lo)`), que termina e devolve a
   soma de outro troço — e sangra em sete.
2. **M4 sobrevivia porque o afilamento da beira NÃO TINHA RÉGUA.** Os quatro gates que liam o campo amostram
   onde `prox = 255` — ali `T = 1` **por construção** — ou afirmam sobre o plano do NÍVEL. ⚠️ *Quatro gates
   sobre o mesmo objecto podem ter todos o mesmo ponto cego, e a contagem deles lê-se como cobertura.*
3. **M6 sobrevivia porque o gate PROMETIA no comentário o que a régua dele não alcança** (a correcção está
   acima). ⇒ a metade partiu-se em premissa + fiação.

⚠️ *E a 2.ª ronda não bastou: com a M2 e a M4 curadas, a M6 continuou a sobreviver mesmo depois de eu mudar
o raio da fixtura — o que provou que o problema não era o número, era o gate estar a afirmar outra coisa.*

## §8 — Aberto

- ⛔ **Item 4** (re-captar a própria tinta) — **CONSTRUÍDO e RETIRADO** por veredito do dono
  (2026-09-20). ⚠️ Recusa MEDIDA: a cerca de idade funcionava e o custo era nulo; o que ele reprovou
  foi a **DISTRIBUIÇÃO da carga** — doc 40 §S2-C. *Reconstruir a mesma lei com outro knob volta ao
  mesmo veredito.*
- ⏳ **ACHADO PRÉ-EXISTENTE, fora desta wave, agora com CONTROLO:** no U desta régua o composite incremental
  e o full divergem em raios isolados — **e a lei ANTIGA lê o mesmo número**, que é o que o data. Varrido por
  [`diag_a_escada_do_raio_da_janela`]: `r = 88` lê `161` molhado · `94` seco · **`94` com a lei antiga**;
  `r = 96` lê `139` · `93` · **`90`**; `r = 120` lê `2` · `2` · **`2`**. ⚠️ Ele aparece **a seco**, onde o
  campo da reserva mal participa (o `reach` é o `core_any` e o raio do campo é `~9`) ⇒ não é deste campo, e a
  escada não é monótona. Parente dos dois `watercolor_app_params_incremental_*` que seguem `#[ignore]`
  (CLAUDE.md §5), cuja nota já diz que `pad += 2·raio` **não** é a cura.
  ⚠️ *É por isso que o gate irmão mede a `120`: uma barra posta num raio onde outro defeito já vive não
  afirma nada sobre este.*
- ✅ **O CUSTO está MEDIDO** (sonda versionada `measure_the_cost_of_the_reserve_field`, `--ignored`; traço em U
  inteiro = depósito + composites por quadro + o assar do pen-up; mínimo de 3; **CPU `99 %` ociosa**, medida
  por `vmstat` e não pelo `loadavg`, que estava a decair de `45` e mente).

  | canvas | `r` | knobs | nova lei | lei antiga | razão |
  |---|---|---|---|---|---|
  | 512 | 32 | **charge 1 (sem mapa)** | `80,8` | `72,6` | **`1,11`** |
  | 512 | 32 | seco | `118,3` | `68,2` | `1,73` |
  | 512 | 32 | rewet 1,0 | `287,2` | `174,4` | `1,65` |
  | 512 | 96 | **charge 1 (sem mapa)** | `214,2` | `213,1` | **`1,00`** |
  | 512 | 96 | seco | `330,4` | `205,4` | `1,61` |
  | 512 | 96 | rewet 1,0 | `923,8` | `719,7` | `1,28` |
  | 2048 | 250 | **charge 1 (sem mapa)** | `1 553,0` | `1 512,4` | **`1,03`** |
  | 2048 | 250 | seco | `1 903,5` | `1 212,4` | `1,57` |
  | 2048 | 250 | rewet 1,0 | `5 345,4` | `3 823,8` | `1,40` |

  ⭐ **A linha que decide é a do CONTROLO:** com `Charge = 1` — o valor de fábrica, onde não há mapa de
  reserva — a cura é **de graça** (`1,00` · `1,03`; a célula de `r = 32` lê `1,11` em release e `0,99` em dev,
  ou seja **as duas leituras cavalgam o `1,00`** sobre um total de `~75 ms` ⇒ ruído, e é a célula mais
  pequena da tabela). *O artista que não usa Charge não paga nada por isto.*
  ⚠️ Armada (`Charge < 1`), ela custa `1,28`–`1,73×` o traço. **Derivado** (não medido directamente): sobre
  `~90` composites de um U, isso são `+1,3 ms` por quadro a `r = 32` e `+2,1 ms` a `r = 96` — e a `2048/250` o
  quadro já custava `34 ms` **na lei antiga**, logo ali o orçamento é um problema que não é desta wave.
  ⛔ **E o `--release` NÃO é 20× o dev aqui, ao contrário do que a lei geral do §5.0 faz esperar:** as duas
  colunas batem (`seco` lê `1,73`/`1,75`, `rewet 1,0` `1,65`/`1,49`), porque a `ph2d-tool-painter` está na
  lista `[profile.dev.package.*]` `opt-level = 2` do `Cargo.toml` da raiz. *Quem re-medir isto não precisa de
  pagar um build de release.*
- ⚠️ **PROMOÇÃO PEDIDA à família de flakes de fan-out (CLAUDE.md §5.0)** — a linha pede, o integrador escreve:
  `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`
  (`ph2d-tool-painter`, `tool::paint::tests::measure_input_cost`). Assinatura completa: único ✗ de `1 257`
  numa corrida da suíte da crate · **zero** linhas do diff desta linha naquele caminho (`git diff --stat`
  devolve vazio) · **3 de 3 verde sozinho a `load 45,37`**, que é MAIS carga do que aquela em que reprovou.
  ⇒ *o discriminador é o FAN-OUT e não o relógio*, como a nota da família já regista.
- ⏳ Sob **Tiling**, o arrasto dos níveis levanta toroidal como o da base; não tem gate próprio.
