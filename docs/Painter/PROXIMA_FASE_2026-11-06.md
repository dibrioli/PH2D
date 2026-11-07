# Painter — próxima fase (ordem do dono, 05/11 noite)

Ordem do dono para a próxima fase, na ordem dele. Estado de partida: [HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-05_NOITE.md](handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-05_NOITE.md). **Pendente do dono: a ordem de integrar.**

## Antes de tudo (dono 07/11): reduzir a memória do 1.º traço

O registo do undo guarda quase a grade inteira no pen-down: pico do Dabber 367 → 490 MiB a raio 300. Marcar só a caixa do traço e baixar `TECTO_DA_MEMORIA_MIB`. [Detalhe](../../project-memory/project_first_stroke_undo_memory_cut_next_round_2026_11_07.md).

## 0) «Ver se REDO mata a tinta de Wet Paint (para a simulação)»
O que a auditoria deixou: no teste, três undo e três redo com Borracha/pintura/Borracha funcionam. **Hipótese não provada:** `wetpaint/undo_vivo.rs`, `p.depois.take()?` (linha 196) consome o passo sem devolvê-lo se a refação falha, e `can_redo` (linha 187) só consulta o motor. Começar por **reproduzir**: um traço, undo, redo com a água ainda correndo, e olhar se a simulação para. Depois o teste vermelho, depois a cura.

## 1) Área de contato/mescla das tintas aguadas sobre o óleo: «vários pincéis têm resultado ruim»
Fotos em [`material_2026-11-05/proxima_fase/`](handoffs/material_2026-11-05/proxima_fase/).
- **Foto 1 — Wet Wash sobre Dabber:** [foto1](handoffs/material_2026-11-05/proxima_fase/foto1_wet_wash_sobre_dabber.jpg).
- **Foto 2 — Wet Liner sobre Dabber:** [foto2](handoffs/material_2026-11-05/proxima_fase/foto2_wet_liner_sobre_dabber.jpg).
- **Foto 3 — Watercolor Basic (modo Watercolor) sobre Dabber:** [foto3](handoffs/material_2026-11-05/proxima_fase/foto3_watercolor_basic_sobre_dabber.jpg). «faz uma mescla interessante embora não realista». Estudar a interação e descobrir **por que** dá bom resultado, para melhorar os outros do Wet Paint. ⛔ «esse resultado Watercolor Basic não pode ser modificado pois considero bom».
- **Foto 4 — Wet Drip sobre Dabber:** [foto4](handoffs/material_2026-11-05/proxima_fase/foto4_wet_drip_sobre_dabber.jpg). Razoável, ainda com artefatos, áreas pixeladas duras.
- **Foto 5 — provável solução:** [foto5](handoffs/material_2026-11-05/proxima_fase/foto5_wet_round_receita_boa.jpg), Wet Round com **Drag 0,5, Solubility 1,0, Cover 0,1, Dry Cover 1,0**.

Leitura nossa (marcada como tal, a confirmar): nas fotos 1, 2 e 4 há uma **linha verde dura** na junção da tinta aguada com o óleo; nas fotos 1 e 2 o amarelo/azul do óleo aparece **salpicado** por baixo da aguada.

**Ordem sugerida:** (a) reproduzir a foto 5 por script (Wet Round com os quatro valores, sobre Dabber) e comparar com as fotos 1, 2 e 4; (b) partir dos testes existentes `agua_sobre_oleo_tests` (`ph2d-tool-painter/src/tool/paint/wetpaint/agua_sobre_oleo_tests.rs`, de onde vêm `com_oleo`, `ponteiro`, `preset`, `relato`); (c) só então mexer em pincel.
**Restrição:** os pincéis de fábrica do dono só mudam com o olho dele: alarmes de largura/miniatura + relatório, ele aprova.

**Nomes dos controles no código** (`ph2d-wet-paint/src/tuning.rs`, chaves em `ph2d-panel-wet-tuning/src/rows.rs`): Drag = `Knob::RewetDrag` (chave `rewetDrag`) · Solubility = `Knob::Solubility` (`solubility`) · Cover = `Knob::WetCover` (`wetCover`) · Dry Cover = `Knob::DryCover` (`dryCover`) · `Knob::Dissolve` existe e multiplica com Solubility. **Não confirmado:** que os rótulos do painel sejam exatamente «Drag», «Solubility», «Cover», «Dry Cover» (só as chaves); e que o `Knob::Drag` do começo do enum seja outro controle (arrasto das cerdas, não conferido).
**Confirmar com o dono:** o mapeamento das fotos. O anexo `5.png` chegou danificado; usamos `6.png` (mesmas dimensões) como «última foto»: **suposição nossa**.

## 2) Travamentos: estudo e pesquisa na internet sobre o problema e as soluções
Dado do dono: no Watercolor Basic o 1.º traço é muito mais lento; nos outros, pouco. Nossas medições e recusas: [BUGS #72](BUGS_painter.md) (Wet curado a 1080p na sonda; Watercolor Basic sem cura: 7–11 ms no pen-down + ~5 ms de reconstrução no tique; telas grandes pagam a cópia da tela, 21–27 ms a 4096×2560). Ideia discutida: **aquecer (warm-up) numa tela invisível**, não com um toque no documento. Pesquisar a literatura/prática de «first-stroke latency» e pré-aquecimento em apps de pintura, e medir no app real com a régua de [MEDIR_VELOCIDADE.md](../DevOps/MEDIR_VELOCIDADE.md).

## 3) Pincéis mais importantes do Clip Studio, para imitar alguns na nossa engine
CLAUDE.md §0.9 e [`docs/_ComoInvestigarApps/`](../_ComoInvestigarApps/README.md): outro app é **oráculo que se corre**, nunca fonte que se lê; passo 1 = triagem de licença por artefato instalado. **Achado (só leitura):** `pacman -Qs 'clip.?studio'` não acha nada (só `wl-clipboard`); existe `/home/enio/Apps/clipstudio` com `wineprefix`, `instalador`, `csp.sh`, logs e 38 entradas (c00.png… capturas), ou seja, uma instalação via Wine, **não conferido se corre**. É software proprietário: tratar como oráculo de caixa-preta e, se algo restrito entrar, [SKILL_Cleanroom_Reimplementacao.md](../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
