# Handoff de integração — line/PainterWatercolor, pós-missão 4K (11/11, ondas 15–20 + fecho)

Leitor: o agente integrador. Continua o [handoff da missão 4K](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-10_MISSAO_4K.md)
(§2 vendor `winit`, §8 riscos — tudo continua a valer); aqui só o que esta janela acrescentou. O mecanismo de cada item está
na linha do [BACKLOG](../missao_4k/BACKLOG.md) (id `B<nn>.<n>`) e no commit; o que cada onda devolveu, no
[despacho](../missao_4k/DESPACHO.md). **Nada integrado nem enviado.**

## 1. Identidade

- Branch `line/PainterWatercolor`, merge-base com o main `bc4a6ce930`; a linha soma 368 commits (1 814 ficheiros).
- Esta janela: `e81e56bb1f..1d40807b12` = **82 commits** (539 ficheiros, +38 999/−1 830, dos quais `docs/Painter` +27 698);
  os commits deste fecho F3 vêm por cima (só docs: despacho, handoff, prompt, `CLAUDE.md` §5.1).
- Tocado nesta janela (linhas + / −, ficheiros): `ph2d-tool-painter` 5 511/452 (156) · `ph2d-painter-brush` 1 100/179 (34) ·
  `ph2d-wet-paint` 966/113 (30) · `ph2d-panel-painter-layers` 848/182 (38) · `ph2d-panel-wet-tuning` 828/350 (14) ·
  `ph2d-inpaint` 817/210 (12) · `ph2d-render` 458/109 (14) · `ph2d-pigment` 298/0 (3) · `ph2d-app-flip` 141/130 (2) ·
  `ph2d-det-math` 109/3 · `ph2d-app-painter` 72/15 · `ph2d-i18n` 66/7 · `ph2d-gpu` 52/11 · `ph2d-editor-core` 11/0 ·
  pequenos: `ph2d-wet-gpu`, `ph2d-flip-render`, `ph2d-app-motion` · `shells/desktop` só `project_schema{,_tests}.rs` ·
  `.cargo/config.toml` e ADR-0174 (só comentário: B43.5) · 39 presets/miniaturas em `assets/brushes`.
- Continuação: [prompt 11/11 n.º 2](PROMPT_CONTINUACAO_2026-11-11_pos_missao_4k_2.md).

## 2. Superfície de colisão (`bash scripts/collision-surface.sh`, 11/11, linha inteira)

```
merge-base bc4a6ce930 · 368 commit(s) · 1814 arquivo(s)
⚠ PROJECT_SCHEMA 233 (base 231) · ⚠ tripla (233, 13, 29) (base (231, 13, 29)) · VEC 29 · FLIP 13 · DOC_VERSION 25
ph2d-ecs 121 (base 121) · ✗ SONDA CEGA ph2d-render/ph2d-script = defeito conhecido do SCRIPT (a linha não toca componentes)
Contrato congelado §6: node.rs intocado · tool.rs intocado
ADR: último no disco 0202 · esta linha cria 0201 0202 — reconte contra o main do dia (esta janela: nenhum ADR novo)
Cargo.lock: nenhum '+name' novo · marcadores de conflito: nenhum
TETOS DE LOC: os do vendor/winit (§2 do handoff anterior) · ph2d-wet-paint/examples/sonda_f32_v3/kern.rs 861 e
  sonda_secagem.rs 1 240 (examples, fora de crates/*/src) · shells/desktop main.rs 1 031 e app_state.rs 920 = iguais ao main
  (esta linha não os toca nesta janela)
```

## 3. Números que se CONTAM (DELTA)

- **`PROJECT_SCHEMA`: +2 na linha** (231 → 233). O degrau desta janela é `602f277f35` (B34.1: `PaintedDocument.fator_aparencia: f32`
  no FIM; lado curto ÷ 1080, nunca < 1). Na fusão: `python3 scripts/schema-recount.py <valor do main + 2>` DENTRO do conflito.
  Sem migração (CLAUDE §0.11). O `SCHEMA` do preset de pincel continua 2 (B47.6 regravou as fábricas nele sem mudar um píxel).
- **ADR: 0 nesta janela** (a linha continua a criar 0201 e 0202). ADR-0174 só corrigido em texto (B43.5). **Registo de
  componentes: 0. Pacotes externos novos: 0.**
- `CAMPOS_DO_PINCEL` 43 → 42 (B32.2: sai `pigment_espectral`) · `KNOBS_NO_CARTAO` 18 → 21, `FIELDS` 40 → 43, Tuning 67 → 65,
  `SECTIONS` 9 → 8 → 7 (B35.12 Relief; B47.7 Bristles) · `ids::CERDAS_NO_SHAPE` 17 (B35.13).
