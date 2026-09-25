---
name: a-pgrep-watcher-catches-its-own-shell
description: Um vigia `until ! pgrep -f <padrão>` NUNCA termina — o shell do laço tem o padrão no próprio cmdline, logo o pgrep auto-apanha-se
metadata:
  type: feedback
---

`until ! pgrep -f "<padrão>"; do sleep N; done` **nunca fica falso**: o shell que corre o laço tem
`<padrão>` dentro do próprio `/proc/<pid>/cmdline`, logo o `pgrep -f` **encontra-se a si mesmo** e a
condição é verdadeira para sempre. O mesmo vale para `pgrep -c` (conta 1 a mais) e para `pkill -f`,
que **mata o próprio shell** (saída `143`/`144`).

**Medido (PH2D, 2026-09-20).** Armei um vigia com `until ! pgrep -f "ph2d_app_field3d-.* --ignored"`
para ser avisado do fim de uma bateria de GPU. Ele **expirou aos 15 min sem um único evento** — e a
bateria já tinha morrido havia muito. A seguir, um `pgrep -c 'ph2d_app_field3d'` leu **`1`** sobre
**zero** binários vivos, e o `pkill` correspondente matou o shell que o invocava.

**Why:** o modo de falha é o caro — *o silêncio de um vigia lê-se exactamente como «ainda a
trabalhar»*, e a contagem inflacionada lê-se como «há um processo pendurado». Foram duas conclusões
erradas sobre o estado da máquina, seguidas.

**How to apply:**
- Para esperar por um processo, use o **PID**: `tail --pid=$PID -f /dev/null` com `timeout`, ou o
  `wait` do próprio job — nunca um padrão de linha de comando.
- Para CONTAR, pergunte ao que não se auto-inclui: `ls -l /proc/*/exe | grep -c <binário>`.
- Se o padrão for mesmo necessário, exclua-se: `pgrep -f "<padrão>" | grep -v "^$$\$"` — mas é mais
  frágil do que usar o PID.
- ⛔ E um vigia que devolve o mesmo resultado quer o processo esteja vivo quer esteja morto **não é
  um vigia**: antes de o armar, pergunte *«se isto já tivesse acabado, ele emitia?»*

Irmã: [[a-tail-is-a-window-not-a-verdict]] · [[a-mutation-proof-needs-a-control-on-its-own-filter]]

⛔⛔ **RECORRÊNCIA (2026-09-21, a instalar o Bevy Solari): armei CINCO destes na mesma sessão, com
esta nota já escrita.** Os cinco ficaram a girar, e a limpeza teve de ser feita **DUAS** vezes: à primeira procurei
pelo nome de um deles (`fotografa_bevy`) e matei **três**, dando o assunto por fechado; os
outros dois — vigias de COMPILAÇÃO, com outro padrão — só apareceram quando procurei pela
**FORMA** (`until ! pgrep`) e já giravam havia 18 minutos.
⇒ *uma limpeza que procura pelo NOME de um caso limpa esse caso; a varredura tem de procurar
pela FORMA do defeito, senão o número que se reporta ao dono está errado — e eu reportei «três».* *Uma nota que
descreve a armadilha não impede a armadilha: o que a impede é não escrever o padrão duas vezes.*
⇒ **a forma segura é o `Bash` com `run_in_background`**, que notifica pelo PID do próprio comando e
não precisa de padrão nenhum; o `Monitor` com `until` fica para condições que NÃO são «este
processo acabou» (um ficheiro aparecer, uma linha num log).

⛔ **E a VERIFICAÇÃO tem a mesma doença**, que é como eu quase me convenci de que sobrava um vivo: o
`pgrep`/`ps | grep` da conferência também se auto-apanha. ⚠️ **Nesta máquina o `grep` é o `ugrep`**,
e ele aparece no `ps` como `ugrep -G --ignore-files … -F <padrão>` ⇒ o idioma clássico
`| grep -v grep` **não casa** (o cmdline não contém a cadeia que se está a excluir). Conte pelo
`/proc/*/exe`, ou olhe para a lista e leia-a.
