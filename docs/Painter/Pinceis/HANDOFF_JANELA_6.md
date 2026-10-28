# HANDOFF da JANELA 6 — Missão Pincéis 2 (28/10, aviso de 300 mil)

> Estado vivo: [`ESTADO.md`](ESTADO.md) (SHAs e números por item). Fila: [`JANELA_6.md`](JANELA_6.md). Commits: `git log --oneline ca3fb8d2b4..HEAD` (55). Ramo `line/PainterWatercolor`; nada integrado, sem push.

## 1. Feito

**Motor** (cada knob com gate + mutação):
- `wetOpenEdge` (borda da mancha molhada esbate) e `strokeFilmDome` (miolo carregado do filme largo) — `8394f62eed`; a pena recua metade — `5f6ae47051`.
- `pontas_rectas`/`pontas_alem` (Flat ends / Ends beyond) — `10eee50b0d`, `fd73b349ca`.
- `strokeFilmEnds` (tampas do filme esbatem) — `5cd611289a`; `pontas_esbate` (esbatem ao comprido) — `5b8c03e2ca`.
- `dryerLevel` — `48da05baa5`. `blendReach` — `eac8182beb`. `smear_fio` — `3f6377fcec`. `wetCover` — `600152c535`.
- `DabJitter::subpixel_area` — `f9fbbe7c26` (+ `9fba4165d8`: o grão de 1 px sobrevive ao óleo). `bristleLaneSizeExp` — `a7cd157000`.
- Óleo 2× mais rápido ao bit (`d94e19a57e`); bancada fotografa o óleo pelo caminho do traço (`78fb8c7c77`); Oil/* de fábrica com Real Oil (`e314e58cec`).
- Gate novo `a_lamina_de_cada_preset_e_a_do_oraculo`; sem_glaze verde (`e42004f3e8`).

**Aprovados novos:** Soft Pastel (cega r2, 6 linhas). **Mantidos** com Real Oil (cega de 9, Rebelle achado 11/27, acaso 9): Filbert, Knife Textured, Knife Dry, Knife Smooth 3, Knife Oily. Total 26 aprovados.

**Curados NÃO aprovados** (porquê):
- Square Hard: faixa sólida / pontas arredondadas · Wet Liner: chapado, tampa dura · Wet Wash: contorno fino escuro, cápsula, quadrados nos cruzamentos.
- Dry/Lift: falta a linha escura 1,3 · Blender Grainy: grumos grossos rasgados · Smudge Finger: fio pálido, língua oca.
- Gouache Wash/Dry: sem fiapos, tampa redonda, 200 px chapado · Spray Can: faltam salpicos escuros e discos translúcidos · K-1 Clouds.
- Rough Sketch (B-7): o revisor tomou o NOSSO pelo de fora em todas as linhas (discos nítidos onde o MyPaint tem grão de pontos e pontas rectas) · Optical Wash (I-12): achado em todas as linhas, contorno forte e contínuo, o Krita tem franja fraca irregular.
- Mop: REPROVADO. C-4 e C-6 fundem com Hard Round / Pixel 1px.

## 2. EM CURSO agora

- **Agente do óleo:** grão fino derretido pelo óleo em K. Stain (162), K. Grainy (226), Dabber 4 (322), Metallic Flat (326); gate `cada_preset_parece_a_sua_midia`; tabela de largura Oil.
- **Canetas/marcadores** (C-9, C-15, E-3, E-4) e **aguarela/texturas** (I-9, I-10, I-11, K-9, F-10) em `EM_ESTUDO_J6`.

## 3. PRÓXIMO (ordem)

1. Largura do traço fino r 1,5–1,8 pela cobertura útil ⇒ regerar Gouache Flat com `wetCover` 1 (régua 404→424, `cc49c5616a`) e Wet Flat.
2. Lote ÚNICO de promoção dos aprovados à fábrica (uma regeração).
3. Biblioteca inteira de comparação. 4. Fecho da linha (DIRETRIZ §1.5.9; suíte completa é do integrador). 5. Tutorial em PDF.

## 4. Armadilhas

- A biblioteca `j6_wet` está INCOMPLETA (28/66): `d` medidos contra ela são suspeitos; a boa é `/tmp/claude-1000/j6_digital/biblioteca`.
- Fábrica medida 64 PNG × 52 declarados — por conferir no passo da biblioteca (`the_factory_presets_are_counted`).
- Paralelismo + /tmp órfão causaram OOM (11,5 GB de targets apagados); use `ph2d-run.sh`.
- Brecha `arch_color_space_typed`; Ink Brush a vigiar (p95 12,54 ms).

## 5. Leis e achados

- O Rebelle NÃO gira a lâmina: plano fixo no eixo do preset ([`J6_lamina.md`](J6_lamina.md)); inércia/memória refutadas.
- O fio do arrasto era amostragem por ponto (re-amostrar por área; linhas vazias 35/118 → 0).
- O vermelho do sem_glaze vinha de `5291edbc75` (25/10, lei deliberada do tubo), não do motor novo.
- Charcoal/Wax Scrape: o Rebelle acaba as pontas REDONDAS (0,50/0,28) ⇒ lacuna à parte, não é `pontas_rectas`.
- Gouache Wash não tem riscas no Rebelle; só o Dry ganha com `bristleLaneSizeExp`.
- Sem oráculo (fora): Particles/Glitter, M-7, C-13, C-14, D-2, D-4, G-13, J-4, M-5.
- Motor pendente: B-6, L-5, CAP-19 (C-11, G-9, G-11, I-13), L17, L15, L20, L10, lâmina que gira, partículas, mip das pontas.
- Mistura em luz linear: FORA (dono). Pendentes do dono: decisões 5 e 8, no PDF.

Docs: [digital](exames/digital_j6.md) · [guache](exames/guache.md) · [texturas](exames/texturas.md) · [pontas](J6_pontas_rectas.md) · [bancada](04_BANCADA.md).
