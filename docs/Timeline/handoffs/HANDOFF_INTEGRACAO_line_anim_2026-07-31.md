# HANDOFF DE INTEGRAÇÃO — `line/anim` → `main` (2026-07-31)

**Status:** FECHADO 2026-07-31 · no `main` em `7862b6ddd` (o commit que trouxe este arquivo).

> **Para o agente integrador.** A linha está **FECHADA**: árvore limpa, 88 commits, todos os
> smokes aprovados pelo Enio. Ela **não** foi pushada e **não** foi integrada.
> Leia §1 e §2 antes de qualquer comando: os dois pontos que exigem decisão humana estão lá.

---

## §0 — Cartão de identidade

| Fato | Valor **medido** |
|---|---|
| Branch | `line/anim` · worktree `Worktrees/line-anim` |
| Tip | `a1a6976bc` |
| Commits à frente do `main` | **88** |
| `main` andou desde o fork | **329 commits** (base `7ec917506`) |
| Diff cumulativo | 119 arquivos, +13 362 / −1 857 |
| **`Cargo.toml` tocados** | **ZERO** — nenhuma dep nova, nenhuma crate nova |
| Contratos congelados (§6) | **INTACTOS** — `architecture_contract_surface` 3/3 e `architecture_tool_contract_surface` 4/4 verdes, e o diff não toca `ph2d-nodegraph` nem `editor-core/src/tool.rs` |
| Gates de GPU | **nenhum** (a linha não toca `*gpu*` nem `*.wgsl`) |
| Workspace | `cargo test --workspace` **exit 0** · `cargo fmt --all -- --check` **0** · clippy **0** |

---

## §1 — ⚠️ DECISÃO 1: OS DOIS ADRs COLIDEM E TÊM DE RENUMERAR

A linha traz dois ADRs **cujos números já foram levados no `main`**:

| Na linha | No `main` de hoje | Renumerar para |
|---|---|---|
| `0145-timeline-expressions-are-per-clip-so-a-strip-windows-them.md` | `0145-wet-paint-solver-row-parallel-passes-rayon-exception.md` (Painter) | **0151** |
| `0146-timeline-expressions-are-a-first-class-lane-source-that-fades.md` | `0146-wet-paint-gpu-solver-is-a-second-model-not-a-faster-one.md` (Painter) | **0152** |

O maior ADR no `main` é **0150** (`3d-sculpt`), então os próximos livres são **0151** e **0152**.
É a **6ª e 7ª vez** que isto acontece no repo, sempre pelo mesmo mecanismo: *um número
escolhido numa linha paralela é PROVISÓRIO*, e como os NOMES de arquivo diferem **o git nunca
conflita** — quem pega é o gate `architecture_adr_numbers_are_unique`.

⚠️ **O rewrite do token é ESCOPADO AOS ARQUIVOS QUE A LINHA MUDOU**, nunca à árvore
([[feedback_a_token_rewrite_scopes_to_the_changed_files_not_the_whole_tree]]): há dezenas de
citações a `ADR-0145`/`ADR-0146` no `main` que falam do **wet paint**, e um `git grep -l | xargs sed`
as corromperia. Receita:

```bash
# 1. os arquivos QUE A LINHA TOCA (e só eles)
git diff --name-only main...HEAD > /tmp/line_files.txt
# 2. renomeia os dois ADRs
git mv docs/architecture/decisions/0145-timeline-expressions-are-per-clip-so-a-strip-windows-them.md \
       docs/architecture/decisions/0151-timeline-expressions-are-per-clip-so-a-strip-windows-them.md
git mv docs/architecture/decisions/0146-timeline-expressions-are-a-first-class-lane-source-that-fades.md \
       docs/architecture/decisions/0152-timeline-expressions-are-a-first-class-lane-source-that-fades.md
# 3. reescreve o TOKEN só nesses arquivos (ADR-0145 -> ADR-0151, ADR-0146 -> ADR-0152)
# 4. CONTROLE: confira que nenhum arquivo do wet paint entrou na lista
```

