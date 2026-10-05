═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)

⛔ Fale com o Enio SEMPRE em PT-BR (também nas mensagens curtas entre ferramentas), curto e sem jargão (CLAUDE.md §0.8).
⛔ ORDEM DO DONO (05/10): «integrar só quando não houver nada em aberto» e «resolver tudo numa ÚNICA onda».
   ⇒ esta onda fecha TODOS os itens abaixo — cada um CONSTRUÍDO e medido, ou RECUSADO por medição com o critério
   escrito ANTES. Nada fica «para depois». A linha continua sem integrar e sem push: no fim você reporta e PARA
   (CLAUDE.md §0.7); quem integra é um integrador, por ordem do Enio.
⛔ REGRA DO DONO (CLAUDE.md §0.10): problemas equivalentes num ÚNICO bloco — um plano, um lote de binários, UMA rodada
   de medição intercalada (docs/DevOps/MEDIR_VELOCIDADE.md; controlo A/A `PH2D_SONDA_AA=k`), um gate batched.
   Série só quando o resultado de um passo muda o seguinte — e diga-o por escrito ANTES da 2.ª medição.
⛔⛔⛔ NUNCA mate um processo pelo PAI (`kill $PPID`, `pkill -P`, «o pai do órfão»): o pai de um órfão é o
   `systemd --user` e em 05/10 isso fez LOGOUT da sessão do dono. Termine só o que lançou, pelo PID dele; prefira o
   prazo (`timeout`, `PH2D_PRAZO`) e o `TaskStop` das tarefas de fundo. Vigia de fundo sempre com prazo.

FASE 0 (já, sem pedir confirmação):
1. cd Worktrees/line-motion-value && pwd && git branch --show-current   → line/motion-value
2. git log --oneline -10 && git status --short --ignored | grep -v target/   → HEAD = o commit «o prompt da 4.ª onda»
   (o último da linha); árvore limpa (só `!! assets/sprites/`).
FASE 1:
3. git cherry main HEAD | grep -c '^+' (a linha NÃO está integrada). Se o main andou: `git range-diff main...HEAD` e
   `git rebase main 2>&1 | tee target/rebase.log` (conflito fora dos seus ficheiros ⇒ PARE e reporte ao Enio).
4. bash scripts/cargo-check-narrow.sh ph2d-shape-gpu && bash scripts/cargo-check-narrow.sh ph2d-vec-render &&
   bash scripts/cargo-check-narrow.sh ph2d-app-motion
FASE 2 (leia DENTRO da worktree):
5. docs/Motion Nodes/121_as_formas_na_placa.md §9.17 (só a tabela e os veredictos) e §9.18 INTEIRA — o plano, e por
   baixo os resultados de (D), (F), (E) e (C). O handoff de integração
   docs/Motion Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md §6, §10 e §11.
6. DIRETIVA_IMPLEMENTACAO.md (§5: two-strikes) · regras A–K do MODELO_ABERTURA_LINHA.md.
7. Reporte em UMA linha e siga.

O QUE JÁ EXISTE (NÃO reconstrua):
- A placa de formas com os pedaços do §9.15 dobrados; a sonda intercalada (`mede_intercalado.sh`, variantes `F`·`F-D`·
  `F-c2`, `conta`/`escreve` à parte, as arestas, a imagem contra o `F` byte a byte); a porta `PH2D_FORMAS_CONSTANTES`
  e a lista `CONSTANTES_DO_CONTORNO` (vazia, à espera do próximo `override`).
- (C) curada (`0321a1b46`…`e619c0143`): a porta CPU da lei do contorno (`crates/ph2d-shape-gpu/src/contorno_cpu.rs`,
  o `percorre` do `contorno.wgsl` em `f32`, polígonos fechados; `nivel_da_copia`, `caneta_de`); as portas da geometria
  (`tolerancia`, `extensao`, `eixo_do_nivel`, `contorno_conforme` em `geometry.rs`); o gancho
  `draw_shared_instances_com_traco`/`TracoProprio` (`crates/ph2d-vec-render/src/instance.rs`); o traço pela lei
  `TracoDaPlaca` (`crates/ph2d-app-motion/src/motion_shape_traco.rs`). Gate `a_rota_vello_traceja_o_pedaco_rente_
  como_a_placa` (alfa `≤ 1` contra a placa nas duas placas); sonda `custo_do_encode_tracejado` (o `encode` e a parede
  do Vello, rota pela lei × rota de antes, intercaladas); mutações `mutacao_a_mordida_do_traco_rente_2026-10-05.py`
  (`12/12`).
