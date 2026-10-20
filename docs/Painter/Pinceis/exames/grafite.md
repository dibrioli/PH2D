# EXAME da família `grafite` (J3, catálogo B: B-1 HB · B-2 2H · B-3 2B · B-4 8B · B-5 Mechanical 0.5 · B-11 Erodible)

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/grafite.rs`. Pranchas: `pranchas/grafite/` (`<slug>.png`
final, `<slug>_antes.png` o de antes da J3, `_krita`/`_mypaint` o 2.º oráculo; cegas e chaves ao lado). Régua:
`exames/grafite/`. SHAs: `07bf02f7dc` (orla + grão fino + o dual deita-se) · `8f8b1db5ab` (poros, caroços do 8B) ·
`8d182780bc` (o 2B com poros).

## O que mudou nos quatro (o achado)

A revisão cega do 8B (pedida pelo ESTADO) e a 1.ª dos quatro deram o MESMO: o Rebelle era «claramente outro» (borda «aos
caroços, couve-flor», o traço que ondula) e o nosso parecia-se com o VIZINHO. Os presets do Rebelle explicam porquê
(`ParametersEx`/`Parameters` do `.rebelle-brush.png`): a forma é uma imagem oval girada 180° por marca, `edge_wave_jitter`
0,5, espaçamento 0,2 — a borda ondula à escala da PONTA (não do papel). Medido no recto de 40 px (régua `onda` do portão):
Rebelle 0,057–0,065 da largura, 6–7 px; o nosso 0,026–0,034, 1–2 px (só o grão). A cura é a **orla**: um dual Multiply
(L6) 0,45 × 3 marcas (0,7 × 2 no 8B) → 0,054–0,063, 8–10 px; o centro do traço também ondula (Rebelle 0,05 · nosso 0,04).
O **grão fino fraco** dos duros (o pedido do §7.2): Noise `Tiled` Size 256 de contraste 0,35 (HB) / 0,25 (2H) — cv5 HB
0,130 → 0,071 (Rebelle 0,066), ac5 0,78 → 0,34 (0,24); 2H 0,116 → 0,077 (0,061), ac5 0,77 → 0,36. ⛔ Recusa medida: a
orla 0,6 × 2 dava a mesma onda mas ac5 0,43 (as marcas de 3 px correlacionam o colar a 5 px).

**Motor (pequeno, com portão):** o dual não recebia a caneta (`dual_comeca/estende` sem `set_tilt`, e `pincel()` zerava
o `pen_tilt`) — com a orla o 8B DEITADO perdia o lado do grafite (através 1,11×, Rebelle 3,45×). Agora a 2.ª ponta deita-se
com a 1.ª: 3,11× (`a_caneta_deitada` vermelho antes, verde depois); com o rato, ao byte (`o_rato_e_a_caneta_de_pe…`).
Mutação: tirar o `set_tilt` → `o_lapis_deitado_alarga…` sangra (1,27×); repor o `pen_tilt` neutro →
`dual::a_segunda_ponta_deita_se_com_a_primeira` sangra.

**Portões novos** (`familias::grafite::portoes`): `os_grafites_sao_uma_familia_com_a_borda_do_rebelle` (onda ≥ 0,04 e
≥ 4 px nos quatro; cv5/ac5 do HB e 2H ≤ 1,6× / +0,15 do Rebelle; 2H ÷ HB = 0,757 ± 0,08 e HB < 8B) e
`sem_a_orla_e_com_o_grao_de_antes_o_hb_reprova` (o controlo por dentro: sem o dual onda 0,020; o grão de antes cv5
0,123). Mutação da receita (2H carga 24 → 40): o eixo sangra (2H ÷ HB 1,006). Regravado: o 8B na `a_lei_da_tinta`.
Verdes por nome: `a_caneta_deitada`, `o_grao_do_lapis` (os dois), `a_pressao_de_cada_preset…` (grafites 0,087–0,118),
`cada_preset_parece_a_sua_midia` (grafites e irmãos), `os_fios`, `presets_de_fabrica` (receita, miniatura, largura) — os
vermelhos que sobram nessas são de outras famílias (Ink Bullet/Brush, Wood Stick, Hard Round).

## B-4 Graphite 8B (WP) — ⛔ REPROVADO na revisão cega (a 3.ª), com 2 das 3 diferenças por decisão já tomada

Receita: a base `GRAPHITE_8B` (intacta: a família `lapis` lê-a) + orla 0,7 × 2 (espalhar 0,3, através 0,1), carga 150,
papel 0,55. Régua 428/454 (antes 416). Vizinho Oil Pastel 0,709 fora; família: Colored 4B 0,59, 2B 0,63. Origem 2.º
(0,544; 1.º Pastel Dry/Oval 0,534). MyPaint `Dieterle/8B_Pencil_1` 394/454. Razão: valores escuros e tom, o lápis deitado.
**Revisão cega 3** (novo revisor): achou o oráculo nas 6 linhas, o NOSSO como imitação nas 6, o vizinho «claramente outro»
nas 6 (as 2 de antes punham o Rebelle como o «outro»). RESUMO dele: (1) a 200 px o original fica ~41 px — é o Max Size 80
do preset do Rebelle (recusa medida do J1, 04_BANCADA ⛔); (2) a borda do original tem caroços maiores, a nossa «recta, só
grão fino»; (3) a rampa 0→1 do original entra a ¼ e fica densa cedo, a nossa sobe devagar — o piso de pressão que o
igualava foi REVOGADO pelo dono (18/10). Fica aberto para a fila: (2) — caroços maiores sem sair do vale do lápis deitado
(com a orla 0,8 × 1 + através 0,3 o «ao longo» sai do vale, 1,44 × 1,24 ± 0,12).

## B-1 Graphite HB (WP) — ⛔ REPROVADO na revisão cega (a 2.ª)

Receita: lápis 40, papel 0,18, orla 0,45 × 3, grão 256 · 0,35. Régua 424/454 (antes 430). Fora: Oil Pastel 0,838; família
2H 0,51, 2B 0,52. Origem 5.º (0,656; os do Rebelle 2H/H/HB estão a 0,13–0,17 entre si). Krita `c)_Pencil-2` 198/454.
Revisão 2: oráculo e imitação = o NOSSO nas 6 linhas; «o terceiro (o 2H) não é outro — é o mesmo desenho, ~76 % do tom»
(é o eixo de dureza, de propósito: a mesma orla nos dois). RESUMO: (1) Max Size (decisão J1); (2) o original tem grão
salpicado e borda trémula, o nosso «bolhas lisas» — o dual de disco lê-se como bolhas; (3) a pressão fraca do original afina
numa ponta salpicada, a nossa esmaece; o rápido nosso sai mais escuro (cobertura 0,69 × 0,63, régua ✗).

## B-3 Graphite 2B (WP) — ⛔ REPROVADO na revisão cega (a 2.ª)

Receita: lápis 46, papel 0,4, curva do tamanho 0,4, orla 0,45 × 3, grão 128 · 0,55. Régua 426/454 (antes 434). Fora:
Square Hard 0,633; família HB 0,52, Colored 4B 0,57. Origem ✓ 1.º (0,613). MyPaint `deevad/2B_pencil` 264/454.
Revisão 2: oráculo e o NOSSO como imitação nas 6; o vizinho «claramente outro» nas 6. RESUMO: (1) Max Size; (2) a rampa
0→1 do original entra a ¼ e engrossa depressa (o piso revogado); (3) poros maiores e borda com fiapos no original.

## B-2 Graphite 2H (WP) — ⛔ REPROVADO na revisão cega — DECISÃO DE PRODUTO para o coordenador

Com a cor da bateria o 2H do Rebelle É o HB (d 0,30 na família dele: 2H→H 0,17, H→HB 0,13; o nosso estava a 0,23 do HB,
«PASSO INVISÍVEL»); o eixo de dureza do Rebelle mora na COR de omissão (cinza 149 × 115 do HB), que os nossos presets não
levam. Escolha: o 2H no tom dela — carga 24 → 2H ÷ HB 0,746 (106/140 = 0,757), passo ao HB 0,51; a Cobertura da mídia do
2H = a da foto × 106/140 (`COBERTURA_DO_2H`). Régua 320/454 (as ✗ são o tom, de propósito). Revisão 2: o revisor tomou o
vizinho (2B) por original e o Rebelle por «outro» — o tom separa-os. Razão: linhas de construção que somem. **Alternativa
para o dono/coordenador**: fundir o 2H no HB (é o que o Rebelle faz com a mesma cor) — sai um preset e a família fica HB ·
2B · 8B.

## B-5 Mechanical 0.5 — ⛔ FUNDIDO no Hard Round (A-1)

Feito no Wet (o Rebelle `Pentell`: 3,84 px, sem grão, quina 0,6, pressão 1,0) e medido: régua 402/454, mas o vizinho é o
Hard Round a **0,289** (Dry Ink 0,294, Brush Pen 0,318) — PERTO. O próprio Pentell do Rebelle está a 0,306 do Digital/Round
Hard dele: a lapiseira do Rebelle é uma ponta dura fina. Receita e ficheiro retirados. ⭐ Achado: no Wet uma ponta seca SEM
grão pinta as CERDAS do motor (riscas ao longo, textura 4,4 %); um Noise de contraste 0 e brilho 1 arma a ranhura do grão
e tira-as (0,06 %).

## B-11 Erodible Pencil — ⛔ NÃO SAI: motor GRANDE (L17)

Fonte (WebSearch; a página da Adobe recusa o fetch, 403): a ponta erodível do Photoshop gasta-se com o USO e fica gasta
entre traços até «Sharpen Tip» (Softness = a velocidade do gasto; formas Point/Flat/Round/Square/Triangle, o quadrado perde
os cantos). Falta: (1) estado da ponta no tool que atravessa traços + o comando Sharpen (atalho, painel, i18n); (2) a
FORMA a mudar (cantos a arredondar), não só a largura; (3) a dureza por dab (`Dab` não a tem: o carimbo lê a do `spec`, CPU
e placa). A superfície de um campo do taper hoje: 12 ficheiros em 5 crates (o painel no tecto de LOC, a macro com censo
de portas). Uma largura por comprimento DENTRO do traço não é a erodível do Photoshop (afia a cada traço) — não a fiz.

## Custo (`--custo`, smoke, 5 · 40 · 200 px; p95 ms; RSS) — ⚠️ load 15–66 (outras famílias): o relógio não vale, repetir calmo

| pincel | p95 5 · 40 · 200 | RSS 200 | load |
|---|---|---|---|
| HB | 0,83 · 4,26 · 9,07 | 99 MiB | 22–25 |
| 2H | 1,48 · 4,08 · 10,13 | 99 MiB | 17–20 |
| 2B | 1,29 · 4,32 · 16,88 | 99 MiB | 14–18 |
| 8B | 0,64 · 2,79 · 5,97 (o de antes 5,12 a seguir) | 98 MiB | 19–23 |

A/B intercalado com o preset de ANTES (`git show 536d1ae213:…`), a 200 px: 8B 5,97 × 5,12 (load 19 · 14); 2B 23,7 × 7,4
(load 14 · 19) e na 2.ª volta 29,3 × 31,3 (load 44 · 47) — sob esta carga o A/B não decide (uma volta 3,2× pior, a outra
igual). ⚠️ Fila: medir o 2B a 200 px calmo antes do fecho (J5); se o p95 passar de 16,7, a orla dele a 2 marcas. Caneta deitada: alarga e alisa (o 8B: através 3,11×, ao longo 1,33×,
Rebelle 3,45 · 1,24).

## Revisões cegas (resumo do cruzamento; as respostas inteiras nas pranchas `pranchas/grafite/cegas`, chaves ao lado)

| pincel · ronda | oráculo achado | imitação = nosso | o 3.º é outro |
|---|---|---|---|
| 8B (pranchas da J2) | 0/6 (o Rebelle foi o «outro») | 0/6 (o vizinho 2H) | — |
| 8B · HB · 2B · 2H, ronda 1 | 0/6 nos quatro (o Rebelle foi o «outro» nos 24) | — | — |
| 8B ronda 3 · HB ronda 2 · 2B ronda 2 | 6/6 · 6/6 · 6/6 | 6/6 · 6/6 · 6/6 | sim · não (o 2H, o eixo) · sim |
| 2H ronda 2 | 0/6 (o tom) | — | — |
