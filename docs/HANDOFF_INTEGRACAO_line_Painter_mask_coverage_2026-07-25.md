# Handoff de INTEGRAÇÃO — `line/Painter`: a cobertura da máscara (lei do canal)

**Para:** o agente integrador (por ordem EXPLÍCITA do Enio). **De:** a `line/Painter`, 2026-07-25.
**Escopo:** a reescrita da cobertura da máscara pedida em
[`HANDOFF_line_Painter_mask_rewrite_2026-07-25.md`](HANDOFF_line_Painter_mask_rewrite_2026-07-25.md).
Detalhe técnico completo: [`docs/Painter/25_avaliacao_gpu.md` §13.9](Painter/25_avaliacao_gpu.md).

---

## 1. Identidade

| | |
|---|---|
| branch | `line/Painter` |
| HEAD | o tip de `line/Painter` — **o último commit é este handoff** (um sha literal aqui se auto-invalidaria a cada correção do próprio arquivo; `git log --oneline main..HEAD` é a fonte) |
| base (merge-base com `main`) | `df91ef6ec` |
| commits desta wave | **9**, de `d8018d6bc` até o tip: a lei · o controle morto · o smoke · doc 25 §13.9 · os handoffs · CLAUDE.md §5 · fmt · este handoff · 1 de memória |
| commits da linha desde a base | ⚠️ **46** — esta wave são os **9 do topo**; os 37 abaixo são **waves ANTERIORES da mesma linha que ainda não integraram** (ver §7) |

⚠️ **CORREÇÃO de uma afirmação que eu quase shipei errada:** a 1ª versão deste handoff dizia que a linha
não tinha commits pendentes de waves anteriores. **Tem 37** — `git rev-list --count main..HEAD` dá 46, e
só os 9 do topo são desta wave. Ver §7 para a lista e o que ela significa para a ordem de integração.

## 2. Foundational / compartilhado tocado, e por quê

| arquivo | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-painter-brush/src/stroke_cover.rs` | **NOVO** — a lei da cobertura por-traço (`StrokeCoverLaw`, `StrokeCover`, `cover_add`) | sim (arquivo irmão novo, o padrão de isolamento) |
| `crates/ph2d-painter-brush/src/stroke_cover_tests.rs` | **NOVO** — 4 gates da lei | sim |
| `crates/ph2d-painter-brush/src/lib.rs` | `pub mod stroke_cover;` + `pub use {StrokeCover, StrokeCoverLaw};` | sim, 2 linhas em ponto de extensão |
| `crates/ph2d-painter-brush/src/dab.rs` | o parâmetro `mask: Option<&mut [u8]>` das 3 assinaturas virou `cover: Option<StrokeCover<'_>>` | **NÃO** — troca de tipo (ver §3) |
| `crates/ph2d-painter-brush/src/dab/bands.rs` | o kernel per-pixel pergunta a lei a `cover_add` em vez de conhecê-la | não |
| `crates/ph2d-painter-brush/src/dab/tests.rs` | 2 gates do cap de pigmento passam `StrokeCover::build_up(&mut mask)` | não |
| `crates/ph2d-panel-painter-layers/src/paint_brush.rs` | a row **Accumulate** ganha `&& !brush.is_mask` | sim (1 condição) |
| `crates/ph2d-panel-painter-layers/tests/seam.rs` | +1 gate (presença E ausência) | sim |
| `shells/desktop/src/mask_smoke.rs` | **NOVO** — a cena `PH2D_MASK_SMOKE=1` | sim |
| `shells/desktop/src/main.rs` | `mod mask_smoke;` + `mask_smoke_done: false` | sim, 2 linhas |
| `shells/desktop/src/app_state.rs` | `pub(crate) mask_smoke_done: bool` | sim, 1 campo |
| `shells/desktop/src/render_loop/mod.rs` | o bloco de spawn do smoke, ANTES do bloco do Wet Paint (espelho exato do impasto/wetpaint) | sim, 1 bloco |
| `shells/desktop/tests/the_smokes_open_the_painter_in_digital.rs` | o gate varre o 3º smoke | sim |

O resto é a pasta do Painter (`ph2d-tool-painter/src/tool/paint/*`).

## 3. Símbolos que podem COLIDIR com outra linha

**Zero ids, zero consts de UI, zero tokens, zero chaves de i18n, zero entradas em lista ordenada.**
Nada de `NodeId(...)`/`PAINTER_*` novo. O que o integrador deve grepar:

