# Exame da família `guache` (J4–J5, catálogo H)

Receitas e portão: `crates/ph2d-tool-painter/tests/it/familias/guache.rs` (`WET` = a Gouache Flat; Round, Wash e Dry em
`EM_ESTUDO`, sem ficheiro). Pranchas `pranchas/guache/` (+ `cegas/`, `chaves/`); réguas `exames/guache/`; a régua das
fotos `exames/guache_foto.py`. Oráculo: Rebelle `rebelle_bateria/WATERCOLOR/gouache_gouache_*`.
⚠️ **O detalhe da J4 (as rondas g1–g7, as recusas medidas) está em `96868e656e`**
(`git show 96868e656e:docs/Painter/Pinceis/exames/guache.md`).

**Estado (J5, 22/10):** aprovados nenhum · **Gouache Flat** curado não aprovado (fica como estava — a cura NÃO entra,
nem se funde; abaixo) · **Gouache Round** reprovado (em estudo) · **Gouache Wash** curado não aprovado (2 rondas) ·
Gouache Dry reprovado (em estudo; motor: cerdas que crescem com o diâmetro).

## O que as fotos dizem do guache do Rebelle
- Seca plano (A = B ao byte em quase todas as cenas); sem grão do papel e sem a orla da aguarela.
- Sobre o vermelho SECO é véu (o azul escurece: 91,111,165); sobre o vermelho MOLHADO **cobre**, numa mancha redonda
  e clara (1,25 × a largura, escuro 1,00; cor [205,203,219] ≈ o azul no papel) e não empurra o vermelho (anel 1,02).
- ⚠️ Fotos más: `round_2_sobre_{A,B}` = `round_2_tamanhos_B` (md5); `liner_2_gestos_A` = `liner_gestos_B`. Não refeitas:
  nenhum pincel desta ronda precisou delas (o `Gouache Round` tem `gouache_gouache_round_sobre_*`).

## H-2 Gouache Flat — decisão J5: FICA como estava (nem a cura, nem a fusão)
- A cura da J4 (`felt` 0,1 + pigmento 100 → 60) separava-a do Wet Flat (vizinho 0,763 → 0,899; o revisor g6 já não os
  emparelhava). **Aplicada e regerada nesta janela, parte TRÊS leis da J2/J3** (medido por nome): a pressão a **0,228**
  da origem (barra 0,18, `a_pressao_de_cada_preset_e_a_da_sua_origem`); o recto de 5 px cobre **0,39** do de 40 (o
  Rebelle 1,13, `o_traco_fino_pesa_por_area_o_que_o_largo_pesa` — o feltro apaga o fino); a `RampaCobertura` 0,648 fora
  do vale 0,43 (`cada_preset_parece_a_sua_midia`). ⇒ revertida (preset e receita ao bit).
- **Fundir no Wet Flat: não.** A matriz põe-nos a **0,763** (≥ 0,596: distintos); o emparelhamento do g1 diz que a
  nossa não é a do Rebelle, não que duplica o Wet Flat. Régua 404/454; curada não aprovada (bordas rectas e lisas
  contra as rasgadas dele, a espiral fechada, a cor menos carregada — g6).

## H-1 Gouache Round — REPROVADO (em estudo)
- J4: g2, g3 tomaram o nosso pelo original; g5 achou o Rebelle (pontas em cápsula, degradê de centro escuro, o vermelho
  empurrado dos lados — 3 revisores). Duas rondas já feitas.
- **J5, a lei molhado × molhado NÃO serve a guache** (sonda `familias::aguarela::molhado::sonda_o_molhado`): o nosso
  cruzamento já é redondo (1,27) — o defeito é o vermelho EMPURRADO (anel 1,14) e o azul que não COBRE. A parede
  (`levelingOwnStroke` 1) tira o empurrão (anel 1,02) mas o cruzamento fica vermelho ([198,172,190]) e a régua cai
  **409 → 376/454** (`exames/guache/gouache_round_j5_regua.txt`); a lei por cima abre tampas vermelhas escuras (anel
  1,31–1,45). Nenhuma das duas é um recurso feito para ela ⇒ sem 3.ª ronda nem revisor (REGRAS_DE_CUSTO 1).
- O que falta (motor): **o azul que cobre o vermelho molhado** (a opacidade do pouso sobre tinta molhada de outra cor).

## H-4 Gouache Wash — CURADO, NÃO APROVADO (2.ª ronda J5)
- Receita: ponta quadrada (quina 1) de lados nítidos, pigmento 16, orla 0, a pressão só na carga, `retention` 0,95,
  tecto 56 px. Régua 354/454; vizinho Wet Wash 0,783.
