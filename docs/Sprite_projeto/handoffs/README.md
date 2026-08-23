# `Sprite_projeto` — handoffs (registro cronológico de sessão)

> **O que é esta pasta:** o registro de **como** o módulo foi construído — um arquivo por
> sessão de linha. O **pensamento** do módulo (spec numerada, auditorias, planos) fica **um nível
> acima**, em [`docs/Sprite_projeto/`](..).
>
> ⚠️ **Isto NÃO é o estado atual do módulo.** O estado vivo é o **[`CLAUDE.md §5`](../../../CLAUDE.md)**;
> um handoff descreve o mundo **no dia em que foi escrito** e não é atualizado depois. Use-os
> para responder *"por que isto ficou assim?"* — nunca para decidir a próxima ação.

⚠️ **Este índice nasceu em 2026-08-23, e a razão é uma cicatriz deste próprio módulo.** Até aqui a
pasta não tinha índice e **nada no repositório citava os handoffs dela** (`git grep` do nome do de
22/08: zero). ⛔ `docs/Sprite_projeto/` também **não entra** no gerador `scripts/doc-index.sh`, e
isso é deliberado — o `README.md` um nível acima é a **spec**, e gerar um índice por cima dela
apagá-la-ia. Por isso este ficheiro é escrito à mão, e um handoff novo entra aqui **no mesmo commit**
em que é criado.
>
> *Um doc órfão do roteador é uma regra que ninguém lê* — e este módulo já pagou exactamente isso:
> o `CLAUDE.md §5` dizia «fechado sem pendência» enquanto **três seções da spec nunca tinham
> nascido**, e a informação existia desde 2026-05-31 num handoff arquivado.

**3 handoffs** · **2** citados pelo `CLAUDE.md §5` (marcados **◆**).

| Data | | Arquivo | Papel | Assunto |
|---|---|---|---|---|
| 2026-08-22 | | [HANDOFF_INTEGRACAO_line_Sprite_2026-08-22.md](HANDOFF_INTEGRACAO_line_Sprite_2026-08-22.md) | integração | §5 9-Slice + §12 Sockets/Âncoras + o gizmo de canvas |
| 2026-08-23 | ◆ | [HANDOFF_INTEGRACAO_line_Sprite_MOUNT_2026-08-23.md](HANDOFF_INTEGRACAO_line_Sprite_MOUNT_2026-08-23.md) | integração | **o consumidor de uma âncora** — `AnchorMount`, a lei das duas travessias, e o bloqueio medido de Luau/MCP |
| 2026-08-23 | ◆ | [HANDOFF_INTEGRACAO_line_Sprite_ANIM_AUDIT_2026-08-23.md](HANDOFF_INTEGRACAO_line_Sprite_ANIM_AUDIT_2026-08-23.md) | integração | **o transporte da §11** — a caixa com duas fontes de verdade, o rebobinar que não movia a imagem, e os 10 gates de costura que faltavam |

---
*Handoff novo entra nesta tabela, não só na pasta.*
