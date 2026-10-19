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

## 6. Rodada 3 — o despacho INDIRETO (o item 1 do §4, por ordem do dono «vamos seguir»)

**O despacho vazio, medido** (`custo_e1.log`, `custo_e2.log`; diagnóstico fora do git):

| diagnóstico | borda cena 5 / 6 (cunha 0,328 / 0,318) |
|---|---|
| rec. 2 | 0,608 / 0,592 |
| E1 — um kernel VAZIO sem bindings, na captura inteira | 0,563 / 0,547 |
| E2 — o `c3_cantos` com as texturas grandes, UM grupo | 0,363 / 0,348 |

⇒ o custo era LANÇAR 245 mil grupos (a taxa de lançamento de grupos, não a transição das texturas de 126 MB).

**A forma (r3):** o `c3_borda_cantos` (entrada própria; o `c3_borda` sem cantos fica como era) marca o bloco, escreve a
`V` do bloco marcado, põe `SEM_BLOCO` na origem da `V` E da `V2` de todo bloco, e o thread 0 do bloco marcado entra na
lista `luz2d/buf/lista_dos_cantos` (declarada na `PassList`; o cabeçalho é o despacho indireto `[n, 9, 1]`, zerado por
`c3_cantos_zera` a cada corrida). O `c3_cantos` corre em `dispatch_workgroups_indirect`: o grupo `(i, k)` cobre o bloco
`k` dos 3×3 à volta do `i`-ésimo listado (dois listados vizinhos escrevem os mesmos bits). A `V2` recebe a sentinela UMA
vez (`c3_cantos_inicia`, na 1.ª gravação) e o θ₁ volta CRU (some a perda de precisão do §3).

| variante (`*_r3.log`, `loadavg` 35–55 das outras linhas) | cantos | perfil | F3 | (152, 245) | (a) / (b) / anéis / bits | C3 cena 5 / 6 | borda |
|---|---|---|---|---|---|---|---|
| lei W6d | 7 | ✗ | 0,0531 | 0,140 | 1 / 37 / 0 / 0 | 3,491 / 2,235 | 0,272 / 0,269 |
| cunha | 6 | — | 0,0689 | 0,173 | 1 / 37 / 0 / 0 | 3,522 / 2,268 | 0,301 / 0,290 |
| **10 + 9 (r3)** | **1** ✓ | **0,03** ✓ | **0,0309** ✓ | **0,1476** ✓ | **1 / 38 / 0 / 0** ✓ | **3,823 / 2,491 (+9,5 % / +11,5 %)** ✗ | **0,490 / 0,481** |

**Ainda morre pelo custo.** Bisseção (`custo_r3_e3.log`, `_e5.log`): SEM o despacho indireto (E3) a borda fica em
0,473 / 0,474; com um direto de 1 × 9 (E5), 0,482 / 0,476. ⇒ o despacho indireto custa ~0,01; os **+0,17 que sobram
estão no `c3_borda_cantos` ou no `c3_cantos_zera` antes dele**, e são iguais com 515 e 63 blocos (não é trabalho). Na
rec. 2 a borda COM as marcas e sem o despacho do `c3_cantos` (D3) custava só +0,02: o que a r3 juntou ao `c3_borda`
foram a 2.ª textura de escrita (`V2`, binding 18), o buffer da lista (binding 19) com o `atomicAdd`, e o despacho do
`zera` (uma barreira antes da borda).

Portões com o padrão (`target/w6g/portoes_r3_*.log`): gpu 9/9 · app 15/15 — o
`os_tetos_da_placa::a_luz_recua_quando_a_placa_recusa_a_memoria` reprovou a `load` 39 e passou SOZINHO a `load` 15
(`portao_recua.log`): é o membro da família de [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) (§93), não o código.

## ⛔ Recusas MEDIDAS (r3)

| Recusa | Porquê |
|---|---|
| O `c3_cantos` na captura INTEIRA (com ou sem saída antecipada) | O lançamento de 245 mil grupos custa ~0,2 ms com o kernel vazio (E1). |
| Atribuir o custo à transição das texturas grandes | E2: um grupo com elas custa +0,035. |

## 7. ABERTO (por ordem — substitui o §4)

1. **Os +0,17 do `c3_borda_cantos`/`zera`** (o kill-criterion do §1 tal e qual; a qualidade já passa): bisseção num
   build, por `W6G_DIAG` temporário no `Borda::gravar` (o molde está em `target/w6g/`): (i) sem o `zera` (a lista não
   reposta: só para o custo), (ii) sem a escrita da origem da `V2` (binding 18), (iii) sem o `atomicAdd` da lista, (iv)
   a entrada `c3_borda` sem cantos no lugar da `c3_borda_cantos` com o resto igual. Suspeito por medir: a 2.ª textura de
   escrita de 126 MB no MESMO kernel de 245 mil grupos. Cura candidata se for ela: a origem da `V2` reposta pelo
   próprio `c3_cantos` dos blocos listados no quadro ANTERIOR (lista dupla), sem a `V2` no `c3_borda`.
