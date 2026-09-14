---
name: feedback_the_grep_in_this_agents_shell_is_ugrep_and_qv_always_says_no
description: "O `grep` dentro do shell deste agente é um wrapper para ugrep — `grep -qv PADRÃO` devolve «não encontrei» SEMPRE, e todo predicado escrito assim lê VERDE"
metadata:
  type: feedback
---

⛔⛔ **O `grep` que este agente invoca NÃO é o `grep` do sistema.** O harness injecta uma **função de
shell** chamada `grep` que roteia para o **ugrep**; o binário real (`/usr/bin/grep`, GNU 3.12) só se
alcança pelo caminho absoluto.

E os dois **discordam num caso silencioso**, medido em 2026-09-14:

```
printf 'exit=0\nexit=100\n' | grep -qv "exit=0"           # ugrep:  rc=1  («não encontrei»)
printf 'exit=0\nexit=100\n' | /usr/bin/grep -qv "exit=0"  # GNU:    rc=0  (encontrou a linha sem o padrão)
```

⚠️ **O `-v` sozinho funciona nos dois** (`grep -v` imprime a linha; `grep -vc` conta `1`). É a
combinação **`-q` + `-v`** que devolve `1` sempre — logo **todo predicado da forma
`grep -qv PADRÃO` responde «não» incondicionalmente**, e um portão escrito com ele lê VERDE sobre
qualquer entrada.

⛔ **Como isto mordeu:** o veredito de um portão de fecho era
`if <logs> | grep -qv "exit=0"; then echo VERMELHO; else echo VERDE; fi`. O ficheiro tinha
`exit=100` numa das doze linhas — **dois testes reprovados**, um deles um teto de LOC real — e o
vigia anunciou **«GATE VERDE: todos os passos exit=0»**. O que salvou foi ler o ficheiro à mão a
seguir; sem isso, um vermelho real ia para a integração com um verde escrito ao lado.

⚠️ **Os scripts do REPO estão a salvo** — eles correm fora desta shell, onde `grep` é o GNU (há dois
`grep -qv` vivos em `scripts/fix-travamentos.sh` e `scripts/ram-build.sh`, os dois correctos lá). *A
armadilha é da ferramenta do agente, não da máquina* — o que a torna pior: ela só aparece no lado que
MEDE.

**Why:** um predicado que responde sempre «não» é a forma mais cara de régua partida — ele não falha,
ele **aprova**. E o que ele aprova é precisamente o que se estava a tentar apanhar.

**How to apply:** ⛔ nunca escreva `grep -qv` (nem `-vq`) num veredito. Use a forma que não depende
do par: `[ -n "$(… | grep -v PADRÃO)" ]`, ou conte (`grep -vc`, e compare com zero), ou grepe a
condição **positiva** (`grep -qE "exit=[^0]"`). ⚠️ E **antes de confiar num predicado de shell novo,
corra-o contra uma entrada que ele DEVE reprovar** — a metade justa de um gate vale igual para um
one-liner. Ver [[reference_topic_measurement_discipline]] e
[[feedback_a_tail_is_a_window_not_a_verdict]] (a mesma família: a leitura que parece um veredito e
não é).
