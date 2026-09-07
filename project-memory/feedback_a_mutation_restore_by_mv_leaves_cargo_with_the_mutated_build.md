---
name: a-mutation-restore-by-mv-leaves-cargo-with-the-mutated-build
description: Restaurar uma prova de mutação com `mv backup file` devolve um mtime ANTIGO, e o cargo mantém o artefacto compilado DA MUTAÇÃO — a corrida seguinte mede o programa errado.
metadata:
  type: feedback
---

A bancada de mutação deste repo é `cp f f.bak` → mutar → testar → `mv f.bak f`. ⚠️ **O `mv`
devolve o mtime do BACKUP**, que é *anterior* ao do ficheiro mutado que o cargo acabou de
compilar. O cargo mede frescura por **mtime**, logo ele conclui que o artefacto é mais novo que a
fonte e **não reconstrói**: a corrida seguinte mede o binário DA MUTAÇÃO.

**Why:** medido em 2026-09-07 (wave 20b da `line/UIUX`). Depois de restaurar o `control_gap_px`,
o gate da escada reprovou a dizer `dois CONTROLOS (8)` quando a fonte diz `3` — o `8` era o valor
da mutação. O sinal secundário é um aviso de `dead_code` sobre uma função que a fonte visivelmente
chama (`widget_margin_y_px` «never used»): *quando o compilador contradiz o ficheiro que está
aberto à frente, suspeite do artefacto, não da leitura*.

⚠️ **Isto esconde-se atrás do `cargo fmt`**: se houver um `fmt` entre a restauração e a corrida, e
ele reescrever aquele ficheiro, o mtime actualiza e o defeito não aparece. Foi por isso que ele
sobreviveu a várias waves sem dar sinal.

**How to apply:** `mv "$f.bak" "$f"; touch "$f"` — o `touch` no fim de toda restauração. O mesmo
vale para qualquer script que reponha um ficheiro a partir de uma cópia (`cp -p` tem a mesma
armadilha, e é pior porque preserva o mtime de propósito). Ver [[feedback-python-replace-silent-noop-after-fmt]].