2. A fina da 10 + 9 (+0,11 ms na cena 5): depois do item 1.
3. A família (b) (37–38): da W5g, fora.

## 8. A bisseção dos +0,17 e da fina (o §7.1, num build por diagnóstico, tudo FORA do git e reposto)

Só a régua do custo (`PH2D_W6D_CUSTO=1`), cada variante um processo, duas voltas em ordem invertida; diferença à
`cunha`/lei W6d do MESMO processo. Moldes: `target/w6g/diag_{f,g,h,i,j}.py`, lotes `bissecao_{f,g,i}.sh`, logs
`custo_{f,g,h,i,j}_*.log`.

**A borda** (`W6G_DIAG`, `load` 14–16):

| variante | borda c5 / c6 (Δ à cunha) | C3 c5 / c6 |
|---|---|---|
| r3 (controle) | +0,179 / +0,193 · +0,195 / +0,202 | +9,6 % / +10,6 % |
| (i) sem o `zera` (e sem o espalha) | +0,177 / +0,196 | = r3 |
| (iii) sem o `atomicAdd` | +0,184 / +0,192 | = r3 |
| (v) sem a `workgroupBarrier` | +0,170 / +0,202 | = r3 |
| **(ii) sem a escrita da origem da `V2`** | **+0,066 / +0,062** (as duas voltas) | **+5,9 % / +5,1 %** |
| (vi) sem origem nenhuma (`V` e `V2`) | +0,068 / +0,063 | = (ii) |
| (iv) a entrada `c3_borda` com o resto igual | +0,023 / +0,022 | +4,8 % / +6,0 % |

⇒ **os 0,11 são a sentinela `SEM_BLOCO` na `V2` em TODOS os 245 760 blocos** (uma escrita espalhada por bloco numa
textura de 126 MB que o `meio` amostra). A mesma escrita na `V` sai de graça (vi = ii); o `zera`, o `atomicAdd` e a
barreira não custam nada. Com (ii) o `preenche + borda` fica a +0,098 / +0,091 da lei W6d (passa o ≤ +0,17), mas o C3
da cena 5 não: o resto é a fina.

**A fina** (`W6G_FINA` sobre a (ii); Δ à lei W6d):

| variante | fina c5 | fina c6 |
|---|---|---|
| f0 (só a ii) | +0,112 · +0,117 · +0,134 · +0,120 | +0,013 · +0,012 · +0,022 · +0,028 |
| f1 sem `CANTO_UM_LADO` | +0,131 / +0,151 | +0,018 / +0,016 |
| f5 o `meio` sem o `bloco_valido` | +0,082 / +0,094 | −0,011 / +0,000 |
| f6 sem `CANTO_NORMAIS` · f7 sem `CANTO_LUZ` · f8 sem `CANTO_FRACAO` | +0,094 · +0,043 · +0,129 | −0,004 · **+0,062** · +0,013 |
| f2 `Canto` todo desligado | +0,020 / +0,039 / +0,011 | +0,052 / +0,043 / +0,010 |
| f3 o `meio` sem a `V2` | +0,059 / +0,031 | +0,034 / +0,049 |
| **f4 (f2 + f3: o código FORA do shader)** | **−0,004 / +0,006 / +0,014 / +0,003** | **+0,008 / −0,007 / +0,014** |
| **f12 o mesmo código COMPILADO, morto em execução** (`cas.tamanho.x == 12345u`) | **+0,040 / +0,038** | **+0,049 / +0,029** |

- **Contadas** (sonda 9 temporária: chamadas à `visibilidade` cheia, à marcha pela face e passos da outra face, somadas
  na fina): a 10 + 9 muda as marchas em < 0,3 % (cena 5: −323 cheias, +436 pela face, +118 outra face sobre 232 879
  texels com marcha — os mesmos texels em toda variante). **A hipótese «a 10 + 9 marcha mais» está REFUTADA**; o f7
  que encarece a cena 6 também não muda as contagens.
- ⇒ duas parcelas: **~0,035 ms por o código do canto/`V2` EXISTIR no kernel** (f12 × f4, nas duas cenas: código
  gerado — registos/ocupação, por provar com um contador de registos que o wgpu não expõe) e **~0,08 ms de execução
  só na cena 5** (f0 × f12) que não são marchas a mais. Na cena 6 a execução chega a poupar (f0 < f12).

## ⛔ Recusas MEDIDAS (§8)

