---
name: project_outline_polygon_owner_decisions_2026_11_08
description: "Dono 08/11 — polígono pelo contorno da imagem: um contorno só no Animated Sprite, reentrâncias sólidas; aviso de corpo sem forma aprovado; linha GEF02 segue sem integrar"
metadata:
  node_type: memory
  type: project
  originSessionId: d13573b3-8c52-4287-aff8-371cde7d6085
  modified: 2026-11-08T19:56:22.891Z
---

Dono, 08/11/2026, sobre a física em cartões (`line/GameEngineFuncional02`), respondendo à proposta do polígono pelo contorno da imagem (Shape «Outline», Detail, Alpha cutoff, alças, «Follows the picture»):

- **Animated Sprite: UM contorno só** que cobre todos os quadros (não um por quadro).
- **Forma com reentrâncias: a parte de dentro é SÓLIDA** (peças convexas), não linha de borda oca.
- **Buraco fechado na imagem (argola) CONTINUA buraco** na colisão (o Godot tapa; nós não).
- Padrões técnicos do oráculo (`docs/GameEngineFuncional/oraculo_contorno/`): Alpha cutoff 0.5, Detail 2 px, contorno nunca corta a arte para dentro, ilha minúscula não vira colisão, teto de peças.
- Smoke 08/11 do fecho 2 (corpo sem forma, lista da aba, nome inteiro): OK nos 3.
- **Add ▸ Collision Shape:** sem seleção → objeto próprio com corpo Static invisível (nunca entra noutro corpo); com seleção → objeto FILHO (nunca componente); pai sem corpo → o filho tem o seu Static.
- ⛔ **REVISTO no smoke 08/11:** a lista «Shapes» no cartão do corpo «ficou confuso» — sai. **Opção B (modelo Godot): a Collision Shape SAI dos componentes; TODA forma é um objeto filho invisível na Hierarchy**; o corpo só tem o cartão Body; tamanho, Touch, Bounce, Friction, Layer e Outline vivem no Inspector da filha; nada de uma forma aparece no Inspector de outro objeto. Corpo nasce com a filha; projetos antigos migram (a forma vira filha). Respostas 08/11 à B: (1) Add ▸ Collision Shape sem seleção → «Static Body» invisível + «Collision Shape» filha; (2) com imagem sem corpo selecionada → a imagem VIRA corpo Static e a forma nasce filha dela; (3) a forma é como qualquer objeto: **alças só aparecem e só se edita quando ELA está selecionada** (corpo selecionado NÃO mostra alças da forma). Plano inteiro: `docs/GameEngineFuncional/componentes/f2_contorno_e_formas_plano.md` (GEF02).
- **Aviso de corpo sem forma** no cartão, com botão que cria a forma: «faça como sugere».
- Aparência da aba Collision Layers: depois das fotos, **o modelo é o editor de animações ATUAL da `line/GameEngineFuncional`** (não a maquete v2, que ficou velha): lista apertada, faixa discreta da largura toda na escolhida/pairada. Foto: `docs/GameEngineFuncional/handoffs/fotos_fisica/referencia/` (GEF02).
- **A linha NÃO integra agora; segue trabalhando. Integração à noite** (ordem dele).

**Why:** o dono decide produto; [[project_physics_cards_keep_all_config_power_2026_11_07]] continua valendo (não perder poder de configuração).
**How to apply:** construir o Outline com essas duas leis; não reabrir a pergunta. Integração só quando ele mandar à noite.
