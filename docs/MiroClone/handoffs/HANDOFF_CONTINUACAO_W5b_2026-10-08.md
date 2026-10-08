# HANDOFF DE CONTINUAÇÃO — `line/MiroClone` → W5b «Guias e alinhamento de objectos rodados» (2026-10-08)

> Para a PRÓXIMA JANELA desta linha. Antecessor: [integração W5](HANDOFF_INTEGRACAO_line_MiroClone_W5_2026-10-07.md)
> (o estado da linha, §10 o aberto, §12 os dois smokes do dono — **W5 aprovada**). Plano:
> [`../02_plano.md`](../02_plano.md) (§6 recusas medidas: leia antes de propor desenho).
> **Ordem do dono (08/10): ESTUDO detalhado primeiro, depois IMPLEMENTAR sem voltar a perguntar** —
> decisões técnicas são suas (padrão-ouro); só uma decisão de PRODUTO que o estudo não resolva volta
> ao dono, e em linguagem dele.

## §0 — Identidade

- Branch `line/MiroClone`, worktree `Worktrees/line-MiroClone`. Base `2a925859d` (o `main` não andou até
  08/10 — confira: `git rev-parse main`; se andou, `git rebase main` antes de tudo, MODELO_TROCA §FASE 1).
- O handoff de integração W5 está escrito e **não foi integrado**: a W5b continua por cima e fecha com
  um handoff de integração NOVO (`HANDOFF_INTEGRACAO_line_MiroClone_W5b_…`) que substitui o da W5 como
  «último» no CLAUDE.md §5.1 e diz o que mudou desde ele.
- Formato do quadro: `FORMAT_VERSION = 5`. Campo novo no documento = 6, com leitor do 5 e fixture
  GRAVADA pelo build antigo (idioma de `legacy.rs`). O `PROJECT_SCHEMA` não se toca.

## §1 — O pedido, nas palavras do dono (08/10)

> *«investigar e implementar melhor sistema de alinhamento e guias para objetos inclinados,
> considerando a possibilidade alinhar a rotação e criar linhas na rotação das formas. Deve estudar como
> os melhores apps fazem. O alinhamento de formas e frames inclinados não está bom, o alinhamento de
> frames nem sei se existe. Frames com formas dentro de si só o frame gera guias e alinha quando o frame é
> arrastado ou rotacionado, não buscando alinhamento entre formas dentro de frames.»*

Quatro queixas — **reproduza CADA UMA num teste vermelho antes de mexer** (onde um defeito ATERRA ≠ o
que o PROVOCA):
1. objectos INCLINADOS alinham mal;
2. ALINHAR A ROTAÇÃO (um objecto rodado a encaixar no ângulo de outro) não existe;
3. FRAMES: alinhamento «nem sei se existe»;
4. DENTRO de um frame (rodado ou não) as formas não procuram alinhamento entre si como deviam.

## §2 — O que o código faz HOJE (lido em 08/10 — confira, os números de linha andam)

- `crates/ph2d-board-edit/src/snap.rs` (`snap_box`): guias SÓ de eixos do mundo — 3 paragens por eixo
  (borda, centro, borda) da caixa ALINHADA AOS EIXOS; o mais perto dentro de `tol` px de ecrã ganha; a
  guia vai de ponta a ponta do que se alinha naquela linha.
- `gesture.rs` `targets()`: alvos = **só `el.shape()`** vivos que tocam a vista, pela `aabb()` ⇒ **frames
  nunca são alvo**, traços da caneta e setas também não; um rodado entra pela caixa ENVOLVENTE.
- Arrasto (`Gesture::Move`): a caixa que se encaixa é `union_aabb` dos originais que são `shape()` ⇒ ao
  arrastar um FRAME, quem alinha é a união dos FILHOS (o frame não é `shape()`); arrastar um frame vazio
  não alinha nada. Os alvos excluem tudo o que se move (lei de 08/10, teste
  `dragging_a_frame_does_not_snap_to_its_own_children` — **mantém-se**).
- Redimensionar e criar: encaixam só o PONTO do ponteiro (`snap_box([p, p])`).
- Rodar (`Gesture::Rotate`): nenhum encaixe a outros objectos; só `Shift` = passos de 15° (`ROTATE_STEP`).
- Um frame RODA (W5, 08/10) e leva os de dentro à volta do mesmo centro; a pertença e o clique lêem a
  caixa rodada (`box_contains`). As guias, essas, ignoram a rotação toda.
- Desenho das guias: `ph2d-board-render` `paint_overlay` (`overlay.guides`), linhas de mundo.
- ⚠️ `gesture.rs` está em **698/700** linhas: CORTE antes de acrescentar (mova o encaixe para `snap.rs`
  ou um módulo novo — cura por mover, nunca subir o tecto).
- Custo nunca medido: os alvos são TODOS os vivos à vista, varridos a cada movimento do rato.

## §3 — Fase A: o ESTUDO (um documento, antes de uma linha de código)

Escreva `docs/MiroClone/03_guias_e_alinhamento.md` com:

1. **Como os melhores apps fazem**, aplicação a aplicação, com a FONTE de cada lei (help oficial com
   citação e URL, ou corrida medida):
   - produto de referência do dono: **Miro** (smart guides, espaçamento igual, objectos rodados, frames)
     — help público (a API Zendesk do §2 do briefing W5 funciona) + capturas do dono se precisar (peça-as
     no fim, em passos numerados, nunca a meio);
   - **Figma** (smart guides, rotação, frames/auto-layout, `Shift` ângulos), **Excalidraw** (snapping de
     objectos e de vãos, rotação), **tldraw** (MIT: snapping de limites e de pontos, rotação), **Penpot**,
     **Inkscape** (encaixe por ângulo, ao longo de rotações, perpendicular), **Affinity/Illustrator**
     (guias de construção em ângulos), **Godot** (MIT, passo de rotação) — os que não se puderem correr,
     pela documentação oficial, marcados como tal.
