═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F12: TIRAR O CARTÃO (MODAL) DE TODAS AS FORMAS, AS SETAS NO LUGAR DELE,
        O FURO E O CANTO DO CÍRCULO/PIZZA DE VOLTA, A ESPIRAL, E O TRAÇO QUE «BUGA»
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie). Modo L.

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: FASE 0 → 1 → 2 do
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com MODULO=UIUX.
Estado em 11/10: F0–F8 no `main`; F9 + F10 + F11 FECHADAS na linha e NÃO integradas (siga SEM reset;
se o `main` andou: `git rebase main`). O último handoff de integração:
docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-11_F11_GIZMOS.md.

AS REGRAS DA JANELA: as R1–R9 do topo de
docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md (R1 = ~150 chamadas ⇒
commit + handoff de continuação ≤ 8 KB + PARE). ⚠️ Esta onda é GRANDE: planeie-a em sub-ondas (abaixo)
e espere fechar pela R1 a meio — o handoff de continuação diz qual sub-onda vem a seguir.

O PEDIDO DO DONO (11/10, depois do smoke da F11) — palavra por palavra:
 *«smoothing no retangulo nÃO é necessário pode tirar»*
 *«Corner pode ser representado no gzimo com a mesma seta que representa a quantidade de pontas na
  estrela. Pode tirar o modal»*
 *«A estrela ficou maravilhosa.»*  ⇐ a estrela é a REFERÊNCIA: não mexa nela; copie o idioma dela.
 *«O gzimo do espiral não funciona. Para expansion e Inner crie setas e tire o modal. Faça isso para
  todas as formas que usam o modal.»*
 *«Para o círculo não foi uma boa idéia tirar o furo central. Volte com ele e coloque um ponto para
  ajustar os raios das quinas quando o círculo não está inteiro. Faça o mesmo para pie.»*
 *«Poly ficou todo bugado (fotos) com stroke bizarro e perdeu o ponto de ajuste da curva da aresta.
  Corrija tudo e volte o ponto de ajuste. Curve e Smooth viram setas como eu já disse.»*
 *«setas ficaram boas mas veja que a depender da altura da seta o stroke buga.»*
 *«Coração a depender do ajuste o stroke buga . O mesmo acontece com Cloud e vários outros.»*
 *«Vamos corrigir um por um e tirar os modais de todos. As setas de ajustes que ficam no lugar dos
  ajustes do modal podem ficar na lateral uma do lado da outra»*
 (Sem resposta ainda: a pergunta da F11 sobre o *Resize Box* manter a espessura do traço — não a
  decida; ela segue aberta no handoff da F11 §7.)

AS FOTOS DO DONO (fora do repo, já guardadas): /var/tmp/ph2d_F12_dono/ — LEIA-AS antes de medir.
A leitura da janela anterior (HIPÓTESES — meça antes de acreditar):
  1 `1_poligono_4_lados…` (Curve 0,010): esporões em «X» nas quinas de cima, esquerda e direita, a
    sair da caixa — o traço com junta MITER onde a aresta curva chega à quina com tangente quase
    paralela à vizinha (o bico do miter dispara).
  2/3 `pentagono…smooth −0,020 / −0,010`: abas e dentes por DENTRO das quinas — o *Smooth* negativo
    deixa um laço/cúspide minúsculo na geometria cozida, e o lado de dentro do traço desenha-o.
  4 `seta_dobrada…`: a ponta da farpa da cabeça CORTADA numa altura e inteira noutra — o miter passa
    do limite e cai para BEVEL num salto (a quina aguda muda com a altura da seta).
  5/6 `coracao…`: o mesmo salto miter↔bevel no bico entre os lóbulos (comprido numa, cortado noutra).
  ⇒ É UMA FAMÍLIA (CLAUDE.md §0.10): o traço sobre quinas agudas e laços da geometria cozida, em
  TODAS as formas (o dono: «Cloud e vários outros»). Resolva num bloco: um censo de todas as formas ×
  ajustes (a faixa de cada campo) que procura (a) o salto miter→bevel ao variar um campo, (b) laços /
  segmentos invertidos na geometria cozida, (c) o bico do miter fora da caixa visual — antes de curar.
  O ORÁCULO: como o Illustrator e o Inkscape tratam o limite do miter (o Illustrator: *Miter Limit*
  10 por omissão; o Inkscape: 4) — confirme pelo manual/foto ou pelo Inkscape por script (CLAUDE.md
  §0.9; o Inkscape está instalado, `oraculo_alcas/` mostra como se corre). A cura certa é decidir
  pela lei do produto do dono, não remendar forma a forma.

