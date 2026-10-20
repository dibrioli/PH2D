# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W12 (os cruzamentos em Wet Paint sem «nó»: a tinta do artista por cima, a marca é o `pa` da água dele) → `main` (2026-10-20)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo, medições, decisão do dono e recusas: [doc W12](../10_W12_cruzamentos_em_wet_paint.md). Pedido:
> [PROMPT_W12](PROMPT_W12_cruzamentos_em_wet_paint_2026-10-20.md).
> ⚠️ **W8–W11 ainda NÃO estão no `main`** (mesma branch, integram juntas): [W8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md)
> · [W9](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W9.md) · [W10](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W10.md)
> · [W11](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W11.md) continuam válidos para a parte delas. Este só acrescenta a W12.
> ⚠️ A linha **NÃO foi rebaseada** (ordem do dono: «sem reset nem rebase»): o integrador rebasa.

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Merge-base `ad5f8fe39c`; o `main` está em
  `100d34c8e0` (os mesmos 3 commits de processo dos handoffs W10/W11).
- Commits da W12: `fc699ddbcf` (prompt) · `3783aba95d` (test: vermelho antes da cura) · `4bcd45e3ad` (fix) · `d0713d4fdd` (test: a composição sobre a marca que já havia; a sonda do custo) · `8429143f37`
  (os gates do `caminho.rs` num ficheiro irmão: 723 → 642 linhas, tecto 700) · `ab61b2d8cc` (fix do gate do relógio do
  ponteiro, regressão da W11 — §6) · o commit deste handoff (docs + fotos).

## §2 — O que a W12 entrega

1. **Decisão do dono (20/10, perguntada com a medição):** no Wet Paint a pincelada pintada depois de Parar fica **por
   cima** da gravada (não se funde); a água da gravação que ainda escorra pára no 1.º toque do artista. Memória
   `project_wet_paint_after_stop_paints_over_the_recording_2026_10_20` (+ a linha no índice).
2. **A sessão molhada tem dono** (`WetSession::marca`, o id de `MarcaViva::nasce_a_molhada`). `stamp_dabs` →
   `wet_da_marca`: com a marca a escrever, a sessão que não é a do artista assa e a dele nasce antes de qualquer marca.
   A borracha molhada fica com a lei da W9.
3. **A cobertura é o `pa` do composite** (`wetpaint/composite.rs` devolve-a num plano `u16` da caixa escrita; ×
   `keep`), recomposta a partir da marca do nascimento da sessão (`MarcaDoArtista::recompoe_da_agua`:
   `c = 1 − (1 − c₀)(1 − pa)`, `B` = o do 1.º toque ou a tela congelada da sessão sobre o papel). Exacta em Mix, camada
   única, sem vidro, sem alfa preso; fora disso o texel pintado fica `CHEIA`. A água que anda depois do passo reescreve
   `c` a cada composite.
4. **Histórico:** o passo vai de `comeca_o_passo` ao registo; a água depois do registo estica o delta na absorção
   (`undo_absorb` → `estende_pela_agua`). Os discos provisórios, o miolo e a cobertura do Solid na água deixam de marcar
   na água do artista.
5. **Watercolor, Impasto e os outros modos de mistura ficam com `c` cheia** — medido (doc W12 §1): a aguada funde-se na
   mesma lavagem e é Beer–Lambert linear por canal; o Impasto acende o relevo de todas as pinceladas depois de compor.
6. **Smoke `=6`:** variante `PH2D_MACRO_CAMINHO=cruzamolhada` (a `cruza` em Wet Paint).

### §2-bis — Medições

Gate `a_pincelada_nova_a_cruzar_as_gravadas_nao_deixa_resquicio` (app; `MEIOS` = Digital + Wet Paint; Wet também com a
água a correr, 20 quadros depois do Up), desvio máximo em níveis:

| | antes (W10) | W12 |
|---|---|---|
| Wet Paint, 7 pincéis | 71–116 | **1–2** (a correr e assentada) |
| Digital, 7 pincéis | 0–5 | 0–5 |

Candidatos recusados com a fusão (sonda, pincel macio): marca cheia 112 · sombra ideal 53 · diferença de massa 63.
Assar no 1.º toque: identidade `atual = P + (1 − pa)·B` a 1,4 níveis, Play a 1,7 (doc W12 §1).

Custo (`load` ~35, mesmo processo, intercalado): `on_tick` 1024² Wet com/sem marca — mediana 2,36 / 2,57 ms, média
2,60 / 2,54 (dentro do ruído); A3 `liga+sobe` com traço Wet marcado 0,760 ms (W10: 0,747).