**Verificação final:** `cargo test -p ph2d-editor-core architecture_adr_numbers_are_unique`.

---

## §2 — ⚠️ DECISÃO 2: OS NÚMEROS DE SCHEMA

**Medido, os dois lados:**

| Constante | `main` | `line/anim` | Resolução |
|---|---|---|---|
| `DOC_VERSION` (`ph2d-timeline`) | **15** | **17** | **17** — a linha bumpa 2× e ninguém mais toca este arquivo |
| `PROJECT_SCHEMA` (`shells/desktop/src/project.rs`) | **46** | 37 (= o valor do fork) | **46** — a linha **NÃO** bumpa; o git leva o de `main` |
| tripla `(PROJECT, FLIP, VEC_SCENE)` | `(46, 12, 13)` | *arquivo não existe na linha* | fica `(46, 12, 13)` |

⚠️ **Não há colisão de schema nesta integração, e vale entender POR QUÊ** — para não
"consertar" o que está certo: o `TimelineDoc` viaja como **blob DENTRO do `ProjectFile`** e
carrega a própria versão, então a forma do `ProjectFile` não muda e o `PROJECT_SCHEMA` não
acompanha. O `project_schema_tests.rs` (a tripla) **nasceu no `main` depois do fork** e a linha
nunca o viu ⇒ chega limpo.

⚠️ **`DOC_VERSION` 15→17 é QUEBRA DURA** (postcard é posicional): dois campos apendados —
**16** = a expressão por-CLIP (`NamedClip.expr`), **17** = `NamedClip.paths` (a trajetória é do
clip). Projetos salvos com v15/v16 têm a **timeline recusada no load** (o resto do projeto
abre). É a política que este documento segue desde o ADR-0133.

---

## §3 — A superfície de conflito: **8 arquivos, e só eles**

Interseção medida entre *o que a linha mudou* e *o que o `main` mudou desde o fork*:

| Arquivo | Linha | `main` | O que a linha fez |
|---|---|---|---|
| `shells/desktop/src/render_loop/mod.rs` | +33/−26 | +574/−48 | K do motion path (o `match path_key_time`), fiação do `keys_mode`, drenos |
| `shells/desktop/src/input_dispatch.rs` | +3/−0 | +473/−16 | 3 argumentos `self.timeline.keys_mode` |
| `shells/desktop/src/main.rs` | +4/−2 | +54/−0 | `mod expr_blend_smoke` + `mod morph_fade_smoke` (troca `mod expr_smoke`) |
| `shells/desktop/src/app_state.rs` | +4/−1 | +47/−0 | 2 latches de smoke (troca `expr_smoke_done`) |
| `shells/desktop/src/project.rs` | +5/−0 | +51/−1 | `expr_owed::forget_owed_poses()` no load |
| `shells/desktop/src/undo.rs` | +1/−0 | +2/−0 | uma linha de comentário |
| `crates/ph2d-i18n/src/lib.rs` | +2/−0 | +40/−0 | 2 linhas de comentário |
| `CLAUDE.md` | +1/−1 | +12/−5 | uma linha da §5 |

**Todos são "os dois lados APENDARAM à mesma lista"** — resolução mecânica, mantendo as duas
metades. ⚠️ Resolva **pelos ESTÁGIOS do índice** (`:1` base · `:2` ours · `:3` theirs), nunca
lendo marcadores ([[feedback_resolve_conflicts_from_index_stages_not_markers]]), e **varra
marcadores em cada commit**.

⚠️ **`CLAUDE.md` §5:** a entrada da Timeline é uma **lista compartilhada** — só ACRESCENTE;
remover linha alheia é integração. A entrada nova desta linha está pronta em §7.

---

## §4 — O que a linha ENTREGA (88 commits, quatro frentes)

### (A) ADR-0145→**0151** — a expressão é POR-CLIP, então um strip a janela

`DOC_VERSION` 15→16. Uma fórmula deixa de ser um fato global da prop e passa a viver no
clip: a strip que toca o clip **janela** a expressão, como janela as keys.

