# EXAME da família `pastel_carvao` (catálogo F: F-1..F-10) — estado depois da J5

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/pastel_carvao.rs` (`WET`, `EM_ESTUDO`, `MIDIAS`, `portoes`).
Pranchas finais: `pranchas/pastel_carvao/` (cegas e chaves em `cegas/`, `chaves/`; rondas antigas em `r1/`, `r2/`, `r4/`,
`r5/`). Régua: `exames/pastel_carvao/<slug>_regua.txt`. **O detalhe da J4 (as rondas r1–r5, os motores `PressureGrain`,
`DustDrop` e o pó solto no Wet, as medições por pincel) está em `82b685a5e9`.** SHAs J4 `3ea7f0ff32` · `778299f8f9` ·
`7928edffb8` · `e0ea982caf` · `9965b60b59` · `3d79d08cd6`; J5 `2e1212e7ec` (Square Hard) · `c6c99afab7` (Pastel Side).

**Aprovado: Pastel Side (J5).** Curados, não aprovados: Charcoal, Soft Pastel, Square Hard, Oil Pastel, Wax Scrape.
Reprovados (`EM_ESTUDO`, sem ficheiro): Compressed Charcoal, Charcoal Powder, Wax Crayon. Fundido: Charcoal Pencil → Soft
Pastel. Fábrica da família: 6 (Pastel Side entra).

## J5 — Pastel Side ✅ APROVADO

- **Receita** (Wet): o Soft Pastel com a lâmina fixa 0,47 a 0°, pontas rectas (`dab_quina` 1), tecto 50, `pressureSize`
  0,2 + `pressureGrain` 0,5, curva da opacidade 1,2, vetas `ViewPlane` 16 — e na J5 o **pó solto medido** no Rebelle
  `Flat Soft` (`PO_DO_LADO`: `po` 2,5 · alcance 4 · 1,5 px · opacidade 0,12 · lado 0,35; a variante ps2 v0 da J4) e a
  **carga 150** (o tom: o nosso era mais claro).
- **Oráculo**: Rebelle `Pastel/Pastel Dry/Flat Soft` (`fixtures/rebelle_bateria/PASTEL/pastel_dry_flat_soft`).
- **Régua** 350/454 (J4 r3: 366). O tom dos deitados passa (cobertura e cor dos rectos ✓); perde nos de PÉ (o lado largo):
  cobertura 0,78 contra 0,69 e opacidade 0,84 contra 0,77 nos verticais da `sobre` (a carga 150 enche o lado largo, que
  no Rebelle é translúcido), e a rugosidade/textura da borda (o pó conta como borda).
- **Vizinho** Soft Pastel 1,093 (≥ 0,596; o Compressed Charcoal, que a J4 juntava a ele, não está na fábrica).
- **Razão de uso**: fundos e massas em pastel — a faixa larga do bastão deitado, translúcida e riscada, com o pó à volta.
- **Revisor cego r4** (sonnet, novo): L1–L2 (gestos) ORIGINAL = **nosso** (média), imitação = Rebelle — «a imitação tem
  borda em degraus e padrão de grade»; L3–L4 (tamanhos) ORIGINAL = **nosso** (média) — «a imitação, barra mais compacta e
  menos poeira»; L5–L6 (sobre) acha o Rebelle (**baixa**) — «a imitação, azul mais opaco nos cruzamentos». Terceiro
  (Soft Pastel) claramente outro em todas.
- **Cruzamento**: 4/6 linhas o revisor toma o nosso por original com confiança média e as diferenças que lista são do
  lado do Rebelle (os degraus do carimbo dele); a única do nosso lado (o azul mais opaco sobre o vermelho) é de confiança
  baixa — pela decisão 12 não é diferença visível. A régua confirma-a nos verticais (acima): é o resíduo medido.
- **Custo** p95 5 · 40 · 200: 11,2 · 3,2 · 2,3 ms; RSS 74 · 89 · 92 MiB (load 9–11: indicativo).
- **Portão** `o_pastel_side_e_o_lado_do_bastao_com_po`: lâmina 1,80 (Rebelle 1,89; 1,6–2,3) e pó 59,4 grãos/100 px
  (Rebelle 60; ≥ 30). **Mutação** por dentro: sem a lâmina (`dab_flatten` 0) 1,00 e sem o pó (`po` 0) 0,0 — as duas
  SANGRAM. Verdes por nome: `a_largura_e_a_prometida`, `cada_preset_de_fabrica_volta_ao_byte`, `a_miniatura…`,
  `cada_preset_de_fabrica_e_a_sua_receita`, `a_contagem_por_meio`, `the_factory_presets_are_counted`,
  `os_reprovados_nao_estao_na_fabrica`, `a_regra_do_spacing_da_fabrica`, `a_pressao_de_cada_preset_e_a_da_sua_origem`.

## J5 — Square Hard: curado, NÃO aprovado (2.ª ronda fechada)

- **Decisão 16** (a foto nova do Rebelle vence número antigo): já aplicada na J4 2.ª ronda (`3d79d08cd6`: carga 110, papel
  0,8; o `MIDIAS` perdeu a Textura e o Cv da foto antiga e ganhou `o_square_hard_tapa_o_vermelho_como_a_foto`). Conferido
  na J5 contra a foto da bateria: o que o portão ainda prende **não é contradito** — Cobertura 0,705 (foto nova 0,709),
  Largura 31 (32), o vermelho por baixo do azul (0,16 · nosso 0,19), a largura que não muda com a pressão (Δ 0,009).
  `cada_preset_parece_a_sua_midia` verde. Nada a regravar.
- **Régua** 412/454 (J4 r3: 270). Vizinho Wax Scrape 0,864.
- **Revisor cego r4**: acha o Rebelle em 5/6 linhas (média nos tamanhos, média-baixa nos gestos; na `sobre A` toma o nosso,
  baixa). Diferenças: (1) o nosso **granulado e salpicado, com furos brancos** — o Rebelle liso e denso, mais a 200 px;
  (2) **as pontas do recto de 200 esmaecem em degradê** — o Rebelle corta a direito; a rampa mais transparente no início;
  (3) a espiral perde a linha branca entre voltas; bordas mais ásperas.
- **Veredito**: curado-não-aprovado — já teve as duas rondas (J4 r1/r3 e a 2.ª ronda); as diferenças (1) e (2) são de
  confiança média. (2) é o item 3 da fila de motor (a tampa recta); (1) é o grão Noise 100 + manchas, que a J4 escolheu
  contra a Textura da foto antiga (recusa abaixo).

## Os curados que ficam (J4, sem mudança na J5)

| pincel | identidade (receita) | régua | vizinho (d) | o que o revisor ainda vê |
|---|---|---|---|---|
| Charcoal | tecto 76, mármore (Noise `Random` 8), pó solto (lado 0,43) | 424/454 | Graphite 2H 0,865 | manchas de tom e arranhões brancos de fibra; mais escuro; pontas redondas |
| Soft Pastel | papel 0,6, carga 85, `pressureGrain` 0,5, pó solto (0,37) | 414/454 | Hard Round 0,782 | grumos com furos; pontas RECTAS no Rebelle; 5 px tracejado (cv 0,28 × 0,02) |
| Oil Pastel | estrias: Shape Noise em Flow (0,02 × 20), papel 0,6, carga 160 | 434/454 | Graphite 2B 0,777 | a 200 px a grelha do carimbo e borda em festões |
| Wax Scrape | traços da cera (Flow 8 × 50), dureza 0, carga 200 | 354/454 | Soft Pastel 0,903 | fitas de borda dura com falhas; o azul tapa o vermelho |

## Reprovados (`EM_ESTUDO`)

- **Compressed Charcoal** (Rebelle `Charcoal Flat`): lâmina 2,33 (Rebelle 2,31); os revisores juntavam-no ao Pastel Side.
  Falta: riscas paralelas limpas sem grão por cima (ponta de imagem riscada).
- **Charcoal Powder** (Rebelle `Powder`): a 200 px o Rebelle é bloco com nódoas e furos de 10–20 px; o nosso halo gaussiano.
- **Wax Crayon** (Rebelle `Wax Pencil`): o revisor junta-o ao Graphite 8B; o Rebelle tem carimbos rectangulares e festões.

## Portões da família (verdes por nome)

`cada_pastel_e_carvao_se_reconhece_pela_foto` + `sem_o_que_faz_cada_identidade_o_portao_reprova` (Oil Pastel, Wax Scrape,
Charcoal, Square Hard; controlo Soft Pastel) · `o_pastel_side_e_o_lado_do_bastao_com_po` (J5) ·
`a_pressao_leve_desfaz_o_traco_em_grao` · `o_po_solto_cai_a_volta_do_traco_no_wet` · `o_square_hard_tapa_o_vermelho_como_a_foto`
· `a_gota_do_po_e_do_preset` · `os_reprovados_nao_estao_na_fabrica`.

## ⛔ Recusas MEDIDAS
| recusa | medido |
|---|---|
| estrias pelos fios do Wet (`bristleStrength`) com o grão desligado | só salpico, contraste 0,65–0,83, nenhuma estria |
| Shape em Flow nas lâminas | o Flow roda o dab com o traço: a lâmina fixa vira 1,0 |
| `pressureGrain` 0,5 no Square Hard | a largura da rampa leve cai a 0, a lei da pressão 0,24 (> 0,18) |
| Square Hard com o grão Noise 160 | Textura da mídia 14 contra 21 do Rebelle, «ruído fino» para o revisor |
| a gota do pó grossa (8) no Powder | grão 4 × 4 px mas sem as nódoas; régua 246 → 229 |
| `dab_quina` 0,5–0,7 no Soft Pastel (pontas rectas) | a mídia lê Orla 3 × 6: a quina endurece a borda lateral |
| estrias em Flow + quina no Charcoal (r5) | régua 424 → 396, o revisor vê o mesmo |
| o pó dentro das lanes do Wet | nódoas no miolo: 92 → 109 px |

## Fila (com a medição)

- **Tampa RECTA de um dab redondo** (Square Hard, Soft Pastel, Charcoal, Wax Scrape — r1–r5 e a r4 da J5 repetem): cortar
  a cápsula a direito no início e no fim do traço, não o carimbo (motor; a `dab_quina` endurece a borda lateral).
- **Square Hard liso e denso** (r4 J5): o grão salpicado com furos brancos; uma 3.ª ronda só com recurso de motor novo.
- **O lado largo translúcido do Pastel Side** (resíduo da régua: de pé cobertura 0,78 × 0,69): a carga é uma só para os
  dois lados da lâmina; uma opacidade pela orientação do dab seria motor.
- **O fino tracejado** (Soft Pastel a 5 px, cv 0,28 × 0,02): uma lei de dente pelo diâmetro (motor).
- A ponta de imagem (Compressed Charcoal, Wax Scrape, Oil Pastel, Powder) por medir com `GrainLaw::area`.
