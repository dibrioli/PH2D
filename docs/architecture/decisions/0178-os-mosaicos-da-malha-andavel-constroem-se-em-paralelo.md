# ADR-0178 — Os mosaicos da malha andável constroem-se em paralelo: exceção `rayon` na `ph2d-navmesh`

- **Status:** Accepted (decisão técnica delegada — `feedback_architecture_decisions_are_delegated_to_the_gold_standard`)
- **Data:** 2026-10-04
- **Linha:** `line/components` (plano 30 §22.2, W14)
- **Amparo:** ADR-0109 (os TRÊS invariantes que qualificam uma exceção ao «sem `rayon`») · ADR-0145 /
  0156 / 0158 / 0159 / 0175 (cada uso novo, o seu ADR)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — quem integrar reconta-o contra o `main` do
> dia, nunca o copia daqui.

## Contexto

Ao dar play numa cena grande (`100 × 100 m`, `1 000` obstáculos, `100` lamas), a ponte monta a malha
andável a frio: `~55–60 ms` num só tique — um engasgo visível. Medido por fase (plano 30 §22.2,
sonda `sonda_frio_w14` com cronómetros provisórios): `~95 %` do tempo é trabalho POR MOSAICO (recuo e
corte canónico, união e diferença do Clipper, os pedaços das lamas, a triangulação do `spade`, a fusão
em convexos), e só a montagem final (`MalhaPorBlocos::monta`, `~2,6 ms`) é de toda a malha. O piso das
bibliotecas (§20.5) não se move.

## Decisão

`TiledMesh::update_with_areas` separa a actualização em três tempos:

1. **série** — a assinatura de cada mosaico; os que não mudaram ficam; os outros vão para a lista
   `faltam`, pela ordem da chave;
2. **paralelo** (`rayon`, a partir de `MOSAICOS_EM_PARALELO = 2`) — `constroi` de cada mosaico da lista;
   o `collect` de um iterador indexado devolve os resultados NA ORDEM da lista;
3. **série** — cada resultado entra na montagem (`blocos.poe`) pela mesma ordem de antes, e a montagem
   corre uma vez.

**Os três invariantes do ADR-0109 valem:**

1. **Sem redução entre tarefas.** Cada mosaico é função pura das formas que lhe tocam, do anel da região
   e dos parâmetros; nada se soma entre mosaicos (a costura é exacta pelo CORTE CANÓNICO, não pela
   ordem — cabeçalho de `tiles.rs`).
2. **Sem estado mutável partilhado.** A tarefa lê `&self` (parâmetros, lado) e os índices montados na
   fase série; escreve só o seu `Mosaico`. A montagem e as estatísticas são da fase 3, em série.
3. **Sem RNG nem transcendental.** Tudo em inteiros da grelha e `sqrt`; o gate do caminho do hash já lê
   esta crate inteira.

⇒ a malha é **bit-idêntica independentemente do número de núcleos** — e na web o `rayon` cai sozinho
para uma thread (stack §11).

**Cerca:** `rayon` entra na `ph2d-navmesh` SÓ nesta função (comentário no `Cargo.toml`). Qualquer outro
uso exige ADR novo.

## Prova

- A impressão digital das malhas (`sonda_frio_w14 IMPRESSAO=1`: 3 cenas × 3 raios × lamas sim/não ×
  fusão sim/não × 5 mudanças incrementais, `180` malhas — vértices, anéis, áreas, paredes, ilhas, área,
  falhas): `42f3d779b02e0329` antes e depois.
- Gate `os_mosaicos_feitos_em_paralelo_sao_os_de_uma_thread_ao_bit` (`tiles_tests.rs`): a mesma cena
  num pool de 1 thread e num de 8, a frio e depois de mudanças — a malha campo a campo.
- Os oráculos ao bit de sempre (`a_montagem_por_blocos_e_a_montagem_inteira_ao_bit`,
  `a_triangulacao_e_a_fusao_de_agora_sao_as_de_antes_ao_bit`, `incremental_e_a_frio_dao_o_mesmo`) e o
  hash/replay da ponte.

## O ganho (o mínimo de 8 alternadas antes/depois, `medir_custo SO_GRANDE=1`)

| | antes | depois |
|---|---|---|
| a frio, `100` lamas | `55,3 ms` | **`6,8 ms`** |
| a frio, sem lama | `32,1` | **`5,1`** |
| uma lama a mexer | `3,42` | **`2,42`** |
| uma porta (sem / com lama) | `1,12` / `1,74` | `1,12` / `1,80` |

(load `18–57` na corrida; os números a load `≤ 5` no plano 30 §22.2.)

## Alternativas recusadas

| alternativa | porquê |
|---|---|
| `std::thread::scope` | a stack (§11) manda o `rayon` como a única biblioteca de paralelismo; e na web um `spawn` falha onde o `rayon` cai para uma thread |
| optimizar os pedaços das lamas (o maior por mosaico com lama) | o paralelo leva o total abaixo de um quadro; o que fica O(malha) é a montagem, que é série por natureza |
| mosaicos mais pequenos | recusado no §20.5 (a procura paga as costuras) |
