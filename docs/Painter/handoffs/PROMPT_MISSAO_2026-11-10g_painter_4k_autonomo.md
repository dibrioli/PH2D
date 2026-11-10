/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_MISSAO_2026-11-10g_painter_4k_autonomo.md

# MISSÃO AUTÓNOMA — o Painter inteiro, funcional e rápido em 4K (dono 10/11)

## 0. A ordem do dono (verbatim, 10/11)

> «A missão da próxima janela é vasta e solitária. Não deverá fazer perguntas ou tirar dúvidas ou solicitar smokes. Não
> deverá parar para nada até concluir todo o trabalho. Além dos itens em aberto, terá de estudar toda pesquisa realizada
> sobre otimização/performance. Deverá relembrar das próprias soluções para os problemas de performance já resolvidos.
> Missão: otimizar e resolver problemas de todo o módulo painter de cabo a rabo e de fora a fora, exaustivamente e sem
> parar, com testes dos principais pincéis. O objetivo é ter o módulo painter completamente funcional e rápido em 4k.
> Precisamos ir além de todos os apps já existentes. Esse trabalho você deve iniciar e completar sozinho sem parar. Smoke
> no fim de tudo. Não integre a linha. Trate cada aspecto e cada arquivo da linha. Faça pente fino, se necessário refatore.
> Se a aparência dos pincéis mudar um pouco, não se preocupe. Isso é esperado.»

**Decisões do dono tomadas ANTES da missão (10/11, respostas a perguntas fechadas) — não as reabra:**

1. **Adotar TODAS as recomendações dos dois estudos** (secção (c) do estudo do Painter e §0.3 do estudo de performance)
   como decididas. Para cada uma que muda a aparência, preparar a prancha antes/depois (3 fundos: branco, sobre blocos de
   cor, papel Textured com relevo; em 1080p e em 4K) para o smoke final. A lista, para não depender de memória:
   - a água do Wet que parou **dorme** (cada poça seca no seu ritmo) — fotos a 1, 4 e 10 s;
   - luz do **Óleo Real** mais simples **só enquanto o pincel está na tela**, completa ao parar — botão por pincel,
     **qualidade alta por omissão no Óleo Real**;
   - **guardar a tinta com mais precisão** (Flow baixo e pincel macio chegam à cor pedida; a orla macia sem degraus) — o
     Watercolor Basic não muda;
   - **arrumar os controlos do Wet**: juntar os dois pares que fazem o mesmo, gaveta fechada para os números internos da
     física, «Tooth fill» para a segunda «Hardness», mostrar Opaque, subir Drag e Emboss ao cartão, duplo clique volta ao
     pincel que abriu; tirar as 19 linhas do Tuning que repetem o Inspector, as 5 chaves gravadas que ninguém lê, e
     esconder as ~50 linhas inertes enquanto o controlo de que dependem está desligado;
   - **criar o pincel «Oil Wash»** (óleo diluído que escorre: a lei de 01/11, «Paint type: Water · Oil», UMA lei);
   - **a aparência acompanha o tamanho da tela** (papel, orla da aguada, relevo) — um ajuste por documento, igual a hoje
     por omissão, ligado só em documentos NOVOS;
   - **mistura de cores do Wet** (vermelho + azul não pode dar verde-oliva/quase preto) e **papel que se repete a cada
     256 px**: consertar;
   - **ritmo fixo** da secagem/aerógrafo (o mesmo resultado a 60 e a 144 Hz);
   - **a placa mostra a prévia, o processador dá o resultado** que se grava e exporta; a prévia na placa com a mesma conta
     do processador (≤ 1 nível de cor);
   - **pincel de mistura grande 3–8× mais rápido** com diferença invisível na cor apanhada (a meia definição é o caminho:
     ver §3);
   - janela fora de foco: **10 quadros/s para a interface**; Play, som e animação a tocar sem limite.
   As outras recomendações do estudo de performance (Vetor, Flip, Pixel, Motion, Luz 2D, Play, física) **ficam fora**
   (decisão 3).