- O código MEDIDO da emissão por peça está em `afa0cc014` (`git show afa0cc014`: `contorno_pecas.wgsl` com a tabela dos
  troços, o `cs_soma_pecas`, o `cs_pecas` com as duas buscas binárias, o `cs_fecha`, a caixa em bits ordenados, a reserva
  atómica e a contada, as ablações `P4`–`P6`; `contorno_pecas.rs` com os passes dentro do relógio `escreve`). Reaproveite-o.
- ⛔ RECUSAS MEDIDAS (não se reconstroem): a emissão por peça com o PASSEIO EM SÉRIE por cópia (§9.18 E: a prova `0,183`,
  o passeio sozinho `0,130` na iGPU); L/H/P/G32 (§9.17); o GRUPO por cópia (§9.14); D1/D2 do `cs_varre`; a esparsa;
  estreitar a célula; herdar a bissectriz; costurar as correntes; as pontas partilhadas. As marcas conformes da placa
  pelo `expand_stroke` NÃO mordem (varrido, §9.18 C) — não as troque.

A TAREFA — CINCO ITENS, uma onda, por esta ordem de PROVA (a construção e a medição são um lote só):

 (1) O PASSEIO EM PARALELO — a emissão tracejada por peça, 4.ª topologia ⇒ two-strikes: PROVE O MODELO ANTES do produto.
     O modelo: um fio por TROÇO calcula o `arco` dele; um prefixo SEGMENTADO por sub-caminho dá o `s0` de cada troço
     (e o total do fechado); o `n0`/`n1` e o prefixo das peças saem dele; o resto é o `afa0cc014` (a tabela, a busca,
     um fio por peça). ⚠️ Um prefixo paralelo soma noutra ordem: a imagem deixa de ser a do `F` ao bit — os critérios de
     imagem são os de baixo, escritos aqui e não depois.
     Prova (ablação, sem arestas, pela sonda intercalada): o passeio paralelo + a tabela + o `pedaco` por peça com
     `escreve` das esticadas tracejadas da iGPU `≤ 0,13` ms (hoje `0,183` com o passeio em série; a parte por peça mediu
     `0,056`: linhas `0,021` + buscas `0,019` + `pedaco` `0,002` + despachos `0,014` ⇒ o passeio paralelo tem de caber
     em `~0,07`). Acima ⇒ (1) fecha por recusa medida, com a decomposição.
     Kill-criterion do produto: `escreve` `≤ 0,15` (de `0,239`) e soma `≤ 0,86` (de `0,956`) nas esticadas tracejadas
     da iGPU; nenhuma cena pior que `+5 %` na iGPU nem `+10 %` na RTX (a troca de hoje — RTX esticadas `−15 %`, densas
     `+10 %` com o passeio em série — conta: escreva a troca aceitável ANTES da rodada); a imagem: alfa `≤ 1` contra o
     `F` em todas as cenas da sonda (acrescente a régua de alfa ao lado do «imagem = F»), os `14` gates da crate nas
     duas placas, os gates `motion_shape_placa` e o arnês do tracejado contra o Vello; a reserva igual (ou a memória
     nova escrita e justificada). Mutações dos pedaços aceites.

 (2) A PAREDE DO VELLO NA ROTA PELA LEI. A cura (C) preenche um polígono por peça/junta/ponta e a parede do Vello subiu
     (`custo_do_encode_tracejado`: densa `+7 %` RTX · `+25 %` iGPU; `16 384` cópias `4,39 → 6,33` ms RTX · `15,5 → 23,3`
     iGPU). Candidatos (meça antes de escolher): fundir num só contorno as peças de um traço que se ligam pela FAIXA (a
     aresta partilhada não se escreve, como na placa); um caminho por traço em vez de um por peça; o que o perfil do
     Vello mostrar. Critério: parede `≤ +10 %` da rota de antes nas quatro células da sonda, o `encode` sem perder mais
     de `10 %` do ganho (`−83 %`), o gate do traço rente alfa `≤ 1` nas duas placas, as `4` famílias de hoje não pioram
     (alfa `46`, `165` px). Senão, recusa medida com a tabela.

 (3) A PONTA E A JUNTA NO CARTÃO DA FORMA. O nó `source.shape` do Motion não expõe ponta (rente · redonda · quadrada)
     nem junta (esquadria · redonda · chanfro) — o produto traceja sempre rente/esquadria. Construa: os parâmetros no
     nó (`crates/ph2d-node-motion-shape/src/param.rs`) e no manifesto; o `build_shape_path` (`motion_shape_gen.rs`,
     ⚠️ `672/700` — cresceu? MOVA para um irmão) põe-nos no `StrokeSpec`; o cartão com a UI canónica (tokens, i18n, os
     censos — CLAUDE.md §0.3); a rota do DISPOSITIVO (`motion_bridge_gpu_forma*`) e a placa honram-nos (o eixo já sabe
     as três juntas e as três pontas); a gravação no `.ph2dproj` (um projecto antigo abre igual: omissão = rente/
     esquadria). Gates: cada combinação pela placa contra a rota Vello (a família do traço rente com os novos valores,
     alfa `≤ 1`); o catálogo/manifesto do nó; ida e volta da gravação. Contrato congelado `NodeManifest=8` (§6): ler o
     ADR-0039 ANTES — acrescentar parâmetros não pode mudar o contrato; se mudar, PARE e reporte ao Enio.

 (4) A PROVA DIRECTA DA CURA DO SIGSEGV DA RTX (§9.18 F). A cura está no `pass_profiler` (`42ce59c51`); a evidência é
     `4/6` falhas com o perfilador segurado contra `0/9` sem ele, e a falha não se reproduziu sob carga sintética nem
     com a disputa presa a dois núcleos. Desenhe a reprodução ANTES: a sonda de `13` variantes na RTX, `N` corridas com
     a carga da MÁQUINA (não da fatia), binário de ANTES da cura (um `worktree` temporário em `42ce59c51^`, com
     `CARGO_TARGET_DIR` PRÓPRIO — ⛔ nunca partilhado entre worktrees) contra o de agora, intercalados. Critério: ANTES
     `≥ 2` falhas e DEPOIS `0` em `N ≥ 20` ⇒ provada; ANTES `0` ⇒ escreva que a condição não se reproduz nesta máquina
     e feche com a evidência que existe (é medição, não desistência). Remova o worktree temporário no fim.

 (5) O CONTACTO DOS COLISORES NA PLACA — a medição que FALTA para a decisão (§9.18 D). A recusa do doc 115 foi
     renovada pela conta da CPU (`4 096` peças `9,3 %` de um quadro; expira a `16 384`, `34,7 %`) — mas o QUADRO
     INTEIRO no app com Play nunca correu (o roteiro de foto não carrega em Play). Construa a porta (uma variável que
     arranque a cena em Play, ou a que já existir — procure ANTES), e meça a `=114` com `PH2D_PILHA_LADO`/
     `PH2D_PILHA_COLIDE` a `1 024` · `4 096` · `16 384` peças, as duas placas, pelo `mede_formas_na_placa.sh` e o
     `[frame] MOTION`. Critério (o do §9.18 D): se a `4 096` o quadro passa de `16,7` ms na iGPU, a recusa expirou e o
     contacto da CAIXA no dispositivo entra NESTA onda (o solver de `ph2d-contact` em WGSL, paridade por passo contra a
     CPU, as cercas de `motion_bridge_gpu_colisor.rs` a cair só para o que o kernel honra); senão, a recusa renova-se com
     o número do app, e o ponto de expiração escreve-se.

