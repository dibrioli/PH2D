# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-10 — F7e: O EDIT SHAPE

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Jornada anterior: [F7d, 10/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-10_F7d_QUICKSHAPE_SETAS.md)
> — citada, não reescrita. F0–F7d: os handoffs delas.

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `2a925859d` (o `main` não andou — sem rebase)
  · F0–F7d continuam por integrar e vão JUNTAS com esta jornada.
- Commits desta jornada: `8bfc4157e..HEAD`.
- **O pedido (dono, 10/10, depois do smoke OK das setas):** de três opções, *«ajustar a forma
  depois»* — o «Edit Shape» do Procreate: a forma perfeita acabou de nascer e o artista AFINA-A com
  nós próprios dela, sem ir ao modo de nós genérico.

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

UI/UX: «Último:» → este ficheiro; a frase ganha «e Edit Shape»; o smoke `…|11|12` → `…|6..13`
(690 → 686 de 700 bytes).

## §1 — O que mudou

| peça | o que é | onde mora |
|---|---|---|
| ⭐ **os oráculos** | o Excalidraw `next` (a forma que o «draw shape» dele reconhece, editada no editor REAL) e o tldraw 5.5.2 (as formas nativas; **«tldraw license», não permissiva: corre-se, não se porta**), corridos com o rato do Playwright: que alças cada forma tem e o que arrastar cada uma faz. 22 fixtures, 2 passagens com as formas iguais | `oraculo_quickshape/corre_excalidraw_edita.mjs`, `corre_tldraw_edita.mjs`, `fixtures_*_edita/`; `00_resultados.md` §3i |
| ⭐ **a lei** | `quickshape::edit::{nodes, drag, scale}` — os nós PRÓPRIOS por tipo e o tipo nunca muda: rectângulo pelos cantos (oposto fixo, rotação guardada, Shift = proporção pela maior razão); elipse pelos extremos dos eixos (as alças de lado do tldraw); linha/polígono livres (Shift a 15° pelo lado que chega); arco/biarco por início · ápice · fim (as pontas guardam a flecha, o ápice só a mexe; o de círculo continua de círculo, o de elipse e o biarco pela afinidade EXACTA da corda); a seta pelo cabo; o corpo escala | `ph2d-vec-edit/src/quickshape_edit.rs` (novo) |
| **o ciclo no lápis** | a forma SOBREVIVE ao soltar (oferecida) até ao gesto seguinte; o botão abre/fecha os nós; o press agarra um nó ou o contorno (o mesmo caminho fica `active`); o arrasto escreve em LOCAL pelo afim da caneta; soltar = um passo de undo; `Esc`/botão direito a meio repõe; a validade é a geometria (outro escritor acaba-a; o undo reencontra a forma) | `ph2d-vec-edit/src/pencil_edit.rs` (novo); `pencil.rs` (as portas) |
| **a entrada** | o botão **Edit Shape** (ícone de nós) À FRENTE da barra de ações da seleção ④, só com a forma oferecida, aceso com os nós abertos; a dica ⑤ do Edit Shape; o clique fica no painel e o QUADRO do lápis consome-o | ids/estado/`bars.rs`/`event.rs`/`populate_modes.rs` do painel; i18n `vector_bars.rs`; `ph2d-app-vec/src/quickshape_frame.rs` |
| **o desenho** | os nós com a gramática dos pinos do Envelope (`draw_shape_nodes`); as alças genéricas da forma afinada saem enquanto os nós próprios estão à vista | `ph2d-vec-render/src/shape_nodes.rs` (novo); `ph2d-app-vec/src/edit_marks.rs` (novo) |
| **a shell encolhe** | o press do lápis → `pencil_input::press` (agarra um nó com a mão CRUA: o estabilizador é do traço); o anel do nó soldado → `edit_marks::draw` | `despacho_clique_vetor_premido.rs`, `fase_vector_edit_overlay.rs`: **+24 −35** |
| **cena 13** | o rectângulo tosco seguro, o botão da ④ pelo clique real, o canto de baixo-direita arrastado pelo press real; foto [`f7e_01`](../vetor_nova_ui/fotos/f7e_01_cena13_o_edit_shape_aberto_e_um_canto_arrastado.png) | `ph2d-app-vec/src/vector_mode_quick.rs` (estágios 55–56), `vector_mode.rs` (o roteador) |

## §2 — Medido e decidido (técnico, delegado)

Tabelas em [`00_resultados.md`](../vetor_nova_ui/oraculo_quickshape/00_resultados.md) §3i.