- **Tectos:** fmt/clippy `-D warnings` verdes nas 17 crates tocadas (`9cdcbb95b7`); `paint_texture.rs` 630 → 600 (`7c01a2a029`);
  partições `budget.rs`, `pass_collect.rs` (ph2d-app-flip), `rows.rs` 604 → 306.

## 4. O que entra (números antes → depois; 4K, Radeon integrada salvo nota)

| Tema | Itens | Número |
|---|---|---|
| Ctrl+Z | B17.5 (upload só nas caixas), C1 (B08c.M2/M3) | Ctrl+Z Impasto 4K blocos **42,8 → 34,3 ms** (= caminho antigo 34,1; a regressão fechou) |
| Soltar o pincel | C2, C3 (dívida da luz Cheia em faixas de 16 Ki px, ≤ 8 ms/quadro) | soltar Filbert 4K **47,8 → 34,0 ms**; 1024² 43,2 → 31,0. **Defeito curado:** a metade de baixo do traço ficava com a luz Simples no repouso |
| Medição «depois» no app | B08c, [ANTES_DEPOIS.md](../missao_4k/medicoes/ANTES_DEPOIS.md) §M1–M5 | 41 células; as 3 regressões explicadas (Filbert = a luz Cheia no repouso, B14.2; Water = dispersão; Impasto = C1) |
| Água Wet | B13.2 cont7/cont8 (feito), B13.3 sono (DESLIGADO), B16d hash2 | ladeira e zeros só nos tiles, ao bit; sono por tile 64² não ganha a 4K |
| Mistura | B32.1–.4, B28.3 | Wet vermelho + azul: oliva c14 → violeta c54 (lei espectral de 10 bandas; `PH2D_WET_MISTURA=km` volta); 20 pincéis mudam. Color mixing r100 2048² 21,59 ms/q → ~4 ms/q acima do alvo (parcial) |
| Papel e luz | B34 (fator de aparência), B34.1b, B31.1, B31.2, B29.3 | luz do dente 4K 4,561 → 9,318 (1080p 8,555); sem aro branco à volta do Impasto; realce sem anéis 15,8× → 1,18×; grão é tecto sob a película |
| Tinta | B29.1 (tecto largo u16) | Flow 2 % não pára mais em 25/255; 10 pincéis mudam ≤ 4 níveis |
| Oil Wash | B36.1 | preset novo (Dilution 0,6, lei E10g); escorre 77 × 63 px a 1080p, 143 × 126 a 4K |
| Clone/Heal | B38.1–.3 | camada transparente sem orla escura; Heal placa = CPU ao byte |
| Controlos | B35.3/.5–.13, B39.1–.6, B47.7 | linha com pai desligado esconde; fábrica única; caixas de marcar; % no cartão Wet; Erase Strength único; «Tooth fill»; Opaque/Paint drag/Emboss no cartão Wet; 17 cerdas no cartão Shape |
| Memória e placa | B42.1–.5 | 64 camadas 4K na integrada (GPU 180 ms × CPU 720 ms); undo pela RAM (era 533,8 MB fixo) |
| Determinismo | B43.1, B43.5, B43.6 | `det_powi` (47 chamadas); portão do `RUSTFLAGS` que apaga o v2; cgu1 recusado no crate inteiro (~4×) |
| Mutação | W15-A | os 2 sobreviventes da missão (`funde_as_caixas`, folga B25.1) sangram |
| Recusa | B16.8 | a água da Dilution não é inerte (backrun) |

Pranchas novas: B28.3, B29, B29.1, B31.1, B32, B34, B35.9, B36, B38 ([INDEX](../missao_4k/pranchas/INDEX.md)).
BACKLOG: **151 feitos · 26 parciais · 10 recusas medidas · 135 abertos** (na missão: 117 · 16 · 9 · 174).

## 5. Prova de fecho (11/11)

Nada de suíte completa nem `nextest-impacted` (dono 23/10). Cada item red-first e verde por nome no seu commit. Fecho F1:
`fmt --check` e `clippy -D warnings` das 17 crates de `e81e56bb1f..HEAD` verdes (`9cdcbb95b7`); `os_cartoes` verde
(`32d4989ccf`); tectos (`7c01a2a029`); `architecture_claude_md` (este fecho, §5.1).

**Vermelhos herdados (B18.Y):** os 3 da tool-painter são mudanças de lei da missão (dois de B29.3, um de B33.1) — alarme, nenhum
gate afrouxado.

