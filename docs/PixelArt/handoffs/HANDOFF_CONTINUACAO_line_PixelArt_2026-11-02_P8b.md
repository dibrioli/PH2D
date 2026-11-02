# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, P8b (smoke da cena 19 e o que o dono pediu) — 2026-11-02

> Leitor: o agente que assume a linha na janela seguinte. Anterior: [P8a](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-02_P8a.md)
> (núcleo + UI dos menus; o que lá existe não se reconstrói). Plano: [P8_plano.md](../P8_plano.md).
> Merge-base `3df5894489`; tudo comitado em `line/PixelArt`, árvore limpa. P7 continua aprovada e NÃO integrada;
> P7+P8 integram só por ordem do dono. Shas: `git log --oneline b63e7758ba..HEAD` (último `4a18fc8688`).

## 1. Feito nesta janela (P8a → P8b)

- **Convolution Matrix**: núcleo `filter/convolution.rs` (`clamp(trunc0(Σw·v/div)+bias)`, borda replicada, Tiled = volta). 37 de 39 kernels em `filter/kernels.rs`, GERADOS por `docs/PixelArt/ferramentas/fit_convolucao.py` (+ `fit_convolucao_r3.py`), 0 células fora nas rodadas 1–3; `edges-find*` = 128 constante como o Trial; fora só `outline-transparent-layer-(1)/(2)` (alfa não linear). Canais por kernel medidos (`A5_kernel_channels`: R G B sem A em brightness/contrast/negative/misc-contour/marmolize/rock/rock-edges; RGBA nos outros 30). Lei da cor junto ao transparente: a célula de alfa 0 sai da soma e o peso dela sai do divisor (0→1); alfa linear só com A ligado (`fit_convolucao_alfa.py`, 0 fora em 20 casos). Gate `convolution_oracle` (6). Diálogo F9 em 3 colunas, largura dupla; escolher kernel põe os canais dele; os canais do Convolution são guardados à parte dos outros diálogos.
- **Brightness/Contrast** `⌊y·255,5⌋` fecha os 43 casos da rodada 2 (gate cobre 52 LUTs).
- **Gradient ▸ Palette ramp** no núcleo (`gradient.rs` `ramp_color_at`/`palette_ramp`, gate `gradient_ramp`). Ferramentas Gradient (Shift+G, grupo do balde, matriz de estado próprio, Linear/Radial, Palette ramp) e Blur/Jumble (R alterna, grupo novo): `ph2d-app-pixel/src/degrade.rs`; testes `shells/pixel-scenes/tests/it/degrade.rs`.
- **Hue/Saturation ▸ Alpha** (`a'=⌊a·(100+v)/100⌋`, rodada 3 B4); alvo Selected|All nos diálogos (todos os quadros × camadas que se pintam, UM passo); Outline com fundo de cor conferido (B3); Invert e Gradient em Indexed FUNCIONAM (recusa medida de copiar as falhas B1/B2 do Trial) — `filter_indexed.rs`, `alvo_todos.rs`.
- **Texto**: núcleo `ph2d-pixel-text` — dropout SCANTYPE (sem stubs); AA = cobertura FreeType gray (subdivisão por curvatura 1/16, `⌊área·256⌋`) + LUT de tom `TRIAL_TEXT_TONE`; corte da caixa por baixo (`box_bottom_cut`, 39 casos); `carimbo.rs` (Fonte, over, font_list, Editing). Ferramenta T + Edit ▸ Insert Text, caixa no canvas (moldura + cursor), opções no painel (fonte Previous/Next, tamanho, B, I, Antialias, Hinting), fontes do sistema via `ph2d_system_fonts::face_bytes` (nova), respeita a selecção. App: `ph2d-app-pixel/src/texto.rs`; testes `shells/pixel-scenes/tests/it/texto.rs`.
- **Rodada 3 (+3b) no Trial**: `crates/ph2d-pixel-core/tests/fixtures/oracle_trial_round3.json` (A1–A5, B1–B4, C, text, text_b). Método novo: alfa lido pelo clipboard (Ctrl+A/C, PyQt6), documento novo por caso, sentinela. No Trial a caixa de texto é de UMA linha.
- **Tecto** `ph2d-app-pixel` curado MOVENDO (testes para `shells/pixel-scenes`, carimbo para `ph2d-pixel-text`): 13 306 / 13 424.
- **Cena 19** (`PH2D_PIXEL_SMOKE=19`, `max_level` 19), sonda `cena19` em `shells/pixel-scenes`, fotos `docs/PixelArt/handoffs/smoke_P8_cena19_*_2026-11-02.png`; diálogos de filtro posicionados para tapar o mínimo do sprite (8 posições).