**Mutação 7/10 sangram** (controlos verdes, cada filtro casa 1 teste, âncoras 1×; T1 o gate dos cruzamentos, T2
`a_pincelada_do_artista_em_wet_paint_…`, T3 `a_agua_do_artista_compoe_…`): M1 a gravação não assa (T1+T2) · M2 o
composite marca sempre cheio (T1+T2+T3) · M3 ignora a marca do nascimento (T3) · M4 `B` sempre a tela da sessão (T3) ·
M5 a absorção não estica o delta (T2, o refazer) · M6/M7 os discos provisórios de volta na água do artista
(T1+T2+T3). **Sobrevivem:** M8 (sem o `antes_do_passo` do `comeca_o_passo`: equivalente nos fluxos testados — o
`begin_undo_step` absorve antes e o `registo` já é o início do passo), M9 (o que o passo muda FORA do composite não
recompõe o nascimento: nenhum fluxo testado o faz) e M10 (o alfa preso fora do «exacto»: sem teste com Alpha Lock) —
§8 itens 2–3.

**Testes novos/reescritos:** `palcos_cobertura_tests` (Wet Paint + idades da água) · `marca_painter_tests::a_pincelada_do_artista_em_wet_paint_assa_a_gravacao_e_marca_a_tinta_dele`
(substitui `a_pincelada_do_artista_marca_onde_o_pincel_passou_e_o_desfazer_a_desliga`: a lei da W9 «a água da gravação
continua a correr» mudou de propósito) · `marca_cobertura_tests::a_agua_do_artista_compoe_sobre_a_marca_que_ja_havia`
· SONDAS ignoradas `sonda_w12_tests` (candidatos, assar, custo).

## §3 — Foundational / partilhado tocado (aditivo)

| Crate | O quê |
|---|---|
| `ph2d-tool-painter` | `macro_track/marca.rs` (`recompoe_da_agua`), `undo_marca.rs` (`Molhada`, `nasce_a_molhada`/`acaba_a_molhada`/`molhada_viva`/`agua_cobre`/`estende_pela_agua`), `undo_absorb.rs` (1 linha), `tool/paint/stamp_route.rs`, `tool/paint/wetpaint.rs` (`wet_da_marca`, `wet_lote_do_artista`, o guard), `wetpaint/{session,authored_actions,composite,dab_route,mascara}.rs`, `macro_track/roteiro.rs` (`grava_duas_horizontais_ligado_em`) |
| `ph2d-app-painter` | `macro_track/caminho.rs` (variante `cruzamolhada`), `smoke.rs` (comentário), testes |

Contratos congelados (§6) **não tocados** — `git diff --name-only main...HEAD | grep -E 'ph2d-editor-core/src/tool|ph2d-vec|ph2d-vector|ph2d-nodegraph/src/(op|manifest|cook)'` sai vazio (exit 1).
Sem `Cargo.toml`, sem i18n, sem `PROJECT_SCHEMA`, sem crate nova, sem ADR (`git diff --name-only fc699ddbcf..HEAD | grep -E 'Cargo|i18n|project_schema'` sai vazio).

## §4 — O que um merge pode partir

- ⛔⛔ **Os códigos do `Passo` 53–56 colidem com a `line/PainterWatercolor`** (achado 20/10, `git diff
  main...line/PainterWatercolor -- crates/ph2d-tool-painter/src/macro_track/passo.rs`): lá `53 TiraPedidoMangueira · 54
  MangueiraRgba · 55 TiraPedidoDual · 56 DualRgba`; cá (W11) `53 ColorDropComeca · 54 LimiarDoFill · 55 FillCommit · 56
  ColorDropRestaura`. O número é gravado no blob da macro: a linha que integrar em SEGUNDO renumera os seus para depois
  do último do `main` (contar, nunca escolher — §5.0), e uma macro gravada com a numeração antiga dessa linha
  descodifica-se como a OUTRA variante. A fusão dos dois `match` deixa braços `53 =>` repetidos (o `clippy -D
  warnings` reprova o padrão inalcançável), mas o encode compila calado.
- ⚠️ **`line/PainterWatercolor`** mexe em `tool/paint/wetpaint/` (dab_route, composite, session, authored_actions,
  mascara, `wetpaint.rs`, `stamp_route.rs`) e no motor: conflito textual
  provável (os 7 ficheiros da W12 estão também no diff dela) — em `composite.rs` (o laço por píxel ganhou um `zip` com o plano da cobertura e o bloco do `exata` antes do
  `let Some(sess)`), `dab_route.rs` (o `if !self.wet_lote_do_artista()` à volta do `marca_os_dabs`), `session.rs` (campo
  novo `marca` + o `Clone`) e `authored_actions.rs` (o `nasce_a_molhada` no nascimento, o `acaba_a_molhada` no fim). A
  resolução é manter os dois lados; quem acrescentar outro sítio que ponha `session = None` tem de chamar
  `acaba_a_molhada` (hoje: `wetpaint_end_session` e o `wetpaint_guard`).
- `WetSession { .. }` construída noutra linha (literal de struct) deixa de compilar: acrescente `marca: None`.

## §5 — Contadores (DELTA sobre o merge-base, só a W12)

