# Handoff de continuação — Luz 2D, W6e: o texel do canto convexo (2026-10-18)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `ad5f8fe39c`, por cima da [W6d](HANDOFF_CONTINUACAO_W6d_FECHO_2026-10-18_A_BORDA_E_O_CANTO.md); sem rebase (é do
> integrador). A suíte do fecho da W6d (`target/w6d/suite.log`): 13 969/13 969 verdes — sem item 0.
> SUÍTE: recusada pela cadência no fecho da onda (4 commits desde a verde da W6d; o passo é 10) · BUILD: o smoke compilado
> depois do último commit (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida
> `Finished \`smoke\` profile [optimized] target(s) in 0.23s`).
>
> **Em uma linha:** a cunha no vértice acerta o campo e o texel do aberto (0,016 → 0,601) a +1 % de custo, mas reprova o F3
> — a luz do canto é decidida pela mistura das faces no `c3_direta`; a cunha fica DESLIGADA e a onda para (two-strikes).
> Entregue: o portão do campo da borda, que dá ao app o observador da M3.

## 0. O ponto de partida (a lei da W6d, `f0a98caf19`; números do handoff W6d §23)

| | (a) | (b) | B (caixa) | F3 penumbra | vértice girado (gab. 0,207) | canto da caixa | C3 cena 5 / 6 |
|---|---|---|---|---|---|---|---|
| antes da W6d | 139 (−0,964) | 40 | verde | 0,0563 | 0,203 | **0,575** | 3,384 / 2,191 |
| a lei atual (W6d) | 1 (−0,176) | 37 | verde | 0,0531 | 0,209 | **0,012** | 3,047 / 2,094 |

O canto da caixa = o texel fino (1,8516; 0,9141) da sonda `medida_luz2d_w6d_canto` (zoom 1, fase 3/8, lei antiga
do miolo), a 0,22 texel fino das duas faces — fora do gabarito do portão B (que só mede a mais de 1 texel da borda).

## 1. A candidata 8 — a CUNHA no parcial do vértice — e o seu kill-criterion (escrito ANTES de medir)

Mecanismo (W6d §14/§23): o parcial do vértice guarda a SUA reta (o Sobel de duas faces sai diagonal) e o iso-0 do campo
corta o canto ~0,45 texel da captura. Cura: no `c3_borda`, o parcial cuja reta NÃO explica a própria 3×3 (a cobertura
que a reta dele prevê para os 8 vizinhos difere da lida em mais de `pre`) procura as retas dos parciais a DOIS texels
(o anel de Chebyshev 2, cujas 3×3 já não contêm o vértice; as adjacentes saem inclinadas ~16°), toma o par de normais
mais afastadas, decide convexo/côncavo pelo teste `d₂(c_q₁) + d₁(c_q₂) > 0` e, se a CUNHA dessas duas retas deixa
dentro do texel a área igual à cobertura dele (a menos de `area`), grava a distância da cunha (convexo: o mínimo das
duas; côncavo: o máximo) no lugar da da própria reta. O disco não passa: a cunha de duas tangentes deixa dentro mais
área que o arco. `Config::borda_vertice: Option<Vertice { pre, area }>` (overrides `BORDA_VERTICE_PRE/AREA`).

**Régua** (um processo, intercalada): o censo + anéis + bits + custo da `medida_luz2d_w6d_borda` com as variantes
antes da W6d · a lei atual · **cunha**; a sonda do canto (`medida_luz2d_w6d_canto`) com as três; o portão B, o F3
(penumbra e o texel do vértice girado) e o portão NOVO do canto com a cunha ligada no padrão.

**PASSA** (todos):
1. o texel do canto da caixa em **0,575 ± 0,1** (o «antes da W6d»);
2. o portão B verde; o F3 verde (penumbra ≤ 0,062) e o texel do vértice girado a ≤ 0,01 do gabarito 0,207;
3. (a) ≤ 14 com o pior ≥ −0,25 · (b) ≤ 42 · anéis sem regressão sobre a lei atual em todo zoom;
4. bits de fora dos corpos IGUAIS aos de antes (0 em 24);
5. C3 ≤ **+5 % sobre a lei atual** nas cenas 5 e 6 (o mínimo de 7 intercaladas, no mesmo processo).

