# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, a missão «cada controlo funciona em cada meio» (2026-10-04)

> **Para a próxima janela da MESMA linha** (troca de agente, `MODELO_TROCA_DE_AGENTE_NA_LINHA.md`).
> Não é o handoff de integração — esse escreve-se no fecho e cobre os 16 commits desde `1ad60a1ce`
> (os 10 da rodada de 02/10, listados no briefing da troca de 02/10, e os 6 abaixo).

## §1 — Coordenadas

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` |
| ramo | `line/PainterWatercolor` · HEAD `86b666703` · merge-base `1ad60a1ce` (= o `main`, que não andou) |
| árvore | limpa (`!! assets/sprites/` só) |
| a missão | ordem do dono de 2026-10-02 (o briefing da troca) + as DECISÕES de 2026-10-04 no [doc 46](../46_plano_cada_controlo_em_cada_meio.md) |
| a tabela medida | [doc 45](../45_censo_dos_controlos.md) |

## §2 — O que esta janela fez (não reconstruir)

| commit | o quê |
|---|---|
| `9dbf23ba2` | Wet Paint: carimbo FORA da grade rebentava a Blow (e a Smear) — `rect_around_nao_vazio`; gate `a_dab_outside_the_grid_touches_nothing_in_any_tool` |
| `ff38c9a48` | **o censo** (`crates/ph2d-panel-painter-layers/tests/it/censo_dos_controlos.rs`, sondas `#[ignore]`) + a feature `test-support` da `ph2d-tool-painter` (`set_wet_relogio_fixo`: a água num relógio FIXO) + doc 45 |
| `9c35b6792` | o **Reset** de uma secção repõe o pincel com que o MODO nasce e nunca troca o meio (`fabrica.rs`, `PaintState::pinceis_de_fabrica`) |
| `fedbb192f` | **Accumulate/Space Attenuation** só onde o meio os oferece (`BrushSettings::accumulate_offered`; o `authored_spec` desliga-os) |
| `d2e9f7c5d` | o **esmaecido** (`inercia.rs` na ferramenta, `esmaecer.rs` no painel, 12 dicas) + o gate permanente `a_linha_esmaecida_e_a_que_nao_age` |
| `86b666703` | docs: plano 46 actualizado, Bug #15 FECHADO (medido) |

Gates novos (todos com prova de mutação no corpo do commit): `reset_de_fabrica` (3), `acumulacao_por_meio` (2), `seam_accumulate_por_meio`, `a_linha_esmaecida_e_a_que_nao_age`, `a_dab_outside_the_grid_touches_nothing_in_any_tool`.

## §3 — O que falta (ordem do plano 46 §2)

1. **Dry Time — a aquarela SECA com o tempo** (item 5; ordem explícita do dono). Factos (lidos, com
   `file:line`):
   - hoje a sessão molhada é TUDO-OU-NADA: `wet_session_continues` (`watercolor_backdrop.rs:296`) e
     o `dry_session_now` (`:420`) quando o `canvas_wet` (o mapa de humidade) chega a zero em toda a
     parte (`watercolor_dry.rs` ~157). O Dry Time já decide QUANDO a sessão inteira seca — o censo
     leu `0` porque os traços estavam a 67 ms um do outro; o que NÃO existe é secar pixel a pixel;
   - o `canvas_wet` é poured pelos traços (`pour_canvas_wet`) e erodido pelo `dry_canvas_wet`
     (row-parallel, ADR-0109, `28,87 → 3,45 ms` a 4096²); hoje só o brilho o lê
     (`crates/ph2d-app-painter/src/painter_bridge_wetness.rs:30`);
   - o desenho padrão-ouro já estava escrito e adiado: doc 12 (EDGE-1, `:136-148`) e doc 14 #12b
     (`:121-133`) — Curtis 1997 / DiVerdi 2013, máscara molhada persistente com decaimento.
   - **Desenho proposto (a medir antes de construir):** no `paint_begin` de um traço que continua a
     sessão, os texels da sessão cujo `canvas_wet` já chegou a `0` ASSAM-SE para a base da sessão
     (`wet_session_base` ← o pixel visível) e saem das uniões (`stroke_coverage`/`stroke_color` e o
     resto que o `dry_session_now` limpa) — o traço novo VELA por cima deles e funde onde ainda está
     molhado. Só no pen-down (nada anima enquanto o artista espera). ⚠️ A régua: a fronteira
     molhado/seco ganha borda escura no re-render — medir com a IMAGEM antes de aceitar; e o gate
     de reprodutibilidade do bake (`watercolor_session_rerender_reproduces_the_bake_byte_exact`)
     tem de continuar verde. Gate red-first: dois traços separados por `> Dry Time` velam; por
     `< Dry Time` fundem; o mesmo par com Dry Time diferente dá imagens diferentes.
