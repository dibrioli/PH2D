# EXAME da família `lapis` (J3, catálogo B: B-6 Tilted Shade · B-7 Rough Sketch · B-8 Colored 4B · B-9 Craft Flat · B-10 Scratch)

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/lapis.rs`. Pranchas: `pranchas/lapis/` (`<slug>.png` a
última; `cegas/` e `chaves/` com as rondas revistas, `_r1`…). Régua: `exames/lapis/<slug>_regua.txt`. A caneta deitada:
`exames/lapis/bateria_deitado.json` (as linhas B/D/A de `rebelle_tilt`) e `fixtures/krita_deitado/`. **Os cinco REPROVADOS.**

## Achados da bancada (valem para todas as famílias)

- **A bancada pinta a caneta deitada**: o ponto da bateria aceita `[x, y, p, tilt_x°, tilt_y°]` (`exame_bancada::cp`; as
  baterias de antes pintam ao bit o mesmo). O Krita recebe a inclinação por `handleTabletEvent` (`xTilt`/`yTilt`; cópia
  das ferramentas em `exames/lapis/`). ⚠️ `ferramentas/krita_bateria_dentro.py` tem um defeito: o `base` do preset das
  faixas tapa o `base` do nome da PNG, e a 2.ª/3.ª cena de cada preset gravam `<PyKrita.krita.Resource object …>_<cena>.png`
  na pasta de onde se corre (curado na cópia).
- **Os `params` das texturas procedurais**: os DOIS primeiros são o tom (contraste, brilho: `apply_tone`); os do padrão
  começam no 3.º. Um Gradient com `[0, 0, …]` apaga o dab; Stripes com a largura no 1.º é um cinza liso.
- **A lei da largura do Wet** mede cada tamanho: uma Shape de riscos SEM fundo (contraste 1) mede 1 px e o preset gerado
  não pinta nada; com penugem entre os riscos (contraste 0,45) fica sã.
- **A cápsula do Wet** (0,025 D) funde os dabs: os rectângulos soltos do Craft Flat só saem no Digital — que não mistura
  o lápis no vermelho como o Wet. **O Multiply** no Digital escurece o próprio traço (azul-marinho, cruzamentos pretos).

## B-8 Colored 4B (WP) — REPROVADO (a receita fica: melhor que a de antes em tudo o que se mede)

Com a cor da bateria o «Colored Pencil/4B Color» do Rebelle É o «Graphite Pencil/4B» (ao byte): o degrau 2B → 8B.
Receita: a base 8B + orla (dual 0,6 × 2, espalhar 0,25, através 0,2), tufos (Noise `Random` 8 · 0,6), grão `Tiled` 128 ·
0,5, carga 140, papel 0,45, tecto 62 (o recto de 200 do Rebelle). Régua 436/454 (antes 432). Fora da família: Oil Pastel
**0,739** (antes 0,498: o «perto» resolvido); família 2B 0,56 · 8B 0,52; a origem é o 1.º dos 228 (0,665). Razão: o lápis
de cor ceroso, o degrau médio da série. Portão `o_colored_4b_tem_a_borda_aos_carocos_entre_o_2b_e_o_8b` (onda 0,059; Rebelle
0,061) + a foto da mídia. Revisões: r1 (orla 0,8 × 1) oráculo 6/6, nosso imitação 6/6, o 8B «não é outro» (o eixo); r2 e
r3 (confiança BAIXA) tomaram o nosso 8B pelo original e o Rebelle por «outro: contorno ondulado e granuloso, rampa a ⅓»
(a rampa é a decisão 9). Falta: o aberto (2) do `grafite.md` (os caroços maiores do Rebelle na MESMA base) — sobe com ele.

## B-10 Scratch (WP) — REPROVADO; volta ao preset de antes

O Rebelle pinta um punhado de IMAGENS de riscos por marca (rodadas ±90°): a 40 px penugem, a 200 px riscos nítidos
cruzados. Tentado: Shape Stripes `ViewPlane` (1,5, largura 0,1, freq. 0,9, macieza 0,3, deitada, contraste 0,45) rodada
±90°, carga 300 — régua 452/454, vizinho Airbrush 0,866. Revisões (média): r1 (contraste 0,5) «palha de 1 px nítida,
esparsa»; r2 (0,45) oráculo 6/6 e nosso 6/6, mas «a 200 px uma névoa com fibras curvas», o grão não cresce com o tamanho.
Os dois tamanhos puxam o contraste para lados opostos, e a versão de riscos reprova a foto da mídia (cobertura 0,307 ·
cv 0,067 contra 0,255 · 0,126). **Falta (motor/fábrica)**: a ponta de N imagens (L19 existe no motor) na RECEITA de
fábrica — o gerador só leva texturas do ficheiro `nasce_de`; as imagens de riscos geradas por nós entrariam por aí.

## B-6 Tilted Shade (Digital, novo) — REPROVADO

Oráculo Krita «c) Pencil-5 Tilted» (o Size pela INCLINAÇÃO; com a caneta: `fixtures/krita_deitado/`). Receita: elipse 0,4 ×
D fixa (de pé o recto fino e o vertical largo, Krita 2 × 5 px), rotação pela caneta, a RAMPA (Gradient 0,25, deslocada
0,5, 180°), acumulação 0,55, grão `Tiled` 128 · 0,6, tinta pela pressão (curva 1,3), inclinação 15° → 63°, ×5 através,
×3 ao longo; tamanho 5,2 (o Krita deitado pinta 17 px). Régua de pé 314/432, deitada 157/176 (129 sem a rampa). Vizinho
Grainy Shader 0,93. Razão: sombrear com o lado da mina sem trocar de pincel. Portão `o_tilted_shade_deita_como_o_lapis_do_
krita` (de pé ÷ 63° 0,099 · 35° ÷ 63° 0,528 · ao longo ÷ através 0,317; Krita 0,118 · 0,529 · 0,353). Revisões deitada
(média; r3, r4): oráculo 3/3, nosso imitação 3/3, vizinho «outro» 3/3 — mas a faixa nossa é simétrica e mais estreita
(7–13 px contra 9–18), o Krita tem a ARESTA de cima e o esbatido para baixo e a cauda de ~10 px ao longo. De pé (2 px) o
r4 confundiu o nosso com o vizinho (o mesmo papel `Tiled` 128 «falha por falha»). **Falta (motor)**: a ponta-gradiente
dentro do tecto do traço — a rampa por dab desfaz-se na cobertura que soma (com a acumulação sai um degrau, não o
esbatido) — e a faixa deslocada para o lado da mina (o Krita não centra a ponta no cursor).

## B-7 Rough Sketch (Digital, novo) — REPROVADO

Oráculo Krita «c) Pencil-4 Soft» (o CSP Rough pencil). Receita: FIBRAS (Noise `Random` 3 × 16 a 0,7, `rake`), ponta quase
quadrada (quina 0,7) que segue o traço, espaçamento 0,2 (o do Krita), papel 128 · 0,3, jitter 0,15, tamanho 0,9 em S 0,4,
tinta curva 0,3, tamanho 16 (128 × 0,125). Régua 394/454 (440 com as fibras a 0,4). Vizinho Airbrush 0,733. Razão: o
rascunho solto e riscado. Portão `o_rough_sketch_vai_do_claro_ao_escuro_pela_pressao` (a 0,4: tinta 0,538, largura 0,254;
Krita 0,505 · 0,325). Revisões: r1 (média) e r2 (baixa) acharam o oráculo 6/6 e o nosso 6/6, vizinho «outro»; o que os
DOIS repetem: o recto de 5 px nosso falhado/pontilhado contra a linha contínua do Krita, e a textura (r1 «homogénea, sem
direcção»; r2 «riscas esticadas» contra «blocos»). A rampa que entra tarde é a decisão 9.

## B-9 Craft Flat (Digital, novo) — REPROVADO

Oráculo Rebelle «Graphite Pencil/Craft Flat» (PEN_TILT). Receita: achatado 0,38 a 0° (Rebelle 21 × 30 px), quina 1,
dureza 0,7, espaçamento 0,25, jitter 0,15 (os degraus), grão 0,1, a caneta do Rebelle (37,5° → 67,5°, ×4,2), tamanho
14,6, tecto 85 (o recto de 200: 53 px no lado fino). No Wet (régua 422) a cápsula fundia os rectângulos num traço liso.
Régua 386/454; vizinho Graphite 8B 0,864. Razão: letras e hachura de largura variável com a mina chata. Portão
`o_craft_flat_e_chato_e_fixo_na_tela` (0,700; redondo 1,02). Revisão (oráculo 6/6, nosso 6/6, vizinho «outro»): o lápis
do Rebelle MISTURA-SE com o vermelho (média), o nosso pousa por cima; rectângulos duros e translúcidos contra o borrão de
um lado. **Falta (motor)**: dabs soltos no Wet (sem a cápsula) ou a mistura sobre tinta no Digital.

## Custo (`--custo`, smoke, p95 ms a 5 · 40 · 200 px; ⚠️ load 15–20: repetir calmo)

| pincel | p95 | RSS | load |
|---|---|---|---|
| Colored 4B | 12,7 · 6,5 · 5,2 | 86 MiB | 15–16 |
| Tilted Shade | 0,13 · 0,47 · 1,58 | 26 MiB | 17–20 |
| Rough Sketch | 0,24 · 0,74 · 2,02 (fibras 0,4) | 25 MiB | 19 |
| Craft Flat | 0,10 · 0,21 · 0,50 | 25 MiB | 18 |

Caneta deitada: Tilted Shade e Craft Flat a usam (as leis acima); Colored 4B a do 8B; Rough Sketch não.

## Portões e mutação

Os quatro portões de `familias::lapis::portoes` levam o CONTROLO por dentro (a mutação corre em cada corrida e tem de
sangrar): sem esticar a ponta deitada (`tiltLength` 1) de pé ÷ 63° 0,40; a ponta redonda 1,02; sem a pressão na tinta
0,680; sem a orla a onda 0,035. Verdes por nome com `cada_preset_parece_a_sua_midia`, `cada_preset_de_fabrica_volta_ao_
byte`, `the_factory_presets_are_counted`, `a_miniatura…` (o vermelho de `…_e_a_sua_receita` é o Broad Nib, de outra família).
