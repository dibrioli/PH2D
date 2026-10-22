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

1. **A 10 + 9 na cena 5: faltam 0,025 ms** (+5,8 % contra ≤ +5 %). **DECIDIDO pelo dono (19/10): o alvo MANTÉM-SE —
   C3 ≤ +5 % sobre a lei W6d de HOJE (a r7), nas cenas 5 e 6, no mesmo processo.** Uma rodada nova, com a regra do
   §1 tal e qual (os quatro itens de qualidade já passam, r3/r4; two-strikes de novo). Ponto de partida:
   - a borda primeiro (o maior resto: `c3_borda_cantos` +0,07 sobre a cunha, que já é +0,03 sobre a lei W6d; a
     bisseção do §8 tem as peças, e a r4 deixou a borda a +0,066–0,071) — os dois passes da borda têm `C3_BORDA` na
     régua do custo; depois a fina (+0,06; os dois a 128 registos, o binário 32 % maior);
   - a régua de registos corre em segundos SEM compilar Rust: `bash scripts/ph2d-run.sh
     target/w6g/regua_registos/target/release/regua_registos <rc.wgsl> <entrada> <constantes.txt>...` (recolha do rc
     e das constantes: `python3 -I target/w6g/diag_t.py` + `W6G_REGUA=<pasta>` na régua do custo, e repor) — serve
     para qualquer entrada (`c3_borda_cantos`, `c3_cantos`), não só a fina; as formas testam-se sobre o texto do rc
     (molde `target/w6g/regua_x.py`) antes de tocar no produto;
   - a prova de exatidão: `diag_r.py` + `W6G_DUMP` (32 casos), base em `target/w6g/dump_head/` (= r6 = r7).
2. A lei W6d com `DISTANTES` no relógio (uma cena com objetos fora da grade na régua do custo).
3. A anomalia da r5b (§12).
4. A família (b) (37–38): da W5g, fora.

## 21. A rodada nova (§20.1, ordem do dono 19/10): a borda pela régua de registos (escrita ANTES de mexer no produto)

**A régua de registos sobre a borda** (`rb_x.py`, `rb_corre.py`, `target/w6g/rb/`; o rc da r6, que tem a borda da r7):

| `c3_borda*` | lei W6d | cunha | 10 + 9 (`c3_borda_cantos`) |
|---|---|---|---|
| como está | 32 reg · 7 KB | **55** · 45 KB | **61** · 50 KB |
| a `pela_cunha` fora (`A`) | 32 | **32** | **37** |
| o `area_da_intersecao` fora (`C`) | 32 | 55 | 61 |
| as `pelas_retas_proprias` fora (`D`) | 24 | 55 | 60 |

⇒ **o degrau é a `pela_cunha` embutida no kernel da captura inteira** (corre em ~centenas de texels; o registo paga-se
nos 15,7 milhões). 55–61 registos = 36–32 warps por SM contra 48 com ≤ 40.

**No relógio** (`diag_u.py` + `diag_u2.py`, `custo_u_*`, `custo_u2_*`: um override `W6G_STUB` que mata a `pela_cunha` e
dois cortes de despacho, duas voltas alternadas, `load` 10–25; tudo FORA do git e reposto):

| borda c5 (ms) | lei W6d | cunha | 10 + 9 |
|---|---|---|---|
| como está | 0,271–0,273 | 0,302 | 0,373–0,374 |
| s1 a `pela_cunha` morta | 0,272 | **0,272–0,273** | **0,323** |
| s2 + sem `limpa`/`cantos` (vazios) | | | 0,316 |
| s3 + a entrada `c3_borda` e sem o `zera` | | | **0,272–0,273** |

⇒ a cunha inteira (+0,03) é o degrau; na 10 + 9, −0,05 da `pela_cunha`, 0,007 dos dois indiretos vazios e **0,044 da
forma do `c3_borda_cantos`** (a barreira do grupo, a marca em todo bloco na origem da `V`, o `zera` — juntos).

**A cura (r8)** — o vértice ADIADO, a borda da captura volta à forma da lei W6d:
- `c3_borda_adia` (captura inteira, só com os cantos): o `borda_do_texel` com a `pela_cunha` fora — o texel cuja reta
  não explica a 3×3 grava a própria reta e reclama o seu bloco (`atomicExchange` numa marca por bloco, buffer novo; o
  primeiro acrescenta-o à lista deste quadro). Sem barreira, sem atómica de grupo, sem escrita na `V`.
- `c3_borda_vertices` (indireto sobre a lista deste quadro): o `borda_do_texel` com a `pela_cunha` nos 64 texels do
  bloco — regrava a borda (a mesma conta: os mesmos bits) e escreve a `V` do bloco inteiro.
- a `V` passa a valer a sentinela fora dos blocos listados (o `c3_cantos_inicia` uma vez; o `c3_cantos_limpa` repõe a
  `V` do bloco e a marca dele, além da `V2` dos 3×3) ⇒ o `c3_cantos` lê a apara 12×12 sem a fase das marcas.
- A lista passa a ser a dos blocos com CANDIDATO (⊇ os de cunha aceite): o bloco listado sem cunha tem a `V` toda
  sentinela, e o `c3_cantos` escreve nele o que a `V2` já tinha. Exata por construção. A cunha sem os cantos (que
  não é o padrão) fica como está.

**PASSA a r8** (todos): (i) 0 bytes diferentes na fluência, r7 × r8, nos 32 casos (`diag_r.py` + `W6G_DUMP`, base
`dump_head/`); (ii) a régua de registos: `c3_borda_adia` ≤ 40; (iii) o §1 tal e qual — os itens 1–4 pela rodada
completa (F3, caixa, borda) e o **5: C3 da 10 + 9 ≤ +5 % sobre a lei W6d nas cenas 5 e 6** (o mínimo de 7
intercaladas, no mesmo processo), `preenche + borda` ≤ +0,17. **MORRE**: um byte diferente, ou o C3 acima de +5 % numa
das cenas. É a 1.ª reconstrução desta rodada (two-strikes conta daqui).

## 22. A r8 medida — exata; a 10 + 9 passa o custo nas duas cenas

(`exato_r8.log`, `custo_r8_{1,2,3}.log`, `regua_r8/`; `load` 3.)

- **(i) 0 ficheiros diferentes** em 32 (r8 × `dump_head/` = r7). **(ii)** régua: `c3_borda_adia` **32** registos (o
  `c3_borda_cantos` tinha 61); o `c3_borda_vertices` 59, só nos blocos listados; `c3_cantos` 31 (sem a fase das marcas).
- **(iii) o custo**, três processos:

| ms | lei W6d | cunha + 10 + 9 | C3 | borda | preenche + borda |
|---|---|---|---|---|---|
| cena 5 | 3,041 · 3,059 · 3,045 | 3,168 · 3,168 · 3,170 | **+4,2 · +3,6 · +4,1 %** ✓ | 0,318–0,319 (r7: 0,372) | +0,054–0,057 ✓ |
| cena 6 | 2,092 · 2,087 · 2,089 | 2,115 · 2,124 · 2,124 | **+1,1 · +1,8 · +1,7 %** ✓ | 0,306–0,307 (r7: 0,358) | +0,043–0,044 ✓ |

A borda da 10 + 9 desce 0,054 ms; a cunha sem os cantos (que não é o padrão) fica nos seus +0,03 (o degrau dela,
§21). O resto da borda sobre a lei W6d (+0,046) são os cinco despachos pequenos e as barreiras; a fina +0,07 / −0,01.

## 23. A rodada completa com a 10 + 9 no padrão — o §1 passa; o padrão NÃO liga (a memória e o G9 holográfico)

(`fecho_r8.sh`; `{f3,caixa,borda}_r8.log`, `suite_r8_{gpu,app}.log`, `sozinhos_r8_{gpu,app}.log`; `load` 3–6.)

- **O §1 tal e qual passa:** F3 e caixa saem linha a linha os da r6 (`diff` vazio fora dos relógios); a borda com os
  mesmos (a)/(b)/anéis/bits (`1 −0,176 / 38 / 0 / 0`); o 4.º custo +3,7 % / +0,9 %.
- **As suítes das duas crates com o padrão ligado** (`PH2D_SUITE="motivo: o padrão da luz muda"`): gpu 41/42, app
  77/84. Sete da app são `SemMemoria` sob os testes em paralelo na mesma placa e passam sozinhos (`--test-threads=1`,
  7/7). **O `holografica::g9_o_rebote_na_holografica` reprova sozinho**: Baixo 0,0093 / **0,0105** / 0,0098 contra
  `TOL_G9_HOLO[0]` = 0,0101 (medido 0,0092 na lei W6d). Por medir: qual peça da 10 + 9 o move e se o gabarito do
  rebote o vê como erro ou como o canto mais exato.
- **A memória** (`w6g_memoria_dos_cantos`, teste temporário no `os_tetos_da_placa.rs`, reposto): a `V` e a `V2` são
  duas texturas `rgba16float` da captura INTEIRA para ~150 blocos com conteúdo em 245 760.

| vista · degrau | sem os cantos | com | + |
|---|---|---|---|
| 1080p · Baixa / Média / Alta | 414 / 574 / 1143 MB | 669 / 829 / 1381 | **+255 (+61 %)** / +255 (+44 %) / +238 (+21 %) |
| 1440p · Baixa / Média / Alta | 712 / 973 / 1865 | 1153 / 1414 / 2271 | **+441 (+62 %)** / +441 (+45 %) / +406 (+22 %) |
| 2560×1080 · Baixa / Média / Alta | 536 / 735 / 1451 | 867 / 1066 / 1762 | +331 (+62 %) / +331 / +311 |
| 2160p (k 1) · Baixa / Média / Alta | 582 / 1143 / 4089 | 837 / 1381 / 4327 | +255 / +238 / +238 (+6 %) |

⇒ ligar assim é pagar +44–62 % de memória da luz nos degraus Baixa/Média para 0,06 % da captura: não é o padrão-ouro.
O padrão fica a lei W6d (`borda_vertice: None`); comitado o `Config::sem_os_cantos` e as variantes de comparação das
réguas e portões («antes da W6d», «lei W6d», «cunha», o controlo das formas redondas) já presas a ele — a identidade
hoje, o seu significado intacto quando a 10 + 9 ligar.

## ⛔ Recusas MEDIDAS (§21–§23)

| Recusa | Porquê |
|---|---|
| A `pela_cunha` dentro do kernel da captura inteira | 55–61 registos contra 32; a cunha inteira +0,03 ms e a 10 + 9 +0,05 só disso (§21). |
| A barreira de grupo + a marca na origem da `V` em todo bloco | 0,044 ms juntos (s2 × s3, §21); a marca por bloco num buffer com `atomicExchange` sai de graça. |
| Ligar a 10 + 9 com a `V`/`V2` densas | +255–441 MB (+44–62 %) nos degraus Baixa/Média (§23). |

## 24. ABERTO (por ordem — substitui o §20)

1. **A `V` e a `V2` ESPARSAS** (a cura da memória, §23): só os blocos listados (e os 3×3 deles) num pool, com uma
   tabela por bloco da captura (`u32`, ~1 MB; sentinela = sem cunha) que o `meio` lê antes da `V2`. Atenção: a leitura
   da `V2` está no `meio` da fina — a tabela é uma leitura DEPENDENTE a mais no laço; meça na régua de registos e no
   relógio ANTES (a fina da 10 + 9 está a +0,06–0,07 / −0,01). O pool precisa de teto do dispositivo e de uma recusa
   dita (não um corte calado) quando a cena tem mais vértices que ele. O kill-criterion: o §1 tal e qual + a memória
   da 10 + 9 ≤ +5 % sobre a lei W6d em toda vista da tabela + os bytes da fluência iguais aos da r8 (32 casos).
2. **O G9 holográfico com a 10 + 9** (§23): Baixo 0,0105 > 0,0101. Bissecção pelas peças do `Canto` e pela `V2`; se
   for o canto mais exato, o gabarito do rebote tem de o mostrar (não se sobe a tolerância sem isso).
3. Depois dos dois: ligar no padrão (`borda_vertice: Some(VERTICE_PADRAO)` + `canto: Canto::LIGADO` +
   `borda_cantos: true`) e as suítes das duas crates (os `SemMemoria` em paralelo têm de sumir com o item 1).
4. A lei W6d com `DISTANTES` no relógio (o §20.2).
5. A anomalia da r5b (§12).
6. A família (b) (37–38): da W5g, fora.

## 25. A rodada r9 (§24.1): a V e a V2 esparsas — o diagnóstico ANTES do produto (tudo fora do git, `target/w6g/esparsa/`)

**A régua de registos** (`x.py`, `corre.py`, sobre o rc da r8): a leitura dependente da tabela no `meio` NÃO
sobe o degrau — fina da 10 + 9 em 128 registos em toda forma (tabela em textura `r32uint` ou em buffer, com
ou sem ramo, cedo ou tarde); o binário +8–15 % (283 → 299–326 KB). A forma por `subgroupAny` (ramo uniforme
no warp) sobe a **168** registos e +47 % de binário.

**O relógio** (`diag.sh`/`so_custo.sh`: a tabela IDENTIDADE sobre a `V2` densa — os mesmos bytes, a mesma
leitura dependente —, variantes no MESMO processo pelo `modulo_rc`, 3 voltas de mínimos de 7; `load` 20–26
das outras linhas, a placa pela vez). Fina da cena 5 (ms; a densa 1,996–2,013):

