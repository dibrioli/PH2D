---
name: feedback_no_old_projects_no_compatibility_until_the_app_works
description: "Dono 07/11: não existem projetos antigos; zero migração, fixture antiga ou desenho torto para não quebrar arquivo até o app ser funcional. Só o dono revoga"
metadata:
  type: feedback
---

Dono, 07/11: «Não existem projetos antigos e essa preocupação não deve existir até que o app seja funcional.» Até ele dizer o contrário: nada de migração, degrau de leitura, fixture de versão antiga, leitor de formato velho, nem componente irmão ou campo torto escolhido para não quebrar arquivo. Mude o tipo do jeito certo e suba o `PROJECT_SCHEMA` (o número continua contado). Teste de fixture antiga que reprovar por isso: apague-o. Regra em `CLAUDE.md` §0.11; HR-14 suspensa.

**Why:** nesta linha, `SpriteAnimStart` e `SpriteCutNotice` viraram peças separadas só para abrir projetos que não existem, e isso chegou ao dono como pergunta de produto: «por que o total de componentes sobe?».

**How to apply:** não proponha nem cite compatibilidade como custo ou motivo de desenho. Continuam valendo: contrato de código (§6), determinismo e o `PROJECT_SCHEMA` contado. Quando o dono declarar o app funcional, a questão volta, por ordem dele.