2. **NÃO usar a RTX.** Tudo no processador e no caminho de vídeo que o app já usa hoje. Nada de Wet na placa, nada de
   sonda pesada na RTX (travou o PC duas vezes em 31/10). `PH2D_GPU=1` proibido nesta missão.
3. **Fora do Painter: só o que trava a pintura** — redesenho dos painéis/chrome quando nada mudou, a forma de entregar o
   quadro ao monitor (latência da caneta), cópias da tela inteira, composição das camadas aos pedaços, o relógio do
   quadro. Com testes nos módulos que partilham essas peças (varredura POR CONSUMIDOR). Vetor, Flip, Pixel, Motion, Luz 2D
   e Play: outra missão.
4. **Alarmes de pincel vermelhos ficam vermelhos.** Não regravar `a_largura_de_fabrica_e_a_aprovada` nem
   `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`: gerar `target/relatorio-pinceis/index.html` (antes/depois) para o
   smoke final; o dono aprova pincel a pincel depois.

## 1. Regras da missão autónoma (inegociáveis)

- **Zero perguntas, zero pedidos de smoke, zero paragens** até o fim. Uma dúvida de produto decide-se pela recomendação
  do estudo; uma técnica, pelo padrão-ouro (memória `feedback_architecture_decisions_are_delegated_to_the_gold_standard`,
  `feedback_finish_choices_go_to_the_state_of_the_art`). Registe a decisão no BACKLOG (§2) com o porquê.
- **Nunca parar num pedido de permissão** (memória `feedback_an_autonomous_mission_must_never_stop_on_a_permission_prompt`):
  o hook `~/.claude/hooks/aprova-sem-perguntar.sh` está no `.claude/settings.local.json` desta worktree. Em TODO briefing de
  agente, as formas proibidas: `cd X && …` em comando composto (use caminhos absolutos), `bash -c`/`sh -c`, `rm -rf` com
  variável ou glob, `kill` de PID alheio (e NUNCA do pai de um órfão — logout do dono), escrever em `.git/`, `.claude/` ou
  rc da shell, laço de espera sobre `pgrep -f`, vigia de fundo sem prazo.
- **Não integrar, não fazer push, não fazer rebase sobre um main que ande.** A linha fecha no handoff.
- Os guardas do repo (`ph2d-run.sh`, R5 teste sem nome, R6 edição sem assert, R7 clippy a meio, R8 `--release`,
  commit-barato) continuam a valer: obedeça, não contorne. Pesado por `bash scripts/ph2d-run.sh`; sondas em
  `--profile smoke`; teste por NOME; clippy só no fecho (`PH2D_SUITE=fecho`).
- **Subagente: 45 min impostos.** Corte cada tarefa para caber; um agente por item ou por bloco equivalente (§0.10 do
  CLAUDE.md: problemas equivalentes num ÚNICO bloco — uma medição intercalada, um gate batched).
- **Contexto:** esta janela vai crescer. A janela principal DECIDE e CONFERE; ler o estudo, varrer ficheiros, compilar e
  testar vai a agentes (`explorador`, `verificador`, `mutacao`, `documentador`, `general-purpose` para implementar). O
  estado vive no BACKLOG commitado (§2) — se a conversa compactar, o BACKLOG e o `git log` são a verdade.
- Respostas ao dono, quando houver (só no fim): PT-BR, curtas, sem jargão, imagens como link clicável relativo à RAIZ do
  repo principal (`[foto](Worktrees/line-PainterWatercolor/docs/…png)`).
- O app é só em inglês (strings de UI e tooltips em Plain English, HR-15: zero hex, zero `f32` literal de UI, i18n).
- Sem compatibilidade com projeto antigo (§0.11): mude o tipo certo e suba o `PROJECT_SCHEMA` (conta-se).

## 2. Primeiro passo: o BACKLOG único

