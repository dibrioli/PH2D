# ESTADO da missão Pincéis (a FILA viva; ≤ 8 KB)

> Toda janela lê isto primeiro e actualiza-o antes de sair. Contrato: [`00_MISSAO.md`](00_MISSAO.md). Custo:
> [`REGRAS_DE_CUSTO.md`](REGRAS_DE_CUSTO.md) (ordem do dono 21/10 — valem para TODO agente). Última escrita: JANELA 4,
> 2026-10-21. Detalhe por janela: `HANDOFF_JANELA_<n>.md`; por família: `exames/<família>.md`.

**Quem é quem (20/10):** J2 `ph2d-d2` · J3 `ph2d-94` · J4 `ph2d-d1` · J5 `ph2d-fa` (confirme por `ListAgents`).

## Feito

- **J1** ([handoff](HANDOFF_JANELA_1.md)): bateria, oráculos fotografados, catálogo (112), bancada, `03_MOTORES.md`.
- **J2** ([handoff](HANDOFF_JANELA_2.md)): as lacunas de motor L1–L19 e as 4 leis sistémicas (`06_LEIS_DAS_LACUNAS.md`).
- **J3** ([handoff](HANDOFF_JANELA_3.md)): fábrica por família, `BRIEFING_FAMILIA.md`; A–E, J: 6 aprovados. Fábrica 34.
- **J4** ([handoff](HANDOFF_JANELA_4.md)): F, G, H, I, K, L, M: **11 aprovados** — Filbert (refeito), Blender Soft
  (Wet); Hard Eraser, Kneaded Eraser, Halftone, Stars, Bokeh, Dodge, Burn, Color Glaze, Ballpoint (Digital).
  Fundidos/saídos: Acrylic Round, Rigger, Charcoal Pencil, Smudge Spiky, Hue/Overlay. **Fábrica 44** (Wet 28, Digital 16).

## Decisões de produto (com o porquê)

1. Filbert/Flat Oily/Knife Smooth só voltam com o exame e o Rebelle ao lado; a lâmina do Rebelle é FIXA no eixo do
   preset (06 §1). J4: o Filbert voltou (aprovado); as lâminas continuam em estudo.
2. A matriz é TRIAGEM (d ≥ 0,596; série graduada: degrau ≥ 0,40; `matriz/escala.json`); o veredito é prancha + cega.
   J4: ferramentas (tiram/movem tinta) têm grupo e escala próprios (`matriz/escala_ferramenta.json`).
3. Os pré-existentes passam o mesmo exame: cura contra a origem, ou fundem/saem; «curado, não aprovado» fica.
4. Oráculo que manda: Rebelle; ArtRage/MyPaint/Krita onde ele não tem. GIMP fora.
5. A cauda do taper volta só no Digital. ⚠️ Perguntar no smoke do dono.
6. Dual brush só no Digital e no Wet.
7. Modos = W3C em sRGB; Saturation/Linear Burn/Dodge do Rebelle a 10× não se perseguem.
8. Tecto de tamanho por preset (Max Size do Rebelle). ⚠️ Smoke.
9. A rampa de pressão é a do DONO, não a do Rebelle (piso revogado 18/10).
10. 2H = HB com menos pigmento. 11. Round Pressure → Hard Round. 13. Canetas duras fundem pela matriz.
12. «Diferença visível» = confiança média/alta, ou repetida por dois revisores novos.
14. Grafites ficam no Spacing 0,1 + Continuous (dono 18/10).
15. A régua da rampa não reprova a entrada tardia do oráculo — FEITO na lei (J4, `399a387c6d`): abaixo do 1.º decil
    que o oráculo pinta, erro 0.
16. Foto nova do Rebelle vence número antigo gravado num portão (Square Hard).
17. O Dabber fica como está até a matriz final da J5.

## A FILA (por ordem) — para a JANELA 5, com as regras de custo

**Motor (o que destrava mais pincéis por recurso; medição em cada relatório):**
1. **Molhado × molhado no Wet** — o azul do Rebelle abre uma mancha redonda e escura no vermelho molhado e o empurra
   para os lados; o nosso dá quadrado limpo ou lente pálida (Gouache Round, Wet Wash, Mop, Wet Liner, Flat Bristle).
2. **O traço de 200 px do filme** (`Knob::StrokeFilm`) esbate as pontas em degradê e carrega o miolo (Liner, Wash, Mop).
3. **Pontas cortadas a direito num dab redondo** — cortar o traço no início/fim, não o dab (Soft Pastel, Charcoal,
   Wax Scrape); o traço de 5 px tracejado do pastel.
4. **A mistura em luz linear** (o vermelho cobre mais que o azul é o render em sRGB, `exames/mistura.md`): muda TODOS
   os presets ⇒ decisão + re-pinar portões num só bloco.
5. A ondulação lenta do contorno (Oil Bristle) · o tapete das cerdas em px fixos (Gouache Dry) · o advecção do smudge
   (Smudge Finger) · mip pyramid das pontas de imagem (Foliage, Noise Grit).
**Exames por fazer (motor já pronto, sem revisão cega):** Square Hard (decisão 16) · Blender Grainy · Dry/Lift ·
Water Brush 2.ª ronda (`wetLift 0,2`) · Gouache Wash 2.ª ronda (a mais perto de passar) · Pastel Side (valores do pó
medidos) · Glow (o caroço não pode crescer a 200 px).
**Decisões pendentes:** Gouache Flat (aplicar a cura que a separa do Wet Flat, ou fundir) · Wet Dry Brush → Wet Flat.
**Depois:** o resto da J5 (`JANELA_5.md`): matriz final, fecho batched, handoff, smoke/tutorial.
**Fotos a refazer:** `gouache_gouache_round_2_sobre_{A,B}`, `gouache_gouache_liner_2_gestos_A` (duplicadas).

## Bancada: o essencial

- `exame.sh "<preset>" fixtures/<oráculo>/<prefixo>` · `--matriz` · `--custo`; SEMPRE `PH2D_EXAME_DIR=/tmp/…`.
- J4: a foto da bancada é a camada composta sobre o papel; ferramenta que não pinta ganha a base do seu meio
  (`PH2D_EXAME_BASE`, `PH2D_EXAME_ESCALA_BASE`); `PH2D_EXAME_CAMADA=1` pinta numa camada transparente;
  `exame.sh` corre o cargo da SUA árvore. Fotos novas do Rebelle: `fixtures/rebelle_mistura/`
  (`PH2D_REBELLE_REPETE_TROCA=1`: o 1.º traço depois de trocar de ferramenta saía ao tamanho anterior).
- Fábrica: `ReceitaFerramenta`/`FERRAMENTAS_WET` (ferramentas Wet: Blend/Wet/Dry); `ReceitaMeio::texturas` (imagens
  desenhadas por código); a largura e a miniatura das ferramentas medem-se sobre a base.
- ⚠️ Custo: TODAS as medições da J4 com load 8–68 — a J5 re-mede calmo os 11 novos.
- ⚠️ Relatórios acima do tecto de 10 KB: `pastel_carvao.md`, `efeitos.md`, `mistura.md` (cortar na J5).
