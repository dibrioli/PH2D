# HANDOFF — linha `line/FLIP`, Waves W0 (dados) + W1 (render GPU) + W2 (tool + painel + desenho + borracha, **COMPLETA**)

**Status:** INDETERMINADO — ver o corpo (diz "NÃO integrado") · no `main` desde 2026-07-11 (`d7e7ce9e7`).

> 🟥 **PRÓXIMO AGENTE: comece por [`HANDOFF_flip_NEXT.md`](HANDOFF_flip_NEXT.md)** — o Modo L
> (o seu contrato), o estado da linha, as seis lições que custaram caro, e a **sua 1ª tarefa**: o
> problema aberto do balde (a referência do fill vs. a espessura da linha — causa já PROVADA, com
> os números). Este arquivo aqui é o tracker exaustivo do estado — leia-o DEPOIS.

> **Dois leitores:** (a) o **integrador** (§1.5.9) — o que fundir, símbolos que
> colidem, gates; (b) o **próximo implementador desta linha** — W2 fechou; o
> próximo tópico é **W3 (Frames · Ghost · Tween)**, guia em §W3-NEXT abaixo. A
> linha está **aberta, commitada local, NÃO integrada/pushada** — commits
> `--no-verify`, fast mode.
>
> **W2 fechou (2026-07-11, esta sessão):** o painel docado `ph2d-panel-flip`
> (Mode/Brush/Color/Layers), a borracha (Soft/Hard/Stroke), o seam painel↔tool +
> ops de camada no drain, a camada-ativa como alvo do traço, e o ready-to-smoke
> (ativar Flip num doc vazio cria um objeto). **Todas as decisões interinas de
> §W2.4 foram revertidas/resolvidas.** Gate W2 + auditoria abaixo.

---

## ⚠️ Recorte de 2026-08-18 — o que este arquivo é AGORA

Este era o **tracker exaustivo** da linha `line/FLIP`: 1 169 linhas de wave-a-wave (W0 → W7.5), com
gates, auditorias, rodadas de smoke e as decisões interinas de cada uma. Tudo isso **está no `main`
desde 2026-07-11 e depois** — é história —, e foi movido **verbatim** para
[`docs/archive/docs-2026-08-18/Flip/handoffs/HANDOFF_flip_impl.md`](../../archive/docs-2026-08-18/Flip/handoffs/HANDOFF_flip_impl.md).
⛔ Nada foi resumido: as duas metades remontam o original byte-a-byte (sha256).

**O que ficou aqui é a única coisa que ainda decide alguma ação: a fila ABERTA**, no fim.

### Índice do que foi para o arquivo (procure pelo título)

| bloco | o que responde |
|---|---|
| `## 1. Identidade` · `## 2. Foundational tocado` · `## 3. Símbolos novos` · `## 4. Contratos congelados encostados` · `## 5. O que SÓ o ship.sh pega` · `## 6. Ordem / dependências` | o **briefing de integração** da linha (já integrada) |
| `## Gate W0/W1/W2 — resultado` + `## Auditoria W0/W1/W2` | o que cada gate mediu, e as asserções-vermelhas reais |
| `# W2 — Tool de desenho + Painel + Borracha` (§W2.1-§W2.7) | o fluxo do desenho ponta-a-ponta, os arquivos, e os **gotchas** do painel/borracha |
| `## Smoke do Enio` (3ª · 4ª · 5ª · 6ª · 7ª rodadas) | as rodadas de smoke do traço, incluindo a **REPROVADA** e o *"mastigado"* |
| `## WT — O TRAÇO: a mordida está MORTA` + `### A mordida — mecanismo e fix` | o mecanismo do bug #1 do módulo (⚠️ o post-mortem canônico dele é o [`BUGS_flip.md`](../BUGS_flip.md)) |
| `## W3 — Frames · Ghost · Tween` · `## W4 — Fill` · `## W4.1 — A âncora do fill é o EIXO` | as waves de quadro e de balde |
| `## W5 — Reshape` · `## W5.1` · `## W5.2` | a escultura de traço, e o BUG que ficou aberto nela (fechado depois) |
| `## W6 — Edit Mode` · `## W6.1 — Os gestos` | a seleção de traço |
| `## W7 — Multiframe` · `## W7.1 Instance` · `## W7.2 pose do quadro` · `## W7.3 régua de scrub` · `## W7.4 falloff` · `## W7.5 pose AFIM` | a família da instância e da pose |
| `## Fixes do smoke pós-integração (2026-07-14)` | o que a integração cobrou |

