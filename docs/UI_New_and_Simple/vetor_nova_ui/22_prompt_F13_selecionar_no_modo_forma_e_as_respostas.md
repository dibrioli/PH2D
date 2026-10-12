═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE: line/UIUX     (PH2D · DIRETRIZ §1.5)
Tarefa: NOVA UI DO VECTOR — F13: O MODO FORMA SELECCIONA (E MOSTRA AS ALÇAS, NUNCA OS NÓS) +
        AS QUATRO RESPOSTAS DO DONO — ATÉ O SMOKE DELE
═══════════════════════════════════════════════════════════════════
Sua linha: line/UIUX · worktree Worktrees/line-UIUX/ (JÁ EXISTE — não crie). Modo L.

⛔ ANTES DE ABRIR QUALQUER ARQUIVO: FASE 0 → 1 → 2 do
docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com MODULO=UIUX.
Estado em 12/10: F0–F8 no `main`; F9–F12 FECHADAS na linha (portão verde, smoke do dono OK) e NÃO
integradas. Siga SEM reset; se o `main` andou: `git rebase main`.

LEIA, NESTA ORDEM (e só isto antes de agir):
  1. docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-12_F12_SETAS.md — o que a
     F12 entregou, mediu e recusou; o §7 tem as respostas do dono.
  2. As regras R1–R9 do topo de docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md
     (R1 = ~150 chamadas ⇒ commit + handoff de continuação ≤ 8 KB + PARE).

O PEDIDO DO DONO (12/10, depois do smoke OK da F12) — palavra por palavra:
 *«faça como sugere.»* (às cinco perguntas do relatório — ver abaixo)
 *«Uma coisa que ainda não foi feita: O modo shape além de criar shapes deve selecionar shapes tb. Uma
  vez seleciona com a ferramenta Shape, o gizmo de ajustes deve reaparecer (shape não deve mostrar
  nós como a ferramenta node.»*

ESCOPO — produto final, por esta ordem (cada item fotografado antes de passar):
  A. ⭐ SELECCIONAR NO MODO FORMA. Já existe um clique que selecciona (`shape_live::after_the_shape_release`,
     F12 `b0d629a800`) — mas o dono diz que NÃO está feito. REPRODUZA pelo press/release REAL da
     shell (o `vec.shape.on_press`/`on_release` + `after_the_shape_release` + `shape_params::frame_fields`
     + `shape_handles_live::hides_the_nodes`/`view`), num teste como `vector_mode_setas_tests.rs`,
     ANTES de curar. Hipóteses lidas no código em 12/10 (MEÇA, não acredite):
     H1 o «armado»: escolher uma forma na paleta arma (`VectorTool::set_shape` → `shape_armed`); o
        `frame_fields` só DESARMA quando o alvo MUDA — clicar na forma que JÁ está seleccionada (ou
        voltar a ela) deixa armado, e armado esconde as alças (`shape_field_target` → `None`).
     H2 armado ⇒ `hides_the_nodes` = `false` ⇒ os NÓS desenham-se no modo Forma — o que o dono viu.
        A lei pedida: no modo Forma os nós NUNCA se desenham (nem armado).
     H3 o `path_at(…, CLICK_HIT_PX)` pode só acertar perto do CONTORNO — um clique no meio de uma forma
        cheia não a selecciona. Compare com o pick do Select (o preenchimento conta).
     H4 uma forma sem `VecShape` (convertida em curvas, ou de outro sítio) não tem alças — o que mostrar.
     O clique é um EVENTO: seleccionar com a ferramenta Forma DESARMA, mesmo que a selecção não mude;
     arrastar continua a desenhar (também por cima de outra forma). Cena NOVA `=20` (a contagem vive
     em `vector_mode.rs`): duas formas, a paleta arma uma terceira, um clique selecciona a 1.ª — a
     foto mostra as alças verdes dela e nenhum nó; depois um clique DENTRO da 2.ª.
  B. A *Resize Box* MANTÉM a espessura do traço (o *Scale Strokes & Effects* do Illustrator, desligado
     — o dono: «faça como sugere»). O desvio e a cura medidos estão no doc 17 §2 (a cura vive só na
     moldura; `m·(r − 1)`). Gate: redimensionar 2× pela caixa deixa a largura igual e a caixa visual
     exacta nas duas bordas.
  C. O ⇕ do *Viewed* dos isométricos: subir = «De cima» (só ele inverte; o resto da fila fica).
  D. Caixa BAIXA: a 1.ª ⇕ (a `h/4` do canto, na borda) toca a alça de canto abaixo de ~62 px de altura
     no ecrã (quadrado 12 px, ⇕ ~22 px — F12 §2). ⚠️ O «meio da borda» que o relatório sugeriu tem a
     alça do MEIO lá: meça e escolha o sítio livre (p. ex. a fila começa um passo `RAIL_PITCH` POR
     FORA da borda, à meia altura), com um gate que varre a altura da caixa de 20 a 400 px e afirma
     que nenhum ⇕ toca nenhuma alça de escala nem a ④.
  E. (Desbloqueado pela resposta (a), FORA desta linha se não couber): a placa da GPU do Motion
     (`ph2d-shape-gpu/src/contorno_cpu.rs` + os shaders, `limite_esquadria`) ainda SALTA para o
     chanfro — portar o bico cortado (o espelho CPU + WGSL). É do motor do Motion: se a janela não
     chegar lá, deixe-o escrito no handoff como o 1.º aberto.
  JÁ DECIDIDO (não re-pergunte): o bico cortado fica; o *Smoothing* antigo fica como está (sem
  controlo); alças só no modo Forma; o press nas alças ganha a desenhar; as setas ⇕ no idioma da
  estrela, lado a lado na lateral; o cartão saiu.

FECHO (DIRETRIZ §1.5.9, nesta ordem): mutação de cada cláusula nova (`mutacao`, ANTES do portão) ·
portão batched (`verificador`: `nextest-impacted.sh` + clippy do `ship.sh` + `fmt --all --check`) ·
handoff de INTEGRAÇÃO novo (F13, cita o da F12) · CLAUDE.md §5.1 UI/UX («Último:», smoke `…6..20`,
≤ 700 bytes) · `doc-index.sh` · `agent-loop-profile.sh` · `rm -rf target/*/incremental` · binário
smoke 2× (a 2.ª saída no handoff) · o SMOKE ao dono, curto, sem jargão, em passos numerados.
⚠️ Lições da F12: fotos só com a placa livre (`cat /run/user/1000/ph2d-gpu.lock.dono`; o lote de fotos
tenta de novo no exit 75 em vez de desistir); NUNCA `pkill -f` (apanha o próprio shell) — pare pelo
PID/grupo; nada de fotos enquanto a `mutacao` tem um ficheiro mutado.
NÃO integra, NÃO pusha. Depois do smoke do dono vem a integração das DUAS linhas do Vetor — não é sua.
═══════════════════════════════════════════════════════════════════
