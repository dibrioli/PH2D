# Exame da família `guache` (J4, catálogo H)

Receitas e portão: `crates/ph2d-tool-painter/tests/it/familias/guache.rs` (`WET` = a Gouache Flat de antes; os três novos em
`EM_ESTUDO`, sem ficheiro). Pranchas `pranchas/guache/` (cegas e chaves em `cegas/`, `chaves/`); réguas `exames/guache/`;
a régua das fotos da família `exames/guache_foto.py` (opacidade, orla, grão, falhas, borda). Oráculo: Rebelle
`rebelle_bateria/WATERCOLOR/gouache_gouache_*`. Bancada `PH2D_EXAME_DIR=/tmp/ph2d_exame_guache`, variantes num processo;
load 13–90 (o relógio não vale). Sob as REGRAS DE CUSTO a partir da 2.ª metade: ≤ 2 rondas, revisor `sonnet` curto.

**Aprovados: nenhum. Pré-existente: Gouache Flat fica como estava (curado não aprovado). Em estudo: Gouache Round,
Gouache Dry, Gouache Wash. Fundidos: nenhum (a fusão Flat → Wet Flat fica proposta, abaixo). Motor: nada entra.**

## O que as fotos dizem do guache do Rebelle
- **Seca plano**: A = B ao byte em quase todas as cenas (`ab.py`: ≤ 0,1 % dos píxeis mudam) — nem orla nova nem clarear.
- **Sem grão do papel e sem a orla da aguarela**: orla 0,91–1,03 contra 1,32 do `Wash Round` da aguarela; grão 0,003–0,04
  contra 0,11 do `Round` da aguarela.
- **Sobre o vermelho SECO é véu** (o azul escurece: 91,111,165), **sobre o vermelho MOLHADO cobre** e não o empurra
  (109,145,194 ≈ o azul no papel 113,161,208). O nosso mistura e empurra o vermelho em manchas escuras dos lados do cruzamento.
- ⚠️ Fotos más: `round_2_sobre_{A,B}` = `round_2_tamanhos_B` (md5); `liner_2_gestos_A` = `liner_gestos_B`.

## H-2 Gouache Flat — pré-existente, CURADO NÃO APROVADO (fica a receita de antes)
- Régua 404/454; vizinho Wet Flat 0,763. **Ronda 1** (revisor g1, opus, `cegas/gouache_flat_r1.png`): emparelhou o
  NOSSO com o **Wet Flat** em 6/6 linhas (o par com confiança alta) e deixou o Rebelle como «o terceiro»: a borda dele
  serrilhada e nítida, cheio chapado sem estrias, pontas arredondadas — o nosso é o Wet Flat mais escuro.
- **Ronda 2** (cura: `felt` 0,1 + pigmento 100 → 60): o miolo chapado e a borda nítida; vizinho Wet Flat 0,899, régua
  396/454. Revisor g6 (sonnet, `cegas/gouache_flat.png`): o Wet Flat passa a «claramente outro» em 6/6, mas achou o
  Rebelle em 4/6 (média): as nossas bordas rectas e lisas contra as rasgadas dele, a espiral fechada (ele tem furos), a cor
  menos carregada. Duas rondas ⇒ fica. A cura NÃO foi aplicada (mexe no hash de `a_lei_da_tinta`, nas `MIDIAS` e em
  `as_formas`/`os_fios`): **decisão do coordenador** — aplicar a cura (separa-a do Wet Flat) ou fundi-la no Wet Flat.

## H-1 Gouache Round — EM ESTUDO
- Receita: a ponta redonda dura (0,9) que nasce da Flat, pigmento 32, a pressão no tamanho (0,8) e na carga (curva 1),
  `retention` 0,97 (seca depressa: as manchas do vermelho empurrado diminuem). Régua 409/454 (escala 0,974); vizinho
  fora da família Wet Round 0,879 (Soft/Hard Round mais longe); na família Gouache Wash 0,81.
- g2 (opus): o par certo, nosso tomado pelo original (baixa); no «sobre» o Rebelle é o terceiro — o nosso e a Wash
  empurram o vermelho molhado. g3 (opus, `pranchas/guache/gouache_round.png`): **o NOSSO tomado pelo original em 6/6
  (média)**; do nosso lado: riscas de cerdas e pontas rasgadas (repetido do g2) e o vermelho empurrado dos lados (repetido).
