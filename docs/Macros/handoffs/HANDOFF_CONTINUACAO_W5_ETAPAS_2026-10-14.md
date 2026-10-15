# Continuação — line/Macros-Automação, W5 (ETAPAS): W5a..W5d feitas → W5e

> Para a PRÓXIMA janela. Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`, base
> `0aab2c1cb6` (= `main` a 14/10, 0 atrás). Leia antes de agir: o **plano** [`02_plano_W5.md`](../02_plano_W5.md)
> (medições, recusas, ondas) e, para o que veio antes, o [handoff W4](HANDOFF_CONTINUACAO_W4_COMPONENTE_2026-10-13.md).
> Decisão do dono (14/10, «siga»): arrastar uma etapa para mais tarde abre uma ESPERA (o relógio corre), nunca
> câmara lenta.

## Commits da W5

| commit | o quê |
|---|---|
| `5b2f45636d` | W5a: `TocadorDoPainter::passa_ao_relogio_fixo` (+ `WetSession::solta_o_roteiro`, `PainterTool::wet_passa_ao_relogio_fixo`), sondas `#[ignore]` `ph2d-tool-painter/tests/it/sonda_etapas.rs` e `ph2d-app-motion/src/macro_track/etapas_sonda_tests.rs`, plano `02_plano_W5.md` |
| `9b40399255` | núcleo: `ph2d_macro::etapa` (`LeitorDaFamilia`, `EdicaoDeEtapa`, `efetiva`, `Track::tocavel`, `Track::edicao_mut`); `Track.edicoes`; codec 1.1 (Track tag 5, MacroDoc tag 3 `exige`, `CodecError::Exige`; sem edições o escritor continua 1.0 ao byte) |
| `7cc0483bae` | Painter: `macro_track/etapas.rs` (`LeitorDoPainter`, `etapas()`, `edita_cor/tamanho`, `desloca_caminho`, `espera`, `assenta`), `Passo::SaiDaGravacao` (52, nunca gravado), `TocadorDoPainter::aplica_ao_tool`, `PainterTool::brush_size_px`, a cena toca `tocavel`, roteiro `grava_pinceladas` |
| `7f25bbf6b1` | Motion: `macro_track/etapas.rs` (`LeitorDoMotion`, `etapas()` com `TipoDaEtapa`, `edita_valor`, `espera`), `TocadorDoMotion::base`, `ErroDaMacro::Edicao`, a cena toca `tocavel`, `smoke::grava_gestos`; `quadros_da_espera`/`no_quadro` (cadência EXACTA do `dt`) |
| `665aed0bec` | gates das leis que a cena não exercia (cor herdada, conferência que diverge, sessões/arrastos multi-quadro, relógio de dentro, valor de um arrasto) |
| `95ca6db68e` | as 3 sobreviventes da mutação (a cena vê a espera; k quadros = k; a pincelada começa no toque) |
| `ff001e1bc7` | rustfmt nos ficheiros da W5b (só formato) |
| `83829932e7` | W5c: o razão da RAM passa a `ph2d_core::razao` (o Motion fica com `razao::da_fita`/`dos_aneis`, antes métodos de `Razao`) |
| `a6567f06fe` | W5c: `Clone` do `Engine` (sem `on_dab`), o `SolverScratch` nasce vazio na cópia, `bytes_retidos` |
| `898aa276c9` | W5c: `PainterTool::instantaneo` (padrão sem `..`), `SemInstantaneo`, `derive(Clone)` em 52 tipos de dados |
| `8c92e9cd19` | W5c: `MacroEmCena` com instantâneos por etapa, `reedita`, a dobra no razão; `MacroPlayer::advance_to_limit`; o palco reedita; gates + dhat |
| `d21a072d6b` | W5d: Gravar ▸ Parar; crates NOVAS `ph2d-app-macros` + `ph2d-panel-macros`; `Gravadora`; gate 7 |
| `a704aaa8aa` | W5d: rustfmt (só formato) |
| `9d4ffcf755` | W5d: a linha «Macro steps» na Timeline (arrastar re-cronometra a etapa e as seguintes) |
| `2329d6ddcf` | W5d: 3 ficheiros da W5c que os `derive(Clone)` levaram a 701/702 voltam a ≤ 700 por corte de comentário (`ph2d-wet-paint/src/painter.rs`, `tool/paint/state.rs`, `tool/paint/stamp_cache.rs`; gate `workspace_src_files_under_loc_cap`) |
| `e2f1455e9a` | W5d: cartão Macro no Inspector, undo das duas pilhas, o Parar que chega ao Painter |
| `a3475fb6fb` | W5d: as sobreviventes da mutação fechadas (M6/M8, M17, M11) → 18/18 |