Crie `docs/Painter/missao_4k/BACKLOG.md` (commitado e atualizado a CADA item fechado) com uma linha por item:
`id · tema · origem (estudo §/achado/bug) · arquivos · muda aparência? · medição antes · cura · gate · medição depois · sha · estado`.
Monte-o com agentes `explorador` em paralelo, um por secção, devolvendo só itens com endereço (file:line) — NUNCA leia o
estudo inteiro na janela principal (≈ 1 MB):

- **Estudo do Painter** — `/home/enio/Documentos/Projetos/PH2D_Estudos/Painter_2026-11-09/`:
  `ESTUDO_PAINTER_2026-11-09.md` (resumo no topo) e `secoes/` 00–16: **16_onda2_avaliacao.md** (o que se mediu e
  reproduziu — prioridade), **01_metodo.md §1.7** (correções ao estudo — valem sobre o resto), 03 (porque não pinta em 4K),
  04 (plano de otimização, 15 itens), 05 (defeitos), 06 (inconsistências), 07 (parâmetros, §7.5 limites seguros), 08
  (aparência), 09 (testes visuais), 10 (medições), 11 (limites), 12 (técnicas externas), 13 (portões), 14 (recusas MEDIDAS —
  não reconstruir o recusado), 15 (anexos). Fotos: `fotos/onda2`; relatórios dos agentes: `achados/`; sondas: `sonda/`,
  `medicoes/`, `pinceis/`.
- **Estudo de performance** — `/home/enio/Documentos/Projetos/PH2D_Estudos/Performance_Rust_2026-11-09/`:
  `ESTUDO_PERFORMANCE_RUST_2026-11-09.md` e `secoes/`: **90_ranking_global.md** (os ~30 consertos que mais rendem),
  00_sumario, 01_metodo, **10_transversal** (painéis, apresentação do quadro, cópias), **20_painter_tela_pincel**,
  **21_painter_wet_relevo_placa**, 99_anexos. (30/40/50 só para o que for transversal à pintura.) `achados/` por agente.
