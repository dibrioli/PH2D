# Exame da família `caligrafia_marcadores` (J3, catálogos D e E) — 21/10

> Briefing: [`BRIEFING_FAMILIA.md`](../BRIEFING_FAMILIA.md). Receitas e portões:
> `crates/ph2d-tool-painter/tests/it/familias/caligrafia_marcadores.rs` (`WET`, `MEIOS`, `EM_ESTUDO`, `EM_ESTUDO_WET`).
> Pranchas: [`pranchas/caligrafia_marcadores/`](../pranchas/caligrafia_marcadores/) (cegas, chaves; `_mypaint`/`_krita`/
> `_rebelle` = 2.º oráculo, 1.ª receita). Réguas, custo, mutação, a cena do tremor: [`caligrafia_marcadores/`](caligrafia_marcadores/).
> 14 revisores cegos novos (05 §2; os «doc.» com a frase da identidade, BRIEFING §2.9).

## Resumo

| # | pincel | veredito | porquê (o resto que dois revisores repetem) |
|---|---|---|---|
| E-1 | Marker Bullet (Wet, existente) | ⚠️ curado, fica, não aprovado | 12/12 linhas o revisor escolheu o NOSSO como original; resta a orla do AA mais dura, as «orelhas» azuis no cruzamento sobre o vermelho (o glaze no AA, motor da tinta) e o 5 px pontilhado do Wet |
| E-2 | Marker Liner (→ Digital, existente) | ⚠️ curado, fica, não aprovado | azul sobre azul: o Multiply do Digital dá marinho `4,47,132`, o Rebelle `13,71,152` (= a meio do Multiply) — motor |
| E-3 | Marker Chisel (novo) | ❌ em estudo | o mesmo azul sobre azul; as pontas do Rebelle são rectas e esmaecem longo, as nossas arredondam (a taper romba que só esmaece não age no Digital) |
| E-4 | Dry Marker (novo) | ❌ em estudo | a 200 px o Rebelle mostra as riscas dos carimbos e é mais baixo; o nosso 5 px quase some |
| D-3 | Broad Nib (novo) | ❌ em estudo | o MyPaint entra com GANCHO e afina com a velocidade (o Wet não lê `speedSize`); a borda dele é macia com estrias |
| D-2 | Script (novo, doc.) | ❌ em estudo | cumpre a descrição nas 4 linhas × 2 revisores; falta: «MUITO estabilizado» (0,97 px de tremor), as pontas em bico côncavo/gancho |
| D-4 | Shale (novo, doc.) | ❌ em estudo | cumpre-a nas 4 linhas × 2; falta: as pontas em «losango» (o bico côncavo), o grão só se vê a 200 px, o tremor residual |
| D-1 | Monoline | 🔀 fundido no Pixel 1px / Hard Round | d **0,066** do Pixel 1px; a identidade (o StreamLine) é a linha Stabilizer |

Fotos: o **Fountain Square do Rebelle refotografado** (`53ceb569d2`, guarda `foto_estavel`): a gestos A era a Fountain Round
B e a tamanhos A/B era uma cena `sobre`. Refeito, ele NÃO é um bico chato: L 25 × T 19 px (quase redondo) e Max Size
~33 px — o oráculo do Broad Nib passou a ser o MyPaint `classic/calligraphy` (L 48 × T 2,3, eixo 48°). O
`indice.csv` da bateria com as três linhas recontadas no fecho da J3 (`28edf951f1`).

**Fecho da J3 — a MÃO DO DONO** (`leis_sistemicas::a_pressao_de_cada_preset_e_a_da_sua_origem`, R_MAO 2,5; estava
vermelho desde `127689b266`): sem a forma de cerdas o Marker Bullet cobria cheio desde p 0,3 (mão 2,72 → **1,52**) e o
Liner no Digital não levava a tinta (**1,33**). Curas de receita: o Bullet com a curva da carga 3 (era 2) → 2,79, a
largura a 0,139; o Liner com `pressureOpacity` 0,8 e o tecto POR DAB na camada (`tecto_por_dab`: sem ele a camada
acumulava os dabs ao tecto e a tinta não seguia a mão) → 2,87, a largura a 0,048. A Reed Pen saiu de `MEDIDOS`.

## O que se aprendeu (comum)

- **Os biséis do Rebelle são FIXOS** (L3): Liner e Chisel deitados a 4° (L 0,94·D, T 0,37·D), o Dry a 100°.
- **A lei da largura do Wet promete o traço HORIZONTAL**: num bisel deitado é o lado fino — o slider a 40 dava um bisel
  de 102 px (o Rebelle: 41). Por isso o Liner foi para o Digital, onde o Size é o eixo longo. ⇒ fila (abaixo).