**MORRE**: qualquer um dos cinco. Bater na 2.ª reconstrução sem passar ⇒ PARE e prove o modelo (two-strikes).

## 2. Rodadas 1–3 (`target/w6e/rodada1..3.log`; `load` 1–18, o custo o mínimo intercalado no mesmo processo)

| | (a) | (b) | anéis | bits fora | C3 cena 5 / 6 | o texel do canto |
|---|---|---|---|---|---|---|
| antes da W6d | 139 (−0,964) | 40 | 13 | — | 3,392 / 2,202 | 0,576 |
| lei W6d (sem a cunha) | 1 (−0,176) | 37 | 0 | 0/24 | 3,499 / 2,245 | 0,016 |
| **8 — cunha, listas de 49/16 + recorte de polígono** (rodada 1) | 1 (−0,176) | 38 | 0 | 0/24 | 3,759 / 2,506 (**+7,7 % / +11,7 %** ✗) | 0,601 |
| **8b — a mesma lei, código leve** (rodadas 2–3) | **1 (−0,176)** | **37** | **0** | **0/24** | **3,520 / 2,273 (+0,6 % / +1,2 %)** | **0,601** |
| cunha larga (`pre` 0,02, `area` 0,05) | 2 (**−0,824**) | 38 | **5** (polígono, zoom 0,8) | 0/24 | 3,530 / 2,265 | 0,601 |

- **A 8 morreu pelo custo** (item 5) e o custo era o MESMO com o filtro largo ou apertado: não é o trabalho dos texels que
  passam o filtro, é o peso do código (as listas de 49 e 16 e os polígonos de 8 vértices) na passagem inteira — a família da
  união da W6d (§21). A **8b** lê as 3×3 do anel por `textureLoad` só no caminho raro, guarda só as DUAS normais extremas
  (a mais girada para cada lado da normal própria — num canto, uma de cada face) e mede a área da cunha pela integral em 32
  colunas: +0,03 ms na borda.
- `VERTICE_PADRAO = (0,05; 0,02)`: o canto sai igual (0,601) em toda a grelha `pre` 0,02–0,10 × `area` 0,01–0,05
  (`medida_luz2d_w6d_canto`); a «larga» deixa passar o arco do disco por canto (5 anéis errados, um preto −0,824).
- Itens 1, 3, 4 e 5 do §1: **passam**. Item 2 (B, F3, o vértice girado): os portões com a cunha no padrão — §3.

### O canto da caixa inteiro (4 cantos × os 8 casos do B, lei antiga; `RESUMO CANTO`)

antes da W6d **4** errados (pior −0,303) · lei W6d **7** (−0,304) · cunha **6** (+0,147). A cunha devolve ao «antes» três das
cinco regressões da W6d (o aberto, zoom 1 · 3/8 cima-direita; zoom 1 · 6/8 e 2,5 · 4/8 cima-direita). As duas que ficam
(baixo-esquerda em zoom 1 · 6/8 — 0,002 contra 0,235, antes 0,242 — e 2,5 · 0/8 — 0,001 contra 0,317, antes 0,387) e os 4
de antes da W6d **não são o campo**: a régua do campo (`medida_luz2d_w6e_campo`, `PH2D_W6E_LISTA=1`) lê ali o campo a
≤ 0,05 da distância exata. O texel do canto fica com UMA face (o valor de uma ou de outra, nunca a média) — o `c3_direta` no
canto (a média no eixo / a outra face / a normal secante junto ao vértice). ⇒ **ABERTO 1 da W6e**, por medir com a sonda de
cor no `c3_fina` (a da W6d §14). Portão: `TETO_CANTOS = 6` (o medido) e, no caso do aberto, 0 com o controlo sem a cunha a
errar.

### O campo da borda (`o_campo_da_borda.rs`)

