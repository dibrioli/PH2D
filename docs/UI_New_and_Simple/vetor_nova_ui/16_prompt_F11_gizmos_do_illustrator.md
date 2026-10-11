═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F11: OS GIZMOS DAS FORMAS COMO O ILLUSTRATOR, A CAIXA COM O TRAÇO, E AFASTAR O CARTÃO E A ④
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie). Modo L.

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: FASE 0 → 1 → 2 do
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com MODULO=UIUX.
Estado em 11/10: F0–F8 no `main`; F9 + F10 FECHADAS na linha e NÃO integradas (`git cherry main HEAD`
dá os commits delas + este prompt: siga SEM reset). Se o `main` andou: `git rebase main`.

AS REGRAS DA JANELA (PROMPT_LINHAS_RAPIDAS_2026-10-11 §2 — valem do 1.º ao último passo):
R1 janela curta (~150 chamadas ⇒ commit + handoff de continuação ≤ 8 KB + PARE) · R2 `Edit`, nunca
script `python … .replace()` (só a MESMA troca em N arquivos, com assert) · R3 editar e testar nunca no
mesmo comando (o `cargo-check-narrow` NÃO compila os `#[cfg(test)]`) · R4 teste dirigido por NOME; a
suíte e o `nextest-impacted` 1× no fecho, pelo `verificador` · R5 nada de `until/sleep` · R6 chamadas
independentes juntas · R7 doc > 40 KB por `grep -n` · R8 escreva menos · R9 nenhum teste > 10 s.

O PEDIDO DO DONO (11/10, depois do smoke da estrela da F10) — palavra por palavra:
 *«Ainda não gostei dos nossos gizmos. Se for necessário simplifique. Veja como faz o Illustrator.»*
 *«O gizmo de transformação (comum a todas as formas) não engloba o stroke. Faça englobar.»*
 *«O modal das formas e a barra superior abrem muito próximos das formas. Afaste um pouco.»*

LEIA (dentro da worktree, nesta ordem, só o necessário):
  docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md (curto)
  docs/UI_New_and_Simple/vetor_nova_ui/15_mais_parametros_das_formas.md §1b (a estrela pelas fotos),
    §3 (a tabela das alças), §5–§6 (portas e recusas) — por `grep -n`, R7
  docs/UI_New_and_Simple/vetor_nova_ui/13_alcas_das_formas.md §1 (a lei: `Spot` × `Law`)

O QUE JÁ EXISTE (confira no código antes de acreditar numa ausência):
  • As alças: `ph2d-tool-vector/src/shape_handles.rs` (lei; `Spot`, `Law`, `handle_grip`, `inset`,
    `park`), `shape_handles_spots.rs` (o eixo, a caixa do contorno), `shape_handles_table.rs` (forma a
    forma — 42 formas). O mundo: `ph2d-app-vec/src/shape_handles_live.rs` (press/drag/Esc/duplo-clique,
    `draw`). O desenho: `ph2d-vec-render/src/shape_nodes.rs::draw_shape_handles` (hoje: todas iguais,
    pontos verdes `shape-handle`).
  • A caixa de transformação da forma vetorial: `ph2d-app-vec/src/vec_gizmo_view.rs::anchor_half` —
    nasce da `scene.path_curve_bbox(…)` (SÓ a geometria; o traço fica de fora). A âncora da ④ e do
    cartão: `ph2d-app-vec/src/bars_publish.rs::selection_on_screen` (`path_world_curve_bbox`, idem).
  • A ④ (a barra de ações flutuante — a «barra superior» do dono): `ph2d-editor-core/src/screens/hero/
    float_bar.rs::strip_rects` (a `Spacing::Lg` da seleção). O cartão lateral: `screens/hero/
    tool_bars_card.rs::card_gap` (`Spacing::Sm` + uma alça).

