# J6 — aguarela e texturas por VALORES (I-9 · I-10 · I-11 · K-9 · F-10)

> Exame 28/10 (`BRIEFING_FAMILIA.md`, `REGRAS_DE_CUSTO.md`): 2 rondas, ≤ 4 variantes cada. Biblioteca FIXA
> `/tmp/claude-1000/j6_digital/biblioteca` (65), `PH2D_EXAME_DIR=/tmp/claude-1000/j6_aguarela`, limiar da matriz
> 0,591 (pedido ≥ 0,596). Receitas fora da fábrica, nas listas `EM_ESTUDO_J6` (o coordenador promove); o estudo grava-se
> com `escreve_os_estudos_da_aguarela` (`"receita": "<nome>"`) e o Digital com `familias::texturas::escreve_os_em_estudo`.
> A revisão cega NÃO foi feita aqui. Pranchas: `pranchas/J6a_<nome>.jpg` + `_cega.jpg` + `_chave.txt`.

| Pincel | Receita escolhida (ficheiro) | Régua | Vizinho (d) | Estado |
|---|---|---|---|---|
| I-10 Fringe Bloom | «Fringe Bloom», Digital (`texturas::EM_ESTUDO_J6`) | 432/454 | Mapping Pen 1,011 | pronto p/ cega |
| F-10 Wax Crayon | «Wax Crayon J6 D», Wet (`pastel_carvao::EM_ESTUDO_J6`) | 436/454 | Graphite 8B 0,783 | pronto p/ cega |
| K-9 Grunge | «Grunge C», Wet (`texturas::EM_ESTUDO_J6_WET`) | 372/454 | Express Oil 0,890 | pronto p/ cega (diferença escrita) |
| I-9 Granulation | «Granulation C», Wet (`aguarela::EM_ESTUDO_J6`) | 259/454 | Wet Round 0,930 | **motor em falta** |
| I-11 Spatter Watercolor | «Spatter Watercolor G», Wet (`aguarela::EM_ESTUDO_J6`), J6s | 43/83 | Burn 2,239 | pronto p/ cega (diferença escrita) |

## I-10 Fringe Bloom (oráculo Krita `j) Watercolor Fringe`)

- **Motor: Digital, não Wet.** O oráculo é um carimbo OPACO de folha serrilhada, chapado, o azul tapa o vermelho, sem
  água nem orla — nada de aguarela. Receita: Shape de imagem NOSSA (`franja()`: 11 dentes de comprimento sorteado,
  ângulo «losango» sem trigonometria), dureza 1, Constant, spacing 0,2, `jitter_rotate` 1, pressão → opacidade
  (largura fixa, como o Krita).
- r1: A 432 · B (pressão 0,3 no tamanho, spacing 0,12, escala ±0,25) 430 — a rampa do B afina, a do Krita esmaece.
  r2: C (9 lóbulos redondos) 426 · D (C + escala ±0,3) 428 — lóbulos curtos e rombos: piores à vista. Fica A.
- **Ainda vê:** as folhas do Krita são lanceoladas (incham a meio e afinam), as nossas triângulos; o início do traço
  de 200 px no Krita tem folhas translúcidas; o recto de 5 px do Krita é mais pálido.
- **Razão de uso:** a borda viva de folhagem/franja num gesto — sebes, copas, pelagem estilizada, molduras.

## F-10 Wax Crayon (oráculo Rebelle `Pastel Oil/Wax Pencil`)

- J4: reprovado (revisor juntava-o ao Graphite 8B). r1 (sem grão, dureza 0,5–0,6, spacing 0,25–0,3, B com quina 0,6):
  412 · 400, vizinho Hard Round 0,626/0,596 — chapado de borda dura e a rampa a AFINAR (o Rebelle esmaece).
- r2: C = pressão na opacidade (tamanho 0,2, curva 1,5), dureza 0,3, carga 200, as manchas da cera (Noise Random 6),
  a orla do dual: 434, Graphite 8B 0,839. **D = C sem o dual e jitter 0,12: 436, Graphite 8B 0,783** (escolhido: o
  dual mordia uma dentada no recto de 200 px). Pontas redondas (sem `pontas_rectas`, como medido na J6).
- **Ainda vê:** a borda do Rebelle é um cordão de carimbos (contas, festões) e a 200 px os carimbos lêem-se; a nossa
  borda é uma faixa lisa e esbatida. O vizinho Graphite 8B tem contas parecidas — a cega deve olhá-lo.
- **Razão de uso:** o giz de cera denso e mosqueado de pressão leve a translúcida — ilustração infantil, cor chapada
  com textura sem o risco do lápis.

## K-9 Grunge (oráculo Rebelle `Oil/Grunge/Grunge`, OIL_AND_ACRYLIC)

- Base: as cerdas do Express Oil mais cheias (600 × 3 px, faixa 2, força 2,5, pigmento 120, papel 0,3), pasta
  `ExpressOil/Grain` (fora de `Oil/`: sem o Real Oil). r1: com a 2.ª ponta (L6) a rasgar a borda: 344 · 264 — o dual
  dá DENTADAS redondas, não rasgos. r2 sem dual, `bristleStreak` 0,6 · corte 0,5 · 200, `bristleTipRagged` 0,6:
  **C 372** · D (streak 0,9 · 120, ragged 1, lift 0,4) 356.
- **Ainda vê:** as riscas nossas são contínuas e regulares (pente); as do Rebelle partem-se em traços curtos e
  irregulares, a borda em lascas, falhas brancas soltas — é a textura de IMAGEM do Rebelle por baixo das cerdas.
  Fila: grão de imagem rasgado sob as cerdas do Wet (a `shape` Image no Wet), medir antes da cega.