Crates novas 0 · chaves i18n 0 · ADR 0 · `PROJECT_SCHEMA` 0 · contratos 0 · cenas de smoke: nenhum número novo (`=6`
ganhou a variante `cruzamolhada`; a próxima cena continua `=7`).

## §6 — Prova de fecho

Cadência: 10 commits desde a última verde (W10, `d7abe5bab0`) — a bateria inteira correu, sem rebase (ordem do dono).

| Portão | Resultado |
|---|---|
| `cargo fmt --all -- --check` | ✓ |
| `PH2D_SUITE=fecho` clippy `-p` tool-painter, app-painter, host-desktop `--all-targets -D warnings` | ✓ |
| `architecture_workspace_file_loc_cap` | ✗ → ✓ (`caminho.rs` 723 > 700: os gates foram para `caminho_tests.rs`, `8429143f37`) |
| `censos-da-arvore-combinada.sh` | ✓ 121/121, 12 de 12 censos |
| `PH2D_SUITE=fecho BASE=$(git merge-base main HEAD) nextest-impacted.sh` | 18 760 / 18 762, 11 166 saltados, 978 s (load 37 → 76). Os 2 vermelhos: `ph2d-host-desktop::it::the_pointer_clock_starts_where_the_paint_starts` (os 2 testes) — **regressão da W11**, não flake (lêem o fonte): a W11 fez do `deliver_canvas_pointer` um embrulho do `deliver_canvas_pointer_colordrop` e o gate lia o embrulho. Curado (o gate lê a irmã) e re-corrido por nome: ✓ 2/2 |
| Prova de mutação | 7/10 (§2-bis) |

`agent-loop-profile.sh`:
```
  ✗ paralelismo de ferramenta              1.21/passo   alvo: >= 1,5  (13% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                123   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                 266 : 98   alvo: <= 1,0  razao 2.7x (baseline: 4,3x)
  ✗ edicoes SEM conferir que casaram             2.0%   alvo: <= 2%  (16 de 782; o guarda R6 recusa-as desde 14/10)
  ✗ contexto relido por passo (media)         285 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               65 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

Smoke compilado (2.ª corrida, depois de `rm -rf target/*/incremental`): «Finished `smoke` profile [optimized] target(s) in 0.40s», zero `Compiling`.

## §7 — Smoke do dono e fotos

Binário: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=cruzamolhada ./target/smoke/ph2d-host-desktop`
2. A cena corre sozinha (15 s, não clique): Gravar, duas pinceladas de Wet Paint, Parar, uma vertical por cima das
   duas, ⏮ e ▷ na aba Keys da Timeline.
3. Certo: ao tocar, a vertical aparece inteira desde o início e limpa; as duas gravadas nascem depois, por baixo dela.
4. Errado: onde a vertical cruza uma gravada que ainda não nasceu, um pedaço escuro dela (o «nó»), ou uma falha branca
   na vertical.

Fotos (`smoke_2026-10-20_W12/`, 1930×1040, rajada de 1 s; ⏮ 6,92 s, ▷ 7,40 s no `cruzamolhada.log`):
[▷ 0,87 s: só a vertical, sem nós](smoke_2026-10-20_W12/cruzamolhada_02.png) ·
[▷ 1,98 s: a de cima nascida, o cruzamento de baixo limpo](smoke_2026-10-20_W12/cruzamolhada_03.png) ·
[▷ ≈3 s: a de baixo a nascer](smoke_2026-10-20_W12/cruzamolhada_04.png) ·
[▷ 4,15 s: as três](smoke_2026-10-20_W12/cruzamolhada_05.png). Smoke do dono depois da W12: **OK (20/10)** — «smoke parece ok».

## §8 — ABERTO

1. **A água da gravação pára no 1.º toque do artista** (decisão do dono): se ele pinta enquanto a gravada ainda
   escorre, o documento fica com ela parada naquele instante; o Play mostra a gravação até ao fim dela.
2. As condições do «exacto» (alfa preso, vidro de aguada, pilha de camadas, papel que atravessa vidro, outro modo de
   mistura) não têm teste próprio — fora delas o texel pintado fica cheio (mutação M10 do §2-bis).
3. Um passo que mude a tela FORA do composite dentro da sessão do artista compõe-se também no nascimento — nenhum
   fluxo testado o faz (mutação M9).
4. Seguem de pé o §8 da [W11](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W11.md) (o tutorial «fazer» salta o Fill) e
   os itens 2–7 do [§8 da W10](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W10.md) (o item 1 da W10 — o Wet Paint — foi
   feito aqui; Watercolor/Impasto ficam cheios por medição).

## Para o integrador (`CLAUDE.md` §5.1, entrada «Macros & Automação»)

Novo «Último» = este handoff (W8–W11 continuam a valer para a parte deles). Frase da natureza: «a marca do artista é
cobertura (…)» → acrescentar «; no Wet Paint a tinta pintada depois de gravar fica por cima» só se couber nos 700
bytes (gate `architecture_claude_md_cabe_no_orcamento`).

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
