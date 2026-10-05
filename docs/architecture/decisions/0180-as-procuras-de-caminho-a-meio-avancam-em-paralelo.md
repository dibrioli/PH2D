# ADR-0180 — As procuras de caminho A MEIO avançam em paralelo: exceção `rayon` na ponte da física

- **Status:** Accepted (decisão técnica delegada — `feedback_architecture_decisions_are_delegated_to_the_gold_standard`)
- **Data:** 2026-10-05
- **Linha:** `line/components` (plano 30 §23, W15)
- **Amparo:** ADR-0109 (os TRÊS invariantes que qualificam uma exceção ao «sem `rayon`») · ADR-0178 (o paralelo
  dos mosaicos, na `ph2d-navmesh`) e os anteriores (cada uso novo, o seu ADR)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — contado contra o máximo do `main` e das worktrees do dia
> (`0179`); quem integrar reconta-o contra o `main`, nunca o copia daqui (o nome do ficheiro, o título, e as
> citações no plano §23 e no `Cargo.toml` da `ph2d-physics-ecs`).

## Contexto

Com muita lama, uma procura de caminho custa `~7 ms` (`~65 000` de trabalho) e há procuras de `25–40 ms`. A W15
tirou o pico do tique (toda procura paga de um orçamento por tique e PÁRA a meio quando ele acaba — plano 30
§23.2), mas em série o orçamento passou a ser a VAZÃO: na cena de stress (`200` agentes, `~40 %` do chão em lama),
`157` agentes esperavam o 1.º caminho ao fim de `8 s`. Subir o orçamento troca a espera pelo pico (`80 000`: o tique
a `15 ms` e ainda `14` à espera). As procuras são independentes, e a máquina tem núcleos parados.

## Decisão

Antes da condução, `fila::procuras_a_meio` avança em PARALELO as primeiras `PROCURAS_EM_PARALELO = 16` procuras a
meio (a mais adiantada primeiro), cada uma com uma fatia do orçamento; as respostas entram na condução pela ordem
das entidades. A fila e a condução, em série, ficam com o que a maior fatia não gastou: o caminho crítico do tique é
UM orçamento.

**Os três invariantes do ADR-0109 valem:**

1. **Sem redução entre tarefas.** Cada procura é função pura da malha, dos custos, dos atalhos, de onde parte e do
   alvo; nada se soma entre procuras.
2. **Sem estado mutável partilhado.** Cada tarefa tem a memória do SEU agente (tirada do mapa antes, devolvida
   depois) e o plano dele (os buffers dele); a malha e a consulta são lidas por todas. Quem avança e quanto
   decide-se ANTES de correr, em série — logo o resultado NÃO depende do número de threads.
3. **Sem RNG nem transcendental.** A procura é `+ − × ÷ sqrt` (a cerca da `ph2d-nav`).

⇒ o resultado é **bit-idêntico com qualquer número de núcleos**; na web o `rayon` cai para uma thread e o tique
paga as fatias em série (o resultado é o mesmo, o relógio não). `PROCURAS_EM_PARALELO` é FIXO e não o número de
núcleos da máquina: o hash do CI compara três sistemas.

**Cerca:** `rayon` entra na `ph2d-physics-ecs` SÓ nesta função (comentário no `Cargo.toml`). Qualquer outro uso
exige ADR novo.

## Prova

- Gate `o_passo_em_paralelo_da_o_mesmo_com_uma_thread_e_com_oito` (`tests/it/nav_fatias.rs`): vinte agentes no
  lamaçal com um orçamento curto, num pool de 1 thread e num de 8 — a posição e a memória de cada agente, a cada
  tique, ao bit; a fixtura exige tiques com várias procuras juntas.
- A procura em fatias é a procura inteira ao bit (`a_procura_em_fatias_e_a_procura_inteira_ao_bit`, `ph2d-navmesh`),
  e um scrub a meio dela devolve a mesma corrida (`nav_fatias`).
- Mutação: o passo que não corre, a resposta que se perde, a fatia que não vai à mais adiantada, uma procura de
  outras entradas avançada — todas sangram (plano 30 §23.7).

## O ganho (`medir_replaneio`, as versões no mesmo processo, 7 rodadas intercaladas, o mínimo; load `22–37`)

`150` lamas, `200` agentes: em série, `189` sem caminho no fim; com `8` em paralelo `0` (o pior tique a `50` agentes
`11,1 ms`, o crítico `80 000`); com **`16`**, `0` e o pior tique depois da porta `12,9 ms` contra `65,9` da procura
inteira (plano 30 §23.5).

## Alternativas recusadas (medidas)

| alternativa | porquê não |
|---|---|
| subir o orçamento | o tique sobe a `15 ms` e ainda há quem espere (§23.4) |
| `PROCURAS_EM_PARALELO` = os núcleos da máquina | o resultado dependia da máquina |
| guardar a malha velha para a procura a meio acabar nela | o anel reteria uma versão da malha por âncora (`~3 MB` cada) |
