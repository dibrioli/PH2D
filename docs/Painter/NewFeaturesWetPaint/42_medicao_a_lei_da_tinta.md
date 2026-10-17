# 42 — A lei da tinta por pincel: o que se MEDIU antes de construir (16/10)

> Prompt [42](42_PROMPT_a_lei_da_tinta_por_pincel.md), passo 1. Régua: o cruzamento do [18 §2](18_mapa_das_midias.md)
> (luminância do miolo vermelho × azul ÷ a menor das duas fora dele), no recto da bancada, seco.

## §1 — Onde o motor decide (o mapa)

- A mistura (`Params::mix`, `km_mixing`) e as velaturas (`glaze`, `glaze_filme`) são **da sim inteira**, tiradas uma vez
  por passo (`Sim::gather_params`); a lei do ecrã (`Ecra::Cobre { esconde, filme, molhada }`) também. Nenhum plano da
  célula guarda «qual lei» (o `Grid` tem `film · susp · susp_rgb · sett · sett_rgb · relevo`). O precedente de uma
  propriedade que VIAJA com a tinta é o `relevo` (Bug #48): escrito no pouso (`trail/transfer.rs`), pesado pela massa.
- As caixas da obra: `WetSession::reconcile_facts` — «Pigment mixing» liga `km_mixing` e põe `glaze = 0`; «Glaze
  layering» troca `esconde` 5 → 2 (`TINTA_ESCONDE_EM_VELATURA`). `VELATURA_MEDIDA` = `VELATURA_DO_FILME` = 0 hoje.

## §2 — O mecanismo do «tapa» (sonda `sonda_a_celula_do_cruzamento`)

Na bancada o azul ainda está **MOLHADO** quando o vermelho chega (suspenso 3 500–8 500, assentado 0) — o 18 §3 dizia
«o traço seco tapa»: errado. Cada carimbo denso pousa com o peso da cobertura `in_a/(e9+in_a)` ≈ 1 e **substitui** a cor
da água pela sua: `susp_rgb` (23,101,179) → (191,0,18) no 1.º dab, nos seis candidatos e em toda variante sem K–M.
Com K–M + velatura 1 o cruzamento vai a (11,13,13): a velatura re-aplica-se A CADA carimbo sobre a cor já escurecida
(o H1 do [05c](05c_estado_mistura_e_claridade.md), recusado) — 0,037 contra 0,22 do Rebelle.

| cruzamento (seco) | hoje | caixas da obra | só K–M | só Glaze layering | velatura 1 | filme 1 + esconde 1 | K–M + velaturas 1 | Rebelle |
|---|---|---|---|---|---|---|---|---|
| Ink Brush (`na_a`) | 1,011 | 0,836 | 0,846 | 1,010 | 0,975 | 1,009 | **0,057** (preto) | 0,239 |
| Dip Pen (`ca_a`) | 1,001 | 1,001 | 1,001 | 1,001 | 1,001 | 1,001 | **0,037** | 0,223 |
| Ink Bullet (`ca_b`) | 1,000 | 1,000 | 1,000 | 1,000 | 1,000 | 1,000 | 0,131 | 0,117 |
| Marker Liner (`ma_a`) | 0,985 | 0,773 | 0,777 | 0,984 | 0,934 | 0,980 | 0,036 | 0,016 |
| Marker Bullet (`ma_b`) | 1,000 | 1,000 | 1,000 | 1,000 | 1,000 | 1,000 | 0,036 | 0,016 |

## §3 — O oráculo, corrido de novo: o cruzamento SECO (fixtures `*seco.png`)

| preset | molhado (11/10) | **seco** (16/10) | fracção multiplicada (canal R) |
|---|---|---|---|
| Ink · Pen/Bullet | 0,117 | 0,101 | 0,77 · 0,79 |
| Ink · Pen/Dip Pen | 0,223 (controlo da sessão 0,283) | 0,304 | 0,62 · 0,51 |
| Ink · Brush/Brush | 0,239 | 0,187 | 0,76 · 0,82 |
| Marker · Liner | 0,016 | 0,016 | 0,98 · 0,98 |
| Marker · Bullet | 0,016 | 0,016 | 0,98 · 0,98 |
| Watercolor · Round (controlo) | 0,926 | 0,673 | — |

**A tinta e o marcador escurecem IGUAL sobre o molhado e sobre o seco**: é uma lei de CAMADAS (a de cima filtra a de
baixo — o marcador é a multiplicação pura: (20,0,15) contra (191,0,18)⊙(23,101,179)/255 = (17,0,13)), não de mistura
na água. Os flags dos 281 presets dizem o mesmo: `use_glazing` é verdadeiro na aquarela Round (0,93) e falso no Ink
Bullet (0,12) e no Marker Bullet (0,016) — o escurecer é da FERRAMENTA, não da opção.

## §3b — O oráculo, terceira corrida: a MESMA tinta (fixtures `mbazulmolhado`, `inkazulazul`, `mblaranja`)

| cruzamento (Rebelle) | fracção multiplicada (sRGB, por canal) |
|---|---|
| marcador azul sobre o mesmo azul (molhado) | 0,48 · 0,49 · 0,51 |
| caneta azul sobre o mesmo azul (seco) | 0,69 · 0,55 · 0,46 |
| marcador amarelo sobre laranja (`d` 0,30) | 0,67 · 0,89 · 0,61 |
| marcador vermelho sobre azul (`d` 0,71) | 0,98 |

A mesma tinta também filtra (metade) — a «diferença de pigmento» sozinha dava 0 e o marcador ficava em 0,34.
⇒ `vela_da_camada = Glaze × min(1, 0,5 + 0,7·d)` (`colorops`). (A cena «marcador azul sobre o azul SECO» saiu em
branco: o 1.º envio da sessão foi antes de a tela existir.)

## §4 — A decisão (padrão-ouro, delegada; o dono pode vetar)

- **Uma lei do pincel: o Glaze `g ∈ [0, 1]`** — a fracção da camada de cima que FILTRA a de baixo (`0` = a lei da obra,
  ao byte; `1` = multiplica). Guardado por célula num plano `vela` que pousa com a tinta (como o `relevo`), lido pela
  lei do ecrã de cada célula — trocar de pincel não muda a tinta que já está na tela.
- **No pouso**, a tinta de outro pigmento que estava MOLHADA na célula assenta (na fracção `g × outra_tinta` — §5 —,
  preservando o ecrã) e a tinta nova forma uma camada por cima: a multiplicação acontece UMA vez por camada, nunca por
  carimbo (o que levava ao preto).
- **A mistura do pincel NÃO entra**: a tabela do §2 mostra que nenhuma mistura reproduz o cruzamento sem a velatura
  composta, e o §3 que o Rebelle não depende do molhado. Um knob de mistura por pincel não mudaria nenhum dos cinco —
  seria um controlo sem leitor que decide. A «Pigment mixing» continua da obra.

## §5 — A lei como ficou, e o que ela custa

- **Só onde há camada por baixo**: o Glaze de um carimbo sobre o papel limpo é 0 — com ele em toda a tinta o recto
  escurecia ao secar (cobertura 0,90 → 0,93; portão `o_traco_sozinho_nao_muda_com_o_glaze`, ao byte).
- **A fracção fixa-se no POUSO**, contra a camada de baixo de então (o plano guarda `vela_da_camada`, não o Glaze):
  lida da cor de agora, caía 0,94 → 0,77 enquanto o assentado absorvia o filme e o cruzamento do marcador clareava
  19 → 33 ao secar; fixa, 21 → 21.
- **O assentar guarda o ecrã** pela inversa (Newton) com o limiar próprio `CEGO_COM_GLAZE` = 0,01 onde há Glaze: na
  grelha de 300 estados, a lei da obra muda > 1,5 níveis em 41 (pior 42,6); Glaze 0,5 em 11 (6,4); Glaze 1 em 15
  (12,7). Sem o palpite fechado do produto e com o limiar de antes: 70 e 33. Há SEMPRE uma cor que guarda o ecrã
  (força bruta < 0,4, `sonda_alcance`).
- **O azul que sobe ao secar** nas tintas molhadas (+5 a +24 no canal B do cruzamento) é o ESCOAMENTO do traço
  vizinho, ainda molhado (com nivelamento, capilaridade e gravidade desligados: +1 a +5; a lei de hoje sem Glaze
  +5). Nos marcadores (água 0,1) ≤ 5. Física da tinta molhada, não a lei — aberto no handoff.
- **Custo** (`sonda_o_custo_do_glaze`, mínimo de 9 intercalados, Wet Liner, o vermelho que cruza o azul): o traço
  15 ms sem Glaze, 42 ms com Glaze 1 (99 ms antes de `OUTRA_TINTA_MINIMA` = 0,1 — a ponta suja fazia de cada
  carimbo uma camada); 120 quadros de água 483 → 526 ms (+9 %).
- **O formato**: o Glaze é o knob `glaze` no mapa `engine` do preset, por NOME — um preset sem a chave lê 0 (a lei
  da obra); `SCHEMA` fica 1 (o leitor ignora chaves que não conhece: um build antigo pinta-o sem Glaze, não o
  recusa nem o lê errado). A macro leva-o (o estado inicial é o preset capturado).

## §6 — O halo do Ink Brush e o «azul por cima» (prompt 42b, sondas `sonda_o_perfil_do_cruzamento` e `sonda_as_metricas_das_variantes`)

**Mecanismo (medido célula a célula, Glaze 1 contra Glaze 0):** o assentar do Glaze levava 100 % do azul molhado para
`sett` mesmo onde o carimbo quase nada pousava — na orla mole da coluna (dureza 0,1) a célula `x 279` recebia ~2 de
vermelho e assentava 1 083 de azul, MAS ficava com a água toda (filme 0,63, suspenso 0). Essa água LIMPA escoava para o
azul vizinho ainda molhado e levava-lhe o pigmento: `x 277` (y 488) suspenso 440 → 265 → 115 em 0 / 60 / 120 quadros,
assentado seco 184 contra 750 do azul (o halo, +72 de luminância). Com Glaze 0 a mesma célula tem suspenso 1 084 e
nada escoa. Refutado: a difusão (`extDiffusion` 0 → halo +75) e o arrasto (só desloca o azul ~6 px para baixo, igual
com Glaze 0).

**Lei:** a camada que vai para baixo leva a sua água (`film × (1 − dm/susp)`, `trail/transfer.rs`). Halo +72 → sem
halo branco; Glaze 0 ao byte. A orla fica com uma linha azul mais escura (o azul molhado que entra na célula agora seca).

| recusado (medido) | porquê |
|---|---|
| assentar pela cobertura do carimbo (`× in_a`) | halo +9, mas o resto do azul repinta-se de vermelho a cada carimbo: Ink Brush 0,51, Marker Bullet 0,18 |
| rampa `min(1, in_a/A0)` + água | cura também a linha da orla; custa aos marcadores (A0 0,01: Liner 0,060, Bullet 0,038; 0,05: 0,072/0,070) |
| o arrasto não move a camada de baixo | congela as estrias de cerda do azul (riscas horizontais) |
| vela sem média pela massa | muda os outros quatro (Dip Pen 0,217 → 0,128, Ink Bullet 0,113 → 0,055) |

**Receitas:** Ink Brush Glaze 1 → **0,8** (a fracção multiplicada que o oráculo mediu, §3: 0,76 · 0,82): cruzamento
0,257 (Rebelle 0,239), miolo (97,40,68) contra (95,37,55). Marker Liner dureza 0,9 → **0,55**, medida à lei de
largura do próprio preset (raio 9): borda 0,721 (Rebelle 0,716), textura 2,78, cruzamento 0,043.

## §7 — A 3.ª camada da mesma tinta clareava (smoke do dono 16/10; sondas `sonda_a_terceira_camada`, `sonda_a_celula_que_clareia`)

**O defeito** (capturas do dono): três–quatro traços do MESMO vermelho; onde o último passa sobre a zona escura de duas
camadas aparece uma cunha/faixa mais clara de borda reta, com um «reflexo» quase da cor pura ((188, 50, 50) dentro de
(166, 34, 34)) — só com a tinta de baixo ainda molhada. Reproduzido nos 5 pincéis de Glaze (cena `camadas`: dois
horizontais meio sobrepostos, um vertical, uma diagonal; esperas por traço; luz linear por número de camadas).

**Dois mecanismos, medidos célula a célula:**

1. **A mesma tinta molhada nunca era camada.** O assentar no pouso só corria para OUTRA tinta (`outra_tinta` ≥ 0,1); a
   mesma tinta molhada fundia-se na água e a fracção não subia: o 3.º traço molhado sobre o 2.º não escurecia NADA (Dip
   Pen, `[1800, 0]`: as 252 células de 3 camadas antes e depois do traço, ecrã (170,7; 37,8) igual) e o escoamento,
   que só pode tirar filme de uma célula saturada (a cobertura satura em 3 000), deixava-a mais clara que as de 2
   (+1–7 % na mediana). Com nivelamento, capilaridade e gravidade a 0 o clarear some — refuta o arrasto, o re-molhar e
   a recolha (cada um desligado, o clarear fica). Um traço molhado sobre molhado (espera 0) nem a 2.ª camada escurecia
   — o Rebelle escurece igual molhado e seco (§3, §3b `mbazulmolhado`).
2. **A fracção sobre um RESTO de assentado.** `vela_aqui` pedia só `sett > 0`: o arrasto do traço novo leva a orla de
   uma camada (2 473 → 6,4) e a célula de massa < 1 contava como camada. O traço pousava ali com a fracção inteira; ao
   secar, a inversa do assentar guardava o ecrã com um assentado BRANCO ((254, 252, 252)) e a célula secava mais clara
   que a tinta: 2 395 · 3 651 · 2 992 células (Dip Pen · Ink Bullet · Marker Bullet), até +15,5 níveis — o reflexo, e a
   borda reta é a frente do arrasto.

**A lei** (`trail/transfer.rs`, `colorops::vela_sobre`): o primeiro toque de um traço numa célula que estava MOLHADA
antes dele assenta a tinta de lá — é de OUTRO traço — também a mesma tinta (`Trail::por_assentar`, tirado da grade no
1.º transfer do traço); e a fracção pesa a cobertura da camada de baixo até `COBERTURA_QUE_FILTRA` 0,5 (a tabela no
código: 0,5 cura 2 395 → 1 sem mexer no cruzamento; 1 custava ao Ink Brush 0,257 → 0,311). ⛔ «As células que o traço
ainda não TOCOU» em vez de «as molhadas antes dele» falha com a Live Water: a tinta do próprio traço corre à frente e a
orla vela-se a si mesma (os três azuis sozinhos, Glaze 1 contra 0, o pior canal: Dip Pen 119 · com o retrato 13 · o
motor de antes 39; Marker Bullet 35 · 49; Ink Brush 14 · 14).

| luz linear, cena `camadas` `[1800, 0, 0]` | 1 | 2 | 3 | 4 | miolo de 1 camada, p95 |
|---|---|---|---|---|---|
| Marker Bullet antes | 0,1396 | 0,0937 | **0,0937** | **0,0948** | 0,1500 |
| Marker Bullet depois | 0,1396 | 0,0937 | 0,0799 | 0,0730 | 0,1430 |
| Dip Pen antes | 0,1396 | 0,1000 | **0,1067** | 0,1033 | 0,1526 |
| Dip Pen depois | 0,1396 | 0,1000 | 0,0960 | 0,0950 | 0,1406 |

Depois, as 5 × {seco, molhado sobre molhado, 60/60, `[1800,0,0]`, `[1800,1800,0]`} × 2 cenas são monótonas e o molhado
dá o mesmo que o seco (Marker Bullet 0,1396 → 0,0937 → 0,0799 → 0,0730 em todas). O Dip Pen (Glaze 0,8) converge para o
ponto fixo `(1−m)·u / ((1−m) + m·(1−u/255))` já na 3.ª camada. Cruzamento das mídias: Ink Bullet 0,113 → 0,104 (Rebelle
0,117) · Dip Pen 0,217 → 0,208 (0,223) · Ink Brush 0,257 · Marker Bullet 0,025 → 0,015 (0,016) · Marker Liner 0,043 →
0,037 (0,016). Glaze 0 ao byte. Custo (carga ~20, só relativo): o traço que cruza a MESMA tinta molhada 18 → 31 ms com
Glaze 1 (outra tinta 20 → 28 ms); a água igual.

Portões `a_lei_da_tinta::a_terceira_camada_da_mesma_tinta_nao_clareia` (vermelho no `2e43def369`: Marker Bullet
0,1396 · 0,0937 · 0,0937 · 0,0948) e `o_primeiro_toque_nao_assenta_o_proprio_traco_com_agua_viva` (Dip Pen ≤ 25 níveis).
Mutação 5/5 sangram (sem a cobertura · sem o primeiro toque · o retrato nunca tirado de novo · o primeiro toque com
`outra_tinta` · o retrato = todas as células).

| recusado (medido) | porquê |
|---|---|
| a fracção × cobertura inteira (`COBERTURA_QUE_FILTRA` 1) | Ink Brush 0,257 → 0,311, Liner 0,037 → 0,045; o pálido igual ao de 0,5 |
| só a cobertura, sem o primeiro toque | o clarear da 3.ª camada fica (Dip Pen 0,1000 → 0,1067) |
| só o primeiro toque | os 2 397 · 3 393 · 3 074 assentados pálidos ficam |

**Aberto:** o resto (Marker Bullet 30 células, +5,1) é a orla de cima de uma camada SECA que o arrasto do traço novo leva
(o arrasto move o assentado, lei do port, também com Glaze 0); o mesmo traço sobre si próprio não faz camada.
