# HANDOFF — refatoração do Painter, janela 11 (01/11)

Ramo `line/PainterWatercolor`, de `18edf350e2` a este commit. Nada integrado nem empurrado. Anterior:
[HANDOFF_01_11_janela10.md](HANDOFF_01_11_janela10.md). Detalhe e números: [J11_agua_no_oleo.md](J11_agua_no_oleo.md).

## 1. Feito

| commit | o quê |
|---|---|
| `J11` (feat) | **a água × o óleo seco num bloco**: o atrito do leito (`chao_forca.rs`, `v ÷= 1 + 0,3·(½|∇chão|·dif ÷ f)²`; a frente no corpo do óleo 99,8 → 90,1 contra 91,5 no papel); a água PARADA dissolve `Re-wet × Solubility` (`solver/remolha_parada.rs`, `TAXA 0,02`; orla 0,803 → 0,071 com Re-wet 1); o chão segue o assentado (`Grid::sett_do_traco`). Sonda `agua_no_oleo_tests` (+ contador `painter/conta_sujo.rs`), 3 réguas `agua_no_oleo_regua_tests` com mutação M1–M3; réguas J10 e portões ao bit verdes |
| este | smoke do dono, handoff da janela e handoff de integração da linha |

## 2. Smoke do dono (01/11, app `--profile smoke`, Radeon)

Foto [`fotos/j11/smoke_dono_1.png`](fotos/j11/smoke_dono_1.png) (Dabber azul seco em V, Wet Drip amarelo por cima):
**«Realmente melhorou muito! Parabéns.»** A água empoça no fundo do V, mistura-se com a orla (verde) e não invade o
corpo do azul. Ordem seguinte do dono: **integrar no main antes de seguir** (só os handoffs nesta janela).

## 3. Aberto (por ordem, depois da integração)

1. **FPS com o Real Oil** (J11 §4): o único marcador de sujo é o passo (`painter/stage.rs:48/53`); o Dabber seca em
   poucos tiques; o custo do Real Oil está no TRAÇO (+2,8 ms por evento do rato numa tela 1200×800) e na água que
   corre por cima do óleo (a caixa inteira recomposta a cada passo). Medir no app (tela real, FPS) e atacar a
   composição por passo onde a água corre (ritmos relevo/luz do dono, J9 2b).
2. O rectângulo velho do smoke J10 (foto 2): o motor não o reproduz (0 px, relógio fixo e real) — se voltar, a subida
   da tela na shell.
3. Vermelhos antigos N, A, P, S ([bisseção](J10_bissecao_vermelhos.md)).
4. Do §4 da janela 7 (E9 ⚑ furos, chip de Dilution, ficheiros em 700 linhas, E10c, E11 GPU com o meio e o chão, E12,
   E13, outros 3 modos, ajuda ilustrada, crateras).