- **As NOSSAS soluções já feitas** (relembre antes de reinventar):
  - `docs/Painter/BUGS_painter.md` **#84** (Wet Wash Mix: curas 1–6 — salto da cópia do véu, película em faixas no pool,
    taxa por tabela, kernel no pool, pegada lisa na mesma visita, memória do grão Tiled, plano do traço que compõe uma vez
    por quadro, pegadas do lote em paralelo ao bit, **meia definição**) e **#85** (pen-up 4K: luz do relevo por faixas,
    1,7 s → ~120 ms; o bloco «Auditoria de 2 lentes» de 10/11);
  - `docs/Painter/Pinceis/COMO_ACELERAR_UM_PINCEL.md` (método, curas, recusas medidas, §5 como aplicar a meia definição a
    outro pincel, §6 medir o pen-up no app real);
  - `docs/architecture/decisions/0200-o-carimbo-de-mistura-e-o-kernel-em-faixas-no-pool-do-rayon.md` e as exceções
    rayon 0109/0145/0158;
  - `docs/Painter/historia_comentarios_app_painter_2026-11-10.md`;
  - o handoff `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-10_PEN_UP_E_MISTURA.md` e o prompt
    `PROMPT_CONTINUACAO_2026-11-10f_pen_up_incremental.md` (os itens abertos: §3);
  - memórias (`project-memory/`): `reference_topic_painter_perf`, `feedback_a_cost_probe_without_the_owners_paper_has_no_phenomenon`,
    `reference_perf_on_this_pc_opens_the_app_no_callgraph`, `feedback_a_blocking_wait_inside_a_pool_job_is_a_deadlock`
    (Bug #66: espera dentro de job do rayon = deadlock), `feedback_sharing_a_target_dir_between_worktrees_corrupts_the_build`,
    `feedback_changing_a_shared_widgets_arithmetic_is_swept_by_consumer_not_by_call_site`, `feedback_brush_oracle_must_include_blank_canvas`,
    `project_brushes_as_they_are_are_the_reference_2026_11_05` (a lei VELHA é a referência; a nova está arquivada),
    `project_wet_paint_is_the_only_painter_mode_to_evolve_2026_10_07`, `project_oleo_real_priority_relief_shader_light_2026_10_26`,
    `project_paint_type_water_oil_one_law_2026_11_01`, `project_color_mixing_tuned_for_our_engine_not_csp_2026_11_08`.

Depois de montado, ORDENE por (ganho em 4K × certeza) ÷ risco, agrupe os equivalentes em blocos, e ataque bloco a bloco.

## 3. Itens já abertos na linha (entram no BACKLOG)

1. **Leitura incremental de `imagem_aplicada`** (os ~120 ms que sobram no pen-up a 4K com papel): desenho completo no
   prompt 10f e no BUGS #85 — dois buffers (composto NÃO aceso e aceso; a luz não é idempotente), acumulador próprio de
   área suja alimentado por `mark_dirty` (o `dirty_rect` é consumido pelas duas pistas), recompor a suja e re-acender a
   suja ⊕ 1 px; invalidação total em `invalidate_composite`, `set_source`, undo total, `reconcile_substrate`, ajustes,
   vidros/máscaras e mudança de luz (chave própria: rig + `SubstrateKey`). **Gate obrigatório: incremental == inteiro ao
   bit depois de cada tipo de mudança.** Alternativa a medir junto: cache do plano do dente do papel.
2. **A meia definição para os outros pincéis lentos largos e macios** (proposta aceite pelo dono em 10/11): medir todos os
   pincéis à velocidade do dono e ligá-la onde render (o guia §5); agora a aparência pode mudar um pouco.
3. Lacunas de teste da auditoria de 10/11: Save as Default sem verificação da vista; Paint Mixing sem teste de troca de
   preset; miniatura sem teste de `softness`/`tip_depth`/`fill`.
4. O gate `a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela` mede 1,26–1,29 contra o tecto 1,3 sob carga: candidato a
   flake de carga (`docs/DevOps/FLAKES_DE_CARGA.md`) — curar a régua (mínimo intercalado), nunca subir o tecto.

## 4. O que cobrir (de cabo a rabo, de fora a fora)

**Os modos:** Digital, Watercolor, Wet Paint (os 56 pincéis Wet), Impasto, Óleo Real, Pastel/Chalk/Crayon, Composite
Brush, mistura (Color/Paint Mixing), Shape/cerdas, Dual brush, Smudge/Blend/Finger Tip, borracha, Fill, seleção e
proteção de alfa, simetria, camadas (16+, grupos, máscaras, ajustes, Gaussian e efeitos da pilha, vidros), papel e relevo
(Relief, Tooth, Tooth fill), a luz do relevo, a prévia do pincel e os presets (menu, Save as New/Default, Import), a folha
ao vivo do Animated Sprite (o selo), o undo/redo, salvar e abrir, exportar.

**As fases de cada gesto, em 1080p e em 4K:** toque (1.º dab), traço (por quadro), soltar (o quadro do Up e o segundo
seguinte — a folha ao vivo lê ~0,8 s depois), parado (CPU ≈ 0; a água do Wet a mexer à vista), Ctrl+Z, trocar de camada,
mexer num slider de luz/material, abrir um documento 4K, duplicar/apagar camada e grupo.

**Os defeitos já conhecidos (estudo §(d), §0.4 e §05/§16)** — todos: apagar um grupo troca os desenhos (e o Delete não
chega lá); duplicar camada de Impasto dá cópia lisa; a Borracha deixa a crista a fazer sombra; a deformação em malha deixa
o relevo para trás; soltar o botão fora da janela deixa o pincel a pintar (e o Ctrl+Z não o tira); da 256.ª pincelada da
aguada as antigas trocam de cor; a tinta molhada acorda noutro desenho do mesmo tamanho; Wet: vermelho+azul dá
verde-oliva (53 de 56) e dez ferramentas de esbater/borrar não fazem nada numa imagem já pintada; Wet em 4K congela 7–8 s
e salta; Impasto com o editor de animações aberto congela depois de cada traço; pilha com um Gaussian vai a um núcleo;
Repeat Image em 4K com telhas que somem; Flow baixo abaixo da força pedida; nenhum dos 56 Wet de fábrica usa a placa (a
mistura de pigmentos ligada de fábrica impede-o — confira se ainda é verdade e se importa sem a RTX). Reproduza cada um
ANTES de curar (o defeito tem de ser visto vermelho), cure com gate red-first.

**Os alvos (4K, no app real, nesta máquina, sem a RTX):** traço ≤ 16,6 ms por quadro com os principais pincéis em
tamanho médio; toque e soltar sem perder mais de 1 quadro; parado ≈ 0 de CPU; Wet a mostrar a água a ≥ 10 imagens/s;
Ctrl+Z e troca de camada dentro de 1–2 quadros. «Ir além de todos os apps»: compare com o Krita e o MyPaint instalados
(oráculos que se CORREM, §0.9 — triagem de licença por artefacto antes) nas mesmas fases e escreva a tabela.

## 5. Método por item (o que provou funcionar nesta linha)

1. **Medir no app real, primeiro.** `docs/Painter/ferramentas/mede_pen_up_4k.{sh,py}` (Xvfb + rato real XTEST,
   `PH2D_COMPOSITE_TELA=3840x2160`, `PH2D_COMPOSITE_PAPEL=1`, `PH2D_COMPOSITE_PAINEL=digital|water|impasto|wet`,
   `PH2D_COMPOSITE_PRESET=<pincel>`, `PH2D_QUADRO_LENTO_MS=<n>`); `mede_rato_xvfb.sh` para o traço. O perf neste PC:
   o perf ABRE o app (`perf record -k CLOCK_MONOTONIC -F 997 -o … -- <bin>`), SEM `-g`; compile o smoke com
   `CARGO_PROFILE_SMOKE_STRIP=none` (senão o perfil é `[unknown]`); Bash com sandbox desligado para o perf. Estenda os
   roteiros (outras fases, ctrl+z, troca de camada) em vez de escrever outros. **A sonda de motor sem o papel/relevo do
   dono não tem o fenómeno** — confira sempre no app.
2. **Uma rodada de medição por bloco**, variantes no MESMO processo, intercaladas, o mínimo (`docs/DevOps/MEDIR_VELOCIDADE.md`).
3. **Cura com gate red-first** (visto vermelho antes da cura). Quando a cura NÃO deve mudar o desenho: **paridade ao bit**
   contra o caminho antigo guardado verbatim como oráculo `#[cfg(test)]` (o padrão de `as_faixas_dao_o_passe_por_pixel_ao_bit`).
   Quando muda: prancha antes/depois nos 3 fundos + relatório de pincéis.
4. **Provas de mutação** nas leis que importam (agente `mutacao`), duas por gate novo.
5. **Alarmes de pincel por nome** a cada mudança de motor/preset (vermelho = registar no relatório, não regravar).
6. **Widget partilhado** (cartões, `pair`/`compact`, chrome) varre-se POR CONSUMIDOR.
7. **rayon:** nada de espera bloqueante dentro de job do pool (Bug #66); paralelo só com ordem determinística (o mesmo bit);
   cada exceção rayon nova leva ADR (conta-se; o próximo número é o seguinte ao último em `docs/architecture/decisions`).
8. Commit por item (`git add -- <paths>`; mensagem técnica com os números antes/depois e o gate; termina com
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`), BACKLOG atualizado no mesmo commit.

## 6. Pente fino de todos os ficheiros

Liste os ficheiros de cada crate da família — `ph2d-tool-painter`, `ph2d-painter-brush`, `ph2d-app-painter`,
`ph2d-panel-painter-layers`, `ph2d-panel-wet-tuning`, `ph2d-wet-paint`, `ph2d-wet-gpu`, `ph2d-paint-gpu`, `ph2d-paint-tinta`,
`ph2d-pigment`, `ph2d-painter-effects`, `ph2d-brush-presets`, `ph2d-inpaint`, `ph2d-anim-sheets` (a folha ao vivo) e as
partes do Painter em `ph2d-render`/`ph2d-i18n`/`ph2d-editor-core` — e passe cada um por agentes de auditoria de 2 lentes
(`/pd-auditoria`: CORREÇÃO — dois lugares que devem concordar e discordam; COSTURA DE UI — pintado, populado, clicável, o
clique chega ao barramento e o leitor decide; mais DESEMPENHO — trabalho na tela inteira por toque/quadro/soltar, cópias,
alocação por quadro, um núcleo só onde cabe o pool, O(camadas×tela)). Cada achado: mecanismo · reprodução · gate que
faltava · por que o gate existente era verde. Refatore onde o ficheiro mistura responsabilidades ou passa de 700 linhas;
comentários curtos (o porquê histórico vai para o doc). Tectos: 700 linhas por ficheiro, `the_shell_only_shrinks` por crate
(ph2d-app-painter ≤ 24 429 — cura por MOVER/cortar, nunca subir), CLAUDE.md ≤ 40 KB.

## 7. Os testes dos principais pincéis

Uma matriz automatizada (no app, por Xvfb, e no motor): pincéis × tela (1080p, 4K) × fundo (branco, blocos, Textured com
relevo) × fase (toque, traço, soltar, +1 s). Pincéis: o por omissão de cada modo, o Round Mixing, o Wet Wash Mix, o Wet
Liner, o Watercolor Basic, o Pastel, o Chalk, o Crayon, o Finger Tip, o Filbert, o Óleo Real, o pincel de Impasto por
omissão, o Composite Brush, o Oil Wash (novo) e os 10 mais lentos que a medição achar. Grave a tabela (ms por fase, fps,
CPU parado) e as fotos em `docs/Painter/missao_4k/` e mostre antes × depois.

## 8. Fecho (só quando o BACKLOG estiver todo fechado ou registado como recusa MEDIDA)

1. Bateria por nome de tudo o que a missão tocou (DIRETRIZ §1.5.9), fmt e clippy 1× (`PH2D_SUITE=fecho`) nas crates
   tocadas, censos da árvore combinada (`scripts/censos-da-arvore-combinada.sh`), os gates de tecto (`workspace_src_files_under_loc_cap`,
   `the_shell_only_shrinks`, `architecture_claude_md`), provas de mutação, `bash scripts/agent-loop-profile.sh`,
   `rm -rf target/*/incremental`. Na integração (não sua) corre a suíte inteira.
2. Documentação: `BUGS_painter.md` (um número por defeito novo), o guia `COMO_ACELERAR_UM_PINCEL.md`, os ADR que couberem,
   memórias duráveis no `project-memory/` do primário (índice ≤ 17 KB).
3. **Handoff de integração** `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_<data>_MISSAO_4K.md` no
   formato do de 10/11 (superfície de colisão por `collision-surface.sh`, números que se contam, estado do fecho, smokes) e a
   entrada Painter do CLAUDE.md §5.1 (≤ 700 bytes) com o link novo. **Não integre.**
4. **Prompt de continuação** completo para a janela seguinte.
5. **O smoke do dono, só agora**, numa mensagem curta em PT-BR: o que ele consegue FAZER agora; passos numerados ((1) o
   comando completo com o `cd` · (2) onde clicar, com o nome que aparece na tela · (3) o que tem de acontecer · (4) como
   saber que deu errado); a tabela antes × depois em 4K em palavras simples; as pranchas antes/depois das decisões que
   mudam a aparência e o relatório de pincéis (`target/relatorio-pinceis/index.html`) como links clicáveis — fotografados
   antes de mandar (`import -window`, nunca `spectacle`).

Estado de partida: linha `line/PainterWatercolor` reaberta sobre o main `bc4a6ce930` (10/11; a linha anterior entrou
inteira no main — ponta antiga guardada na etiqueta `backup/line-PainterWatercolor-pre-reabertura-2026-11-10`). Árvore limpa.