2. **Oráculos que se CORREM** (§0.9 — nunca ler o fonte): o Excalidraw 0.18.1 já instalado
   (`ferramentas/excalidraw_oracle/`, Playwright; README §W5 ensina a montar o editor e a ler a cena) e o
   **tldraw** (instalar ao lado, triar a licença por ARTEFACTO instalado). Cenas NOSSAS: rectângulos
   rodados (15°, 30°, 45°, ângulos quaisquer), um frame rodado com filhos, dois frames, vãos desiguais.
   Grave a saída (posição/ângulo final de cada arrasto e as linhas de guia quando o estado as expõe) como
   fixtures `saidas/guias_*.json` com `meta.json`; duas corridas byte a byte iguais.
3. **A tabela lei → fonte → número** de cada uma destas perguntas (é ela que decide o desenho):
   - um objecto rodado: alinha pela caixa envolvente, pelas bordas e cantos RODADOS, ou pelos dois?
   - linhas «na rotação das formas»: guias paralelas/prolongando as bordas de um rodado; quando aparecem
     (mesmo ângulo? qualquer ângulo? só o de baixo do ponteiro?); que comprimento;
   - **alinhar a rotação**: ao rodar, encaixar no ângulo de outros objectos (à vista? mais perto?), em
     0°/90°, com que tolerância (graus? px no arco?), e como se mostra;
   - frames: como ALVO e como quem se MOVE (a caixa do frame, rodada); os filhos de um frame arrastado
     também procuram alinhamento com o que está fora?;
   - **dentro de um frame rodado**: as guias dos filhos no referencial do FRAME (eixos locais) — que apps o
     fazem e como;
   - espaçamento igual (o Miro e o Figma mostram-no): entra nesta onda ou não, e porquê;
   - o que a tecla de suspender (`Ctrl`, hoje) e o `Shift` (15°) passam a fazer.
4. **As recusas** que o estudo justificar (tabela `⛔ Recusas MEDIDAS`, como a do plano §6).
5. **Medir antes de limitar (§0.0):** o custo do encaixe por movimento com 1 000 / 10 000 / 100 000
   elementos à vista (régua nova ao lado de `ph2d-board-render/tests/it/measure_*` ou em `ph2d-board-edit`),
   variantes no MESMO processo, intercaladas, o mínimo. Só se a régua mandar: índice espacial dos alvos
   (com a tabela ao lado), nunca um `MAX_*` de alvos sem o recurso que o justifica.

## §4 — Fase B: IMPLEMENTAR (sem voltar ao dono)

- Leis no `ph2d-board-edit` (o encaixe sai de `gesture.rs` — o corte primeiro), desenho das guias rodadas
  no `ph2d-board-render`, nada na shell. Mantém: alvos excluem o que se move; `Ctrl` suspende; `Shift`
  15°; frames agarram-se pelo miolo vazio; redimensionar um frame não escala os filhos.
- Gate por lei com os números do oráculo TRANSCRITOS no cabeçalho do teste (idioma de `organize_tests.rs`),
  e a divergência ESCOLHIDA do oráculo escrita no teste quando o Miro manda outra coisa.
- Pixels pela placa (`frames_paint`, `PH2D_GPU=1`) para uma guia RODADA desenhada no ângulo certo.
- Barra: se nascer controlo novo (ex.: ligar/desligar o encaixe), passa pelo arnês `shell_click_bar` e
  pelo gate «nenhuma zona de gesto da shell sobre um botão da barra»; zero texto fora do i18n; nada de
  `←→↑↓` em frases (a letra da interface não as tem — tofu); números de desenho com `LITERAL-PX-OK`
  em constantes com nome, uma por linha (o `rustfmt` separa a marca do número).
- Cena de smoke **7** (`PH2D_BOARD_SMOKE=7`; o número conta-se em `ph2d-app-board/src/smoke.rs`,
  hoje `max_level: 6`): formas rodadas a 30° e 45°, um frame rodado com filhos, dois frames — o que o
  dono arrasta e roda para VER as guias novas. **Fotografada** antes de mandar
  (`docs/Components/ferramentas/fotografa_cena.sh`), de longe e de perto.

## §5 — Fecho da W5b

`/pd-linha-fechar`: portão batched 1× sobre `2a925859d..HEAD` (`nextest-impacted` com `PH2D_GPU=1`,
clippy do `ship.sh` `-D warnings`, `fmt`, `ph2d-editor-core --test it` inteiro, os gates da shell, os
censos da árvore combinada), auditoria de 2 lentes (correcção das leis · costura/desenho), **prova de
mutação** em cada lei nova (4 controlos: verde antes, filtro que casa, âncora única, árvore restaurada), o
handoff de integração W5b, o CLAUDE.md §5.1 a apontá-lo, `rm -rf target/*/incremental`, e o binário
`smoke` compilado 2× (a 2.ª em segundos, colada no handoff). Ao dono: o que ele consegue FAZER agora e o
smoke em passos numerados (comando com `cd` · onde clicar · o que tem de acontecer · como saber que deu
errado). **Não integre nem faça push.**
