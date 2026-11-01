# E9b — o desvio de cor do Thick Impasto na lei `Pigmento` (31/10)

Célula: plano, papel padrão, lei `Pigmento`, `Oil/Flat/Thick Impasto`, P600 / W0,5 / pr1 / N3.
Antes: `mix_p99 9,5` · `mix_max 13,9` (tecto do `portao_mae_f0_lei_pigmento_minimo`: 8), píxeis cobertos
`31,89,225`/`30,89,227` contra o azul `40,90,220`. Wet Round na mesma célula: `3,0 / 3,4`.

## Resultado: o mecanismo NÃO está no depósito — é o ruído de SATURAÇÃO por cerda do preset

O Thick Impasto nasce do Filbert e herda `bristleNoiseSat = 0,3` (lido no PNG do preset,
`assets/brushes/WetPainter/Oil/Flat/Thick Impasto.ph2d-brush.png`; `bristleNoiseLight 0`). A ferramenta
(`crates/ph2d-tool-painter/src/tool/paint/wetpaint/pousa_o_dab.rs:425`, `fios.tem_cor()`) dá a cada FIO uma cor
HSLuv própria → `Trail::tinta_r/g/b` (a tinta limpa por célula) → `transfer.rs` (`ponta = w·limpa + sujidade`) →
`Pigmento::pousa`. Na lei nova cada célula coberta mostra a cor do fio que a pousou; o desvio `31,89,225` é um fio
MAIS saturado que o azul escolhido. A régua `mix_*` conta-o como erro de mistura.

## A medição (instrumentação temporária, já removida)

Um gancho em `transfer.rs`, ramo `Pigmento`, trocava por variável de ambiente a cor que chega ao `pousa` e somava,
pela massa `v`, a cor média de cada candidata. Um binário, cinco corridas da célula (só Thick Impasto, Pigmento):

| cor que pousa (`PH2D_E9B`) | o que é | `mix_max` | piores píxeis |
|---|---|---|---|
| `ponta` (hoje) | `w·limpa + (tip − w·base)`, linear sRGB | **13,9** | `30,89,227` C 0,991 |
| `cura` (suspeita a) | soma da ponta (`ponta_tinta[l].cor()`) ⊕ limpa pelo `limpo`, em Oklab | **13,9** | igual |
| `limpa` | só `tinta_r/g/b` (a cor do fio) | **13,9** | igual |
| `tip` | só `tip_r/g/b` (a mistura Oklab do `ponta_pigmento`, SEM a cor do fio) | **3,4** | `40,89,222` C 0,991 |
| `base` | a cor do pincel | **3,4** | `40,89,222` |

Médias pela massa (azul, 1,6 M pousos): `tip 39,99·90,00·220,00` · `ponta 39,76·90,20·219,59` ·
`cura 39,78·90,20·219,59` · `limpa 39,81·90,20·219,57`; `limpo` médio `0,992`, ponta com soma em 99,8 % dos pousos.

Leitura:
- **(a) refutada**: a fórmula linear da sujidade e a cor Oklab da soma da ponta diferem `≤ 0,25` sRGB na média, e
  trocar uma pela outra deixa o `mix_max` em `13,9`. O espelho sRGB do bico não é o mecanismo.
- **(b) refutada**: sem a cor do fio (`tip`/`base`) o Thick Impasto dá `3,4` — o MESMO chão do Wet Round (`3,4`);
  a espessura do óleo (`M_UM` a m = 1) não muda a cor de masstone da célula.
- **(c) confirmada**: a cor por cerda que a ferramenta entrega (o ruído HSLuv do preset). Média ≈ azul (o ruído é
  simétrico no azul), cauda p99 ≈ 9,5. No amarelo `245,205,25` o ruído NÃO é simétrico: `limpa` média `244,4·205,1·39,3`
  (B +14) — a saturação satura no tecto e o ruído só pode dessaturar (outra entrada para a mesma decisão).

## Cura: fora destes paths — decisão do coordenador

Nada mudou no código (os dois ficheiros instrumentados voltaram ao HEAD). Opções, todas fora de `ph2d-wet-paint`:
1. **A régua**: `mix_*` comparar cada píxel com a cor do FIO que o pousou (ou correr a célula com `bristleNoise*` a 0)
   — a mistura é a lei medida; a variação por cerda é desenho do preset (`sonda_varredura_regua.rs`/`portao_mae_f0.rs`).
2. **O preset**: baixar `bristleNoiseSat` do Thick Impasto (olho do dono: é o «cada cerda com a sua cor» do Filbert).
3. **A ferramenta**: o ruído HSLuv do amarelo enviesado (média B +14) — um ruído de saturação que não é de média
   zero muda a cor escolhida; vale um teste em `bristles_cor::com_ruido` (média do ruído = a cor do pincel).

## Cura (o coordenador, janela 4)

Opções 1 e 3 juntas (a 2 é olho do dono e fica sem mudança):

- **3 → `wp/bristles_cor.rs` `com_ruido`:** o `ds` escala-se pela folga até ao limite mais perto, `min(s, 100 − s)/50`
  (com `|ds| ≤ 0,5`, a faixa do registo, nunca sai de `[0, 100]`): ruído simétrico ⇒ saturação média = a da cor
  escolhida. Teste `bristles_cor::tests::o_ruido_de_saturacao_tem_media_zero` (amarelo `245,205,25`, azul, cinzento ×
  ±0,1/0,3/0,5); prova de mutação: sem a folga reprova (`bristles_cor.rs:216`), com ela passa. A luminosidade continua
  presa (o veio é escuro de propósito e não tem média zero) — aberto se o olho o pedir.
- **1 → a régua:** `sonda_varredura_oleo.rs` `preset()` tira `bristleNoise{Hue,Sat,Light}` do preset salvo
  `PH2D_SONDA_COR_DO_FIO=1` — `mix_*` mede a lei de MISTURA, não o desenho do pincel.

| célula (plano, padrão, P600/W0,5/pr1/N3) | antes mix_p99/max | depois | orla_p99 |
|---|---|---|---|
| Thick Impasto Pigmento | 9,5 / 13,9 | **3,0 / 3,4** | 29,1 |
| Wet Round Pigmento | 3,0 / 3,4 | 3,0 / 3,4 | 25,3 |

`portao_mae_f0_lei_pigmento_minimo` **verde** (30 s) ⇒ sai o `#[ignore]`: o portão-mãe corre na suíte.
A grelha E9 inteira re-corre com esta régua (o binário congela-se agora numa cópia em `$OUT/sonda.bin`, `corre_f0.sh`;
a corrida anterior morreu às 23:19 com 1 798/3 240 linhas, sem causa no registo).
