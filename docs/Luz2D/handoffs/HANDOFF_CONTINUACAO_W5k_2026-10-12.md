# Handoff de continuação — Luz 2D, W5k: a OPACIDADE contínua (2026-10-12)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D`, base `main`
> `8f20c60de`; W5d–W5j por integrar. O estado da W5j (o dono de cada cópia, o Unlit do Flip, os cartões
> «Light Occluder»/«Unlit», os itens 6–7 desenhados, os vermelhos anteriores): [W5j](HANDOFF_CONTINUACAO_W5j_2026-10-11.md).

## 1. O smoke do dono (12/10) e o que a medida achou

«Ao reduzir o Opacity do Light Occluder do objeto azul, o laranja da sombra desapareceu — nem sempre.»
O «nem sempre»: na cena 7 as duas bolas de CIMA são UM objeto Flip (uma opacidade só); as de baixo, duas
imagens. Mas a medida achou um defeito maior. Sonda `sonda_a_opacidade_do_oclusor`
(`ph2d-app-light2d/tests/it/a_opacidade_do_oclusor.rs`, `--ignored`; cena 7, as duas imagens juntas, 40
quadros, `DEGRAU_PADRAO`, RTX 5060 Ti):

| opacidade do azul | face acesa do azul (R G B) | anel do azul | anel do laranja | sombra R/B |
|---|---|---|---|---|
| 1 | 2,70 2,98 2,71 | 2,04 2,01 1,81 | 2,12 1,82 1,56 | 1,196 |
| 0,999 | **0,0003** 0,0002 0,0002 | 2,02 1,80 1,58 | 2,11 1,72 1,46 | 1,222 |
| 0,99 | 0,038 0,034 0,030 | 2,03 1,80 1,59 | 2,11 1,72 1,46 | 1,222 |
| 0,9 | 0,33 0,29 0,25 | 2,10 1,87 1,65 | 2,11 1,72 1,46 | 1,221 |
| 0,5 | 1,65 1,47 1,29 | 2,43 2,16 1,90 | 2,12 1,73 1,47 | 1,216 |

⇒ Dois saltos em `α = 1`, da MESMA família: (a) o corpo a 0,999 fica PRETO (a 1 mostra a luz do exterior
mais próximo — F3 + miolo; abaixo de 1, a transmitida por dentro, ≈ 0); (b) o rebote é tudo-ou-nada (o azul
deixa de devolver o teal: o G/B do anel cai 10 %).

## 2. A causa (mapa, file:line)

«Interior de opaco» é IGUALDADE a opacidade 1 nos dois lados. Placa: `tau <= 0` (`c0_dono.wgsl:13-19`,
`tau_dentro = 1 − opacidade` do dono; `albedo` só `opacidade >= 1`) em `c3_direta.wgsl:591-608`
(`onde_calcular`/`remapeado`, a escolha é SUBSTITUIÇÃO), `c3_pe.wgsl:54,75,202`, `c3_preenche.wgsl:26`,
`c3_miolo.wgsl:59-78` (classifica, via o alfa `alfa_do_miolo` `c3_direta.wgsl:901`), `c4_rebote.wgsl:37` (x⁺).
Referência: `opaco()` `ph2d-light2d-ref/src/lib.rs:206` (`transmissao <= 0`), `fora_dos_opacos` `:213-244`,
`no_miolo` `:333-352`, `miolo.rs:60-92`, `rebote.rs:65` (`reflete`). ECS: `coleta.rs:169-170, 201-202, 229`.

## 3. A lei nova (decisão técnica; o dono pediu «descubra e corrija»)

Com opacidade `α` (dono do texel): **mostrado = mix(transmitida, exterior/miolo, α)** e **albedo efetivo =
α · a**. Em `α = 1` é a lei de hoje BIT A BIT (o miolo dos opacos fica como o dono mandou); em `α → 0`, o
corpo some. O plano §2.4 («o translúcido não reflete») muda: escreva o porquê com esta tabela.

Ordem (red-first): (1) a sonda vira portão — face, anel do azul e sombra são CONTÍNUOS em `α` (0,999 dentro
de ~1 % de 1) e `α = 1` byte-idêntico à base; (2) a referência primeiro (`opaco` → «corpo», o `fora` e o
miolo para todo dono com `α > 0`, a mistura por `α`; `reflete` com `α·a`), com os G9 da referência; (3) a
placa: o predicado geométrico passa a «dono» (`tau < 1`), a mistura final por `α` (`select` quando
`tau <= 0`, para o bit), o albedo `α·a` no `c0_dono`, o x⁺ do `c4_rebote` para todo dono; (4) a coleta
(`albedo`/`reflete` sem o corte em 1). Reescrever contra a lei nova: `o_translucido_nao_reflete_na_placa`
(g9 placa), `o_translucido_e_o_que_brilha_nao_refletem` (g9 ref), e conferir
`um_opaco_translucido_fora_da_tela_deixa_passar_a_sua_parte` e `o_interior_de_um_opaco*`. ⚠️ Não
confirmado: o `tau` com DOIS donos sobrepostos (o `c0_captura` faz o produto).

## 4. Aberto, herdado

Os da [W5j §3](HANDOFF_CONTINUACAO_W5j_2026-10-11.md) (itens 6–7, o Unlit do vetor e das formas do Motion,
a pergunta do vão da cena 8) e o censo dos cartões (22 na dívida).

## Smoke do dono (o que já dá para ver)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
2. Clique em «Disco grande (imagem)» (a bola azul de baixo); no cartão «Light Occluder», baixe «Opacity» de 1 para 0,99.
3. HOJE (o defeito): a bola azul fica quase preta. Depois da W5k: quase igual a antes, e escurece/clareia aos poucos.
