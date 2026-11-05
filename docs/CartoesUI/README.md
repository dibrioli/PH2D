# Cartões UI — painéis como listas do Kanban, campos como o Figma

> **Estado (05/11): a UI em cartões é o padrão do `main`.** O parágrafo seguinte é a história da linha de teste.
>
> **Linha de TESTE FECHADA em 2026-11-02** pelo dono («Já conseguimos testar a UI e ela funciona muito
> bem e é linda»). `line/TrelloUITeste` (worktree `Worktrees/line-TrelloUITeste`), aberta em 2026-11-01
> a partir de `line/TrelloClone` @ `4e648ec2b4`. **Nunca foi integrada nem teve push.** A implementação
> real será uma **linha nova a partir do `main`**, e a especificação dela é
> **[05_ESPECIFICACAO_UI_COMPLETA.md](05_ESPECIFICACAO_UI_COMPLETA.md)** (leia-a primeiro). Muitos
> outros painéis NÃO foram trabalhados: foi só um teste de UI.
>
> **Regra de referência (dono 02/11, ordem):** (1) NÓS — os painéis/cartões cujo smoke o dono aprovou;
> (2) **Figma** para todo widget ainda não criado; (3) **Rive** se o Figma não resolve; (4) Inkscape,
> Godot e Unity como último recurso.

## O que é

Refazer a UI dos painéis laterais do PH2D com duas referências ao mesmo tempo:

- **Kanban (TrelloClone)** — a APARÊNCIA, as ANIMAÇÕES e a DINÂMICA de arrastar: o painel é uma
  **lista**, cada secção do painel é um **cartão**.
- **Figma** — o LAYOUT dentro do cartão: nomes das propriedades, disposição de cada widget, o campo
  numérico com ícone + valor na mesma caixa, que se arrasta pelo ícone com o mouse a atravessar o ecrã.

Alvo declarado pelo dono: **superar o Figma e a Godot** (a referência atual).

Todo o app está em cartões desde 05/11 ([handoff](handoffs/HANDOFF_INTEGRACAO_NovaUI_2026-11-05.md)).

## ABERTO

Falta o portão que reprova painel registado sem `panel::cards` (ao lado de
[`crates/ph2d-panel-registry-init/tests/it/lei_da_compactacao.rs`](../../crates/ph2d-panel-registry-init/tests/it/lei_da_compactacao.rs)).
Até lá a regra é só dos documentos. Entra no próximo ship.

## Índice

| ficheiro | conteúdo |
|---|---|
| [01_decisoes_do_dono.md](01_decisoes_do_dono.md) | cada decisão do dono, com data e a palavra dele — **lei** |
| [02_especificacao.md](02_especificacao.md) | anatomia, medidas, tokens, tipografia, comportamento dos campos, arrasto, cartão solto no canvas |
| [05_ESPECIFICACAO_UI_COMPLETA.md](05_ESPECIFICACAO_UI_COMPLETA.md) | **A especificação para a linha real**: regra de referência, tabela «smoke OK», vocabulário completo, soltos/corda/face, Camera card, ABERTO consolidado (substitui a 02 onde divergirem) |
| [06_COMO_PORTAR_PARA_O_MAIN.md](06_COMO_PORTAR_PARA_O_MAIN.md) | **Como a linha real usa este CÓDIGO** (estudo medido 02/11): portar o estado final por família e em ordem de dependência (diff a partir de `4e648ec2b4` + `git apply -3`), nunca merge nem cherry-pick dos 89 commits; conflitos, esquema, shell, crates novas, interruptor `PH2D_CARDS` |
| [07_PROMPT_DA_LINHA_REAL.md](07_PROMPT_DA_LINHA_REAL.md) | **O prompt para colar** na janela que abre a linha real a partir do `main` (abrir, ler, régua, porte por família, regressão, Camera e ABERTO, protocolo, fecho) |
| [03_mapa_do_codigo.md](03_mapa_do_codigo.md) | onde vive hoje cada peça (painéis, secções, Kanban, mola, scrub, cursor), o que reutilizar |
| [04_plano_de_construcao.md](04_plano_de_construcao.md) | ondas, ordem e o que cada onda prova |
| [handoffs/](handoffs/) | o fecho de cada janela da linha de teste (blocos 3–7, 8, 9, 10 e o [FECHO](handoffs/HANDOFF_line_TrelloUITeste_2026-11-02_FECHO.md) do bloco 11): o que foi feito, provas, ABERTOS |
| [maquete/](maquete/) | a maquete APROVADA (HTML + PNG), as referências e o «antes» |

## A maquete aprovada (smoke OK do dono, 2026-11-01)

- [`maquete/bones.png`](maquete/bones.png) — **A** Bones em repouso · **B** Bones a mexer (cartão
  no ar, scrub no Length, conta no campo) · **C** Inspector principal com uma forma escolhida.
- [`maquete/canvas.png`](maquete/canvas.png) — **D** cartão solto no canvas.
- Fonte das imagens: `bones.html` / `canvas.html` (Chrome headless:
  `google-chrome-stable --headless=new --screenshot=… --window-size=1900,1180 file://…`).
- Referências: [`referencia_figma_painel.png`](maquete/referencia_figma_painel.png) (captura do Figma
  enviada pelo dono), [`referencia_kanban.png`](maquete/referencia_kanban.png)
  (`PH2D_KANBAN_SMOKE=1`), [`antes_painel_bones.png`](maquete/antes_painel_bones.png)
  (`PH2D_VEC_BONE_SMOKE=7`, como estava).

⚠️ A maquete é a **referência visual vinculativa** («quero idêntico à maquete»). Onde o texto e a
imagem divergirem, a imagem manda e o texto corrige-se.
