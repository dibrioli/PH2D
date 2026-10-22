# EXAME da família `pastel_carvao` (J4, catálogo F: F-1..F-10)

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/pastel_carvao.rs` (`WET`, `EM_ESTUDO`, `MIDIAS`, `portoes`).
Pranchas (r3, as finais): `pranchas/pastel_carvao/` (cegas e chaves em `cegas/`, `chaves/`; as rondas que os revisores viram
em `r1/`, `r2/`). Régua: `exames/pastel_carvao/<slug>_regua.txt`. SHAs `3ea7f0ff32` · `778299f8f9` · `7928edffb8`.
Método: variantes no mesmo processo (`PH2D_EXAME_VARIANTES`) medidas contra a foto do Rebelle por scripts da família
(estrias = ½-autocorrelação ao longo × através e contraste do recto de 200 px; lâmina = de pé ÷ deitado; rampa por
décimo; a lei da pressão de `leis_sistemicas`), depois o exame inteiro e a revisão cega (05 §2 tal e qual, um revisor NOVO
por ronda).

**Aprovados: nenhum.** Curados, não aprovados (decisão 3): Charcoal, Soft Pastel, Square Hard, Oil Pastel, Wax Scrape.
Reprovados (`EM_ESTUDO`, sem ficheiro): Compressed Charcoal, Charcoal Powder, Pastel Side, Wax Crayon. Fundido: Charcoal
Pencil. Fábrica: 34 (sem novos; os cinco ficam).

## Motor (aditivo, 0/omissão = a lei de antes ao bit)

1. **`Knob::PressureGrain`** (`ph2d-wet-paint` `trail/deposit.rs`): a parte `k` da opacidade da pressão entra ANTES do
   portão do papel (`opacidade^k` no carimbo, `^(1-k)` depois) — com pouca tinta só os picos do dente vencem: os pontos
   soltos do pastel do Rebelle a pressão leve. Portão red-first `a_pressao_leve_desfaz_o_traco_em_grao` (Square Hard, o grão
   da rampa leve p90/média: 1,53 = 1,53 antes do preset; 1,94 contra 1,71 sem a lei depois); mutação (o ramo desligado)
   SANGRA. Sem portão do papel (`paperGate` 0, o Charcoal) não morde: medido 1,72 · 1,72.
2. **`Knob::DustDrop`** — a fila «a gota do pó por preset»: o lado da gota do `dust` em células (2 = a constante de antes).
   Portão `a_gota_do_po_e_do_preset` (o pó do Powder a 200 px: gota 2 → 2 × 2 px, 8 → 4 × 4); mutação (gota fixa) SANGRA.
   Nenhum preset o usa: no Powder engrossa o grão mas não faz as nódoas do Rebelle (régua 246 → 229).
Verdes: `fingerprint`, `a_lei_do_po`, `deposit_rows`, `a_opacidade_do_dab` (wet-paint), `tuning` (lib).

## Os cinco que ficam — curados, NÃO aprovados

| pincel | o que mudou (receita) | régua r3 | vizinho (d) · origem | custo p95 5·40·200 ms, RSS (load) |
|---|---|---|---|---|
| Charcoal | tecto 76 (pintava 151 a 200), jitter 0,15, miolo marmoreado (Noise `Random` 8), pressão → opacidade 0,6 | 418/454 | Graphite 2H 0,804 · ✓ 1.º | 1,15 · 5,28 · 4,01 · 86 MiB (22–24) |
| Soft Pastel | papel 0,6, carga 85, `pressureGrain` 0,5 | 416/454 | Hard Round 0,777 · 7.º | 13,8 · 13,7 · 15,7 · 93 MiB (20) |
| Square Hard | `pressureSize` 0 (rectângulo: a rampa 26→33 px), `pressureGrain` 0,25, dureza 0,7, grão Noise 100 + manchas | 270/454 | Airbrush 1,048 · 6.º | 14,0 · 15,9 · 15,5 · 92 MiB (20) |
| Oil Pastel | as ESTRIAS: Shape Noise em **Flow** (0,02 × 20), papel 0,6, carga 160 | 434/454 | Graphite 2B 0,777 · ✓ 1.º 0,577 | 9,9 · 13,2 · 17,0 · 103 MiB (19) |
| Wax Scrape | os TRAÇOS da cera (Flow 8 × 50, viés 0,78), dureza 0, carga 200 — saiu do Soft Pastel (0,484 → 0,903) | 354/454 | Soft Pastel 0,903 · ✓ 1.º | 11,7 · 12,6 · 14,7 · 102 MiB (19) |

⚠️ Custo com load 18–24 (as outras famílias): indicativo, a J5 re-mede calmo; o Oil Pastel a 200 passou de 16,7 só nessa leitura.
Leis verdes: `cada_preset_parece_a_sua_midia`, `a_pressao_de_cada_preset_e_a_da_sua_origem` (Square Hard 0,140; Charcoal
0,166), `sem_glaze…` (os fnv novos do Charcoal, Soft Pastel e Oil Pastel), miniatura, byte, contagem.

**O que os revisores ainda vêem (todos acharam o Rebelle; a lista é o que falta):**
- **Charcoal** (r1 alta, r3 média): no Rebelle riscos brancos de fibra e manchas «em vírgula», pó solto fora da borda,
  pontas quase rectas; a rampa nasce em agulha lisa (a nossa em fio pontilhado). As riscas regulares da 1.ª cura
  (`ViewPlane`) foram lidas como «cerdas» — trocadas pelo mármore.
- **Soft Pastel** (r1 baixa, r2 e r3 média — repetido): o PÓ SOLTO à volta (até 50 px a 200), tampas rectas contra as nossas
  ovais, 5 px pontilhado no Rebelle. Sobre a pressão leve os revisores contradizem-se (névoa × agulha).
- **Square Hard** (r1 média, r3 alta): o Rebelle mais escuro e tapa o vermelho (o nosso arroxeia), pontas cortadas a
  direito (as nossas esmaecem no fim do recto de 200), fibras curtas com direcção (o nosso salpicos). ⚠️ Conflito medido:
  a carga 90 tapava como o Rebelle mas a mídia (`squarehard`) lia Cruzamento 0,82 contra 0,58 — ficou 75.
- **Oil Pastel** (r1 média): a 200 px o Rebelle tem a grelha do carimbo e a borda dura em festões; o nosso estrias finas
  sobre grão. Baixar o papel abaixo de 0,6 desfaz as estrias (contraste 0,96: medido).
- **Wax Scrape** (r3 média): a 200 px fitas de cera de borda dura com falhas grandes; a pressão muda a opacidade a largura
  quase cheia; o azul tapa o vermelho.

## Reprovados (`EM_ESTUDO`)

- **F-2 Compressed Charcoal** (Rebelle `Charcoal Flat`): a lâmina fixa (0,55 a 0°, de pé ÷ deitado 2,33; Rebelle 2,31),
  tecto 40 (o do LADO FINO: o slider promete a largura deitada), carga 80, vetas `ViewPlane` 24. r2 e r3: os dois revisores
  juntam-no ao **Pastel Side** e dão o Rebelle como «outro» (riscas paralelas limpas, translúcido, fiapos). d 0,729.
  Custo 2,19 · 3,53 · 2,94. Falta: riscas paralelas limpas sem o grão por cima (uma ponta de imagem riscada).
- **F-4 Charcoal Powder** (Rebelle `Powder`): `dust` 0,7, carga 11, pressão a abrir a meio (lei 0,132). r2 e r3 (média,
  a mesma): a 200 px o Rebelle é um bloco de borda definida com nódoas e furos de 10–20 px, o nosso um halo gaussiano; a
  5 px o nosso pontilha. d 1,02 (Scratch). Custo 1,97 · 5,60 · 9,32 · 106 MiB.
- **F-7 Pastel Side** (Rebelle `Flat Soft`): lâmina 0,47 (1,93; Rebelle 1,89), pontas rectas, tecto 50, `pressureSize` 0,2 +
  `pressureGrain` 0,5, vetas. r2 → r3: a pressão já muda a densidade e não a largura; ficam o PÓ SOLTO (a dispersão de
  partículas é de `texturas`), as faixas do carimbo do Rebelle a 200 px e o tom (o nosso mais claro). d 0,729.
- **F-10 Wax Crayon** (Rebelle `Wax Pencil`): orla do dual 0,8 × 2, Shape `Random` 6 a 0,8, pressão 3,5 · 1, tecto 140. A
  matriz: Soft Pastel 0,528 → Graphite 8B 0,658 com a Shape (variantes medidas: 0,528 · 0,633 · 0,604 · 0,699). r3: o revisor
  toma o NOSSO por original e o 8B por imitação; o Rebelle é «outro» (carimbos rectangulares, festões). Reprovado.

## Fundido
- **F-3 Charcoal Pencil → Soft Pastel** (d 0,616, à beira do limiar; a prancha mostra a mesma linha fina granulosa — o
  Liner do Rebelle é o carvão a tamanho pequeno: pinta 7 px a 40 pedidos, decisão 13).

## Portões da família (verdes por nome)
`cada_pastel_e_carvao_se_reconhece_pela_foto` (Oil Pastel: estrias ≥ 30 px ao longo e ≤ 8 através, contraste 0,12–0,35 ·
Wax Scrape: traços 3–40 × ≤ 4 px, contraste ≥ 0,22 · Charcoal: tecto 75 ± 8 · Square Hard: largura ± 10 % do 6.º décimo ao fim e
escurece · controlo Soft Pastel sem estrias nem lâmina) e `sem_o_que_faz_cada_identidade_o_portao_reprova` (Shape fora, tecto
fora, `pressureSize` 0,9: todas SANGRAM) · `a_pressao_leve_desfaz_o_traco_em_grao` · `a_gota_do_po_e_do_preset` ·
`os_reprovados_nao_estao_na_fabrica`.

## ⛔ Recusas MEDIDAS
| recusa | medido |
|---|---|
| estrias pelos fios do Wet (`bristleStrength`) com o grão desligado | só salpico, contraste 0,65–0,83, nenhuma estria |
| Shape em Flow nas lâminas | o Flow roda o dab com o traço: a lâmina fixa vira 1,0 |
| `pressureGrain` 0,5 no Square Hard | a largura da rampa leve cai a 0, a lei da pressão 0,24 (> 0,18) |
| Square Hard com o grão Noise 160 | Textura da mídia 14 contra 21 do Rebelle, «ruído fino» para o revisor |
| a gota do pó grossa (8) no Powder | grão 4 × 4 px mas sem as nódoas; régua 246 → 229 |

## 2.ª ronda (J4, 21/10) — o pó solto no Wet

SHAs `e0ea982caf` (motor) · `9965b60b59` (Soft Pastel e Charcoal com o pó) · `3d79d08cd6` (Square Hard). Pranchas
`pranchas/pastel_carvao/r4/` (as da fábrica de hoje) e `r5/` (variantes recusadas). Medição do pó: `po2.py` (o `po.py` do
motor das texturas por faixa de distância à borda; scratchpad `pastel2/`). Corte de custo do dono a meio: 2 rondas no máximo.

**Motor (aditivo, `po = 0` ao byte):** `stroke::spray::graos_do_po` = a lei dos grãos com dois chamadores (o Digital ao bit
e o Wet) · `Engine::pousa_grao_solto` (wet-paint `painter/po.rs`): cada grão pela lei do dab num rasto PRÓPRIO, recomeçado e
pousado logo — sem corda, âncora nem carga da lane (o miolo não muda: 191 → 193 px; antes 92 → 109) · `wetpaint/po.rs`: os
grãos de cada dab do lote depois do lote, à volta do raio VISTO (não o do motor), peso `1 ÷ FRACAO_DO_FINO` (com o de um dab
eram névoa) · ⭐ `DabJitter::po_lado` — medido no Rebelle: o pó cai à DIREITA de quem anda (recto: 411 × 191 em baixo; vertical
para baixo: à esquerda), Soft Pastel 0,37, Charcoal 0,43 · `presets_de_fabrica::direito` mede a largura SEM o pó (com ele a
tabela do Charcoal lia 1,5× e o miolo ficava mais fino que o slider). Portão red-first `o_po_solto_cai_a_volta_do_traco_no_wet`
(Soft Pastel a 200 px: 94 grãos/100 px longe da borda, 0 sem o pó, lado 2,99 × 1,38); mutações M1 (sem pousar), M2 (sem o
lado), M3 (sem o peso) SANGRAM. Verdes: volta_ao_byte, a_miniatura, a_lei_da_tinta, leis_sistemicas, wetpaint_preset (lib),
o_po_solto_espalha (Digital), a_largura_e_a_prometida, parece_a_sua_midia, a_pressao_de_cada_preset.

| Rebelle (recto 200, `po2.py`) | grãos/100 px a ≥ 0,2 meia-larg. | área · tom | nosso |
|---|---|---|---|
| Soft Pastel `Dry Thin` | 56 (16 13 15 13 15 7 6 por faixa) | 3 · 0,18 | 54 · 2 · 0,20 (`po` 2,5 · 1,0 · 1,5 px · 0,08 · lado 0,37) |
| Charcoal | 24 (todos a 0,1–0,5) | 2 · 0,15 | 0 25 10 6 2 · 3 · 0,16 (3,5 · 0,4 · 1,5 · 0,1 · 0,43) |
| Flat Soft (Pastel Side) | 60, 21 além de 1,5 | 2 · 0,21 | ps2 v0 63 · 4 · 0,30 (2,5 · 4 · 1,5 · 0,12 · 0,35) — não aplicado |

**Soft Pastel — curado, NÃO aprovado.** Régua 414/454, vizinho Hard Round 0,782. r4 (6/6, baixa→média): o pó já não é o
defeito principal; ficam (1) grão aos grumos com furos × riscas finas ao longo e borda que se esvai em névoa, (2) pontas
RECTAS no Rebelle e a rampa que ganha LARGURA em lente, (3) a 5 px o Rebelle tracejado (o nosso contínuo: cv 0,02 × 0,28) e
o azul que tapa mais o vermelho. Ronda 2 (r5, `dab_quina` 0,5–0,7 com o dab a seguir o traço): as pontas ficam rectas mas a
mídia lê Orla 3 × 6 (a quina endurece a borda lateral) — recusada, a fábrica fica a da r4.
**Charcoal — curado, NÃO aprovado.** Régua 424/454, vizinho Graphite 2H 0,865. r4 (6/6 alta) e r5 (estrias em Flow +
quina, régua 396; 6/6 média) dizem o mesmo: o Rebelle tem manchas de tom e arranhões brancos (fibra), é mais escuro e
compacto, a rampa some no leve e engrossa de repente em fuso agudo; o nosso mais pálido, auréola de pontinhos mais larga,
pontas redondas. Fábrica = r4 (mármore + pó).
**Square Hard — curado, sem revisão cega (corte de custo).** Decisão 16: carga 110 · papel 0,8 — R nos cruzamentos 75,6
(Rebelle 74; antes 114), régua 188 → 277/295 (variante só A). O MIDIAS perde Textura e Cv (a foto antiga lia 13 × 20,9 e
0,044 × 0,077; na foto nova os do Rebelle) e ganha `o_square_hard_tapa_o_vermelho_como_a_foto` (a carga de antes: 97).
**Pastel Side, ponta de imagem: não feitos** (corte de custo; o `escreve_os_em_estudo` já os pinta fora da fábrica).

### Fila (com a medição)
- **Tampa RECTA de um dab redondo** (Soft Pastel, Charcoal, Wax Scrape — as revisões r1–r5 repetem): a `dab_quina` dá-a mas
  endurece a borda lateral (Orla 3 × 6). Falta cortar a cápsula a direito no início e no fim do traço, não o carimbo (motor).
- **O fino tracejado** (Soft Pastel a 5 px, cv 0,28 no Rebelle × 0,02): o peso do traço fino enche-o; uma lei de dente pelo
  diâmetro (motor).
- Pastel Side com o pó (valores da tabela) + carga 150 (tom); Square Hard pela revisão cega; a ponta de imagem (Compressed
  Charcoal, Wax Scrape, Oil Pastel, Powder) por medir com `GrainLaw::area`.
