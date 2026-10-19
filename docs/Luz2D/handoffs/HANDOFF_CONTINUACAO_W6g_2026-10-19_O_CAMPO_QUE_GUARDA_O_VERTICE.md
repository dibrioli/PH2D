# Handoff de continuação — Luz 2D, W6g: o campo que guarda o vértice, leve (2026-10-19)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `ad5f8fe39c`, por cima da [W6f](HANDOFF_CONTINUACAO_W6f_2026-10-18_A_MISTURA_DAS_FACES.md) (§8–§10); sem rebase
> (ordem do dono: é do integrador).

## 1. A candidata 10 + 9 LEVE — e o kill-criterion da rodada (escrito ANTES de mexer)

O ponto de partida é a W6f §10.1: a 10 + 9 (rec. 2) passou o perfil, os cantos, o F3 e (a)/(b)/anéis/bits, e morreu
em dois itens com o mecanismo medido. A rodada cura os dois:

- **(a) o custo** — `preenche + borda` 0,505 → 1,222 ms (cena 5) e 0,497 → 1,202 (cena 6): o mesmo +0,7 ms nas duas
  cenas, logo de passes que varrem a captura INTEIRA (a escrita de `V` e o `c3_cantos` com a `V2`), não do `meio`
  (que segue a cena). Cura: o trabalho só onde há vértice, ao grão do grupo 8×8 que os dois passes já partilham.
  - o `c3_borda` marca, por bloco, se algum texel dele achou cunha (a `cunha_achada`, partilhada no grupo); só o
    bloco marcado escreve a `V` (todos os seus texels: sentinela ou cunha);
  - o `c3_cantos` lê as marcas dos 3×3 blocos à volta (Chebyshev 2 < 8); sem nenhuma, salta, e da apara 12×12 só
    lê a `V` dos blocos marcados (a dos outros é velha);
  - a `V2` continua válida em toda a captura para o `meio`: um bloco que saiu da cobertura escreve a sentinela UMA
    vez (a marca `suja` por bloco, que nasce 1 em todos: a textura nova não tem sentinela);
  - **instrumento:** a borda passa a um passe de cálculo próprio com o seu par de carimbos (`C3_BORDA`), para a
    régua do custo dizer onde vai o tempo em vez de o deduzir.
- **(b) o texel (152, 245)** — a normal pelo `pe_na_face` CENTRADO sai a 2,4° (a paralela cruza a outra face, pouco
  o bastante para o cosseno não o ver). Cura: no texel que uma face corta (`corta_o_texel`), as normais SEMPRE pelo
  `pe_de_um_lado` (não só quando a centrada as dá «iguais»); a peça entra na `Canto` como `um_lado`.

**Régua** (UM lote, um processo por régua, variantes intercaladas): `medida_luz2d_w6f` (cantos da caixa · o perfil do
iso-0, o `arredondado`) e `medida_luz2d_w6f_f3` com lei W6d · cunha · cunha + 10 + 9 (leve, sem `um_lado`) · cunha +
10 + 9 + `um_lado`; a `medida_luz2d_w6d_borda` com a variante 3 = «cunha + 10 + 9» (a leve, com `um_lado`), resumida
por `python3 -I docs/Luz2D/ferramentas/resumo_borda.py <log>`.

**PASSA** (todos — o kill-criterion da W6f §8, tal e qual, e o alvo do custo):
1. o perfil: |iso-0| ≤ 0,06 a ≥ ¼ texel fino do vértice em todo canto da caixa;
2. cantos da caixa (lei antiga, 4 × 8 casos) ≤ 1 errado;
3. o F3 verde: penumbra ≤ 0,062 · o texel (152, 245) a ≤ 0,01 do gabarito 0,142 · o vértice girado a ≤ 0,01 de
   0,207 · o portão B e o do campo verdes · o interior na difusão sem regressão (`TETO_ERRADOS_DIFUSAO` = 2);
4. (a) ≤ 14 com o pior ≥ −0,25 · (b) ≤ 42 · anéis sem regressão sobre a lei W6d em todo zoom · bits de fora dos
   corpos iguais (0 em 24);
5. C3 ≤ +5 % sobre a lei W6d nas cenas 5 e 6 (o mínimo de 7 intercaladas, no mesmo processo) — e, em particular,
   `preenche + borda` ≤ +0,17 ms sobre ela.

**MORRE**: qualquer um dos cinco. A W6f já gastou duas reconstruções da 10 — esta é a 3.ª tentativa SÓ sobre os dois
itens com mecanismo medido; bater numa 2.ª reconstrução DESTA rodada sem passar ⇒ PARE e prove o modelo
(two-strikes).

Se passar: no padrão `borda_vertice: Some(VERTICE_PADRAO)` + `borda_cantos: true` + `canto: Canto::LIGADO`, e os
portões por nome da W6f §9 (gpu `c3_grade_fina`, G5, G10 · app `o_interior_do_opaco`, `o_campo_da_borda`,
`a_borda_sem_pontos_pretos`, `g6_ponto_neutro`, `a_vista_renasce`, `os_tetos_da_placa`).
