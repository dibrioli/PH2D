# J11 — a água × o óleo seco: o atrito do leito, a água parada que dissolve, o pixelado e o FPS (janela 11, 01/11)

Ordem: o §3 do [handoff da janela 10](HANDOFF_01_11_janela10.md) (smoke do dono J10: «a água ataca o óleo com
velocidade maior que o próprio escorrimento» · «não há a mistura dissolvendo e reduzindo o relevo — deve ser
configuração, já vi funcionando» · «o pixelado duro persiste»). Base `18edf350e2`. Sonda:
`wetpaint/agua_no_oleo_tests.rs` (`diag_j11_agua_no_oleo`, `diag_j11_quem_suja`, `diag_j11_fps_do_dabber`); réguas:
`wetpaint/agua_no_oleo_regua_tests.rs`.

## 1. A água atravessava o óleo mais depressa que escorria no papel — MEDIDO

Cena: Dabber azul seco (1200 tiques), Wet Drip d0,9 Water por cima em `y = 55`; a mesma água sobre o papel nu como
controlo. «Frente» = a linha mais funda (média das colunas 70–170) com massa nova de B > 50; amostras de 15 tiques
depois de seis passagens.

| frente (linha) aos 60 · 90 · 120 tiques · fim | papel nu | óleo, antes | óleo, depois (atrito) |
|---|---|---|---|
| seis passagens | 78,0 · 84,6 · 91,5 · 156 | 81,1 · 90,3 · **99,8** · 144 | 79,2 · 84,5 · **90,1** · 123 |

Velocidade média da água: sobre o óleo 0,6 contra 0,15–0,2 no papel ao lado (3–4×). **Mecanismo:** dentro do relevo o
chão como força (J10) empurra a água pelas rugas do pincel a cada passo e nada a freia — no papel o papel bebe a água e
o travão (`Brake`, a sonda à frente) segura-a; sobre o óleo o filme fica cheio e o travão solta.

**Lei** (`wp/solver/chao_forca.rs`, `K_ATRITO`): o atrito do LEITO de águas rasas — a água rasa sobre o leito rugoso é
segurada, a funda corre (Manning): `v ÷= 1 + K·(½|∇chão|·dif ÷ f)²`, `K = 0,3`, nas células com declive de chão, na
mesma passagem da força. Sem chão (sem Real Oil na grade, Real Oil na mão) não corre: os portões ao bit ficam.

Varredura (mesmo processo, réguas J10 ao lado): 

| atrito | frente aos 120 · fim | 6 passagens: abaixo | 1 passagem: abaixo · acima | curto: lado |
|---|---|---|---|---|
| sem | 99,8 · 144 | 1 069 607 | 0 · 440 710 | 30 319 |
| linear 0,3 | 90,7 · 125 | 573 889 | 0 · 459 952 | 19 245 |
| linear 0,5 | 89,7 · 122 | 441 394 ✗ | 0 · 453 748 | 20 193 |
| **quadrático 0,3** | **90,1 · 123** | **532 961** | 0 · 456 222 | 27 521 |
| quadrático 1 | 89,0 · 117 | 282 494 ✗ | 0 · 433 476 | 17 538 |
| quadrático 3 | 87,2 · 109 | 55 214 ✗ | 0 · 407 054 | 18 175 |

Foto: [`fotos/j11/atrito_antes_depois.png`](fotos/j11/atrito_antes_depois.png) — à esquerda (antes) o verde pontilhado
que invadia o corpo do óleo e os pingos; à direita a água cobre o óleo por igual e transborda pela orla de baixo.

## 2. A mistura que dissolve e baixa o relevo — o motor tinha-a só sob o pincel

`Knob::Solubility` (a tinta de baixo, na célula) × `Knob::Dissolve` («Re-wet», o pincel de cima) já estão no cartão do
pincel (`populate_wetpaint.rs`); nenhum pincel de fábrica os usa (matriz F1: todos 0). Medido com Solubility 1 no
Dabber: o re-molhar só corria sob o pincel, no traço (`trail/transfer/remolha.rs`) — a orla do óleo perdia 8 % (Re-wet
0,35) a 20 % (Re-wet 1); a poça parada contra a orla não dissolvia nada, e o chão da água (a fotografia do início do
traço) não baixava com o que se dissolvia.

**Lei** (um bloco):
- `wp/solver/remolha_parada.rs`: a água PARADA dissolve a cada passo `Re-wet·Solubility·TAXA·min(1, f ÷ FILM_CHEIO)` do
  assentado do início do traço, até ao tecto `Re-wet·Solubility` dele (o mesmo tecto do re-molhar do pincel), pela porta
  `lift_settled` com `Ecra::Porto` (duas tintas na mesma água: o azul com o amarelo dá verde). `TAXA = 0,02`.
