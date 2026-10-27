# W2 E4 — os Dabber do Rebelle (Dabber · Dabber 2 · Dabber 3 · Dabber 4 · Fan), 27/10

Agente Dabber da E4 (Óleo Real W2). Método: bancada `04_BANCADA.md` + revisão cega `05_REVISAO_CEGA.md`, ≤ 2 rondas,
um revisor `sonnet` por ronda. Oráculo: `fixtures/rebelle_bateria/OIL_AND_ACRYLIC/dabber_*` (6 PNG por preset).

## Resultado

| preset | veredito | rondas | régua | vizinho (d; limiar 0,591) | escala | prancha |
|---|---|---|---|---|---|---|
| Dabber | **mantido** (aprovado na J5, não refeito) | — | J5 | — | — | `pranchas/dabber.png` (J5) |
| Dabber 2 | curado-não-aprovado | 2 | 370/454 | Dabber 0,603 | 0,561 | `pranchas/W2_Dabber_2.png` |
| Dabber 3 | curado-não-aprovado | 2 | 280/454 | Dabber 1,452 | 0,729 | `pranchas/W2_Dabber_3.png` |
| Dabber 4 | curado-não-aprovado | 2 | 336/454 | Square Hard 1,551 | 1,138 | `pranchas/W2_Dabber_4.png` |
| Fan | curado-não-aprovado (o mais perto) | 2 | 292/454 | Dabber 1,228 | 1,167 | `pranchas/W2_Dabber_fan.png` |

Nenhum entrou na fábrica (precedente `os_reprovados_do_oleo_nao_estao_na_fabrica`): os ficheiros vivem em
`W2_Dabber_candidatos/` (o JSON é o do Dabber com as mudanças abaixo; a miniatura é a do Dabber, por regerar).
sha256 (16): Dabber 2 `e6fd6e9e0eefaee3` · Dabber 3 `88cc326c252256b0` · Dabber 4 `64c0c7804a3e6f2b` · Fan `30c398b22c295bad`.

⚠️ **A bancada não liga o Óleo Real**: `exame_bancada::pinta_cena_com` cria `PainterTool::default()` (`oleo_real: None`)
e não há variável para o ligar. Todas as pranchas acima (como as dos 18 aprovados) são o render de sempre (emboss do
preset). Ligar = ~3 linhas em `crates/ph2d-tool-painter/tests/it/exame_bancada.rs` (`t.set_wet_oleo_real_ligado(true)`
sob `PH2D_EXAME_OLEO=1`) — código alheio a este agente; fica pedido.

## Valores (sobre o Dabber de fábrica)

- **Dabber 2**: `bristleSize` 3 · `paperGate` 0,5 · `brush.jitter` 0,6.
- **Dabber 3**: `bristleSize` 8 · `bristleCount` 400 · `bristleLift` 0,8 · `felt` 0 · `emboss` 0,8 · `brush.jitter` 0,5 ·
  `brush.texture` Stucci Tiled size 45.
- **Dabber 4**: `dabRotation` 0 · `brush.dab_flatten` 0,6 · `brush.dab_angle_deg` 90 · `brush.jitter` 0,1 · `brush.texture`
  Stucci Tiled size 180.
- **Fan**: `dabRotation` 0 · `bristleSize` 3 · `emboss` 0,8 · `bristleLaneWidth` 3 · `bristleRemove` 0,3 · `pigmentPerDab`
  1000 · `brush.dab_flatten` 0,78 · `brush.dab_angle_deg` 0 · `brush.jitter` 0,15 · `brush.continuous_spacing` false.
- `size_px` ficou o do Dabber (51): a escala da tabela é o que o preset tem de multiplicar ao entrar.

## O cruzamento (chave em `pranchas/chaves/W2_Dabber_*.json`)