- Ronda 2 (cura: `pressureSize` 0,5 para a rampa em cunha + `glaze` 0,2 para o azul × azul que escurece; estudo): régua
  354 → 351/454. Revisor m5 (sonnet, `cegas/gouache_wash_j5.png`): **o NOSSO tomado pelo original em 6/6** (média), o
  Rebelle como a imitação, o vizinho claramente outro — como o g4 da J4. Do nosso lado lista as riscas de cerdas e a
  mancha irregular, as pontas rasgadas, a espiral mosqueada; do Rebelle as faixas verticais de carimbo a 200 px, as
  pontas de corte recto esfumado, o cruzamento pálido e quadrado. Duas rondas ⇒ fica; a cura NÃO entra na receita (a
  régua desce). A diferença que falta: as riscas e as pontas rasgadas (o Rebelle liso) — `felt` 0,1 com quina 1 dá a
  espiral quadrada (recusado na J4).

## Motor J6 (28/10) — a guache COBRE a tinta molhada (`Knob::WetCover`, lacuna 7)

Régua (`resto`, cena `sobre` A, cruzamentos x 250/500 na faixa molhada): `|cruzamento − azul no papel| ÷ |vermelho longe −
azul no papel|` — 0 = o azul cobre, ~1 = o vermelho fica. Rebelle: **Flat 0,11 · Liner 0,15 · Round 0,33 · Filbert 0,37 ·
Dry 0,64 · Wash 0,95** (a diluída NÃO cobre: vela, o cruzamento mais escuro que as duas).

**A lei** (aditiva, 0 = ao bit): a cada pouso, `q = wetCover × outra_tinta`; o suspenso perde `q × peso` (`peso` = a parte
da tinta nova na mistura de opacidade) e a cor nova pesa `1 − (1 − peso)(1 − q)` (`trail/transfer.rs` `cobre_a_outra`).
Nossa Flat 0,36 → **0,10** (1,0) / 0,13 (0,5); Round 0,61 → 0,17 (0,15) / 0,07 (0,5); o anel não muda (Flat 1,09–1,11).
⛔ Recusas medidas: só a massa (a cor mista já não é «outra»: cruzamento lilás claro, 0,36 → **0,50**); a massa inteira à
1.ª passagem (abre o papel: 0,77); só a cor (a massa vermelha passa a azul: o Dry escurece). Portões
`o_azul_da_guache_cobre_o_vermelho_molhado` (it) + `cobre_tests` (lib); mutações (cor desligada, massa fica, mistura sem
`q`, lei desligada) todas VERMELHAS. A cena `a_lei_da_tinta` dá os mesmos hashes com e sem este motor.

- **Gouache Wash** — o `wetCover` vai ao contrário (0,73 → 0,31); a cura é o VÉU: `glaze` 0,3 (cruzamento [207,192,207]
  contra o Rebelle [208,195,210]). Régua 355 → **363/456** (`guache/gouache_wash_J6_regua.txt`); r2 `felt` 0,05: 361 —
  fica a r1. Vizinho Wet Wash **0,740**. Prancha `pranchas/J6_Gouache_Wash{,_cega}.jpg` + chave. Fica à vista: as riscas
  das cerdas (grão 0,076 contra 0,018) e as pontas rasgadas; as faixas de carimbo verticais dele a 200 px.
- **Gouache Dry** — a lei não serve (0,89 → 0,75, o cruzamento escurece; o dele é claro [208,187,202]); sem receita nova
  (as 2 rondas foram da J4). Régua 383/454, vizinho Wet Dry Brush **1,473**. Prancha `pranchas/J6_Gouache_Dry{,_cega}.jpg`.
  Fica: a 200 px o Rebelle são faixas sólidas largas, o nosso uma nuvem de pontilhado (o tapete das cerdas — motor 2).
- **Gouache Round** — NÃO refeito: a lei dá o resto (0,05–0,15 já o passa de 0,33), mas o que a reprovou fica — o vermelho
  empurrado (anel 1,20 contra 1,11) e as pontas em cápsula.
- **Gouache Flat** (fábrica, não aprovada): `wetCover` 0,5–1 casa-a com o Rebelle (0,10–0,13 contra 0,11) — candidata a cura
  quando o coordenador a reabrir (regerar o preset e os portões dela).

## Portão
`os_reprovados_do_guache_nao_estao_na_fabrica` (os três de `EM_ESTUDO` sem ficheiro; mutação J4: a Round renomeada
«Gouache Flat» → VERMELHO). A Flat continua presa pelos de antes (`MIDIAS` gouacheflat, `a_lei_da_tinta`), verdes.

## Para a fila
1. Motor: o azul que COBRE o vermelho molhado (Gouache Round; o «sobre» de toda a guache).
2. Motor: o tapete das cerdas que escala com o diâmetro (Gouache Dry a 200 px).
3. Gouache Wash: riscas e pontas rasgadas contra o liso do Rebelle (sem valor que o desfaça; medido na J4).
4. Fotos: refazer `gouache_gouache_round_2_sobre` e `gouache_gouache_liner_2_gestos_A` (sem pincel que as peça agora).
