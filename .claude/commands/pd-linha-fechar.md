---
description: Gate batched 1× sobre o diff acumulado, depois o handoff §1.5.9.
argument-hint: [Nome da linha]
---
Feche a linha `$1`.

1. Gate batched, 1× sobre o diff acumulado (nunca por task):
   - `BASE=$(git merge-base main HEAD) bash scripts/nextest-impacted.sh` — a régua é o merge-base;
     o default `origin/main` mede também o que ainda não foi enviado. Ele já força os gates
     GLOBAIS (tectos de ficheiro, o tecto da shell, o censo de órfãos)
   - `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` — um `dead_code` que só a
     workspace inteira vê não aparece num `check -p`
   - `bash scripts/check-standalone-optional.sh` + `bash scripts/check-workflow-packages.sh` —
     crate opcional que não compila sozinha · pacote apagado/renomeado que um workflow cita por `-p`
   - moveu código da shell? `git grep -n 'shells/desktop/src' -- crates tools`: cada leitor por
     caminho é gate seu, e corre no fecho
   - clippy `--all-targets` + features
   - `cargo machete` — ⚠️ **obrigatório se a linha MOVEU código**: mudar código de casa não muda a linha do `Cargo.toml`, nada disso falha a compilar, e o CI reprova. Medido na W2 (12/09): **79** dependências mortas deixadas pelas mudanças de casa, que nenhuma linha correu ao fechar (HOWTO §2.18)
   - `shells/desktop/tests/it/file_loc_caps.rs` (o gate da shell; o workspace_file_loc_cap
     cobre só crates/ — essa lacuna já deixou vermelho latente 2×)
   - `arch_safe_clamp_only` e os arch-gates de shell (só correm na varredura impactada)
   - auditoria com ≥2 lentes
2. Corrija TODO ✗. Não escreva o handoff sobre árvore vermelha.
2-bis. ⭐⭐⭐ **O que só a ÁRVORE COMBINADA pega — e é VOCÊ quem o cura** (DIRETRIZ §1.5.9 item 5-bis):
   ```bash
   git cherry main HEAD && git rebase main       # o rebase que o §1.5.2 item 3 já manda antes de integrar
   bash scripts/censos-da-arvore-combinada.sh    # ~2 min com a árvore quente
   ```
   ⚠️ **Duas famílias que nenhuma linha vê sozinha, por construção:** o **censo de texto (HR-15)** é
   escrito por uma linha e o literal que o acorda por outra (nenhuma das árvores tem as duas coisas,
   e as duas fecham verdes); e o **tecto de LOC SOMA entre linhas sem ninguém a contar**. ⛔ O CI não
   os corre. Medido em 17/09: **7 das 8** falhas da rodada eram destas duas — todas descobertas pelo
   integrador, em série, uma linha de cada vez.
   ⚠️ **A cura é sua porque só você sabe se aquele texto chega ao ecrã** (⇒ chave em `ph2d-i18n`) ou
   se é formato/terminal/nome de fixtura (⇒ isenção NOMEADA, **com o mecanismo**). ⚠️ Isenção ÓRFÃ
   **e** literal sem abrigo na mesma corrida = o ficheiro MUDOU DE SÍTIO, e a isenção viaja com ele.
3. Escreva o handoff de integração (DIRETRIZ §1.5.9) em `docs/<Módulo>/handoffs/`, incluindo:
   - o que mudou em foundational e por que é aditivo
   - contratos/schemas tocados (ou a prova por grep de que NÃO foram)
   - números MEDIDOS, não estimados
   - os smokes com o comando exato de rodar
4. A narrativa da jornada vai no HANDOFF. **A linha NÃO edita o `CLAUDE.md §5`** (DIRETRIZ §1.5.5:
   só na integração, no primário — duas linhas a editá-lo nas worktrees dão conflito no ficheiro
   que todo agente carrega): escreva no handoff a **UMA LINHA** que propõe para ele (o que está
   ABERTO / o smoke novo); quem a aplica é o integrador. Nunca um parágrafo. Foi o append
   por-jornada que levou o §5 a 868 KB, injetados em todo agente e toda worktree,
   antes da primeira palavra do Enio (DIRETRIZ §1.5.9 item 8).
5. **`wc -c` no tracker/handoff do módulo. Passou de ~100 KB? CORTE-O agora:**
   `python3 scripts/doc-split.py <doc> --keep <faixas> --archive docs/archive/docs-<data>/<mod>/<doc>`
   depois `python3 scripts/archive-index.py docs/archive/docs-<data>`.
   ⚠️ Mandar a narrativa para o handoff **realocou** a doença: o tracker da física chegou a
   **710 KB** — 77% do que o §5 chegou a ser — com **1 `Read` para 407 comandos de shell** e
   **89% dele nunca lido**. O joelho medido é **80–110 KB**.
   ⛔ **E indexe as recusas:** arquivar um `⛔ MEDIDO E REJEITADO` sem o índice no doc vivo é
   apagá-lo (o log de perf do Painter guardava 47; o §5 citava cinco). As mais duras são
   **títulos de seção** — um extrator que só lê o corpo perde 31 de 126.
5b. **Instrumento que o handoff cita vive VERSIONADO** (`scripts/` ou `docs/<Módulo>/ferramentas/`),
   nunca numa pasta não rastreada da worktree — ela morre com a worktree. Depois reclame o
   incremental: `rm -rf "$(git rev-parse --show-toplevel)"/target/*/incremental` (DIRETRIZ §1.5.9
   item 7) — ANTES do passo 6, porque o perfil `smoke` também é incremental.
6. **Deixe o smoke COMPILADO — último passo, depois do commit final** (DIRETRIZ §1.5.9 item 9).
   Dentro da SUA worktree, o binário do comando exato que você vai entregar:
   `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` (+ as `--features` de cada smoke que as exija;
   `--release` só para smoke de PERFORMANCE — o `smoke` reconstrói em 3 s, o `release` em 161 s).
   Rode 2× e cole a 2ª saída no handoff — *Finished* em segundos e **zero** linhas `Compiling`
   é a prova. Nada do seu dia produz esse binário (`check` não gera código; o gate é perfil
   `ci-test`, outro target/), e o Enio não espera build. Uma env `PH2D_*` NÃO é outro build;
   feature, perfil e árvore do `cd` são.
7. PARE. Não integre, não pushe.