O QUE JÁ EXISTE (confira no código antes de acreditar numa ausência):
  • A lei das alças: `ph2d-tool-vector/src/shape_handles.rs` (`Spot`, `Law`, `handle_grip`,
    `HandleDesc::in_corner`), `shape_handles_table.rs` (forma a forma), `shape_handles_face.rs`
    (`glyph_of`: ⊙ ⇕ ○ — a cara sai da LEI), `shape_handles_spots.rs`. O ⇕ das pontas da estrela é
    `Spot::Box` + `Law::Steps` (doc 15 §1b, doc 17 §4) — é ESSA seta que o dono quer para o Corner,
    o Curve, o Smooth, o Expansion e o Inner.
  • O cartão lateral (o «modal» do dono): `ph2d-editor-core/src/screens/hero/tool_bars_card.rs` e os
    campos que nenhuma alça move (`shape_handles::fields_without_handle` — é CONTA sobre a tabela:
    quando todo campo tiver alça o cartão some sozinho; confirme que some, não que fica vazio).
  • O que a F11 tirou do canvas (doc 17 §4): a envergadura do polígono/estrela, o furo da elipse/
    pizza, o raio interno da espiral → cartão. O dono quer o furo e a envergadura (o «ponto de ajuste
    da curva da aresta», `Spot::EdgeMid`) DE VOLTA no canvas.
  • O traço: `ph2d-vec-render` (`stroke_plan`, `visual_bounds`), as juntas/limite de miter no
    `StrokeSpec` (`ph2d-vec-scene/src/stroke_style.rs`), a geometria cozida em `ph2d-vec-scene`
    (`round`, `shapes`, `cook`).

ESCOPO — produto final, em sub-ondas (por esta ordem; cada uma fotografada antes de passar):
  A. ⭐ O TRAÇO (a família acima): o censo, o oráculo, a lei, a cura, um gate por espécie (o salto
     miter↔bevel; os laços; o bico fora da caixa) que corre sobre TODAS as formas.
  B. O POLÍGONO: o ponto da curva da aresta de volta (`EdgeMid`), *Curve* e *Smooth* como setas ⇕.
  C. O RECTÂNGULO: o *Smoothing* SAI do produto (ordem do dono: o campo, a fila ② e o cartão — e o
     save: um campo que sai de um `ShapeValues` é foundational; confira o `PROJECT_SCHEMA`/migração
     antes, e se for preciso migrar diga-o no handoff); o *Corner* (tipo do canto) como seta ⇕.
  D. O CÍRCULO e a PIZZA: o furo central de volta (a alça); e um ponto novo que arredonda as QUINAS
     quando o arco não é inteiro (o círculo aberto e a pizza) — confira se o `cook` já tem um raio de
     quina para o arco; se não tiver, é um parâmetro NOVO (apenso, neutro em zero — a regra da F10).
  E. A ESPIRAL: medir porque o gizmo «não funciona» (reproduzir pelo press/arrasto da shell, como as
     cenas 15–17) e curar; *Expansion* e *Inner* como setas ⇕.
  F. TODAS as outras formas que ainda abrem o cartão: cada campo do cartão vira uma seta ⇕; as setas
     ficam na LATERAL da caixa, uma ao lado da outra (o dono), cada uma com a ficha do nome durante o
     arrasto (`handle_readout`, como o «Points: 7»). No fim, nenhuma forma abre o cartão.
  G. Cenas: refaça as que mudarem (`vector_mode.rs` conta-as; a próxima nova é a `=18`), fotografe
     TODAS e compare com as fotos do dono ANTES do relatório.
  FORA: a estrela (está aprovada — só herda a cura do traço se ela também saltar); integrar; enviar.

DECISÕES QUE NÃO SÃO SUAS: tirar qualquer parâmetro além do *Smoothing* do rectângulo; mudar o limite
do miter por omissão de TODO traço do produto sem mostrar o antes × depois ao dono (é a cara de
todos os desenhos dele); a cor das alças fora dos tokens; instalar programas.
JÁ DECIDIDAS (não re-pergunte): alças no modo Forma, nunca no Select; o press nas alças ganha a
desenhar; a fila ② fica com todos os números; as setas ⇕ no idioma da estrela.

COMECE AUTOMATICAMENTE: FASE 0–2, reporte ao Enio em 3 linhas e SIGA com a sub-onda A.

COMO TERMINA (DIRETRIZ §1.5.9): gate batched 1× (`verificador`) · mutação de cada cláusula nova
(`mutacao`, ANTES do portão — nunca ao mesmo tempo) · handoff de integração NOVO (cita o da F11) ·
CLAUDE.md §5.1 «Último:» ≤ 700 bytes · `bash scripts/doc-index.sh` · `rm -rf target/*/incremental` ·
binário smoke 2× · smoke ao dono em passos numerados, sem jargão. NÃO integra, NÃO pusha.
═══════════════════════════════════════════════════════════════════
