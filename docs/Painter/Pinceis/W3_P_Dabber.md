# W3 P-Dabber — Dabber 2 · Dabber 3 · Dabber 4 · Fan com os motores da W3 (27/10)

Agente P-Dabber da MISSÃO W3 do Óleo Real. Método da W2: bancada [`04_BANCADA.md`](04_BANCADA.md) +
[`05_REVISAO_CEGA.md`](05_REVISAO_CEGA.md), **sempre `PH2D_EXAME_OLEO=1`**. Oráculo:
`fixtures/rebelle_bateria/OIL_AND_ACRYLIC/dabber_*`. Custo ([`REGRAS_DE_CUSTO.md`](REGRAS_DE_CUSTO.md)): 2 rondas,
≤ 4 variantes por cura e por preset, um revisor `sonnet` por ronda (r1 viu os 4, r2 viu D2/D4/Fan).
Nada entrou na fábrica: os valores finais estão nos ficheiros de [`W2_Dabber_candidatos/`](W2_Dabber_candidatos/)
(o JSON é o do Dabber de fábrica com as mudanças abaixo; a miniatura continua a do Dabber, por regerar).

## Resultado

| preset | veredito | rondas | régua (W2 → W3) | vizinho (d; limiar 0,591) | escala | prancha |
|---|---|---|---|---|---|---|
| Dabber 2 | **curado-não-aprovado** | 2 | 370 → 232/454 | Dabber 0,872 | 0,542 | [`W3_Dabber_2.jpg`](pranchas/W3_Dabber_2.jpg) |
| Dabber 3 | **aprovado** (r1) | 1 | 280 → 304/454 | Wax Scrape 1,458 | 0,673 | [`W3_Dabber_3.jpg`](pranchas/W3_Dabber_3.jpg) |
| Dabber 4 | **aprovado** (r2) | 2 | 336 → 162 (r1) → 310/454 | Dabber 1,253 | 1,222 | [`W3_Dabber_4.jpg`](pranchas/W3_Dabber_4.jpg) |
| Fan | **curado-não-aprovado** (o revisor não achou o Rebelle; a régua da cerda discorda) | 2 | 292 → 264 (r1) → 300/454 | Dabber 1,196 | 1,0 | [`W3_Dabber_Fan.jpg`](pranchas/W3_Dabber_Fan.jpg) |

⚠️ A régua da W2 media sem o Óleo Real; a de agora mede com ele (o relevo e o brilho somam à `textura` e tiram
`cobertura`). Os números não são a mesma régua — comparam-se só dentro da W3.

⚠️⚠️ **Viés do revisor (as duas rondas):** nas 4 pranchas ele PAROU o NOSSO com o Rebelle 6/6 (a «imitação» dele é
sempre o oráculo, o vizinho sempre «claramente outro»), mas escolheu o NOSSO como original 6/6 — e disse porquê na
r2: *«chutei original/imitação pelo critério "original tem sombra e relevo, imitação é chapada"»*. O relevo do Óleo
Real lê-se como «o programa de referência». Pela regra (§3 da 05) ele não achou o Rebelle; mas isto é um sinal de
SEMELHANÇA (par certo), não de indistinguível. Por isso o veredito exige também a régua.

## Valores finais (sobre o Dabber de fábrica)

- **Dabber 2**: `bristleSize` 3 · `paperGate` 0,5 · `bristleHeight` 0,4 · `textureSizeExp` 0,7 · `brush.jitter` 0,6 ·
  `brush.texture` Stucci Tiled 120, Brightness 0,55 (params `[0,5; 0,55; 0,5; 0,5; 0,5; 0,5]`).
- **Dabber 3**: `bristleSize` 8 · `bristleCount` 400 · `bristleLift` 0,8 · `felt` 0 · `emboss` 0,8 · `textureSizeExp` 0,7 ·
  `brush.jitter` 0,5 · `brush.texture` **Flakes** Tiled 18, Size 0,75, Elongation 0,3, Irregularity 0,9, Invert 1,
  lei Subtract (`lei.mode` 1).
- **Dabber 4**: `dabRotation` 0 · `textureSizeExp` 0,5 · `brush.dab_flatten` 0,6 · `brush.jitter` 0,1 ·
  `brush.continuous_spacing` false · `brush.spacing` 0,3 · `brush.texture` Stucci Tiled 180, Brightness 0,6.
