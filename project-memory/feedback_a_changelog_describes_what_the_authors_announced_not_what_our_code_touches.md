---
name: feedback_a_changelog_describes_what_the_authors_announced_not_what_our_code_touches
description: Um plano de migração escrito a partir de changelogs erra de quatro formas estáveis — e a quebra que mais custa é a que ninguém anuncia, porque não é da biblioteca, é do encontro dela com o nosso código
metadata:
  type: feedback
---

**Um plano escrito a partir de changelogs descreve o que os autores acharam digno de
anunciar — não o que o NOSSO código encosta.** Medido em 2026-08-29 sobre uma subida de
13 dependências, as quatro classes de erro são estáveis:

| classe | exemplo medido |
|---|---|
| *(a)* mudança anunciada que **não** foi publicada | `VertexState::buffers` viraria `Option` — a linha é **byte a byte igual** à anterior; **34 edições pedidas, zero feitas** |
| *(b)* regra antiga anunciada como nova | `@interpolate(flat)` em varyings inteiros — a regra **já existia** na versão anterior |
| *(c)* mudança real cujo alcance no nosso código é **zero** | 9 das 14 tarefas de um bloco inteiro |
| *(d)* ⭐ **a quebra que ninguém anuncia** | 5 quebras de texto, 18 de física, e a causa do `linesweeper` — não são da biblioteca, são do **encontro dela com o nosso código** |

⛔⛔ **E um plano assim manda o dono do produto procurar o que NÃO PODE ACONTECER.** Um
item mandava o Enio verificar se o gradiente do selector de cor mudara de espaço de
mistura; lido no gerador de rampa, o modo novo é **opcional**, nenhum sítio nosso o
escreve, e o caminho por omissão é a mesma chamada. *Gastar a atenção do dono no que não
pode acontecer ensina-o a não confiar na lista.*

**Why:** o custo de um bloco não se lê no changelog. Lê-se no **diff da superfície
pública contra os nossos greps** — e é por isso que a quebra classe *(d)*, invisível em
qualquer changelog, foi a que consumiu a jornada.

**How to apply:** para cada item do plano, **antes** de editar, grepe o nosso código
pelo símbolo e leia a assinatura nova no registo local
(`~/.cargo/registry/src/*/<crate>-<ver>/`). Um item cujo grep dá zero é um item que não
existe. E o que o compilador apanha é **seguro**; o perigo mora no que compila e muda de
sentido — ver
[[feedback_a_type_that_forces_a_decision_today_pays_when_the_platform_stops_forcing_it]].