| forma | fina c5 | leitura |
|---|---|---|
| tabela → atlas, sem ramo (S3) | 2,038–2,059 | +0,035–0,045 |
| o mesmo endereço SEM ler a tabela (S8) | 2,014–2,039 | a aritmética ~0 |
| a tabela num registo privado por bloco (S6) | 2,078–2,086 | pior (o ramo diverge) |
| a tabela cedo, antes dos gathers (S5) | 2,050–2,059 | e o preenche +0,09 |
| com ramo (`e ≠ 0`), textura ou buffer | 2,038–2,068 | = sem ramo |
| a tabela lida sem dependência | 2,028–2,035 | |
| SÓ a tabela, sem o atlas (resultado errado) | 2,009–2,019 | = a densa |
| a DENSA + um ramo NUNCA tomado com uma leitura | 2,033–2,044 | **o mecanismo** |
| buffer, ramo grande (atlas + cunha + retorno dentro), tabela só nos blocos COM cunha | 2,023–2,040 | **a melhor** |
| a V2 lida cedo (densa D / tabela cedo C / as duas cedo E) | 2,04–2,05 / 2,07 / 2,24 | pior |

⇒ **o custo da esparsa é a ESTRUTURA de um ramo com leitura dentro** (a densa com um ramo nunca tomado paga o
mesmo), não a leitura nem a dependência; +0,02–0,03 ms na fina da cena 5, +0,02 na 6. Com a melhor forma a 10 + 9
dá C3 **+4,0 a +5,6 %** na cena 5 (o mínimo da lei W6d oscila 3,025–3,076 entre voltas) e +2,6–3,2 % na 6 — em
cima da linha. A borda: com os 4 kernels pequenos VAZIOS 0,288–0,295 (lei W6d 0,272, a 10 + 9 0,320) ⇒ despacho +
barreiras ~0,02, trabalho ~0,03; os cantos só nos blocos com cunha (a lista B, abaixo) 0,316–0,320 — o trabalho dos
cantos não era o custo.

**O censo** (cena 1–8, a 10 + 9, 1080p): listados (candidatos) / com cunha na `V` / blocos da `V2` com cunha —
c1 4/4/9 · **c2 2 451/1 975/3 250** · c3 8/8/32 · c4 12/12/28 · **c5 157/71/152** · c6 7/7/20 · c7 7/7/16 · c8 0.
⇒ o pool cabe MILHARES (a cena 2 do smoke, ~1 % dos 245 760 blocos), e só 45 % dos listados da c5 têm cunha.

## ⛔ Recusas MEDIDAS (§25)

| Recusa | Porquê |
|---|---|
| A tabela lida antes dos gathers / a `V2` lida fora do ramo `\|d\| < 1` | Toda chamada do `meio` paga: preenche +0,09, fina +0,04–0,24. |
| O ramo uniforme por `subgroupAny` | 168 registos, binário +47 %. |
| A entrada da tabela num registo privado (lida ao mudar de bloco) | Fina 2,08 (o ramo diverge). |
| O sinal «há cunha» no canal 0 (sinal) ou 1–2 (normal) da borda | O `c3_pe` lê a normal e o `m.c` tem consumidor: muda bytes fora da cunha. |
| A tabela a apontar todos os 3×3 dos CANDIDATOS | O ramo entra demais; só os blocos com alguma cunha. |

## 26. A r9 — o produto (escrita ANTES de mexer; 1.ª reconstrução desta rodada)

- `meio`: `|d| < 1` ⇒ `e = cantos_tabela[bloco(t)]` (buffer, `u32`; 0 = sem cunha, senão o slot + 1 no pool da
  `V2`); `e ≠ 0` ⇒ a `V2` no atlas e a decisão da cunha DENTRO do ramo (o ramo grande); senão `d < 0`.
- a `V` e a `V2` em ATLAS de blocos 8×8 (`rgba16float`, 128 blocos por linha): a `V` no slot = o índice do
  listado (`marcas[b] = i + 1` pelo vencedor do `c3_borda_adia`), a `V2` num slot por bloco COM cunha (o vencedor
  de um `atomicCompareExchange` na tabela tira-o de um contador); os listados com cunha numa 2.ª lista (B) que o
  `c3_cantos` percorre. Sem textura da captura inteira, sem `c3_cantos_inicia`; o `limpa` repõe a tabela dos 3×3
  e a marca.
- **o teto** (o recurso é a MEMÓRIA): `blocos / 32` slots em cada pool (7 680 a 1080p = 3,1 % dos blocos; a
  cena 2 pede 2 451 / 3 250) — `Config::cantos_por_blocos`. Acima dele o bloco fica na lei W6d (o
  `c3_borda_vertices` sai sem regravar; a `V2` fica sem entrada) e conta-se: `Solver::cantos_recusados` (a recusa
  DITA), com um portão que força o teto a 1 slot.
- **PASSA** (todos): (i) 0 bytes diferentes na fluência, nos 32 casos (`dump_head/`, a r7 = r8); (ii) o §1 tal e
  qual (F3, caixa, borda pela rodada completa; C3 ≤ +5 % sobre a lei W6d nas cenas 5 e 6, o mínimo de 7, três
  processos); (iii) a memória da 10 + 9 ≤ +5 % sobre a lei W6d em toda vista da tabela do §23; (iv) o portão da
  recusa. **MORRE**: qualquer um. O C3 está em cima da linha (§25): se morrer só nele, a alavanca medida que
  resta são os dois despachos sem trabalho (`zera`, `limpa`: ~0,008 ms) por um carimbo de quadro.

## 27. A r9 medida — o alvo 1 PASSA (a V e a V2 esparsas)

(`target/w6g/esparsa/r9.sh` · `custo_r9_{1,2,3}.log` · `{f3,caixa,borda}_r9.log` · `memoria_r9.log`; `load` 27–29 das outras
linhas, a placa pela vez.)

- **(i) os bytes:** 0 ficheiros diferentes em 32 (`dump_head/` = r7 = r8). **A 1.ª corrida dava 6** (c2, c5, c7 da 10 + 9) e o
  censo da cena 5 109–116 blocos com cunha em vez de 152, a variar por volta: o `c3_borda_adia` reivindicava o bloco por
  `atomicExchange(marca, RESERVADO)` — um candidato que chegasse DEPOIS do vencedor ter escrito o índice voltava a trocá-lo
  por RESERVADO, e o `c3_cantos` lia a `V` daquele bloco como vazia. Cura: `atomicOr` de um bit alto (idempotente) e o
  índice por outro `atomicOr`; quem lê mascara. ⛔ Uma troca por um valor PROVISÓRIO não é reivindicação quando o vencedor
  escreve por cima.
- **(ii) o §1 tal e qual:** F3 e caixa linha a linha os da r8; a borda `(a) 1 −0,176 · (b) 38 · anéis 0 · bits 0` (as linhas
  BORDA/ANEIS/BITS iguais às da r8); **C3 da 10 + 9 sobre a lei W6d, três processos: cena 5 +4,2 · +4,6 · +4,4 % · cena 6
  +2,3 · +2,3 · +2,5 %** ✓; preenche + borda +0,053–0,061 ✓.
- **(iii) a memória** (portão permanente `os_tetos_da_placa::os_cantos_custam_ate_5_por_cento_da_memoria`, a `PassList`):

| vista · Baixa / Média / Alta | lei W6d (MB) | com os cantos | + |
|---|---|---|---|
| 1080p | 414 / 574 / 1142 | 428 / 588 / 1155 | +3,3 / +2,4 / +1,1 % |
| 1440p | 711 / 972 / 1865 | 735 / 996 / 1887 | +3,4 / +2,5 / +1,2 % |
| 2560×1080 | 536 / 734 / 1451 | 554 / 752 / 1467 | +3,3 / +2,4 / +1,2 % |
| 2160p (k 1) | 582 / 1142 / 4089 | 596 / 1155 / 4102 | +2,4 / +1,1 / +0,3 % |

- **(iv) a recusa DITA:** `os_tetos_da_placa::o_teto_dos_cantos_recusa_e_diz` (a cena 5 lista 157; o teto mínimo, 128 slots,
  recusa; o padrão não; o número volta pelos tempos; a luz finita) — prova de mutação: sem a contagem no
  `c3_borda_vertices` e sem a cópia no `resolver` dos tempos, as duas VERMELHAS; repostas, verde. O número chega ao
  Inspector: `TemposMs::cantos_recusados` → `LuzDoQuadro` → `AvisoDaLuz::CantosArredondados { blocos }` →
  `panel.inspector.luz.corners_rounded_n` (o `seam_luz` pinta-o).

**O código (r9):** `c2_rc.wgsl` (o `meio`: `cantos_tabela` em buffer, binding 10, e o ramo grande); `c3_borda.wgsl` (os atlas,
a marca por `atomicOr`, a lista B dos listados com cunha, a reivindicação da tabela por `atomicCompareExchangeWeak`, os
contadores; sem `c3_cantos_inicia`); `solver_c3_borda.rs` (os grupos, `tabela`, `contadores`); `passlist_cantos.rs` (novo:
os nomes, `atlas_dos_cantos`, `buffers_dos_cantos`); `Config::cantos_por_blocos` (`CANTOS_POR_BLOCOS_PADRAO` = 32);
`Solver::cantos_lidos` (a vista DENSA reconstruída para as réguas; substitui o `cantos_da_captura`) e
`Solver::cantos_recusados`. **O tecto de 700 LOC** que a linha já passava (o `config.rs` 801, o `passlist.rs` 705 no HEAD
da r8): as escolhas da `Config` num irmão `config_escolhas.rs` (666 linhas), os recursos dos cantos no `passlist_cantos.rs`
(692), a tabela e o acessor dos cantos no `solver_c3_borda.rs` (o `solver_c3.rs` 695).

**ABERTO (por ordem):** o G9 holográfico com a 10 + 9 (§24.2; a régua da bisseção pronta, fora do git: `w6g_g9_bissecao`,
as peças contra o gabarito a 0,01 e a 0,005 m) → ligar no padrão → as suítes das duas crates (os `SemMemoria` em paralelo
devem sumir: −250–440 MB por solver) → o resto do §24.

## 28. O G9 holográfico com a 10 + 9 — bissecção, cura, e a 10 + 9 no PADRÃO

(`g9_r9.log`, `g9_r9_sem_rebote.log`: a régua `w6g_g9_bissecao`, fora do git — cada peça da 10 + 9 nos três degraus, o erro
por ponto contra o gabarito do rebote a 0,01 m E a 0,005 m.)

| Baixo (c0 a 4 px) | p1 | p2 | p3 | p4 (meio da sala) |
|---|---|---|---|---|
| lei W6d | 0,0052 | 0,0071 | 0,0030 | 0,0063 |
| cunha · cunha + canto · as 4 peças do canto | 0,0056 | 0,0071 | 0,0030 | 0,0063 |
| **cunha + V2** · a 10 + 9 | 0,0093 | **0,0105** | 0,0053 | 0,0098 |
| a 10 + 9 com o `c3_rebote` SEM a V2 | 0,0056 | 0,0071 | 0,0030 | 0,0063 |

- Alto e Médio: todas as variantes iguais à lei W6d até ao 4.º decimal. **O gabarito a 0,005 m é o de 0,01 m** (convergido): não
  há canto mais exato a ver — a 10 + 9 piorava os QUATRO pontos, o do meio da sala incluído (um efeito global da fonte).
- **O mecanismo:** a fonte do rebote (`c3_rebote`, a luz que cada superfície recebe) é amostrada na grade GROSSA das cascatas
  (a c0 a 4 px no Baixo) e lia o `meio` com a cunha da captura FINA: junto aos cantos côncavos da sala a geometria da
  amostra não é a da cunha.
- **A cura (no produto):** o `c3_rebote` compila com `BORDA_CANTOS = 0` (a constante própria vence a comum) — o
  `TOL_G9_HOLO` NÃO mudou.
- **A 10 + 9 no padrão:** `borda_vertice: Some(VERTICE_PADRAO)` + `canto: Canto::LIGADO` + `borda_cantos: true`.
- **Dois defeitos que o padrão acordou** (a 10 + 9 nunca tinha nascido sob a placa recusada nem sem a holográfica):
  as listas dos cantos nasciam MAPEADAS para escrever o cabeçalho `[0, 9, 1]` — com a memória recusada o buffer nasce
  inválido e o `get_mapped_range` rebentava (`os_tetos_da_placa::a_luz_recua_quando_a_placa_recusa_a_memoria`, sozinho);
  cura: o `c3_cantos_zera` escreve o cabeçalho a cada gravação (a lista «anterior» do 1.º quadro é zero: o indireto é
  vazio). E o G5: a tabela vazia (16 B) não declarada — agora uma por sítio, declarada (`tabelas_vazias`: C2, holográfica,
  C3 sem os cantos).
- **Medido com o padrão novo:** 0 bytes diferentes em 32 (`dump_r9b`); **as duas crates pelo nextest (as faixas da luz)
  140/140**. ⚠️ Pelo `cargo test` puro (sem as faixas, como o `fecho_r8.sh`) o
  `a_nitidez_da_luz::com_o_automatico_desligado_o_custo_e_o_do_nivel_escolhido` reprova por relógio (Low 7,27 / 35,05 ms
  contra 2,37 sozinho): os sete que morriam em `SemMemoria` na r8 agora correm e disputam a placa — o portão de
  verdade é o nextest.

## 29. ABERTO (por ordem — substitui o §24)

