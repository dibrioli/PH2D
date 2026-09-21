---
name: a-ruler-must-know-every-form-an-id-is-declared-in
description: Uma varredura que separa ids «escritos no fonte» de ids derivados tem de conhecer TODAS as formas de declaração — a que faltar erra sempre no sentido que subestima.
metadata:
  type: feedback
---

⚠️⚠️ **Há TRÊS formas de declarar um `NodeId` no PH2D, e uma régua que as separe tem de as
conhecer às três** (medido 2026-09-21):

| forma | exemplo | quem a usa |
|---|---|---|
| `hash_node_id("<lit>")` | `hash_node_id("tokens.close")` | a maioria dos painéis |
| `hash_node_id_runtime("<lit>")` | `hash_node_id_runtime("asset_browser.panel")` | `asset-browser` |
| **`NodeId(<n>)` cru** | `pub const GS_CFG_QT_MAX_DEPTH: NodeId = NodeId(1033);` | **`grid-snap`** |

⛔ **Uma forma que falte erra sempre no sentido MAU:** os ids dela deixam de ser reconhecidos como
«escritos no fonte», caem no balde dos derivados, **colapsam** com os vizinhos, e o painel lê-se
**mais barato do que é**. Medido: sem a terceira forma o `grid-snap` lia `10` comandos distintos em
vez de `20`.

⭐ **A cura não é lembrar-se das três — é o CONTROLO que reprova quando aparece uma quarta:** todo
painel cujo número bruto exceda o distinto tem de estar numa lista NOMEADA, com a **fábrica**
derivada escrita ao lado, com a metade da obsolescência e piso de população. Um painel que apareça
sem estar na lista é a varredura cega outra vez.

⚠️ **E a fábrica pode viver noutra crate** — a 1.ª redacção do controlo exigia-a na crate do
painel, e o `wet_tuning` desmentiu-a: os *Reset* dele nascem em `ph2d-tool-painter`.

Relacionado: [[a-biased-ruler-sends-work-to-where-it-is-itself-wrong]].
