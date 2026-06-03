═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · FINALIZAÇÃO COMPLETA DA UI VETORIAL (W2 close)
Autor: Implementador Vector · 2026-06-03 · pedido direto do Enio
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
O **comportamento** do Vector W2 está completo + auditado (T2.1-T2.6, veredito
SHIP_WITH_FIXES, 10/11 fixes landados). Falta a **camada de UI/UX**, que o Enio
quer finalizada de uma vez, **coerente e tokenizada** (HR-15: zero hex, zero f32
de UI literal, tudo via tokens/i18n). Isto é chrome/design-system = **tua pasta**.

**Pedido NOVO do Enio (prioridade):** *"o handle selecionado deve ter cor
diferente dos demais."* → §1.

Para não colidir, **NÃO vou mais tocar** o overlay (`vector_selection_bridge.rs`)
nem a chrome — assumo que esta passada de UI é tua. Onde precisar de uma ação de
dado (set kind, set fill, …) os helpers do tool já existem (cito abaixo).

---

## §1 — Direct-Select overlay: estados visuais dos handles/vértices (PEDIDO DO ENIO)
Arquivo: `shells/desktop/src/render_loop/vector_selection_bridge.rs` (eu escrevi
no rank 5; **agora é teu** para a passada de UI). Hoje TUDO usa um único
`ACCENT_RGB = (255,170,60)` **hardcoded** (viola HR-15) com alphas variados:

| Camada (atual) | O que pinta | Cor hoje |
|---|---|---|
| 1 | bbox da network selecionada | accent α210 |
| 1.5 | **TODOS** os vértices + handles de tangente (Direct ativo) | accent α210 / linha α180 / dot-ctrl α235 |
| 2 | vértice selecionado | accent α235 |
| 3 | marquee | accent fill α40 + linha α180 |

**Finalizar (recomendação, padrão Illustrator/Figma/Affinity):**
1. **Handle SELECIONADO ≠ demais (o pedido):** o handle (tangente) atualmente
   *grabbed* deve ter cor própria (ex.: token `accent.active`/azul) vs os
   inativos (cinza/accent-dim). Fonte da verdade do "qual está grabbed":
   `VectorDirectTool::grab() -> Option<DirectGrab>` (`GrabTarget::Tangent(seg,side)`
   ou `Vertex(id)`) — já público. O overlay lê o tool via downcast (mesmo padrão
   do marquee, `vector_selection_bridge.rs:91-94`).
2. **Tipo de vértice visível por FORMA** (pro convention): Smooth=`Mirror`→círculo,
   Asymmetric=`Aligned`→círculo, Corner=`Free`→quadradinho, Auto→losango/oco.
   O kind está em `vertex.kind` (`VertexKind`, 4 variants frozen). Isso fecha o
   loop visual do feature de tipos-de-ponto (commit `b62a70d`).
3. **Vértice selecionado vs hover vs normal** — 3 estados (cor/tamanho), hoje só
   há selecionado-vs-resto.
4. **Tokenizar TUDO** (HR-15): trocar `ACCENT_RGB`/alphas/larguras-px por tokens
   do design system. As larguras já dividem por `k` (camera scale) — manter.

---

