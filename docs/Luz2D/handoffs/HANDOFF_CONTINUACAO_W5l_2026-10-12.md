# Handoff de continuação — Luz 2D, W5k fechada: a OPACIDADE contínua (2026-10-12)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D`, rebaseada sobre
> `main` `9ea782138` (12/10); W5d–W5k por integrar. Esta janela: `2ef1074ea` (a lei) · `45447c188` (a placa e os
> portões) · o deste handoff. A tarefa e o mapa da causa: [W5k](HANDOFF_CONTINUACAO_W5k_2026-10-12.md) §1–§3. O
> porquê para o dono e a régua: [BUGS #5](../BUGS_luz2d.md) e o [plano §2.4](../01_plano.md).

## 1. A lei (gabarito = placa = coleta)

Com `α = 1 − ∏τ` dos corpos que contêm o ponto: **mostrado = `mix(transmitida, exterior/miolo, α)`**, **albedo
efetivo `α·a`**. «Corpo» = `τ < 1` (todo emissor e todo oclusor de opacidade > 0). Em `α = 1` é a lei de antes
**bit a bit** (provado: `cmp` da fluência da cena 7 a `α = 1`, base × W5k, após cada mudança da placa —
`sonda_grava_a_fluencia_a_alfa_1` + `PH2D_ALFA1_SAIDA`).

- **Gabarito** (`ph2d-light2d-ref`): `opaco()` → `corpo()`; `alfa_em`; `linha` = `fora_dos_opacos` +
  `linha_sem_projetar` (a transmitida); `no_miolo` mistura por `α` (o resto em `no_miolo_do_corpo`); o miolo e o
  `fora` para todo corpo; `reflete` para todo corpo com `min(α·a, A_MAX)`.
- **Coleta** (`ph2d-light2d-ecs`): o albedo vai CRU a todo oclusor que recebe luz (sem o corte `opacidade ≥ 1`);
  a opacidade multiplica-o no solver (`c0_dono.wgsl`: `opacidade * albedo`).
- **Placa** (`ph2d-light2d-gpu`):
  - `num_corpo` (`c2_rc.wgsl`) = o predicado GEOMÉTRICO (`τ < 1`) nos 17 sítios da C3 (preencher, pé, face,
    `x⁺` do rebote); a marcha e as cascatas (transporte) continuam no `τ`.
  - A fina calcula também a transmitida no próprio ponto do texel translúcido remapeado (`c0_tau = τ`) e
    `guardar_fina` mistura-a (sem miolo) ou grava-a na textura nova `luz2d/tex/miolo_transmitida` (rgb + `α`),
    que o `miolo_final` mistura (`select` no `α = 1`: o bit). Sem miolo a ligação 6 recebe uma 1 × 1.
  - `peso_de_fora` (`c0_em`): vista de FORA, a sonda num translúcido pesa o τ dela (só peso).
  - A holográfica: o «+1» pesa a transmissão `τ_adiante/τ_aqui`; o «partir de dentro» multiplica pelo τ do
    TEXEL; o borrão em cruz pesa `min τ / max τ`.
  - `visibilidade_na_face`: num translúcido a metade de TRÁS da abertura marcha-se a partir do ponto
    espelhado DENTRO (`P − afasta`): a partida conta a entrada × τ. De `Q` (fora) o raio quase rente a uma
    face convexa nunca entra — era o salto no terminador.

## 2. Medido (RTX 5060 Ti, `dev`)

| régua | antes | depois |
|---|---|---|
| cena 7, face acesa do azul (R G B) a 0,999 × 1 (`a_luz_e_continua_na_opacidade_do_oclusor`) | 0,0003 × 2,70 (PRETO) | 2,692 2,973 2,706 × 2,696 2,977 2,710 |
| idem, anel do azul G/B a 0,999 | −10 % | +0,03 % |
| disco translúcido × gabarito no miolo fundo, COM miolo, pior · rms (`o_translucido_da_placa_e_o_do_gabarito`) | ½: 0,102 · 0,084 | 0,9: 0,034 · 0,019 · ½: 0,015 · 0,008 (o opaco: 0,039 · 0,022) |
| idem SEM miolo, rms | 0,9: 0,169 · ½: 0,106 | 0,036 · 0,012 (o opaco: 0,044) |
| sala do g9 a 0,99 × 1, pior ponto: gabarito · holográfica · clássica | — | 0,14 % · **4,6 %** · **7,0 %** (§4.1) |

⛔ **Recusas MEDIDAS:**

| tentativa | medido | porquê não |
|---|---|---|
| reescalar a sonda translúcida para o meio do ponto (`v·τ_ponto/τ_sonda`) no `c0_em` e no borrão | a sonda do gabarito NÃO mudou (4 casas); a sala a 0,99 divergiu para 10⁴ | a sonda dentro de um corpo que reflete apanha a própria emissão sem o τ: dividir realimenta o rebote |
| o «+1» com `mix(f_u, f_u+1, τ_adiante)` e o «partir de dentro» × τ da SONDA lida | a face a 0,999 −14 % no G/B | o texel de fora lia o interior × 0,001 (o −43 % da W5c) |

## 3. Portões

Novos/reescritos: `a_luz_e_continua_na_opacidade_do_oclusor` (app; vermelho na base: a face a 0,0003) ·
`o_interior_e_continuo_na_opacidade` e `um_vidro_mistura_a_transmitida_e_o_exterior_pela_opacidade` (gabarito;
Lipschitz em α relativo à cena VAZIA — atrás do corpo a 0,999 passa 0,1 % e a sombra quase preta sobe 3 %: físico)
· `o_translucido_reflete_a_sua_parte_e_o_que_brilha_nao` (gabarito) · `o_translucido_da_placa_e_o_do_gabarito`
(placa, com e sem miolo) · `quem_reflete_e_o_corpo_que_recebe_luz` (coleta) · `o_translucido_reflete_a_sua_parte_na_placa`
(⛔ VERMELHO DE PROPÓSITO, §4.1). `um_opaco_inteiro_na_grade_nao_conta_duas_vezes`: a barra ganhou `NaoRecebeLuz`
(a régua é `ref::fluencia`, sem rebote, e o translúcido agora reflete). Sondas: `sonda_o_translucido_contra_o_gabarito`,
`sonda_o_translucido_na_sala`. ⚠️ O portão da cena do dono monta UMA bancada (`varrer`): o solver nasce em ~14 s
em `dev` (a compilação; 39 quadros = 0,5 s) — o piso de todo teste da família com `LuzDoQuadro`.

Gate batched: §5.

## 4. ⚠️ ABERTO, na ordem

0. ⛔ **REGRESSÃO da W5k: `g3_ceu_vidros_sobrepostos_e_alcance_finito_por_degrau` (Alto) 0,0074 → 0,0100** (tecto
   0,0082; Médio e Baixo não medidos depois — o teste para no 1.º). Ablação na mesma sessão (cada uma sozinha
   desligada): o peso τ do `c0_em` → 0,0121 (ajuda); o borrão antigo 0,0100; o «+1» antigo 0,0100; a face opaca 0,0102
   ⇒ não são as curas da placa. Hipótese (NÃO medida): a régua conta «fora dos OPACOS», logo o interior dos vidros —
   que agora copia o exterior junto à face, e com CÉU esse exterior já sai baixo nas cascatas (o ABERTO da W5a, citado
   no `o_interior_de_um_opaco_e_o_exterior_mais_proximo`). 1.º passo: partir o RMSE do g3 em «dentro de um vidro» ×
   «fora de tudo». Não suba o tecto.
1. **O rebote de corpos translúcidos que refletem não é contínuo na PLACA** (o gabarito é: 0,14 %). Sala do g9 com
   as quatro paredes a 0,99: a holográfica muda 4,6 · 3,9 · 4,4 · 3,5 % nos pontos (o do MEIO da sala também ⇒ é a
   série, não a fronteira perto do ponto); a clássica 7,0 · 5,0 · 6,4 · 3,6 %. Na cena 7 (um corpo, pouco rebote)
   fica em 0,2 %. Suspeitos, por medir um a um com a sonda: (a) as regras da fusão que só conhecem o opaco — o
   «(0, 0) se o intervalo começa dentro» (`marchar_ate`), `MASCARA_DA_FUSAO` e `FRONTEIRA_NA_FACE` chaveados em
   `sup.a ≤ 0`/`τ ≤ 0` (`c2_rc.wgsl` ~200–300) e a fusão T3+ da holográfica; (b) a emissão de quem reflete apanhada
   na PARTIDA por uma sonda de dentro (`luz_ant` do 1.º passo) — 10⁴ quando dividida, ver Recusas. A régua pronta:
   `sonda_o_translucido_na_sala`. Feito ⇒ o g9 do vidro fica verde.
2. **A clássica (não-produto) não tem nenhuma das curas da holográfica** para o translúcido (o «+1» não existe nela;
   as máscaras da fusão são do opaco). Só os portões de comparação a usam.
3. **Dois corpos sobrepostos**: `τ = ∏`, nos dois lados por construção (`alfa_em`, `c0_captura` ×); sem portão.
4. Herdados: os da [W5j §3](HANDOFF_CONTINUACAO_W5j_2026-10-11.md) (itens 6–7, o Unlit do vetor e das formas do
   Motion, o vão da cena 8), o censo dos cartões (22 na dívida), e os vermelhos ANTERIORES (§5).

## 5. Gate batched e binário do smoke

`nextest-impacted` (base = merge-base, load até ~65; a 1.ª corrida bateu no prazo de 1 800 s a 17 516 / 17 541, os 25
restantes corridos à parte, 29 / 30): vermelhos ANTERIORES, iguais na base (abate: os `src/` das quatro crates
voltados a `dde505de25`, os testes de agora) — `so_a_regiao_se_calcula_e_por_dentro_e_o_mesmo_bit` (texel 37,101),
`o_interior_de_um_opaco_e_o_exterior_mais_proximo` (faísca 0,00005), `g5_o_que_a_placa_alocou…` (enchimento de
`luz2d/buf/miolo` `[16,16,32,32,16]`), e os da app já listados no W5j §5 e W5h §3.3 (`as_formas_redondas` ×3,
`o_miolo_das_formas`, `o_rebote_nao_depende_da_oitava`, a posição na grade, cenas 5/6/7 «sem faces», o Flip
3,12e-3). Timeouts só de carga (passam sozinhos): `o_oclusor_fino_faz_a_sombra_do_gabarito`,
`a_sombra_nitida_nao_tem_escada`. **Desta onda:** o g9 do vidro (de propósito, §4.1) e ⛔ **o g3 «céu e vidros
sobrepostos» (Alto): 0,0074 na base → 0,0100 (tecto 0,0082), reprodutível com load 2,9** — §4.0. clippy
`-D warnings` e `fmt --check` nas quatro crates: limpos.

Binário (depois de `45447c188`), 2.ª corrida: `Finished \`smoke\` profile [optimized] target(s) in 0.21s`.

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
2. Clique em «Disco grande (imagem)» (a bola azul de baixo); no cartão «Light Occluder», baixe «Opacity» de 1 para 0,99, depois 0,9 e 0,5.
3. Tem de acontecer: a 0,99 a bola quase igual; ao baixar, ela fica transparente aos poucos — o lado aceso escurece um pouco, o lado da sombra clareia (a luz atravessa), e o reflexo azulado à volta continua, cada vez mais fraco.
4. Errado: a bola fica preta, ou o reflexo azulado some de uma vez.