| símbolo | valor / forma | onde |
|---|---|---|
| `StrokeCoverLaw` (enum novo, 2 variants) | `BuildUp`, `Envelope` | `ph2d-painter-brush::stroke_cover` |
| `StrokeCover<'a>` (struct nova) | `{ buf: &'a mut [u8], law: StrokeCoverLaw }` | idem |
| `cover_add` (`pub(crate)`) | — | idem |
| `PainterTool::stroke_cover_law` (`pub(super)`) | — | `tool/paint/stamp_route.rs` |
| `mask_smoke_done` | `bool` em `AppState` | `shells/desktop/src/app_state.rs` |
| `PH2D_MASK_SMOKE` | env var nova | `shells/desktop/src/mask_smoke.rs` |
| módulos de teste novos | `mask_probe`, `mask_tests` (filhos de `paint::mask`) · `stroke_cover_tests` | — |

⚠️ **A mudança de ASSINATURA é o único ponto de atrito real:** `stamp_dab_textured_masked` e
`stamp_dab_ramped` trocaram o tipo do último-menos-um parâmetro (`Option<&mut [u8]>` →
`Option<StrokeCover<'_>>`). **Chamadores no `main` de hoje: 4** (2 no `ph2d-tool-painter/tool/paint/
stamp_cache.rs`, 2 nos testes da própria `ph2d-painter-brush`), todos já convertidos aqui. Se outra
linha tiver criado um 5º chamador, o conflito aparece como erro de tipo no `cargo check --workspace` do
gate da árvore combinada (não como conflito textual) — a conversão é mecânica:
`Some(&mut buf)` → `Some(ph2d_painter_brush::StrokeCover::build_up(&mut buf))` para pigmento.

## 4. Contratos congelados encostados

**NENHUM.** Conferido por grep, não por auto-relato:

- `Tool = 12` / `RasterEditTool = 5` / `CanvasPaintTool = 1` / `PanelEvent = 4` — intactos (não há
  método novo em trait nenhum desses; `stroke_cover_law` é um método inerente do `PainterTool`).
- `NodeOp`/`OpResolver`/`NodeManifest` — não tocados.
- Superfície do `ph2d-vector-doc`/`-traits` — não tocada.
- `ph2d-painter-brush` **não é** superfície congelada: os ABIs de pintura (`Brush`/`Stamp=96B`/…) e o
  gate `architecture_painter_contract_surface` foram **removidos** com a crate `ph2d-painter-contracts`
  pelo ADR-0099. É por isso que trocar o tipo do parâmetro do `stamp_dab_*` não exige ADR.
- **Nenhum schema bumpou:** `PROJECT_SCHEMA` fica **29**, `DOC_VERSION` fica **11**,
  `VEC_SCENE_SCHEMA_VERSION` fica **13**. A lei nova não guarda nada no documento — o buffer
  (`PaintState.stroke_mask`) é **por-traço** e já existia; o scratch commitado (`mask_scratch_rgba`,
  que ESTÁ no `ModelSnapshot`) não mudou de representação (segue RGBA u8). Há gate para isso
  (`a_mask_stroke_is_one_undo_step_and_the_next_stroke_starts_fresh`).

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **Nenhuma dep nova** (nem no `Cargo.toml`, nem no `Cargo.lock`) ⇒ `machete`/`deny`/`audit` não têm
  superfície nova; RUSTSEC idem.
- `fmt` rodado com o **pin 1.95** (`rustup run 1.95 cargo fmt`), `--check` limpo na árvore inteira.
- `clippy --all-targets` limpo nas 4 crates tocadas (`ph2d-painter-brush`, `ph2d-tool-painter`,
  `ph2d-panel-painter-layers`, `ph2d-host-desktop`) — **5 warnings de `doc list item overindented`
  foram corrigidos** no `mask_smoke.rs` antes do commit.
- `typos`: não rodado isoladamente; os docs novos são em pt-BR/en com termos do repo (`Wash`,
  `Alpha Darken`, `envelope`). Se o `typos` reclamar, é palavra nova de vocabulário, não erro lógico.
- LOC caps verdes nos dois gates (`architecture_workspace_file_loc_cap` e o
  `shells/desktop/tests/file_loc_caps.rs` da shell — este último **não** roda com `cargo test -p`, e
  entrou no fechamento de propósito). Maiores arquivos novos: `mask_probe.rs` 543, `mask_tests.rs` 336.
- `arch_safe_clamp_only` verde (entrou no fechamento pela lição da `line/physics`).

## 6. Ordem, dependências e o que smoke-testar

**Ordem:** os 9 commits desta wave são sequenciais e independentes entre si exceto que o `535df958c` (a row
Accumulate) só faz sentido depois do `d8018d6bc` (a lei). Nenhum depende de outra linha.