A régua lê o 4.º canal do `c3_borda` (`Solver::borda_da_captura`, novo) contra a distância exata ao polígono. No aberto (zoom 1
· 3/8) o parcial do vértice vai de −0,207 (a sua reta diagonal) a 0,000 (exata ≈ 0). Faces alinhadas à grade dão ±1 contra
±0,5 (nenhum parcial: simétrico, o iso-0 certo) — por isso o portão usa a caixa GIRADA (0,5 rad). **A M3 da W6d** (o sinal do
cheio, `d >= 0` → `d < 0`, aplicada à mão e reposta, `campo_m3.log`) não troca sinais (o cheio sem candidato fica +1) mas
estraga a magnitude: texels de face a mais de 0,2 por caso **1–4 → 171–701**. Portão `o_campo_da_borda_e_o_da_geometria`:
sinal 0, `TETO_FACE = 8` (o dobro do medido), controlo = a lei de antes da W6d (o recuo lido como distância).

## 3. Item 2 — a 8b MORRE no F3 (`ac2d6fc26b` → `552dad18e8`; agente `verificador` + `medida_luz2d_w6e_f3`, `target/w6e/f3.log`)

Com a cunha no padrão a bateria deu: gpu `--lib` 6/6 · gpu `it` 41/42 · app `it` 84/84 (os 11 de A a L, o portão novo do
campo e o B com o canto: «cantos errados 6 (teto 6) · o caso do aberto: 0 (sem a cunha 1)»). O vermelho é
`c3_grade_fina::o_interior_de_um_opaco_e_o_exterior_mais_proximo`, a **penumbra 0,0689 > 0,062**. Foi igual em três
corridas, a `load` 6, portanto não é carga.

O mecanismo, pela sonda `medida_luz2d_w6e_f3` (F3 com e sem a cunha, no mesmo processo):
- **A cunha piora UM texel fino:** o (152, 245), junto ao vértice de baixo-esquerda (151,4; 245,1) do quadrado girado,
  passa de 0,140 para **0,173** (gabarito 0,142). Os outros mudam menos de 0,006.
- **O campo da captura ali fica MAIS certo:** muda só um texel junto a cada vértice de baixo (o parcial de cobertura
  0,98), que passa de +0,57 para **+0,48**, com a exata em +0,50.

⇒ O modelo «acertar o campo no vértice acerta a luz» está **refutado**. A luz do texel do canto depende da etapa que
escolhe e mistura as faces no `c3_direta` (a média no eixo / a outra face / a normal secante junto ao vértice). Essa
etapa erra com o campo certo, como os 6 cantos da caixa da §2, e aqui o campo errado compensava-a. Pelo two-strikes (a 8
morreu no custo, a 8b no F3) a onda PARA aqui: a 3.ª tentativa começa por medir essa etapa, não pelo campo.

**Estado:** `Config::borda_vertice = None` no padrão. A cunha (8b) fica no `c3_borda.wgsl` desligada: o override
`BORDA_VERTICE_AREA = 0` elimina o ramo na criação da pipeline. Fica como variante para a próxima onda medir no mesmo
processo. O canto saiu do portão B. Ficam:
- o portão do campo `o_campo_da_borda_e_o_da_geometria` (o item 2 da onda: o observador da M3 no app);
- `Solver::borda_da_captura`;
- as sondas `medida_luz2d_w6d_canto`, `medida_luz2d_w6e_campo` e `medida_luz2d_w6e_f3`.

## ⛔ Recusas MEDIDAS (W6e — não reconstruir sem ler a §2/§3)

| Recusa | Porquê |
|---|---|
| A cunha com listas de 49/16 e recorte de polígono (8) | +7,7 % / +11,7 % na C3. É o peso do código, igual com o filtro largo ou apertado. |
| A cunha leve (8b) no padrão | Acerta o campo e o texel do aberto (0,601), mas o F3 da penumbra sobe a 0,0689 > 0,062: a etapa da mistura das faces no canto erra com o campo certo. |
| A cunha «larga» (`pre` 0,02, `area` 0,05) | O arco do disco passa por canto: 5 anéis errados, um preto −0,824. |
| Gatear o canto da LUZ com teto | Os erros que sobram são de outra etapa (§2): um teto sobre eles seria uma catraca sobre um defeito conhecido. |