- **Dabber 2** — r1 (`bristleSize` 3, `bristleNoiseLight` 0,4, `emboss` 0,8, Spacing sem cápsula): o revisor achou o
  oráculo 6/6 e deu o VIZINHO (o nosso Dabber) como imitação 6/6; o nosso era «o outro» (riscas azul-claras, contorno
  em gomos). r2 (luz 0,1, emboss 0,5, cápsula): o revisor confundiu o oráculo com o Dabber de fábrica (4/6 «original»
  = Dabber) e pôs o nosso como terceiro em gestos/sobre: «1,5–2× mais largo, liso, opaco; a 200 px massa fechada
  onde o original tem buracos e borda salpicada». ⇒ o Dabber 2 do Rebelle é quase o nosso Dabber; o nosso Dabber 2
  não o imita melhor que o Dabber. Diferença aberta: a 200 px buracos e borda rasgada com o miolo cheio a 40 px — o
  Stucci Tiled não escala com o pincel (abre buracos a 40 px antes de os abrir a 200).
- **Dabber 3** — r1 (Stucci 35, sem cápsula): o revisor escolheu o NOSSO como original 6/6, mas viu o oráculo como
  «o outro» em 4/6 (lascas agudas) — não é imitação. r2 (Stucci 45, cápsula): oráculo achado 6/6 (média–alta), o
  nosso = imitação 6/6. Diferenças: grão em ruído fino × FLOCOS grandes e angulosos «de escala de cerda»; papel demais
  (o vermelho aparece pelos furos); borda pontilhada × saliências de flocos. Lacuna de motor: carimbo de lascas
  (forma de flocos por dab, não um grão fixo na tela).
- **Dabber 4** — r1 (sem flatten): oráculo achado 6/6, nosso = imitação 6/6: «mais largo, ralo, grão grosso; a espiral
  enche». r2 (flatten 0,6 a 90° fixo, Stucci 180, jitter 0,1): oráculo achado 6/6 (baixa–média), nosso = imitação 6/6,
  Square Hard claramente outro. Diferenças: borda lisa e redonda × ondulada; grão salpicado × fino em cerda; os
  verticais mais claros e pontilhados.
- **Fan** — r1: oráculo 6/6, nosso = imitação 6/6. r2 (`bristleRemove` 0,3, pigmento 1000, jitter 0,15): o revisor
  escolheu o NOSSO como original em 4/6 (gestos e sobre) e só achou o oráculo nos tamanhos (alta): «a 200 px o
  original é um PENTE de cerdas com espigas verticais; a imitação um salpicado esparso». Diferença aberta = motor.

## Fila de motor (não começada — nenhum cabe nos ficheiros deste agente)

1. **Leque a 200 px** (Fan; CAP-01/19 do `01_CATALOGO` G-8): as cerdas do Rebelle deixam espigas PERPENDICULARES ao
   traço (cada carimbo pinta os fios do leque); as nossas faixas de cerdas correm AO LONGO do traço. Precisa de fios
   no referencial do carimbo, não do traço.
2. **Lascas por carimbo** (Dabber 3): flocos angulosos grandes que acompanham o tamanho do pincel; o grão `Tiled` está
   preso aos px da tela (size 45 = período 5,7 px a qualquer tamanho) e o modo View, à pegada.
3. **Buracos que crescem com o tamanho** (Dabber 2): o mesmo — um grão por pegada com limiar.
4. Óleo Real na bancada (acima).

## ⛔ Recusas MEDIDAS

| tentativa | medido | porquê não |
|---|---|---|
| `grain_depth` 0,35–0,5 com Stucci (graduar os buracos) | o traço sai liso, sem grão, e mais largo | o depth não é uma graduação útil do Stucci no Wet |
| Clouds / DistortedNoise / Noise / Grain Tiled size 1–160 | sem efeito visível ou manchas do tamanho da tela | só o Stucci (e o Voronoi, em grelha) mordem o dab |
| Voronoi Tiled size 6–16 | grelha regular de quadrados | padrão de rede, não esponja |
| Spacing sem cápsula no Dabber 2/3 | contorno em gomos a 200 px | o revisor apontou-o («contorno em gomos») |

Nota de processo: subagentes irmãos partilham o MESMO scratchpad da sessão — `PH2D_EXAME_DIR=$scratchpad/exame` colidiu
com o agente Metallic (o `rm -rf candidato` dele apagou a pintura deste: `exame_pinta` NotFound). Use um dir com o nome
da família.