| Recusa | Porquê |
|---|---|
| Curar a borda pelo `zera`, pelo `atomicAdd` ou pela barreira | (i), (iii), (v): zero. |
| Marcar a `V2` com sentinela em todo bloco a cada quadro | (ii): 0,11 ms — a escrita espalhada nos 245 mil blocos. |
| Atribuir a fina a marchas a mais | Contadas: < 0,3 % de diferença. |
| Curar a fina tirando um `CANTO_*` | f1/f6/f7/f8: nenhum sozinho chega, e o f7 encarece a cena 6. |

## 9. A cura (r4) — a lista DUPLA, a `V2` sempre válida

O que (ii) + f5 medem juntos: a `V2` deixa de ter a marca por bloco. Duas listas em alternância (`L[p]` deste quadro,
`L[1 − p]` do anterior): o `c3_borda_cantos` escreve a `V` e acrescenta a `L[p]` (sem a `V2`, binding 18 fora); um
`c3_cantos_limpa` indireto sobre `L[1 − p]` repõe a sentinela `SEM_CUNHA` nos 3×3 blocos de cada listado do quadro
anterior; o `c3_cantos` sobre `L[p]` escreve os de agora. A `V2` fica válida em toda a captura e o `meio` larga o
`bloco_valido` (uma leitura a menos no laço). Previsão pelas medições: borda +0,066, fina −0,03 ⇒ C3 c5 ≈ +5,3–5,8 %,
c6 ≈ +3,5–3,9 % — **ainda acima na cena 5**; o que falta é a parcela da fina (§8), por isso a r4 cura a borda e mede.

## 10. A r4 medida (`custo_r4_{1,2}.log`, `{f3,caixa,borda}_r4.log`, `portoes_r4_*.log`)

| | borda (Δ cunha) | preenche + borda (Δ lei W6d) | fina (Δ lei W6d) | C3 c5 / c6 (≤ +5 %) |
|---|---|---|---|---|
| r3 | +0,18 / +0,19 | +0,22 | +0,10 / +0,02 | +9,5 % / +11,5 % |
| **r4** (2 corridas + o censo) | **+0,071 / +0,070** | **+0,116 / +0,102** ✓ | +0,080 · +0,105 / −0,026 · +0,001 | **+5,8 · +6,3 · +6,0 %** ✗ / **+3,4 · +4,7 · +3,8 %** ✓ |

- **A qualidade é a da r3, byte a byte:** os logs do F3 e da caixa saem iguais (só muda a posição do aviso «60 s»);
  o resumo da borda dá os mesmos (a)/(b)/anéis/bits (1 −0,176 / 38 / 0 / 0). Era a previsão por construção: o bloco
  que não está entre os 3×3 de um listado tem `SEM_CUNHA` na `V2`, que o `meio` lê como antes lia o bloco inválido.
- Censo novo (`BLOCOS … com cunha`): 152 e 20 blocos com cunha espalhada na `V2` (de 245 760).
- Portões com o padrão: gpu 9/9 · app 14/15 — o `a_luz_recua_quando_a_placa_recusa_a_memoria` reprovou a `load` 40 e
  passou SOZINHO a `load` 40 (`portao_recua_r4.log`): o flake de carga conhecido.

**MORRE pelo custo da cena 5** (+5,8–6,3 %), e é a 2.ª reconstrução desta rodada sem passar (r3, r4) ⇒ PARE
(two-strikes): a r4 fica comitada e **desligada** (`borda_cantos: false` no padrão) — ela só barateia e não muda a luz.
O modelo do que falta já está medido (§8): a borda está a +0,07 e a fina é a parcela que sobra.

## 11. ABERTO (por ordem — substitui o §7)

1. **A fina da 10 + 9 na cena 5** (o kill-criterion do §1 tal e qual; o resto passa). O §8 separa ~0,035 ms de código
   gerado (o código do canto/`V2` presente no kernel, f12 × f4) e ~0,08 ms de execução só na cena 5, sem marchas a
   mais. Cura candidata, a medir ANTES: **o canto num passe próprio** — a fina comum compila SEM o código do canto e da
   `V2` (o f4, que fica à lei W6d), e um `c3_fina_cantos` indireto sobre os blocos listados recalcula com ele os texels
   à volta dos vértices. Pré-condição (uma sonda, sem custo): contar os texels da fina FORA dos 3×3 blocos de um
   listado em que algum ramo `CANTO_*` ou a cunha da `V2` muda a decisão — se forem zero, a partição é exata por
   construção; se não, o raio da vizinhança cresce até serem. Atenção: a fina é da grade FINA e a lista é da captura
   (`ESCALA_DA_CAPTURA`).
2. A família (b) (37–38): da W5g, fora.