1. Smoke do dono (`PH2D_LIGHT2D_SMOKE=5`/`6`: os cantos vivos agora no padrão).
2. A lei W6d com `DISTANTES` no relógio (o §20.2). 3. A anomalia da r5b (§12). 4. A família (b) (37–38): da W5g, fora.
5. Os zera/limpa por um carimbo de quadro (~0,008 ms) — só se o C3 voltar à linha dos +5 %.

## 30. O bug 6 do smoke (20/10) — medido; a causa é o ENTORNO, não a lâmpada

O smoke do §29.1 passou nos cantos vivos e abriu o bug 6 (`BUGS_luz2d.md` §6: com o zoom que tira a lâmpada da vista,
a sombra e a face do L mudam). Tudo FORA do git: `target/bug6/bug6_regua.rs` (cinco réguas `#[ignore]`, para pôr em
`tests/it/` e no `main.rs` de volta) e os logs ao lado. Nenhum código do produto mudou.

| Régua (`bug6_regua::…`) | O que mediu |
|---|---|
| `bug6_a_lampada_atravessa_a_borda_da_grade` (linha e um `git archive main` em `target/`, fora) | linha = `main` ao 3.º decimal em toda vista: o bug é da W6a, não da W6b–g |
| `bug6_a_lampada_anda_para_fora_da_grade` (`PH2D_BUG6_ALTURA=3.0`/`2.0`) | câmera parada, lâmpada de x = −1 a 5 m: erro contra o gabarito CONTÍNUO na borda (0,3036 → 0,2966) |
| `bug6_o_caminho_do_app` (estufa, Baixa, sprites por texel, 1920 e 2560×1080) | igual ao síncrono; na vista do dono a lâmpada nem sai da grade; o salto é na OITAVA (h ≈ 4,2 m) |
| `bug6_a_grade_esticada` | a mesma vista com a grade sobre a sala inteira: o entorno erra +69 % (sombra à esquerda da haste) e +141 % (abaixo da base); sem os refletores de fora, 0 % |
| `bug6_o_entorno_tapa` | só o L e um corpo que brilha fora da grade: atrás do L a vista + entorno dá 0,047 contra 0 (24 % do valor sem o L); entorno 4× 0,028 · 2× 0,018; sem `miolo` igual |

As fotos do dono lidas nos mesmos pontos do mundo (sRGB → linear, a sombra junto ao L): +80/+34/+32 % — a régua dá
+67/+40/+38 % na troca de oitava. ⚠️ À parte, e não deste bug: a cena 5 inteira na grade que cobre tudo fica +38 % de
vermelho na sombra contra o gabarito `Rebote` (os corpos que recebem luz por dentro × o gabarito de fronteira) — por
medir se é o gabarito ou o produto, antes de o usar como teto.

## ⛔ Recusas MEDIDAS (§30)

| Recusa | Porquê |
|---|---|
| Curar o bug 6 na lâmpada (`distantes`, `ForaDaVista`) | A lâmpada atravessar a borda é contínuo; na vista do dono ela nem sai da grade. |
| Afinar o entorno (8× → 4× ou 2×) como cura | A fuga cai 0,047 → 0,028 → 0,018, não some, e a memória cresce com o quadrado. |

## 31. ABERTO (por ordem — substitui o §29)

1. **O bug 6:** porque é que o entorno deixa passar 24 % da luz de fora através de um opaco da vista, quando o mesmo
   HRC na grade da vista tapa a 0 (a pista: só o entorno, com `so_a_c0`, `mascarar_miolo`, a c0 somada na saída do
   `c2_hrc`) → a cura → os gates permanentes (a cena mínima, teto perto de 0 com o controlo sem o L; a cena 5 na
   troca de oitava contra a grade esticada) com prova de mutação. Candidata de desenho se a fuga for do método: a luz
   de fora a ENTRAR pela borda da grade da vista (o céu de cada raio que sai = a radiância do entorno naquela
   direção), tapada pelos opacos finos da vista.
2. A lei W6d com `DISTANTES` no relógio (o §20.2). 3. A anomalia da r5b (§12). 4. A família (b) (37–38): da W5g, fora.
5. Os zera/limpa por um carimbo de quadro (~0,008 ms) — só se o C3 voltar à linha dos +5 %.

## 32. O bug 6 — o MECANISMO bissectado (20/10) e a cura escrita ANTES de mexer

Réguas: `crates/ph2d-app-light2d/tests/it/bug6_regua.rs` (as cinco do §30 + `bug6_bissecta_o_entorno`,
`bug6_bissecta_a_vista`, `bug6_a_parede_fina`; `#[ignore]`, só leitura das texturas, nenhum código do produto),
logs em `target/bug6/bug6_bissecta*.log` e `bug6_parede_fina.log`. Cena mínima do §30 (o L + o corpo que brilha
fora da grade), ponto «atrás da haste» (−4,2; −2,0), canal vermelho.

| Variante (mesmo processo) | c0 do entorno ali | c0 da vista | fluência |
|---|---|---|---|
| entorno 8× (a base) | **0,0000** | 0,0476 | 0,0473 |
| entorno 4× · 2× · 1× | 0 · 0 · 0 | 0,0277 · 0,0182 · **0** | igual |
| entorno 8× com a fina (`so_a_c0 = false`) | 0 | 0,0476 | 0,0473 |
| **vista SEM o rebote**, entorno com ou sem | 0 | **0,0000** | **0,0000** |
| vista com o rebote, entorno SEM o rebote | 0 | 0,0476 | 0,0473 |

1. **O entorno TAPA** (a c0 dele é 0 atrás do L em todo fator): a fuga não está nele, nasce na VISTA.
2. **É o REBOTE da vista** (sem ele, 0 ao 4.º decimal). A c0 do entorno guarda luz DENTRO dos opacos (0,147 no
   interior do L, alfa 0) e a soma do `c2_hrc.wgsl` (saída) amostra-a com o sampler `Linear` SEM o alfa: a até um
   texel do ENTORNO (8 da vista) da face, os texels de FORA da vista recebem a luz do interior (0,0092 a 0,1 m da
   face de trás, sem rebote) — e a c0 da vista tem lá alfa 1. O `c4_rebote` acha nesses texels a luz da face de
   TRÁS do L, que passa a brilhar e ilumina a sombra. Cai com o fator (4×, 2×) porque a faixa encolhe; a 1× os
   centros coincidem e some.
3. **Há uma 2.ª fuga, do MÉTODO** (`bug6_a_parede_fina`, uma parede no lugar do L, vista sem rebote): com 0,06 m
   ou mais o entorno tapa a 0; com **0,03 m** (dois texels da vista, ¼ do texel do entorno) o entorno não a vê e
   a luz de fora passa a **90 %** (0,166 contra 0). Uma c0 (fluência, sem direção) somada depois dos raios da vista
   não pode ser tapada por um opaco que só a vista resolve. Com o rebote, ~0,043 atrás de toda parede (a fuga 2).

### A cura (escrita antes de mexer): a luz de fora ENTRA PELA BORDA

A fuga 2 só por si curava-se com o alfa na amostra; a 3 não — o padrão-ouro é o método, que cura as duas:

- **O entorno guarda os R de todos os níveis** (`Config::arquivo_da_borda`, posto pelo `config_do_entorno`;
  `PH2D_LIGHT2D_BORDA=0` volta à soma da c0 para comparar): o `hrc_desce` escreve cada R_n(xi, v, m) também num
  buffer `luz2d/buf/hrc_arquivo` (todos os quadrantes, níveis 1..N−1; declarado na `PassList`, G5).
- **A vista lê-o onde hoje lê o CÉU:** o `r_de_cima` responde `céu × arco` quando a sonda de cima está fora do
  que se guarda (o raio saiu da grade) ou no topo. Com o entorno: a sonda `e` recua pela direção do centro do cone
  até à borda da grade (`b`, o ponto onde a luz de fora entra), `b` passa à grade canónica do entorno no MESMO
  quadrante, e o R dele no MESMO cone (o mesmo nível: os cones não dependem do texel) sai pela bilinear das sondas
  do entorno; acima do topo do entorno, o cone-pai × `arco(n, j)/arco(K, pai)`; fora do entorno, o céu. A luz de
  fora passa então pelos opacos FINOS da vista (os T dela até `b`) e o `c0_em` dela mascara o interior — as duas
  fugas fecham pelo mesmo caminho.
- **A soma da c0 na saída sai** no modo da borda (fica só para `PH2D_LIGHT2D_BORDA=0`). O miolo continua
  mascarado no entorno (o `o_entorno_nao_conta_duas_vezes` guarda-o).
- **Custo previsto:** o arquivo, Σ quadrantes Σ níveis `nx·Y·2ⁿ·8 B` — no entorno 8× de 1920×1080 (256×160)
  ≈ 9,7 MB; o tempo, uma escrita a mais por R no entorno e a bilinear só nos ramos do céu da vista. A medir nos
  portões dos tetos da placa.
- **Gates (com prova de mutação):** «a luz de fora não atravessa um opaco da vista» — a cena mínima (o L) e a
  parede de 0,03 m, teto perto de 0, controlo = sem o opaco (a luz chega); a cena 5 na troca de oitava contra a
  grade esticada. Continuam verdes: `o_rebote_de_fora_da_grade`, `o_entorno_nao_conta_duas_vezes_…`,
  `a_luz_num_ponto_nao_muda_com_a_vista*`.

## 33. A cura do bug 6 — a luz de fora ENTRA PELA BORDA (20/10, feita e medida)

O desenho do §32 com três correções que a medição obrigou (cada uma com a régua que a mostrou; réguas em
`target/bug6/bug6_regua.rs`, fora do git pelo tecto de LOC, `#[ignore]`; logs `target/bug6/borda_*.log`):

1. **O arquivo sai de uma 2.ª passagem do entorno com o miolo VÁCUO** (`meio_do_traco`: na passagem com
   `arquivo.w = 1`, o miolo nem tapa nem brilha). Com a passagem normal, as sondas do entorno nos níveis altos
   (16 m umas das outras) caíam dentro da vista e o R delas vinha tapado pela versão GROSSA do L: o lado aceso
   ficou a 0,004 contra 0,171. A 1.ª passagem fica como era (a c0 dele = o rebote dos refletores de fora, tapado
   pelos objetos da vista).
2. **A vista lê a coluna de sondas mais perto ATRÁS da borda, sobre o raio** (nunca a bilinear em `u`): a sonda
   além da borda perdia o que fica entre as duas (o corpo que brilha a 2–3 m da borda): 0,067 contra 0,171. O
   nível é o mais fino em ângulo cuja coluna cabe na vista (`fora.z`, da CPU, sem laço no shader).
3. **O céu passa ao entorno** (`quadro.rs`): a W5d tirava-o dele porque a soma o contava DUAS vezes (medido:
   1,256 contra 2π·céu = 0,628 na sala vazia); pela borda o céu da vista é o que o entorno lhe dá (0,6284).

E dois cortes de custo: **o arquivo guarda só a JANELA da vista** (`hrc::janelas_do_miolo`, as sondas sob a
grade dela + uma linha de cada lado; `Config::arquivo_da_borda = Some(janela máxima)`) e a radiância MÉDIA do
cone (R / arco). `PH2D_LIGHT2D_BORDA=0` volta à soma da W5d.

| Medido (cena mínima, vermelho) | soma (W5d) | borda | esticada | gabarito |
|---|---|---|---|---|
| atrás do L (−4,2; −2,0) | 0,047 | **0,0000** | 0 | 0 |
| atrás de uma parede de 0,03 m | 0,166 | **0,0000** | — | 0 |
| sem o opaco (o controlo) | — | 0,1364 | — | 0,1387 |
| acima do L | 0,191 | 0,1719 | 0,1765 | 0,1711 |

| Cena 5 no app, h = 3,8 (texel 0,0156) contra a esticada | soma (W5d) | borda |
|---|---|---|
| sombra à esquerda da haste (R/G/B) | +69/+41/+39 % | −14/−1/−1 % |
| sombra abaixo da base | +141/+75/+43 % | −11/−3/−1 % |
| faces de cima (haste, base) | −1…−5 % | −10…−11 % (R) |
| só o L com o ambiente (sombra) | — | +0 % |
| do outro lado da oitava (h = 4,27, texel 0,0312) | — | 0,0 % |

Os −10 % no vermelho das faces são o rebote dos refletores de fora (sem eles: 0 %). A esticada fica +38 % de
vermelho acima do gabarito `Rebote` na mesma cena (§30, por medir) — o desvio da borda aponta para o gabarito,
não para a esticada; antes de o usar como teto, mede-se quem está certo (§34.2).

**Custo** (`bug6_o_custo_da_borda`, `bug6_as_fases_da_borda`; variantes no mesmo processo, intercaladas, o mínimo):

| | soma | borda |
|---|---|---|
| memória do entorno, 1920×1080 Baixa / Média / Alta | 10,1 / 35,4 / 125,2 MB | +2,4 / +9,7 / +37,5 MB (+0,6 / +1,6 / +3,1 % do par) |
| idem 2560×1440 | 18,3 / 61,3 / 244,1 | +4,3 / +17,6 / +66,0 (+0,6 / +1,7 / +3,2 %) |
| idem 3840×2160 | 35,4 / 125,2 / 509,5 | +9,7 / +37,5 / +155,9 (+0,6 / +1,7 / +3,1 %) |
| placa, entorno 1920×1080 / 2560×1440 (padrão) | 0,632 / 0,723 ms | 0,948 / 1,105 ms |
| placa, vista (c2) 1920×1080 / 2560×1440 | 7,707 (1,528) / 9,902 (2,164) | 8,064 (1,890) / 10,495 (2,752) |

