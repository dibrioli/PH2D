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
5. PARE. Não integre, não pushe.
