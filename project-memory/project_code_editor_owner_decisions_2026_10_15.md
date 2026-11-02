---
name: project-code-editor-owner-decisions-2026-10-15
description: Editor de scripts Luau (line/CodeEditor) — as 4 decisões de produto do dono em 15/10/2026
metadata:
  node_type: memory
  type: project
  originSessionId: af38a609-65da-48fc-a6db-15ebe72c095a
  modified: 2026-10-24T13:31:55.857Z
---

O editor de scripts in-app (line/CodeEditor, plano em `docs/CodeEditor/`) é para o Luau que o `ph2d-script` já roda. Decisões do dono em 15/10/2026:
1. Script mora DENTRO do `.ph2dproj` (não arquivo solto) — degrau de schema.
2. Aceita empacotar o `luau-lsp` (~13 MB/OS) como processo-filho.
3. IA desligada agora; entra «da forma mais completa e poderosa possível» só no FIM da implementação.
4. Script pode apontar outro objeto pelo nome (arrastar da Hierarchy, como no Godot) — «e tudo que torne mais fácil, rápido e intuitivo». Reverte a regra antiga «scripts só por sinais».

5. (19/10, smoke do Acabamento) Ctrl+clique numa função da API (`ph2d.set`, `math.sin`) abre uma PÁGINA DE AJUDA numa aba (como o Godot: o que faz, cada argumento, exemplo, irmãs ao lado) — o balão preso do I6 era redundante com o rato parado. Nome mal escrito (`self.ampiltude`): SEM mão, e está certo assim.
6. (20/10) A API tem de FUNCIONAR: `ph2d.wait`, `find_by_name`, `spawn*`/`despawn`/`attach_script`, `input` (teclado/rato), `get` de objecto sem script e o estado lateral entram na fila da linha (item 6 de `docs/CodeEditor/03_onda2.md`), depois do smoke da página de ajuda. A página já diz às claras o que hoje não funciona.
7. (21/10, item 8 — arrastar para o código) O que os scripts não sabem tocar (componentes como Vida/Câmera, imagens, propriedades além da pose): **ensinar os scripts JÁ**, antes de fechar o item 8 — não recusar, não escrever só o dono. Fase 8d de `docs/CodeEditor/07_arrastar_para_o_codigo.md`.
8. (17/10, I8) As explicações dos erros só em INGLÊS, como o resto do app (ordem de 19/09); o texto mora em `ph2d-i18n/src/code_explain.rs`, pronto para outro idioma quando o app inteiro o tiver.
9. (24/10, as 3 perguntas de 22/10) (a) **SIM**, um script renomeia um objecto durante o jogo. (b) O cartão «Render Source» é da estratégia de ATLAS, não é o sprite: o nome dele (no cartão e no script) tem de dizer isso — **nunca `sprite`**. (c) Subtítulos nos cartões compostos (Câmera, Paralaxe, HUD): seguir o padrão-ouro do mercado, pesquisado.

**Why:** são produto, não técnica — só o dono muda. Detalhe e consequências: `docs/CodeEditor/00_decisoes_do_dono.md`.
**How to apply:** não reabrir estas perguntas; não pôr superfície de IA antes da última onda; toda referência a objeto é durável (StableId/nome), ver [[feedback_architecture_decisions_are_delegated_to_the_gold_standard]].