O arquivo inteiro (sem a janela) custava +8…+19 % (814 MB no 4K Alta); com o laço do nível no shader a `c2` da
vista pagava +0,55 ms. O resto do custo está nos dois níveis de cima da descida (n = 8, 9: +0,10 e +0,08 ms em
1920×1080), onde quase todo cone sai da grade.

**Gates** (`tests/it/a_luz_de_fora_nao_atravessa.rs`): `a_luz_de_fora_nao_atravessa_um_opaco_da_vista` (o L e a
parede de 0,03 m: atrás ≤ 0,002, medido 0,0000; o controlo sem o opaco ao gabarito ±5 %, medido −0,1…−1,8 %),
`a_luz_de_fora_passa_a_fenda_perto_da_borda` (placas de 0,25 m a 0,5 m da borda da vista com uma fenda de 0,06 m:
≥ ½ do gabarito, medido +14…+18 %) e `a_cena_5_na_troca_de_oitava_segue_a_grade_esticada` (h = 4,27 e 3,8, ±20 %
por canal). Continuam verdes: `o_rebote_de_fora_da_grade` (com o `o_entorno_nao_conta_duas_vezes_…`),
`a_luz_num_ponto_nao_muda_com_a_vista*`, `os_tetos_da_placa` (20/20 no app), G5 e `architecture_workspace_file_loc_cap`
(o `solver_hrc.rs` 799 → 686: o entorno, o miolo e a borda no filho `solver_hrc_entorno.rs`).

**Prova de mutação** (cada uma sozinha, a árvore reposta):

| Mutação | Morre em |
|---|---|
| M1 a borda desligada (`de_fora` → céu) | o opaco (0,0000 contra 0,1387 sem ele) e a cena 5 |
| M2 o miolo NÃO vácuo na passagem do arquivo | a fenda (0,0009 contra 0,0343; limpa 0,0400) — sobrevivia aos outros dois: só um opaco entre a coluna lida e a borda a vê |
| M3 a coluna ALÉM da borda | o opaco e a cena 5 |
| M4 a soma da c0 de volta, além da borda | o opaco (atrás do L 0,0473) e a cena 5 |
| M5 o entorno sem o céu (o da W5d) | a cena 5 (0,1304 contra 0,2267) |
| M6 o arquivo não se escreve | o opaco e a cena 5 |

## 34. ABERTO (por ordem — substitui o §31)

1. **O custo que fica:** a `c2` da vista +0,36 ms (1920×1080) / +0,59 ms (2560×1440), quase todo nos dois níveis
   de cima da descida (quase todo cone sai da grade); o entorno +0,32 / +0,38 ms (a 2.ª passagem inteira — ela só
   precisa da parte da grade dele a partir da janela no sentido do quadrante). A régua: `bug6_o_custo_da_borda` e
   `bug6_as_fases_da_borda` em `target/bug6/bug6_regua.rs`.
2. **Quem está certo no rebote de fora:** a cena 5 com a grade a cobrir tudo fica +38 % de vermelho contra o
   gabarito `Rebote`, e a borda fica −10 % contra a esticada nas faces (só o rebote dos refletores de fora) — medir
   o gabarito convergido antes de usar qualquer dos dois como teto.
3. **O smoke do dono (20/10): «sombra ok. laranja (influência do objeto superior) diminui»** (fotos em
   `docs/Luz2D/smoke/2026-10-20_bug6_cura_foto*.png`). É o item 2 visto na tela: o rebote do refletor de cima,
   quando ele sai da grade e passa ao entorno, chega −10…−14 % (vermelho) contra a esticada. Por bissectar, com as
   réguas de `target/bug6/` (a `bug6_a_grade_esticada`, casos «cena inteira»/«sem o disco»/«só o L e a cruz»):
   (a) a fonte do rebote dele no entorno (a c0 da 1.ª passagem, grossa, com o miolo mascarado — ele não recebe a
   luz que a vista emite nem a que ela devolve); (b) o nível capado (`fora.z`) e o cone-pai na leitura pela borda;
   (c) o gabarito convergido (o item 2) — qual das três explica os −10 %, antes de mexer.
4. A lei W6d com `DISTANTES` no relógio (o §20.2). 5. A anomalia da r5b (§12). 6. A família (b) (37–38). 7. Os
   zera/limpa por um carimbo de quadro — só se o C3 voltar à linha dos +5 %.

## ⛔ Recusas MEDIDAS (§33)

| Recusa | Porquê |
|---|---|
| Curar só com o alfa na amostra da c0 do entorno | Fecha a fuga do rebote, não a da parede fina (90 % atravessa: o entorno não a vê). |
| Ler o R do entorno pela bilinear das sondas em `u` | A sonda além da borda perde o corpo que brilha entre as duas: o lado aceso a 39 %. |
| Arquivar a passagem NORMAL do entorno | As sondas dentro da vista vêm tapadas pelo L grosso: o lado aceso a 2 %. |
| O arquivo de todos os níveis na grade inteira do entorno | +8…+19 % de memória (814 MB no 4K Alta); a janela dá +0,6…+3,2 %. |
| O laço que procura o nível no shader | +0,19 ms na `c2` da vista em 1920×1080 pelo mesmo resultado; o nível vem da CPU. |
| Esperar que guardar a média (R / arco) poupe tempo | Medido: o tempo não muda — o custo é a quantidade de cones que saem da grade, não a leitura. |

## 35. O laranja do refletor de cima (§34.3) — bissectado e curado (20/10)

Régua `bug6_o_laranja` em `target/bug6/bug6_regua.rs` (fora do git, `#[ignore]`; para correr, copiá-la para
`tests/it/` e pôr `mod bug6_regua;` no `main.rs`), logs `target/bug6/laranja_{1,2}.log` (antes / depois). Cena 5,
h = 3,8, a vista `[-9, -5, 1, 1]` m: o Disco (y 1,0…2,4) fica TODO fora dela, encostado à borda de cima. Três
secções, nada do produto mexido: S1 a parte do Disco (cena − sem o Disco) vista × esticada × gabarito, o entorno
8×/4×/2×/1×; S2 a FONTE do rebote na face acesa do Disco; S3 o Disco trocado por um EMISSOR (só a leitura pela borda).

| Suspeito (§34.3) | Medido | Veredito |
|---|---|---|
| (b) o nível capado / o cone-pai na leitura pela borda | S3: vista = esticada a ±0,4 % com o entorno 1×; +2…+11 % a 8× (a cavalo ou todo fora) | não perde luz |
| (c) o gabarito | S1: a parte do Disco na esticada = gabarito em 5 de 6 pontos (0,96/0,99 · 0,653/0,646 · 0,154/0,151); a sombra à esquerda da haste é o raio rasante na quina da haste (0,125 contra 0,0185 — o +38 % do §30, à parte) | a esticada está certa |
| **(a) a fonte do rebote no entorno** | S1: a parte do Disco na vista = **0,66–0,76** da esticada em TODO fator (8× a 1×); S2: a luz que a face recebe no entorno 0,66 contra 0,94 (esticada) e 0,96 (gabarito com rebotes; direta 0,385) | **é este** |

**O mecanismo:** a 1.ª passagem do entorno (a c0 dele, que só alimenta o rebote DELE) mascarava o miolo — herança
da W5d, onde essa c0 se somava à da vista e a máscara evitava contar a vista duas vezes. Pela borda essa soma não
existe; a máscara só escondia dos refletores de fora a luz que a vista emite e DEVOLVE (a cruz branca junto à
lâmpada, o L): o Disco recebia metade do indireto (0,27 de 0,57).

**A cura** (`Holografica::miolo`, `solver_hrc_entorno.rs`): com o arquivo da borda, só a 2.ª passagem (o arquivo,
`arquivo.w = 1`, o miolo VÁCUO) leva o miolo; a 1.ª fica `SEM_MIOLO`. A soma da W5d (`PH2D_LIGHT2D_BORDA=0`) continua
a mascarar. Custo: nenhum (a mesma traça; um ramo). Não conta duas vezes: o arquivo, que é o que a vista lê, segue
com a vista em vácuo.

| Depois (vermelho, contra a esticada) | 8× (o padrão) | 4× | 2× | 1× |
|---|---|---|---|---|
| sombra à esquerda da haste | +1,4 % (era −13,7) | −5,1 % | −2,1 % | +0,3 % |
| sombra abaixo da base | −4,7 % (era −10,5) | −1,4 % | −0,3 % | 0,0 % |
| faces de cima | −4,1…−4,2 % (era −10) | −1,2…−1,4 % | −0,4…−0,6 % | 0,0…+0,1 % |
| a parte do Disco (razão) | 0,90–1,03 (era 0,66–0,76) | 0,91–0,97 | 0,96–0,97 | 1,00 |
| a face do Disco recebe (S2, −90°) | 0,884 (era 0,659; esticada 0,942) | | | |

O resto a 8× (−4 %) é a grossura do entorno e aparece igual SEM o Disco (−3…−4 %: os outros refletores) — é o
preço do fator 8×, não deste defeito. O app (`bug6_a_grade_esticada`, `LuzDoQuadro`): −14/−11/−10 % → +1/−5/−4 %.

**Gate:** `a_cena_5_na_troca_de_oitava_segue_a_grade_esticada` aperta a tolerância 0,2 → **0,07** (medido o pior
−4,7 %). **Prova de mutação** (M7 = a 1.ª passagem volta a mascarar, `if false && borda`; árvore reposta): o gate
morre a −13,7 % (`target/bug6/mutacao_m7.log`); controlo verde depois de repor (`mutacao_m7_controlo.log`).
Continuam verdes (lote único, `laranja_2.log`): `a_luz_de_fora_nao_atravessa::*` (3),
`o_rebote_de_fora_da_grade::*` (2, o «não conta duas vezes» a 0,000 %), `a_luz_nao_muda_com_a_vista::*` (6).

## ⛔ Recusas MEDIDAS (§35)

| Recusa | Porquê |
|---|---|
| Curar o laranja na leitura pela borda (o nível, o cone-pai) | Com um emissor no lugar do Disco a borda dá a esticada a ±0,4 % (1×). |
| Usar o gabarito `Rebote` como teto na sombra junto à haste | Lá ele diverge da esticada pela quina rasante (0,0185 contra 0,125); nos outros 5 pontos concorda. |
| Afinar o entorno (8× → 4×) para os −4 % restantes, nesta rodada | É a grossura de TODOS os refletores de fora (igual sem o Disco); a memória cresce com o quadrado — decisão de custo à parte (§36.2). |

## 36. ABERTO (por ordem — substitui o §34)

1. ~~Smoke do dono do laranja~~ — **«smoke ok» (20/10)**: o bug 6 fecha inteiro (a sombra e o laranja).
2. Os −4 % que ficam a 8× (a grossura do entorno sobre todos os refletores de fora; 4× → −1,3 %): medir a memória e
   o tempo do entorno 4× nos tetos da placa antes de propor.
3. O custo da borda (o §34.1): a `c2` da vista +0,36 ms / o entorno +0,32 ms em 1920×1080.
4. O emissor encostado à borda a 8× lê +2…+11 % acima da esticada (S3) — pela grossura do arquivo; por medir se pesa.
5. A lei W6d com `DISTANTES` no relógio (o §20.2). 6. A anomalia da r5b (§12). 7. A família (b) (37–38). 8. Os
   zera/limpa por um carimbo de quadro — só se o C3 voltar à linha dos +5 %.

## 37. A grossura do entorno (§36.2–§36.4) — MEDIDA, o padrão NÃO trocado (20/10, à espera da decisão do dono)

Régua `target/bug6/grossura_regua.rs` (fora do git, `#[ignore]`; copiar para `tests/it/` + `mod` no `main.rs`),
logs `target/bug6/grossura_{1,2}.log`. Cena 5, h = 3,8. Tempo: as quatro variantes (8×/4× × borda/soma) no MESMO
processo — UMA vista, os quatro entornos vivos, 6 blocos rodados, 3 quadros de aquecimento + 8 medidos, o MÍNIMO;
a fina é a que o app escolhe (`fina_que_cabe`, `k1` a 2160p). Memória = `PassList::bytes_alocados` (com a borda).

| alvo · nível | entorno MB 8× → 4× | luz MB 8× → 4× | entorno ms 8× → 4× (borda) | vista ms (igual nos dois) |
|---|---|---|---|---|
| 1920×1080 Baixa | 12,5 → 45,0 | 421 → 453 (+8 %) | 0,95 → 1,42 (+0,48) | 8,04 |
| 1920×1080 Média | 45,0 → 165 | 606 → 726 (+20 %) | 1,43 → 3,21 (+1,79) | 8,62 |
| 1920×1080 Alta | 163 → 665 | 1 265 → 1 768 (+40 %) | 3,25 → 11,40 (+8,15) | 19,24 |
| 2560×1440 Baixa | 22,6 → 78,9 | 724 → 780 (+8 %) | 1,12 → 1,90 (+0,79) | 10,41 |
| 2560×1440 Média | 78,9 → 317 | 1 029 → 1 267 (+23 %) | 1,93 → 5,50 (+3,57) | 12,19 |
| 2560×1440 Alta | 310 → 1 179 | 2 110 → 2 979 (+41 %) | 5,35 → 21,77 (+16,4) | 28,58 |
| 3840×2160 Baixa (k1) | 45,0 → 165 | 614 → 734 (+20 %) | 1,45 → 3,30 (+1,85) | 11,49 |
| 3840×2160 Média (k1) | 163 → 665 | 1 265 → 1 768 (+40 %) | 3,25 → 11,60 (+8,35) | 18,84 |
| 3840×2160 Alta (k1) | 665 → 2 677 | 4 578 → 6 589 (+44 %) | 11,63 → 52,25 (+40,6) | 77,99 |