2. **o gate permanente do CENSO** (item 4): a sonda vira teste com a lista do doc 45 §2 escrita (ids
   por *slug*, motivo por linha) — controlo novo sem motivo = vermelho; entrada que passa a viver =
   vermelho. ⚠️ Custo: a exploração BFS a 8 fios leva `~5 min` por meio sob carga; o gate deve usar
   profundidade 1 ou os estados da tabela das pré-condições (como o do esmaecido, `37 s`).
3. **Blend no Wet Paint** (item 6), **Solid · Shape Color Ramp · fios na Aquarela** (item 7),
   **Composite no Wet Paint** (8), **Solid · fios no Wet Paint** (9), **esconder a Shape Color Ramp
   no Wet Paint** (10) — a análise e os kill-criteria estão no doc 46 §1.
4. **o Spread da aquarela** fora do esmaecido: a lei não está num sítio só (`watercolor_field/style.rs:112`
   captura `spread_px`; o censo acordou-o com Wet `0,8` e com Dilution) — achar a lei exacta antes.

## §4 — O instrumento (como correr)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
# a tabela de um meio (BFS prof. 3, 8 fios) + a procura de pré-condições (1 = vizinhos, 2 = estado inteiro)
CENSO_MEIO=Digital CENSO_ARMAR=1 bash scripts/ph2d-run.sh cargo test -p ph2d-panel-painter-layers --test it sonda_a_tabela_dos_controlos -- --ignored --nocapture > target/censo.log 2>&1
python3 docs/Painter/ferramentas/censo_dos_controlos/nomes_dos_ids.py < target/censo.log | python3 docs/Painter/ferramentas/censo_dos_controlos/classifica.py
python3 docs/Painter/ferramentas/censo_dos_controlos/arma.py target/censo.log --so-mortos
# as pré-condições escritas à mão (e CENSO_MODO=depois mede o gesto DEPOIS dos traços)
bash scripts/ph2d-run.sh cargo test -p ph2d-panel-painter-layers --test it sonda_as_pre_condicoes -- --ignored --nocapture
```

## §5 — O que custou tempo (e não deve custar outra vez)

- **A régua mentiu três vezes** (doc 45 §1): o Wet Paint sem relógio (a água não andava), o Wet Paint
  ao relógio real sob carga (`1 148` texels de ruído entre corridas iguais — por isso o relógio
  fixo), e dois traços da MESMA cor (o Pickup «morto»). E o `thread_local` do painel (o stop
  seleccionado da rampa) passa de ensaio para ensaio no mesmo fio — o alvo de um número recalcula-se
  no momento do gesto.
- `CENSO_ARMAR=2` não cabe nos 30 min do `ph2d-run` sob carga no Impasto/Wet Paint — use `1`.
- O `rustfmt` REORDENA os `mod` e deixa o comentário de fim de linha no vizinho (foi o que
  aconteceu ao `mod fabrica;` — corrigido em `d2e9f7c5d`): comentário de módulo vai em `///` por cima.
- Inserir antes de `pub fn X(` rouba o doc de X (aconteceu com `param_specs`; corrigido antes do
  commit) — insira acima do bloco de doc e confira com `grep -B4`.
- Os gates da `ph2d-panel-registry-init` (`nenhum_rotulo_do_app_pinta_nada`, `a_marca_tem_a_altura_da_linha`)
  só valem no âmbito do WORKSPACE (`cargo nextest run --workspace -E 'test(...)'`): com `-p` falham
  por falta de painéis — corra-os no gate do fecho (esta janela escondeu linhas do painel e não os
  correu no âmbito certo).
- Flake de carga visto e confirmado 3/3 sozinho: `the_mask_stroke_cost_does_not_follow_the_canvas`.
