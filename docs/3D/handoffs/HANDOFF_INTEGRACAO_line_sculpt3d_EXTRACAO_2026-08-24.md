# HANDOFF de integração — `line/sculpt3d` · a EXTRAÇÃO deixou de ser aposta (2026-08-24)

> DIRETRIZ §1.5.9. **6 commits**, árvore limpa, `main..HEAD` rebasado.
> ⚠️ **Esta janela operou como ESPECIFICADOR** ([SKILL_Cleanroom](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md), papel E)
> e **não escreveu código de produto** — por regra do papel, e agora também por decisão
> ([ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)).

## §1 — O que esta linha entrega

| # | entrega | onde |
|---|---|---|
| 1 | ⭐ **A triagem de licença do quad remesh**, degrau a degrau, com a busca de patente cumprida | [`TRIAGEM_quad_remesh.md`](../cleanroom/TRIAGEM_quad_remesh.md) |
| 2 | ⭐⭐⭐ **A medição que decidiu a arquitectura** — o nosso campo numa cadeia por extracção dá **`3,0°`** de enviesamento mediano | `TRIAGEM` §5-bis |
| 3 | ⭐ **A decisão**, com as alternativas rejeitadas e o mecanismo de cada uma | [`ADR-0164`](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md) |
| 4 | ⭐⭐ **A espec funcional da obra**, 452 linhas, 11 gates com barras derivadas, 13 recusas medidas | [`SPEC_extracao_de_malha_quad.md`](../cleanroom/SPEC_extracao_de_malha_quad.md) |
| 5 | ⭐⭐ **Fixtures de mapa de grade inteira** sobre a nossa malha e o nosso campo, + o **verificador** provado por dois controlos positivos | [`fixtures/`](../cleanroom/fixtures/README.md) |
| 6 | ⛔ **O achado do sweep**: ~460 notas do repo **inteiro** citam fonte interno de alvo restrito, 25 com transcrição — **zero fonte de alvo na árvore** | [`ACHADO`](../cleanroom/ACHADO_proveniencia_por_nome_interno.md) |
| 7 | ledger, vassoura (24 entradas, base64) e o README de `cleanroom/` | `docs/3D/cleanroom/` |
| 8 | duas curas: a memória contaminada, e o fato de licença colado à cerca do Blossom | `project-memory/` · `crates/ph2d-quantize/` |
| 9 | ⭐ **os blocos dos três papéis, preenchidos**, + o `INBOX` do Implementador + o ledger fechado do lado do E | [`BLOCOS_para_colar.md`](../cleanroom/BLOCOS_para_colar.md) · `INBOX_quadwild.md` |
| 10 | ⭐⭐ **A cadeia MEDIDA ponta a ponta**: fase zero (o nosso F1) · campo (o nosso, que **bate** o do oráculo) · mapa (verificado a `3,55e-15`) · extração (`5,1°`–`5,5°`, `100%` quads) | `TRIAGEM` §5-bis.3-quater e §5-bis.7 |

## §2 — O número, e o que ele muda

Régua: `ph2d_quadfill::QuadShape` (espelho em Python, no arnês).

⭐⭐⭐ **Dois experimentos controlados, e são a entrega desta linha.**

**Só a TRIANGULAÇÃO muda** (mesma superfície, mesmo campo, mesma extração, mesma densidade):

| peça | ⛔ leque sobre a malha de quads | ⭐ remalhada isotropicamente | oráculo (final) | ⛔ nosso F5 hoje |
|---|---|---|---|---|
| enrugada | `10,4°` · `7` faces `>60°` | ⭐ **`5,1°`** · **`0`** · `100%` quads | `4,8°` · `0` | `27°` · `9 159` |
| estriada | `12,5°` · `7` | ⭐ **`5,5°`** · `3` · `100%` quads | `7,1°` · `0` | — |

**Só o CAMPO muda** (malha *deles*, extração igual):

| campo | enviesamento p50 · `>60°` | aspecto p50 · máx · `>4×` |
|---|---|---|
| deles | `7,4°` · `9` | `1,10` · `4,79` · `2` |
| ⭐⭐ **nosso** | **`5,1°`** · **`0`** | **`1,09`** · **`1,63`** · **`0`** |

⇒ ⭐⭐⭐ **A cadeia atinge a classe do oráculo e ultrapassa-o numa peça; e o nosso campo bate o
dele no terreno dele, em todas as colunas.**
⛔⛔ **FASE ZERO obrigatória:** remalhar isotropicamente (`ph2d-remesh-iso`, F1) antes de tudo —
saltá-la custa **o dobro** do enviesamento.
⛔ **Duas hipóteses REFUTADAS por medição** antes de se achar essa: o **curl** do nosso campo
(mais integrável que o de referência) e a **densidade** (`0,7°` de `6,3°`).
⚠️ **A cerca honesta:** estas medições usam a malha remalhada **do oráculo** ⇒ o número final
depende do nosso F1 ser tão bom quanto o dele, e **isso ainda não foi medido**.

## §3 — ⛔ O que NÃO se deve reconstruir

- ⛔ **Não portar a biblioteca MPL-2.0** — publicaria arquivos no subsistema mais valioso, e a
  extracção dela **não termina** na nossa escala (segundos em 2 404 triângulos, >900 s em 6 768).