Tudo cabe no lado (8 192) e na placa (16 GB). O 4× custa ~3,5–4,5× no entorno (o quadrado e um pouco mais), e a
vista não muda (a leitura pela borda não depende do fator: fases idênticas a ±0,003 ms).

**O erro segue o TEXEL DO ENTORNO EM METROS, não o nível** (`grossura_por_nivel`, 1920×1080, vermelho contra a
esticada do mesmo nível, os 4 pontos fora da quina rasante):

| | entorno 0,125 m | 0,0625 m | 0,0312 m | 0,0156 m |
|---|---|---|---|---|
| Baixa | 8×: −4,0…−4,7 % | 4×: −1,2…−1,4 % | | |
| Média | | 8×: −1,5…−1,8 % | 4×: −0,7…−0,9 % | |
| Alta | | | 8×: −0,8…−1,2 % | 4×: −0,5…−0,7 % |

O ponto da quina rasante (sombra à esquerda da haste, §35) oscila sem ordem (+1,4 / −5,1 / −6,0 / −3,0 / −2,9 /
−0,6 %) — é o gate de 0,07 que o cobre. ⇒ os −4 % são SÓ da Baixa a 8×; a Baixa a 4× fica com o entorno da Média
(o mesmo texel em metros, a qualquer zoom: ambos `c0_px · fator` = 16 px).

**§36.4 — o emissor encostado à borda** (`grossura_o_emissor_na_borda`, S3 com 8/4/2/1×): 8× +1,5…+11,0 % (a
cavalo) e +1,9…+8,3 % (todo fora); 4× −1,6…+1,1 %; 2× −0,8…+1,7 %; 1× ±0,4 %. A mesma família: o 4× cura-o.

**§36.3 — o custo da borda cresce com o nível e a janela** (o fator não o muda): vista +0,37 (1080 Baixa) · +0,54
(1080 Média) · +1,55 (1080 Alta) · +1,64 (1440 Alta) · +5,94 ms (2160 Alta); entorno (8×) +0,31 · +0,52 · +1,31 ·
+2,17 · +5,14 ms. Na vista está todo na `hrc_desce` (≈ +0,01–0,03 ms por nível, n = 8 +0,10, n = 9 +0,08 em
1080 Baixa); a `hrc_saida` paga MENOS com a borda (−0,004). O alvo de uma cura: a 2.ª passagem do entorno só na
parte da grade a partir da janela no sentido do quadrante (o §34.1) e os cones dos níveis 8–9 que saem da grade.

**Recomendação (para o dono decidir; nada trocado):** o entorno **4× só na Baixa** (`degrau_do` dá 4× à Baixa,
8× à Média e à Alta). Cura o −4 % e o emissor da borda onde eles existem, pelo preço menor de todos (+0,48 ms /
+33 MB a 1080p, +0,79 ms a 1440p, +1,85 ms a 2160p); a Média e a Alta já estão a ≤ −1,8 %. Recusado: o 4× em
todos os níveis (+8…+41 ms e até +2 GB na Alta por −1 % a menos).

## 38. ABERTO (por ordem — substitui o §36)

1. A decisão do dono sobre o §37 (4× na Baixa). Se sim: o gate `a_cena_5_na_troca_de_oitava_segue_a_grade_esticada`
   aperta (medido −1,4 % fora da quina), a nota do `ENTORNO_PADRAO` reconfere-se (§0.0) e o governador conta o +0,48 ms.
2. O custo da borda (§37, §36.3): +5,9 ms na vista e +5,1 ms no entorno a 2160p Alta — maior que o +0,36/+0,32 do padrão.
3. A lei W6d com `DISTANTES` no relógio (o §20.2). 4. A anomalia da r5b (§12). 5. A família (b) (37–38). 6. Os
   zera/limpa por um carimbo de quadro — só se o C3 voltar à linha dos +5 %.

## 39. O entorno da Baixa a 4× (o §37, por ordem do dono «siga como sugere», 20/10) — feito

`fceb09c79b`. `ENTORNO_PADRAO = 4.0` (o `DEGRAU_PADRAO` é a Baixa) e `ENTORNO_ACIMA_DA_BAIXA = 8.0`, que o
`degrau_do` dá à Média e à Alta; as duas notas reconferidas (§0.0) com a tabela do §37. O governador conta o entorno no
`fixo`: da Baixa à Média o texel dele é agora o MESMO (16 px) — a previsão da subida passa a exata nesse passo (era
subestimada ×4 a 8×); da Média à Alta continua ×4 por fora do `fixo` (como antes).

**Gates** (`a_luz_de_fora_nao_atravessa.rs`): `a_cena_5_na_troca_de_oitava_segue_a_grade_esticada` aperta 0,07 → **0,025**
(medido o pior −1,4 % nos três pontos fora da quina) com a quina rasante à parte em 0,07 (medido −5,1 %; oscila com a
grossura sem ordem, §37); gate NOVO `o_emissor_encostado_a_borda_segue_a_grade_esticada` (o S3: a cavalo e todo fora,
5 pontos, ±3 %, medido −1,6…+1,1 %). Os 20 da família verdes (`grossura_portoes_1.log`: `a_luz_de_fora*`,
`o_rebote_de_fora*`, `a_luz_nao_muda*`, `os_tetos_da_placa`); a fenda a 4× lê 0,0392/0,0343 (≥ ½ do gabarito).

**Prova de mutação** (cada uma sozinha, a árvore reposta, `target/bug6/mutacao_{m8,m2_4x,m8_m2_controlo}.log`):

| Mutação | Morre em |
|---|---|
| M8 a Baixa de volta a 8× | a cena 5 (−4,7 %, sombra abaixo da base) e o emissor (+11,0 %, a cavalo) |
| M2 o miolo NÃO vácuo na passagem do arquivo, a 4× | a fenda (0,0001 contra 0,0343) e a cena 5 — a fenda de 0,06 m (agora 1 texel do entorno) ainda a vê |
| controlo | os 3 verdes |

**Suíte** (`nextest-impacted`, `target/bug6/suite_37.log`): 17 790 / 17 793; os 3 vermelhos são de `ph2d-tool-painter` (a
linha não o toca), a `load` 105–127: `the_authoring_pause_stops_the_sim_…` (FAIL 1,9 s), `o_glaze_do_pincel_escurece_o_cruzamento`
(TIMEOUT 180 s) e `pular_a_pincelada_e_esperar_nao_cumpre_nenhuma` (TIMEOUT 600 s) — sozinhos, a `load` ~40, os 3 PASSAM
(1,1 · 39 · 152 s): flakes de carga (candidatos à tabela do `FLAKES_DE_CARGA.md`, decisão de quem possui o Painter). A
marca da última suíte verde NÃO avançou (a corrida saiu vermelha).

## 40. ABERTO (por ordem — substitui o §38)

1. ~~Smoke do dono do entorno 4×~~ — **«smoke ok» (20/10)**.
   O binário `smoke` está compilado na worktree (2.ª corrida: `Finished \`smoke\` profile [optimized] target(s) in 0.35s`).
2. O custo da borda (§37): +5,9 ms na vista e +5,1 ms no entorno a 2160p Alta (+0,37 / +0,31 a 1080p Baixa) — a 2.ª
   passagem do entorno só na parte da grade a partir da janela (§34.1) e os cones dos níveis 8–9.
3. A lei W6d com `DISTANTES` no relógio (o §20.2). 4. A anomalia da r5b (§12). 5. A família (b) (37–38). 6. Os
   zera/limpa por um carimbo de quadro — só se o C3 voltar à linha dos +5 %.

## 41. O custo da borda (§40.2) — bissectado; o recorte da 2.ª passagem do entorno (20/10, por ordem «vamos seguir»)

Réguas fora do git: `target/bug6/custo_borda_regua.rs` (precisa de um interruptor de diagnóstico, também fora do git e
já reposto: `fora.w = 2` = a conta do `de_fora` sem ler o arquivo, por um `AtomicU32` na `ph2d-light2d-gpu`) e
`target/bug6/recorte_regua.rs`; logs `custo_borda_1.log`, `recorte_1.log`. Cena 5, h = 3,8, mesmo processo, blocos
rodados, o mínimo.

**(a) A vista — metade conta, metade leitura, e nos níveis de CIMA:**

| `c2` da vista | céu | conta sem ler | borda |
|---|---|---|---|
| 1920×1080 Baixa | 1,528 | +0,198 | +0,396 |
| 2560×1440 Média | 3,935 | +0,385 | +0,722 |
| 3840×2160 Alta | 53,35 | +3,105 | +6,902 |

Na 2160p Alta os dois níveis de cima da descida levam +4,44 dos +6,90 ms (n = 11: 1,98 → 4,29; n = 12: 0,96 → 3,09):
lá quase todo cone sai da grade e cada fio chama o `de_fora` até 4× (as duas metades × `f0`/`f1`). Os níveis baixos
pagam +0,01–0,06 ms cada (os fios da borda divergem o warp). Não curado nesta rodada (§42.2).

**(b) A 2.ª passagem do entorno é uma passagem INTEIRA** (as fases somadas: a 2.ª = a 1.ª menos a soma e a saída):
1080p Baixa 0,61 ms (traça 0,19 · sobe 0,14 · desce 0,28), 2160p Alta 5,25 ms (traça 0,59 · sobe 1,12 · desce 3,56).
As subidas (T) NÃO se recortam: o T do nível de cima cobre a grade inteira e puxa todas as colunas e linhas abaixo dele
(a recorrência `sT(n−1) = min(…, 2·sT(n))` arredonda o começo a um múltiplo de 2^N, ⇒ 0). A descida SIM: R_n lê R_{n+1}
só para a frente (colunas ≥ xi/2) e n descidas alcançam ±(2^{n+1} − 2) linhas, e os R desta passagem só se leem no arquivo.

**A cura (`0aafae551e`):** `fora_do_arquivo` (`c2_hrc.wgsl`) — na descida da 2.ª passagem (`arquivo.z = 1`,
`Config::hrc_recorta_o_arquivo`, `PH2D_LIGHT2D_RECORTE=0` compara) sai cedo o fio fora das colunas da janela em diante
(margem 2) e das linhas dela ± 2^{n+1}. O mesmo bit. Medido (a 2.ª passagem, inteira → recortada):

| | Baixa | Média | Alta |
|---|---|---|---|
| 1920×1080 | 0,517 → 0,500 (−3 %) | 0,516 → 0,501 (−3 %) | 1,279 → 1,048 (−18 %) |
| 2560×1440 | 0,744 → 0,645 (−13 %) | 0,749 → 0,641 (−14 %) | 2,223 → 1,709 (−23 %) |
| 3840×2160 | 1,293 → 1,016 (−21 %) | 1,287 → 1,006 (−22 %) | 5,064 → 3,714 (−27 %) |

Abaixo do estimado (~⅔ da descida): nas grades pequenas a passagem está presa ao NÚMERO de despachos (4 quadrantes ×
~19 etapas ≈ 76 por passagem, ~0,03 ms por nível de descida qualquer que seja o trabalho); nas grandes, os grupos que
saem cedo ainda se lançam.

**Gate:** `o_recorte_do_arquivo_nao_muda_um_bit` (`a_luz_de_fora_nao_atravessa.rs`): a cena 5, duas câmeras na Baixa e
uma na Alta, a luz da vista com o recorte × sem ele: 0 de 3 932 160 / 3 932 160 / 3 674 112 texels. Prova de mutação
(`mutacao_m9_alcance.log`, `mutacao_m10_coluna.log`, `mutacao_m9_m10_controlo.log`):

| Mutação | Morre |
|---|---|
| M9 o alcance 2^n (metade) | 474 texels diferentes — o alcance de 2^{n+1} é o justo |
| M10 o recorte 2 colunas DENTRO da janela | 1 690 432 texels |
| controlo | 0 / 0 / 0 |

**Suíte** (`nextest-impacted`, `target/bug6/suite_41.log`): 17 794 / 17 794 verdes. O binário `smoke` recompilado (2.ª corrida: zero «Compiling»).

## ⛔ Recusas MEDIDAS (§41)

| Recusa | Porquê |
|---|---|
| Recortar as SUBIDAS (T) da 2.ª passagem | O T do nível de cima cobre a grade inteira: a recorrência leva o começo a 0 em todo nível. |
| Esperar que o recorte corte ⅔ da descida em toda janela | Medido −3 % a 1080p Baixa: o custo lá é o número de despachos, não o trabalho. |

## 42. ABERTO (por ordem — substitui o §40)

1. O número de DESPACHOS da holográfica (4 quadrantes × ~19 etapas por passagem, na vista e no entorno): é o custo fixo
   da Baixa a 1080p (a 2.ª passagem 0,5 ms quase sem trabalho). Uma etapa com os 4 quadrantes num despacho (z =
   quadrante) mexe nos uniformes por passo — medir antes o custo por despacho vazio.
2. A vista nos níveis de cima (§41 a): +4,4 ms a 2160p Alta. Candidatos por medir: o R de fora da grade de cada nível
   numa tabela calculada uma vez (hoje até 4 `de_fora` por fio, ~2× o necessário) e o despacho recortado aos fios que leem.
