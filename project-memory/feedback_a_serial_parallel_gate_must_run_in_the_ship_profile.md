---
name: feedback_a_serial_parallel_gate_must_run_in_the_ship_profile
description: Um portão de identidade (série × paralelo, ao byte) verde em dev não prova o perfil do ship — o LLVM 21–22 vectoriza mal `tile[base + ((x−1) & mask)]` com overflow-checks a o2
metadata:
  type: feedback
---

O `paper_rows` (bake do papel do Wet Paint) passava 2 400/2 400 em `dev` e reprovava 100/100 no `ci-test`
(o perfil do `ship.sh` e do `nextest-impacted`): 7 403 células com o texel da linha SEGUINTE do ladrilho.
Não era corrida nem carga — era código mau do LLVM 21.1.8–22.1.8 (rustc 1.92–1.98; o nightly com LLVM 23.1.1
acerta). Forma: um laço que só escreve, com o índice `base + ((x as i32 − 1) & 511)`, a `opt-level ≥ 2` **com
`overflow-checks`**: o vectorizador copia 8 texels contíguos e a verificação em tempo de execução testa o
overflow de `base + 511 + x` em 32 bits SEM a máscara — os 7 texels depois de cada volta saem errados. A
partição em CGUs / `lto` / o inlining só decidem se o `y` chega opaco ao laço; um kernel `#[inline(never)]`
erra também no `dev` a o2. Reprodutor de 40 linhas em
`docs/Painter/NewFeaturesWetPaint/ferramentas/reprodutor_llvm_volta_mascarada.rs` (rustc puro).

**Why:** o perfil muda o programa; um portão ao byte só vale no perfil em que o portão do ship corre. E um
portão com oráculo independente (o laço de antes congelado) foi o que disse ONDE — as duas rotas partilham o
corpo, compará-las entre si não apanha um corpo mal compilado.

**How to apply:** (1) portões série × paralelo e ao byte correm também `--profile ci-test` antes do fecho
(o `nextest-impacted` já o faz — leia a falha dele como sinal, não como flake); (2) não escreva cópia com
índice mascarado por célula — copie os troços (`copy_from_slice` entre voltas); (3) censo do código gerado:
`cargo rustc --lib --profile ci-test -- --emit=llvm-ir` e procure `and … 511` dentro de um `vector.body`
(controlo: o reprodutor acusa). (4) ⛔ baixar a o1 o `ci-test` é silenciar, não curar. Ver
[[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]].
