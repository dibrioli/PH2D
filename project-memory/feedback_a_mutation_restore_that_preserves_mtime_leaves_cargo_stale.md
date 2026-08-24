---
name: feedback-a-mutation-restore-that-preserves-mtime-leaves-cargo-stale
description: "Restaurar uma mutação com shutil.copy2/move devolve o conteúdo E o mtime antigo, e o cargo serve o binário da MUTAÇÃO sobre o fonte já correto — a suíte fica vermelha acusando código certo"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6a6caccb-4d8f-423e-885d-d18bb2df8b6f
  modified: 2026-08-24T22:01:53.181Z
---

Num arnês de mutação (backup → mutar → testar → restaurar), `shutil.copy2` **preserva o
mtime**, e `shutil.move(bak, p)` devolve o arquivo com o carimbo ORIGINAL — mais velho que o
artefato que o cargo compilou da versão mutada. O cargo compara mtime: julga o binário
atualizado, **não recompila**, e a suíte seguinte reporta o comportamento da MUTAÇÃO sobre um
fonte que já está correto.

Medido em 2026-08-24 (`line/components`, F0): quatro mutações mataram os gates certos; depois
do restore a suíte ficou vermelha dizendo `field_id repetido (2 depois de 2)` enquanto o
arquivo mostrava `f(3, "Scale", …)` — o valor certo. Cura: `find <crate> -name '*.rs' -exec
touch {} +` (ou `shutil.copy` em vez de `copy2`).

**Why:** o modo de falha aponta para o lugar errado. A leitura natural de "suíte vermelha +
fonte correto" é *"o gate está errado"* ou *"eu li mal o fonte"*, e a resposta natural é
mexer no código certo. O defeito não está em nenhum dos dois: está no relógio do arquivo.

**How to apply:** todo arnês de mutação **carimba o restore** (`touch`) e **re-roda a suíte
como último passo**, exigindo verde — o restore só está provado quando o verde volta.
⚠️ E **`git status` não serve de conferência quando a crate é nova**: dentro de um diretório
`??` o git mostra uma linha só para a pasta e não vê arquivo nenhum lá dentro — foi
exatamente o que me deu falso sossego. Confira o **conteúdo**, não o status.
Irmã de [[reference_topic_mutation_proofs]] (os 3 controles no arnês: verde-antes ·
`Compiling <pkg>` · `running 1 test`) e de
[[feedback_python_replace_silent_noop_after_fmt]] (a outra armadilha do script que edita).