3. A lei W6d com `DISTANTES` no relógio (o §20.2). 4. A anomalia da r5b (§12). 5. A família (b) (37–38). 6. Os
   zera/limpa por um carimbo de quadro — só se o C3 voltar à linha dos +5 %.

## 43. O custo fixo dos despachos e a vista nos níveis de cima (§42.1–§42.2) — MEDIDO, nada no produto (20/10)

Instrumento fora do git e reposto: `target/bug6/diag_despacho_42.patch` (`DIAG_DESPACHO`: 0 normal · 1 junta = o q0 ×4
em z, os q1–q3 saltados · 5 junta DESLOCADA = cada réplica lê outras linhas (`v + r·Y/4`), sem partilhar a cache · 2 vazio
1 grupo · 3 vazio com os grupos todos · 4 sem os despachos de quadrante); régua `target/bug6/despachos_regua.rs`; logs
`despachos_1.log`, `despachos_2.log`. Cena 5, h = 3,8, o par vista + entorno com a borda, mesmo processo, blocos rodados,
o mínimo. Duas rodadas em SÉRIE: a 1.ª deu a junta com réplicas que leem a MESMA memória (otimista); a deslocada decide
se o ganho é paralelismo ou cache.

**(a) O despacho VAZIO custa 0,15–0,38 µs** (com a barreira; `(vazio 1 grupo − sem) / despachos`): os ~76 por passagem
somam ~0,02 ms. ⇒ a premissa do §41/§42.1 («o custo fixo é o NÚMERO de despachos») está REFUTADA. O que pesa é cada
despacho pequeno não encher a placa: 1080p Baixa, o entorno gasta 1,09 ms em 936 despachos de quadrante (1,16 µs cada,
dos quais 0,18 de lançamento e ~0,07 ms no total de grupos que saem cedo).

**(b) Juntar os 4 quadrantes (proxy):**

| teto | vista `c2` normal → junta / deslocada | entorno normal → junta / deslocada |
|---|---|---|
| 1920×1080 Baixa | 1,947 → −0,547 / **−0,518** | 1,437 → −0,503 / **−0,451** |
| 1920×1080 Média | 2,955 → −0,402 / −0,351 | 1,439 → −0,508 / −0,451 |
| 1920×1080 Alta | 12,02 → −0,567 / +1,129 ¹ | 3,042 → −0,756 / −0,632 |
| 2560×1440 Baixa | 2,741 → −0,558 / −0,510 | 1,824 → −0,413 / −0,282 |
| 2560×1440 Média | 4,675 → −0,405 / −0,423 | 1,829 → −0,417 / −0,286 |
| 2560×1440 Alta | 20,96 → −0,898 / −0,585 | 4,926 → −0,570 / −0,272 |
| 3840×2160 Baixa | 5,804 → −0,692 / −0,577 | 3,063 → −0,761 / −0,638 |
| 3840×2160 Média | 11,90 → −0,380 / +1,320 ¹ | 3,053 → −0,760 / −0,633 |
| 3840×2160 Alta | 52,35 → −1,123 / −0,627 | 10,05 → −1,423 / −1,310 |

¹ a grade 2496×1472: a réplica deslocada cai em linhas sem o vazio que o q0 salta — artefacto do proxy. O ganho
SOBREVIVE à deslocada (paralelismo, não cache): ~−1,0 ms por quadro a 1080p Baixa (vista + entorno, ~28 %). O proxy
corre 4× o q0 (que tem ≥ os níveis do q1), logo o trabalho dele ≥ o real.

**O preço é MEMÓRIA** (os T de todos os níveis e o par R, por quadrante; hoje são partilhados em série):

| grade | hoje | juntos |
|---|---|---|
| vista 640×384 (1080p Baixa) | 33 MiB | 124 MiB |
| vista 1280×768 (1080p Média, 2160p Baixa) | 141 | 524 |
| vista 2496×1472 | 563 | 2 106 |
| vista 4992×2944 (2160p Alta) | 2 364 | 8 873 |
| entorno 512×320 / 992×576 / 1984×1152 | 21 / 81 / 342 | 77 / 295 / 1 248 |

⇒ a cura real (os 4 quadrantes por etapa, `z` = quadrante, um jogo de T/R por quadrante, a soma na ordem q0..q3 — o
mesmo bit) só cabe abaixo de um orçamento de memória; nas grades grandes fica a série. NÃO construída: à espera do dono.

**(c) §42.2 — a tabela do R de fora: RECUSADA pela contagem** (`conta_de_fora.py`, exata, as chamadas do `soma_da_descida`
e os valores distintos que elas pedem): a razão chamadas / distintos é 1,21–1,32 nos níveis de cima (onde está o custo:
n = 11 de 3840×2160, 26,5 M / 21,9 M; o transposto 47,2 M / 35,7 M) e 1,1–1,7 nos de baixo — não os «~2×» do §42.
Com ~55 % do custo do `de_fora` em leitura (§41 a), uma tabela troca cada chamada por uma leitura e ainda escreve os
distintos: a 1,2× sai mais cara. O «despacho recortado aos fios que leem» é a mesma tabela só nos distintos — a
contagem já é essa.

**(d) Achado de passagem — os cones por fio, nível a nível** (`hrc_cones_por_fio` 1 × 4, fases da descida da vista com a
borda, ms): a 3840×2160 Alta o 1 ganha nos níveis 0–8 (n = 0: 2,068 contra 2,795; 1–8: −0,01…−0,25) e perde nos 9–12
(+0,09 · +0,10 · +0,29 · +0,21); a 1920×1080 Baixa o 1 ganha em todos (2,006 contra 2,131); a 2560×1440 Média empata (o
1 só ganha em n = 0–1). Um corte por nível (o `por_fio = 0` no uniforme da descida — a mesma conta do
`soma_da_descida`, por provar bit a bit) daria ~−1,5 ms a 2160p Alta (~3 %). Medido em modo de fases (fronteira de passe a
mais); confirmar pela `c2` antes.

## ⛔ Recusas MEDIDAS (§43)

| Recusa | Porquê |
|---|---|
| Cortar o NÚMERO de despachos por si | 0,15–0,38 µs cada; ~0,02 ms por passagem. |
| A tabela do R de fora (e o recorte dela) | Chamadas / distintos 1,2–1,3 onde está o custo: a tabela custa mais do que poupa. |

## 44. ABERTO (por ordem — substitui o §42)

1. **Decisão do dono:** os 4 quadrantes num despacho, só onde a memória ×3,7 cabe (~−1,0 ms por quadro a 1080p Baixa).
2. Os cones por fio por nível (§43 d): confirmar pela `c2` e provar o mesmo bit.
3. A vista nos níveis de cima (+4,4 ms a 2160p Alta) continua sem cura que dê o mesmo bit.
4. A lei W6d com `DISTANTES` no relógio (o §20.2). 5. A anomalia da r5b (§12). 6. A família (b) (37–38). 7. Os
   zera/limpa por um carimbo de quadro — só se o C3 voltar à linha dos +5 %.

## 45. Os 4 quadrantes num despacho (§44.1, ordem do dono 21/10 «construa… ligado só em low») — feito

`024d51d4c7` + `5933700879`. Com `Config::hrc_juntos` (o `Degrau::quadrantes_juntos`, só na Baixa; `PH2D_LIGHT2D_JUNTOS=0`
compara) cada etapa da holográfica (T0..TN, R_{N−1}..R0, nas duas passagens do entorno) é UM despacho dos 4 quadrantes:

- **Memória:** cada quadrante tem a sua FAIXA em cada nível (`hrc::faixas`, alinhada a 64; os T e o par R dimensionam-se
  pela soma das faixas). As `bases` do `Hrc` (as ligações `t_a`, `t_b`, `r_cima` e a escrita) somam-se ao índice nos
  acessores dos dois prelúdios (`base_de`, só com `JUNTOS_HRC`).
- **Uniforme:** `var<uniform> hrcs: array<Hrc, 4>` — o `Hrc` passa a 256 B (= `PASSO_UNIFORME`) e a ligação de todo passe
  lê quatro fatias seguidas (o `n_passes` conta-as e mais 3 no fim). Fora do despacho junto o índice é a CONSTANTE 0
  (`iq()`).
- **O quadrante do fio:** `id_do_fio` — o grupo (dobrado em 2D) cai no quadrante com o maior 1.º grupo ≤ g (`grupos.w`)
  e decompõe-se nos `[⌈Y/64⌉, nx, k]` dele; o quadrante sem este nível tem 0 grupos. Só em 3D com um fio por sonda.
- **A soma** continua um quadrante de cada vez, na ordem q0..q3 (o acumulado: o mesmo bit). **O topo** de um quadrante
  não é o de outro numa grade não quadrada: junto, a descida liga sempre o R de cima (o topo não o lê).
- **O teto** (`teto::config_do_quadro`) conta os bytes dos quadrantes juntos.

**`JUNTOS_ATE` = 589 824 texels** (medido, `juntos_2.log`/`juntos_3.log`, a Baixa, cena 5, um a um → juntos, ms):

| teto (vista) | vista `c2` | entorno | soma |
|---|---|---|---|
| 1920×1080 (640×384) | 1,908 → 1,494 | 1,392 → 1,037 | **−0,769** |
| 2560×1440 (832×512) | 2,667 → 2,305 | 1,798 → 1,507 | **−0,653** |
| 3440×1440 (1120×512) | 3,684 → 3,368 | 2,012 → 1,763 | −0,565 |
| 3000×1700 (992×576) | 3,654 → 3,395 | 2,073 → 1,816 | −0,516 |
| 3200×1800 (1056×608, a vista um a um) | 4,392 → 4,412 | 2,114 → 1,861 | −0,233 |
| 3840×2160 (1280×768, a vista um a um) | 5,717 → 5,728 | 2,946 → 2,795 | −0,140 |

Sem o corte a vista de 1280×768 perdia +0,29 ms (juntos os quatro só disputam a memória numa grade que já enche a placa).

**O caminho um a um não mudou** (`juntos_serie_{1,2,3}.log`, o shader de antes do §44 pelo `modulo_rc` no MESMO processo):
0 texels diferentes nas 9 combinações (3 tetos × 3 níveis); o tempo empata (+0,2 · +0,1 · 0,0 % na Média). As diferenças
de +1,5…+15 % que apareceram seguem a ORDEM de criação do solver, não o shader (a 2160p Alta: 52 ou 60 ms conforme o
solver nasce em 1.º ou 2.º, nos dois sentidos) — o regime da memória da W5p/W5x.

**Gate:** `os_quadrantes_juntos_nao_mudam_um_bit` (`a_luz_de_fora_nao_atravessa.rs`): a cena 5, a Baixa, duas câmeras
deitadas e uma janela EM PÉ, a vista e o entorno juntos × um a um: 0 de 3 932 160 texels nas três; e o corte (a vista de
2160p um a um, o entorno dela junto; a Média e a Alta nunca juntam). Prova de mutação (`target/bug6/mutacao_juntos.sh`,
`mutacao_m1{1,2,3,4}_*.log`, controlo `mutacao_juntos_controlo.log`):

| Mutação | Morre |
|---|---|
| M11 o último quadrante nunca escolhido | 1 690 432 texels |
| M12 sem as faixas (os quadrantes uns por cima dos outros) | 1 690 432 texels |
| M13 o topo liga o vazio também junto | só a janela EM PÉ: 536 800 texels |
| M14 `JUNTOS_ATE` = 2^21 | «a vista de 2160p passa o JUNTOS_ATE» |
| controlo | 0 / 0 / 0 |

**Suíte** (`nextest-impacted`, `target/bug6/suite_44.log`): 17 794 / 17 795 — o vermelho era o tecto de 700 LOC do
`solver_hrc.rs` (761); curado movendo os ajudantes de ligação e grupos para `solver_hrc_ligar.rs` (698), o gate verde por
nome. Os 14 de `teto|memoria|recua|placa` verdes por nome depois do último commit.

## ⛔ Recusas MEDIDAS (§45)

| Recusa | Porquê |
|---|---|
| Juntar os quadrantes em toda grade da Baixa | A vista de 1280×768 (2160p) +0,29 ms; acima de ~600 mil texels a placa já está cheia. |
| Juntar na Média/Alta | A memória ×3,7 (+384 MiB a +6,5 GiB) e as grades passam o `JUNTOS_ATE`. |
| Juntar também a soma | O acumulado soma os quadrantes por ordem; num despacho a ordem (e o bit) perde-se. |

## 46. ABERTO (por ordem — substitui o §44)

1. Smoke do dono: a luz da Baixa igual com e sem (`PH2D_LIGHT2D_JUNTOS=0`).
2. Os cones por fio por nível (§43 d): confirmar pela `c2` e provar o mesmo bit.
3. A vista nos níveis de cima (+4,4 ms a 2160p Alta) continua sem cura que dê o mesmo bit.
4. A lei W6d com `DISTANTES` no relógio (o §20.2). 5. A anomalia da r5b (§12). 6. A família (b) (37–38). 7. Os
   zera/limpa por um carimbo de quadro — só se o C3 voltar à linha dos +5 %.

## 47. Os cones por fio por nível (§46.2) — pela `c2`: só o nível 0, feito (21/10)

`53142e3f58`. `hrc::DESCIDA_UM_CONE = 1`: o nível 0 da descida vai pelo `soma_da_descida` (`por_fio = 0` no uniforme
dele) também acima do `POR_FIO_DESDE`. O nível 0 tem UM cone: os mesmos fios, menos código. O mesmo bit.

