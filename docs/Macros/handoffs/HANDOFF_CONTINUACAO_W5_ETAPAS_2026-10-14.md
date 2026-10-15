# Continuação — line/Macros-Automação, W5 (ETAPAS): W5a + W5b feitas → W5c (instantâneos)

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

## ABERTO — a ordem

1. **W5c — instantâneos do tocador do Painter** (custo medido: 245 ms/s a 1024², 537 ms/s a 2048² do zero; plano
   §3). Leitura feita (14/10): `PainterTool`/`Engine`/`WetSession` não são `Clone`; uma cópia profunda custa
   **≈ 83 MB a 1024², ≈ 332 MB a 2048²** por instantâneo (tela 4 + `images` 4 + grelha viva 63 B/célula +
   `pigment` 4 + `base` 4; mais 19 B/px por camada com relevo/vidro; as pilhas de undo do `Engine` até +612
   B/célula) ⇒ 20 etapas a 1024² ≈ 1,7 GB. **Desenho a medir:** o molde é o `ModelSnapshot` do undo estrutural
   (`tool/layers/undo.rs:14-30`, campos `Arc` em COW) para as camadas + a grelha como `GridSnapshot`
   (`ph2d-wet-paint/src/grid.rs:526`, 51 B/célula, já `Clone`) + o estado da sessão (sim, trail, stroke, fita
   em cópia PROFUNDA da fila, o `Weak` da tela re-armado como `wetpaint_rearm_after_own_write`); `on_dab` e o
   `SimWorker` ficam `None` (o tocador não os tem — confirmar). Intervalo entre instantâneos = do orçamento de
   MEMÓRIA medido (CLAUDE.md §0.0), nunca um número escolhido. Ficheiros que mudam e colidem com
   `line/PainterWatercolor`: `tool/mod.rs`, `tool/paint/wetpaint.rs` — preferir os irmãos (`session.rs`,
   `offthread.rs`, `grid.rs`, `undo.rs`), que ela não toca. Gate: instantâneo + avançar == tocar do zero ao bit;
   editar a k recomeça do instantâneo k.
2. **W5d — UI** (plano §5): `ph2d-panel-macros` (Gravar ▸ Parar nas duas famílias — `macro_abre` do Painter passa
   de INTERNA a pública com decisão no censo `macro_portas`; lista de etapas; propriedades; renomear; erro 5.6),
   a linha de etapas na Timeline (campo novo no snapshot; losangos como `paint_lane_keys`; arrastar = evento lido
   pela família; ripple + D′/laço/`length_override` numa transação), cartão Macro no Inspector (molde
   `a777f2f541`), undo (o `Macro` pelo diff do mundo, o clipe pelo histórico PRÓPRIO da Timeline — medir o
   Ctrl+Z das duas pilhas). Gate 7 na ui-testkit.
3. **W5e** — cenas `=1`/`=2` com as etapas à vista, fotos com a máquina calma, perf (compor 1× por quadro;
   premultiplicar/subir; a reprodução para editar pode saltar as composições intermédias), fecho batched, handoff
   de integração, binário do smoke.
4. Fica de antes: 5.6 (erros na UI — vem com o painel), os outros meios do Painter.

## Foundational / partilhado tocado (para o integrador)

`ph2d-macro` (formato 1.1: `Track.edicoes`, tags novas, `exige`, `CodecError::Exige` — leitor 1.0 do `main`
tocaria uma macro editada sem as edições; nenhum ficheiro de utilizador existe ainda) · `ph2d-tool-painter`
(`wetpaint.rs` +1 fn, `wetpaint/offthread.rs` +1 fn, `brush_core_settings.rs` +1 getter, `macro_track/*`) ·
`ph2d-app-motion` (`macro_track/*`). Shell e contratos congelados: **nenhum**. Schema: não muda.

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
