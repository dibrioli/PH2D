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

## §4 — A decisão (padrão-ouro, delegada; o dono pode vetar)

- **Uma lei do pincel: o Glaze `g ∈ [0, 1]`** — a fracção da camada de cima que FILTRA a de baixo (`0` = a lei da obra,
  ao byte; `1` = multiplica). Guardado por célula num plano `vela` que pousa com a tinta (como o `relevo`), lido pela
  lei do ecrã de cada célula — trocar de pincel não muda a tinta que já está na tela.
- **No pouso**, a tinta de outro pigmento que estava MOLHADA na célula assenta (na fracção `g ×` a diferença de pigmento,
  preservando o ecrã) e a tinta nova forma uma camada por cima: a multiplicação acontece UMA vez por camada, nunca por
  carimbo (o que levava ao preto).
- **A mistura do pincel NÃO entra**: a tabela do §2 mostra que nenhuma mistura reproduz o cruzamento sem a velatura
  composta, e o §3 que o Rebelle não depende do molhado. Um knob de mistura por pincel não mudaria nenhum dos cinco —
  seria um controlo sem leitor que decide. A «Pigment mixing» continua da obra.
