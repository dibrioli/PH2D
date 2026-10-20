# HANDOFF — JANELA 1 → JANELA 2 (2026-10-20)

> Leitor: a JANELA 2 (motores). Leia antes: `ESTADO.md` (a fila e as decisões), `03_MOTORES.md` §4–§5 (as lacunas),
> `04_BANCADA.md` (como se mede). Árvore: `Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.

## O que a J1 deixou

- **A bateria** (`ferramentas/bateria.py` → `bateria.json`): 3 cenas 1000×640 — `gestos` (recto lento/rápido, S,
  espiral, rampa 0→1, rampa que afunila, verticais), `tamanhos` (5/40/200 px), `sobre` (faixa vermelha seca + molhada,
  o pincel azul cruza). É a fonte ÚNICA para todo oráculo e para os nossos motores. Não a mude sem regenerar todos.
- **Oráculos com fotos da bateria**: Rebelle 281/281 (`fixtures/rebelle_bateria/`, A molhada e B seca; o teste expira
  ~06/11 mas já não importa para as fotos) · ArtRage 35 (`fixtures/artrage_bateria/`, só A; demo sem prazo) ·
  MyPaint 196 (`fixtures/mypaint_bateria/`, CC0 — portáveis) · Krita 144 (`fixtures/krita_bateria/`); nenhum expira. Clip Studio, Painter, Photoshop: não correm sob Wine (o porquê no 02).
- **A bancada** (`04_BANCADA.md`): `exame.sh <preset> <pasta-do-oráculo>` pinta a bateria com os nossos motores
  (`tests/it/exame_bancada.rs`, perfil smoke), mede (`exame_regua.py`), monta a prancha legível e a cega
  (`exame_prancha.py`), a matriz (`exame_matriz.py`, escala congelada em `matriz/escala.json`) e o custo (`--custo`).
  Revisão cega: o briefing em `05_REVISAO_CEGA.md`.
- **Catálogo** 112 pincéis (`01_CATALOGO.md`), **motores** e 19 lacunas (`03_MOTORES.md`).
- **Fábrica**: 31 → 28 (Filbert, Flat Oily, Knife Smooth fora; portões verdes por nome em `d9e85a11c4`).

## O que a J2 deve saber antes de tocar no motor

1. **Comece pelas leis sistémicas** (ESTADO, fila J2 item 0): a pressão (o Rebelle desvanece a carga a largura
   cheia; nós estreitamos), o traço fino de 5 px (claro e pontilhado em nós, sólido no Rebelle), as pontas (cápsula ×
   corte desfiado), a lente clara sobre a tinta molhada. Elas tocam quase todo pincel; uma lacuna medida antes
   delas mede contra um alvo que vai mudar.
   - Meça a lei da pressão nas 281 rampas (`rampa_0_1` da cena `gestos`) com a `exame_regua.py`, por ferramenta.
     A janela J1 só a viu no 8B e no Oil Bristle.
   - ⚠️ Sem piso de pressão: `project-memory/feedback_pen_pressure_floor_must_keep_the_owners_hand_range.md`.
     O que se ajusta é para ONDE vai a pressão (carga × largura), não a faixa da mão.
2. **L1 (presets dos 4 meios)** é a maior: a captura grava sempre `"WetPaint"` (`wetpaint_preset.rs:38,475`), a
   carga força o meio (`:334–336`), o navegador só abre com o Wet (`painter_bridge_presets.rs:324–331`), a secção
   decide-se em `crates/ph2d-panel-painter-layers/src/paint_brush_sections.rs:116`. O formato aguenta (`media` texto).
3. **O que o 03_MOTORES achou que JÁ existe** (não reimplemente): `DabRotation::FollowTrajectory` (o Wet já segue o
   traço — falta inércia, memória e barril), o modo de mistura do pincel no Wet (`wetpaint.rs:608`), «Use Texture
   Colors».
4. **Cerca**: os 3 meios antigos ficam byte-idênticos com o recurso novo desligado (`os_outros_meios_nao_mudam.rs`,
   `cerca.sh`, as miniaturas de fábrica).

## Recursos e armadilhas medidos na J1

- **Nenhum comando pode pedir autorização** (dono, 20/10 — um pedido parou a missão): sem `cd X && …`, sem
  `bash -c`, sem `rm -rf $VAR`. Está no `00_MISSAO` §1 e tem de ir no briefing de cada agente. Há um hook local
  que aprova (`.claude/settings.local.json`); os guardas do repo continuam a recusar.
- Briefing comum dos agentes da J1 (reaproveite): o preâmbulo que está no fim deste ficheiro, §Preâmbulo.
- Rebelle: UMA instância, `gotas_inicia.sh` com `PH2D_GPU=1`, fecho só por `gotas_fecha.sh`; o lote corre em sessões de
  30 min com a vez da placa (`lote_bateria.sh`). Apps sob Wine: Xvfb próprio (`:71`–`:75`), nunca a tela do Rebelle.
- Custo na bancada: p95 ≤ 6,1 ms/quadro a 200 px, ~96 MiB (8B, Oil Bristle) — margem larga para os 60 Hz.

## §Preâmbulo (o briefing comum dos agentes)

FASE 0 `cd` absoluto + `pwd` + `git branch --show-current`; só esta árvore; autonomia total (decisão técnica =
padrão-ouro, de produto = escrita com o porquê); proibido push/merge/stash/`add -A`/matar processos alheios/`spectacle`;
ler `00_MISSAO.md`; inner loop `cargo-check-narrow.sh`, teste por nome `cargo-test-narrow.sh`, pesado por
`ph2d-run.sh`, sondas `--profile smoke`; editar com Edit; a bateria é a fonte única; no máximo 2 compilações pesadas;
commits só dos seus caminhos (`git add -- <p>` para os novos), `--no-verify`, PT, `Co-Authored-By`; nunca copiar
pincéis/texturas/código de apps proprietários; NENHUM comando que peça autorização; relatório final ≤ 60 linhas.