## 2. Smoke do dono (02/11, cena 19) — o que reportou, diagnóstico, ordem de ataque (um bloco, §0.10)

1. **«Adjustments não funciona nenhum» e «Outline não funciona».** Diagnóstico provável: a CENA ensina errado — a selecção pronta está sobre a zona branca vazia e os filtros agem só dentro dela; o Outline precisa de transparência e a camada é «Background» opaca. CURA: refazer a cena 19 com o boneco numa camada TRANSPARENTE (não Background) e SEM selecção pronta; e MESMO ASSIM conferir pela app real (sonda) cada ajuste (B/C, Hue/Sat, Curve) e o Outline — se algum não agir sem selecção, é defeito de verdade.
2. **«Eraser pinta com a cor secundária».** Hipótese: lei do Aseprite na camada Background (opaca) — a borracha pinta a cor de fundo; numa camada normal apaga. Confirmar no código que é isso e não defeito; a cena nova (camada transparente) mostra a borracha a apagar; explicar ao dono.
3. **Gradient «comporta-se como o lápis» depois do 1.º gradiente.** A região é a do balde a partir de `from`; sobre um gradiente cada célula é diferente ⇒ região de 1 célula. O dono QUER SOBREPOR. Decisão: a região do Gradient passa a ser a SELECÇÃO, ou o cel inteiro sem selecção (deixa de usar a contiguidade do balde); as opções do balde saem do painel do Gradient. Gates `gradient_oracle` (o Trial mediu sobre canvas branco = cel inteiro; devem continuar verdes) e `degrade::`.
4. **Texto: mover e rodar antes de gravar.** Enter transforma o texto numa MOLDURA SOLTA (a da P6/P7, `solta.rs`/moldura afim) que se arrasta/roda/escala; 2.º Enter (ou clicar fora) grava UM passo; Esc cancela.
5. **Fontes.** (a) Fonte de pixel tem tamanho NATIVO (múltiplos inteiros da altura da linha; `TextStyle.px` já documenta) e fontes do sistema têm mínimo legível: o campo tamanho salta para o mínimo/múltiplo válido ao trocar de fonte. (b) DROPDOWN de fontes ACIMA dos botões, com PRÉ-VISUALIZAÇÃO (nome desenhado na própria fonte), as mais usadas em pixel art no TOPO (fábrica Cozette/Tom Thumb/Unscii + as do sistema populares em pixel art, se instaladas), depois as restantes do sistema. Exige lista dinâmica com rolagem (o dropdown actual regista opções no arranque).
6. **Pendente do dono (perguntado, sem resposta):** Convolution Matrix em 3 colunas largas (tapa ~metade do sprite em 1930×1012) OU lista estreita com rolagem como o Aseprite. Não mexer antes da resposta.

## 3. Abertos técnicos (não do smoke)

- `edges-find*`: alfa nunca medido (a lei dá alfa 128 numa imagem opaca com A ligado) — uma rodada curta pelo clipboard.
- Divisor 0 na cor junto ao transparente: usa 1 (escolha, não medido).
- Texto mono: 1–2 células por caso a 14/15/24/32 px e Sans 8 «g» = empates centro-na-curva contra a FreeType (não é o corte). FreeType é FTL/GPL dual; a FTL é permissiva com crédito — avaliar porte do rasterizador mono pela triagem de licença (§0.9) antes de mais tentativas por especificação.
- Fecho da linha (DIRETRIZ §1.5.9): mutações (P8_plano §6), fmt/clippy das crates tocadas 1×, `agent-loop-profile.sh`, handoff de INTEGRAÇÃO.

## 4. Avisos

- Hook `prazo-do-agente` do main com defeito de caminho relativo (curado só nesta linha, `288d03792f`): agentes com cwd na RAIZ da worktree.
- Agentes em paralelo na mesma worktree: `git commit -- <paths>` leva o ficheiro INTEIRO (aconteceu: `796645460f` levou edições alheias de `selection.rs`) — dê a cada agente ficheiros disjuntos.