## O que existe (não reconstruir)

- **Etapa** = função pura da gravação imutável (`Track.events`) + `Track.edicoes` ([plano §1](../02_plano_W5.md#1-o-que-é-uma-etapa-medido)).
  Painter: pincelada do toque ao levantar · recusa. Motion: lote do grafo (arrasto
  `BeginDrag..EndDrag` = um), sessão de param (mesmo `(nó,param)` com ≤ 1 `Quadro` entre lotes), `Escritas`
  (mesma regra), `Documento`, recusa. Relógio: Painter tique/água/drenos; Motion `Quadro`.
- **Espera** = relógio sintetizado a correr (`LeitorDaFamilia::espera`, cadência exacta do `dt` gravado:
  arredondar o quadro a µs punha o último quadro de 1 s fora — o gate apanhou); encurtar só tira relógio e recusa
  passar a etapa anterior (`EsperaMaiorQueOIntervalo`). Motion: o `tempo` dos quadros seguintes anda
  (`desloca`). Painter: a 1.ª espera insere `SaiDaGravacao` (a água larga a fita; pela fita DIVERGE — medido).
- **Edição de valor**: Painter cor/tamanho = inserção antes do toque + o antigo depois do levantar (só aquela
  pincelada; a seguinte que herdava fica com o seu); caminho = trocas a partir do GRAVADO (substitui). Motion
  valor = cada amostra anda `(novo − último)·(j+1)/n`. `assenta()` (Painter): edição de valor fica na fita; se
  a conferência divergir, a 1.ª etapa editada ganha `sai_da_gravacao`.
- **Nomes**: estruturados, sem texto — Painter `TipoDaEtapa::{Pincelada(n), Recusada}` + cor/tamanho no toque
  (lidos por um tool que só recebe os passos de configuração, `pinceis`); Motion `TipoDaEtapa::{Acrescenta,
  Liga{de,para}, Desliga, Apaga, Move, Renomeia, Param{no,param,de,para}, Grafo, Direta, Recusada}` (tipos de nó
  canónicos; o texto é do i18n `node.<tipo>.name` — W5d).

## Gates (todos verdes por nome, 14/10)

`ph2d-macro` `etapas` (7) + `format` (8) + `player` (6) · `ph2d-tool-painter` `etapas` (10 + 3 sondas ignoradas;
`macro_track::etapas_tests` 7, `macro_etapas` 3) e `macro` (26) · `ph2d-app-motion` `macro_track` (36 antes dos
novos; `etapas_tests` 7). ⭐ Verdade = GRAVAR a cena com a mudança: Painter cor/tamanho/espera e Motion
espera ±/valor dão o MESMO bit (nos dois modos do Motion), com o controlo «o original difere».

**Prova de mutação W5b (14/10, HEAD `665aed0bec`, agente `mutacao`): 18/21 sangraram; as 3 sobreviventes
fechadas em `95ca6db68e` e vistas a sangrar → 21/21.** Núcleo M1–M7 (espera sintetizada, `andou`, encurtar só
relógio, a marca de saída, `exige`, o minor 1.0, o limite do intervalo); Painter P1–P4, P7 (a cor que volta, a
marca, o tocador que larga a fita, o `assenta`, a cena que toca `tocavel`); Motion Q1–Q6 (sessão, `tempo`
deslocado e sintetizado, a proporção do valor, a cena, `de`/`para`). Sobreviventes: **M8** (a tolerância de
½ µs — gate novo: k quadros arredondados a µs são k), **P5** (a pré-visualização dos quadros da espera — gate
novo: a meio da espera a cena mostra a tela da gravação que esperou), **P6** (o início da pincelada no 1.º
evento do quadro = mutante EQUIVALENTE ⇒ o código passou a começar no toque). ⚠️ O agente reportou que
`cargo-test-narrow.sh <crate> <filtro>` pode imprimir «0 passaram» quando os testes vivem só em `tests/it`
(lê o 1.º `test result:`); na dúvida, `bash scripts/ph2d-run.sh cargo test -p <crate> <filtro>` e leia todos.

## W5c — feito (14/10)

Medições, desenho e o porquê: [plano §3.1](../02_plano_W5.md#31-os-instantâneos--medido-w5c-1410-smoke). Em
resumo: **60 MB a 1024² / 228 MB a 2048²** por instantâneo (o `SolverScratch` vazio na cópia tirou um terço),
tirar um custa **3,9 / 14,3 ms** com a água molhada; o tecto é o razão da RAM (`1/32`, por processo) e a macro
DOBRA os seus quando passa. ⛔ **Recusado (medido):** copiar o `SolverScratch` (+25 MB a 1024², conteúdo
reescrito inteiro em cada passo); um `Clone` PÚBLICO do `PainterTool` (o do editor tem worker e ponte de
dispositivo: a cópia mentiria) — a porta é `instantaneo()`, com recusa; um razão PRÓPRIO copiado do Motion (a
lei passou ao core; duas cópias divergem). A régua do razão conta por baixo (rasto do pincel, rascunho da
autoria): o gate dhat aceita retido ÷ declarado em `[1; 1,15]`.

Gates (por nome, verdes): `ph2d-tool-painter --test it macro_instantaneos` (7: continuar = do zero ao bit nos dois
relógios, recuo, edição cor/espera = palco novo ao bit com o controlo «o original difere», a dobra, a fita
reeditada, o instantâneo com uma resposta consumida mudada recusa, o razão do processo) · `--test
medida_instantaneo_memoria` (dhat) · `ph2d-app-painter --lib macro_track` (o palco que muda reedita na mesma
ranhura — a lei antiga «renasce numa ranhura nova» mudou de propósito) · `ph2d-core razao` · `ph2d-eval-motion
razao`. Sonda `#[ignore]`: `sonda_etapas::sonda_instantaneos`.

**Prova de mutação W5c (14/10, agente `mutacao`, 3 rondas): 13/13 sangram.** 1.ª ronda 9/13; as 4
sobreviventes eram portões meus fracos e fecharam-se: **M1** (as pistas da sessão) e **M2** (o pigmento) — os
instantâneos só se tiravam ENTRE pinceladas; o gate passou a tirá-los também a MEIO de cada pincelada e a
conferir o plano de pigmento da cópia (`PainterTool::wet_pigmento`; o seu único leitor fora do traço é «há
tinta?» ao trocar o Blend); **M10** (a fita própria da cópia) — o tocador copiava a fita uma 2.ª vez e mascarava
a falta: passou a ler a da cópia (`wet_fita`); **M13** (o espaçamento do passo) — o tecto do gate da dobra era
calibrado pelo tocador NO FIM (água assentada, ~0) e cada foto fora do passo fazia dobrar de novo: o tecto é
agora ~3,5 instantâneos com água e o espaçamento confere-se a cada quadro (M13 → `quadro 116: [0, 2, 4, 5] de 2
em 2`). As outras: o `Sim`, o `Roteiro`, a fila e a guarda da fita reeditada, o prefixo comum, a dobra, o
recuo, a régua do pigmento (dhat), o instante do `advance_to_limit`.

## W5d — feito (14/10)

Mecanismo, undo medido e decisões: [plano §5.1](../02_plano_W5.md#51-w5d--o-que-se-mediu-e-decidiu-1410).

- `ph2d_macro::Gravadora` (abre/passos/para) — `PainterTool` (alvo 0 = a sprite da Macro) e `MotionState` (o
  objecto em edição). Censo do Painter: `macro_abre/passos/para` = PORTAS `Decisao::Ambiente`.
  `PainterTool::bound_document`; `etapas::tela_no_fim` pública.
- **`ph2d-app-macros`** (crate nova): estado da família num `thread_local` (`com`); `Macros::{pede, quadro, info,
  etapas_na_linha, resumo, sincroniza}`; `Familias { painter (SEMPRE), painter_activo (só armar), motion }`,
  `Cena { sim, timeline, intents }`; módulos `sessao` (MacroRecorder, passos por quadro com o relógio do
  chamador), `instala` (Parar ⇒ Macro{Fiel p/ Painter, Explicativo p/ Motion} + clipe Autoplay, laço [0,D],
  activo; UM passo da pilha da Timeline), `etapas` (vistas com cache pelos bytes; edita cor/tamanho/valor/
  espera/nome; conferência `assenta` adiada ao 1.º quadro sem edição nova), `erros` (5.6: `panel.macros.error.*`).
- **`ph2d-panel-macros`** (crate nova; Window → Macros, ícone `Macro`, barra 851): Gravar/Parar (Gravar só se
  pinta com quem grave; senão a frase diz porquê), etapas nomeadas pelo i18n (`nomes.rs`), propriedades
  (renomear, Wait before, cor pelo selector partilhado, Size, Value), erro. Clicar numa etapa = Scrub ao fim dela.
- **Timeline:** `TimelineViewSnapshot::macro_etapas` (`EtapaDaMacroView`), `TimelineHitKind::MacroStep{index}`,
  `ph2d-panel-timeline/src/macro_steps.rs` (linha na vista Keys depois da última faixa; arrastar mostra a etapa
  e as seguintes a andar; ao largar `Espera(antiga+Δ)` por `EditorAction::Macro`; clicar = Scrub ao início).
  `TimelineDoc::{macro_clip_identity_end, retime_macro_clip}`.
- **Inspector:** secção MACRO (família Animação): modo Faithful/Explanatory, resumo, «Edited from step N», erro,
  «Edit steps…» abre o painel; `ComponentEdit::Macro`; LIVE_SECTIONS 51; comandos do Inspector 87→88.
- **editor-core:** `macro_edits` (EtapaView, MacroPanelInfo, MacroRequest, CampoDaEtapa, ErroDaMacroView,
  recusas), `EditorAction::Macro` (+2 linhas; o reflow de +78 do aviso do `action_bus.rs` já não acontece).
- **Shell:** `render_loop/fase_macros.rs` (composição, depois de `fase_painter_dispatch`), dreno do pedido em
  `fase_bus_inspector.rs`, publicação da linha em `fase_timeline_drain.rs`.

Gates (verdes, por nome, 14/10): `ph2d-app-macros --test it gate7` (4: Gravar→3 pinceladas→Parar ⇒ Macro + clipe
activo + Autoplay + 3 etapas, Scrub ao fim, cor pelo selector só naquela pincelada, arrastar na Timeline move a
etapa 2 e a 3 ~0,5 s e o clipe acompanha, undo do mundo devolve a duração, undo da Timeline tira o clipe; trocar
de ferramenta a meio e Parar fecha a gravação do Painter; a conferência adiada corre só no quadro seguinte e o erro
chega ao painel; a câmara lenta do artista não é re-cronometrada, com controlo) · `ph2d-panel-macros --test it`
(seam 4 + censo 3) · `ph2d-panel-timeline --lib macro_steps` (2) · `ph2d-panel-inspector --test it a_seccao_macro`
(4) · `ph2d-tool-painter --test it macro_portas` (5) · registry-init (armação do painel MACROS;
`nenhum_painel_e_medido_vazio_sem_o_declarar`) · editor-core arquitectura (every_panel_is_painted, icons, scrollbar,
menu, chrome_dispatch_in_sync, hr12, interactive_crate_has_behavioral_test, panel_wiring_parity,
hit_indexed_ids_are_registered, foundation dag, the_shell_only_shrinks, cycle_prevention, hr15, no_literal_color,
loc_cap).

**Prova de mutação W5d (14/10, agente `mutacao` + 4 re-corridas): 1.ª ronda 15/18; fechadas → 18/18 vistas a
sangrar.** Sobreviventes: **M6/M8** (a conferência adiada nunca correr / nunca ser pedida — a fixture em relógio
fixo não diverge: gate novo), **M17** (a forma do clipe ignorada: gate novo com a câmara lenta), **M11** (o valor
da espera só na Timeline: passou a ser lido no gate 7).

⚠️ **Riscos para o smoke (não exercidos em gate):** o palco em cena do Painter e o `PainterTool` vivo apontam a
mesma sprite depois do Parar; a gravação do Motion ao vivo depende de o Motion gravar `Quadro` a `Quadro` no
editor (só as cenas a exercem).

## ABERTO — a ordem

1. **W5e** — cenas `=1`/`=2` com as etapas à vista, fotos com a máquina calma, perf (compor 1× por quadro;
   premultiplicar/subir; a reprodução para editar pode saltar as composições intermédias; o custo da conferência
   adiada numa macro longa — ainda NÃO medido), fecho batched (clippy), handoff de integração, binário do smoke.
2. **Por decidir com o dono:** APAGAR uma etapa (plano §5 cita-o; fora do pedido da W5d; precisa da lei do que
   acontece à água/relógio).
3. Fica de antes: os outros meios do Painter. (5.6, erros na UI: feito na W5d.)

## Foundational / partilhado tocado (para o integrador)

`ph2d-macro` (formato 1.1: `Track.edicoes`, tags novas, `exige`, `CodecError::Exige` — leitor 1.0 do `main`
tocaria uma macro editada sem as edições; nenhum ficheiro de utilizador existe ainda; W5c: `MacroPlayer::
advance_to_limit`) · `ph2d-tool-painter` (`wetpaint.rs` +1 fn e +1 `derive`, `wetpaint/offthread.rs` +1 fn e o
`Clone` do `EngineSlot`, `brush_core_settings.rs` +1 getter, `macro_track/*`; W5c: `tool/instantaneo.rs` novo,
`tool/mod.rs` +3 linhas, `derive(Clone)` em ~40 ficheiros do `tool/paint/`) · `ph2d-app-motion` (`macro_track/*`;
W5c: `motion_state_fita_demo.rs` 1 linha) · **W5c:** `ph2d-core` (`razao.rs` novo — a lei do razão), `ph2d-eval-motion`
(`razao.rs` reexporta; `Razao::da_fita()`/`Razao::dos_aneis()` passaram a `razao::da_fita()`/`razao::dos_aneis()`
— um chamador novo noutra linha não compila e a cura é esse caminho; `Cargo.toml` +`ph2d-core`), `ph2d-wet-paint`
(`painter/instantaneo.rs` novo; `derive(Clone)` em `Grid`, `GridPatch`, `Layer`, `Sim`, `Stroke`, `StrokeEvent`,
`Trail`; `SolverScratch` com `Clone` à mão), `ph2d-painter-brush` (3 `derive`), `ph2d-app-painter` (o palco
reedita). ⚠️ **Com `line/PainterWatercolor`** (toca `tool/mod.rs`, `wetpaint.rs`, `trail.rs`): só linhas de
`derive`/`mod` minhas nesses três; um CAMPO novo que ela acrescente ao `PainterTool`, `WetSession` ou `Engine` não
compila na árvore combinada até entrar no padrão do `instantaneo`/`clone` — é o gate a funcionar, e a cura é
decidir se o campo viaja na cópia. Shell e contratos congelados: **nenhum**. Schema: não muda.

**W5d:** `ph2d-macro` (`gravadora.rs`) · `ph2d-editor-core` (`macro_edits`, `action_bus` +variante, `interaction/types`
+`TimelineHitKind::MacroStep`, `ids/chrome/macros.rs`, `chrome/macros_toggle.rs`, menus, ícone, `scrollbar_ids` 851,
LIVE_SECTIONS, `ComponentEdit::Macro`) · `ph2d-timeline` (snapshot +campo; `doc_owner` retime/identity) ·
`ph2d-panel-timeline` (`macro_steps`, state/state_drags, interact, geom, tracks, ids) · `ph2d-panel-inspector`
(secção macro) · `ph2d-app-components` (`macro_inspector`) · `ph2d-i18n` (`macros.rs` novo, timeline, chrome_menus,
inspector_macro) · `ph2d-tool-painter` (portas, `bound_document`, `tela_no_fim`, `impl Gravadora`) ·
`ph2d-app-motion` (`impl Gravadora`, palcos `erro_de`) · `ph2d-app-painter` (`erro_de`) · `registry-init` (registo +
fixturas) · shell (`Cargo` +2 deps/feature, `fase_macros`, drenos) · cortes de comentário em `ph2d-wet-paint`
(`painter.rs`) e `tool/paint/{state,stamp_cache}.rs`. Crates novas: `ph2d-app-macros`, `ph2d-panel-macros`.
Contratos congelados: **nenhum**. Schema: não muda.

## Medido (W5a, 14/10, smoke, load 17–28)

Tabelas no [plano §3](../02_plano_W5.md#3-editar-propriedades--medido-w5a-sondas-ignore-perfil-smoke-load-1728).

## ⚠️ Memória a corrigir no fecho

`project-memory/feedback_the_owner_of_a_scrollbar_id_is_the_dispatch.md` descreve a tabela
`scrollbar_panel_for_id`, que morreu a 29/09 (`ph2d-editor-core/src/interaction/state/scroll_state.rs:92`), e diz
que o skeleton pinta com `VECTOR_SCROLLBAR_ID` (é `SKELETON_SCROLLBAR_ID`, `ph2d-panel-skeleton/src/paint.rs:92`).

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`;
inner loop `bash scripts/cargo-check-narrow.sh <crate>` (⚠️ cego ao código atrás da feature `panel-motion-graph`
e aos `tests/`: use `bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --lib macro_track::etapas_tests`).
