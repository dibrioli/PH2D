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

## 2. Rodadas (`target/w6g/{f3,caixa,borda}.log` = rec. 1 · `*_r2.log` = rec. 2 · `custo_*.log` = a bisseção)

Rec. 1 = o §1 tal como escrito (marca de bloco, `c3_cantos` com o prólogo numa thread, `um_lado` só no texel que uma
face corta). Rec. 2 = o prólogo em 9 threads, o θ₁ da `V2` deslocado de 8 (o zero da textura nova sai na 1.ª leitura do
`meio`) e o `um_lado` em TODA mistura de duas faces.

| variante | cantos (≤ 1) | perfil (≤ 0,06 a ≥ ¼) | F3 (≤ 0,062) | (152, 245) (0,142 ± 0,01) | (a) / (b) / anéis / bits | C3 cena 5 / 6 (≤ +5 %) | borda (ms) |
|---|---|---|---|---|---|---|---|
| lei W6d | 7 | ✗ | 0,0531 | 0,140 | 1 (−0,176) / 37 / 0 / 0 | 3,495 / 2,251 | 0,286 |
| cunha | 6 | — | 0,0689 | 0,173 | 1 / 37 / 0 / 0 | 3,536 / 2,290 | 0,327 |
| 10 + 9, rec. 1 | 1 ✓ | ✓ | 0,0371 ✓ | 0,1238 ✗ (`um_lado` não entrou) | 1 / 38 / 0 / 0 ✓ | 3,940 / 2,569 (+13 % / +15 %) ✗ | 0,602 |
| **10 + 9, rec. 2** | **1** ✓ | **✓** (0,03; −0,23 só NO vértice) | **0,0296** ✓ | **0,1448** ✓ (sonda 2 = 108) | **1 / 38 / 0 / 0** ✓ | **3,926 / 2,560 (+12,3 % / +13,7 %)** ✗ | **0,607** |

**MORRE na 2.ª reconstrução** (two-strikes) SÓ pelo custo. Os outros quatro itens passam.

- **O (152, 245)** (rec. 1 → 2): a sonda 2 deu `8` na rec. 1 — o ramo nem entrava: as distâncias às faces são 0,70 e
  0,92 texel, o texel está INTEIRO dentro (é o eixo medial que o corta, não uma face; a decomposição 0,29 / 0,71 do
  gabarito é pela face mais perto, não pela área cortada). Com o `um_lado` em toda mistura entra (+100) e a luz vai a
  0,1448. A normal sai (−0,25; −0,97) contra a verdade (−0,262; −0,965).
- **O custo — a bisseção** (só a régua do custo, o shader mexido FORA do git e reposto; `custo_d1..d4.log`):

| diagnóstico | borda cena 5 / 6 (cunha: 0,327 / 0,318) | C3 cena 5 / 6 |
|---|---|---|
| rec. 2 | 0,607 / 0,586 | +12,3 % / +13,7 % |
| D1 — o `c3_cantos` sai NA ENTRADA | 0,576 / 0,562 | — |
| D2 — D1 + o `c3_borda` sem as marcas | 0,563 / 0,551 | — |
| D4 — D2 + sem o ramo «anel cheio» do `pela_cunha` | 0,543 / 0,536 | — |
| **D3 — D2 + SEM o despacho do `c3_cantos`** | **0,346 / 0,334** | **+4,7 % / +6,8 %** |

⇒ **o custo é o DESPACHO do `c3_cantos`, não o trabalho dele** (o kernel vazio custa ~0,21 ms; a cobertura é
515 e 63 blocos de 245 760 — 0,2 % / 0,03 %, `BLOCOS ·` no log). A minha hipótese da rec. 2 (as 9 leituras em série
do prólogo) ficou REFUTADA: nada mudou. O suspeito por medir é a transição de estado das duas texturas de 126 MB
(`V` escrita→amostrada e `V2` amostrada→escrita a cada quadro, 5120 × 3072 rgba16f) e o lançamento de 245 mil grupos.
A fina também sobe (cena 5 2,383 → 2,496): com a borda em +0,02 a cena 6 ainda sai a +6,8 %.

## ⛔ Recusas MEDIDAS (W6g — não reconstruir sem ler a §2)

