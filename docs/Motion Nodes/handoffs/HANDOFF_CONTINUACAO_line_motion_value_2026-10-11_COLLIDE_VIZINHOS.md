# CONTINUAÇÃO — `line/motion-value`, o `motion.collide` na CPU por vizinhos (11/10)

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
  integrado nem enviado.** Onda anterior (linha rápida):
  [`…_LINHA_RAPIDA_FECHO.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_LINHA_RAPIDA_FECHO.md).
- Commits desta onda: `61ba12aec` (grelha) · `a8d53c05f` (clippy/fmt) · `82d2405c8` (cada par uma vez)
  · o deste handoff. O §10 do [handoff de integração](HANDOFF_INTEGRACAO_line_motion_value_2026-10-11_CICLO_20.md)
  resume as duas ondas para o integrador.

## O que esta onda fez

**O `push_apart` (CPU) deixou de ser O(n²)** e dá **os mesmos bits**:

- `ph2d-contact` ganhou uma porta pública, só acrescentada: `GrelhaDeDiscos` (`grelha.rs`, fim do
  ficheiro + `pub use`) — a grelha CSR da separação de peças, numa camada, com `constroi(p, lado)`,
  `vizinhos_de(k, out)` (crescente), `candidatos_previstos()`, `lado()`.
- `crates/ph2d-node-motion-collide/src/vizinhos.rs`: o MESMO laço de todos-os-pares (`i` crescente,
  os `j > i` crescentes, o par espalhado aos dois) com os `j` vindos da grelha. `Discos::par` (em
  `lib.rs`) é a única conta do par para os dois caminhos. `push_apart_com(…, grelha: bool)` existe
  para o gate ter a referência.
- **Lado = `2 · r_max`, sem folga.** Procurado em float32 (`busca_par_perdido.py`, não versionado):
  108 milhões de pares que tocam, coordenadas de `2⁻¹⁰` a `2⁴⁰` — zero em células não vizinhas;
  controlo `lado = r_max`: 1,9 milhão perdidos. (É a lei que a `ph2d-contact` já usava.)
- Recusa a grelha: coordenada NÃO FINITA (um NaN passa o guarda do `par` e toca toda a gente; o
  `f32::max` ignora NaN — a 1.ª redacção deixava-o passar e o gate apanhou) e custo
  (`vale_a_pena`: `c · (3 + 0,5·log₂ c) < n²/4`).

**Sonda `sonda_custo_da_grelha`, `--release`, load 1,5, intercalada, o mínimo de 2:**

| caso | grelha | todos-os-pares |
|---|---|---|
| 16 384, lado 1, raio 0,3, 12 passadas | 0,004 s | 2,378 s |
| raio 0,6 | 0,012 s | 2,378 s |
| raio 2 | 0,116 s | 2,414 s |
| raio 12 | recusada | 2,817 s |
| **cena `=8`** (32 761, gap 0,25, raio 0,405, 8) | **0,105 s** | **6,381 s** |

**Gates** (`vizinhos_tests.rs`): os mesmos bits em 3 nuvens soltas (raios variados, alfinetes,
`falloff` 0/0,5, corpos com repouso, coincidentes), uma apertada (6 000), longe da origem (3e5,
4e6 — a grelha CORRE), com NaN; a decisão de custo (raio 0,6 e 2 usam, 12 recusa). **Mutações:**
8/8 sangram na versão anterior (ordem, ramo, sinal, lado curto, finitude, custo sim/não) + lado
curto longe da origem + ordem inversa na versão final (2 vermelhos). O eixo do coincidente sobrevive
por desenho (os dois caminhos usam o mesmo `par`). Portão (`verificador`, sobre `61ba12aec`):
`nextest-impacted` 5 228/5 228 (todos os dependentes da `ph2d-contact`), placa do collide 10/10,
censos 117/117; clippy/fmt limpos depois de `a8d53c05f`. Depois de `82d2405c8`: os 39 do collide.
Binário do smoke refeito por último (o anterior levava o registo temporário): 2.ª corrida
`Finished \`smoke\` profile [optimized] target(s) in 0.22s`, e zero ocorrências do registo nele.

## ABERTO (nomeado, com o número)

- ⭐ **Sem placa, a cena `=8` cozinha o `collide` ~20 vezes por segundo a 1–2 fps** — medido com um
  registo temporário no smoke (já retirado): 412 chamadas em ~20 s, 42 ms cada, e o HUD a dizer
  `1 fps · 520 ms` (com placa: `38 fps · 25,8 ms`). Ou seja, VÁRIOS cozimentos por quadro desenhado
  — suspeita: a bomba recupera os tiques atrasados um a um, e um nó `Pure` não precisa dos tiques
  intermédios. Comece por `MotionCookPump::advance_or_scrub_scoped` (`ph2d-eval-motion`). É o que
  falta para o caminho de reserva ficar fluido; o `collide` em si já custa 42 ms nesta cena.
- Se a `ph2d-contact` deve ganhar `opt-level = 2` em `dev`: medido sob load 40–100, sem sinal —
  por medir com a máquina livre (as cenas `pilha`/`carimbo` usam o motor de contacto).
- Docs > 40 KB que ainda não foram partidos (o maior: `115_o_colisor_sai_do_grafo.md`, 170 KB).
- Os abertos do ciclo 20 continuam: §8 e §9 do handoff de integração.

## Próxima janela

O item ⭐ acima. Integrar e enviar só por ordem do Enio.
