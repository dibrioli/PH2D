---
name: project-pixel-slices-owner-decisions-2026-11-07
description: "Decisões do dono na P11 (Slices do Pixel): a slice segue a arte, quatro extras além do Aseprite, corte nos dois lugares da exportação"
metadata:
  node_type: memory
  type: project
  originSessionId: 87d1f040-27f6-47d0-8077-cfd481f5dff7
  modified: 2026-11-08T16:03:40.134Z
---

Dono, 07/11, na abertura da P11:

1. **A slice segue a arte ou fica parada como no Aseprite?** Delegou: «veja o que mais os artistas gostam». Decidido pela linha: **segue**, nos quadros e na geometria. Quadros têm pedido público aberto (aseprite#1469, #3142, fórum 5329). Virar e girar: nenhum pedido achado; segue por coerência, e o smoke confirma.
2. **Os quatro extras entram:** nome na tela quando selecionada · marcas na linha do tempo · remover a mudança de um quadro (botão direito) · `Transform` (levar os pixels junto, desligado por omissão).
3. **Corte na exportação nos dois lugares:** `Split slices` no Export Sheet e `Area` (Canvas ou o nome da slice) no Export As.

Smoke 08/11: virar/girar seguindo a arte **OK**; mover slice de uma só chave muda daquele quadro em diante (G4) **OK**. Pediu: letra dentro do campo numérico (arrastar a letra), X Y / W H em duas linhas, alça de largura nas janelas acessórias, realce + seta ao pairar nas linhas 9-slice, alças e pivot, e o Transform a levar o desenho DURANTE o arrasto (só ao largar lia-se como «não funciona»). Smoke 2 (08/11, depois da auditoria): **«smoke OK»** — P11 fechada; próxima onda P12 ([handoff](../docs/PixelArt/handoffs/HANDOFF_CONTINUACAO_line_PixelArt_2026-11-08_P11.md) na linha).

**Why:** o Aseprite deixa a slice para trás quando se mexe em quadros; é defeito aberto, não lei a copiar. O Trial não grava imagem, então o corte por slice é lei nossa, sem paridade.

**How to apply:** `docs/PixelArt/P11_plano.md` §4.4 e §9. Não reabrir sem o dono. Se o dono rejeitar virar/girar no smoke, é só essa parte que muda. Relacionado: [[feedback_the_owners_reference_product_decides_the_law]].