### (B) ADR-0146→**0152** — a expressão é uma FONTE DE LANE que FADEIA (W0–W7)

A entrega grande da linha. Uma expressão per-clip entra na composição como qualquer outra
fonte e **desvanece com o strip** — antes ela era um passe pós-composição que ignorava o
fade. Inclui: `LinkFrame` (prop-links lendo a fonte já **fadeada**, em ordem topológica) ·
a separação limpa *global-transform × per-clip-source* · **keying coerente através da
expressão** (probe-through-expr: o K inverte a fórmula afim e **recusa** a não-inversível) ·
o `expr_owed` (o livro-razão das poses devidas — sem ele `value + 250` fica em 250 para
sempre depois de um DELETE) · e o corpus de medição.
⚠️ **`fade_fingerprint`/`fade_fingerprint_channels`** pinam o fade num hash literal e estão
**verdes** — é a prova executável de que o blend não regrediu.

### (C) A AUTORIA de expressões foi CONSTRUÍDA e RETIRADA (o motor ficou)

O Expression Editor (catálogo de 55→31 receitas, card, planilha, preview vivo, FASES
0/A/B/C/E) foi construído, auditado e **reprovado no smoke** (*"todos aparecem como
custom"* — a folha é write-once). **Retirado inteiro por ordem do Enio**: 11 125 linhas, a
crate `ph2d-expr-recipes`, o card, 6 ids, 7 chaves i18n, 2 `TimelineHitKind`, o preview e 2
smokes. Registro em [`docs/Timeline/14_a_autoria_de_expressoes_foi_retirada.md`](../14_a_autoria_de_expressoes_foi_retirada.md).
⚠️ **Remover a FEATURE não é remover o SCHEMA:** `TargetBinding.expr` (v15) e `NamedClip.expr`
(v16) **continuam serializados** — apagá-los recusaria todo projeto salvo.
⚠️ **É por isso que o `Cargo.toml` é ZERO:** a crate nasceu e morreu dentro da linha.

### (D) A cauda de duração/escopo + o MOTION PATH (os 6 últimos commits)

Duração 0 = infinito · projeto legado abre com 4 s + véu · **Arrange é escopo
INDEPENDENTE** · o canal de **Morph** é autorável · o `lead_out` do ÚLTIMO strip de uma
lane (era inerte) · e a cadeia do motion path, que é **uma pergunta só** —
*a que clip esta geometria pertence, e quem está olhando para ele?*:

1. **a trajetória é do CLIP** (clip novo nasce em branco — matou a alça fantasma);
2. **no Arrange quem manda é o STRIP que dirige**, não o clip aberto no Keys;
3. **o fade compõe PONTOS** (misturar duas réguas de distância não significa nada);
4. **o autokey não planta âncora sobre a pose que o apply escreveu** (a "curva de
   transição" que deformava os dois paths — 32 âncoras num único fade, medidas);
5. **a trajetória só é OFERECIDA na aba Keys** (desenho, âncoras, alças, hit, hit-de-curva);
6. **o K e o AutoKey não ANCORAM fora dela**, com motivo dito em voz alta
   (`KeyRefusal::PathNeedsKeysTab`).

Detalhe por defeito, com mecanismo e mutação: [`docs/Timeline/BUGS_timeline.md`](../BUGS_timeline.md) §1, §2a–§2g.

---

## §5 — Gate de fechamento (rode NA ÁRVORE COMBINADA, não só na linha)

```bash
cd <worktree-de-integracao>
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace                    # DEBUG e RELEASE: um deles já escondeu pânico neste repo
cargo test -p ph2d-editor-core --test architecture_adr_numbers_are_unique   # §1
cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap
cargo test -p ph2d-host-desktop --test file_loc_caps                        # o cap do SHELL é outro gate
cargo test -p ph2d-timeline --test fade_fingerprint --test fade_fingerprint_channels
```

⚠️ **Os dois gates de LOC são DOIS** — o `architecture_workspace_file_loc_cap` só varre
`crates/`, e um `cargo test -p` filtrado não alcança nenhum dos dois. Esta linha já pagou
três splits por causa deles (`autokey.rs` 698→460, `motion_path_overlay_tests.rs`,
`tracks.rs` do painel).

⚠️ **A falha EMERGENTE a esperar:** um arquivo que **nenhuma** das duas linhas cruzou
sozinha pode cruzar o cap na soma (aconteceu com `keyboard.rs` na integração de 27/07). O
corte é por **RESPONSABILIDADE**, nunca por tamanho.

⚠️ **Flake conhecida, PRÉ-EXISTENTE:** `the_cost_of_depth_is_linear_not_explosive`
(`ph2d-timeline/tests/nesting_clock.rs`) é gate de RAZÃO sensível a carga. Medido isolado
agora: **6/6 verde**. Re-rode sozinho antes de suspeitar do merge.

---

## §6 — Smokes (o que o Enio já aprovou, e o que um re-smoke deve olhar)

| Cena | Comando | O que julgar |
|---|---|---|
| **Expressões no blend** | `env PH2D_EXPR_BLEND_SMOKE=1 cargo run -p ph2d-host-desktop --release` | a fórmula desvanece **com o strip** |
| **Morph no fade** | `env PH2D_MORPH_FADE_SMOKE=1 …` | o morph não estala na costura |
| **Motion path** | `env PH2D_MOTION_PATH_SMOKE=1 …` | **(1)** clip novo abre **em branco** · **(2)** em Arrange cada strip anda na curva do PRÓPRIO clip · **(3)** strips SOBREPOSTAS com paths diferentes: o objeto atravessa liso e **nenhum path desenhado se move um pixel** · **(4)** as alças **não existem** em Arrange nem dentro de container · **(5)** ali, arrastar com AutoKey ou apertar **K** não planta nada **e avisa** |
| Nesting / onion / joias | `PH2D_NEST_SMOKE=1|2|3`, `PH2D_ONION_SMOKE=1`, `PH2D_TIMESCALE_SMOKE=1`, `PH2D_STAGGER_SMOKE=1` (⚠️ **Ctrl**+drag, o KDE rouba o Alt), `PH2D_BUFFER_SMOKE=1`, `PH2D_EXTRAP_SMOKE=1`, `PH2D_SIGNAL_SMOKE=1` | regressão das waves anteriores |

⚠️ **`PH2D_EXPR_SMOKE` MORREU** com o card. Quem procurar a cena do motor procura o
`PH2D_EXPR_BLEND_SMOKE`.

---

## §7 — Linha pronta para a §5 do `CLAUDE.md` (APENDAR na entrada **Timeline**)

> **⬛ A EXPRESSÃO FADEIA E A TRAJETÓRIA É DO CLIP — `line/anim` INTEGROU (2026-07-31, 88
> commits, todos os smokes aprovados; ADR-0151/0152, handoff
> [`docs/Timeline/handoffs/HANDOFF_INTEGRACAO_line_anim_2026-07-31.md`](HANDOFF_INTEGRACAO_line_anim_2026-07-31.md)):**
> quatro frentes. **(A) ADR-0151:** a expressão é **por-CLIP**, então um strip a **janela**
> (`DOC_VERSION` 15→16). **(B) ADR-0152:** ela vira **fonte de lane que FADEIA** — antes era
> um passe pós-composição cego ao fade —, com `LinkFrame` (prop-links lendo a fonte já
> fadeada, em ordem topológica), keying coerente **através** da fórmula (o K inverte a afim e
> **recusa** a não-inversível) e o `expr_owed`, o livro-razão sem o qual `value + 250` fica em
> 250 para sempre depois de um DELETE. ⚠️ Os `fade_fingerprint` seguem **verdes no mesmo
> hash**: é a prova executável de que o blend não regrediu. **(C) A AUTORIA de expressões foi
> construída e RETIRADA** (11 125 linhas, a crate `ph2d-expr-recipes` inteira, o card, o
> preview e 2 smokes) — o smoke reprovou (*"todos aparecem como custom"*: a folha é
> write-once) e o **MOTOR ficou intacto**; ⚠️ **remover a feature não é remover o SCHEMA**
> (`TargetBinding.expr`/`NamedClip.expr` continuam serializados — apagá-los recusaria todo
> projeto salvo), e é por isso que a linha inteira tem **ZERO `Cargo.toml` tocado**. **(D) O
> MOTION PATH**, seis defeitos que são **uma pergunta só** (*a que clip esta geometria
> pertence, e quem está olhando para ele?*): a trajetória é do **CLIP** (clip novo nasce em
> branco — morre a alça fantasma) · no Arrange quem manda é o **STRIP que dirige** · o fade
> **compõe PONTOS** (misturar duas réguas de distância não significa nada) · o autokey **não
> planta âncora sobre a pose que o apply escreveu** (32 âncoras num único fade, medidas — a
> *"curva de transição"* que deformava os dois paths) · a trajetória só é **OFERECIDA** na aba
> Keys (as CINCO portas pela porta única `active_path`) · e o K/AutoKey **não ANCORAM** fora
> dela, com motivo (`KeyRefusal::PathNeedsKeysTab`) — ⚠️ recusar em silêncio transformaria o
> gesto numa ferramenta quebrada, então a recusa é um **VALOR**, e ela é de **ESCOPO** (*um
> clip que você não está olhando*), não de expressão, por isso campo próprio `path_refused` e
> não um `PropKind` na lista `refused`. ⚠️ **Um gate de unidade é CEGO à fiação do shell**, e
> isto está medido: com o `draw` em `true` literal os **20 testes do overlay ficam VERDES** e
> só o arch-gate sangra. Mais: duração 0 = **infinito**, projeto legado abre com 4 s + véu,
> **Arrange é escopo INDEPENDENTE**, o canal de **Morph** é autorável e o `lead_out` do ÚLTIMO
> strip de uma lane (era inerte). **`DOC_VERSION` 15→17** (dois campos apendados, quebra dura:
> v15/v16 têm a timeline recusada no load) · **`PROJECT_SCHEMA` INTOCADO** (o `TimelineDoc`
> viaja como blob e carrega a própria versão) · contrato congelado **intacto** · **nenhuma dep
> nova**. ⚠️ **Os ADRs nasceram 0145/0146 e RENUMERARAM na integração** (6ª e 7ª vez): a
> `line/Painter` levou os dois números na mesma janela, e como os NOMES de arquivo diferem o
> git **nunca conflitou**. Smokes: **`PH2D_EXPR_BLEND_SMOKE=1`** · **`PH2D_MORPH_FADE_SMOKE=1`**
> · **`PH2D_MOTION_PATH_SMOKE=1`**. Bugs com mecanismo e mutação:
> [`docs/Timeline/BUGS_timeline.md`](../BUGS_timeline.md) §1, §2a–§2g.

---

## §8 — Aberto (nomeado, NÃO construído — nada disto bloqueia a integração)

1. **A expressão PURA (sem keys) extrapola a strip.** Ela não tem track, logo nenhuma strip a
   referencia, logo não há janela a obedecer. A metade *per-clip* fechou (ADR-0151); ligar a
   **global** exige vínculo explícito — decisão de produto + provavelmente `DOC_VERSION`.
2. **O catálogo de receitas está morto, a pesquisa não.** Os docs 09–13 da pasta Timeline
   viram históricos; o que eles MEDEM sobre o catálogo segue válido se a autoria for
   reprojetada (a lição central: *uma folha write-once faz toda fórmula reaberta voltar como
   `Custom Formula`*).
3. **W4.T4 — o dock da timeline dentro do Motion.** A contradição entre `line/anim`
   (rejeitou) e `line/motion-value` (construiu com cap de 0,45) continua **aguardando
   re-smoke**; esta linha não tocou nisso.

---

## §9 — Protocolo

O agente integrador **integra**; o **ship + push são do Enio** e só por ordem explícita
(CLAUDE.md §0.7). Se o gate da árvore combinada acusar algo, a regra é a de sempre: **o
número se CONTA a partir do `main` do dia, não se escolhe**
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
