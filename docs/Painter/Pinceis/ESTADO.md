# ESTADO da missão Pincéis — MISSÃO ENCERRADA (J5, 2026-10-22) (≤ 8 KB)

> **Encerrada com o alvo NÃO alcançado:** 18 pincéis APROVADOS (alvo 80–120); fábrica 45 (Wet 29, Digital 16), os
> outros 27 «curados, não aprovados». Porquê: o que trava os 67 itens restantes é MOTOR (depois da 2.ª ronda o
> rendimento é zero) e as [regras de custo](REGRAS_DE_CUSTO.md) do dono limitam a uma lacuna de motor por agente. A fila
> de motor, medida, está em [01_CATALOGO §3d](01_CATALOGO.md) — é o escopo de uma missão futura. Integração:
> [HANDOFF_INTEGRACAO ...PINCEIS](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-22_PINCEIS.md).
> Contrato: [`00_MISSAO.md`](00_MISSAO.md). Detalhe por janela: `HANDOFF_JANELA_<n>.md`; por família: `exames/<família>.md`.

## Feito

- **J1** ([handoff](HANDOFF_JANELA_1.md)): bateria, oráculos fotografados, catálogo (112), bancada, `03_MOTORES.md`.
- **J2** ([handoff](HANDOFF_JANELA_2.md)): as lacunas de motor L1–L19 e as 4 leis sistémicas (`06_LEIS_DAS_LACUNAS.md`).
- **J3** ([handoff](HANDOFF_JANELA_3.md)): fábrica por família, `BRIEFING_FAMILIA.md`; A–E, J: 6 aprovados. Fábrica 34.
- **J4** ([handoff](HANDOFF_JANELA_4.md)): F, G, H, I, K, L, M: **11 aprovados** — Filbert (refeito), Blender Soft
  (Wet); Hard Eraser, Kneaded Eraser, Halftone, Stars, Bokeh, Dodge, Burn, Color Glaze, Ballpoint (Digital).
  Fundidos/saídos: Acrylic Round, Rigger, Charcoal Pencil, Smudge Spiky, Hue/Overlay. **Fábrica 44** (Wet 28, Digital 16).
- **J5** ([handoff](HANDOFF_JANELA_5.md)): **Pastel Side aprovado** (`c6c99afab7`); curados, não aprovados: Square Hard
  `2e1212e7ec`, Water Brush/Dry-Lift/Blender Grainy `db170ae238`, Wet Liner/Wet Wash `4c0a2b7457`, Gouache Wash
  `953eb21e32`; reprovados Mop, Gouache Round; Gouache Flat fica, Wet Dry Brush não funde. Motor `Knob::WetOpen`
  (`73b8e11688`, `eb04c3440d`). Biblioteca (matriz, custo calmo, ordem) `2d6116b4c3`, `002c75267b`. Tutorial PDF
  `a054d6508f`. Fecho 1× (fmt, clippy, 19 576 testes, 0 falhas; 17 curas) até `b74b608d0e`. **Total: 18 aprovados, fábrica 45.**

## Decisões de produto (com o porquê)

1. Filbert/Flat Oily/Knife Smooth só voltam com o exame e o Rebelle ao lado; a lâmina do Rebelle é FIXA no eixo do
   preset (06 §1). J4: o Filbert voltou (aprovado); as lâminas continuam em estudo.
2. A matriz é TRIAGEM (d ≥ 0,596; série graduada: degrau ≥ 0,40; `matriz/escala.json`); o veredito é prancha + cega.
   J4: ferramentas (tiram/movem tinta) têm grupo e escala próprios (`matriz/escala_ferramenta.json`).
3. Os pré-existentes passam o mesmo exame: cura contra a origem, ou fundem/saem; «curado, não aprovado» fica.
4. Oráculo que manda: Rebelle; ArtRage/MyPaint/Krita onde ele não tem. GIMP fora.
5. A cauda do taper volta só no Digital. ⏳ AGUARDA o smoke do dono (pergunta no PDF).
6. Dual brush só no Digital e no Wet.
7. Modos = W3C em sRGB; Saturation/Linear Burn/Dodge do Rebelle a 10× não se perseguem.
8. Tecto de tamanho por preset (Max Size do Rebelle). ⏳ AGUARDA o smoke do dono (pergunta no PDF).
9. A rampa de pressão é a do DONO, não a do Rebelle (piso revogado 18/10).
10. 2H = HB com menos pigmento. 11. Round Pressure → Hard Round. 13. Canetas duras fundem pela matriz.
12. «Diferença visível» = confiança média/alta, ou repetida por dois revisores novos.
14. Grafites ficam no Spacing 0,1 + Continuous (dono 18/10).
15. A régua da rampa não reprova a entrada tardia do oráculo — FEITO na lei (J4, `399a387c6d`): abaixo do 1.º decil
    que o oráculo pinta, erro 0.
16. Foto nova do Rebelle vence número antigo gravado num portão (Square Hard).
17. O Dabber ficou como está (matriz final da J5: sem par < 0,596).
18. (J5) Gouache Flat fica (0,763 do Wet Flat; a cura partia 3 leis). Wet Dry Brush não funde (vizinho Airbrush 1,236).

## A FILA para uma missão futura (medida em [01_CATALOGO §3d](01_CATALOGO.md), por pincéis destravados)

1. Borda da mancha molhada esbatida ~10 px (anel 1,04 Rebelle × 1,09–1,25 nosso): Wet Liner, Wet Wash, Mop.
2. Pontas cortadas a direito num dab redondo: Square Hard, Soft Pastel, Charcoal, Wax Scrape.
3. O traço de 200 px do filme (`Knob::StrokeFilm`): Liner, Wash, Mop.
4. A mistura em luz linear (`exames/mistura.md`): muda TODOS os presets ⇒ decisão do dono + re-pinar portões num bloco.
5. Orla do secador nos lados (Dry/Lift) · misturador com grão perto da tinta (Blender Grainy) · o azul que cobre o
   vermelho molhado na guache · fio do arrasto (Smudge Finger) · lâmina que gira (Flat Oily/Knife, C38).
6. Partículas (Particles, Glitter) · ondulação do contorno (Oil Bristle) · mip das pontas de imagem (Foliage, Noise Grit).
Também: brecha do gate `arch_color_space_typed` (`&[u8]` em linha à parte); Ink Brush a vigiar (p95 12,54 ms a 200 px).

## Bancada: o essencial

- `exame.sh "<preset>" fixtures/<oráculo>/<prefixo>` · `--matriz` · `--custo`; SEMPRE `PH2D_EXAME_DIR=/tmp/…`.
- J4: a foto da bancada é a camada composta sobre o papel; ferramenta que não pinta ganha a base do seu meio
  (`PH2D_EXAME_BASE`, `PH2D_EXAME_ESCALA_BASE`); `PH2D_EXAME_CAMADA=1` pinta numa camada transparente;
  `exame.sh` corre o cargo da SUA árvore. Fotos novas do Rebelle: `fixtures/rebelle_mistura/`
  (`PH2D_REBELLE_REPETE_TROCA=1`: o 1.º traço depois de trocar de ferramenta saía ao tamanho anterior).
- Fábrica: `ReceitaFerramenta`/`FERRAMENTAS_WET` (ferramentas Wet: Blend/Wet/Dry); `ReceitaMeio::texturas` (imagens
  desenhadas por código); a largura e a miniatura das ferramentas medem-se sobre a base.
- Custo da J5: calmo (load 1–3,4), 135 medições, [exames/biblioteca.md](exames/biblioteca.md).