| Recusa | Porquê |
|---|---|
| A 10 com `V`/`V2` em textura da CAPTURA inteira e um despacho na captura inteira | O despacho sozinho custa ~0,21 ms com o kernel vazio (D1 × D3), seja qual for o trabalho. |
| O prólogo do `c3_cantos` em paralelo como cura do custo | Medido: 0,602 → 0,607. |
| O `um_lado` só no texel que uma face CORTA | O (152, 245) não é cortado por nenhuma face (0,70 / 0,92 texel); o ramo não entra. |

## 3. Estado do código (padrão INALTERADO)

`borda_cantos: false` e `canto: Canto::DESLIGADO` no padrão; o código da rec. 2 fica (os overrides apagam os ramos):
- `Canto::um_lado` (override `CANTO_UM_LADO`; `comuns` passa a `[_; 24]`) — no `LIGADO`;
- o `c3_borda` escreve a `V` só no bloco 8×8 com parcial de vértice (o resto do bloco guarda o velho; a marca
  `SEM_BLOCO` na origem), o `c3_cantos` só espalha nos blocos com marca à volta, o `meio` consulta a origem
  (`bloco_valido`); a `V2` guarda θ₁ + 8 (`V2_THETA`) — ⚠️ o f16 a 8–11 tem passo de 0,45° contra ~0,1° no θ cru: é
  uma perda de precisão do ângulo por medir se a 10 voltar com texturas (a rec. 2 do F3 mudou de 0,0325 para 0,0296
  entre as duas codificações);
- a sonda 2 ganha `+100` (entrou nas normais de um lado), `+200/+400` (a face 1/2 não se mediu), `+800` (iguais);
- **instrumento:** a borda num passe de cálculo próprio com carimbo (`tempos::C3_BORDA`, `TemposMs::c3_borda` =
  parcela do `c3_preenche`, que continua a contá-la — as réguas antigas não mudam de sentido); a régua do custo
  imprime `[borda …]` e o censo `BLOCOS · cena n · cobertos x/y`.

Portões por nome com o padrão (`target/w6g/portoes_*.log`): gpu 9/9 (`c3_grade_fina`, G5, G10) · app 15/15
(`o_interior_do_opaco`, `o_campo_da_borda`, `a_borda_sem_pontos_pretos`, `g6_ponto_neutro`, `a_vista_renasce`,
`os_tetos_da_placa`).

Tocam `ph2d-light2d-gpu` (`config.rs`, `tempos.rs`, `solver_c3.rs`, `solver_c3_constantes.rs`, `wgsl/c2_rc.wgsl`,
`wgsl/c3_borda.wgsl`, `wgsl/c3_direta.wgsl`, `tests/it/c3_grade_fina.rs`) e `ph2d-app-light2d`
(`tests/it/medida_luz2d_w6d.rs`, `tests/it/medida_luz2d_w6f.rs`). Sem contrato congelado, sem schema, sem shell. Quem
constrói `Canto` sem `..` parte com o campo novo (nenhum fora das réguas).

## 4. ABERTO (por ordem — substitui o §10 da W6f)

1. **A 10 sem textura da captura inteira** (o kill-criterion do §1 vale tal e qual; os quatro itens de qualidade já
   passam com a rec. 2): primeiro MEDIR o que custa no despacho vazio (a transição das texturas × o lançamento: um
   `c3_cantos` vazio sem bindings de textura grande); depois a forma esparsa — a cunha dos parciais de vértice numa
   LISTA (buffer com contador atómico, o `c3_borda` acrescenta), um índice por bloco 8×8 (1 MB, limpo por
   `clear_buffer`) para o `meio`, e o `c3_cantos` em despacho INDIRETO só sobre os blocos listados. Sem `V`/`V2` de
   126 MB some também a questão do θ deslocado.
2. A fina da 10 + 9 (+0,11 ms na cena 5): medir depois do item 1, com a borda já em +0,02.
3. A família (b) (37–38): da W5g, fora.

## 5. Smoke do Enio

Nada mudou na tela: a cura está desligada. O smoke da W6d continua valendo
(`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=6 cargo run -p ph2d-host-desktop --profile smoke`).
