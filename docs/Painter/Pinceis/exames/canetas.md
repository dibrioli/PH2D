# Exame da família `canetas` (J3, catálogo C, meio Digital) — 20/10

> Briefing: [`BRIEFING_FAMILIA.md`](../BRIEFING_FAMILIA.md). Pranchas: [`pranchas/canetas/`](../pranchas/canetas/) (`*_o2` =
> 2.ª onda; cegas e chaves ao lado). Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/canetas.rs`; espelho
> da medida: [`canetas_foto.py`](canetas_foto.py). 1.ª onda (11 revisores): `git show 84bbb1f393`.

## Resumo (2.ª onda)

| # | pincel | veredito | porquê |
|---|---|---|---|
| C-5 | Mapping Pen | ⚠️ na fábrica (aprovada na 1.ª onda) | 6 revisores novos: acertam o Krita 5–6/6 a confiança BAIXA medindo píxeis; restos ao píxel abaixo |
| C-4 | Studio Pen | ❌ | o S casa o G-pen (régua 450/454, a prancha igual), mas é o Hard Round com outra curva: **d 0,273** < 0,596 |
| C-6 | Technical Pen | ❌ | sem bolha agora, régua 426/454, mas **d 0,33** do Pixel 1px: a velocidade (−3 %) não o separa |
| C-7 | Ballpoint | ❌ em estudo (`EM_ESTUDO`) | traço leve, papel e 200 px casados; o cruzamento pede a mistura ESPECTRAL do MyPaint |
| C-11 | Dry Ink | ❌ | os alvos pedidos não são uma tinta seca distinta (medido abaixo) |
| C-10 | Brush Pen | 🔀 fundido | inalterado (d 0,187 do G-pen, 1.ª onda) |

## O motor (aditivo, neutro = o traço de antes ao bit; nenhum preset de fábrica mudou salvo o Gloop Pen)

Cada um: portão vermelho visto ANTES da cura; mutação na CÓPIA isolada da árvore (`scratchpad/canetas_motor/mutacao.py`:
controlo verde → mutado vermelho, nunca na árvore partilhada).

1. **Curva em S** — `Pen::s_curve`, linha «Pressure S-curve» (`pressureSCurve`, 0–1): `t + s·(smoothstep(t) − t)` entre o
   piso e o tecto. A rampa do G-pen (Krita `Ink-3`, medida na foto: 0,076 até 0,14, smoothstep até 0,86) cabe a ± 0,04.
   Portões `the_s_curve_softens_the_floor_and_the_ceiling_into_the_g_pen_ramp` (pen_tests) e o da família (vermelho: rampa
   0,3 → 0,283 × 0,217 e 0,7 → 0,80 × 0,90 com a recta; verde com o S). **M1** (S → recta) sangra. Custo: um smoothstep por
   amostra.
2. **L12 pela velocidade da MÃO desde o 1.º dab** — `stroke/speed.rs` (`Mao`): o caminho do ponteiro por tique, a rampa por
   arco; os dabs do 1.º quadro esperam pela 1.ª medida e saem no tique (antes do composite: latência 0). Era `[26, 24, 10, 6]`
   px (início × meio, Marker 50 px/evento) → `[6, 6, 6, 6]`. Portão `a_velocidade_vale_desde_o_primeiro_dab`. **M2** (a espera
   não escala) e **M3** (L12 volta à velocidade do traço atrasado pelo estabilizador) sangram. O Gloop Pen (speedSize 0,25)
   regerado; a isenção do speedOpacity no censo do Wet saiu (`o_censo_so_encolhe`: a linha agora age).
3. **A cauda (L5) com a cápsula** — `traco_plano.rs`: o re-afilar não escalava o estado de trás da cápsula (`Dab::ante`), e o
   sub-dab de trás voltava ao raio cheio — a bolha. Antes/depois:
   [`cauda_L5_curada.png`](../pranchas/canetas/cauda_L5_curada.png). Portão `a_cauda_com_a_capsula_afila_como_a_cabeca`
   (vermelho: cauda 10 px × cabeça 0 a 2 px do fim). **M4** sangra. O «espinho côncavo» do início é a lei da cabeça
   (smoothstep, ponta 0) — fica: mudá-la mexe em todo preset com taper; a cauda agora espelha-a.
4. **A pressão leve espalha** — `Dynamics::jitter_pressure`, linha «Light-touch scatter» (`pressureJitter`): o disco de
   Jitter somado `j·(1 − p)·D` (MyPaint `deevad/ballpen`, CC0: `offset_by_random` 0,354 → 0). Portões
   `a_pressao_leve_espalha_e_a_cheia_nao` e o da família (vermelho sem ele: borrão 0,033 × 0,28 do MyPaint; com 0,279).
   **M6** sangra nos dois.
5. **O papel não é pigmento** — `blend.rs`: o K–M deixava o papel branco opaco ser pigmento (o azul sobre o papel ficava
   0,82 × o over 0,68). Agora a mistura entra inteira com o canal mais escuro do fundo ≤ `PAPEL_DO_PIGMENTO` 0,75 e
   desliga-se até ao branco. Portão `o_papel_branco_e_o_substrato_do_pigmento`; **M5** sangra.
6. `LinhaDaMao::grava_no_neutro`: as duas linhas novas só vão para o preset mexidas — toda a fábrica fica ao byte.
   Portão `as_linhas_novas_so_vao_para_o_preset_mexidas`; **M7** sangra. Painel: 18 → 20 linhas (ids, rótulos, i18n).
   `stroke.rs` voltou abaixo das 700 linhas (o batente `MAX_DABS_PER_WALK` mudou-se para `walk.rs`).

## Por pincel

- **Mapping Pen** — Pow4 `hardness 0,16`, passo 0,08, `taper start 0,2 · tip 0,8`, `speedSize 0,025`, 3 px. O Krita
  `Ink-1` é tamanho por VELOCIDADE (`1 → 0,526`; medido: o rápido 0,97 do lento, o vertical 0,93) e o início dele 0,82 → 1
  em ~8 px. Régua 442/454; vizinho Pixel 1px d 0,798; 200 px casado (orla 45,7 × 46,9 px, ½ 115 × 114, 10 % 164 × 162).
  Custo p95 0,12 · 0,27 · 1,06 ms a 5/40/200, RSS 25 MiB, load 17–20. Revisores (rondas 5–10, cada um novo): o oráculo
  certo 5, 5, 6, 6, 5, 5 de 6, sempre «quase indistinguíveis a olho, só medindo». Os pedidos CONTRADIZEM-SE entre
  rondas (o início fino: «o Krita começa em ponta» com o taper fora, «começa redondo» com ele), e o que resta é: (a) o fio
  de 5 px mais pálido (0,92 × 0,98 de pico) — o Krita não é invariante à escala nas pontas pequenas e o nosso perfil é:
  pede uma dureza pelo TAMANHO (motor, fila); (b) o ruído dos sensores do Krita (a velocidade que oscila ao longo de um
  recto rápido; a pressão < 0,04 que afina os primeiros píxeis) — não é lei. Portão `cada_caneta_se_reconhece_pela_foto`.
- **Studio Pen** — `hardness 0,9`, cápsula, `pressureSize 0,93 · floor 0,14 · ceiling 0,86 · pressureSCurve 1`, 5 px.
  Prancha [`studio_pen_o2.png`](../pranchas/canetas/studio_pen_o2.png): as rampas são as do G-pen. Mas a matriz põe-no a
  d 0,273 do Hard Round (só a curva de pressão difere) — condição 4 do exame. Sem revisor (já reprovado). Se o produto
  quiser a pressão do G-pen, é a linha «Pressure S-curve» num Hard Round (decisão do coordenador/basicos).
- **Technical Pen** — `hardness 0,9`, cápsula, `pressureSize 0,25 · speedSize 0,058`, 5 px (Krita `Ink-2`, `diameter 5`,
  pressão 0,754 → 1). A bolha saiu; d 0,33 do Pixel 1px — a velocidade do Fineliner tira só 3 % ao recto.
- **Ballpoint** (em estudo) — Sharp, dureza 0, passo 0,16, `pressureSize 0,25 · pressureOpacity 1 · curve 1,2 ·
  pressureJitter 0,35`, Strength 0,7, o plano do traço (tecto 0,99) com pigmento 1. d 0,847 do Soft Round (distinto).
  Revisor 1 (prancha só com o plano): acertou 6/6 — halo mais curto, anéis soltos no traço leve, o cruzamento mais azul.
  Revisor 2 ([`ballpoint_o2.png`](../pranchas/canetas/ballpoint_o2.png)): escolheu o NOSSO como original nos gestos A e B;
  acertou os tamanhos (contas a 200 px, o 5 px mais claro) e o «sobre»: o MyPaint cruza o vermelho em violeta-cinza,
  o nosso em azul. Medido (o vermelho só a 211,80,93 nos dois): MyPaint `64,74,121`; o over `83,94,152`; o nosso K–M
  `48,94,137` — o verde fica 94 em TODA mistura de 3 canais (K–M e média geométrica); o MyPaint mistura ESPECTRAL (a tinta
  misturada lê `43,54,106`, mais escura em G que as duas). ⇒ **motor grande (fila)**: uma mistura espectral multi-banda.
- **Dry Ink** — reprovado com o porquê medido: o Krita `Ink-4 Pen Rough` é o Hard Round com a orla rasgada (rugosidade
  0,145, d 0,179 do Hard Round na 1.ª onda); o MyPaint `classic/textured_ink` são borrões espalhados (não uma tinta seca);
  o Rebelle `Ink/Brush/Brush 3` (opaco, orla rasgada 0,357, nada parecido com o Ink Brush da `tintas`) arrasta tinta
  molhada — vai para o Wet Paint (fila J4, família `tintas`).

## Portões por nome (20/10)

Verdes: `familias::canetas::cada_caneta_se_reconhece_pela_foto` (Mapping + a Ballpoint em estudo) · `mao::` (inclui
`a_velocidade_vale_desde_o_primeiro_dab`, `as_linhas_novas_…`, `pressao_leve::…`) · `traco_plano_tests` · `pen_tests` ·
`blend` · `cada_preset_de_fabrica_e_a_sua_receita` · `cada_preset_de_fabrica_volta_ao_byte` ·
`the_factory_presets_are_counted` · `seam_pen`. ⚠️ Vermelhos de outros a meio (não meus): a miniatura do Hard Round
(`basicos`, ficheiro por comitar), `a_largura_e_a_prometida` e o 2B/Ink Brush (Wet, `grafite`/`tintas`).