**Alarmes de pincel ([ALARMES_FECHO_2026-11-11.md](../missao_4k/medicoes/ALARMES_FECHO_2026-11-11.md), NÃO regravados):** largura
2 vermelhos (Oil Wash sem largura aprovada; Smear Wet já vermelho antes) · miniatura 74 (B32.1: 20 · B28.3: 11 · B29.1: 6 ·
B49/anteriores ou sem origem: 37). Relatório `target/relatorio-pinceis/index.html` (gerado 11/11 12:17).

`agent-loop-profile` (20 sessões):
```
✗ paralelismo de ferramenta              1.15/passo   alvo: >= 1,5
✓ respostas por sessao (mediana)                174   alvo: <= 800
✗ cargo test : cargo check                424 : 141   alvo: <= 1,0  razao 3.0x (baseline: 4,3x)
✓ edicoes SEM conferir que casaram             1.6%   alvo: <= 2%
✗ contexto relido por passo (media)         255 mil   alvo: <= 250 mil
✓ contexto no inicio da sessao               68 mil   alvo: <= 80 mil
```

Binário do smoke (2.ª corrida de `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`):
```
▸ linha line_painterwatercolor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.43s        (1.ª corrida: 12,71 s; zero «Compiling» na 2.ª)
```

**NA INTEGRAÇÃO, suíte inteira:** formato gravado (schema +1 nesta janela), campos do preset e do Tuning (censos de contagem),
código movido (`budget.rs`, `pass_collect.rs`, `rows.rs`), `ph2d-render` (`LayerPixels.dirty_parts`: flip/motion passam `&[]`).

## 6. Smokes do dono

Nenhum visto pelo dono. Fotos: as pranchas (cada uma mostra antes | depois em 3 fundos, 1080p e 4K).

1. **Soltar o pincel em 4K sem soluço.**
   (1) `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop`
   (2) Painter → imagem nova 3840×2160 com papel Textured → pincel **Filbert** (WetPainter ▸ Oil ▸ Flat) → um traço longo, soltar.
   (3) Ao soltar a tinta ganha o brilho final aos poucos, sem trava visível; o traço INTEIRO fica com o mesmo brilho.
   (4) Errado: a tela congela um instante ao soltar, ou a metade de baixo do traço fica mais «chapada» que a de cima.
2. **Ctrl+Z do Impasto.** No mesmo documento, pincel com **Impasto** ligado, traço diagonal, **Ctrl+Z**. Tem de voltar na hora; errado = engasgo visível.
3. **Oil Wash (pincel novo)** — [foto](Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/pranchas/B36/4k_papel_filbert_oil_wash.png).
   WetPainter ▸ Oil ▸ Flat ▸ **Oil Wash**, pintar e esperar 15 s com a tela inclinada: escorre mais que o Filbert. A largura dele ainda não tem a sua aprovação (relatório, passo 9).
4. **Vermelho + azul no Wet dá violeta** — [foto](Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/pranchas/B32/1080p_papel_express_oil_km_espectral.png).
   Pintar vermelho, por cima azul, com Express Oil ou Knife. Certo = violeta vivo; errado = verde-oliva ou quase preto.
5. **Papel a 4K igual ao de 1080p** — [foto](Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/pranchas/B34/4k_papel_chalk_antes_depois.png).
   Imagem nova 4K com papel, pintar com **Chalk**: o grão tem o mesmo tamanho à vista que numa imagem 1080p. Errado = chiado fino.
6. **Impasto no papel** — [sem aro branco](Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/pranchas/B31.1/1080p_papel_impasto_antes_depois.png) ·
   [dente resiste ao repasse](Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/pranchas/B29/1080p_papel_pastel_antes_depois.png).
   Traço grosso com Impasto: nenhuma linha branca em volta. Chalk/Pastel com Impasto, ir e voltar 4×: o grão do papel continua a aparecer.
7. **Flow baixo chega ao preto** — [antes](Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/pranchas/B29.1/1080p_branco_flow2_antes.png) ·
   [depois](Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/pranchas/B29.1/1080p_branco_flow2_depois_continuous.png).
   Flow 2 %, Continuous ligado, repassar muitas vezes: escurece até ao preto. Errado = pára num cinzento.