COMO: inventário (agente `explorador`, em paralelo por item) → as provas de (1) e a medição de (5) (decidem o que se
constrói: série justificada, escreva-o) → construir tudo → UMA rodada intercalada (a sonda com as variantes novas, o
`custo_do_encode_tracejado`, a `=114` com Play) → gate batched 1× (agente `verificador`: `nextest-impacted`, `check
--workspace --all-targets -D warnings`, clippy `--all-targets --all-features -D warnings` das crates tocadas, `fmt`,
`machete`, `check-standalone-optional`, `check-workflow-packages`, `ph2d-shape-gpu --test it --ignored` nas duas placas,
`motion_shape_placa` nas duas, `ph2d-gpu-cook formas` nas duas, `ph2d-vec-render --ignored` nas duas, censos, tecto de
LOC) → mutações SOZINHAS na árvore (um roteiro por item construído, no padrão de
`docs/Motion Nodes/ferramentas/mutacao_a_mordida_do_traco_rente_2026-10-05.py`) → os resultados por baixo de cada
item no doc 121 (uma §9.19 para esta onda; NÃO reescreva o §9.18) → uma §12 no handoff de integração de 05/10 (a linha
ainda não foi integrada) com a superfície de colisão nova → memória → smoke ao dono em passos numerados, FOTOGRAFADO
(FORMAS + SIMULAÇÃO; a `=127` tracejada com a ponta REDONDA escolhida no cartão; e a `=114` com colisão em Play) →
`rm -rf target/*/incremental` e o smoke compilado 2× → `bash scripts/agent-loop-profile.sh` → máquina limpa.

