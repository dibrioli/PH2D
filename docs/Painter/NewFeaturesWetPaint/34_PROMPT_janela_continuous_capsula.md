TROCA DE AGENTE — line/PainterWatercolor: **o Continuous que IGUALA o Spacing pequeno** (smoke do dono 13/10, 3.ª
rodada). Vem ANTES do prompt 30.

Leia SÓ: [31 §4](31_spacing_por_meio_o_pontilhado.md) inteiro · o
[handoff P32](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-13_P32_SPACING_CONTINUO.md) (R1–R9 no topo
do handoff W0/W1). ⚠️ «faça como achar melhor e pare de me fazer perguntas» (dono, 11/10).

## O relato (13/10, duas capturas, vermelho: Spacing pequeno × Spacing grande com a caixa Continuous)

*«Fica diferente do traço com spacing menor e quanto maior o spacing, mais diferente fica, não apenas por reduzir o
pigment (isso é normal) mas por criar algo como RETÂNGULOS e deixar o traço MENOS NÍTIDO.»* A captura grande mostra
blocos com costura vertical a cada dab e tracinhos dentro de cada bloco (o C12 do [26](26_ABERTOS.md), generalizado).

## ⭐ Pedido do dono (13/10, mesma rodada): *«continuous deve estar disponível para os outros modos de pintura também»*

Hoje a caixa APARECE em todo meio mas só se ARMA com o Shape Bristles preso ao dab (`BrushSpec::drags_bristles`), e
esmaece no resto; e a Watercolor não lê o passo (sonda: CV das cerdas 0,037 a todo Spacing — confirme a rota em
`watercolor_accum.rs`). A cápsula é uma lei da PONTA, não das cerdas: ela vale para QUALQUER ponta (a redonda, as
texturas do Shape, a imagem) e tem de agir nos cinco meios — Digital, Impasto, Watercolor, Wet Paint, Composite.
⇒ `drags_bristles` dá lugar a «a caixa ligada e o método espaça dabs» (Drag Dot/Anchored/Grid continuam fora);
`Dependente::ContinuoSemCerdas` passa a esmaecer só onde nada a lê (meça com o censo
`a_linha_esmaecida_e_a_que_nao_age` em cada meio). O dab-cápsula da redonda, recusado no [31 §4.2](31_spacing_por_meio_o_pontilhado.md)
como opção NOVA, entra aqui DENTRO da caixa que o dono pediu — reescreva essa recusa com o porquê.

## O mecanismo (a confirmar por medida no passo 1)

O traço denso (passo 1 px) dá a cada píxel `Σ_k w(h_k)·tile(h_k)` — cada pelo pesado pela queda (`w`, falloff) da
posição DELE relativa ao sub-dab que o carimbou. As duas leis de hoje (`texture/bristles.rs`) erram nisso:
1. **Média (carimbo):** pesa a média da janela pelo `w(o)` do CENTRO do dab (o carimbo multiplica a silhueta no
   píxel) — o peso salta no meio do passo entre dois dabs ⇒ retângulos.
2. **Máximo (Wet Paint):** cada pelo vira um tracinho do comprimento do passo, à intensidade do pico ⇒ blocos com
   tracinhos, menos nítido.

## A lei exata (o dab-CÁPSULA com a textura dentro) — num bloco (§0.10)

Para um dab com passo `D` (texels), sub-posições `off_k` (`k < n`, as de hoje, centradas, meio-abertas):
`cobertura(o) = (1/n) Σ_k w(o − off_k) · tile(q − off_k)` — a queda E as cerdas avaliadas na MESMA sub-posição;
a silhueta do carimbo deixa de multiplicar por `w(o)` nesta rota, e a caixa do dab cresce `|D|/2` ao longo do
passo (a cápsula: `tip::dab_extent`/`stamp_domain`). É, ao bit no regime linear, a soma dos `n` sub-dabs do Spacing
pequeno; só a composição (1 em vez de `n`) difere — a «tinta menor» que o dono aceita.
- **Digital/Impasto/…**: a rota por píxel (`stamp_dab_textured` → `silhouette_at`/`texture::sample`): o kernel
  cápsula quando `drags_bristles`. Memo: `w·tile` por posição NÃO dobrada (o ladrilho tem período 128; um dab de
  raio > 64 cobre mais de um período — chave = raio, falloff/hardness, `D`), e sem memo é `n` leituras por píxel.
- **Wet Paint**: o mesmo dentro do motor — `brush.rs::stamp_row_shaped` lê `fall × texv`; a cápsula entrega o
  `Σ fall·tile / n` como `texv` com `fall = 1` e a caixa crescida. O depósito (`trail/deposit.rs::cell`) subtrai
  o portão de papel do `stamp`: meça se a soma passa o portão como os `n` dabs passavam (é a razão do `Varre::Maximo`).
  As 12 miniaturas de fábrica continuam ao byte (a caixa vem desligada).

## Passos
1. **Medir antes de mexer**: sonda que pinta o MESMO traço a 0,025 (o «alvo») e a 0,25 com a caixa ligada, Digital e
   Wet Paint, raio 20 e 45; a régua é a diferença píxel a píxel contra o alvo NORMALIZADO pela tinta (não só o CV ao
   longo — o CV 0,055 de hoje passou e o dono viu retângulos) + o perfil ao longo de UMA fila de pelo (a costura).
   Confirme os dois mecanismos acima (zere o peso → retângulos somem?).
2. A lei cápsula nos dois motores; portões: a régua do passo 1 ≤ o limiar medido, com CONTROLO (a lei de hoje
   reprova); `o_spacing_continuo_risca_no_digital_e_no_wet_paint` continua; desligada = ao bit.
3. **Custo** (MEDIR_VELOCIDADE, variantes no mesmo processo): a 0,25 com a caixa ≤ o traço a 0,025 sem ela (o motivo
   de existir); recto e curva; com pressão (o raio muda por dab ⇒ o memo falha).
4. Prancha ao dono: alvo 0,025 · 0,25 hoje · 0,25 cápsula, nos cinco meios, com a redonda e com as cerdas.

## Testes — a CADÊNCIA (dono 13/10)
Por item: `cargo-check-narrow.sh` + os testes do item POR NOME. A suíte só no fecho (`PH2D_SUITE=fecho`).

## Fecho
R1 ~150 chamadas · `verificador` · handoff ≤ 8 KB · atualizar o 26 e o 31 · commit por caminho, nunca push · smoke ao
dono em passos numerados (cada meio, redonda e cerdas, Spacing 0,025 contra 0,25 com Continuous: o mesmo desenho).