- **Os gates LEEM as fixtures:** 32 arrastos de canto (Excalidraw rect/quadrado; tldraw rect e rect
  rodado 25°, livre e Shift) + 16 de lado da elipse/círculo do tldraw + as pontas da linha e o Shift
  a 15° do Excalidraw + a seta curva do tldraw (o ápice cai na alça do meio dele) — todos a 0,01 px.
- **Ao soltar** o Excalidraw deixa a ferramenta na mão e NADA selecionado, e um arrasto num canto
  desenha outra forma ⇒ os nós NÃO aparecem sozinhos (o Procreate também pede o botão).
- **A forma não se reconstrói do caminho** (ele não sabe se era rectângulo ou quadrilátero): vive no
  lápis até ao gesto seguinte — sem componente, sem schema. A geometria é a prova de validade
  (tolerância 1e-6 do tamanho: o assentamento soma e subtrai o mesmo número).
- **A ordem do quadro** (conferida): assenta → afins → `pen.set_xforms` → o quadro do lápis; o afim
  que o Edit Shape usa é o deste quadro.
- **Nenhuma constante de produto:** `MIN_FRACTION` e `RIGHT_COS` (1e-6) são guardas numéricas.
- **⛔ Recusas:** reconstruir do caminho; nós sempre à vista ao soltar; o nó do meio que acrescenta um
  vértice (muda o tipo); arredondar a rotação no corredor (0,02 px — instrumento, corrigido).

## §3 — Foundational tocado e contratos

- **`ph2d-vec-edit`** (motor novo, não congelado): `pub mod quickshape::edit` (`nodes`, `drag`,
  `scale`); `Pencil` ganha o campo privado `edit` e `shape_edit_{state,path,nodes,shape,press,
  regular,escape,tick}`, `toggle_shape_edit`, o `pub enum pencil::ShapeEditState`; `on_press` fecha a
  oferta, `on_drag`/`on_release`/`cancel`/`width_stops` sabem do arrasto; `PenTool::xf` passa a `pub`.
  Dev-dep `serde_json = "1"` (já no lockfile).
- **`ph2d-panel-vector`**: id `VECTOR_PENCIL_EDIT_SHAPE` (`hash_node_id("vector.pencil.edit_shape")`),
  `set_current_shape_edit`, `request_shape_edit_toggle`, `take_shape_edit_toggle`.
- **`ph2d-vec-render`**: `draw_shape_nodes`. **`ph2d-app-vec`**: `edit_marks`,
  `pencil_input::press`, `PencilHand.raw` (privado).
- **`ph2d-i18n`**: `panel.vector.pencil.edit_shape`, `panel.vector.hint.pencil_edit_shape` (+2).
- **Contratos congelados (§6): nenhum.** `Tool=12`/`PanelEvent=4` intocados (o clique não vai à tool).

## §4 — Superfície de colisão e contadores

- Cenas `PH2D_OBJECT_MODE_SMOKE`: **+13**. Sem schema, sem `PROJECT_SCHEMA`, sem registos.
- **Shell (`numstat` de `src`):** +24 −35 nesta jornada (o press e a marca do nó soldado mudaram-se).
- **Em silêncio:** outra linha que mexa no press do lápis em `despacho_clique_vetor_premido.rs` ou no
  bloco do nó soldado de `fase_vector_edit_overlay.rs` (mudaram-se para a crate); quem chame
  `Pencil::on_press` esperando que a oferta do Edit Shape sobreviva (fecha-a).

## §5 — Gate final

Uma corrida do `verificador` (`BASE=2a925859d`, HEAD `370b4237f`, `load` 1 → 25 com as outras linhas):
warnings-deny ✓ · machete ✓ · standalone-optional ✓ (10) · workflow-packages ✓ (32 / 372) · censos
114/114 e 12/12 ✓ · `doc-index` ✓ (17) · `adr-index` ✓ (185) · catracas ✓ (`the_shell_only_shrinks`,
LOC, `file_loc_caps`, `no_orphan_source_file`, `architecture_claude_md_cabe_no_orcamento`) ·
`nextest-impacted` **17 331 / 17 334** (9 545 pulados). ✗ fmt (51 blocos), ✗ clippy (4: `clone_on_copy`,
3× `manual_is_multiple_of`), ✗ 3 testes de `the_pencil_owns_its_whole_gesture` — deterministas: o
gate lia a SHELL e procurava `self.vec.pencil.on_press(`, que se mudou para `pencil_input::press`.
**Curas** (`a808ba194`): o gate SEGUE o press para a casa nova (a chamada no despacho + a porta da
crate tem de abrir o traço, lida no ficheiro dela — vista VERMELHA sob «a porta não abre o traço» e
verde restaurada); clippy e fmt. Depois: clippy do `ship.sh` na árvore inteira ✓, fmt ✓, o gate do
gesto 6/6, as cinco crates tocadas **1 584 / 1 584**. O `nextest-impacted` inteiro NÃO foi re-corrido
depois de `a808ba194` (só formatação, dois lints e o gate).

