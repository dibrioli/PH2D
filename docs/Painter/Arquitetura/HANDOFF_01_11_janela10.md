# HANDOFF — refatoração do Painter, janela 10 (01/11)

Ramo `line/PainterWatercolor`, de `1cd27137be` a este commit. Nada integrado nem empurrado. Anterior:
[HANDOFF_01_11_janela9.md](HANDOFF_01_11_janela9.md). Detalhe e números: [J10_chao_como_forca.md](J10_chao_como_forca.md)
· bisseção: [J10_bissecao_vermelhos.md](J10_bissecao_vermelhos.md).

## 1. Feito

| commit | o quê |
|---|---|
| `38b497468f` | **o chão como FORÇA** (`wp/solver/chao_forca.rs`, `Grid::chao` fotografado no início do traço): `v += G·∇(h + chão)·min(1, f ÷ FILM_CHEIO)`, `G = 0,125`; seco pelo líquido de óleo (`film·m`); a ladeira sai do nivelamento (o `LevelClamp` mordia 73 % da orla). Dabber seco: atravessa 344 982 → 0; curto: lados 249 → 26 614; 6 traços passam por cima (1 082 565). 4 testes `chao_forca_regua_tests` com mutação M1–M3; gates ao bit verdes |
| `6c9217d617` | bisseção dos 9 vermelhos (famílias N e A achadas; P e S por isolar) e a tentativa «só o que se vê» na composição MEDIDA e desfeita |
| este | smoke do dono, handoff, prompt da janela 11 |

## 2. Smoke do dono (01/11, app `--profile smoke`, Radeon)

Fotos [`fotos/j10/smoke_dono_1.png`](fotos/j10/smoke_dono_1.png), [`smoke_dono_2.png`](fotos/j10/smoke_dono_2.png)
(Dabber azul seco, Wet Drip amarelo por cima). **«A simulação ficou melhor e interessante, empoçando corretamente.»**
Fila, nas palavras dele:

1. **«A água ataca o óleo com velocidade maior que o próprio escorrimento — irreal, já que há o relevo.»** Foto 1: o
   verde da mistura ENTRA no corpo do Dabber (setas) em pontos, mais depressa que a água escorre no papel. Hipótese a
   medir: dentro do relevo (chão alto, `w_h = 1`) a parte `∇h` sem clamp + `∇chão` das rugas do pincel puxa a água para
   os vales do óleo; e/ou o pouso da tinta B nas células do óleo é instantâneo.
2. **«Não há a mistura das tintas de forma realista, dissolvendo e reduzindo o relevo. Deve ser configuração, essa
   possibilidade existe neste motor, já vi funcionando.»** O knob `Knob::Solubility` (`tuning/defs.rs:329`, Hidden,
   fábrica **0**; `OleoCelula::soluvel`, J10 de 30/10 «quanto a água de um traço seguinte dissolve desta tinta») — o
   Dabber pousa com 0. Conferir o preset e o caminho que dissolve (`pickup`/remolha, E10e2 §E3) e se dissolver BAIXA o
   chão (a altura assente sai com a massa levantada).
3. **«O pixelado duro persiste.»** Foto 2: degraus quadrados na orla da gota e na borda de baixo do azul (setas), um
   entalhe em bloco onde o escorrido verde encosta. Suspeito: a grade de FLUXO grossa (`rf > 1`): velocidade por bloco,
   o chão lido só na célula probe (`flow_probe`).

## 3. Aberto (por ordem)

1. Os três itens do §2 (uma família: água × óleo seco) — medir antes de mexer; réguas `chao_forca_regua_tests`,
   `a_queda_cresce_com_a_diluicao`, `a_agua_nao_sobe_a_ladeira_do_oleo`.
2. **FPS 70 → 30 com Real Oil**: achar QUEM marca a caixa molhada inteira a cada tique (o filtro «só o que se vê»
   não mudou as células do motor: 17 989 = 17 989) — contador por chamador de `Engine::merge_dirty`; depois religar o
   filtro (código guardado em `target/sondas/j10/vista.rs`, `vista_tests.rs`; o doc J10 §6). Decisões do dono 2b da
   janela 9 continuam (não baixar o passo, adormecer o invisível, marcador ao lado do Dry — UI no app).
3. Vermelhos antigos: N = o miolo `sett+susp > 3000` na unidade antiga (`medida_oleo_real_fenda.rs:252`) — limiar
   relativo + assert de miolo não-vazio, e conferir se o Dabber migrado pousa a mesma altura ([219] × [220]); A = o alvo
   Cycles medido no relevo antigo (correr o oráculo de novo); P e S por isolar (suspeitos no doc da bisseção).
4. Do §4 da janela 7 (E9 ⚑ furos, chip de Dilution, ficheiros em 700 linhas, E10c, E11 GPU com o meio E o chão, E12,
   E13, outros 3 modos, ajuda ilustrada, crateras).
