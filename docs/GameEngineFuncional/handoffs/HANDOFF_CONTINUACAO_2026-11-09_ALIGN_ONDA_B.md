# Continuação — line/GameEngineFuncional, 09/11 → Align, onda B fechada; próximo: defeito do meio texel do átlas, depois onda C

**Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` · branch `line/GameEngineFuncional`
· base `main` `f254e42bf1`. ⛔ Não tocar na `line-GameEngineFuncional02`. Nada integrado, nenhum push.
Antecessor: [`HANDOFF_CONTINUACAO_2026-11-09_ALIGN_ONDA_A.md`](HANDOFF_CONTINUACAO_2026-11-09_ALIGN_ONDA_A.md) · plano:
[`28_alinhar_folhas_estudo.md`](../28_alinhar_folhas_estudo.md).

## Commits da onda B (sobre `27f77d32b5`)

| Commit | O quê |
|---|---|
| `4363f9c9c6` | a prévia Align: onion (anterior + referência), cruz do Pivot, linha do chão, mover/escalar a folha, setas/Alt por quadro, clique direito ▸ Use as reference, cena 5 com fotos pelo rato real |
| `15515581d0` | smoke do dono: 4 quinas, silhueta da referência POR CIMA, conta-gotas, moldura própria embaixo à direita |
| `ffec1bb624` · `d8195929d9` · `d466c93368` | aparência da moldura: botões no idioma dos campos (Bg3), ícone em todos, «Pick», Previous/Reference apagados com o Align desligado, ícone+nome centrados pela largura PINTADA |

## Decisões do dono nesta onda (09/11)

- Pegas de tamanho nas **quatro quinas**; a referência tem **wireframe (silhueta) acima** da imagem que se alinha.
- **Conta-gotas** para escolher a referência (clique num quadro da tira; Esc cancela); o menu do clique direito ficou também.
- Todos os controlos do Align numa **moldura própria embaixo à direita** (ao lado de Cols/Rows e dos modos), levemente
  diferente (fio AccentSoft); botões com ícone (AlignBottom, Eyedropper «Pick», SkipBack, Pin); Previous/Reference
  apagados até o Align ligar; ícone e nome centrados.
- **Filtro de pixel:** já existe nas configurações; **NÃO** pôr pixel nítido como padrão das sprites (a prévia Align usa
  `ImageQuality::Low` por conta própria — fica).

## Desenho (para a próxima LLM)

- `crates/ph2d-anim-editor/src/align.rs`: `AlignUi` (on, show_prev/ref, reference, floor, picking, menu, view congelada
  durante o arrasto), `AlignLayout` (frame/title/toggle/pick/show_prev/show_ref/preview), `AlignPaint` (gates),
  `Silhueta::of` (caixa + arestas em corridas), gestos `align_press/motion/release`, `align_arrow`, `align_secondary`.
  Os gestos emitem `ImageUseEdit::{ShowSheet, MoveSheet, ScaleSheet, NudgeCell}` em `sheet_ui.pending`; o 1.º do gesto põe a
  folha do quadro aberto visível. Undo: um passo por arrasto (o `held_button` suprime; o `frame` aplica antes do
  `post_frame_undo`); cada seta = um passo.
- `paint_align.rs`: camadas (referência tinta Info, anterior α 0,3, aberto), silhueta por cima, linha do chão (base da
  caixa da referência, arrastável), 4 pegas, cruz; `chip` com `Look::{Off, Idle, On}`.
- `layout.rs`: a moldura à direita; aba estreita ⇒ desce para fila própria (`align_below`). A prévia toma a folha toda.
- `EditorSnapshot.geo` ← `ph2d_anim_sheets::align::sheet_geos(world, e, ppm)`; `snapshot(sim, bits, ppm)`.
- `ph2d_editor_core::widget::sheet_cell::paint_sheet_cell_with` (filtro dado).
- Cena 5: fotos `animado_align`, `animado_align_escala`, `animado_align_desfaz` (`Passo::Alinha`, rato real por
  `smoke_pointer_*`); prova: canto 0,48 → 0,32, um Ctrl+Z → 0,48.
- Gates: `shells/game-scenes/tests/it/editor_animacoes/align.rs` (9) e `os_controlos_cabem_e_nenhum_pisa_outro` com a moldura
  no censo. Verdes: `editor_animacoes` 75, `ph2d-anim-sheets align` 7, `the_shell_only_shrinks`, `hr15`, i18n.
- Shell cresceu ~40 linhas (`janela_do_jogo_animado.rs`); folga medida ~610 abaixo do tecto — o integrador reconta.

## ⭐ PRÓXIMO: defeito medido, cura NÃO aplicada (o dono pediu janela nova)

Report do dono: idle 128×32, 4×1, Pivot mudado no canvas; no editor os quadros alinham, **no canvas deslizam**.
Mecanismo (medido com as funções reais; Pivot inocente):
1. `crates/ph2d-render/src/atlas/region.rs:35-43` — `AtlasRegion::uv` encolhe a imagem INTEIRA 0,5 texel por lado.
2. `shells/desktop/src/render_loop/sim_extract_emit.rs:56` e `:96` passam esse rect a `:166-169`, onde
   `sprite_sheet_subrect` (`crates/ph2d-render/src/sprite/subrect.rs:28-44`) divide 127 texels por 4 ⇒ célula de 31,75 a
   começar em `0,5 + 31,75·k`: **−0,25 texel por quadro** (0,75 entre o 1.º e o 4.º) e quadro esmagado (31,75 × 31).
3. Mesmo erro: `sim_extract_sheet.rs:142, 172` (fantasmas/prévia animada), `crates/ph2d-app-components/src/asset_card_portrait.rs:183-194`,
   e `region_subrect` com margem (`subrect.rs:75-79`) antes da grelha.
4. Sem sangramento: `crates/ph2d-gpu/src/recorte_uv.wgsl:7-12` já prende a amostra aos texels da célula.
Cura proposta: dividir a partir do rect EXACTO do átlas (`AtlasRegion::uv_exact` nos três chamadores) e deixar o
`recorte_uv` do shader impedir o vizinho; na região, grelha antes da margem. Gate puro: para folha N×1 no átlas,
`u0 − w·k` e a largura iguais em todo `k`; mais um caso em `crates/ph2d-render/tests/it/sprite_sem_risco_gpu.rs`. Foto antes/depois
na cena 5 (idle parada em cada quadro). ⚠️ Mexe no render partilhado: varrer por CONSUMIDOR quem usa `region_uv`.

## Depois: onda C (plano 28)

Marca na faixa nos quadros acertados; «Align by feet / by centre» (por folha ou por quadro), um Ctrl+Z.

## ABERTO (herdado)

- Após Ctrl+Z, o `ShownSheet` (não registado no undo) volta a 0: o editor e o cartão Image mostram «idle». Antigo.
- `ph2d-app-macros` `tutorial_painter::outra_tela_diz_o_tamanho_gravado` (stack overflow) — alheio, por conferir no main.
- Provas de mutação e clippy das crates tocadas: no fecho da linha.
- «Delete empty animations» não apaga animações de 0 quadros (deliberado); o «+» do cartão Animation não seleciona a nova.
