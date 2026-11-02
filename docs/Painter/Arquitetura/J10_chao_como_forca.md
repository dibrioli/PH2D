# J10 — o chão como força: a aguada empoça, contorna e passa por cima com volume (janela 10, 01/11)

Ordem: o §3.1 do [handoff da janela 9](HANDOFF_01_11_janela9.md) e a lei do dono (memória
`project_watery_paint_against_relief_owner_law_2026_11_01`). Base `1cd27137be`.

## 1. A medição primeiro (o mecanismo)

Sonda `chao_forca_tests::diag_o_chao_como_forca` (a cena de `escorre_contorna_tests`: Dabber seco, Wet Drip d0,9 por
cima, tela inclinada de fábrica), instrumento `solver::CHAO_MEDE` (somas por célula·passo sobre as células molhadas com
declive de chão, no passe novo, com o ganho a zero = o produto de antes):

| grandeza por célula·passo (Dabber, B Water) | valor |
|---|---|
| gravidade da tela `|g·f|` | 0,0049 |
| termo da ladeira ÷ 4 (um quadro em quatro), clampado | 0,053 |
| idem SEM o clamp | 0,168 |
| fracção de células·passo em que o `LevelClamp` morde | **73 %** |

A ladeira entra no NIVELAMENTO clampado a 0,2 num quadro em quatro, e depois do clamp o FREIO e a suavização da
velocidade ainda a comem; a gravidade entra na velocidade persistente a cada passo, depois do freio. A parede perde.

## 2. A lei (`wp/solver/chao_forca.rs`)

- **O chão** (`Grid::chao`, fotografado no início de cada traço em `begin_direct_stroke`): a altura do óleo assentado
  (`ladeira_do_oleo::altura`) × a fracção assente × `seco = (1 − film·m ÷ FILM_SECO)₊` (`FILM_SECO = 0,1`). A tinta de
  ÓLEO ainda fluida (o líquido dela é óleo) corre com o seu líquido; a água que chegou por cima (`m = 0`, J9) não lhe
  tira o chão.
- **A força** (depois do momento, na grade de fluxo, a cada passo): `v += G·(∇chão + w_h·∇h)·min(1, f ÷ FILM_CHEIO)` nas
  células com declive de chão, `G = 0,125` (o peso por passo do nivelamento de fábrica, `Leveling` 0,5 ÷ 4 quadros),
  `w_h = min(1, chão_máx da vizinhança ÷ FILM_CHEIO)`. É a superfície `h + chão` das águas rasas: a poça mais alta que
  a parede passa por cima.
- Com a força, a ladeira sai do NIVELAMENTO (`tira_do_nivelamento`). Com o Real Oil na mão (óleo sobre óleo) nada muda:
  a força não corre e a ladeira de antes (presa) fica.
- Sem `Grid::chao` (sem Real Oil na grade, primeiro traço, miniaturas) o passe não corre: a água ao bit (`flow_bit`,
  `advect_jacobi_`, `oleo_sobre_oleo`, `assenta`, `meio`, `advect_bit`, `fundida` — verdes). A rota fundida cede.
- Interruptores de medição: `CHAO_FORCA` (ganho; `< 0` desliga), `CHAO_SECO`, `CHAO_ROTA` (bit 0 mantém a ladeira no
  nivelamento, bit 1 corre também com o Real Oil na mão), `CHAO_MEDE`.

## 3. Medido (sonda J10, mesmo processo; Wet Drip Water salvo indicação)

| cena | antes: abaixo · acima · lado | depois: abaixo · acima · lado |
|---|---|---|
| Dabber, 1 traço | 344 982 · 279 582 · 0 | **0** · 441 965 · 0 |
| Dabber, 3 traços | 657 164 · 344 420 · 0 | 254 200 · 866 246 · 0 |
| Dabber, 6 traços | 863 653 · 350 593 · 0 | **1 082 565** · 992 395 · 4 177 |
| Dabber curto (x 100–140), 1 traço | 245 802 · 152 507 · 249 | 34 177 · 186 447 · **26 614** |
| Dabber curto, 6 traços | 520 358 · 239 880 · 109 865 | 430 551 · 304 321 · 273 973 |
| Dabber, Wet Drip **Oil** | 0 · 235 566 · 0 | 0 · 396 435 · 0 |
| Dabber diluído (d0,5 Oil, ainda fluido) | 6 636 667 · 138 478 | 6 729 472 · 120 965 (chão máx 0,02) |
| Wet Round Water (sem Real Oil) | 78 715 · 82 660 | igual ao bit |

Foto: [`fotos/j10/chao_forca_antes_depois.png`](fotos/j10/chao_forca_antes_depois.png) — o curto empoça em cima e pinga
pelos dois lados; o comprido segura com a orla macia; com 6 traços passa por cima e pinga por baixo; o Dabber diluído
igual. O rastro esverdeado PIXELADO que a água deixava ao atravessar o óleo (smoke do dono, fotos 1/3/5) some com a
travessia. O pontilhado rosado da tinta B nos buracos do relevo depois do transbordo existe também ANTES (6 traços):
fica aberto (§5).

**Réguas antigas verdes:** `a_queda_cresce_com_a_diluicao`, `a_agua_nao_sobe_a_ladeira_do_oleo`, os módulos
`agua_sobre_oleo_tests`, `queda_na_ladeira_tests`, `oleo_sem_real_oil_tests`.

**Testes novos** (`chao_forca_regua_tests`): `a_aguada_empoca_contra_o_oleo_seco` · `a_aguada_contorna_o_oleo_seco_pelos_lados`
· `com_volume_a_aguada_passa_por_cima_do_oleo_seco` · `a_tinta_de_oleo_ainda_fluida_nao_e_chao`. Mutação: M1 ganho 0 →
os dois primeiros sangram; M2 sem a água na superfície (`w_h = 0`) → o terceiro; M3 sem o `seco` → o quarto.

## 4. ⛔ Recusas MEDIDAS (no caminho)

| tentativa | o que deu |
|---|---|
| a força `−G·∇chão·f` (multiplicada pela água, como a gravidade) | a poça empilha: `film` 142 (1 traço) → 497 (6) numa fileira presa à borda, nunca passa por cima — quanto mais funda, mais empurrada para trás |
| o chão = a altura de AGORA (sem fotografia) | o Dabber diluído ainda molhado virava chão da própria água: os pingos dele sumiam e o relevo ficava riscado ([`fotos/j10/recusa_forca_vezes_a_agua.png`](fotos/j10/recusa_forca_vezes_a_agua.png), 3.ª linha) |
| `seco` pelo `film` só (sem o meio) | do 2.º traço em diante a borda seca já molhada pela aguada deixava de ser chão: a água entrava no óleo numa faixa escura e a linha da poça ficava DURA |
| a parte da água sem peso do relevo | o Dabber diluído (chão 0,02) mudava 25 % (o nivelamento sem clamp em células sem parede) |
| ganho 0,05 | 65 % mais água em cima do óleo (miolo 163 k contra 99 k) |
| o dique no destino (J9) | não refeito |

## 5. Aberto

- O pontilhado da tinta B nos buracos do relevo depois do transbordo (antes e depois): a composição / o pouso.
- A ONDA que desloca o Dabber (smoke do dono, fotos 2/4): nesta cena o Dabber SECO já não se mexe (a água não passa);
  conferir no app com o Dabber molhado.
