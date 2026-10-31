JANELA NOVA — linha line/PainterWatercolor (Modo L) · missão «O que o Rebelle faz melhor, medido» (dono 30/10)

FASE 0 (antes de ler): cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
(line/PainterWatercolor; árvore limpa em 9c46e346ac ou depois). BACKUP do Painter antes desta missão: ramo
`backup/line-PainterWatercolor-antes-do-rebelle-medido-2026-10-30` + tag `painter-antes-do-rebelle-medido-2026-10-30`.
Pendente do dono (não bloqueia): smoke da J11 (docs/Painter/Pinceis/ESTADO.md §J11) e os valores de fábrica de Re-wet/Drag/
Solubility — se ele os mandar, é um item à parte, não desta missão.

## Como esta janela trabalha (ordem do dono: a missão inteira NUMA janela, sem esgotar o contexto)
- A janela principal é COORDENADORA: decide, desenha os briefings, junta os números, escreve o relatório. NÃO lê código
  grande, não corre bancadas nem testes ela própria — tudo isso vai a agentes (`explorador`, `verificador`, `mutacao`,
  `documentador`, general-purpose para construir). Cada agente devolve ≤ 25 linhas com ficheiro:linha e números.
- ≤ 3 agentes em paralelo; 45 min cada (o hook impõe); estourou ⇒ devolve onde chegou. Pesado (bancada, medição) um de
  cada vez na máquina — medições intercaladas no MESMO processo, mínimos, `loadavg` < 5 e `nvidia-smi` 0 % ao lado
  (§0.10, docs/DevOps/MEDIR_VELOCIDADE.md). Construção e leitura podem correr em paralelo com UMA medição.
- Briefing de TODO agente leva esta linha: «⛔ cd X && … · bash -c/sh -c · rm -rf com variável/glob · kill de PID não
  gravado · escrever em .git/.claude/ · AskUserQuestion · push · tocar fora dos seus paths». Commit por item só dos seus
  paths: `git commit --no-verify -m "…" -- <paths>`. Teste POR NOME, red-first. Agentes que constroem na mesma crate:
  diga a cada um os ficheiros do outro.
- A janela principal guarda só: o plano, a tabela de números, os commits. Estado em docs/Painter/49_realshader/
  ESTADO_REBELLE_MEDIDO.md (≤ 6 KB), actualizado a cada item — se a janela compactar, é de lá que se retoma.

## Leia (só isto)
1. O doc do dono: https://claude.ai/artifact/QUaFiJcbRj1YCed8ttaU1L («Por que o Rebelle é mais rápido», §3, 5, 6, 7, 8, 10).
   Os factos medidos do Rebelle (água em OpenMP plano 16 núcleos, AVX2 25/47/28 %, 215–234 B/px; facas quase seriais
   0,35–0,40 s CPU/pincelada; RealShader/impasto/gloss sem custo contínuo; NanoPixel = detalhe SINTETIZADO por quadro à
   resolução do ECRÃ a partir de altura/normal/cor, sem memória extra) vêm de lá, com fonte.
2. docs/Painter/49_realshader/README.md (§2b W0(a), §5) + plano.md («Ordem restante»).
3. CLAUDE.md §0.0, §0.9, §0.10 e §2.

## O pedido original foi escrito SEM conhecer o nosso código — conferido em 30/10 (explorador), corrija por isto:
- **A óptica do Óleo Real é CPU**, em `ph2d-wet-paint` (`oleo_real.rs`, `oleo_luz.rs:198` normal por Sobel 3×3 sobre o
  campo da GRADE; composição em `ph2d-tool-painter/.../wetpaint/composite_oleo.rs`). NÃO há passe de produção em
  `ph2d-render`: lá só existe o protótipo de CUSTO `crates/ph2d-render/examples/mede_oleo_real.rs` (W0(a): K=32 N=8 a 4K =
  6,585 ms na RTX 5060 Ti, 144,9 ms na Radeon onboard — K e N são knobs POR PLACA). O passe de placa é a «W1 GPU» do
  plano, ainda por construir. A grade já não é 1:1 com o pixel (ADR-0146 «O que (A) mudou») — confirme o mapeamento
  célula→pixel no composite antes de desenhar.
- Não existe amostragem à resolução da vista para o Painter (só o zoom da UI); o README §2b prevê «nearest; bilinear se
  W3/W4 pedirem».
- Núcleos do Wet Paint: nunca medidos (confirmado). `amostra_cpu.sh` está só em ~/Apps/rebelle/oraculo/.
- f64 com armazenamento f32 (ADR-0134); `advect_jacobi` (solver/advect_jacobi.rs:122) e `drying_pass_jacobi`
  (drying.rs:434) existem com esses nomes; nenhuma sonda f32 feita.