### Mutação

Agente `mutacao` (verde visto antes, âncoras pré-voadas 1×, restauro `git checkout` + `touch`, árvore
limpa no fim): **39 de 42 sangram à primeira** — a lei inteira (o Shift a 15°, o lado da polilinha
aberta, o rectângulo e a maior razão, o oposto fixo, o eixo da elipse e o Shift dela, o ápice, a
flecha guardada, o paralelo ignorado, o círculo que continua círculo, o sentido do arco, o espelho da
SVD, o cabo da seta, o pivô do corpo), o lápis (o agarrar fora do centro, o contorno, o toque fora, o
cancel, as três condições do tick, o histórico do undo, o LOCAL, a tolerância, a oferta, o fecho no
traço seguinte), o quadro (o clique da ④, o Shift, o `Esc`, a ferramenta), a mão crua, o overlay e o
painel (o botão, aceso, a dica, o braço do clique, o consumo único). Das 3 sobreviventes: **1 buraco
real** — o recuo da ponta (só uma ponta ARMADA com recuo, o triângulo, o distingue): testemunha
`an_armed_head_that_recedes_the_path_keeps_receding_it_while_the_tip_is_dragged`, vista VERMELHA sob a
mutação; **2 equivalentes, retiradas do código** — o descarte do degrau repetido de um clique sem
arrasto (o undo procura pela geometria: invisível) e o perfil uniforme durante o arrasto (o lápis não
tem amostras, e com menos de 3 o perfil já é uniforme).

## §6 — Smoke do dono

Foto [`f7e_01`](../vetor_nova_ui/fotos/f7e_01_cena13_o_edit_shape_aberto_e_um_canto_arrastado.png):
o rectângulo com os quatro nós próprios (pontos), o botão Edit Shape aceso à frente da ④, a dica no
⑤; o painel diz 190 × 120 (o rectângulo seguro tinha 150 × 90: o canto andou e continuou rectângulo).

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=13 ./target/smoke/ph2d-host-desktop`
2. A cena desenha sozinha um rectângulo tosco com o Lápis, segura-o (vira perfeito), clica o botão
   **Edit Shape** (o primeiro da barrinha em cima da forma) e arrasta o canto de baixo-direita.
3. Tem de ficar: um rectângulo maior, ainda perfeito, com quatro pontos nos cantos; o botão aceso; no
   rodapé «Drag a node: adjust · …».
4. Errado: o canto torto (virou quadrilátero), sem pontos, ou sem o botão.
5. Na mesma janela, com o Lápis: desenhe e SEGURE outras formas (círculo, arco, linha, seta,
   triângulo); ao soltar, **Edit Shape** e arraste os pontos — o círculo vira elipse (com `Shift`
   continua círculo), o ponto do meio do arco abre/fecha a curva, o bico da seta leva a ponta, o
   CONTORNO (fora dos pontos) aumenta a forma inteira; `Ctrl+Z` desfaz uma afinação de cada vez.
6. Sair: clique fora da forma, outro traço, `Esc` ou outra ferramenta.

## §7 — O que fica ABERTO

- **Perguntas ao dono (produto):** (1) o lugar do botão — à frente da barra de ações junto da forma
  (decidido por ser a barra das ações da seleção e ligada por decisão dele de 07/10); o Procreate
  põe-no na barra de cima — confirme; (2) arrastar o CONTORNO escala (o manual do Procreate) — ou
  deve MOVER (o Excalidraw e o tldraw)?
- O Edit Shape só existe logo depois do QuickShape (o do Procreate): uma forma de ontem afina-se no
  modo de nós genérico.
- Um arco de elipse ou um biarco afinados deixam de ter arcos de círculo (a afinidade guarda a
  tangência e a elipse, não o círculo) — dito no doc da lei.
- Os oráculos editam com o rato; a mão (o dedo) nunca afinou uma forma no nosso app.

## §8 — Binário de smoke (2.ª corrida)

(preencher)

## §9 — `agent-loop-profile.sh`

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                257   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                855 : 213   alvo: <= 1,0  razao 4.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (1298 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         444 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               62 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
(Edições em lote por `python3` com `assert` de contagem — vários sítios da mesma peça de uma vez.)
