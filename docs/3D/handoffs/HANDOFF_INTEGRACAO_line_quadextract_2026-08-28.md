# HANDOFF DE INTEGRAÇÃO — `line/quadextract` (2026-08-28): **a régua estava errada, e o que faltava era o ACABAMENTO**

> **Leia primeiro:** [`ACHADO_o_acabamento_e_a_regua_da_densidade.md`](../quad-remesh/ACHADO_o_acabamento_e_a_regua_da_densidade.md)
> — é o documento de conteúdo, com as tabelas e as recusas medidas. Este traz o que o
> **integrador** precisa.

## §1 — O que esta jornada descobriu, em três frases

1. ⛔⛔⛔ **A barra do oráculo (`4,8°`–`7,1°` de enviesamento) estava a ser lida a 1/9 da
   densidade dele.** A nossa medição corria com `370`–`576` quads e a saída dele tem
   `3 352`–`4 696`. À densidade dele, a mesma cadeia **sem uma linha mudada** dá
   `3,8°`–`6,5°` — dentro da barra desde 2026-08-25. ⇒ *a semana das amarras dos arcos
   perseguiu um buraco da régua.*
2. ⭐⭐ **O que sobra é o passe de ACABAMENTO dele.** O oráculo grava duas saídas por peça
   (crua e `_smooth`); a nossa saída crua **bate a crua dele** em três peças, e o `_smooth`
   compra-lhe `−0,3°` a `−1,5°` de mediana e `−8°` a `−11°` de `p99`. O nosso acabamento
   eram `6` rondas de Laplaciano herdadas da montagem por patches, **nunca re-medidas** para
   a extracção.
3. ⭐⭐⭐ **A cadeia passa a ter um acabamento próprio, numa porta só**
   (`ph2d_quadfill::finish_extracted`): Laplaciano como **ronda zero**, depois **ajuste de
   quadrado alinhado ao relevo**, e a saída é a **melhor ronda**, não a última.

## §2 — O que mudou no produto

| onde | o quê |
|---|---|
| `crates/ph2d-quadfill/src/finish_extract.rs` | **NOVO** — a porta, as quatro constantes medidas e a comparação de Pareto |
| `crates/ph2d-quadfill/src/relax.rs` | `square_relax{,_capped,_aligned}` públicos · `steer` (o alinhamento) · cerca de viagem · raio de reprojecção que encolhe · saída por assentamento |
| `crates/ph2d-quadfill/src/quality.rs` | `Hint` + `surface_hint` — a direcção que a superfície prefere, por face da saída |
| `crates/ph2d-quadchain/src/lib.rs` | passa a **acabar** (entregava a malha crua) · `ChainTiming::finish` · `ChainReport::finish` |
| ⚠️ `shells/desktop/src/sculpt3d_history_retopo_extract.rs` | o botão chama a porta em vez do Laplaciano cru |
| `crates/ph2d-quadextract/examples/{chain_info,piece_report}.rs` | os instrumentos: `PH2D_RELAX_SCAN=1` varre **através da porta**; `PH2D_REF=<peça>.obj` mede relevo e fidelidade contra a escultura |

⚠️ **Tudo aditivo.** Nenhuma assinatura pública existente mudou de forma; `ChainTiming` e
`ChainReport` ganharam campos (os dois são `#[derive(Default)]`/construídos por nome aqui).

⛔ **O caminho do `ph2d_quadfill::fill` (a montagem por patches) fica INTACTO** — a tabela de
rejeição dele (`SQUARE_ROUNDS = 0`) foi medida noutra conectividade e continua a valer lá.

## §3 — As constantes, e de onde saiu cada número

| constante | valor | de onde |
|---|---|---|
| `EXTRACT_RELIEF_PULL` | **`1,0`** | ⭐ *o peso É a confiança* — a anisotropia crua, sem constante por cima. Numa esfera ela é `0` e a lei degenera **ao bit** no quadrado puro |
| `EXTRACT_SETTLE` | **`1e-3`** da aresta mediana | tabela medida **através da porta**; `3e-4` custa `1,5`–`3×` mais para `0,2`–`0,3°` |
| `EXTRACT_PATIENCE` | **`768`** rondas **sem aceitar nada** | `1,8×` a maior primeira aceitação medida (`418`) |
| `EXTRACT_MAX_ROUNDS` | `1 200` | a rede |
| `quality::HINT_SMOOTH_ROUNDS` | **`0`** | ⛔ construída, medida e **não adoptada** — ver §4.4 |

## §4 — ⚠️ As SETE coisas que uma leitura rápida do diff entende ao contrário

1. **A relaxação por ajuste de quadrado NÃO é nova** — existe desde 2026-08-22, com
   `SQUARE_ROUNDS = 0` **medida e rejeitada**. O que mudou foi a **conectividade** a que ela
   se aplica: a tabela da rejeição mediu a montagem por patches (`27°` de mediana, defeito na
   conectividade); a extracção entrega `1,10 / 3,8°`. *Uma recusa medida responde uma
   pergunta.*