**Gate desta linha, rodado antes de fechar:** `nextest-impacted.sh` = **4073 testes, 4073 passaram**
(1092 deles da shell, então os arch-gates de `shells/desktop/tests/` foram alcançados — a lição da
`line/Vector` de 23/07). Suítes por crate: `ph2d-painter-brush` 269 · `ph2d-tool-painter` 830.
⚠️ Na primeira volta o `ph2d-timeline::nesting_clock the_cost_of_depth_is_linear_not_explosive` falhou:
é a **flake conhecida e PRÉ-EXISTENTE** que o CLAUDE.md §5 nomeia (gate de RAZÃO sensível a carga).
Passa isolado (conferido) e passou na volta seguinte — não é resíduo desta linha.
**Rode as DUAS configurações** (debug e `--release`): o Flip provou que `--release` sozinho esconde
pânico. Ambas rodadas aqui; a sonda de perf dá os mesmos números nas duas (0,9 ms).

**O smoke (é o que decide, e o Enio ainda não o rodou):**

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_MASK_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

O roteiro impresso pela cena, em ordem: pinte arte → chip **MASK**, Size ~60, zoom → **ESFREGUE** a
mesma curva 10-20× sem soltar → lanes vizinhas (os vales têm de ENCHER, sem linha clara) → volte ao
**Brush** (a arte protegida resiste) → **Ctrl+Z**.

⚠️ **Três desvios ESPERADOS contra o build antigo** (medidos, no doc §13.9.6 e no doc do smoke — se o
Enio os reprovar, são decisões de PRODUTO, não bugs): uma passada é mais macia (6,21 px contra 3,53) ·
o miolo fica em 0,984 na 1ª passada e 1,000 na 2ª · a rampa ainda aperta como `N^(−1/2)` (não existe lei
que dê build-up entre traços E borda invariante; a que congela a borda é a união, reprovada no §13.8).

**Não smokado / fora desta wave, NOMEADO:** os métodos de SHAPE (Line/Curve/Ellipse/Polygon/Free Hand)
em modo máscara **não pintam nada** — o roteador de shape intercepta o Down antes do `paint_begin`, o
`ensure_mask_scratch` nunca roda e o scratch fica com 0 bytes (medido). Pré-existente, ortogonal à lei, e
consertar direito exige base congelada por traço (senão o re-stamp por frame re-multiplica o scratch e o
resultado passa a depender da taxa de quadros). Doc §13.9.8.

---

**Resumo:** linha `Painter` pronta — **9 commits desta wave**, sobre 37 de waves anteriores ainda não integradas (§7). Foundational tocado: `ph2d-painter-brush`
(arquivo irmão novo + troca de tipo num parâmetro, 4 chamadores) · `ph2d-panel-painter-layers` (1
condição + 1 gate) · shell (smoke novo, 4 sítios). Zero id/const/token novo; zero contrato congelado;
zero schema. Aguardo ordem de integração.

---

## 7. ⚠️ A linha carrega waves ANTERIORES não-integradas (37 commits)

`git rev-list --count main..HEAD` = **46**. Os 9 do topo são esta wave; os **37** abaixo dela são
trabalho anterior da MESMA linha que nunca integrou, e o integrador precisa saber disso antes de
sequenciar:

| bloco | o quê | estado |
|---|---|---|
| `d8018d6bc`..tip (9) | **esta wave** — a lei do canal de cobertura | gates verdes, **pendente de smoke** |
| `40191df75` + 3 reverts + `1c23b4130`/`38c1f725b`/`c8b48e2e3`/`600a79606` | o §13.6 (envelope entre traços) e o §13.7 (teto por época), **construídos e REVERTIDOS**, + o handoff-tarefa | reprovados pelo Enio; os reverts estão aqui, **não** em `main` |
| `2da916c99`..`4280ba572` + os 4 `diag(painter)` | Onda 5c (máscara toma a via parcial + upload cheio) e o instrumento `PH2D_PAINT_PERF` | doc 25 §13.3-§13.5, smoke OK |
| `abe0123ec`/`608cfa038` | Onda 5b (upload parcial da região suja) | doc 25 §12 |
| `a9057588c`/`ed9563b0d` | Onda 5a (a pintura para de copiar o canvas por move) | doc 25 §11 |
| `97f0ab0a2`..`73fe5b67e` | Ondas 1 e 2 da GPU (máscara/clipping como ops, orçamento do dispositivo) | doc 25 §10 |
| `117023207`..`e8414355c` | doc 24 (transferência sRGB tabelada) + o composite row-parallel | doc 24 |

**Consequência prática:** integrar esta wave integra as anteriores com ela — o que é o esperado para uma
linha, mas muda o tamanho do diff e a superfície de conflito (as Ondas 1/2 mexem no compositor de
`ph2d-render`, e o doc 24 na `ph2d-wet-paint`). Os handoffs de integração DAQUELAS waves já existem em
`docs/` (`HANDOFF_INTEGRACAO_line_Painter_gpu_onda5*`, `..._gpu_ondas_1_2_*`, `..._wet_transfer_*`) e
seguem valendo; este documento cobre **só** a cobertura da máscara.
