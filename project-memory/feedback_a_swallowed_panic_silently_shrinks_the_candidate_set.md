---
name: feedback-a-swallowed-panic-silently-shrinks-the-candidate-set
description: Um catch_unwind que devolve recusa genérica esconde que a cadeia escolheu entre menos candidatas — conte-as; e um número de linha velho não desmente o ficheiro
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-31T01:52:31.732Z
---

Em 2026-08-30 (`line/quadextract`) o botão de retopologia corre **três** candidatas e escolhe
a melhor. Na escultura mais recente do Enio, **duas estouravam** (`ph2d-gridmap`,
`solve.rs:358/359`, `index out of bounds`) — o `catch_unwind` da porta convertia-as em
`RemeshRefusal::TooCoarseToResolve`, e a que sobrava saía topologicamente perfeita **com
`−41,8 %` de alcance**: as pontas cortadas.

⇒ O report dele — *«o remesh amputou pontas»* — era, em parte, **uma cadeia a escolher entre
uma candidata só**, e nada no produto dizia isso.

**Why:** uma rede que apanha o estouro protege a sessão e **apaga o facto**. Sem contagem, «a
melhor de três» e «a única que não morreu» imprimem exactamente a mesma linha. E a recusa
genérica ainda mentia sobre a causa: a frase mandava o artista *subdividir a escultura* — a cura
de um problema que ele não tinha.

**How to apply:**
- Toda rede de `catch_unwind` tem de ter **variante própria** (`Panicked`) e a porta tem de
  **contar as candidatas que chegaram**, não só escolher entre as que sobraram.
- Onde o índice pode faltar, **conte e salte** (`mismatched_locals`) em vez de estourar — e olhe
  **os dois lados antes de escrever qualquer um**: um par meio-acoplado é pior que nenhum.
- ⚠️ Guardar contra o `panic` **não é** achar a causa. Diga-o em voz alta no doc, com o contador
  a apontar para onde ela se mede.

⚠️ **E a segunda metade, que custou quatro dias:** o `CLAUDE.md` dizia *«o panic está SEM
ENDEREÇO — `solve.rs:336` aponta para código que já não existe»*. O **ficheiro estava certo**; a
função tinha sido reescrita e a linha desceu para `358`. *Um número de linha obsoleto não
desmente o ficheiro* — grepe a **operação** (`map.uv[p][l]`, `partners[p][l]`), nunca a linha.

Relacionadas: [[feedback_stopped_because_it_ended_reads_the_same_as_stopped_by_hand]] ·
[[feedback_a_bucket_nobody_fills_reads_as_perfect]] ·
[[feedback_stale_comment_and_dead_code_lie]] ·
[[feedback_an_automatic_tools_exit_code_says_nothing_about_what_it_produced]]

## Adenda 2026-09-01 — o `head` faz o MESMO, e custou uma afirmação errada num commit

Para decidir se quatro chips do trilho eram controlos mortos, corri:

```
grep -rn "TOOL_TRANSLATE\|TOOL_ROTATE\|TOOL_SCALE\|TOOL_PIVOT" … | grep -v tests | head -20
```

O `head -20` cortou a linha do `shells/desktop/input_dispatch.rs`, que é onde o `TOOL_PIVOT` é
LIDO para armar o arrasto do pivô. Conclui «zero consumidores», escrevi-o num doc-comment, num
handoff e numa mensagem de commit — e o erro só apareceu na jornada seguinte, ao construir a sonda
que o media a sério.

**Why:** *um `grep` truncado devolve «zero» com a mesma cara de um `grep` completo.* O `head` foi
posto ali por conforto de leitura, e a pergunta que eu estava a fazer era **de ausência** — a única
classe de pergunta em que um corte silencioso inverte a resposta.

**How to apply:** um censo que vai decidir se algo está **morto/ausente/não usado** corre **sem
`head`** e **conta**.

- Se a saída for grande, **agrupe** em vez de cortar: `| sed 's|:.*||' | sort | uniq -c | sort -rn`
  dá a mesma legibilidade e não perde nenhuma linha.
- Imprima o total (`| wc -l`) ao lado da amostra — um número ao lado de uma lista curta diz logo se
  a lista é a população ou uma fatia dela.
- ⚠️ Vale para `head`, `tail`, `-m`/`--max-count`, `| head` em pipelines de `find`, e para o
  `limit` de qualquer ferramenta de busca.
