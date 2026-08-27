# HANDOFF DE INTEGRAÇÃO — `line/quadextract` (FECHO da jornada de 2026-08-26)

> **Este é o handoff de FECHO.** O irmão
> [`HANDOFF_INTEGRACAO_line_quadextract_2026-08-26.md`](HANDOFF_INTEGRACAO_line_quadextract_2026-08-26.md)
> é o de **conteúdo** (o que cada wave fez, as recusas medidas, a obra seguinte) e continua
> a ser o documento a ler. Este traz o que o **integrador** precisa: identidade, superfície
> de colisão, o que só o `ship.sh` apanha, e o que smokar.

## §1 — Identidade

| | |
|---|---|
| branch | `line/quadextract` |
| HEAD | ⭐ **o commit de fecho desta jornada** (o último de `git log --oneline main..HEAD`) |
| merge-base com `main` | `0f5ce8040` |
| commits | **53** (+ o de fecho) |
| ficheiros | **65** |

## §2 — Foundational / compartilhado tocado, e por quê

⚠️ **Tudo aditivo.** Nenhuma assinatura pública existente mudou de forma.

| onde | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-mesh/` | `manifold.rs` (`DoubledReport`, `drop_doubled_faces`), `feature_edges.rs` (`boundary_feature_edges`) | ⭐ sim — funções novas |
| `crates/ph2d-remesh-iso/` | `DOUBLED_REPAIR` liga, `border.rs` **novo** (corte de LOC) | sim |
| `crates/ph2d-trace/` | `patches.rs`, `prune.rs`, `lib.rs` | sim |
| `crates/ph2d-gridmap/` | `align.rs`, `arcline.rs`, `assembly.rs`, `round_report.rs` **novos**; `weld_solve.rs`/`weld_round.rs`/`round.rs`/`solve.rs` estendidos | sim |
| `crates/ph2d-quadextract/` | `walk.rs` (resgates), `lib.rs`, gates e exemplos | sim |
| `crates/ph2d-quadfill/` | `finish.rs` expõe `smooth`; `examples/fill_chain.rs` **novo** | sim |
| ⚠️ `shells/desktop/` | `sculpt3d_history_retopo_{extract,global}.rs`, `_remesh.rs`, `_scenes_quad.rs`, `_quad_shape.rs`, `_remesh_refusal.rs`, `sculpt3d_history.rs` | sim — ⚠️ **é o shell**, ver §3 |

⭐ **Quatro ficheiros NOVOS nasceram de cortes de LOC** (`border.rs`, `assembly.rs`,
`round_report.rs`) — corte por **responsabilidade**, nunca allowlist. Os três estavam
**abaixo** do tecto na base e esta linha empurrou-os por cima; o gate
`workspace_src_files_under_loc_cap` estava **vermelho** e só o fecho o apanhou.

## §3 — Símbolos que podem COLIDIR

**Saída de `bash scripts/collision-surface.sh`, colada, não escrita de memória:**

```text
SUPERFÍCIE DE COLISÃO — line/quadextract contra main
  merge-base 0f5ce8040   ·   50 commit(s)   ·   62 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                         97   (base: 97)
      └ tripla do gate               (97, 13, 14)   (base: (97, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  71   (base: 71)
    ph2d-script (espelho)                  71   (base: 71)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR
    último no disco: 0167   próximo livre: 0168
    esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC — 5 ✗ (CURADOS depois desta leitura; ver §2)
```

⚠️ **PRAZO DE VALIDADE:** esta tabela mede a linha contra o `main` de **2026-08-26**.
⇒ **O integrador RE-RODA `collision-surface.sh` imediatamente antes de fundir.**

⭐ **Nenhum número que soma entre linhas foi mexido:** zero schemas, zero registos, zero
ADR, zero pacote externo novo. *A única superfície partilhada é o `shells/desktop`, e ali
os ficheiros tocados são todos do módulo **sculpt3d/retopo** — nenhuma linha paralela
conhecida os toca.*

⚠️ **Uma env nova, e a chave pode colidir com outra linha:** `PH2D_GRIDMAP_ARCLINE`
(nasce **desligada**). Vizinhas já existentes na mesma crate: `PH2D_GRIDMAP_WELD`,
`PH2D_RETOPO_EXTRACT`, `PH2D_RETOPO_LEGACY`.

## §4 — Contratos congelados encostados

⭐ **NENHUM.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool` **intocados**
(confirmado pela sonda acima). Nenhum ADR criado ⇒ fora da disputa do `0168`.

## §5 — O que só o `ship.sh` apanha (o gate de integração NÃO roda)

| item | estado nesta linha |
|---|---|
| `cargo fmt --all` | ⚠️ corrido só nas crates tocadas. **O `fmt --all` da árvore combinada pode reexpandir ficheiros e reabrir um tecto de LOC** — precedente registado nesta casa. |
| `typos` | ✓ nos ficheiros da linha; ⚠️ não na árvore inteira |
| `machete` | ⚠️ **uma dev-dependency nova**: `ph2d-quantize` em `ph2d-quadextract` (só para a sonda `chain_info`). É **usada**, mas é o tipo de aresta que o machete comenta. |
| `deny` / `audit` (RUSTSEC) | ⚠️ não corridos — nenhum pacote **externo** novo, então o risco é de deriva do `main`, não da linha |
| clippy `--all-targets` + features | ✓ nas 6 crates do diff; ⚠️ não na workspace inteira |
| ⭐ **gate batched** | ✓ **12 040 / 12 041** (`nextest-impacted --no-fail-fast`, `CARGO_INCREMENTAL=0`). ⚠️ A única ✗ é `only_the_lower_row_breathes_and_it_moves_with_the_playhead` — **membro nomeado da família de flakes de carga** no `CLAUDE.md` §5.0; **verde sozinha**, com a máquina a `load 19,5`, e o diff desta linha não toca em áudio. |
| ⛔ **`physics_ecs_c9`** | **não tocado por esta linha**, mas o `CLAUDE.md` §5 avisa que ele está **por re-capturar** desde a linha `components` — é o item mais provável de reprovar a matriz 3-OS |

## §6 — Ordem, dependências e o que smokar

**Ordem:** os 53 commits são lineares e não têm dependência cruzada com outra linha.
Integração por `--ff-only` sobre o `main`; nenhum conflito esperado fora dos ficheiros do
módulo.

### ⭐ O que MUDA para o artista

⚠️ **Muito pouco, e é de propósito.** A jornada de hoje foi **diagnóstico**: 10
hipóteses medidas, 3 waves construídas, e **a saída do botão `Quad Retopology` é
byte-idêntica** à de antes em todas as peças do corpus.

O que mudou de facto no produto veio das waves **anteriores** desta linha (a reparação de
não-variedade, os resgates de órfã, o alisamento): elas já foram smokadas pelo Enio em
24–26/08 e ele validou («o melhor resultado conseguido até agora»).

### ⛔ O que NÃO foi smokado

1. ⚠️ **Os três cortes de LOC** (`border.rs`, `assembly.rs`, `round_report.rs`) — são
   movimentos mecânicos verificados por `cargo check` + a suíte, **mas nenhum humano
   correu o app depois deles**. *Um corte de ficheiro que compila e passa a suíte ainda
   pode ter mudado um `pub(crate)` que só o binário exercita.*
2. `PH2D_GRIDMAP_ARCLINE=1` — o caminho novo. ⛔ Nasce desligado e **não faz nada** nas
   peças reais (recusa 100 %, §23.17); não precisa de smoke, precisa de saber que existe.

### O smoke, se o Enio quiser confirmar que nada regrediu

```
cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --release
```

1. Abra o pill **MODEL** ⇒ não, é o **3D/Sculpt**: carregue uma escultura e carregue no
   botão **`Quad Retopology`**.
2. O resultado tem de ser **o mesmo** que ele viu em 24/08 — mesma contagem de quads,
   mesmos buracos (ou a falta deles).
3. **Como saber que deu errado:** se a peça sair com buracos que não tinha, ou o botão
   recusar com uma mensagem nova, é regressão dos cortes de LOC — e `git revert` dos três
   commits de corte é suficiente para isolar.

## §7 — A obra seguinte, guardada

⭐ [`PLANO_arcos_no_sistema_dos_fechos.md`](../quad-remesh/PLANO_arcos_no_sistema_dos_fechos.md)
— a restrição «este arco é uma isolinha» entra **dentro** do `ClosureSystem`, com os cinco
passos, o controlo de cada um, e as **cinco tentativas já medidas e rejeitadas** que não se
reconstroem.

## §8 — `incremental/` reclamado

`rm -rf "$(git rev-parse --show-toplevel)"/target/*/incremental` — corrido **depois** do
gate batched e deste handoff, como manda a DIRETRIZ §1.5.9 item 7.

---

**Resumo:** linha `quadextract` pronta. 53 commits sobre `0f5ce8040`, 65 ficheiros, tudo
aditivo, **zero** schema/registo/ADR/contrato tocado, **zero** pacote externo novo. Gate
batched verde. ⚠️ Três cortes de LOC por smokar e uma dev-dependency nova para o machete.
Aguardo ordem de integração.