## 12. A r5 — o canto num passe próprio (por ordem do dono «siga»; logs `target/w6g/{particao,gatilho,exato_r5*,custo_r5*,pecas_*,duas_*}.log`)

**A pré-condição do §11.1, medida** (sem miolo, fina da 10 + 9 × outra configuração, Chebyshev em blocos 8×8 da
captura até ao vértice listado mais perto; razão captura/fina = 2):

| diferença | perto (0–2 blocos) | longe (≥ 5 blocos) |
|---|---|---|
| o canto (10 + 9 × 10 + 9 com `Canto::DESLIGADO`) | c5 182 · c2 2 551 | **c5 350 (até 1,05) · c2 268 · c4 830 · c3 364** |
| a `V2` (10 + 9 sem canto × cunha) | quase toda | c5 28 (até 2,0) · c4 151 |

⇒ **a partição pela lista dos vértices da borda está REFUTADA**: os ramos `CANTO_*` disparam onde a própria fina acha
duas faces (`onde_calcular`, `i2 != SEM_FACE`, o eixo medial), e isso acontece longe dos vértices listados. Mas todo o
código do canto está a jusante desse gatilho, logo **a partição pelo gatilho é exata por construção**. Texels de duas
faces (sonda 2 ≥ 100): c1 517 · c2 28 722 · c3 4 875 · c4 3 889 · **c5 6 367 · c6 2 548** · c7 1 870 · c8 0.

**As formas e o custo** (`Canto::passe_proprio`; prova de exatidão = 10 + 9 em passe próprio × em linha, cenas 1–8, com
e sem miolo: **0 texels diferentes de 3 932 160 nas 16**, em r5, r5b e r5c):

| forma | fina c5 / c6 (lei W6d 2,39 / 1,26) | C3 c5 / c6 |
|---|---|---|
| r4 (em linha) | 2,48 / 1,26 | +5,8–6,3 % / +3,4–4,7 % |
| r5 — a fina sem o canto lista os texels (atómico) e o `c3_fina_cantos` grava nas texturas | 2,46 / 1,41 | +5,2 % / **+11,6 %** |
| ↳ só a fina sem o canto (o passe dos cantos por diagnóstico desligado) | **1,95 / 0,93** | — |
| r5b — pré-lista + cantos num buffer + fina + junta (para sobrepor cantos e fina) | 3,60–3,95 / 1,41 | +38–48 % / +11 % |
| ↳ peças sozinhas: a pré-lista 0,67 / 0,58 · pré-lista + cantos 0,77 / 0,59 · pré-lista + fina 2,6–3,95 / **4,38** | | |
| r5c — a fina sem o canto lista, cantos num buffer, junta | 2,46–2,53 / 1,41 | +4,8–6,5 % / +11 % |
| ↳ o passe dos cantos DUAS vezes seguidas | 2,96 / 1,85–1,89 | — |

- **Os texels de duas faces são a cauda cara da fina da própria lei W6d**: tirados da fina comum, ela cai 0,32–0,44 ms
  (−18 % / −26 %). É a maior alavanca de custo medida na C3 — vale também para o padrão, não só para a 10 + 9.
- **O passe próprio custa a latência deles, ~0,45–0,5 ms**, quase igual com 6 367 ou 2 548 texels, e outra vez numa 2.ª
  corrida seguida (não é cache fria): poucos grupos, cada thread a percorrer a cadeia inteira de marchas de um texel com
  a placa quase vazia. Uma lâmpada nas cenas 5 e 6 ⇒ partir o texel por lâmpada não dá nada.
- **Anomalia por explicar:** depois da pré-lista (r5b, sem a fina) o mesmo passe custava só +0,10 / +0,005 ms; e
  pré-lista ‖ fina sem barreira entre elas saiu MAIS cara que a soma (4,38 na c6). Não está entendido; não construir em
  cima disto sem o medir.
- O código da r5c fica, exato e DESLIGADO (`Canto::LIGADO.passe_proprio = false`): `solver_c3_fina_cantos.rs`, as
  entradas `c3_fina_adia` / `c3_fina_cantos` / `c3_fina_junta` e o `CANTO_ADIA` no `c3_direta.wgsl`, os buffers
  `luz2d/buf/fina_adiados` e `luz2d/buf/fina_cantos_luz` na `PassList` (só com `Config::adia_o_canto`). O `pipeline_em`
  passa a deixar as constantes PRÓPRIAS vencerem as comuns da mesma chave; os espelhos dos uniformes foram para
  `solver_c3_uniformes.rs` (o tecto de 700 LOC).
