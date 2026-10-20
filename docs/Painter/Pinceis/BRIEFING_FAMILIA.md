# BRIEFING de um agente de FAMÍLIA de pincéis (missão Pincéis; J3, reutilizável na J4/J5)

> Leitor: um agente `general-purpose` que recebe UMA família de 4–9 pincéis e a entrega de ponta a ponta. O
> coordenador (a janela) dá-lhe no prompt: o nome da família, o ficheiro dela, a lista de pincéis (linhas do
> `01_CATALOGO.md`) e os oráculos fotografados de cada um. Tudo o resto está aqui.

## §0 — Regras (inegociáveis)

- **Trabalhe sozinho até o fim. Nunca pergunte nada a ninguém** (não há ninguém para responder; `AskUserQuestion`
  proibido). Decisão técnica = a melhor opção medida; decisão de produto = o critério do EXAME (§2), escrita com o
  porquê no relatório da família.
- Árvore: `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
  FASE 0: `pwd` e `git -C <árvore> branch --show-current`. Leia o `CLAUDE.md` da árvore (§0 e §2).
- ⛔ **FORMAS PROIBIDAS** (disparam um pedido de permissão e PARAM a missão): `cd X && …` (use caminhos absolutos e
  `git -C`), `bash -c '…'`/`sh -c` (escreva um `.sh` no seu scratchpad e corra-o com `bash <ficheiro>`), `rm -rf`
  com variável/glob (`rm -r -- /caminho/literal`), `kill` de PID que você não gravou, escrever em `.git/`,
  `.claude/`, `~/.bashrc`. Nunca `git push`, `sudo`, outras worktrees, nem matar processos que não são seus.
- Um comando RECUSADO por um guarda do repo (hooks R5–R8, `ph2d-run`) não é pedido de permissão: leia a mensagem e
  corrija o comando.
- **Outros agentes trabalham na MESMA árvore ao mesmo tempo** (outras famílias). Por isso:
  - Edite SÓ: o ficheiro da sua família em `crates/ph2d-tool-painter/tests/it/` (receitas + portões), os presets
    gerados da família em `assets/brushes/`, e os seus ficheiros de doc (§4). Nenhum outro ficheiro partilhado
    (`01_CATALOGO.md`, `ESTADO.md`, `receitas_de_fabrica.rs`, `main.rs`… são do coordenador).
  - Motor (`crates/*/src`): só se o pincel PRECISA e o recurso é pequeno (§3). Antes de editar um ficheiro de
    motor, `git -C <árvore> status --short -- <ficheiro>` tem de estar limpo (senão outro agente está nele: espere
    ou escolha outro caminho). Cada gravação tem de deixar a árvore a COMPILAR (outros compilam ao mesmo tempo).
  - Se a compilação falhar num ficheiro que você NÃO tocou, é outro agente a meio: espere 2–3 min e repita; não o
    «conserte».
  - Commits: `git -C <árvore> add -- <seus paths>` e `git -C <árvore> commit --no-verify -m "…" -- <seus paths>`
    (os NOVOS também no `add`: `commit -- paths` ignora ficheiros por rastrear). Nunca `-A`, `.`, `stash`, nem
    patch de hunks filtrados. Mensagem `feat(Pincéis J3 <família>): …` + `Co-Authored-By: Claude Opus 5.5
    <noreply@anthropic.com>`. Comite a cada pincel aprovado (não acumule).
- Velocidade: inner loop `bash <árvore>/scripts/cargo-check-narrow.sh ph2d-tool-painter`; testes POR NOME
  (`bash <árvore>/scripts/cargo-test-narrow.sh ph2d-tool-painter <nome>`); pesado por `bash <árvore>/scripts/ph2d-run.sh`.
  Nada de clippy nem suíte inteira (é do fecho da J5). Edite com a ferramenta Edit.
- **A bancada em paralelo**: SEMPRE `PH2D_EXAME_DIR=/tmp/ph2d_exame_<família>` em todo `exame.sh` (a cache da
  biblioteca é por pasta; sem isto duas famílias apagam o trabalho uma da outra). No fim, `rm -r -- /tmp/ph2d_exame_<família>`
  (a fatia da linha conta o `/tmp`).
- **Oráculos exclusivos** (Rebelle, ArtRage, Krita): fotos NOVAS só pela vez —
  `bash docs/Painter/Pinceis/ferramentas/vez_do_app.sh <rebelle|artrage|krita> <comando…>`. Os comandos de cada
  app: `02_ORACULOS.md` e o inventário abaixo. O MyPaint é livre. Foto nova do Rebelle só vale estável
  (`foto_estavel.py`). ⚠️ O teste do Rebelle expira ~06/11.

## §1 — O material (leia por esta ordem, só o que precisar)

`00_MISSAO.md` §2–§3 (os motores e o EXAME) · `04_BANCADA.md` §0, §3.1, §4, §9 (a bancada, a régua, a matriz, a
fábrica por família) · `05_REVISAO_CEGA.md` (o briefing do revisor e o cruzamento) · `06_LEIS_DAS_LACUNAS.md` §0
(as leis numa frase) · `03_MOTORES.md` §1 (o que cada motor exprime) e §4 só as linhas ✅ das lacunas que a sua
família usa · `matriz/vizinhos.md` (os «perto» da fábrica de hoje) · o veredito do dono no P53 (00_MISSAO §0):
**passar a régua não é ser um pincel** — Filbert «nada novo», lâminas «bizarras e inúteis».

## §2 — Por pincel, de ponta a ponta (o EXAME, as SEIS condições)

1. **Receita** no ficheiro da família (`WET` para Wet Paint, `MEIOS` para Digital/Watercolor/Impasto). Motor: o
   que melhor serve a identidade (linha limpa = Digital; papel, água, pigmento = Wet Paint). Tamanho de fábrica =
   o do app de origem quando há fonte MEDIDA (a foto, o ficheiro do preset) — nunca inventado. Gere o preset
   (`04_BANCADA` §9: só o seu, `PH2D_REGERA_SO="Nome"`).
2. **Bancada**: `PH2D_EXAME_DIR=/tmp/ph2d_exame_<família> bash docs/Painter/Pinceis/ferramentas/exame.sh "<Nome>"
   docs/Painter/Pinceis/fixtures/<oráculo>` — régua, matriz, prancha legível e cega. Corra-o contra o oráculo
   PRINCIPAL (o Rebelle quando existe; senão Krita/MyPaint/ArtRage) e, se houver um 2.º, contra ele também
   (`PH2D_EXAME_SUFIXO=_<app>`).
3. **OLHE a prancha** (ferramenta Read no PNG) como um artista: o traço do oráculo e o nosso lado a lado, gesto a
   gesto (início, fim, borda, grão, pressão, curva, cruzamento, seco). Régua verde é condição NECESSÁRIA, nunca
   suficiente. Cure (valores; motor pequeno §3) e repita até a prancha não mostrar diferença.
4. **Distinto da biblioteca INTEIRA**: o 1.º vizinho nosso fora da família a d ≥ **0,596** (`matriz/escala.json`);
   série graduada (dureza/densidade) entra como família — fora ≥ 0,596, cada degrau ≥ 0,40, e o eixo VÊ-SE na
   prancha. Um pincel que um existente já pinta NÃO entra (ou funde-se com ele: diga qual).
5. **Razão de uso** numa linha: para quê um artista o escolhe (o que nenhum outro da biblioteca faz).
6. **Revisão cega por um agente NOVO** que não fez o pincel: lance-o pela ferramenta Agent (`general-purpose`) com o
   briefing do `05_REVISAO_CEGA.md` §2 tal e qual, só com o caminho da prancha CEGA. Cruze com a chave (05 §3).
   Diferença visível listada ⇒ cura-se e repete-se com um revisor NOVO, ou o pincel é REPROVADO. ⛔ Nunca «aberto»
   num pincel entregue. (Se a ferramenta Agent não existir para si, devolva as pranchas cegas por rever ao
   coordenador e não declare o pincel aprovado.)
7. **Custo**: `PH2D_EXAME_DIR=… exame.sh --custo "<Nome>" 5,40,200` — p95 ≤ 16,7 ms/quadro a 200 px e RSS sensato
   (≤ ~150 MiB); escreva o `load` ao lado (acima de ~5 o relógio não vale: repita calmo). Rato e pressão: a bateria
   já tem a rampa de pressão; diga o que a caneta deitada faz se o pincel a usa.
8. **Portão por foto** na família: a identidade do pincel presa (o padrão de `midias_de_fabrica.rs`: a mídia
   reconhece-se pela foto) + os IRMÃOS onde a matriz os acha perto; **prova de mutação** do portão (mude o valor
   que faz a identidade; o portão tem de sangrar; restaure). Verde por nome.
9. **Sem oráculo que corre** (os «doc.» do catálogo, Procreate etc.): o alvo é o comportamento DOCUMENTADO (manual
   oficial, Brush Studio do Procreate, páginas da Adobe/Corel — WebFetch) e fotos/vídeos públicos como alvo VISUAL;
   escreva a fonte no relatório. A revisão cega então compara com o vizinho mais perto e com uma DESCRIÇÃO escrita
   da identidade (dê ao revisor a prancha sem rótulos e a frase da identidade; ele diz qual das imagens a cumpre e o
   que falta). ⛔ Nunca copiar ficheiros de pincel, texturas nem código de app nenhum: copia-se o COMPORTAMENTO.

**Pincel que já existe na fábrica** (da sua categoria): passa pelo MESMO exame (decisão 3 do `ESTADO.md`): cura
contra a origem, ou funde-se/sai. Mudar um preset existente = regerar o ficheiro dele e os portões que o prendem
(miniatura, largura) por nome.

## §3 — Motor em falta

Recurso que o pincel precisa e não existe: **pequeno** (um campo, uma curva, um modo — aditivo, DESLIGADO por omissão,
os outros pincéis byte-idênticos) ⇒ faça-o, com portão red-first (vermelho antes, verde depois), mutação e custo.
**Grande** (um meio novo, física nova) ⇒ não faça: o pincel não sai nesta onda; escreva no relatório o que falta,
com a medição que o prova, para o coordenador o pôr na fila.

## §4 — O que escrever e devolver

- `docs/Painter/Pinceis/exames/<família>.md` (seu, ≤ 10 KB): por pincel uma secção curta — receita (motor, valores
  que fazem a identidade), oráculo(s), prancha (caminho), régua (✓/total), vizinho (d), razão de uso, a resposta
  do revisor cego (colada, curta) e o cruzamento, custo (p95 a 5/40/200, RSS, load), portão + mutação, VEREDITO
  (aprovado / reprovado e porquê / fundido com X).
- Pranchas em `docs/Painter/Pinceis/pranchas/` (a bancada escreve-as; mova as da sua família para
  `pranchas/<família>/` no fim, com as cegas e as chaves em `pranchas/<família>/cegas|chaves/`).
- **Devolva ao coordenador SÓ**: a lista aprovados / reprovados (uma linha cada, com o porquê), os caminhos das
  pranchas, os SHAs, o que mudou no motor (se mudou), e o que falta para a fila (motor grande, foto que não deu).
