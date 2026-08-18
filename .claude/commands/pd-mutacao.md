---
description: RED só conta sobre algo visto VERDE antes.
argument-hint: [Alvo]
---
Prove por mutação os gates de $1.

Protocolo (rode SEMPRE por `bash scripts/cargo-test-narrow.sh <crate>` — ele põe o
`check` na frente, e o exit `2` distingue *não compilou* de *o gate sangrou*, que é
a distinção inteira desta prova):
1. Rode e mostre VERDE primeiro (RED sobre nunca-visto-verde não prova nada).
2. Reinstale cada defeito, um por vez, e mostre o gate sangrando com a mensagem.
   ⚠️ Confirme que sangrou **o gate certo, e só ele** — mutação que derruba três
   testes derrubou o build, não provou o gate.
3. `cp <arquivo> /tmp/<x>.bak` ANTES de mutar; desfaça com `cp` do backup — NUNCA
   `git checkout` — e `touch` o arquivo depois (o mtime restaurado faz o cargo
   reusar o mutante). ⚠️ Medido 2026-08-18: **15.084 mutações rodaram SEM backup
   nenhum**, e **7.289 (33%) mutaram sem `assert` de âncora** — um `str.replace()`
   que não casa é no-op **silencioso** e o script imprime sucesso, ou seja, um RED
   que nunca aconteceu passa por prova.
4. Todo SOBREVIVENTE é gate faltando: ou escreva o gate, ou documente por que a
   mutação é inofensiva por projeto.

Reporte no formato: N mutações, M sangram, e a lista dos sobreviventes com o porquê.
