---
name: feedback-integration-order-comes-from-measured-overlap
description: "A ordem de integração das linhas se MEDE (sobreposição de arquivos par-a-par), não se lê nos handoffs — a primeira linha é fast-forward de graça"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 50723bb0-6f74-4589-81dc-ee242a680d8c
  modified: 2026-08-10T10:01:06.527Z
---

Numa jornada Modo L de N linhas, o `--ff-only` serializa: **a 1ª é fast-forward puro (zero conflito,
de graça), todas as demais rebaseiam** sobre o main que cresceu. Logo a ordem não é arbitrária — e não
se deriva dos handoffs, que descrevem o que cada linha *acha* que vai colidir.

Meça antes de escolher:

```bash
for w in <linhas>; do git -C Worktrees/line-$w diff --name-only main...HEAD | sort > /tmp/f_$w.txt; done
# sobreposição par-a-par
comm -12 /tmp/f_A.txt /tmp/f_B.txt
```

**Why:** integra-se PRIMEIRO a linha de maior sobreposição — ela é a que mais custaria em rebase, e o
FF a torna gratuita. As quase-disjuntas vão por último, quando o main já está gordo, porque o rebase
delas custa quase nada de qualquer jeito. Numa jornada de 6 linhas (2026-07-18) a medição desmontou a
intuição: `Painter` e `gpu-nodes` pareciam gigantes (33 e 51 arquivos) e tocavam **2 arquivos** em comum
com o resto (`Cargo.lock`, `CLAUDE.md`); o gargalo real era o punhado de arquivos quentes do shell
(`app_state.rs`, `main.rs`, `init.rs`, `project.rs`, `render_loop/mod.rs`) compartilhado por 4 linhas.
Resultado: 107 commits, e só **3 conflitos** — todos previstos pela tabela de sobreposição.

**How to apply:** antes da 1ª integração, rode a matriz de sobreposição e ordene por grau decrescente.
A tabela também é o mapa dos conflitos que virão: um arquivo que aparece em 4 pares é onde você vai
parar, então leia-o antes. E ela responde de graça a pergunta que os handoffs erram — *"esta linha
colide com aquela?"* — porque `git diff --name-only` não tem opinião.

Corolário: colisão de **número** (ADR, schema, id) não aparece nesta tabela quando as duas linhas
criam arquivos de nomes diferentes — para essas, o handoff §1.5.9.3 é a fonte, e o valor certo
**se conta** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

⚠️ **Corolário 2 (medido em 2026-07-31, e é um buraco do comando acima): `git diff` é NET, o rebase
é COMMIT A COMMIT.** Um arquivo que a linha **cria e depois apaga** dentro dela mesma tem diff net
ZERO contra a base — some da tabela — mas o rebase replaya cada commit: o nascimento colide
(**add/add**) com um arquivo que outra linha criou no MESMO path, e a morte então ameaça apagar o
sobrevivente (**delete/modify**). Instância: `line/anim` × `line/motion-value` em
`crates/ph2d-editor-core/src/paint_shapes.rs` — a `anim` o criou para o preview de expressão e o
apagou ao retirar a feature; a `motion-value` criou o dela com `fill_diamond`. Dois conflitos que o
handoff da `anim` **não previu**, porque a previsão foi feita com o diff net.

A tabela precisa da segunda metade:

```bash
git log --diff-filter=AD --name-only --pretty=format: main..HEAD | sort -u | grep -v '^$'
```

(arquivos NASCIDOS ou MORTOS em qualquer commit da linha; intersecte com a lista da outra linha).
A resolução certa é pelo CONSUMIDOR: sobrevive a função cujo chamador sobrevive — apagar o arquivo
levaria junto o símbolo do outro dono.

⚠️ **Corolário 3 (2026-08-10): a BASE que o handoff declara também se re-mede.** Os dois handoffs
daquela janela afirmavam estar sobre o `main` do dia — o da `motion-value` com o comando ao lado
(*"`git log HEAD..main` = 0"*) — e os dois estavam **76 commits atrás**: o número foi medido no
fechamento da linha e envelheceu entre ele e a ordem de integrar. Não muda a tabela (o `main...HEAD`
de três pontos é merge-base-relativo, logo imune), mas muda o que você espera do rebase: um handoff
que se diz *ff puro* pode custar um conflito. Custa uma linha conferir antes de abrir a primeira:

```bash
for b in <linhas>; do echo "$b ahead=$(git rev-list --count main..$b) behind=$(git rev-list --count $b..main)"; done
```

**Why:** todo número dentro de um handoff é uma medição *datada*; os que descrevem a LINHA envelhecem
devagar (o diff dela não muda), e os que descrevem a RELAÇÃO com o `main` envelhecem a cada integração
alheia. Trate a §1 do handoff como afirmação sobre o passado, nunca sobre a árvore de agora.
