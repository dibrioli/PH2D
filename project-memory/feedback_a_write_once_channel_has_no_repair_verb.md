---
name: feedback_a_write_once_channel_has_no_repair_verb
description: "Se um verbo ESCREVE um canal e nenhum o EDITA, o defeito nele é imune — imunidade é gap de arquitetura"
metadata:
  node_type: memory
  type: feedback
---

O Inflate ficou o **único** verbo que escreve `covers`/`mats`/`rgba` (quando virou o verbo que MOVE
MATÉRIA, 2026-07-14). Os outros — inclusive o Smooth — escrevem `h` e **só** `h` (§5 do plano 18). Aí ele
criou uma borda de cobertura serrilhada, e o Enio: *"essa irregularidade externa é imune ao filtro global e
ao pincel smooth, **nada** pode corrigi-la"*. Não era Smooth fraco: era **estrutural** — a luz pesa por
cobertura, o rasgo estava na cobertura, e nenhum verbo alcança a cobertura. Write-once, pra sempre.

**Why:** a lei *"o sculpt escreve h e só h"* era verdadeira e virou falsa quando UM verbo ganhou um 2º
canal — e ninguém percebeu que ela agora tem uma **assimetria**: existe `moves_matter()` (quem ESCREVE) e
não existe o simétrico (quem CONSERTA). Um canal com produtor e sem editor não tem conserto possível *por
construção*, e o sintoma se disfarça de "o filtro é fraco". Primo de
[[feedback_a_condition_that_enumerates_its_readers_rots]] e de
[[feedback_stale_comment_and_dead_code_lie]]: a lei documentada continuava lá, verdadeira para 7 verbos e
mentindo sobre o 8º.

**How to apply:** ao dar a um verbo/efeito um **canal novo** (um 2º buffer, um plano a mais), pergunte na
mesma sessão: *"quem EDITA isto depois?"*. Se a resposta é "ninguém", você acabou de criar um estado
write-once — nomeie no plano ou dê o verbo simétrico. **"É imune / nada corrige"** num relato de smoke é
quase sempre isto, não força insuficiente: procure o canal que o efeito visível lê
([[feedback_growing_geometry_without_growing_matter_grows_nothing]] — a luz MODULA o RGBA, ela não o
inventa) e veja **quem consegue escrevê-lo**. Corolário do mesmo bug: os dois canais têm de concordar sobre
o mesmo fato — a altura desvanecia por um taper C¹ e a cobertura era copiada CHEIA, então as duas
discordavam sobre onde a forma termina, e a luz acredita na cobertura
([[feedback_two_doors_to_the_same_question_diverge]], [[reference-topic-impasto-physics]]).
