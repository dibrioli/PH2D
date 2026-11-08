# Composite Brush × Color mixing (Digital) — 2026-11-07

> Pergunta do dono: *«a impressão que tenho é que consigo melhores resultados usando Composite Brush do que com
> esse novo Color mixing. avalie se isso é verdade»*. Avaliação por medição e imagem, sem defender a feature nova.
> `[M]` = medido nesta sonda · `[D]` = deduzido do código ou da imagem.

## Cena e sonda

- Sonda: `crates/ph2d-tool-painter/tests/it/sonda_composite_vs_mixing.rs` (`#[ignore]`; `sonda_cmp_varre` varre o
  Color mixing, `sonda_cmp_folha` faz folha + tabela + relógio). `--profile smoke`, CPU.
- Cena (a mesma de `a_mistura_opaca_nos_presets`): mancha vermelha `200,0,0` em traços próprios (x 140..300), depois
  UM arrasto azul `0,0,220` de x 60 a 340 em y 120, tamanho 40, presets de fábrica reais (redondo, **G-Pen**,
  **Sketch Pencil**). Pressão constante 1 (rato) e pressão variável `0,15 → 1 → 0,15`.
- Folhas (linhas rotuladas; esquerda pressão constante, direita variável):
  [redondo](handoffs/material_2026-11-07/comparacao_composite_vs_mixing_redondo.png) ·
  [G-Pen](handoffs/material_2026-11-07/comparacao_composite_vs_mixing_gpen.png) ·
  [Sketch Pencil](handoffs/material_2026-11-07/comparacao_composite_vs_mixing_sketch_pencil.png)
- O Composite só tem 4 operações (`Brush · Smear · Blur · Erase`, cotas 3/1/1/2); para «levar a tinta» só o
  **Smear** serve. Montado como o artista o monta no cartão (`+` → camada, Strength por camada). Testadas 5 pilhas
  (cima → baixo): `Smear1/Brush1`, `Smear.6/Brush1`, `Smear1/Brush.6`, `Brush.5/Smear1`, `Blur.4/Smear.8/Brush1`.
- **Melhor Composite encontrado:** `Smear 1,0` (camada 1, em cima) sobre `Brush 0,5–0,6` (camada 2). Com o Brush a
  1,0 o Smear não tem o que arrastar (sai azul puro, `dentro 0,0,220` nos três pincéis) [M].
- Color mixing: fábrica `0,5 / 1 / 0,5` nos dois modos; afinado pela varredura (36 combinações × pincel):
  **Blend `0,85 / 1 / 0,25`** e Running `0,5 / 1 / 0,25` (o Running quase não melhora sobre a fábrica) [M].

## Tabela (pressão constante; `dentro` = RGB médio em x 260 sobre o vermelho; `cauda` = px após a borda com R−G > 25;
`branco` = RGB médio sobre o branco antes do vermelho; Off dá `0,0,220`; ms = mínimo de 5, intercalado)

| pincel | variante | dentro | cauda | branco | borda | grão | ms |
|---|---|---|---|---|---|---|---|
| redondo | Off | 0,0,220 | 0 | 0,0,220 | 20,0 | 0,0 | 3,2 |
| redondo | Mix Blend fábrica | 98,7,121 | 38 | 120,101,213 | 15,7 | 10,5 | 4,0 |
| redondo | Mix Blend afinado | 77,0,136 | 36 | **83,56,199** | 17,8 | 10,0 | 3,9 |
| redondo | Mix Running fábrica | 107,29,139 | 38 | 125,114,224 | 14,5 | 6,2 | 4,0 |
| redondo | **Comp Smear1/Brush.6** | 102,102,234 | 32 | 104,104,234 | 15,4 | 1,0 | 8,7 |
| redondo | Comp Brush.5/Smear1 | 128,128,238 | 33 | 128,128,238 | 14,5 | 0,0 | 8,9 |
| redondo | Comp Blur/Smear/Brush | 0,0,220 | 0 | 1,1,220 | 24,1 | 0,8 | 26,3 |
| G-Pen | Off | 0,0,220 | 0 | 0,0,220 | 0,7 | 0,0 | 18,3 |
| G-Pen | Mix Blend fábrica | 99,10,125 | 15 | 123,120,233 | 0,6 | 4,4 | 12,4 |
| G-Pen | Mix Running fábrica | 108,37,147 | 23 | 128,126,236 | 0,5 | 2,6 | 12,6 |
| G-Pen | Mix Blend afinado | 75,0,138 | 0 | 97,92,228 | 0,8 | 6,3 | 12,4 |
| G-Pen | Comp Smear1/Brush1 | 0,0,220 | 0 | 0,0,220 | **10,4** | 6,5 | 11,1 |
| G-Pen | Comp Smear1/Brush.6 | 52,52,227 | 0 | 52,52,227 | **10,3** | 5,6 | 10,9 |
| G-Pen | Comp Brush.5/Smear1 | 32,32,224 | 57 | 34,34,224 | 0,7 | 3,5 | 11,4 |
| Sketch Pencil | Off | 95,0,116 | 17 | 113,88,205 | 21,2 | **39,4** | 23,5 |
| Sketch Pencil | Mix Blend fábrica | 108,5,107 | 53 | 131,106,208 | 17,0 | 16,6 | 14,3 |
| Sketch Pencil | Mix Running fábrica | 116,23,121 | 53 | 136,119,218 | 15,7 | 13,0 | 14,1 |
| Sketch Pencil | Mix Blend afinado | 88,0,123 | 49 | 98,67,195 | 16,0 | 16,3 | 14,3 |
| Sketch Pencil | **Comp Smear1/Brush.6** | **135,0,72** | 38 | 173,173,244 | 22,5 | 7,7 | 19,4 |
| Sketch Pencil | Comp Brush.5/Smear1 | 148,0,58 | 38 | 187,187,246 | 19,4 | 10,3 | 19,4 |
| Sketch Pencil | Comp Blur/Smear/Brush | 2,0,218 | 0 | 8,8,221 | 19,1 | 4,6 | 39,5 |

