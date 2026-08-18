---
name: feedback-moving-a-doc-means-resolving-links-not-matching-strings
description: "Mover doc = reescrever link por RESOLUÇÃO contra o dir de quem cita, com gate antes/depois por path resolvido; casar string erra a forma relativa"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 5923af57-5df6-4517-a056-e8135a28aca3
  modified: 2026-08-10T16:54:18.392Z
---

Mover um `.md` de lugar é 5% `git mv` e 95% reescrever quem aponta pra ele. Na arrumação de
2026-08-10 (257 handoffs saindo de `docs/` para `docs/<Módulo>/handoffs/`) eram **643 links
internos + 58 citações do CLAUDE.md**, e eu cometi **três** erros da mesma família — todos
pegos pelo gate, nenhum por leitura.

**Why:** um link quebrado não falha nada — não há compilador, não há teste. Ele só faz a
próxima LLM não achar o arquivo e concluir que o assunto não tem dono. Os três erros:

1. **`git ls-files` DEPOIS do `git mv`** devolve os caminhos NOVOS ⇒ a passada de reescrita
   concluiu que os 255 movidos não tinham se mexido e não recalculou os links internos deles.
   **522 links quebrados** de uma vez. A lista tem de ser capturada ANTES.
2. **Early-out por string absoluta** (`if OLD not in s: continue`) pula quem cita pela forma
   RELATIVA (`../../Painter/handoffs/x.md`).
3. **Classificador por substring**: `'brush'` testado antes de `'flip'` mandou
   `..._line_FLIP_airbrush_...` para o módulo errado (*airbrush* contém *brush*).

**How to apply:**
- **Resolva, não case:** para cada link, `normpath(join(dir_de_quem_cita, unquote(alvo)))`;
  se o resolvido está no mapa de mudança, reemita `relpath(destino, dir_NOVO_de_quem_cita)`.
  Para os arquivos MOVIDOS, resolva contra o dir **antigo** deles.
- **Reescreva também o path em prosa/label/comentário de código** — `[`docs/x.md`](docs/x.md)`
  tem o path duas vezes, e o código deste repo cita handoff por caminho (`// Tracker: docs/…`).
- **Gate obrigatório:** verificador de links do repo inteiro **antes e depois**, comparando por
  **path RESOLVIDO** (comparar pelo alvo *escrito* conta mudança de grafia como quebra nova).
  A barra é **zero quebrado novo** — nunca o total, que aqui já era 1337 (dívida do
  `docs/archive/`).
- **Varra contaminação por substring** depois de classificar: o nome do arquivo carrega o sinal
  de outro módulo que não o destino? Uma em 257 estava errada.

Ver [[feedback_python_replace_silent_noop_after_fmt]] (o `assert old in s` que salvou o 3º
link), [[feedback_a_token_rewrite_scopes_to_changed_files_not_the_whole_tree]] e
[[reference_topic_gate_discipline]].
