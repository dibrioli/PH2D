# CONTINUAÇÃO — `line/motion-value`, a linha RÁPIDA fechada (11/10)

## As regras (R1–R9, de `PROMPT_LINHAS_RAPIDAS_2026-10-11.md` §2 — valem para toda janela da linha)

- **R1 — Janela curta.** Uma onda = uma janela. Passou de ~150 chamadas ou fechou a onda: commit,
  handoff de continuação ≤ 8 KB, PARE.
- **R2 — Edite pela ferramenta `Edit`.** Novo = `Write`; existente = `Edit`. Script só para a MESMA
  troca em N arquivos, com `assert` de contagem.
- **R3 — Edição e teste nunca no mesmo comando.** Edite; `bash scripts/cargo-check-narrow.sh <crate>`;
  só então teste.
- **R4 — Teste dirigido por NOME no laço:** `bash scripts/cargo-test-narrow.sh <crate> <filtro>`. Suíte
  inteira e `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- **R5 — Nada de `until … sleep`.** Comando > 2 min: `run_in_background` e espere o aviso.
- **R6 — Chamadas independentes na MESMA mensagem.**
- **R7 — Doc > 40 KB nunca se lê inteiro:** `grep -n` + `Read` com `offset`/`limit`.
- **R8 — Escreva menos** (comentário 1–3 linhas; resposta ao Enio curta; prompt de subagente: objetivo,
  arquivos, formato da resposta).
- **R9 — Nenhum teste da suíte normal passa de 10 s em `dev`.** Cura: (a) `opt-level = 2` na crate
  quente (§2.1 do prompt); (b) fixture menor que mantém a lei, provada por mutação; (c) só o que não
  afirma nada (`diag_*`, `sonda_*`, `measure_*`) vai a `#[ignore]`. ⛔ Lei afirmada nunca a `#[ignore]`.

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, base `8f20c60de`. **Nada
  integrado nem enviado.** Commits desta onda: `7aa02b678` (docs partidos) · `e22630d38` (perf) · o
  deste handoff.
- O ciclo 20 continua FECHADO: [`HANDOFF_INTEGRACAO_…_CICLO_20.md`](HANDOFF_INTEGRACAO_line_motion_value_2026-10-11_CICLO_20.md)
  (o §10 dele resume esta onda para o integrador).

## O que esta onda fez

**1. O teste de 88 s era UM nó.** Cronometrando cada nó dentro de
`nenhum_no_emite_mais_do_que_o_tecto` (sonda temporária, já retirada): `motion.collide` **83,2 s** de
87,3 s — o laço de pares O(n²) sobre 16 384 pontos × 12 passadas, na CPU de referência. Depois:
`motion.proximity` 3,0 s · `field.shape` 0,65 s · `force.attractor` 0,26 s · os outros 121 < 0,05 s.
(`samply`/`perf` não correm aqui: `perf_event_paranoid = 2` sem sudo.)

**2. A/B intercalado (§2.1), o mínimo de 2, `CARGO_TARGET_DIR` próprio na scratchpad:**

| variante | teste do tecto | dentro dele, o collide |
|---|---|---|
| o0 (antes) | 87,7 s | 83,5 s |
| opt-level 2 só no collide | 12,9 s | 8,8 s |
| + proximity, field-shape, force-attractor | 9,7 s | 8,8 s |
| + a distância antes das outras guardas (`e22630d38`) | **6,2 s** | — |

Remontar após 1 edição real no collide: o0 18,0 s → opt-2 2,4 s. As cenas `pilha`/`carimbo` NÃO
dependem destas crates (iguais nas duas variantes: 7,1 · 6,6 · 4,0 s sozinhas); os ~10 s delas na
suíte são disputa do fan-out, não custo próprio.

**3. A reordenação do collide** (`crates/ph2d-node-motion-collide/src/lib.rs`, `push_apart`): o
teste `d2 >= min_d2` passou para a frente da massa, do `falloff` e da `Estrutura`. Todas as guardas
são `continue` sem efeito colateral ⇒ a ordem não muda um bit. Prova: os 34 testes do collide, a
paridade de placa (10/10, `--run-ignored all`) e o `collide_ceiling` verdes.

**4. Docs partidos** com `doc-split.py` (sha256 em cada corte), pedaços em
`docs/Motion Nodes/archive/<doc>/parte_N.md`, índice «§ / Bug # → pedaço» no topo do vivo:

| doc | antes | vivo | pedaços |
|---|---|---|---|
| `121_as_formas_na_placa.md` | 248 KB | 15 KB | 7 (32–37 KB) |
| `BUGS_motion_nodes.md` | 97 KB | 7 KB (só o #19 aberto) | 3 (28–37 KB) |
| `110_ciclo_6_valor_e_pulso.md` | 75 KB | 22 KB | 2 (22–34 KB) |
| `129_ciclo_20_o_acabamento.md` | 49 KB | 24 KB | 1 (26 KB) |

Os 9 links `#bug-N` da tabela-resumo do BUGS foram reapontados para os pedaços. 3 links já quebrados
no original continuam quebrados (`#bug-6`, `#bug-9`, `flip_pass_camera_tests.rs`).

## ANTES × DEPOIS

| régua | antes | depois |
|---|---|---|
| suíte `ph2d-app-motion` (1 480 testes) | 97,6 s (load 5) | **14,8 s** (load 14–17) |
| teste mais lento da suíte | 91,7 s (o tecto) | 9,8 s (`the_rolling_lock…`, sob fan-out) |
| testes > 10 s na suíte | 2 | 0 |
| `agent-loop-profile` — Edit vs script | 30 % (10 sessões) | 90 % (esta) |
| contexto relido por passo | 433 mil | 137 mil |
| paralelismo de ferramenta | 1,11 | 1,10 ✗ (continua abaixo de 1,5) |
| `cargo test : check` | 2,2× | 7× (inflado pelas corridas de medição A/B) |

Portão de fecho (`verificador`): `nextest-impacted` 5 081/5 081 · clippy `-D warnings` do collide limpo
· `doc-index --check` 17/17 · censos da árvore combinada 117/117.

## ABERTO (nomeado)

- **O collide na CPU continua O(n²).** A 16 384 instâncias e 12 passadas são ~6 s por cozimento mesmo
  otimizado — é o caminho de reserva (`PH2D_GPU_COOK=0`) e o de referência. A cura é uma grelha
  espacial que visite os parceiros de cada disco em ordem CRESCENTE de índice (a mesma ordem da soma de
  hoje ⇒ bit-idêntico; cuidado com o eixo do caso coincidente, que depende de qual índice é o menor).
  Não foi feita nesta onda: é mudança de motor, não de régua.
- Outros docs > 40 KB na pasta (o maior: `115_o_colisor_sai_do_grafo.md`, 170 KB; `111` 88 KB; `114`
  87 KB) — não estavam na lista dos mais relidos; parta-os se uma onda os for ler.
- Os abertos do ciclo 20 continuam: §8 e §9 do handoff de integração.

## Próxima janela

A fila do Motion pelo §5.1 do `CLAUDE.md` e pelos abertos acima. Integrar e enviar só por ordem do Enio.