Leis que já custaram caro nesta linha:
 - A régua é a SOMA dos passes; o resumo é o MÍNIMO das rodadas; mínimo e mediana longe ⇒ a régua não decide.
 - Um critério «nenhuma cena melhora sem ele» é cego à TROCA entre cenas — escreva a troca aceitável.
 - Uma ablação que deixa de GUARDAR o que um laço calcula mede menos que o laço (o compilador corta a aritmética que
   ninguém lê): segure o cálculo com uma escrita que nunca acontece, e escreva no doc o que a ablação guarda.
 - O defeito atribuído a um componente varre-se ANTES da cura, com um CONTROLO de que ele o produz na fixtura (o
   «kurbo morde» era o Vello).
 - Um gate só prova o que a fixtura contém: a emenda só existe num sub-caminho mais CURTO (a `m6` sobreviveu a uma
   família de contornos únicos).
 - Com poucas cópias (`72` = duas ondas) registos/ocupação NÃO são o custo — o caminho por fio é.
 - ⛔ `contorno.rs` `626/700`, `motion_shape_gen.rs` `672/700`, `contorno_cpu.rs` `~620/700`: cresceu? MOVA para um
   irmão (`contorno_capacidade.rs`, `contorno_sondas.rs`, `contorno_pecas.rs`).
 - O guarda recusa `cargo`/`python3` pesados fora de `bash scripts/ph2d-run.sh` — e confunde TEXTO com comando: um
   heredoc que contém «cargo run» é recusado; edite docs pela ferramenta `Edit`.
 - Script de várias etapas: `&&`, nunca `;` depois de um passo que pode falhar. zsh não parte `$var` em palavras:
   listas de ficheiros vão por `bash -c`.
 - A placa é partilhada: PH2D_GPU=1 bash scripts/ph2d-run.sh …; NUNCA force. Mutação nunca em paralelo com builds.
 - Flakes de relógio de OUTRAS crates no `nextest-impacted` (`ph2d-physics-ecs` `the_cost_of_a_player_is_linear…`,
   `ph2d-tool-painter` `the_mask_stroke_cost…`): confirme sozinho com o `loadavg` ao lado; não são desta linha.
 - A memória do Claude aponta para o project-memory do checkout PRINCIPAL: escreva-a em
   Worktrees/line-motion-value/project-memory/ e comite-a na linha. O índice está no tecto (`~21,9 KB` de `22 KB`):
   entrada nova vai para o `reference_topic_*` da família, e a contagem da família actualiza-se no índice.
FORA DE ESCOPO: o `stroke_uniform` do Vector (outra linha; não o mude).
DoD: os cinco itens fechados — construído com tabela antes/depois e o kill-criterion escrito antes, ou recusado por
medição com a tabela; gate batched verde; mutações a sangrar (as que sobreviverem viram gate NESTA onda); doc 121 §9.19;
handoff §12; memória; smoke fotografado; NÃO integra, NÃO pusha (CLAUDE.md §0.7). No relatório ao Enio diga, em uma
frase, se ficou ALGUMA coisa em aberto — a integração só acontece quando a resposta for «nada».
A MÁQUINA É PARTILHADA (regra K): pgrep -af 'ph2d|cargo|rustc' e fuser -v /dev/dri/* antes e no fim.
═══════════════════════════════════════════════════════════════════