- **O marcador do Rebelle multiplica A MEIO na mesma tinta**: azul/azul = `lerp(azul, azul², 0,5)` exacto; azul sobre
  vermelho ≈ Multiply inteiro. O Wet (glaze K–M) dá-o; o Digital (Multiply) não. O K–M do Digital (`pigment`) cobre o
  vermelho (`24,99,170`): não serve.
- **A Shape `Gradient`**: os dois 1.os parâmetros a 0,5 (o tom; `[0,0,…]` apaga o carimbo), Size ½ + Offset ½ para uma
  rampa só; o eixo segue o `dab_angle`. Com fluxo alto a soma satura e apaga a rampa; a 0,2 o perfil assimétrico fica.
- **O estabilizador (CAP-28)**, cena do tremor (`bateria_tremor.json`): 0,5 (fábrica) 2,72 px · 0,95 1,21 · 0,98 0,97 ·
  1,0 0,92 mas corta a onda (S 4,3 px).
- **A ponta do taper é côncava** (o smoothstep, `exames/canetas.md`): no lettering lê-se «bico, losango, gancho».

## Por pincel

**E-1 Marker Bullet** (Wet) — receita: a de antes + `shape` lisa, `pickup` 0, `tipClean` 1, `drag` 0, `size_max_px`
101. Oráculo Rebelle `marker_bullet` (+ MyPaint `marker_fat` 354/454). Régua **400 → 434/454**; origem 1.º dos 228 (0,441).
Vizinho fora Dip Pen 1,213; família: passo Liner 1,65. Razão: o marcador redondo que acumula. Cego r1 e r2:
o NOSSO como original nas 12 linhas (baixa/média); r1: «(1) o original arrasta o escuro para fora da faixa» → curado
(o arrasto: `vermelho_depois` 0,022–0,042 → 0); r2: «(1) contorno escuro de 1 px e orelhas nos cantos do cruzamento (2)
névoa da rampa mais longa (3) 5 px pontilhado». (2) é a rampa do dono (decisão 9). Medido: nenhum píxel mais escuro que o
corpo (o «fio» é o nosso AA 139 contra 195); as orelhas são o AA azul do glaze (a dureza 0,6–0,9 não as muda). Custo p95 0,54 · 2,23 · 4,16 ms (5/40/200), RSS 74–86 MiB.

**E-2 Marker Liner** (Digital) — dureza 0,35, `dab_flatten` 0,55 a 0°, quina 0,7, Multiply + a camada do traço (tecto
0,99), `pressureSize` 0,4, sem opacidade pela pressão. Oráculo Rebelle `marker_liner`. Régua **338 → 448/454**; origem
**41.º → 1.º** (0,511). Vizinho fora Pixel 1px 1,491; passo do Chisel 1,02. Razão: o contorno a marcador com o bisel
(fino deitado, largo de pé). Cego r3 e r4 acertam (média): (1) azul/azul marinho — **motor**; (2) a pressão leve
translúcida (curada depois do r4: opacidade fora); (3) +5–11 % de largura e borda (a dureza 0,75 → 0,35 pela borda de 8 px
do Rebelle a 200). Custo p95 0,10 · 0,87 · 1,36 ms, RSS 26–32 MiB.

**E-3 Marker Chisel** (em estudo) — o Liner com dureza 0, quina 0,8, taper romba 0,3 D (opacidade 1), a pressão só pela
tinta. Oráculo Rebelle `marker_chisel` (+ Krita `Marker Chisel Smooth`, 1.ª receita 308/432). Régua 428/454; fora 1,50.
Cego r3/r4 acertam: azul/azul; as pontas (o Rebelle: corte recto que esmaece ~10 px; o nosso arredonda e a taper afina
a largura).

**E-4 Dry Marker** (em estudo; Digital) — bisel de pé a 90°, quina 0,8, a forma `Gradient` (cheia em cima), Multiply,
fluxo 0,2 dentro da camada (tecto 0,9), espaço 0,04 solto, `size_max_px` 110 (o Rebelle pinta 71 a 200). Oráculo Rebelle
`marker_dry` (+ Krita, que é um bisel opaco de borda rasgada: 196/432). Régua 292 → 366/454; fora 1,66. Cego r3/r4 (baixa):
200 px sem as riscas dos carimbos e mais alto; 5 px pálido; a rampa.

**D-3 Broad Nib** (em estudo; Wet Ink) — nasce do Ink Bullet: `achatado` (0,95, 48°), quina 0,8, dureza 0,6, água 0,1,
glaze 0 (cobre como o MyPaint), sem arrasto. Oráculo MyPaint `calligraphy` (régua **414/454**); o Rebelle Fountain
Square (_rebelle 268/454) não tem a identidade. Fora 1,03 (Pixel 1px). Razão: a itálica/gótica — a largura pela direção.
Cego r3: o NOSSO como original nas 6 (média); r4 acerta: (1) o gancho de entrada do MyPaint, (2) a borda macia com
estrias a 200, (3) o rápido mais fino (o Wet não tem `speedSize`). A raio 300 o bico inclinado mede 577 × 600 px (3,8 %). Custo p95 10,8 · 13,5 ·
6,3 ms, RSS 73–96 MiB.