## 4. ABERTO (por ordem)

1. **O texel do canto convexo** (o aberto da W6d, zoom 1 · 3/8: 0,016 contra 0,575 de antes) **e a família inteira**: 4
   cantos errados antes da W6d e 7 com a lei W6d, na lei antiga; mais o (152, 245) do F3 quando o campo está certo.
   - **Próximo passo:** a sonda de cor no `c3_fina` (a da W6d §14: o ponto remapeado, a outra face e a fração, a normal
     secante) nesses texels, com a cunha ligada e desligada.
   - **Depois:** curar a etapa da mistura das faces no canto. Só então voltar a medir a cunha como variante (os 5 itens
     do §1, com o F3).
2. **A família (b)** (37, a penumbra dos tangentes): é da W5g, fica fora.

## 5. Prova de fecho (`552dad18e8` + o portão do campo com o sinal «longe»)

- Portões, pelo agente `verificador`, a `load` ≤ 2: F3 penumbra **0,0531** (teto 0,062), B verde, campo verde (sinal 0,
  face 1–4 errados). A bateria com a cunha no padrão (§3) já tinha dado o resto verde: gpu `--lib` 6/6 · gpu `it` 41/42 (o
  vermelho era o F3 da cunha) · app `it` 84/84. O G7 dos outros: disco pequeno −0,21 a −0,30 %, teto 0,33 %.
- **Prova de mutação** do `o_campo_da_borda_e_o_da_geometria` (agente `mutacao`; controlo verde; pré-voo das âncoras = 1):

| mutação (`c3_borda.wgsl`) | resultado | linha decisiva |
|---|---|---|
| M3 o sinal do cheio (`d >= 0.0` → `d < 0.0`) | sangra | face 171–701 errados (teto 8), sinal 0 |
| M1 o ramo do campo (`if any(parcial)` → `if false`) | sangra | sinal [2, 287], face 1 331 |
| M4 o caminho rápido (±1 trocados) | sobrevivia → **sangra** com o sinal até 3 texels (`LONGE`) | sinal 1 775 |
| M5 o sinal da distância do parcial | sangra | sinal [6, 574], face 831 |

  ⇒ a M3 da W6d tem agora um observador no app (o aberto 2 da W6d, fechado).
- Fotos no app: não refeitas. Com a cunha fora do padrão, o produto é o da W6d (`fotos_W6d/`).

## 6. Superfície de colisão e commits

`ac2d6fc26b` (a cunha + os portões) · `552dad18e8` (a cunha sai do padrão; a sonda do F3) · `5453237f7a` (o sinal «longe»;
este handoff) · o `style` do fmt/clippy. Tocam `ph2d-light2d-gpu` (`config.rs` — `Vertice`, `VERTICE_PADRAO`,
`Config::borda_vertice` —, `lib.rs`, `solver_c3_constantes.rs` — `comuns` passa a `[_; 18]` —, `solver_fina.rs` —
`Solver::borda_da_captura` —, `wgsl/c2_rc.wgsl`, `wgsl/c3_borda.wgsl`, `tests/it/c3_grade_fina.rs`) e
`ph2d-app-light2d` (`tests/it/`: `o_campo_da_borda.rs` novo, `o_interior_do_opaco.rs`, `medida_luz2d_w6d.rs`, `main.rs`).
Sem contrato congelado, sem schema, sem shell. Quem constrói `Config` sem `..` parte com o campo novo (nenhum no repo).

## 7. Smoke do Enio

Nada mudou na tela desde a W6d: a cura do canto ficou desligada. O smoke da W6d continua valendo (o handoff dela, §6):
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=6 cargo run -p ph2d-host-desktop --profile smoke`
— a borda acesa do disco lisa em qualquer zoom. O ponto escuro no canto de uma caixa continua lá: é o aberto 1.
