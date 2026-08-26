---
name: feedback-a-ruler-that-crosses-another-crates-panic-measures-the-crossing
description: Um gate que prova a lei da 1ª fase pela saída da travessia inteira reprova por um defeito a jusante que não é o dele.
metadata:
  type: feedback
---

Escrevi o gate «a fase zero remalha para o alvo que lhe deram» medindo os **quads finais** da
cadeia com dois alvos. Ele reprovou — mas não pela lei: com o alvo grosso o `ph2d-gridmap`
entrou em `panic!` (`solve.rs:336`, *"index out of bounds: the len is 74 but the index is 130"*),
defeito de outra linha, sobre uma malha perfeitamente válida.

**Why:** *uma régua que atravessa um estouro de outra crate não mede a lei que se quer — mede a
travessia inteira.* E o vermelho aponta para o meu commit, que não lhe tocou.

**How to apply:** gate a propriedade **onde ela é definida**. A cura foi extrair a fase zero para
uma função pública (`ph2d_quadchain::phase_zero`) que a cadeia chama, e medir os **triângulos que
ELA produz** — instantâneo, sem atravessar nada. ⚠️ Isso só vale se a função extraída for a que o
produto chama: uma função-irmã «igual» seria uma segunda cópia da lei, que é o defeito que este
repo paga em série. Ver
[[feedback-a-sampled-fixture-proves-what-it-sampled-gate-the-property-where-it-is-defined]] e
[[feedback-a-tree-scanning-gate-is-never-reached-by-a-name-filter]].

⛔ **Corolário sobre a fixtura:** ao escolher a direcção da variação, prefira a que **não** entra
no regime que estoura. Aqui, dobrar o alvo (mais grosso) estoura; dividi-lo custaria ~50 s de
gate. A saída não era nenhuma das duas — era mudar de sítio.