Instrumento no produto: `Solver::diag_cones_um(gpu, máscara)` (bit `n` = o nível `n` por um cone; `0` = todos pelos
cones, o antes) reescreve o `por_fio` e os grupos da descida — também no despacho junto (o `primeiro` de cada quadrante
e o `dobrar`) — sem recompilar. Régua `target/bug6/cones_nivel_regua.rs` (logs `cones_nivel_{1,2,3}.log`).

**Três rodadas, em série por necessidade** — a régua mentia nas duas primeiras:

1. Variantes de 8 quadros, blocos rodados, o mínimo (`_1`): toda máscara MISTA saía ~+25 % (1080p Alta +2,9 ms,
   2160p Média +3,1) e só a base e o «todos» rápidos. Não era o código.
2. A base ao lado de cada candidata, com a série dela (`_2`): a base dá 59,2 ms na 1.ª visita e 74 em TODAS as outras
   (2160p Baixa 5,75 → 7,04 · 2160p Média 11,8 → 15,4 · 1440p Alta 20,4 → 26,4), com os relógios fixos (2 827 /
   14 001 MHz, `cones_nivel_2_relogios.csv`). Quem caía na 1.ª visita «ganhava».
3. **As variantes trocadas A CADA QUADRO** (base · base · n0 · 0..=1, a ordem a rodar no ciclo, 60 ciclos), a mediana
   da diferença contra a base do MESMO ciclo (`_3`) — o controlo base × base dá ≈ 0 em todo teto:

| teto | n0 vista `c2` (q1 · q3) | n0 entorno | 0..=1 vista | 0..=1 entorno |
|---|---|---|---|---|
| 1920×1080 Baixa | −0,007 (−0,013 · −0,003) | −0,003 | −0,003 | −0,002 |
| 1920×1080 Média | −0,011 | −0,004 | −0,010 | −0,004 |
| 1920×1080 Alta | −0,096 (−0,130 · −0,051) | −0,009 | −0,140 | −0,004 |
| 2560×1440 Baixa | −0,017 | −0,007 | −0,006 | +0,001 |
| 2560×1440 Média | −0,022 | −0,002 | −0,012 | −0,003 |
| 2560×1440 Alta | −0,179 (−0,278 · −0,086) | −0,019 | −0,195 | −0,002 |
| 3840×2160 Baixa | −0,022 | −0,016 | −0,020 | −0,001 |
| 3840×2160 Média | −0,127 | −0,011 | −0,155 | −0,003 |
| 3840×2160 Alta | **−0,549** (−0,866 · −0,105) | −0,063 | −0,622 | −0,020 |

O n0 ganha nos nove, vista e entorno; o 0..=1 perde para ele na Baixa e no entorno (o ganho a mais na Alta, −0,07 ms
a 2160p, não paga a perda). **Os níveis 0–8 do §43 d eram a fronteira de passe do modo de fases**: pela `c2`, os
níveis 2–12 sozinhos perdem (2160p Alta +0,02…+0,83, `_1`, nos tetos sem o salto de regime).

**O salto de regime era da RÉGUA, não do produto** (`target/bug6/regime_regua.rs`, `regime_1.log`): um solver do
produto, 4 fases de 60 quadros com e sem reescrever os uniformes a cada quadro — estável em todas (2160p Alta 62,9 no
1.º solver e 51,5 no 2.º: o regime do NASCIMENTO do §45; 2160p Baixa 5,73; 1440p Alta 20,2). Só a troca de máscara a
cada 8 quadros o provocou. ⇒ régua A × B por uniforme: troca a cada quadro, a mediana das diferenças pareadas, com o
controlo A × A.

**Gate:** `o_nivel_0_por_um_cone_nao_muda_um_bit` (`a_luz_de_fora_nao_atravessa.rs`): o produto × todos pelos cones ×
todos por um cone, a vista com o entorno, 0 texels em 5 casos (a Baixa — quadrantes juntos — nas duas câmeras deitadas
e na janela em pé, a Média, a Alta), 8 s. Prova de mutação (`target/bug6/mutacao_n0.sh`, `mutacao_m2{1,2,3,4}_*.log`,
controlo `mutacao_n0_controlo.log`):

| Mutação | Morre |
|---|---|
| M21 1 ulp no `soma_da_descida` só no nível 0 | 397 513 texels (× todos pelos cones) |
| M22 o vizinho ímpar errado no `soma_da_descida` | 1 682 360 texels |
| M23 o T reaproveitado errado no `descer_cones` | 1 690 432 texels (× todos por um cone) |
| M24 1 ulp no `descer_cones` só no nível 0 | 397 513 texels |
| controlo | 0 nas 10 comparações |

(As duas câmeras deitadas dão o mesmo «1 690 432 acesos» mas imagens diferentes — somas 9,84 M × 33,8 M: só a
contagem coincide.) Suíte não corrida (1 commit desde a última verde); os dois gates de bit verdes por nome.

⚠️ **Formatação:** `solver_hrc.rs` (4 trechos) e `hrc.rs` (1) não passam o `rustfmt` desde o §44 — formatar põe o
`solver_hrc.rs` acima das 700 linhas (está em 699). Cura no fecho: mover mais um ajudante para `solver_hrc_ligar.rs`.

## ⛔ Recusas MEDIDAS (§47)

| Recusa | Porquê |
|---|---|
| Um cone por fio nos níveis 1–8 (o §43 d) | Pela `c2` perdem; o ganho era a fronteira de passe do modo de fases. |
| Os níveis 0..=1 | Perdem para o 0 sozinho na Baixa e no entorno. |
| Reescrever os uniformes a cada quadro para «segurar» o regime rápido | Não muda o regime (`regime_1.log`). |

## 48. ABERTO (por ordem — substitui o §46)

1. Smoke do dono: a luz da Baixa igual com e sem (`PH2D_LIGHT2D_JUNTOS=0`).
2. A vista nos níveis de cima (+4,4 ms a 2160p Alta) continua sem cura que dê o mesmo bit.
3. O regime do NASCIMENTO a 2160p Alta (62,9 × 51,5 ms, o mesmo solver nascido em 1.º ou 2.º): qual o produto apanha
   no app, e se o «nasce duas vezes» (W5p) cobre a vista de hoje.
4. A lei W6d com `DISTANTES` no relógio (o §20.2). 5. A anomalia da r5b (§12). 6. A família (b) (37–38). 7. Os
   zera/limpa por um carimbo de quadro — só se o C3 voltar à linha dos +5 %. 8. O `rustfmt` do `solver_hrc.rs` (§47).

## 49. O regime do NASCIMENTO (§48.3) — as páginas novas do driver; a vista renasce sozinha uma vez (21/10)

`4e98acba6b`. Régua `target/bug6/nascimento_regua.rs` pelo `LuzDoQuadro` (o caminho do app), a `c2` da vista; lotes
`nascimento_lote.sh` (um processo por caso) e `nascimento_lote5.sh` (ON × OFF, a `c2` pela série do app:
`PH2D_LIGHT2D_CUSTO=1 PH2D_LIGHT2D_SERIE=1`).

**O efeito** (`nascimento_1.log`, `_2.log`; a Alta, cena 5, mediana nasce → renasce → renasce → luz nova):

| teto (vista) | bytes da `PassList` | nasce → renasce (ms) |
|---|---|---|
| 3440×1440 (4416×1920) | 2,27 GiB | 31,63 → 31,60 |
| 3200×1800 (4096×2368) | 2,86 GiB | 35,56 → 35,09 |
| 3600×2000 (4608×2624) | 3,24 GiB | **50,23 → 47,08** |
| 3840×2160 (4992×2944) | 3,82 GiB | **60,5–64,4 → 53,0–55,4**, em 5 processos |
| 5120×2880 (6528×3840) | 6,21 GiB | 91,76 → 91,97 |

Não é «o 1.º do processo»: vinda da Média (1,08 GiB) ou da Baixa, a Alta de 2160p nasce lenta na mesma (63,6 · 64,4 →
55). É a memória que o processo ainda NÃO usou — o regime da W5x (páginas novas lentas; as largadas, rápidas). A
cura antiga (`hrc_nasce_duas_vezes`, o berço) só refaz os buffers da holográfica e, com os níveis em textura (W5z),
está desligada: as texturas vêm da `PassList` do solver. A W5z recusou «nascer 2× com textura» como neutro a 1080p e
1440p — onde o efeito não existe. A régua do §47 (os 52 × 60 ms do §45) era este mesmo efeito.

**A cura** (`quadro_renasce.rs`, o vigia da W6b): a vista que nasce com mais bytes do que todas as deste vigia e acima
de `PAGINAS_NOVAS_DESDE = 3 GiB` pede o renascimento ao fim de `LEITURAS_NA_GRADE = 10` leituras na mesma grade (o
arrasto da janela recomeça a conta; encolher antes cancela); o renascimento é o da W6b/W6c (a velha larga-se, a placa
espera, a nova monta no início do quadro seguinte — o mesmo bit, nenhum quadro sem luz, os gates deles verdes). Custa
uma montagem a mais (~0,15 s) uma vez por pico. `PH2D_LIGHT2D_RENASCE_PAGINAS_NOVAS=0` compara.

**ON × OFF** (`nascimento_8.log`, leituras 2–9 × 12+, duas rodadas): 2160p Alta OFF 63,4–64,6 estável; ON 63,4 →
**55,1** (−8,3 ms, −13 %). 3600×2000 Alta OFF 50,4–52,2 → 50,8–51,3; ON 50,4 → **47,0**.

**Gate:** `a_vista_em_paginas_novas_renasce_sozinha_uma_vez` (2160p Alta: 2 montagens em 60 quadros; 1080p Baixa:
1) + 3 testes de unidade do vigia (uma vez por pico; abaixo do limiar ou desligado nada; o arrasto e o encolher).
Mutações (`target/bug6/mutacao_paginas.sh`, `mutacao_m3{1..5}_*.log`, controlo `mutacao_paginas_controlo.log`):

| Mutação | Morre |
|---|---|
| M31 limiar `u64::MAX` | o gate: 1 montagem |
| M32 sem o pico | 2 testes de unidade + o gate: 3 montagens |
| M33 nascer não cancela | o arrasto: renasce na grade que já cabia |
| M34 `nasceu(0)` no `quadro.rs` | o gate: 1 montagem |
| M35 limiar 0 | o gate: a 1080p Baixa renasce |
| controlo | 9 + 1 verdes |

⚠️ **Os testes que leem o custo do app têm de andar ao ritmo do ecrã:** a leitura SEM ESPERA (`ler_sem_esperar`) pede
o mapa depois do envio; com os quadros colados o envio seguinte cancela-o e o vigia (e o governador) nunca leem. O
gate espera a placa a cada quadro (`wait_indefinitely`), como a régua da W5e. E só há leituras com a preferência do
editor (`preferir`) — sem ela o vigia da W6b também não corre.

Por nome, verdes: os 3 de `a_vista_renasce_com_os_mesmos_bits`, a estufa, os 9 de `os_tetos_da_placa`, os 2 de bit.

## ⛔ Recusas MEDIDAS (§49)

| Recusa | Porquê |
|---|---|
| Renascer toda vista grande, sem o pico | Renasce a cada 10 leituras para sempre (M32: 3 montagens em 60 quadros). |
| Nascer 2× dentro do `Solver::novo` em todo nascimento | O arrasto da janela pagaria 2 montagens por quadro; o vigia espera a grade assentar. |
| Limiar abaixo de ~3 GiB | 2,27–2,86 GiB não ganham (31,63 → 31,60 · 35,56 → 35,09) e pagariam ~0,1 s parado. |

## 50. ABERTO (por ordem — substitui o §48)

1. Smoke do dono: a luz da Baixa igual com e sem (`PH2D_LIGHT2D_JUNTOS=0`); e, num ecrã 4K na Alta, o aviso
   «nasceu em páginas novas» uma vez e o custo do Inspector ~8 ms mais baixo um segundo depois.
2. A vista nos níveis de cima (+4,4 ms a 2160p Alta) continua sem cura que dê o mesmo bit — remedir com a vista
   renascida (o número do §42 pode ter sido do regime lento).
3. O entorno também nasce em páginas novas? (não medido; a 2160p Alta é 1984×1152.)
4. A lei W6d com `DISTANTES` no relógio (o §20.2). 5. A anomalia da r5b (§12). 6. A família (b) (37–38). 7. Os
   zera/limpa por um carimbo de quadro — só se o C3 voltar à linha dos +5 %. 8. O `rustfmt` do `solver_hrc.rs` (§47).

## 51. Os níveis de cima e o entorno no regime do nascimento (§50.2–§50.3) — medidos, nada no produto (21/10)

Régua `target/bug6/regime_topo_regua.rs`, lotes `regime_topo_lote{,2,3,4}.sh`, logs `regime_topo_{2,3,4}.log` (o
`_1`, sem o cadeado da placa, descartado). 2160p Alta, cena 5, céu (`somar_entorno(None)`) × borda (o entorno com o
arquivo); a conta-sem-ler do §41 precisava do `DIAG_FORA` no produto e ficou de fora.

**(a) Os +4,4 ms dos níveis 11–12 SOBREVIVEM no regime rápido** (fases da descida, mínimo, céu → borda):

| | nasce (lento), 3 processos | renasce (rápido), 2 processos |
|---|---|---|
| n = 11 | 2,01–2,03 → 4,26–4,27 | 2,00 → 4,26–4,28 |
| n = 12 | 0,95–0,99 → 3,05–3,06 | 0,97 → 3,06–3,08 |
| n 11 + 12 | +4,31…+4,49 | **+4,35 / +4,38** |
| `c2` da vista (mediana) | céu 52,4–53,5 · borda 59,4–60,5 | céu 45,2–47,3 · borda 51,4–53,5 |