- Com ele desligado (`{f3,caixa,borda}_r5.log`, `portoes_r5_*.log`): F3 e caixa **byte a byte os da r4**, a borda com os
  mesmos (a)/(b)/anéis/bits; nessa corrida a 10 + 9 deu C3 +4,9 % / +3,5 % (`load` ~5–8) — dentro do ruído da r4
  (+5,8–6,3 % na cena 5), não é aprovação. Portões: gpu 9/9 · app 14/15, com o
  `a_luz_recua_quando_a_placa_recusa_a_memoria` a reprovar no lote e a passar sozinho 2/2 (`portao_recua_r5_*.log`):
  o flake conhecido (discrimina pelo fan-out, não pela carga).

## ⛔ Recusas MEDIDAS (§12)

| Recusa | Porquê |
|---|---|
| Um passe só nos vértices listados pela borda (a lista da captura) | O canto e a `V2` mudam a luz a ≥ 5 blocos de qualquer vértice listado (tabela da partição). |
| A pré-lista dos texels de duas faces | Custa 0,58–0,67 ms sozinha (repete o prefixo do `onde_calcular` na região). |
| O passe dos cantos depois da fina, como está (r5/r5c) | Custa ~0,5 ms de latência; poupa 0,32–0,44. |
| Partir o texel de canto por lâmpada | Uma lâmpada nas cenas da régua. |

## 13. ABERTO (por ordem — substitui o §11)

1. **A latência dos texels de duas faces** (o mesmo kill-criterion do §1; a r4 continua a melhor forma: falta a cena 5,
   +5,8–6,3 %). Os texels de duas faces custam 0,32–0,44 ms dentro da fina e ~0,5 ms à parte. Medir ANTES de mexer, por
   uma sonda que conte por texel de duas faces os passos de marcha (`cone_livre`, `soma_da_abertura`, as bisseções, o
   laço do `c0_ao_longo_da_face`) e qual dos passos de luz (o ponto, a outra face) pesa mais. Com o passo dominante na
   mão: encurtá-lo, ou correr os dois passos de luz do texel em duas threads (a `r5c` já dá o sítio para isso, com a
   prova de exatidão pronta em `target/w6g/diag_l.py`). Se a cadeia encurtar, a alavanca vale também para o padrão.
2. A anomalia da r5b (o passe dos cantos barato depois da pré-lista; pré-lista ‖ fina mais cara que a soma).
3. A família (b) (37–38): da W5g, fora.

## 14. A latência dos texels de duas faces, PROVADA (o §13.1; tudo FORA do git e reposto)

**Contada por texel** (`target/w6g/diag_m.py`, `latencia.log`: sondas 9–16 = 24 contadores; sem miolo, com o
`FUNDO_SEM_LUZ` do produto ligado à força; custo = chamadas a `meio` + `livre`, as leituras dependentes da cadeia):

| cena 5, lei W6d (10 + 9 ≈ igual) | n | média | p99 | máx | onde a média vai |
|---|---|---|---|---|---|
| fora (não remapeado) | 684 904 | 100 | 986 | 2 069 | a marcha do ponto |
| uma face | 8 351 | 327 | 865 | 1 539 | `onde_calcular` 170 · ponto 156 |
| **duas faces** | **3 041** | **424** | 1 246 | 1 639 | `onde_calcular` 258 · ponto 152 · outra face 14 |

Cena 6: 1 114 texels de duas faces (média 503, máx 1 180). Com o fundo do produto saltado são 3 041 / 1 114, não os
6 367 / 2 548 do §12 (contados sem o fundo). O passo da outra face corre em ~10 % deles (`passos` 1,1). O pior texel
de duas faces da cena 5 (1248, 618): `onde_calcular` 307 + uma visibilidade pela face de 10 raios × ~67 passos = 668
`meio` + 668 `livre`. ⇒ **a latência do passe próprio (r5, ~0,5 ms) é a cadeia de ~1 600 leituras dependentes do pior
texel** (~0,3 µs cada com a placa quase vazia), não o número de texels — por isso dava o mesmo com 6 367 ou 2 548.

**O modelo do §12 está REFUTADO pela contagem:** o tempo de um warp é o do fio mais lento, e tirar os texels de duas
faces dos warps (Σ do máximo por warp 8×4) poupa só 1,9 % (c5) / 1,2 % (c6) — ~0,05 / ~0,015 ms, não 0,32–0,44.

**TRABALHO × CÓDIGO** (`diag_{n,o,p,q}.py`, `bissecao_{n,o,p,q}.sh`, `custo_{n,o,p,q}_k*_{1,2,3}.log`; um `override
W6G_K` só na fina da lei W6d, a `cunha` do MESMO processo como controle (2,37–2,40 / 1,24–1,27 em todas as corridas);
três voltas em ordem alternada; `loadavg` 2–8):

