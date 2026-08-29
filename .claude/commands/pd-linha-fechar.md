---
description: Gate batched 1× sobre o diff acumulado, depois o handoff §1.5.9.
argument-hint: [Nome da linha]
---
Feche a linha `$1`.

1. Gate batched, 1× sobre o diff acumulado (nunca por task):
   - `bash scripts/nextest-impacted.sh`
   - clippy `--all-targets` + features
   - `shells/desktop/tests/file_loc_caps.rs` (o gate da shell; o workspace_file_loc_cap
     cobre só crates/ — essa lacuna já deixou vermelho latente 2×)
   - `arch_safe_clamp_only` e os arch-gates de shell (só correm na varredura impactada)
   - auditoria com ≥2 lentes
2. Corrija TODO ✗. Não escreva o handoff sobre árvore vermelha.
3. Escreva o handoff de integração (DIRETRIZ §1.5.9) em docs/, incluindo:
   - o que mudou em foundational e por que é aditivo
   - contratos/schemas tocados (ou a prova por grep de que NÃO foram)
   - números MEDIDOS, não estimados
   - os smokes com o comando exato de rodar
4. A narrativa da jornada vai no HANDOFF. No `CLAUDE.md §5` você edita **UMA LINHA**
   (o que está ABERTO / o smoke novo) — nunca acrescenta um parágrafo. Foi o append
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
6. **Deixe o smoke COMPILADO — último passo, depois do commit final** (DIRETRIZ §1.5.9 item 9).
   Dentro da SUA worktree, o binário do comando exato que você vai entregar:
   `cargo build -p ph2d-host-desktop --release` (+ as `--features` de cada smoke que as exija).
   Rode 2× e cole a 2ª saída no handoff — *Finished* em segundos e **zero** linhas `Compiling`
   é a prova. Nada do seu dia produz esse binário (`check` não gera código; o gate é perfil
   `ci-test`, outro target/), e o Enio não espera build. Uma env `PH2D_*` NÃO é outro build;
   feature, perfil e árvore do `cd` são.
7. PARE. Não integre, não pushe.