- Ronda com `felt` 0,1 (sem riscas): g5 (`gouache_round_r3_feltro.png`) achou o Rebelle 6/6 (média) — pontas em cápsula,
  degradê de centro escuro, o vermelho empurrado (3.º revisor). ⇒ reprovado pela regra das rondas.
- O que falta: **o motor molhado × molhado** (o azul cobre o vermelho molhado sem o empurrar — dono `oleo_acrilica`);
  aqui há oráculo que o mostra: `gouache_gouache_{round,flat,wash}_sobre_A`. Nenhum valor o desfaz (`projection` 0,
  `maxVelocity`, `brake`, `capillary` 0, `edgeDarkening` 0, `blendForce`/`pickup`/`drag`/`wetLift`: as manchas ficam;
  `waterPerDab` ≤ 0,2 tira-as mas granula o traço, 226–362/456).

## H-3 Gouache Dry — EM ESTUDO
- Receita: o corpo do Wet Dry Brush (água 0,2, cerdas 1,5/1500, orla 56) com pigmento 70 e a pressão POR FIO
  (`bristlePressure` 2, `bristleNoiseLength` 60, `bristleLaneWidth` 3, `bristleLift` 1): os fios somem e voltam em troços.
  Régua 281 (o Wet Dry Brush) → 382/452; vizinho Wet Dry Brush 1,473 (distinto).
- g7 (sonnet, `pranchas/guache/gouache_dry.png`): achou o Rebelle 6/6 (**alta**): o nosso translúcido com halo e sem
  falhas; a 200 px o Rebelle são FAIXAS sólidas largas sobrepostas, de bordas duras — o nosso uma nuvem de pontilhado fino;
  o 5 px dele quase some.
- Ronda 2 medida (sem revisor: a diferença principal ficou): uma lei aditiva «a largura do fio ≥ fracção do diâmetro»
  (`bristleLaneFrac`, desligada por omissão) — 389/454 e a mesma nuvem a 200 px (0,06 = 0,08 ao byte da régua): o
  pontilhado é o TAPETE das cerdas, não a largura do fio. Revertida (não comitada). `thinStrokeFade` 1 apaga o 5 px.
- O que falta (motor): **cerdas que crescem com o diâmetro** (o tapete 128² do Wet tem o grão em px; o Rebelle escala-o).

## H-4 Gouache Wash — EM ESTUDO (adiada pela ordem de custo)
- Receita: ponta quadrada (quina 1) de lados nítidos cujos dabs somam (as pontas esbatem num diâmetro), pigmento 16,
  orla 0, a pressão só na carga, `retention` 0,95, tecto 56 px (o recto de 200 px do Rebelle). Régua 354/454; vizinho Wet
  Round 0,882 (o Wet Wash 0,972).
- g4 (opus, `pranchas/guache/gouache_wash.png`): **o NOSSO tomado pelo original em 6/6 (média)**; do nosso lado as riscas
  e as pontas rasgadas; do Rebelle o carimbo vertical a 200 px, a rampa em cunha (a pressão no tamanho), o azul×azul que
  escurece. Um só revisor ⇒ não aprovado; a 2.ª ronda não se fez (regra de custo: só se os três P1 fechassem).
- ⛔ Recusado: o filme (`strokeFilm`) — a ponta quadrada vira carimbo de quinas vivas (316/456); `felt` 0,1 com quina 1
  dá a espiral quadrada.

## Portão
`os_reprovados_do_guache_nao_estao_na_fabrica` (os três de `EM_ESTUDO` sem ficheiro). Mutação: a Round renomeada para
«Gouache Flat» → VERMELHO («reprovados na fábrica: ["Gouache Flat"]»); reposta → verde. A Flat continua presa pelos de antes
(`MIDIAS` gouacheflat, `a_lei_da_tinta`, os da fábrica) — verdes por nome. Custo: não medido (nenhum aprovado).

## Para a fila
1. Motor molhado × molhado (oleo_acrilica): o guache cobre o vermelho molhado sem o empurrar — oráculo nas fotos acima.
2. Motor: o tapete das cerdas que escala com o diâmetro (Gouache Dry a 200 px).
3. Decisão: Gouache Flat — aplicar a cura (felt 0,1, pigmento 60: deixa de ser o Wet Flat) ou fundir no Wet Flat.
4. Gouache Wash: a 2.ª ronda (perto: o revisor tomou o nosso pelo original 6/6).
5. Fotos: refazer `gouache_gouache_round_2_sobre` e `gouache_gouache_liner_2_gestos_A`.