- `Grid::sett_do_traco` (fotografado em `chao_forca::fotografa`, no início do traço, com o campo do óleo): o chão de
  agora é `chão × min(1, sett ÷ sett_do_traco)` — a tinta que a água dissolve OU levanta baixa o relevo que a segura.

| Re-wet × Solubility | assentado que resta na orla (sem → com a água parada) | 6 passagens: abaixo do óleo |
|---|---|---|
| 0 × qualquer | 0,95 (o re-molhar da aquarela) → igual | 251 214 |
| 0,35 × 1 | 0,918 → **0,845** (TAXA 0,005: 0,887 · 0,08: 0,802) | 1 250 462 → **4 075 929** |
| 1 × 1 | 0,803 → **0,071** | — |

Foto: [`fotos/j11/agua_parada_dissolve.png`](fotos/j11/agua_parada_dissolve.png) — em cima: hoje · Re-wet 0,35 · Re-wet
1 (a orla aplana e o azul sai com a água); em baixo: 6 passagens insolúvel · 6 passagens Re-wet 0,35 (a água verde
escorre) · Re-wet 1 sem a água parada (antes). Fábrica: Solubility e Re-wet continuam 0 (o óleo seco não se dissolve
na água; a lei é do cartão, como o dono disse).

## 3. O pixelado — duas coisas

- **O «entalhe em bloco»** (smoke J10, foto 2, seta de cima: arestas rectas): a tela INCREMENTAL do motor × a
  recomposta inteira, tique a tique como o app, relógio fixo E relógio real (o worker) — **0 píxeis diferentes** nas 7
  cenas. O motor não deixa bloco velho; se o rectângulo aparecer de novo no app, vem da subida da tela para a placa
  (a shell), por medir no app.
- **O pontilhado de 1 célula** na orla (zoom 4× da sonda): a tinta B pousa célula a célula entre os pontos do Dabber
  (o Dabber é pontilhado por desenho). Com o atrito o verde pontilhado que invadia o corpo some (§1). Grade de fluxo:
  `flow_ratio` e `grid_ratio` de fábrica = 1 (não há grade grossa no produto).

## 4. O FPS com o Real Oil — quem marca

Contador por chamador (`painter/conta_sujo.rs`, `#[track_caller]` em `merge_dirty`/`mark_dirty_full`), 300 tiques
depois de um traço, tique + tela drenada como o app:

| pincel | composições | células do motor | caixa molhada média | chamadores |
|---|---|---|---|---|
| Dabber de fábrica | 2 | 27 846 | 44 | `painter/stage.rs:48/53` (8 + 7 chamadas) |
| Filbert Oil d0,9 | 200 | 3 525 648 | 16 946 | `painter/stage.rs:48/53` (1 400 + 1 400) |

O ÚNICO marcador é o passo (`step_stage`: a caixa molhada de antes e a de agora). O Dabber seca em poucos tiques; o
Filbert diluído CORRE de facto em toda a caixa — por isso o filtro «só o que se vê» da J10 não mudou as células: a cena
dele muda visivelmente em toda a parte. Quadro do app numa tela 1200×800 (Dabber, mínimo de 3, intercalado):

| | ponteiro por evento | tique depois | tela depois |
|---|---|---|---|
| Real Oil | 4,54 ms | 0,28 ms | 0,03 ms |
| sem Real Oil | 1,71 ms | 0,02 ms | 0,01 ms |

O custo do Real Oil está NO TRAÇO (+2,8 ms por evento do rato) e na água que corre por cima do óleo (o Filbert/Wet
Drip: a composição inteira da caixa a cada passo, J9 §3.2). Próximo passo: medir no app (tela real, FPS) o traço do
Dabber e o Wet Drip por cima, e atacar a composição por passo onde a água corre (o ritmo de relevo/luz do dono, J9
2b), não o sujo.

## 5. ⛔ Recusas MEDIDAS

| tentativa | o que deu |
|---|---|
| atrito linear `K·|∇chão|÷f`, K 1 e 3 | segura demais: 6 passagens 73 521 e 0 abaixo (a lei do dono pede transbordar com volume) |
| atrito quadrático K ≥ 1 | 282 494 / 55 214 abaixo com 6 passagens (régua J10 ≥ 500 000) |
| subir a régua J10 para caber o atrito | não: a régua é a lei do dono (transbordar com volume) |
| `TAXA` 0,08 | a orla de Re-wet 1 some (0,016) e a água passa em 3 352 293 com UMA passagem — dissolve num relance |

## 6. Aberto

- O rectângulo velho no app (§3), por medir na shell; o FPS no app (§4).
- O re-molhar sob o pincel (`trail/transfer/remolha.rs`) e o da água parada movem só a lei `Velha` (`lift_settled`); a
  lei `Pigmento` (sonda) pediria `drying::tinta::levanta` — o produto corre a `Velha`.
- Os vermelhos antigos (N, A, P, S — [bisseção](J10_bissecao_vermelhos.md)).