ESCOPO — produto final:
  0. ⭐ O ORÁCULO PRIMEIRO — o Illustrator pelas FOTOS (CLAUDE.md §0.9; não há build para Linux): o
     helpx (*Live Shapes*, *Live Corners*, *Shape widgets*), GIFs/capturas de tutoriais. Por forma
     (retângulo, retângulo arredondado, elipse/pizza, polígono, estrela, linha) e por widget: ONDE está
     (dentro do canto? na caixa? no contorno?), a FORMA e a cor dele (círculo oco, alvo ⊙, ⇕), o que faz,
     em que direção anda, o que aparece durante o arrasto (círculo tracejado, etiqueta «R2: 162 px»), e
     quando aparece/some (com a ferramenta de forma? com a seleção directa? some durante o arrasto?).
     Guarde as imagens FORA do repo (scratchpad) e escreva a leitura numa tabela no doc 17 (novo). O
     agente de pesquisa da F10 achou: R1 numa ponta, R2 num vale, ⊙ de canto um pouco para dentro, ⇕ das
     pontas na borda direita da caixa, círculo tracejado + etiqueta durante o arrasto, a caixa some
     enquanto se arrasta um raio — confirme e complete para as outras formas.
  1. ⭐ SIMPLIFICAR os nossos gizmos para o idioma do Illustrator: MENOS pontos, cada um com cara do que
     faz (o ⊙ de canto ≠ o ponto de raio ≠ o ⇕ de contagem), o retorno visual DURANTE o arrasto (o
     círculo/eixo tracejado e o número), e o que o Illustrator não põe no canvas vai ao CARTÃO (nada se
     perde: o dono disse na F10 *«não podemos reduzir parâmetros»* — simplificar é tirar do CANVAS, não
     do produto). Decida forma a forma com a tabela do item 0 e escreva no doc 17 o antes × depois. A
     lei de cada alça que fica continua a ser a mesma porta (`Spot` × `Law`; os gates automáticos
     `every_handle_sits_on_its_outline` e `the_hand_takes_the_handle_where_it_is_dropped`).
  2. ⭐ A CAIXA ENGLOBA O TRAÇO: a caixa de transformação (todas as formas e caminhos vetoriais) usa a
     bbox VISUAL (geometria + meia largura do traço, com o alinhamento do traço — dentro/centro/fora — e
     as pontas/juntas se o motor as expuser; confira em `ph2d-vec-scene` se já existe uma bbox com
     traço antes de escrever outra — `git grep -n 'stroke.*bbox\|bbox.*stroke'`). A MESMA bbox serve à
     ④ e ao cartão (`selection_on_screen`): uma pergunta, uma porta. ⚠️ Cuidado com o que ela move: o
     arrasto de escala da caixa (o que é «100 %» passa a incluir o traço?), o pivô, o snap às bordas, o
     enquadrar (F8) e os gizmos de sprite/Flip que partilham o `GizmoView` — meça cada consumidor
     (lição: «mudar a aritmética de um widget partilhado varre-se por CONSUMIDOR»). O que o Illustrator
     faz (*Use Preview Bounds*) e o Affinity: confira pelas fotos/manual e siga o produto do dono.
  3. AFASTAR o cartão e a ④ da forma: medir na foto a distância de hoje, propor a nova (por token
     `Spacing`, nunca literal) e prová-la com os gates de geometria do cartão
     (`tool_bars_card_tests.rs`) e da ④. Com a caixa a englobar o traço (item 2) a âncora já se afasta
     um pouco — meça DEPOIS do item 2.
  4. Cenas: refaça a 16 (a estrela) e acrescente o que mostra a caixa com um traço GROSSO (a caixa por
     fora do traço) — conte no roteador `ph2d-app-vec/src/vector_mode.rs` se é refeita ou nova;
     FOTOGRAFE (`docs/Components/ferramentas/fotografa_cena.sh`) e compare com as fotos do Illustrator
     ANTES do relatório.
  5. Gates; mutação de cada cláusula nova (agente `mutacao`, depois do commit); gate batched
     (`verificador`, 1×).
  FORA: as formas do Motion; integrar; enviar.

DECISÕES QUE NÃO SÃO SUAS: tirar qualquer PARÂMETRO do produto (do canvas pode, do cartão não); um
botão novo em barra/painel; a cor das alças fora dos tokens; instalar programas.
JÁ DECIDIDAS (não re-pergunte): alças no modo Forma, nunca no Select; o press nas alças/caixa ganha a
desenhar; o que não cabe numa alça vai ao cartão lateral; a fila ② fica com todos os números.

LIÇÕES DA F10 (não repita):
  · ⛔ O produto de referência do DONO decide a lei, não o oráculo: a torção do Inkscape no vale foi
    medida a 0,3° e reprovada pelo dono («ficou horrível»). Fotos do Illustrator/Affinity ANTES de
    desenhar.
  · ⛔ A caixa abraça o contorno COZIDO: uma alça que «estaciona» a partir do vértice vivo cai por baixo
    da alça azul quando o contorno arredonda — só a FOTO mostrou. Fotografe a forma depois de cada
    ajuste, não só a de omissão.
  · ⛔ A régua que mede só às amostras lê um ponto entre duas como «fora» (1e-3): meça aos segmentos.
  · ⛔ Um gate lê a condição da caixa NA SHELL pelo caminho (`the_object_box_condition_names_every_
    authoring_tool`): mover essa conta para uma crate partiu-o.
  · ⛔ Tecto de 700 LOC por ficheiro (os `*_tests.rs` também): parta ANTES de passar, em irmãos.

COMECE AUTOMATICAMENTE: FASE 0–2, reporte ao Enio em 3 linhas e SIGA com o item 0.

COMO TERMINA (DIRETRIZ §1.5.9): gate batched 1× · mutação · handoff de integração NOVO em
docs/UI_New_and_Simple/handoffs/ (cita o da F10) · CLAUDE.md §5.1 «Último:» (a entrada UI/UX tem 699
de 700 bytes — corte texto para caber) · `bash scripts/doc-index.sh` · rm -rf target/*/incremental ·
binário smoke 2× · smoke ao dono em passos numerados, sem jargão. NÃO integra, NÃO pusha.
═══════════════════════════════════════════════════════════════════