- **Fan**: `dabRotation` 0 · `bristleSize` 3 · `emboss` 0,8 · `bristleLaneWidth` 3 · `bristleRemove` 0,3 ·
  `pigmentPerDab` 2000 · `textureSizeExp` 0,8 · `textureSizeExpAcross` 0 · `brush.dab_flatten` 0,78 ·
  `brush.dab_angle_deg` 0 · `brush.jitter` 0,15 · `brush.continuous_spacing` true · `brush.shape` None ViewPlane,
  ângulo 90, Size `[0,3; 1]`, **Flow desligado**.
- `size_px` continua 51: a escala da tabela é o que o preset multiplica ao entrar.
- sha256 (16): Dabber 2 `f5c348cc9b1132fd` · Dabber 3 `b2aac0dbac726273` · Dabber 4 `dfffe146d42871a9` · Fan `1e678644d60c705a`.

## Números × Rebelle (foto A; tamanhos ÷ D pela régua da K3b/K2)

**Dabber 3** (lascas, `cerda_regua.py d3`, 40 · 200):

| | frac | tam ÷ D | along | solidez |
|---|---|---|---|---|
| Rebelle | 0,153 · 0,25 | 0,159 · 0,104 | 3,35 · 1,91 | 0,70 · 0,61 |
| K5 (Tiled 16, Size 0,85, Elong 0,4, Irr 0,5) | 0,178 · 0,149 | 0,174 · 0,110 | 2,50 · 3,30 | 0,69 · 0,59 |
| **final** (Tiled 18, Size 0,75, Elong 0,3, Irr 0,9) | 0,160 · 0,156 | 0,149 · 0,122 | 3,03 · 2,82 | 0,67 · 0,54 |

Fica: o buraco a 200 é ainda pouco (0,156 × 0,25) e as lascas alongam a 200 onde as do Rebelle encurtam.

**Dabber 4** (gestos, recto lento · vertical): largura 33→35 · 18→16 ✓; cobertura 0,72→0,77 ✓ · 0,86→0,76;
opacidade 0,98→0,89 ✗; textura 29→15 ✓. A r1 (Stucci Brightness 0,5) dava cobertura 0,51 e opacidade 0,86: o
grão comia o traço (o revisor: «granulado, esparso, mais pálido»). Brightness 0,5/0,55/0,6/0,65 → cobertura
0,51/0,63/0,76/0,81: 0,6 é a que casa.

**Dabber 2** (buracos, `manchas.py`, ÷ D): Rebelle fracção 0,003 · 0,026, tamanho 0,078 · 0,043. Stucci 120 γ 0,7
Brightness 0,5 → 0,051 · 0,049 (demais); 0,6 → 0,006 · 0,009 (de menos); 0,7/0,8 ≈ fechado. Régua da r2: largura
32→38 ✗, cobertura 0,89→0,77 ✗, **textura 7→17 ✗** (o Rebelle é amassado e fosco; o nosso tem crista de óleo, brilho
e sombra projetada).

**Fan** (gestos): recto 14→14 ✓; vertical 33→27 (r1, com Flow: 33→17). Anisotropia (fios através, 40 · 200):
Rebelle 1,13 · 1,84 · K5 com Flow 0,96 · 1,20 · **final sem Flow 0,36 · 0,59**.

## O cruzamento (chaves e cegas em `/tmp/…/scratchpad/p_dabber/gerados/`)

- **r1** (os 4): D2 — o revisor parou o NOSSO com o VIZINHO (Dabber) 5/6 e deixou o Rebelle como «o outro»: o nosso
  era o Dabber de novo (confiança baixa; «original largo e grumoso, imitação mais fina»). D3 — nosso = original 6/6
  (média), imitação = Rebelle 6/6, Wax Scrape claramente outro; diferenças: o nosso tem sombra de relevo e miolo
  preto nos vãos, o Rebelle chapado; buracos brancos maiores a 200 no Rebelle; a rampa 0→1 mais nítida no Rebelle.
  ⇒ **aprovado**. D4 — nosso = original 6/6 (baixa), par = Rebelle; diferenças: o nosso esparso e pálido, o Rebelle
  denso, saturado e opaco, borda mais lisa; a régua (162) concorda ⇒ cura. Fan — nosso = original 6/6 (baixa), par =
  Rebelle; diferenças: o nosso picotado (tracejado), os verticais do Rebelle largos e felpudos, a 200 o Rebelle de
  espigões longos ⇒ cura.