**D-1 Monoline** — Hard Round sem pressão, estabilizador 0,95 (Procreate, 01b §1). **0,066 do Pixel 1px**: fundido.

**D-2 Script** (em estudo, doc.) — Digital: elipse fixa `dab_flatten` 0,75 a **135°** (a 45° saía cheio a subir:
o revisor r1 apanhou-o), dureza 1 + orla de 1 px, `pressureSize` 1, taper 1 D (ponta 0, opacidade 0,5), estabilizador
0,98. Fonte: Procreate Script / Brush Pen (01b §1; every-tuesday.com e fontbundles.net sobre o StreamLine 65–100 %).
Fora 0,605 (Shale) · 1,33 (Hard Round). Sobe ÷ desce 0,35, tremor 0,97 px (Hard Round 2,72). Cegos doc r1–r3: escolhem-no
nas 4 linhas; faltam (repetido): «MUITO estabilizado», as pontas (gancho/barbatana a 200), o contraste «moderado».
Custo p95 0,17 · 0,99 · 1,88 ms.

**D-4 Shale** (em estudo, doc.) — Digital: dureza 0,4, Grain fixo na tela (6, fundo 0,8), `pressureSize` 0,8, taper 1 D
(opacidade 0,5), estabilizador 0,98. Fonte: Procreate Shale Brush (01b §1: «macio e muito estabilizado»; every-tuesday:
«pointed taper»). Fora 0,605 (Script) · 1,27 (Soft Pastel). Cegos doc r1–r2: escolhem-no nas 4 linhas; faltam (repetido):
as pontas em losango côncavo, o grão invisível a 40 px, o tremor residual (1,01 px). Custo p95 0,11 · 2,21 · 2,86 ms.

Custo: `caligrafia_marcadores/custo.txt` (perfil smoke; load 7–16 em todas — o relógio é indicativo). Todos ≤ 16,7 ms de
p95; o Dry a 200 px 15,6. Caneta: só a pressão da bateria; nenhum usa a inclinação.

## Portão e mutação

`familias::caligrafia_marcadores::cada_letra_e_marcador_tem_a_sua_identidade` (os da fábrica e os EM_ESTUDO, pela receita):
o Bullet redondo e que acumula; o Liner e o Chisel deitados (de pé ÷ deitado 2,25–2,57 contra 1,00) e que multiplicam; o
Chisel que entra pela tinta (0,54 contra o Liner 0,98); o Dry de cima ÷ baixo 2,11, de pé 0,33, que multiplica (o Flat
Block-in tem a mesma ponta de pé, 1,00); o Script fino a subir (0,35) e estabilizado; o Shale macio (orla 3,6 px) e
estabilizado. `os_reprovados_da_familia_nao_estao_na_fabrica`. Mutação (`PH2D_MUTACAO_CALIGRAFIA`, sem tocar em ficheiros
partilhados; [`mutacao.txt`](caligrafia_marcadores/mutacao.txt)): controlo verde; 6/6 sangram (glaze 0 · achatado 0 · sem
taper · sem Gradient · estabilizador 0,5 · dureza 1). Verdes por nome: os da fábrica (contagem, receita, byte,
miniatura, rótulo), `os_fios`, as mídias, a largura; `sem_glaze_…` só reprova nos fnv de outras famílias a meio.

## O que falta (fila)

1. **O glaze do marcador no Digital** (a mesma tinta escurece a meio, outra multiplica): Liner e Chisel. Ou, no Wet, **a
   lei da largura de um bisel deitado prometer o eixo LONGO** (o Rebelle) — aí os biséis voltam ao Wet com o glaze certo.
2. **A taper romba que só esmaece no Digital** (`tip 1 + opacity`): o Chisel (pontas rectas que esmaecem, largura igual).
3. **Velocidade no Wet** (`speedSize` não é knob do Wet) e o gancho de entrada do MyPaint: o Broad Nib.
4. **A ponta afilada convexa** (o afilar de lettering; hoje o bico côncavo do smoothstep): Script e Shale.
5. **Um estabilizador que alisa sem cortar a curva** (o StreamLine que apanha o fim): Script, Shale.
6. **As riscas dos carimbos do Dry** a 200 px (dabs soltos com a borda visível dentro da camada).
7. O Wet a 5 px (pontilhado) e o AA do glaze sobre o vermelho (o Bullet; motor da tinta).