(`borda` = largura vertical da rampa 10–90 % sobre o branco; `grão` = desvio-padrão da luminância no miolo sobre o
branco. Com pressão variável a cauda cai para ~22 px no Mix e no Composite do redondo, e para 0 no G-Pen [M].)

## Veredito por critério

1. **Levar o vermelho.** No **Sketch Pencil o dono tem razão** [M+olho]: o Composite `Smear1/Brush.6` arrasta
   vermelho forte e saturado (`135,0,72`, um carmim que se lê como «tinta puxada»), enquanto o Color mixing dá um
   roxo de média RGB (`108,5,107`) — mais «misturado», menos «levado». No redondo e no G-Pen é o **contrário**: o
   Composite ou não leva nada (Brush a 1) ou o que «leva» é o vermelho a entrar pelos lados de uma língua azul que
   recua (forma de cometa, não mistura); o Color mixing dá a mistura contínua ao longo do traço com cauda 15–38 px.
2. **Suavidade.** Color mixing: liso no corpo, mas com **anéis concêntricos na cauda** do redondo e **contas
   soltas** na cauda do G-Pen (ver as folhas, à direita da mancha) [olho]. Composite: liso no redondo; **no G-Pen
   está partido** — estrias brancas dentro do traço (`Smear1/Brush1`, `Smear1/Brush.6`) e manchas
   vermelho/branco com Blur [olho]; isto é defeito do Smear com o G-Pen, não gosto.
3. **Carácter do pincel.** G-Pen: o Color mixing mantém a borda dura (0,5–0,8 px) e as pontas afiladas; o Composite
   **perde a ponta final** (sai redonda) e alarga a borda para ~10 px [M]. Sketch Pencil: o Color mixing **apaga o
   grão** (39 → 13–17, e no olho o traço fica liso como aerógrafo — limite conhecido: Grain/Shape não moldam o
   depósito); o Composite também o reduz (7–10) mas o olho ainda vê papel nas zonas claras [M+olho].
   ⇒ **No Sketch Pencil o Composite ganha; no G-Pen o Color mixing ganha claramente.**
4. **Cor sobre o branco.** **Os dois desbotam.** Mix fábrica: ~55 % de azul (`120,101,213`); Composite melhor:
   pior ainda no lápis (`173,173,244`, quase lavanda) porque o Brush tem de estar a 0,5–0,6 para o Smear ter o que
   arrastar [M]. O único que segura a cor e ainda leva o vermelho é o **Mix Blend afinado** `0,85/1/0,25`
   (`83,56,199`, cauda 36 no redondo) — mas no G-Pen a cauda cai a 0.
5. **Controlo.** Color mixing: 1 seletor + 3 números (+2 de pressão), cada um muda uma coisa previsível (amount =
   quanto azul, stretch = quanto dura o vermelho) [M: a varredura é monótona]. Composite: montar 2 camadas, escolher
   a ORDEM e duas Strengths; **não-óbvio**: com Brush a 1,0 o Smear não faz nada visível, e a ordem muda a forma do
   traço (cometa vs mancha) [M]. Mais poder, menos previsível.
6. **Custo** (CPU nos dois; nenhum dos dois vai à GPU [D: nenhuma rota GPU em `stamp_route/mistura.rs` nem em
   `composite_pilha.rs`]). Redondo: Mix ≈ Off (+0,8 ms); Composite ×2,7 (8,7 ms); com Blur ×8 (26 ms). G-Pen e
   Sketch Pencil: o Mix é **mais barato** que o Off (12,4 vs 18,3; 14,3 vs 23,5 — salta a rota da textura); o
   Composite fica no meio (11–19 ms) e o Blur dobra (25–40 ms) [M].

## Conclusão

**O dono tem razão em parte.** Onde ele pinta com lápis (Sketch Pencil e família com grão), o Composite
`Smear 1,0 sobre Brush 0,5–0,6` dá um resultado mais rico: leva mais vermelho, saturado, e guarda algum grão; o Color
mixing ali sai liso e roxo-acinzentado. **Fora do lápis não**: no redondo o Composite só faz um cometa, e no G-Pen
está partido (estrias brancas, perde a ponta). E nos dois o azul sobre o branco desbota.

## Recomendação (ordem)

1. **Manter os dois** — não fazem a mesma coisa: o Smear do Composite *empurra* a tela; o Color mixing *suja o
   reservatório*. Fundir o Color mixing como quinta operação do Composite não cura nenhum defeito medido e herda o
   «Brush a 1 anula tudo».
2. **Melhorar o Color mixing (X1): o depósito tem de passar pelo Grain e pelo Shape** — é a razão inteira de o
   Sketch Pencil perder para o Composite. Gate: `grão` do Sketch Pencil com Mix ≥ 80 % do Off (hoje 42 %).
3. **(X2) Não desbotar sobre o branco:** o branco do papel não deve diluir o reservatório como se fosse tinta (com
   amount 0,5 o azul sobre branco chega a 55 %). Gate: `branco` com Mix fábrica ≥ 90 % da saturação do Off.
4. **(X3) Cauda:** tirar os anéis do redondo e as contas do G-Pen (espaçamento dos dabs na cauda).
5. **Composite com G-Pen:** registar no BUGS as estrias brancas do Smear e a ponta final perdida (fora desta linha).
6. Factory: trocar o default do Color mixing para Blend `0,85/1/0,25` só depois de X2 — hoje ganha no redondo e
   perde a cauda no G-Pen.