- **Razão de uso:** sujidade, ferrugem, madeira gasta — a passada riscada e partida que nenhum óleo da fábrica dá.

## I-9 Granulation (oráculo Rebelle `Watercolor/Granulation/Granulation`) — motor em falta

- r1 (papel 1,2/1,5, carga 25/40, `extGranulation`+`extValleyGran` 1, B +`extGrainDither` 3): o recto quase não
  pinta (a régua nem mede a largura), orla escura. r2: C (papel 0,8, carga 120, dither 6, sem orla) 259 · D (papel
  0,4, pó 0,8 gota 1,5) 222.
- **Medido (recto de 200 px, cena tamanhos A):** o grão píxel-a-píxel |Δ| entre vizinhos — **Rebelle 12,7 · nosso 2,6**;
  cobertura > 8: 0,285 · 0,347; escuro médio 22,8 · 38,7. O Rebelle é um pontilhado de pontos de 1–2 px; o nosso uma
  lavagem lisa com estrias. Nenhum knob do Wet dá poros de 1–2 px (o papel é da obra, liso a essa escala; o dither é
  do render e fraco).
- **Fila (motor):** a granulação em POROS — o pigmento assenta num ruído de alta frequência por traço (1–2 px), a
  quantidade pela água; ou testar primeiro o Digital (grão Noise fino, opacidade baixa).

## I-11 Spatter Watercolor (oráculo Rebelle `Watercolor/Spatter/Spatter`) — motor em falta

- r1: o spray do carimbo (L4) no Wet (6 marcas, jitter 0,7, partícula 0,04): as gotas caem como DISCOS GRANDES pela
  tela inteira (nada perto do traço) — 6/22. ⚠️ **A variante a jitter 0,8 · escala 0,95 mata o gerador por memória**
  (OOM a 24 G no `escreve_os_estudos_da_aguarela`, a tabela da largura; 3 corridas: 12, 10 e 6 marcas a jitter 0,8 · escala 0,95; o A a 0,7 · 0,9 grava).
- r2 pelo pó do Wet: C (pó 0,95 gota 4) 13/83 · D (gota 3 + jitter 0,5) 26/81 — um véu esbatido, sem gotas.
- **Medido (200 px):** grão píxel-a-píxel **Rebelle 13,7 · nosso 1,8**; cobertura > 8: 0,088 · 0,416. O Rebelle são
  gotas soltas de 2–8 px numa nuvem 1,5× a largura; o nosso uma nuvem lisa.
- **Fila (motor):** o spray do Wet (espalhamento e partícula como no Digital, sem o OOM) — ou o Digital `Spray Can`
  (J-2, gotículas pela área) com a cor pálida, a medir.
- **J6s — os dois defeitos do motor, curados (commit `3cc04a5d93`, gates `o_spray_do_wet`):** (1) os discos pela tela
  eram a TABELA DA LARGURA: `largura_que_se_ve` lê por coluna, e numa nuvem lia UMA gota (raio do motor 803 → 63 px);
  a promessa pintava o traço de 40 px com o raio 386 — tinta a p99 317 px. O spray mede-se agora pela nuvem
  (`largura_da_nuvem`): p99 40 px. (2) o OOM era uma LANE POR MARCA (o limiar da partícula, o salto do pincel), cada
  uma com a sua janela de rasto: 645 rastos e 277 MiB num raio cru 4, 64 raios em paralelo > 24 G (hoje também o
  0,7 · 0,9). A marca do spray é uma GOTA: pousa e solta a lane. Depois: 1 rasto, 31 MiB; a tabela inteira 2,8 / 4,1
  / 3,1 GiB a jitter 0,6 / 0,7 / 0,8; os quatro estudos gravam em 187 s.
- **J6s ronda única** (`PH2D_EXAME_DIR=/tmp/claude-1000/j6_spray`, biblioteca fixa da J6): E (o r1 tal qual) 29/72,
  Bokeh 3,09 — gotas soltas perto do traço mas raras, o recto de 200 quase vazio; F (gotas em PÍXEIS 2–8 px, 8
  marcas) 35/83; **G (F + o véu: 35 % das marcas ×2,5 e pálidas) 43/83, Burn 2,24** (escolhido); H (12 marcas, nuvem
  larga, pigmento 25) 34/66. Prancha `pranchas/J6s_Spatter_Watercolor{,_cega}.jpg` + `_chave.txt`.
- **Ainda vê (G):** os gestos são um véu de gotas como o do Rebelle, mais escuro e de gotas maiores; a 200 px a nuvem
  tem o tamanho certo mas as gotas são poucas, grandes e escuras (opacidade 0,45 contra 0,09) onde o Rebelle tem
  milhares de pontos pálidos de 1–2 px (a régua lê largura 8 contra 188); a 5 px o nosso é um CORDÃO escuro (as gotas de 2–8 px maiores que o pincel), o Rebelle
  quase invisível. Fila: a densidade por área (contagem que cresce com o diâmetro) e a gota abaixo do pincel fino.

## Custo e portões

- Custo `--custo` e portão por foto: não corridos (pincéis ainda sem cega). `os_em_estudo_nao_estao_na_fabrica`
  (texturas, agora com o `EM_ESTUDO_J6`) verde por nome.
- Load 15–40 durante as corridas (outras linhas a gerar). O estudo com o pó leva ~30 min a gravar (a tabela da largura).