- x86-64-v2 (.cargo/config.toml:27, ADR-0174); zero multiversion/target_feature/std::arch no nosso código.
- Já MEDIDO e recusado (não refaça): encolher o pool do rayon e o retângulo maior na água (§5.47, só no arquivo
  docs/archive/docs-2026-08-18/Painter/28_…; o doc 28 actual :647 diz que o rayon no depósito «não ganha nada»);
  floor de software à mão sobre o v2 (ADR-0174). ADR-0146 (sim na GPU) é PROPOSTA não medida — não é desta missão.
- Medição de ms de placa do render completo do Painter: não existe (só o timestamp query do protótipo).

## Itens (Fase 1 = conferência curta; depois A ∥ B ∥ C em agentes; D quando a tela do dono estiver livre)
FASE 1 (1 explorador, ≤ 20 min): o mapeamento célula→pixel do composite do óleo; onde a vista do canvas do Painter
amplia (zoom) e com que filtro; o que a W1 GPU do plano.md pede exactamente. Com isso o coordenador fecha o desenho de A.

A. **Detalhe à resolução do ecrã (o nosso «NanoPixel»)** = construir a W1 GPU do plano JÁ com esta lei:
   1. O passe de óptica amostra o campo de altura e a cor à resolução da VISTA (zoom incluído), interpolação bicúbica
      ou Catmull-Rom do campo, normal (Sobel) DEPOIS de ampliar. Nada guardado em resolução maior; o documento mantém os
      seus pixels. O caminho CPU continua a ser a resposta de referência (mesma lei; o tecto é o da placa — §0.0).
   2. Gate (fixture NOSSA): a mesma pincelada de faca a 100 % e a 400 %: variância do laplaciano do recorte ampliado
      ≥ 3× a da ampliação bilinear do mesmo recorte; a 100 %, a vista nova = a de hoje (ao byte na CPU; na placa, o
      «mesmo bit» é POR PLACA — memória project_same_bit_law_is_per_device: a tolerância entre placa e CPU mede-se e
      escreve-se com a tabela). Custo: ms de placa por quadro só na região suja × na vista, 1080p e 4K, RTX e Radeon
      onboard (memória project_onboard_radeon_runs_gpu_tests) — tabela ANTES de qualquer tecto.
   3. Sem refinamento em fundo: medir primeiro se o passe por quadro cabe.
   Se a Fase 1 mostrar que a W1 GPU é maior que uma janela: construir A.1 primeiro na CPU (mesma lei, região suja), medir,
   e levar ao dono a escolha com a prancha 100 %/400 % «hoje | nova» — sem decidir por ele.
B. **Núcleos do produto** (sem decisão do dono): amostra_cpu.sh (do oráculo, copiado para scripts/ com cabeçalho) sobre o
   PH2D (`--release` com PH2D_RELEASE="mede velocidade", ou o perfil que o MEDIR_VELOCIDADE manda), wash 4096×2560
   cobertura 100 %, 30 s de água: mediana e pico de núcleos, threads > 0,1, a thread mais ocupada; `perf stat` de largura
   (fp_ops_retired_by_width.*) contra os 25/47/28 % do Rebelle. Pico ≪ 16 ⇒ `perf record --sort sym` no NOSSO binário e
   dizer ONDE o passo é serial.
C. **Duas sondas, fora do produto** (o dono decide depois; ADR só se ele aprovar):
   1. f32 no solver: `advect_jacobi` + `drying_pass_jacobi` em f32, a poça do ADR-0147 §4, A/B no mesmo processo
      (override em execução, nunca const recompilada): ms/passo e desvio contra f64 (máx. e médio por célula). O gate de
      paridade do ADR-0134 fica como está.
   2. Despacho por função (x86-64-v3 só nas rotinas do solver, multiversion em runtime, binário único): A/B no mesmo
      binário, ms/passo e hash do canvas v2 × v3 (diz se o FMA muda bits; a CPU precisa de ADR próprio).
D. **Rebelle: criação da tela 4K × pinceladas** (fecha a §8 do doc): uma corrida com a tela 4K já criada, só as 20
   pinceladas (custo_facas.sh). Oráculo que se CORRE (EULA d: nada de objdump/strings/gdb). RealShader/NanoPixel só no
   ecrã REAL :0, com a tela do dono DESBLOQUEADA (`qdbus6 org.freedesktop.ScreenSaver /ScreenSaver GetActive` == false;
   o LockedHint do logind fica preso em «yes») — avise o dono, em linguagem simples, ANTES de ocupar o ecrã dele.

## Fecho (1× no fim)
Testes por nome + gates tocados (PH2D_SUITE=fecho só aqui); prova de mutação no gate de A.2 (agente `mutacao`); fmt/
clippy das crates tocadas; foto do smoke no ecrã real; handoff técnico em docs/Painter/49_realshader/HANDOFF_REBELLE_MEDIDO.md
com os quatro números (A custo/nitidez · B núcleos · C.1 · C.2), o que ficou aberto, e as duas perguntas ao dono (f32? despacho
v3?). Ao dono: curto, sem jargão, smoke em passos numerados (comando com cd · onde clicar · o que tem de acontecer · como
saber que deu errado), imagens por link clicável. A linha fecha, escreve o handoff e PARA — não integra nem faz push.