## §2 — Menu de botão-direito: TIPO DE PONTO (spec já enviada)
Detalhe completo em [`HANDOFF_vector_w2_audit_fixes_coord.md`] (bloco "PEDIDO →
Coord · menu botão-direito de TIPO DE PONTO"). Resumo: novo
`ContextMenuKind::VectorPointType` + 4 entries (Corner/Smooth/Asymmetric/Auto) +
expor a escolha drenável (padrão `take_pending_shape_selection`). **Eu fecho o
shell-glue** (Secondary-Down sobre vértice → abre menu → drena → chama
`VectorDirectTool::set_selected_vertex_kind`, já público) assim que o variant
landar. Hoje funciona via teclas **1-4** (stopgap Alt-free, commit `b62a70d`) —
ver §5.

---

## §3 — Consolidar os 5 pills → 1 modo "VECTOR" (§4.3, já sequenciado)
`fixture.rs` (editor-core) já previa ("parallel to IMG"). Paridade
ImageToolsV1: estado `vector_mode` + `paint_vector_tool_row` + backdrop +
active-ring + dispatch do toggle + atualizar o gate
`topbar_painted_pills_are_all_registered`. Reestrutura pills que HOJE FUNCIONAM →
só smoke visual do Enio confirma. **Aguarda greenlight do Enio.**

---

## §4 — Inspector docado: consolidar controles vetoriais
`ph2d-panel-vector-inspector` (tua crate) hoje mostra: swatch **Fill** + picker
**ShapeKind** (quando Shape ativo). Para uma UI "completa":
- **Stroke style** (Pencil/Spiral/Pen-aberto): width / cor / cap / join. Setters
  já existem: `set_default_stroke(color,width)` em pencil+shape; o modelo tem
  `StrokeStyle` (cap/join). Hoje só a cor flui (via §6 do outro handoff, `35e5b37`).
- **Point type row** (alternativa/complemento ao menu botão-direito): 4 botões
  Corner/Smooth/Asymmetric/Auto quando Direct ativo + vértice selecionado →
  `set_selected_vertex_kind`. (Decida: menu-direito, painel, ou ambos.)
- **Fill mode**: hoje só solid. Gradient 2-stop é teu foundational (§6 do outro
  handoff) — quando landar, o inspector ganha o controle.

---

## §5 — Stopgaps de teclado a reconciliar (decisão de UX tua)
Implementei 2 stopgaps Alt-free/mouse-independentes pra desbloquear o Enio JÁ:
- **Shape sub-modo:** teclas **1-5** (Rect/Ellipse/Polygon/Star/Spiral) — commit
  `023f1e5`. (Tu já fez o picker on-screen `1e3a1be`; o 1-5 ficou paralelo.)
- **Direct point-type:** teclas **1-4** (Corner/Smooth/Asymmetric/Auto) — `b62a70d`.

Quando os pickers/menus on-screen estiverem completos, **decida**: manter as
teclas como atalho de power-user (recomendo) ou aposentar. Se manter, documentar
num cheatsheet/tooltip (i18n).

---

## §6 — Pendências de comportamento que afetam a UI (não-UI puro)
- **#10 (MEDIUM, teu):** `VectorSceneRef` (bbox/centroide da gizmo-box) só é escrito
  no spawn (`vector_scene.rs:186`) — após editar vértice no Direct, a gizmo-box fica
  stale. Recompute on Direct-edit. (Já no outro handoff.)
- **Curva do Pen / direção:** preciso do smoke visual do Enio (não testo GUI). Se a
  curvatura sair invertida = 1 flip de sinal em `vector_pen` `drag_handle`.

---

## §7 — Tokens / i18n (HR-15) — varredura
A passada de UI deve **tokenizar** todo o vetor: o overlay (`ACCENT_RGB` + alphas
+ larguras), e conferir que labels/toasts que adicionei estão em **inglês**
(`"Point: Corner"`, `"Shape: ellipse"`, `"Undo"/"Redo"`, `"Loaded N vector
path(s)"`) — segui o app-UI-english-only, mas confirma no teu sweep.

## §8 — Divisão / coordenação
- **Teu (esta passada):** overlay visual (§1), context-menu chrome (§2),
  pills→VECTOR (§3), inspector (§4), tokens (§7), #10 (§6).
- **Meu (sob demanda):** shell-glue do menu-direito (§2) quando o `ContextMenuKind`
  landar; qualquer ação de dado nova (helpers de tool já expostos:
  `set_selected_vertex_kind`, `set_default_fill/stroke`, `apply_fill_to_selection`).
- **NÃO toco** `vector_selection_bridge.rs` / chrome / panel-crate enquanto esta
  passada estiver aberta (anti-colisão). Me avisa o que precisar de dado.
═══════════════════════════════════════════════════════════════════