| variante (fina da lei W6d) | c5 | c6 | lê-se |
|---|---|---|---|
| K0 como está | 2,37–2,40 | 1,25–1,27 | |
| K1 duas faces como uma, o código FORA | **1,87–1,89** | **0,92–0,93** | o efeito todo |
| K2 o mesmo, o código compilado e MORTO | 2,33–2,35 | 1,10–1,12 | trabalho: 0,05 / 0,15 |
| K3 duas faces saem sem luz | 2,36–2,37 | 1,09–1,10 | |
| K4 / K8 o `pe_na_face` fora / morto | = K0 | ≈ K0 | nada |
| K9 / K10 o 2.º `pe_na_borda` fora / morto | 2,29–2,33 | 1,20–1,22 | trabalho, ~0,07 |
| **K5 o passo da OUTRA FACE fora** | **1,93–1,95** | **0,97–0,98** | **o código: 0,45 / 0,18** |
| K7 o passo da outra face compilado e morto | = K0 | 1,15–1,16 | trabalho dele: 0 / 0,11 |
| K11 a outra face pelo modo da GRADE · K16 a conta exata dentro do ramo da grade | = K0 | = K0 | não é o modo `EXATA` |
| **K18 um acumulador só (`l += x · peso`, sem o `face`)** | **1,95–1,96** | **1,11–1,12** | o código volta todo; o trabalho fica |
| K19 a outra face marcha do `p` (o `p2` morto) | 1,96–1,98 | 1,11–1,12 | idem |

⇒ **os 0,32–0,44 ms do §12 são um DEGRAU DE REGISTOS do `c3_fina`, não trabalho dos texels de duas faces**: o passo da
outra face no laço do `luz_do_texel` mantém vivos, por todo o laço (e pela árvore embutida do `luz_das_lampadas`), o
acumulador `face` e o ponto `p2`; tirar QUALQUER um dos dois devolve 0,43 ms na cena 5 e ~0,15 na 6 (K18, K19), com o
mesmo trabalho. A r5 «ganhava» por o kernel da fina compilar sem esse passo (K5 ≈ a fina da r5, 1,95 / 0,93), e pagava
de volta a cadeia inteira no passe à parte. Vale para a LEI W6D (o padrão), não só para a 10 + 9. É um degrau: um
registo a mais no laço volta a pagá-lo — a prova da cura tem de medir os dois lados dele.

## ⛔ Recusas MEDIDAS (§14)

| Recusa | Porquê |
|---|---|
| «Os texels de duas faces são a cauda cara da fina» (o §12) | Contados: ~2 % do Σ dos warps; K2 × K1: o código, não o trabalho. |
| Encurtar o `onde_calcular`/o `pe_na_face` pelo custo da fina | K4 = K8 = K0. |
| Tirar o modo `EXATA` da outra face | K11 = K16 = K0. |
| Partir o texel de duas faces em duas threads (a r5c) | O passo da outra face corre em ~10 % deles e pesa 0 na cena 5; a cadeia é o `onde` + a marcha do ponto. |

## 15. A cura (r6) — a mistura das faces no próprio passo (escrita ANTES de mexer)

A forma EXATA do K18: no passo da outra face, `l = mix(l, x, f2)` ali mesmo (a MESMA operação sobre os mesmos
valores que o `mix(l, face, f2)` do fim — o passo 1 só corre com `f2 > 0` e sempre depois do 0), sem o acumulador
`face`; a sonda 4 guarda as suas componentes no passo de cada uma. Vale para toda a fina (lei W6d, 10 + 9, a exata, o
passe dos cantos da r5c), sem interruptor: não muda um bit.

**PASSA a r6** (dois itens): (i) 0 bytes diferentes na fluência lida, HEAD × r6, cenas 1–8 × com/sem miolo × lei W6d /
10 + 9 (`W6G_DUMP`, dois builds); (ii) a fina da lei W6d ≤ 1,98 / 1,13 ms (o K18). **MORRE:** um byte diferente, ou a
fina no K0 (o degrau não caiu na forma exata).

Depois, a 10 + 9 com a r6 contra o kill-criterion do §1 tal e qual (o C3 relativo à lei W6d, que também desce:
o alvo fica MAIS difícil se as duas descerem o mesmo). Se passar: a rodada completa (`fecho_r5.sh`) e o padrão.

## 16. A r6 medida — o PADRÃO passa; a 10 + 9 não desce