O regime do nascimento soma ~7 ms por igual a céu e borda e não aparece nas fases; a borda continua +6,0…+6,5 ms de
`c2`. ⇒ o §42.2 / §50.2 segue aberto com o número certo.

**(b) O entorno não precisa da regra das páginas novas** (total, 40 quadros, um processo por teto, 2×): 2160p Alta
(1984×1152, 0,65 GiB) 10,00–10,13 → 9,85–9,90 ms; 3600×2000 (0,60 GiB) 9,23 → 9,08 — −0,1…−0,25 ms, longe do limiar.

**(c) ⚠️ Um 3.º regime, só da RÉGUA:** a vista renascida depois de uma vida com `medir_fases_da_hrc` cai muitas vezes a
`c2` 61,9–69,6 ms (pior do que nascer): 8 de 8 processos com fases em todas as vidas, 1 de 4 só no nascimento, 0 de 4
sem fases antes. Causa não medida (o produto não liga as fases). ⇒ nunca medir uma renascida cuja vida anterior ligou
as fases.

## 52. ABERTO (por ordem — substitui o §50)

1. Smoke do dono: a luz da Baixa igual com e sem (`PH2D_LIGHT2D_JUNTOS=0`). (As páginas novas só se veem num ecrã 4K
   na Alta — o do dono é 2560×1080.)
2. A vista nos níveis de cima: a borda custa +4,35 ms nos níveis 11–12 também no regime rápido — sem cura de mesmo bit
   (a tabela do R de fora recusada, §43 c).
3. A lei W6d com `DISTANTES` no relógio (o §20.2). 4. A anomalia da r5b (§12). 5. A família (b) (37–38). 6. Os
   zera/limpa por um carimbo de quadro — só se o C3 voltar à linha dos +5 %. 7. O `rustfmt` do `solver_hrc.rs` (§47).

## 53. A vista nos níveis de cima (§52.2) — o salto do T nulo, o mesmo bit (22/10, smoke do §52.1 aprovado pelo dono)

Instrumento fora do git e reposto: `target/bug6/diag_borda_52.patch` (`DIAG_FORA_W` = o `fora.w` por quadro: 0 céu ·
|2 sem ler o arquivo · |4 salta T = 0; `DIAG_VAR` = um de 6 jogos de pipelines no MESMO solver, `override
DIAG_VAR_HRC`); réguas `target/bug6/borda_topo52_regua.rs` (3 testes), `produto52_regua.rs` + `produto52_lote{,2}.sh`;
logs `borda_topo52_{1,2,3}.log`, `borda_nove_52_1.log`, `borda_bit_52_1.log`, `produto52_{1,2}.log`,
`mutacao_t_nulo.log`. Cena 5, h = 3,8, a vista renascida, as variantes A CADA QUADRO, a mediana pareada, A × A.

**(a) O que custa no topo é a FORMA do código, não a leitura** (2160p Alta, `fora.w` por quadro): sem ler o arquivo
−1,04 ms de `c2`, n = 11 / 12 só −0,30 / −0,25; o instrumento com dois `if` de uniforme a mais pôs +0,45 ms em
n = 11 SÓ por existir (4,71 contra 4,26). ⇒ as variantes medem-se como CONSTANTES de pipeline, nunca como uniforme.

**(b) As variantes da `metade_da_descida`** (C1 = T = 0 não lê o R de cima; C2a = o termo de longe num sítio só —
com xi ímpar `(xi + 1)/2 = xi/2 + 1`, só o `q` muda; C2b = um sítio para f0 e f1 num laço):

| 2160p Alta, a rodada limpa (A × A +0,011) | `c2` − A | n = 11 | n = 12 | o bit (18 vistas: 9 tetos × 2 câmeras) |
|---|---|---|---|---|
| A | — | 4,310 | 3,075 | A × A e a pipeline gémea: 0 |
| C1: `return 0` no lugar de `T·R` | −0,11 | 3,824 | 2,766 | **8 e 10 texels** (1440p Média, 2160p Baixa; rel. 8,8e-4 = 1 ULP do f16) |
| C2a sozinho | (−1,0, outra rodada) | | | **18 texels** a 1080p Alta |
| C1′: só a FONTE do R muda (`R = 0` se T = 0), `a·T.r + T.t·R` igual | −0,32 | 3,978 | 2,804 | 0 |
| **C1′ + C2a** | **−0,75** | **3,881** | **2,729** | **0** |

O `if` que devolve 0 no lugar do produto parte a FMA que o driver faz de `a·T.r + T.t·R` — muda 1 ULP também onde
T ≠ 0. Manter a árvore da conta e trocar só a fonte do R guarda o bit. As rodadas 2–3 (−4,4 / −8,7 ms) eram do REGIME
(o A delas a 63,9 / 61,6 contra 56,5): só a rodada com o A × A apertado (±0,12) conta.

**(c) No produto** (`Config::hrc_salta_o_t_nulo`, `override T_NULO_HRC`, `PH2D_LIGHT2D_T_NULO=0` compara; um processo
por caso, ABAB, `c2` mediana):

| teto | vista sem → com | entorno sem → com |
|---|---|---|
| 1080p Baixa | 1,48 → 1,42 | 1,035 → 1,020 |
| 1080p Média | 2,84 → 2,81 | 1,398 → 1,406 |
| 1080p Alta | 11,70 → 11,54 | 2,942 → 2,956 |
| 1440p Baixa | 2,30 → 2,18 | 1,510 → 1,470 |
| 1440p Média | 4,71 → 4,61 | 1,803 → 1,816 |
| 1440p Alta | 20,22 → 19,86 | 4,820 → 4,846 |
| 2160p Baixa | 5,81 → 5,74 | 2,804 → 2,686 |
| 2160p Média | 11,71 → 11,55 | 2,942 → 2,956 |
| 2160p Alta | 51,7 → 50,9 (n 11: 4,258 → 3,872 · n 12: 3,065 → 2,730) | 9,92 → 9,94 |

A borda nos níveis 11–12 a 2160p Alta: +4,35 → +3,62 ms (−17 %). Vista + entorno ganham em todo teto.

**Gate:** `a_descida_sem_o_t_nulo_nao_muda_um_bit` (`a_luz_de_fora_nao_atravessa.rs`): 1080p Baixa e Alta, 1440p
Média, 2160p Baixa (as vistas que apanharam C1 e C2a), com × sem: 0 texels nas 4. Prova de mutação
(`target/bug6/mutacao_t_nulo.sh`):

| Mutação | Morre |
|---|---|
| M1 o `if` à volta da SOMA (a forma do C1) | 8 texels a 1440p Média |
| M2 salta também T ≠ 0 (`t > 0,25`) | 1 445 493 |
| M3 o `q` de par e ímpar trocados | 1 690 432 |
| M4 a coluna `(xi + 1)/2` no termo unificado | 1 690 431 |

Também: as constantes de pipeline da holográfica passam a `solver_hrc_ligar::constantes`; o `solver_hrc.rs` formatado
pelo `rustfmt` fica com 687 linhas (o §52.7 fechado). O nível 0 (`soma_da_descida`, um cone por fio) não leva o salto:
não medido.

## ⛔ Recusas MEDIDAS (§53)

| Recusa | Porquê |
|---|---|
| Devolver 0 no lugar de `T·R` (o C1) | Parte a FMA do driver: 8–10 texels (1 ULP do f16) em 2 de 18 vistas. |
| C2a sem o salto, C2b (um sítio num laço), C1 + C2b | 18 texels (C2a); +1,9 ms (C1 + C2b); C2b sozinho −0,13 com q3 +3,4. |
| Medir variantes de forma de código por um `if` de uniforme | Os `if` a mais custam por si (+0,45 ms em n = 11). |
| Desligar o salto no entorno fora da Baixa | +0,006…+0,026 ms por quadro (≤ 0,5 % do entorno) não paga uma regra por grade. |

## 54. ABERTO (por ordem — substitui o §52)

1. A vista nos níveis de cima: +3,62 ms de borda em n = 11–12 a 2160p Alta — sem outra cura de mesmo bit à vista (a
   tabela do R de fora recusada, §43 c; a leitura do arquivo só pesa ~0,55 ms lá, §53 a).
2. A lei W6d com `DISTANTES` no relógio (o §20.2). 3. A anomalia da r5b (§12). 4. A família (b) (37–38). 5. Os
   zera/limpa por um carimbo de quadro — só se o C3 voltar à linha dos +5 %.
6. A suíte no fecho da onda (não corre desde o §45).

## 55. A lei W6d com `DISTANTES` no relógio (§20.2) — os distantes não pesam; o ENQUADRAMENTO sim (22/10, medido, nada no produto)

Régua fora do git `target/w6g/distantes_custo_regua.rs` (3 testes; registe o mod em `tests/it/main.rs` só para correr),
logs `target/w6g/distantes_custo_{1,2,3}.log`. O molde do `custo` da `medida_luz2d_w6d` (FHD, `DEGRAU_PADRAO`, sem
rebote): lei W6d (`borda_vertice: None` + `sem_os_cantos`) × o padrão de hoje (a 10 + 9 com os cantos, §28), 9
rodadas intercaladas no mesmo processo, o MÍNIMO da C3 (preenche + visibilidade + fina); a lei W6d de novo como
controlo (±0,2…±2,3 %). `load` 17–32 (as outras linhas): só os relativos do mesmo processo contam.

**(a) Os zooms da vista de fábrica não põem distantes** (0 em 1 · 1,6 · 2,5 · 4, cenas 5 e 6): as lâmpadas ficam
dentro da grade. A varredura de câmeras (centro −6…6 × −3…3, altura 1,5 e 2,5) acha distantes em 36 de 56 (cena 5,
até 3) e 14 de 56 (cena 6, até 2).

**(b) Com e sem distantes, as parcelas** (ms; % do padrão sobre a lei W6d com a MESMA escolha de distantes):

| cena · câmera | dist | W6d C3 com / sem dist | padrão com / sem | padrão − W6d: borda · fina (sem dist) |
|---|---|---|---|---|
| 5 · [−6, −1] h 2,5 | 3 | 2,351 / 2,198 | 2,487 (+5,8 %) / 2,321 (+5,6 %) | +0,040 · +0,086 |
| 5 · [6, −1] h 2,5 | 3 | 2,614 / 2,425 | 2,911 (+11,3 %) / 2,746 (**+13,2 %**) | +0,065 · +0,251 |
| 5 · zoom 1,6 | 0 | 3,630 / 3,625 | 3,825 (+5,4 %) / 3,827 (+5,6 %) | +0,052 · +0,177 |
| 5 · zoom 1 (a régua do alvo) | 0 | 3,047 / 3,031 | 3,171 (+4,1 %) / 3,177 (+4,8 %) | +0,049 · +0,088 |
| 6 · [0, −3] h 1,5 | 2 | 1,931 / 1,738 | 2,067 (+7,0 %) / 1,895 (+9,0 %) | +0,037 · +0,126 |
| 6 · zoom 4 | 0 | 1,534 / 1,535 | 1,671 (+8,9 %) / 1,655 (+7,8 %) | +0,042 · +0,084 |
| 6 · zoom 1 (a régua do alvo) | 0 | 2,090 / 2,090 | 2,141 (+2,4 %) / 2,148 (+2,8 %) | +0,041 · +0,010 |

⇒ **Os distantes custam o mesmo às duas leis** (+0,15…+0,19 ms, na fina e na visibilidade): a lei W6d com eles no
relógio não muda o relativo. **O alvo «C3 ≤ +5 % sobre a lei W6d» só foi medido no zoom 1** (+2,4…+4,8 %); noutras
câmeras das MESMAS cenas o padrão paga +5,4…+13,2 % — a borda dos cantos (+0,04…+0,065 ms, quase fixa) e a fina dos
cantos (+0,01…+0,25 ms, cresce com os cantos à vista). Em absoluto: +0,05…+0,32 ms por quadro a FHD Baixa.

## 56. ABERTO (por ordem — substitui o §54)

1. **DECIDIDO (22/10, o dono delegou: «de modo que o app seja o mais capaz possível»):** os cantos vivos FICAM no
   padrão (a imagem não se troca) e o alvo «C3 ≤ +5 % sobre a lei W6d» vale para TODA câmera das cenas 5 e 6 — as
   7 câmeras do §55 b são a régua. A próxima rodada: o custo dos cantos na fina (+0,01…+0,25 ms) e na borda
   (+0,04…+0,065, quase fixa) — o kill-criterion do §1, a régua de registos do §20.1; os zera/limpa por carimbo
   (§29.5, ~0,008 ms) não chegam para ela.
2. A vista nos níveis de cima: +3,62 ms de borda em n = 11–12 a 2160p Alta (§54.1).
3. A anomalia da r5b (§12). 4. A família (b) (37–38).

**Suíte do fecho da onda** (`nextest-impacted`, base = merge-base, `target/bug6/suite_52.log`): 17 799 / 17 801 verdes. As 2
vermelhas são razões de relógio do `ph2d-tool-painter` (que esta linha não toca), com `load` 108–215:
`the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` (verde ao repetir) e
`a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs` (vermelha 1 de 3 repetições, verde 2) — candidatas
à família dos flakes de carga (`docs/DevOps/FLAKES_DE_CARGA.md`), por confirmar com a máquina calma pela linha do Painter.