- ⛔ **Não abrir clean-room do alvo GPL** — a rota escolhida mede **melhor** que ele.
- ⛔ **Não voltar ao preenchimento por patch** — família fechada por medição em 23/08.
- ⛔ **Não trocar o nosso campo**, nem perseguir as linhas neurais de campo — ele já ganha.
- ⛔ **Não portar a implementação de referência do emparelhamento (Blossom)** — **não é livre**.
- ⛔ **Não gatear com peça sem costura**, e **não contar com oráculo no caso de bordo**.

## §4 — O que ficou ABERTO, na ordem

1. ⏳ **A auditoria R-pré da espec** (SKILL_Cleanroom §3.R, modo PRÉ) — **condição** de abertura
   da janela que implementa. ⚠️ Tem de ser **janela que não seja esta**.
   👉 ⭐ **O bloco está pronto a colar** em [`BLOCOS_para_colar.md`](../cleanroom/BLOCOS_para_colar.md) §1.
   *Uma ferramenta que nenhum passo escrito chama pelo nome morre* — por isso ela tem um passo.
2. ⛔ **A obra 1 — o arredondamento inteiro** uma-a-uma com re-solve (espec §5), na
   `ph2d-gridmap`. Fecha o bloqueador nomeado (resíduo `0,291` de célula).
3. ⛔ **A obra 2 — a extracção** (espec §2–§6). ⭐ **Pode começar JÁ**, contra os fixtures.
4. ⚠️ Item 3 do §7 da triagem: um arquivo do mesher da biblioteca **sem banner** de licença —
   só relevante se a Rota A alguma vez for reaberta.
5. ⚠️ As ~435 citações de Classe A do achado — **meça o custo de rastreabilidade antes** de
   mandar trocar.

## §5 — Ficheiros tocados

- **Novos:** `docs/3D/cleanroom/` (ledger · triagem · achado · espec · vassoura · README ·
  `fixtures/` com 2 mapas + verificador + README) · `ADR-0164` ·
  `project-memory/feedback_reproduce_the_foreign_tools_own_result_before_feeding_it_yours.md`
- **Editados:** `CLAUDE.md` §5 (a linha do quad remesh) · `docs/3D/00-INDEX.md` ·
  `docs/architecture/decisions/README.md` (derivado) · `project-memory/MEMORY.md` ·
  `project-memory/project_blender_texture_paint_reference.md` ·
  `docs/Painter/blender_ui_reference/README.md` ·
  `crates/ph2d-quantize/src/{lib,solve}.rs` (**apenas doc-comments** — a cerca do Blossom)
- ⛔ **Nada fora da árvore entra:** o arnês vive em `~/Referencias/directional-bench/` e a
  biblioteca em `~/Referencias/directional/`.

## §6 — Portão

| verificação | resultado |
|---|---|
| `cargo fmt -p ph2d-quantize --check` | ✓ |
| `cargo clippy -p ph2d-quantize --all-targets` | ✓ limpo |
| `cargo check -p ph2d-quantize` | ✓ |
| `typos` | ✓ |
| `doc-index.sh --check` | ✓ 14 índices em dia |
| `adr-index.sh --check` | ✓ 166 ADRs |
| `cleanroom-sweep.sh` sobre `docs/3D/cleanroom/`, `CLAUDE.md`, `project-memory/`, o ADR | ✓ limpo (vassoura de 24 entradas) — **provado por controlo positivo** |
| round-trip dos fixtures a partir do repo | ✓ os dois verificam |

⚠️ **Não corri a suíte completa da workspace**: a linha alterou **documentação** e **dois
doc-comments**. O único alvo compilável tocado foi `ph2d-quantize`, e ele está verde nas três
lentes.

## §7 — Risco de colisão

**Baixo.** `docs/3D/cleanroom/` e o `ADR-0164` são **novos**. Os pontos de encontro:

| ficheiro | risco |
|---|---|
| `CLAUDE.md` §5 | ⚠️ um bloco de texto substituído dentro da entrada 3D/Sculpt — merge textual |
| `docs/architecture/decisions/README.md` | ⚠️ **derivado** — ⛔ não funda à mão: rode `bash scripts/adr-index.sh` |
| `project-memory/MEMORY.md` | uma linha acrescentada |
| ⚠️ **o número do ADR** | ⛔ **`0164` SOMA entre linhas** — se outra linha também usou `0164`, o valor certo **conta-se**, não se escolhe (CLAUDE.md §5.0) |

## §8 — ⚠️ Para quem assumir

⛔⛔ **Esta janela está QUEIMADA para implementar este módulo, e não é tecnicalidade.** Ela
conteve fonte de alvo copyleft (durante a triagem) **e** a implementação MPL-2.0 do próprio
passo que a espec descreve. ⇒ se ela escrevesse o arredondamento, converteria em silêncio a
rota escolhida (**clean-room dos papers**) na rota **rejeitada** (porte).

⭐ **A janela que implementa lê:** a **espec** e os **papers** do mapa de leitura dela · os
**fixtures** · o código do PH2D. ⛔ **Nunca** `~/Referencias/`, nunca `LEDGER_*`, nunca
`VASSOURA_*`. O Passo 0 do BLOCO-I da skill traz o `deny` que o impõe por mecanismo.