**A r6 passa** (`exato_r6.log`, `custo_r6_{1,2}.log`, `{f3,caixa,borda}_r6.log`, `portoes_r6_*.log`, `fecho_r6.sh`):
(i) **0 bytes diferentes** na fluência inteira (rgba), HEAD × r6, nos 32 casos (cenas 1–8 × miolo × lei W6d / 10 + 9;
`dump_head/` × `dump_r6/`, sonda `diag_r.py`); (ii) a fina da lei W6d **1,949–1,953 / 1,113–1,118** (≤ 1,98 / 1,13).
F3 e caixa saem linha a linha os da r5; a borda com os mesmos (a)/(b)/anéis/bits; portões gpu 9/9 · app 15/15 (`load`
6–9). **O padrão (lei W6d) fica 0,44 ms mais barato na cena 5 — C3 3,49 → 3,05 ms (−12,6 %) — e 0,14 na 6 (2,24 →
2,10, −6,3 %), sem mudar um bit.**

**A 10 + 9 não desce:** fina 2,43–2,44 / 1,23 (r4: 2,46–2,48 / 1,26) ⇒ C3 3,65 / 2,31 = **+19,7 % / +10,5 %** sobre a
lei W6d nova ✗ (era +5,8–6,3 % / +3,4–4,7 % sobre a velha). Continua acima do degrau. A bisseção por config, no MESMO
processo, duas voltas (`diag_s.py`, `custo_s_{1,2}.log`, `load` 2,6):

| fina c5 / c6 | c5 | c6 |
|---|---|---|
| lei W6d (r6) · cunha | 1,95–1,96 | 1,11–1,12 |
| 10 + 9 | 2,44–2,45 | 1,23 |
| J1 sem `luz` · J2 sem `normais` · J3 sem `fracao` · J4 sem `um_lado` | 2,42–2,46 | 1,22–1,34 |
| J5 o `Canto` todo desligado (só a `V2` no `meio`) | 2,42–2,43 | 1,33 |
| J6 o `Canto` ligado sem a `V2` | 2,39 | 1,26 |

⇒ **a `V2` no `meio` e o código do `Canto` põem-na acima do degrau CADA UM POR SI**; só sem os dois (a cunha) a fina
volta a 1,95. A r6 era a tentativa nova sobre o modelo do §14 e, para a 10 + 9, não passou ⇒ PARE (two-strikes): a
r6 fica comitada (exata, no padrão); a 10 + 9 continua desligada.

## ⛔ Recusas MEDIDAS (§16)

| Recusa | Porquê |
|---|---|
| Tirar UMA peça do `Canto` para a 10 + 9 caber | J1–J4: nenhuma desce; J1 e J5 encarecem a cena 6 (o trabalho do canto poupa marchas). |
| A `V2` no `meio` OU o `Canto` sozinhos como «o» culpado | J5 e J6: cada um sozinho mantém o degrau. |

## 17. ABERTO (por ordem — substitui o §13)

1. **A 10 + 9 abaixo do degrau de registos do `c3_fina`** (o kill-criterion do §1 tal e qual, agora contra a lei
   W6d da r6: o alvo é a fina da 10 + 9 ≤ ~2,05 / 1,17). Primeiro uma RÉGUA DE REGISTOS (o wgpu não a expõe: o
   `VK_KHR_pipeline_executable_properties` por uma ferramenta à parte sobre o SPIR-V do naga, ou o tamanho da entrada
   no cache de shaders do driver) — sem ela cada forma é um palpite sobre um degrau. Depois, as duas peças juntas: a
   `V2` no `meio` (a leitura e o `na_cunha` embutidos em CADA sítio do `meio`) e o estado do canto vivo pelo laço do
   `luz_do_texel` (`face_do_ponto`, `face_da_outra`, `face_corrente`).
2. A anomalia da r5b (§12).
3. A família (b) (37–38): da W5g, fora.

## 18. A RÉGUA DE REGISTOS e a r7 (por ordem do dono «siga»; escrita ANTES de mexer no produto)

**A régua** (`target/w6g/regua_registos/`, crate fora do workspace com o `Cargo.lock` dele, `--offline`): o módulo rc e as
constantes de cada pipeline da C3 recolhidos por um gancho temporário no `pipeline_em` (`W6G_REGUA=<pasta>`,
`diag_t.py`), compilados pelo caminho do wgpu-hal 29 (naga `process_overrides` + `spv::write_vec` com as opções dele:
SPIR-V 1.6, `index: Restrict`, `force_loop_bounding`, as robustezes da placa) num `VkDevice` próprio com
`VK_KHR_pipeline_executable_properties` e `CAPTURE_STATISTICS` — o driver devolve `Register Count`, o tamanho do
binário e a memória partilhada (as representações internas: zero, a NVIDIA não as dá). Segundos por variante, sem
compilar Rust. **Validada contra o degrau medido** (§14/§16): a lei W6d tinha 168 registos antes da r6 e tem **128**
com ela (a cunha e o «antes da W6d» também). Na RTX 5060 Ti: 168 → 6 grupos de 64 por SM, 128 → 8 (+33 % de ocupação).

