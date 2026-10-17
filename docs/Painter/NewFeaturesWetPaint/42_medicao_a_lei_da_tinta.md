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