2. **O Laplaciano NÃO saiu** — ele é a ronda zero, e é ele que mata a face extrema (`>60°` de
   `7` para `1` na `sculpt_hooked` fina). As duas leis atacam metades diferentes.
3. **A cerca de VIAGEM existe na API e nasce DESLIGADA** (`square_relax_capped`) — medida e
   rejeitada como cura: a `0,35 h` guarda o relevo e paga o `p99` (`52,8°` contra `34,5°`).
4. **A aceitação é contra a RONDA ZERO, e cobre CINCO colunas** (`>60`, enviesamento `p50` e
   `p99`, aspecto `p50` e `p99`). ⛔ A 1.ª redacção comparava com a **melhor até então** e era
   uma **catraca**: a relaxação mergulha antes de subir, e as quatro peças da densidade fina
   saíam intocadas. A escolha **entre** aceitáveis é a mediana, com o aspecto a desempatar.
5. ⛔ **A paciência conta rondas SEM ACEITAR NADA, não «desde a melhor».** Na
   `sculpt_hooked` fina a primeira aceitação é a `312` e a melhor é a `830`: com `128` rondas
   *desde a melhor* a peça saía **intocada** em vez de ir a `1,04 / 2,0° / p99 22,8`.
6. ⭐ **Há uma queda para a lei CEGA**, e ela só corre quando a alinhada **não se mexeu** — é
   isso que guarda o relevo onde ele estava em jogo (`5` das `8` células ficam com a
   alinhada). ⚠️ Nas outras três o preço é o relevo, e está medido.
7. **O raio de reprojecção encolher não é aproximação** — depois da 1.ª ronda o vértice está
   *sobre* a superfície, e uma esfera de `2×` o que ele andou contém o pé mais próximo. Vale
   `~12×` de relógio.

## §5 — ⭐⭐⭐ O RESULTADO, e o que fica ABERTO

À densidade do oráculo (`alvo 0,667`), contra a saída **`_smooth`** dele:

| peça | nós ANTES | ⭐ **nós DEPOIS** | oráculo `_smooth` |
|---|---|---|---|
| `sphere_uv` | `1,10 / 3,8° / 17,3°` | **`1,04 / 2,6° / 10,1°`** | `1,22 / 5,9° / 20,0°` |
| `sculpt_eared` | `1,10 / 6,3° / 27,2°` | **`1,04 / 3,3° / 11,0°`** | `1,08 / 5,7° / 20,2°` |
| `sculpt_hooked` | `1,11 / 6,5° / 33,0°` (`>60` 1) | **`1,04 / 2,0° / 22,8°`** (`>60` **0**) | `1,19 / 5,8° / 48,1°` (`>60` 4) |
| `sculpt_wrinkled` | `1,12 / 5,2° / 35,5°` | **`1,07 / 2,8° / 22,8°`** | `1,08 / 4,8° / **17,0°**` |

⇒ **batemos a saída alisada dele em TODAS as colunas de forma em três das quatro peças**, e
em duas de três na quarta (ele fica com a cauda da enrugada).

### ⛔ Aberto, com o número ao lado

- **O RELEVO** é a coluna em que ficamos atrás: `11,6°` contra `7,0°` na enrugada e `19,3°`
  contra `13,3°` no gancho (empatados na orelha). ⚠️ *Já estávamos atrás antes desta
  jornada* (`11,8°` e `17,7°`) — a queda para a lei cega paga mais um pouco em três células.
- ⛔ **A hipótese do «campo de direções ruidoso» está REFUTADA** como cura da recusa da lei
  alinhada: a suavização 4-RoSy foi construída, medida e não adoptada (§4.4 do ACHADO §10.4).
- **Preço:** `0,2`–`0,6 s` na densidade do botão, `3`–`12 s` na fina, e o botão corre a
  cadeia **duas** vezes. `PH2D_EXTRACT_FINISH=0` desliga.

## §6 — O que o Enio smoka

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract && env PH2D_SCULPT3D_SMOKE=35 cargo run -p ph2d-host-desktop --release
```

Depois: **`Quad Retopology`** no painel de escultura. `PH2D_EXTRACT_FINISH=0` volta ao
acabamento antigo (o Laplaciano cru), para comparar lado a lado.

## §7 — Gates novos (todos provados por mutação)

`crates/ph2d-quadfill/src/finish_extract_tests.rs` (7) e `relax_tests.rs` (+6).
**14 mutações, 14 mortas** — entre elas duas que a 1.ª redacção dos gates deixava viver:
*a ordem ignora o aspecto* e *a paciência conta do início*. ⚠️ E um gate desta jornada era
uma **tautologia** apanhada por mutação: ele media a rotação com uma função que devolve
`[0°, 45°]` **por construção**, logo não podia falhar.