**O mapa** (`regua_mapa.log`: V2 × as 16 combinações do `Canto`, sobre as constantes da 10 + 9): só **`BORDA_CANTOS`
(a V2 no `meio`) e `CANTO_LUZ`** levam a 168; `normais`, `fracao` e `um_lado` cabem nos 128. E toda variante com
`DISTANTES` (objetos fora da grade num quadro) fica em 168 — **mesmo a lei W6d**.

**A forma (r7), medida na régua** (`regua_x.py`, `rc_x.wgsl`): em `luz_das_lampadas`, as quatro leituras da grade
(`w0..w3`, 8 registos) e `u`/`base`/`fr`/`q`/`r` eram calculadas ANTES do laço das lâmpadas — vivas durante a marcha da
`visibilidade`, o ponto mais fundo. Dentro do ramo `PELA_GRADE`, por lâmpada: as MESMAS leituras nas mesmas
coordenadas (exata), mortas antes da marcha. Régua: lei W6d 128 · **lei W6d dist 168 → 128** · **10 + 9 168 → 128** ·
**10 + 9 dist 168 → 128** · só a V2 168 → 128 · só a `luz` 168 → 128.

**PASSA a r7**: (i) 0 bytes diferentes, r6 × r7, nos 32 casos do §16; (ii) a fina da lei W6d ≤ a da r6 (1,95 / 1,11).
Depois a 10 + 9 contra o kill-criterion do §1 tal e qual; passando, a rodada completa e o padrão
(`borda_vertice: Some(VERTICE_PADRAO)` + `borda_cantos: true` + `Canto::LIGADO`). **MORRE**: um byte diferente, ou a
10 + 9 acima de +5 % numa das cenas (a régua de registos mente sobre o tempo ⇒ medir o que mais pesa).

## 19. A r7 medida — exata; a 10 + 9 a 0,025 ms do alvo na cena 5

(`exato_r7.log`, `custo_r7_{1,2}.log`, `portoes_r7_*.log`; `load` 20–32 das outras linhas, a placa pela vez.)

- **(i) 0 ficheiros diferentes** em 32 (r7 × a base do §16, que é a r6 byte a byte). **(ii)** fina da lei W6d 1,943–1,947 /
  1,107–1,110 (= r6). Portões gpu 9/9 · app 15/15. A r7 PASSA: fica no produto, para todas as variantes.
- **A 10 + 9 desce:** fina c5 2,44 → **2,003–2,010**, c6 1,23 → **1,091–1,093** (agora abaixo da lei W6d na cena 6).
  C3 **3,218–3,220 / 2,179** contra a lei W6d 3,041–3,042 / 2,090–2,092 ⇒ **+5,8 %** ✗ / **+4,2 %** ✓. É mais barata
  que a lei W6d era ANTES da r6 (3,49 / 2,24), mas o §1 mede contra a lei W6d do mesmo processo.
- **Onde estão os 0,177 ms da cena 5** (o alvo deixa 0,152): `preenche + borda` +0,113 (a borda 0,372 contra 0,272 — a
  cunha já +0,03, o `c3_borda_cantos` +0,07; o §10) e a fina +0,06 (os dois kernels a 128 registos; o binário da 10 + 9
  é 32 % maior, 278 912 × 211 584 bytes).
- **Por medir no relógio:** a lei W6d com `DISTANTES` (objetos fora da grade num quadro) passou de 168 a 128 registos
  na régua — nenhuma cena da régua do custo a exercita (as cenas 5 e 6 correm sem distantes).

A 2.ª reconstrução desta rodada sem passar (r6, r7) ⇒ PARE: a r7 comitada; a 10 + 9 continua desligada.

## ⛔ Recusas MEDIDAS (§18–§19)

| Recusa | Porquê |
|---|---|
| Ler as sondas da grade antes do laço das lâmpadas | Vivas durante a marcha: 128 → 168 registos em toda variante com V2, `CANTO_LUZ` ou `DISTANTES`. |
| Pedir à NVIDIA o código de máquina pela extensão | `CAPTURE_INTERNAL_REPRESENTATIONS`: zero representações. |

## 20. ABERTO (por ordem — substitui o §17)

1. **A 10 + 9 na cena 5: faltam 0,025 ms** (+5,8 % contra ≤ +5 %). Decisão do DONO antes de outra rodada: o alvo
   relativo ficou mais duro porque o padrão desceu 13 %; ou se mantém (e a próxima rodada ataca a borda, +0,07 sobre a
   cunha, ou o binário da fina), ou passa a ser absoluto. Os quatro itens de qualidade já passam (r3/r4).
2. A lei W6d com `DISTANTES` no relógio (uma cena com objetos fora da grade na régua do custo).
3. A anomalia da r5b (§12).
4. A família (b) (37–38): da W5g, fora.