## Aberto (fila viva — detalhe em [`HANDOFF_line_FLIP_CONTINUACAO_2026-07-15.md`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-15.md))

- 🔴 **W7.5 Fase 2 — o gizmo de rotate/escala da pose** (a próxima; mapa no handoff de continuação).
- 🔴 **ITEM 0 — W7 · W7.1 · W7.2 entraram no `main` SEM SMOKE** (registro da integração de
  2026-07-13). O smoke dessas três vem ANTES de feature nova.
- **A próxima recomendada: girar/escalar a seleção.** O Edit Mode só translada; assim que o
  Enio mover uma instância vai querer girá-la. Caminho: consumidor novo do gizmo de sprite
  (bbox da seleção → `GizmoView` → delta assado) — e a **pose da chave tem de virar AFIM**
  (hoje é `Vec2`), porque girar uma instância não pode escrever geometria compartilhada. O
  render e a entrada já compõem `Xform`, então só o TIPO do campo muda.
- **Seleção no domínio POINT** (hoje é por TRAÇO — domínio Curve do GP): move uma âncora só e
  dá máscara fina ao Sculpt.
- **W6 (timeline global): ADIADA** — a timeline principal ainda está em desenvolvimento
  (Enio 2026-07-12). O playhead do Flip JÁ é o global, então a integração não terá relógio a
  reconciliar.
- **Refinos do Select (não-bloqueantes):** escala NÃO-uniforme engrossa o traço pela
  escala MÉDIA (`mean_scale`) — aproximação; espessura anisotrópica exigiria passar o
  afim ao shader. Persistência da pose Flip no `ProjectState` (o `Transform` é ECS →
  já entra no `WorldSnapshot`; a geometria local idem — deve funcionar, mas não
  smoke-testei o round-trip pós-move).
- **Refinos do painel/borracha (não-bloqueantes):** duplicar/agrupar camada
  (só `add`/`delete`/reorder landaram; `FlipObject` não tem `duplicate_layer`);
  reorder por DRAG (só ↑↓ por botão); máscaras de camada na UI (`FlipLayer.masks`
  existe no modelo, sem UI); curva de pressão editável + pen real (pressão=1.0 no
  mouse). Borracha: raio dedicado (hoje = tamanho do brush) + preview do círculo.
- **Deferido no W1 (v1 usa flat caps + miter clampado):** round caps, bevel/round
  joins. Máscaras de camada (`FlipLayer.masks`) — o modelo carrega, o compositor v1
  não aplica (o op-list GPU não tem máscara; igual ao Painter).
- **Cache de tesselação:** sem LRU (cresce com nº de desenhos únicos vistos —
  bounded pelo documento, ok pro W1). W2 pode adicionar cap se necessário.
- Persistir `flip` cross-sessão já funciona (entra no `ProjectState`); a UI real de
  Save/Open continua stub (herança do estado atual da persistência).
- **Docs de planejamento** (`docs/Flip/`, `docs/architecture/decisions/0114-*.md`,
  `project-memory/project_flip_module_grease_pencil_2d.md`) estão **untracked na
  árvore primária** — NÃO os commitei nesta linha (senão o `merge --ff-only` da
  integração quebra com "untracked working tree files would be overwritten"). O Enio
  deve commitá-los ao `main` por fora, antes ou depois da integração.
