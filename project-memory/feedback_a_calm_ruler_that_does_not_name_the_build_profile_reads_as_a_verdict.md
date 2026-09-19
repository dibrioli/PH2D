---
name: feedback-a-calm-ruler-that-does-not-name-the-build-profile-reads-as-a-verdict
description: A regua de calma imprimia a ociosidade da CPU e nao o perfil de build — a leitura mais CALMA (98% ociosa) foi a pior das quatro, porque era debug, e eu quase persegui uma regressao que nao existia
metadata:
  type: feedback
---

⛔⛔⛔ **Uma régua de calma que imprime a ociosidade da CPU e NÃO o perfil de build entrega um
veredito sobre a máquina quando a variável que decidiu foi o compilador.**

Medido 2026-09-19 (`line/3DModeling`, o fecho da cura do rebordo de 1 pixel). O portão
`com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento` reprovou por **uma** cena (`11 de 22`,
a barra é a maioria estrita). Ele traz no cabeçalho, por escrito, que é da família das flakes de
carga do `CLAUDE.md` §5.0 e **imprime a ociosidade da CPU** para quem o lê poder julgar. Corri-o
sozinho três vezes para confirmar a flake:

| corrida | ociosa | veredito |
|---|---:|---|
| a | `68 %` | `8 de 22` |
| b | `81 %` | `10 de 22` |
| c | **`98 %`** | `9 de 22` |

⭐⭐⭐ **A leitura mais CALMA foi a pior, e isso é o achado:** as três correram em **debug**, porque ao
comando faltava `--release` — e o `cargo test` recompila em silêncio, logo nada na saída dizia que eu
estava a medir outro programa. A corrida em que o portão passava antes desta wave era `--release`.

⚠️ **E a tabela por cena imitava a assinatura da flake sem o ser:** a MESMA cena de `93` instruções
mediu `14 ms` nas três corridas de debug e `58 ms` na de release, enquanto outra de `934` instruções
fez o contrário. *A vítima muda entre corridas* — que é literalmente o discriminador que o §5.0
prescreve — **e a causa não era o recurso partilhado, era o perfil.**

⛔ O que salvou a leitura foi **comparar os cabeçalhos dos logs**, não os números: o log verde dizia
`Finished 'release' profile … Running target/release/deps/…` e os meus diziam `Compiling …`.

**Why:** a régua respondia à pergunta que alguém se lembrou de fazer em 2026-09-15 (*«a máquina está
calma?»*) e não à que domina o número (*«isto é o mesmo programa?»*). Uma régua de contexto que
nomeia um factor e cala outro **maior** é pior que nenhuma: ela é lida como se fosse a lista
completa, e `98 % ociosa` lê-se como *«esta medição é boa»*.

**How to apply:**
- Toda linha de contexto de uma medição de relógio nomeia o **perfil de build** ao lado da carga.
  Neste repo isso é uma porta (`contexto()` em `ph2d-app-field3d/src/preview_device_tests.rs`), lida
  pelos nove sítios que a escreviam à mão — *acrescentar a coluna a um deles deixaria os outros oito
  a dizer menos do que sabem*.
- Antes de acusar o seu diff por um portão de relógio vermelho, **confira o cabeçalho do log verde
  anterior**: perfil, `--test-threads`, filtro. Duas corridas só se comparam se as três colunas
  coincidirem.
- Um veredito de debug sobre um gate de relógio não é um veredito: re-corra em `--release` antes de
  construir hipótese nenhuma.

Irmãos: [[feedback-an-operation-count-is-not-a-profile-and-the-build-profile-decides-the-number]]
(o mesmo factor a esconder o tecto de uma optimização) ·
[[feedback-the-background-load-of-this-workstation-never-falls-below-five]] (a outra metade do
contexto) · [[reference-topic-measurement-discipline]].
