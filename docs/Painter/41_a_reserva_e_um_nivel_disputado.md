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
Nada a pintar, registar ou ligar; as quatro condições de UI ficam como estavam. ⏳ O **item 4** do doc 40
(o pincel esgotado RE-CAPTAR a própria tinta ao voltar) é **decisão do dono** e não entrou: muda o look
aprovado da depleção e pede um plano `u16` de «arco da PRIMEIRA cobertura» (custo medido no doc 40 §9).

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

Lei nua (`watercolor_reserve_tests.rs`): uma passada lê o próprio `v` · a regra do `max` fica no miolo · a
pálida cede ao longo do feather da escura (**com o controlo da lei antiga dentro**) · nunca atravessa papel
seco (com o controlo «tapada a coluna, difunde») · função do mapa e não da janela (AO BIT) · idempotente e
ausente sem mixer · a cauda a seco É o feather · o Rewet difunde à escala do pincel · leitura bilinear.

Produto (`tests/watercolor_selfseam.rs`): a costura tem a escala do pincel (controlo = lei antiga) · o Rewet
alarga-a em todo raio · o Smudge alcança o traço vivo (controlo: fora do esfregão a régua lê o mesmo) · os
planos não dependem do corte dos lotes · incremental ≡ full com o mixer ligado (e a `r = 80`, onde o Rewet
difunde o triplo do Bleed) · uma passada só fica com a cara que tinha.

Provas de mutação: ver o handoff da linha (a tabela sai do arnês `mutar.py`, com controlo sobre o próprio
arnês antes e depois).

## §8 — Aberto

- ⏳ **Item 4** (re-captar a própria tinta) — decisão do dono (§4).
- ⏳ **ACHADO PRÉ-EXISTENTE, fora desta wave:** no U desta régua o composite incremental e o full divergem
  `Δ124`–`Δ204` em milhares de bytes a `r = 48` e `r = 96` — **nas duas leis e a Charge 1**, logo não é
  deste campo (a `r = 32/64/80` lê `Δ1`–`Δ2`). Reprodução: a sonda `measure_who_owns_the_stale_pixel`.
  Parente provável dos dois `watercolor_app_params_incremental_*` que seguem `#[ignore]` (CLAUDE.md §5).
- ⏳ O custo por quadro do campo com Rewet alto e pincel grande (a janela cresce `R` para cada lado) —
  ver o handoff.
- ⏳ Sob **Tiling**, o arrasto dos níveis levanta toroidal como o da base; não tem gate próprio.
