# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P2 (Cor, paleta, inks, Indexed, Color Mode com dither) — 2026-10-25

> Leitor: a próxima LLM que assume a linha numa janela nova. A onda P2 está **fechada e com o smoke do
> dono APROVADO (25/10: «Smoke OK»)**; a **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada
> integrado, nada pushado. Lei do módulo: [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md)
> · [plano](../02_plano_modulo.md) · [handoff P1](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P1.md) · [P0b](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0b.md) · [P0](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (tem de dizer `line/PixelArt`).
- Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`, merge-base `c3662cd8e4`.
- **42 commits** na linha (`git log --oneline c3662cd8e4..HEAD`); HEAD `c77d01ce4a` (antes deste handoff).
- Desta onda (antigo → novo): `53758d077c` wip (vetores do MVP em JSON, oráculos, cinza, paleta, ficheiros de paleta, inks, ops) · `3b8f8f7f16` wip (conversão de modo, testes de lei, testes de ficheiro) · `08e68d802d` (vocabulário do chrome + diálogo Color Mode) · `e22d5e6115` (família: tinta, paleta, Replace Color, Color Mode, cena 3) · `423138b3a1` (painel PIXEL) · `76cbc928ad` (nome da paleta vem do ficheiro, HR-15) · `9c35b60545` (RGB→Indexed contra o Trial) · `98fd8eca36` (dica do Shading) · `beaceb04c4` (fotos da cena 3) · `b80cff07bf` (smoke do dono: Color Mode) · `c77d01ce4a` (cenas de smoke 4–6).
- `git diff --stat 2d9d59b658..HEAD`: 114 ficheiros, +5931/−226.

## 2. Estado do módulo

Igual ao do [P1 §2](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P1.md) (14 ferramentas, pixel-perfect, simetria, Tiled, formato `PH2DPIXEL`), com: **tinta** por pincel (Simple, Alpha Compositing, Copy Alpha+Color, Lock Alpha, Shading), **paleta** completa (seleção, edição, tamanho, mover, ordenar, usadas, nova do sprite, ler/gravar `.gpl/.pal/.hex/.act`, DB32 de fábrica), **Replace Color**, **Color Mode** RGBA↔Grayscale↔Indexed com dither. Smoke `PH2D_PIXEL_SMOKE=1..6`.

## 3. O que a P2 fez

**Passo 1, como o dono pediu: vetores do MVP em JSON ANTES do código.** `docs/PixelArt/ferramentas/extrai_cor_mvp.mjs` → `crates/ph2d-pixel-core/tests/fixtures/mvp_color.json` (~337 KB; famílias color/OKLab+rampas, palette, padrões de dither, quantize, ramp_quantize, overlay_color, cycling). O MVP não tem `palette.test.js`: a paleta vive em `color.test.js`. As famílias da P13 (rampas OKLCH, ramp_quantize, overlay_color, cycling) estão no fixture mas **NÃO implementadas** (o plano põe-nas na P13). Defeitos do MVP medidos e **não copiados**: `parsePalette` fareja JASC-PAL como lista hex; a lista hex aceita tokens de 3 dígitos soltos (`255` → `#225555`).

**Oráculos (sem licença do Aseprite — o dono recusou comprá-la por agora, 25/10: «tente fazer o melhor com outras referências abertas»; memória `project_pixel_art_module_standalone_aseprite_spine_2026_10_25.md` actualizada):**
- **LibreSprite 1.3** (GPL-2, só CORRIDO) dirigido pelo menu em Xvfb (`ferramentas/oraculo_libresprite.py`): a API JS não converte (`colorMode` sem setter, `app.launch` é no-op). Leis medidas e **REJEITADAS** (linhagem velha do Aseprite): cinza = máximo do canal; nearest = métrica ponderada por luma ((0,0,255)→quase preto); índice 0 reservado.
- **Aseprite Trial 1.3.18.6** como oráculo de OBSERVAÇÃO: converte na UI em Xvfb `:78`, captura a 800 %, lê o centro de cada bloco de 16 px (`ferramentas/trial_le_tela.py`; controlo RGB antes/depois de Ctrl+Z: 0 diferenças). Referência do diálogo: Sprite ▸ Color Mode ▸ More Options.
- ⛔ Uma GUI lançada com `WAYLAND_DISPLAY` aberta abriu no **ECRÃ REAL do dono** uma vez (morta). Sempre `env -u WAYLAND_DISPLAY SDL_VIDEODRIVER=x11 DISPLAY=:77`.
- ⛔ `kill $(pgrep -f '<padrão>')` matou o próprio shell (exit 144): mate por PID.
- Os scripts `ferramentas/trial_*` ainda têm caminhos do scratchpad fixos no código.

**Leis medidas** (fixtures `oracle_trial_color_mode.json`, `oracle_trial_dither.json`, 143 casos):
- **Cinza**: Luminance = (2126R+7152G+722B)/10000 truncado; HSV = max; HSL = (max+min)/2 truncado — 0/216 erros cada (o núcleo velho Rec.601 errava 204/216; agora `color::luma`, e `Sprite::value_of_rgba` em Gray usa-o).
- **Best fit Default(Euclidean) do Trial NÃO é o vizinho mais próximo exacto** (52–114 px/caso diferem; os melhores pesos inteiros (3,11,1) ainda erram 6–9/216) → **recusa registada**. A nossa lei = vizinho exacto (Euclides² RGBA, empate → menor índice), que coincide com o Trial em Octree + best fit RGB (0 erros).
- Com PNG RGBA (camada não-Background) o índice 0 é transparente reservado; no Background é uma cor (a nossa lei coincide).
- **Sem dither e Floyd–Steinberg: 7944/7944 células** (FS = serpentina, v=clamp(c+erro), cada parcela trunc(e·w/16)).
- **Ordered/Old**: melhor modelo, não exacto — par a = nearest(c), b = nearest(2c−a); ordered: b onde t+1,5 < n²·s (s = fracção de luma); old: b onde t/n² < |c−a|/|b−a|; Old 2×2 usa a regra ordered; Bayer lido TRANSPOSTO M[x][y]. Concordância ordered 92,5/86,6/84,2 %, old 95,7/92,3/93,3 % (2/4/8); pisos fixados no teste `rgb_to_indexed_matches_the_aseprite_trial`.

**Núcleo `ph2d-pixel-core`:** `color.rs` (ToGray, luma) · `convert.rs` (convert → Whole, Dither, Bayer, Mapper com memo; defeito achado por teste: sentinela do memo `u32::MAX` = branco opaco mapeava branco→índice 0) · `ink.rs` (Simple/AlphaCompositing/CopyAlphaColor/LockAlpha/Shading pelos docs do Aseprite; sem oráculo executável) · `palette.rs` (MAX_COLORS 256 = byte da célula, move/invert/is_permutation, find_duplicates RGBA, hex, `SortKey` 8 + `sort_permutation` estável) · `op.rs` ops novas `Palette{colors,transparent}`, `Remap{perm}` (lei do MVP: RGBA composto byte-idêntico; undo restaura índices), `Whole(Box<Whole>)` (modo+paleta+transparente+cels; validado). `Paint.ink`, `Stroke::with_shading(ramp, dir)`; ⚠️ **`Stroke::value_at(&sprite, x, y)` mudou de assinatura** (a tinta aplica-se UMA vez por traço sobre o valor do documento). `compose::over` passou a `pub(crate)`.

**`ph2d-pixel-io`:** `palette_file.rs` lê/escreve `.gpl` (com `Channels: RGBA` do Aseprite), `.pal` JASC + RIFF, `.hex`, `.act` (768/772 com contagem+transparente), lê JSON do Lospec; dependência `serde_json` nova; nomes vazios (o rótulo é da UI, HR-15).

**Vocabulário `ph2d-editor-core`:** `pixel_panel.rs` (`PixelRequest`, `PixelOptions.ink`, `PixelInk`), `ids/chrome/pixel.rs`, `types_menu.rs`, `menu_rows.rs`, `pointer_down_menus.rs`, `dispatch/key.rs` (Enter/Esc do Color Mode), `chrome_ops.rs`, `color_mode_ops.rs` (novo), `chrome/mod.rs` regenerado por `ph2d-chrome-sync`, `chrome/pixel_color_mode.rs` (novo), `context_menu_dialogs.rs`, `context_menu_overlay.rs`, `context_menu_color_mode.rs` (novo); campo de estado `new_sprite` → `pixel: PixelDialogs` em `state/mod.rs` + `store_core.rs` (**`state/mod.rs` está no tecto de 700 linhas**).

**Família `ph2d-app-pixel` (UI):** linha de tinta para ferramentas de pintar (borracha sempre Simple; rampa do Shading = seleção da paleta na ordem da paleta, esquerdo +1 / direito −1); seleção da paleta (clique = frente, direito = fundo, Shift intervalo, Ctrl alterna, anel de acento); editar cor pelo seletor partilhado (arrastar = um passo de undo); tamanho da paleta; mover esq/dir; ordenar por (8 chaves, asc/desc → `Remap`); selecionar cores usadas (Indexed); nova paleta do sprite; ler/gravar paleta por diálogo de ficheiro do host (`ph2d-app-host`); carregar paleta de fábrica (DB32); **diálogo Color Mode** (novo `ContextMenuKind` modelado no New Sprite; OK/Enter = um passo de undo; lembra as escolhas; sem atalho porque o Aseprite não tem); **Replace Color como SECÇÃO DO PAINEL** (tolerância + «Replace foreground with background»), não diálogo; dica do rodapé do Shading diz o que cada botão faz. `input.rs` partido → `input_roda.rs` (tecto de LOC). Deps novas: `ph2d-pixel-io` + `rfd` (Cargo.lock).

**Outros:** `ph2d-i18n/src/pixel.rs`; `ph2d-panel-registry-init/tests` (2 literais de `PixelPanelInfo`); shell `render_loop/fase_pixel_panel.rs` (`panel::frame` recebe `&mut g.toasts`). Painel PIXEL (`ph2d-panel-pixel-tools`) com a linha de tinta e a secção Replace Color.

## 4. Smoke do dono — APROVADO em 25/10

Fotos: [`smoke_P2_2026-10-25.png`](smoke_P2_2026-10-25.png) · [`smoke_P2_dono_2026-10-25.png`](smoke_P2_dono_2026-10-25.png) (cena 3, HOME=/home/enio; `~/.ph2d/layout.txt` copiado antes, sha256 igual antes/depois `f7db4c91…`) · [`smoke_P2_cena4_2026-10-25.png`](smoke_P2_cena4_2026-10-25.png) · [`smoke_P2_colormode_2026-10-25.png`](smoke_P2_colormode_2026-10-25.png) · [`smoke_P2_colormode_dono_2026-10-25.png`](smoke_P2_colormode_dono_2026-10-25.png) · [`smoke_P2_cena6_ordered4_2026-10-25.png`](smoke_P2_cena6_ordered4_2026-10-25.png). O chip «Vector» no topo do layout do dono é pré-existente (igual na foto aprovada da P1).

**Cena 3 (tinta e paleta)**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=3 ./target/smoke/ph2d-host-desktop`
2. Abre um sprite Indexed com céu e paleta DB32. Painel **Pixel**: linha de tinta, paleta, Replace Color.
3. Escolha a tinta **Shading** e pinte: botão esquerdo = próxima cor selecionada, direito = anterior; o céu não muda. Em **Color Mode…** escolha RGBA: a imagem fica igual. `Ctrl+Z` desfaz. **Sort by** descendente: a imagem fica igual. Editar uma cor da paleta recolore o desenho.
4. Deu errado se: o Shading pintar com a cor de fundo, o Sort mudar a imagem, o RGBA mudar a imagem.

**Cena 4 (Color Mode que se VÊ)**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. Abre um RGBA 96×64 com degradês (céu, esfera iluminada, arco-íris). Menu Sprite ▸ **Color Mode…**.
3. **Grayscale** fica cinza; **Indexed** com **No Dithering** dá faixas chapadas, **Floyd–Steinberg** granulado, **Ordered 2×2/4×4/8×8** padrões regulares diferentes entre si, **Old** diferente (Old 2×2 = Ordered 2×2, medido). `Ctrl+Z` após cada um volta ao original. Níveis 5 e 6: diálogo aberto / depois do Ordered 4×4 (só para foto).
4. Deu errado se: algum botão do dither mostrar «Ordere…» cortado, o modo actual aparecer apagado em vez de escolhido, ou todos os dithers saírem iguais.

Reports do dono: (1) botões do dither cortados em «Ordere…» idênticos e modo actual apagado → corrigido em `b80cff07bf` (modo actual ESCOLHIDO; botões reagrupados No Dithering|Floyd-Steinberg / Ordered 2×2 4×4 8×8 / Old 2×2 4×4 8×8; gate de não-truncagem, RED primeiro). (2) «não percebi diferença ao trocar de modo» → causa: a cena 3 é Indexed só com cores da paleta, então RGBA↔Indexed é byte-idêntico e todo dither é no-op (cena que não ensina nada) → cena 4 (RGBA em degradês; Game Boy de 4 cores medido: ordered 2/4/8 diferiam só 3 %/0 % → rejeitado) + teste de que a cena 4 mostra cada diferença. ⛔ **Lição:** uma cena de smoke tem de ser medida contra a pergunta «o que muda quando o dono clica?».

## 5. Superfície de colisão NOVA desta onda

Soma-se às de [P1 §5](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P1.md), [P0b §5](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0b.md) e [P0 §3](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md). Contadores ECS/schema/`MODULE_TRUTHS` não mudaram.

- `ph2d-editor-core`: ficheiros listados em §3 (Vocabulário); `chrome/mod.rs` é **gerado** — regenere com `ph2d-chrome-sync`, não funda à mão; `state/mod.rs` no tecto de 700 linhas (campo `new_sprite` → `pixel: PixelDialogs`).
- `ph2d-i18n/src/pixel.rs`; `ph2d-panel-registry-init/tests` (2 literais de `PixelPanelInfo`).
- Shell: `render_loop/fase_pixel_panel.rs`.
- `ph2d-app-pixel`: deps `ph2d-pixel-io` + `rfd` (Cargo.lock); `input.rs` → `input_roda.rs`.
- `ph2d-pixel-core`: `Stroke::value_at` mudou de assinatura; `compose::over` `pub(crate)`.
- Merge pode partir: outra linha que mexa em `state/mod.rs` (tecto), `ContextMenuKind`/diálogos, `chrome/mod.rs`, ou com literal de `PixelPanelInfo`; Cargo.lock (`rfd`, `serde_json`).

## 6. Prova de fecho (por nome, tudo verde)

- `ph2d-pixel-core`: `color_parity` + `color_laws` 14. `ph2d-pixel-io`: 5.
- `ph2d-app-pixel`: `paleta::` 12 (+ testes das cenas 4/5/6).
- `ph2d-panel-pixel-tools`: `seam_pixel_cor` (incl. `the_hint_tells_what_the_shading_ink_does_with_each_button`), `seam`.
- `ph2d-editor-core`: `the_pixel_color_mode_dialog_is_alive` + gate de não-truncagem (RED primeiro).
- `ph2d-i18n`: `a_tabela_inglesa`, `a_fronteira_dos_motores`.
- Clippy `-D warnings` nas crates tocadas e `cargo fmt` limpos. Binário de smoke `target/smoke/ph2d-host-desktop` (2.ª compilação sem `Compiling`).

## 7. ABERTO

- **Não construído nesta onda (itens P2 abertos):** arrastar-para-reordenar swatches; Tint/Shade/Tone; Spectrum; rodas de cor RGB/RYB e Normal Map; presets de paleta (Save as Preset / Save as Default). Edit Palette: atalho `A` no 1.3.18 (o plano dizia F4).
- Lei exacta de Ordered/Old **não ajustada** (octree/best-fit Default do Trial excluídos; sugeridas mais sondas em 0–32 e 224–255).
- Paleta em `.ase` = P9. Famílias OKLCH/ramp_quantize/overlay_color/cycling = P13.
- **Mutação** no fecho da linha: lei do remap · FS serpentina/trunc · reserva do transparente (exceto Background) · sentinela do memo · Shading uma vez por traço · validação do `Whole` · fórmulas de cinza; mais as antigas ([P1 §7](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P1.md), [P0b §7](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0b.md), [P0 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md)).
- Pré-visualização da pegada do pincel (P1) continua em falta. Ink Shading/Lock Alpha sem oráculo executável.
- Licença do Aseprite: **fechada por ora** (dono recusou).
- Fecho da linha pendente: `/pd-linha-fechar`, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`; a suíte completa é do integrador.

## 8. Próximo item

**P3 — Camadas** (plano §5/§6: P1 ∥ P2 ∥ P3, depois P4): Background, normal, grupo, referência; visível/bloqueada/solo; opacidade; 19 modos de mistura no shader com paridade na CPU; renomear, duplicar, Merge Down, Flatten, Layer via Copy/Cut, Paste as New Layer/Reference. Gates: cada mistura contra tabela de fixture (Pillow/oráculo); grupo pass-through vs isolado. Smoke 5 na numeração do plano, mas **os níveis no código continuam do 7** (`PH2D_PIXEL_SMOKE` vai até 6).

Primeiro passo da onda: extrair os vetores do MVP (`docs/Pixel Art/app/tests/{compose,layers}.test.js` e afins) para JSON como os `extrai_*_mvp.mjs`; depois medir o Trial por observação (método do leitor de capturas `trial_le_tela.py`) para os modos de mistura; só então código. Fotografe também com `HOME=/home/enio` (copie antes `~/.ph2d/layout.txt`).
