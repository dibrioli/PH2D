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

## §3 — As quatro constantes, e de onde saiu cada número

| constante | valor | de onde |
|---|---|---|
| `EXTRACT_RELIEF_PULL` | **`1,0`** | ⭐ *o peso É a confiança* — a anisotropia crua, sem constante por cima. Numa esfera ela é `0` e a lei degenera **ao bit** no quadrado puro (as linhas `x0`..`x4` da varredura são idênticas) |
| `EXTRACT_SETTLE` | **`1e-3`** da aresta mediana | tabela medida **através da porta**; `3e-4` custa `1,5`–`3×` mais para comprar `0,2`–`0,3°` |
| `EXTRACT_PATIENCE` | **`128`** rondas sem melhoria | ⛔ sem ela, a `sculpt_hooked` fina gastava `1 200` rondas e `8,3 s` para entregar a malha com que começou |
| `EXTRACT_MAX_ROUNDS` | `1 200` | a rede; medido, `1e-3` gasta `248`–`350` rondas |

⚠️ **O `EXTRACT_SETTLE` foi escolhido DUAS vezes.** A 1.ª (`1e-2`) saiu de uma varredura
**sem a ronda zero à frente** e não sobreviveu ao produto: através da porta deu `23` rondas
em vez de `93`, e `7,8° → 6,8°` em vez de `7,8° → 4,5°`. *O Laplaciano pré-condiciona a
malha, o movimento começa menor, e o mesmo limiar relativo chega muito mais cedo.*

## §4 — ⚠️ As cinco coisas que uma leitura rápida do diff entende ao contrário

1. **A relaxação por ajuste de quadrado NÃO é nova** — ela existe desde 2026-08-22 e estava
   `SQUARE_ROUNDS = 0`, **medida e rejeitada**. O que mudou foi a **conectividade** a que ela
   se aplica: a tabela da rejeição mediu a montagem por patches (`27°` de mediana, defeito na
   conectividade), e a extracção entrega `1,10 / 3,8°`. *Uma recusa medida responde uma
   pergunta.*
2. **O Laplaciano NÃO saiu** — ele é a ronda zero. Medido: na `sculpt_hooked` fina ele leva as
   faces péssimas de `7` para `1`, e **nenhuma** quantidade de ajuste de quadrado faz isso.
   As duas leis atacam metades diferentes (comprimento · ângulo).
3. **A cerca de viagem existe na API e nasce DESLIGADA** (`square_relax_capped`). Ela foi
   medida e **não serve** como cura: a `0,35 h` guarda o relevo e paga o `p99` (`52,8°` contra
   `34,5°`), porque prende exactamente os vértices que mais precisavam de andar.
4. **A comparação é de PARETO, não lexicográfica.** A 1.ª redacção era
   `(faces péssimas, mediana)` e **recusava melhorias reais**: numa esfera-UV crua a
   relaxação leva o aspecto de `1,384` a `1,251` e mexe a mediana em `+0,2°`.
5. **O raio de reprojecção encolher não é uma aproximação** — depois da 1.ª ronda o vértice
   está *sobre* a superfície, então uma esfera de `2×` o que ele acabou de andar contém o pé
   mais próximo, e `faces_in_sphere` devolve toda face que a corte. Vale `~12×` de relógio.

## §5 — ⛔ O que fica ABERTO, com o número ao lado

- ⚠️ **Na `sculpt_hooked` FINA o alinhamento nunca bate a ronda zero.** Aquela peça sai do
  Laplaciano com `1` face péssima e a relaxação alinhada sobe-a para `2` logo à primeira, o
  que a comparação recusa. ⇒ a porta entrega ali **exactamente o que shipava** (sem
  regressão) e a paciência corta o desperdício. ⛔ Com o alinhamento **desligado** aquela
  mesma peça chegava a `1,04 / 2,0° / p99 22,8 / >60 0` — *há ali um ganho que esta lei não
  alcança*, e a hipótese seguinte é a **direcção principal ser ruidosa por face** (ela vem de
  `ph2d_mesh::principal_dirs` sem qualquer suavização de vizinhança).
- ⚠️ **Na `sculpt_hooked` grossa o `fid máx` sobe** (`6,25 %` → `7,61 %`) e as dobras vão de
  `2` para `3`. É **um vértice** (o `p95` fica em `0,27 %`) e a peça já falhava ali; mas está
  nomeado.
- ⛔ **O `>60°` não entra no alinhamento como restrição** — ele só entra na *escolha* da
  ronda. Uma relaxação que nunca criasse uma face péssima é outra obra.

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