8. **Controlos e camadas** — Borracha do Wet ([foto](Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/pranchas/B35.9/1080p_branco_antes_depois.png)):
   **Erase Strength** no máximo limpa o meio do traço. Cartão **Wet**: «Tooth fill», Opaque, Paint drag, Emboss; percentagens em
   Pickup/Edge Darkening/Gravity. Cartão **Shape** com ponta Bristles: grupo «Bristles» com as cerdas. Liga/desliga = caixa de marcar.
   Clone/Heal numa camada transparente ([foto](Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/pranchas/B38/1080p_blocos_heal_antes_depois.png)):
   sem orla escura nem mancha preta. Errado = controlo que não muda nada, ou rótulo repetido.
9. **Relatório de pincéis** — abra [o relatório](Worktrees/line-PainterWatercolor/target/relatorio-pinceis/index.html): 74 miniaturas
   e 2 larguras (Oil Wash, Smear Wet) mudaram, por grupo de origem (mistura violeta 20 · Color mixing 11 · Flow 6 · anteriores 37).
   Só o dono aprova; nada foi regravado.

## 7. Decisões que são DO DONO

1. **Luz cheia também durante o gesto do Óleo Real (B14.2).** Hoje: luz simples enquanto pinta e cheia ao parar → soltar 4K
   **34 ms**. Com a cheia no gesto: **19 ms** ao soltar e o traço não fica mais caro (p50 18,2 × 18,9). Compare:
   `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && PH2D_COMPOSITE_SMOKE=1 ./target/smoke/ph2d-host-desktop`
   e o mesmo com `PH2D_COMPOSITE_LUZ_GESTO=cheia` antes de `./target/…`.
2. **Impasto com dente (B29.3):** o grão do papel passou a mandar sob a película; o brilho e a cera do Impasto no papel ficam
   mais fracos. Fica assim ou volta?
3. **Sono da água desligado (B13.3):** a água parada dormiria por bloco, mas a 4K não ganha (≥ 25 % exigido) e não dá o mesmo
   resultado ao bit. Nada muda na tela; fica desligado salvo ordem contrária.
4. **Larguras e miniaturas** do relatório (passo 9).

## 8. ABERTO, por ordem

1. Smoke do dono (§6) e as decisões (§7).
2. **B14.2 / C3:** se o dono quiser a luz cheia no gesto, ligar e regravar; senão o soltar fica em 34 ms (alvo 33,3).
3. **Parciais:** B28.3 (~4 ms/q acima do alvo a 300 px/q; o carimbo) · B17.5 (A×B no app dos dois cantos) · B13.3 (ε > 0) ·
   B34.1b (sombra do óleo, orla 48 px, warp) · B31.2 (prancha da crista) · B35.13 (dustDrop/wetOpen* junto dos pais) · B39.6
   (a camada de Textura ignora o Angle) · B42.3–.5 (undo vivo; tecto por motor do raio) · B43.6 (drop-crate cgu1).
4. **Por começar:** 135 abertos no BACKLOG — B09 oráculos Krita/MyPaint · B18 · B27 · B30 · B33 · B35b–d · B37 · B44 · B45 · B46.
5. Wet 4K: Ctrl+Z dos dois cantos 56,0 ms e toque 62,5 ms (✗ alvo 33,3, [ANTES_DEPOIS](../missao_4k/medicoes/ANTES_DEPOIS.md) §B17.5).

## 9. Riscos para o integrador

- Os do [handoff anterior §8](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-10_MISSAO_4K.md) (`winit` vendorizado, gates de custo
  sob carga, `quer_quadro`). A árvore suja de outro agente que lá se cita **já não existe** (esta janela partiu de árvore limpa).
- **Schema +2** sobre o main: o degrau B34.1 vive em `project_schema.rs` (doc do degrau) e na tripla do teste irmão.
- `ph2d-render::LayerPixels.dirty_parts`: campo novo; outra linha que construa `LayerPixels` precisa de `&[]`.
- `PH2D_WET_MISTURA=km`, `PH2D_COMPOSITE_LUZ_GESTO=cheia`, `PH2D_WET_OTICA_FORA` são interruptores de execução para A×B; não remova.
- Alarmes de pincel vermelhos por desenho (2 + 74): não regravar.

## 10. Onde está o mecanismo

[BACKLOG](../missao_4k/BACKLOG.md) · [despacho](../missao_4k/DESPACHO.md) (ondas 15–20 + fecho) ·
[ANTES_DEPOIS.md](../missao_4k/medicoes/ANTES_DEPOIS.md) §M1–M5 · [ALARMES_FECHO](../missao_4k/medicoes/ALARMES_FECHO_2026-11-11.md) ·
[pranchas](../missao_4k/pranchas/INDEX.md) · [ADR-0174](../../architecture/decisions/0174-o-produto-compila-para-x86-64-v2.md) (B43.5).