- **r2** (D2, D4, Fan): os três nosso = original 6/6, imitação = Rebelle 6/6, Dabber claramente outro. D2 (média):
  «o original tem sombra projetada e relevo com buracos escuros; a imitação chapada, lisa, mais saturada; a 200 rala e
  salpicada, borda irregular». D4 (baixa): «sombra de relevo; a 200 blocos de toque em bandas com borda ondulada × grão
  fino uniforme; as rampas desmaiam mais macias». Fan (baixa): «a 200 faixa baixa achatada × cerdas verticais
  espetadas; verticais do sobre largos, translúcidos e felpudos no Rebelle; espiral densa e lisa × grão solto».

Veredito: **D3** (r1) e **D4** (r2) aprovados — par certo 6/6, Rebelle não achado, régua no nível da W2 e as
diferenças que restam são do RELEVO do Óleo Real (lado do material, comum a todos). **D2** reprovado pela régua
(textura 2,4×, cobertura, largura) — a diferença escrita: o Rebelle é uma massa fosca e amassada, de borda rasgada e
buracos raros só a 200; o nosso tem crista e brilho de óleo e borda em gomos. **Fan** reprovado pela régua da cerda —
a diferença escrita: o que o revisor viu no Rebelle (cerdas verticais espetadas a 200 px, verticais felpudos) é a
assinatura do leque, e o nosso perdeu-a para ter a largura certa por direcção (abaixo).

## Motor que falta (fila; nada começado — fora dos ficheiros deste agente)

1. **Fan — fios ATRAVÉS sem rodar a pegada.** Medido: o Shape com **Flow** (a composição da K3b/K5, os fios através
   do traço) faz a pegada ACHATADA seguir a direcção do traço — os verticais saem com a largura do recto (17 contra
   33 do Rebelle; sem Flow 27–31). Sem Flow a pegada fica presa ao ecrã (certo para um leque), mas os fios também, e
   a anisotropia cai a 0,36 · 0,59 (Rebelle 1,13 · 1,84). Falta o Flow só da TEXTURA de cerdas (o tapete no
   referencial do traço) com a silhueta `dab_flatten`/`dab_angle_deg` fixa.
2. **Dabber 2 — relevo fosco e amassado.** No Óleo Real o `emboss` não muda a foto (byte-idêntico), nem
   `pigmentPerDab` 2000/3000 nem `bristleSize` 2/3 (D2 e D4, md5 iguais); `bristleHeight` 0,4 mexe pouco
   (textura 14,7 → 14,8). A textura do Rebelle (amassado fino, sem crista) não tem knob de preset no Óleo Real.
3. **Dabber 3 — buraco que cresce a 200** (0,156 × 0,25) e lasca que encurta a 200 (o `textureSizeExp` escala os dois
   eixos da lasca por igual).
4. `roughness`/`iorLevel` no preset: não medidos isoladamente (só juntos com `bristleHeight`, sem efeito separável)
   — não entram nos valores finais.

## ⛔ Recusas MEDIDAS

| tentativa | medido | porquê não |
|---|---|---|
| Fan com Shape Flow (K5: ângulo 90, Size `[0,3; 1]`, γ 0,8, através 0) | vertical 33→17, espiral 25→16 | o Flow roda a pegada achatada com o traço: o leque deixa de ter largura por direcção |
| Fan `dab_angle_deg` 90 com Flow | todos os gestos 29 px | a pegada segue o traço, larga em toda a direcção |
| Fan `dab_flatten` 0,6 | 200 px D 98 (Rebelle 89) | 0,78 dá 13–14 no recto como o Rebelle |
| D3 Flakes Elong 0,25 / Irr 0,9 (Tiled 16) | frac 0,126 · 0,09, along 2,69 · 5,04 | a 200 as lascas viram riscas |
| D3 Flakes Tiled 14, Size 0,8, γ 0,8 | tam 0,165 · 0,204 | lascas a 200 o dobro das do Rebelle |
| D4 sem grão, `bristleSize` 1, `bristleNoiseLight` 0,4 | cobertura 0,16, textura 155 | o tapete fino sozinho não pinta |
| D4 Stucci Brightness 0,65 | cobertura 0,81, textura 7,6 (Rebelle 0,72 · 29) | fecha o salpicado |
| D2 Stucci Brightness ≥ 0,58 com `jitter` 0,3 | buracos 0,001 a 200, largura a 200 D 130 (Rebelle 174) | sem buracos e estreito: o jitter 0,6 é o que dá a largura a 200 |
| `emboss` no Óleo Real (D2 0,5 → 0,3) | foto byte-idêntica | o relevo do Óleo Real não lê o emboss |
