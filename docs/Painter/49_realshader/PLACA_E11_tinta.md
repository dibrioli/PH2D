# E11: a lei `Pigmento` na placa (05/11)

Plano: [F4_2 §6](../Arquitetura/F4_2_plano_wet_paint.md). Portão: `crates/ph2d-wet-gpu/tests/tinta.rs`,
`paridade_da_tinta_por_placa`. A régua é por placa ([memória 24/10](../../../project-memory/project_same_bit_law_is_per_device_2026_10_24.md)).

## O que a placa faz com a lei nova

- **Na placa:** o gather das somas com a massa (`t_transporte`, `t_sedimenta`, `t_bordas`), o meio pelo volume molhado, Bingham e
  mobilidade no campo (`campo_oleo`), a secagem pelas somas com retenção e evaporação do meio (`seca_tinta`), o espelho sRGB, o decaimento
  da abertura (`t_cura`). As passadas são módulos à parte (`advect_tinta.wgsl`, `seca_tinta.wgsl`): com a lei velha nada disto se aloca.
- **Na CPU:** o FECHO da cura (a placa conta quem o pede; o passo desce, fecha, sobe), a composição do ecrã. `r_g`, `r_g0`,
  `altura_fechada` e a aparência da célula (`suave` incluído) não sobem: nada na placa os escreve.
- **`placa_sabe` recusa, com a lei nova:** Pigment mix (lado espectral), chão como força, `Knob::Dissolve`, `Knob::EdgeSoftMass`, Real Oil na
  mão (`p.oleo`), tinta opaca. O motor fica na CPU e não diverge.
- FMA: o ecrã não migrou, logo `camada_banda` (a única conta que o F4-1 pedia sem contracção) não corre na placa; o que corre é linear mais
  `pow`/`exp` na retenção, na mobilidade e na cura. Determinismo medido abaixo, em vez de lutar contra o compilador.

## A medição (320×200, 120 passos, `--profile smoke`, 05/11)

`bit` = células com algum campo diferente entre duas corridas na mesma placa. `cor` = níveis sRGB 0–255 sobre branco, placa × CPU.

| cena | placa | bit | cor p99.9 | cor máx | células > 0,5 | massa rel. |
|---|---|---|---|---|---|---|
| água | Radeon 610M, Mesa 26.2.5 | 0 | 0,0012 | 1,15 | 5 | 1,0e-7 |
| óleo diluído | Radeon 610M | 0 | 0,0016 | 0,003 | 0 | 2,8e-7 |
| pasta alta (`Dilution` 0,2, `Body` 0,5) | Radeon 610M | 0 | 0,107 | 21,6 | 22 | 2,0e-7 |
| óleo meio curado (`o` = 0,5) | Radeon 610M | 0 | 0,0016 | 0,003 | 0 | 2,8e-7 |
| água a sedimentar | Radeon 610M | 0 | 0,789 | 15,9 | 75 | 2,4e-7 |

A RTX 5060 Ti (NVIDIA 615.78.08) deu, antes de a cena entrar pela borda e de haver as duas últimas: 0 bit; água 0,0009/0,0017, óleo
diluído 0,0076/6,18, pasta alta 0,0993/21,6. As duas placas dão a MESMA pior célula na pasta: é aritmética de f32 × f64, não ruído de placa.

**A tolerância, de cada linha da tabela:** cor p99.9 ≤ 0,5 (a do degrau da lei velha) · máx ≤ 32 · acima de 0,5 no máximo 0,1 % das células;
nenhuma pede menos que 1,5× o controlo CPU × CPU a 1 ulp · cada campo no p99.9 (ver as constantes do teste, ~20× o medido).

## A franja: por que o máximo não é 1

A máscara `active` pergunta `film > 0`, e o passo a passo mostra a 1.ª divergência na máscara (5 células em `n = 12`), com film e massa
iguais a 8 algarismos. A CPU sozinha, zerando film e susp abaixo de ε a cada passo, contra a CPU sem isso:

| ε | água | óleo diluído | pasta alta | a sedimentar |
|---|---|---|---|---|
| 1e-38 | 0,00 (0) | 0,00 (0) | 13,2 (3) | 0,00 (0) |
| 1e-30 | 0,00 (0) | 1,37 (3) | 13,2 (15) | 0,00 (0) |
| 1e-20 | 14,1 (73) | 23,8 (14) | 112 (119) | 18,7 (92) |

(cor máx, em níveis; entre parêntesis as células acima de 0,5.) A sedimentar é a mais nervosa: a CPU contra si mesma a ±1 ulp de partida já dá
15,9 em 75 células, o mesmo número da placa. Isto é a sensibilidade da LEI a um resíduo que desaparece, e existia antes da placa. ⚠️ Medido só
como controlo: a mão de um resíduo de 1e-38 decidir um escoamento é um achado aberto no [BUGS #63](../BUGS_painter.md), não curado aqui.

## Custo (RTX 5060 Ti, 4096×2560, água, intercalado no mesmo processo, mínimo de 8 blocos de 6 passos)

Lei velha 5,74 ms/passo · `Pigmento` 9,21 ms/passo, com `load` ~120 na máquina (a CPU não entra no passo da placa).
