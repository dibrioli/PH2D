# PROMPT DE MISSÃO — linha `line/InputMap`, ONDA 2 (cole inteiro numa janela nova)

> Escrito em 10/11 pela janela que entregou a onda 1 (as três versões List · Keyboard · Try it, o comando,
> a tela do jogador, o Luau pela acção). Leitor: a LLM que vai cumprir a onda 2 SOZINHA, do início ao fim.
> O estado técnico da onda 1 está no
> [handoff](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10.md) e no [plano](../01_plano.md) (§10–§12 = contratos).

═══════════════════════════════════════════════════════════════════
## 0 · Quem você é e as regras desta missão

Você é o agente da linha **`line/InputMap`**, Modo L, worktree
`/home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap`. O dono (Enio) **não está ao teclado**: a missão
corre do início ao fim **sem parar e sem perguntar**. As decisões de PRODUTO estão no §2; as TÉCNICAS e de
acabamento são suas, pelo estado da arte
([memória](../../../project-memory/feedback_finish_choices_go_to_the_state_of_the_art.md)). Só pare nos dois
casos do CLAUDE.md §0.2 (contrato congelado; rebase a conflitar fora dos seus arquivos) — escreva o porquê no
handoff e pare.

**Nunca pare num pedido de permissão.** O briefing de CADA subagente leva esta lista, copiada:
- ⛔ `cd X && …` (use caminhos absolutos ou `git -C`), `bash -c` / `sh -c`, `rm -rf` com variável ou glob
  (liste os caminhos e apague um a um), `kill` de PID que não é seu, escrever em `.git/`, `.claude/` ou no rc.
- ⛔ `sed -i`/`perl -i` sem `grep -c` na mesma linha; `replace()`/`re.sub()` que grava sem `assert` de contagem.
  Edite pela ferramenta `Edit`.
- ⛔ Pesado (cargo test/build/run/nextest/clippy) cru: sempre `bash scripts/ph2d-run.sh <cmd>` ou os scripts
  estreitos (`cargo-check-narrow.sh`, `cargo-test-narrow.sh`).
- ⛔ Laço de espera sobre `pgrep -f`, vigia de fundo sem `timeout`.
- ⛔ `git add -A`/`.`, `git stash`, `git update-index`/`hash-object`, `push`, `--force`, integrar, `ship.sh`,
  `suite-completa.sh`, `nextest-impacted.sh`; `#[allow(...)]` para calar um aviso (silenciar é armengo).
- ⛔ A palavra «bancada» em qualquer comando (o hook recusa-a).

**Lições medidas na onda 1 (não as reaprenda):**
1. **Teste de layout pinta na ÁREA REAL da aba**, nunca na janela inteira: `(308,28,988,890)` a 1600×1000 e
   `(297,28,690,604)` a 1280×720 (medidas nas fotos; reconfira-as). Um teste na janela inteira deu verde
   falso (a prévia de 11 acções «a 0,75» estava a 0,47 no app).
2. **O censo de cortes roda nos dois modos de texto**, `Default` e `CrispHeavyPlus` (o modo de FÁBRICA do app),
   sobre o registo de elisão do próprio pintor. No `Default` as larguras são exactas e escondem cortes.
3. **`fotografa_cena.sh` recompila o binário smoke** se o fonte mudou: ninguém edita código enquanto um revisor
   fotografa. Ele espera a vez da placa (`docs/DevOps/A_VEZ_DA_PLACA.md`) — deixe-o esperar.
4. **O arnês de mutação (`mut.py`) restaura com `git checkout HEAD -- <file>`**: faça commit antes de mutar, ou
   perde edições por commitar.
5. **Tectos:** a shell está a ~7 linhas do tecto (189 034 / 189 041) e `ph2d-app-input-map` está no tecto (449).
   Lógica nova vai para `ph2d-input-map-editor` / `ph2d-player-controls`; cada linha nova da shell paga-se
   MOVENDO, nunca subindo número.
6. **Cada agente commita SÓ os seus caminhos** (`-- <paths>`); commit sem pathspec engoliu o trabalho de outro.
7. **A aba desenha acima do HUD** pelo `chrome_over` — não regrida isso.

═══════════════════════════════════════════════════════════════════
## 1 · Abertura (faça já, nesta ordem)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && pwd && git branch --show-current`
   (TEM de imprimir `…/Worktrees/line-InputMap` e `line/InputMap`). É o único `cd`.
2. `git log --oneline HEAD..main | head` — se o main andou (talvez a onda 1 já tenha sido integrada),
   `git rebase main 2>&1 | tee target/rebase.log`; `Cargo.lock`/gerados regeneram-se (DIRETRIZ §1.5.5).
3. Leia: `CLAUDE.md` (já carregado), `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md` (releia a cada
   passo), o [handoff da onda 1](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10.md), o [plano](../01_plano.md)
   §3 e §10–§12, `docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`, `project-memory/MEMORY.md` e
   `project-memory/project_inputmap_three_versions_2026_11_10.md`.
4. **Abra como IMAGEM** a foto do teclado do dono, `docs/InputMap/maquetes/06_teclado_notebook_dono.png`, as
   maquetes `docs/InputMap/maquetes/01..04_*.png` e as fotos de hoje `docs/InputMap/fotos/montagem_*.png`.

═══════════════════════════════════════════════════════════════════
## 2 · A ordem do dono (10/11, depois do smoke da onda 1) — não pergunte de novo

**Verbatim:** *«Muito bom trabalho! Parabéns! Vamos fazer algumas melhorias! Primeiro: o teclado deve ser
completo usando um layout compacto comum em notebooks, como na foto anexa. Segundo: falta polimento em cada
aspecto, efeitos para mouse hover, mouse down, para o foco nos campos numéricos e todos os acabamentos de UI
pro.»*

### 2.1 · O teclado de notebook (a foto)

A foto é um teclado de notebook (padrão US/ANSI compacto), e o desenho das versões B e C passa a ser ele
inteiro:
- **Fila de função:** Esc · F1–F12 · Insert · PrtSc · Delete · Home · End · PgUp · PgDn.
- **Números:** ` 1 2 3 4 5 6 7 8 9 0 - = Backspace.
- **Letras e pontuação:** Tab Q–P `[` `]` `\` · CapsLk A–L `;` `'` Enter · Shift Z–M `,` `.` `/` Shift.
- **Base:** Ctrl · Fn · Win · Alt · Space · Alt · Ctrl.
- **Setas** em T invertido.
- **Teclado numérico** (o quadro VERMELHO da foto): Num Lock (o quadro VERDE) `/` `*` `−` · 7 8 9 `+` · 4 5 6 ·
  1 2 3 Enter · 0 `.`.

**Decisões técnicas a tomar (estado da arte; meça, não adivinhe):**
- **As teclas do numérico são teclas FÍSICAS próprias**: Numpad 7 ≠ 7 da fila ≠ Home. Confira
  `shells/desktop/src/keymap.rs::winit_to_input_keycode` e a tabela `NAMED` de `crates/ph2d-input/src/keyboard.rs`.
  Se os códigos do numérico caem nos dígitos, ou faltam, acrescente-os, com nome («Numpad 7») e nome de script
  («kp7»), e com testes. A ligação é pela tecla física, independente do estado do Num Lock. A luz do Num Lock só
  se desenha se o sistema der o estado; luz falsa, não.
- **Teclas que o computador nunca vê ou que o sistema toma:** Fn é do próprio teclado; Win/Super não chega como
  `Key` (medido na R9b); PrtSc pode ser do sistema. Desenhe-as apagadas e não clicáveis, com tooltip honesto em
  Plain English («Fn is handled inside the keyboard; games never see it.»). Meça cada uma antes de declarar.
- **Geometria = tabela de dados** (larguras em unidades de tecla, como a foto), escalada ao cartão. Nada cortado,
  nada sobreposto, toda tecla clicável, as barras por acção cabem, a 1600×1000 e a 1280×720 na área REAL (a
  1280 a aba tem 690 px de largura). Se a escala passar do mínimo legível, decida pela medição (por exemplo,
  o numérico desce para baixo das setas) e documente. ⛔ Nunca esconder teclas.
- **B e C usam o MESMO pintor** (`view_keyboard/keys*.rs`). A fila «Also:» fica só para o que não está no
  desenho (multimédia, F13+). A busca por tecla da A, a frase do hover, a bancada da C e a escuta funcionam com
  toda tecla nova.
- **Mapa de fábrica e receitas não mudam** (decisão do dono da onda 1).

### 2.2 · O polimento pro (cada aspecto)

Antes de pintar, faça o **inventário** de cada controlo interactivo das três vistas, do seletor, da tela do
jogador e da prévia, cruzado com cada estado. O inventário vai num ficheiro de teste; cada célula é um seam
test que dirige o evento real e afirma a pintura:
- **Hover**, em tudo o que se clica: segmentos, linhas, chips, «+», ×, lixo, teclas, botões e setas da manche
  do comando, ladrilhos, cartões de receita, caixas, botões, o link Forget, polegares de rolagem. O cursor certo
  pelo `pointer_look`: mão no que se clica, I no texto, ↔ no scrub e nas marcas da barra do curso.
- **Mouse down (premido):** estado premido visível (mais fundo ou afundado). Soltar volta. Arrastar para fora
  cancela, como num controlo nativo.
- **Foco nos campos numéricos (Dead/Press):** anel de foco do cartão, cursor de texto, selecciona tudo ao focar,
  ↑/↓ andam um passo (Shift ×10, Alt fino), Enter grava, Esc volta ao valor, Tab vai ao próximo campo, aviso
  visível quando o valor é preso aos limites, marcas D/P.
- **Campos de texto** («New action name», busca): anel de foco, cursor a piscar, selecção, placeholder dim, ×.
- **Desactivado**, com o porquê no tooltip: por exemplo «Remove» sem acção aberta, «Add» com nome vazio.
- **Teclado:** foco visível navegável por Tab; nós de acessibilidade (`DocumentSurface::a11y`) para cada controlo.
- **Movimento:** transições subtis, como o fade do hover, o enchimento das barras do Try it e o pulso da escuta
  tracejada. Vão SEMPRE pelo sistema de movimento do app (prefs `motion_character` / `reduced_motion`; ache os
  helpers da «UI viva»), nunca por cronómetro próprio.
- **Consistência:** raios, paddings, filetes, sombras, tamanho de ícones, alinhamento à grelha das métricas do
  cartão, estados vazios e recusas no mesmo estilo.
- **Molde:** o painel mais polido do app. Ache-o: candidatos são os cartões do Inspector, a Timeline no molde
  do Rive (09/11) e a galeria `PH2D_GALLERY_SMOKE=1`. Copie os ESTADOS dos widgets do cartão; ⛔ nunca um pintor
  próprio de botão ou campo (HR-15: zero hex, zero `f32` de UI, zero string solta; só inglês).

═══════════════════════════════════════════════════════════════════
## 3 · Ondas (uma por vez; cada uma fecha com prova)

| Onda | Entrega | Prova (por NOME + foto) |
|---|---|---|
| **W1** | Códigos de tecla: numérico, Num Lock, pontuação, modificadores e fila de função em `keymap.rs` + `ph2d-input` (nome, nome de script, `Key::label`), mais as medições das teclas que o sistema toma | Teste por tecla da foto: código ↔ nome; Numpad 7 ≠ 7 ≠ Home; o Luau lê `kp7` |
| **W2** | O teclado de notebook desenhado (tabela, pintor, hit, barras, acesas, apagadas, Fn/Win/PrtSc honestos) na B e na C | Censo B/C a 1600/1280 × 2 modos na área real; clique em cada tecla nova liga; cenas de smoke actualizadas + uma cena nova com o numérico (CONTE: `CENAS = 18` hoje) |
| **W3** | Inventário controlo × estado + hover, premido, desactivado e cursores em tudo | Uma célula do inventário = um seam test |
| **W4** | Campos numéricos e de texto pro (foco, selecção, passos, Esc, Tab) | Seam tests de cada gesto |
| **W5** | Movimento (sistema do app, `reduced_motion` respeitado) + acessibilidade + navegação por Tab | Teste com `reduced_motion` ligado = sem animação; nós a11y contados |
| **W6** | Fotos + revisor INDEPENDENTE lado a lado (inclusive o desenho × a foto do dono) + correcções até a lista vir vazia | Montagens finais em `docs/InputMap/fotos/` |

═══════════════════════════════════════════════════════════════════
## 4 · Como trabalhar (obrigatório)

- **A janela principal só decide, desenha e confere.** Leitura ampla vai ao `explorador`; compilar e testar ao
  `verificador`; mutação ao `mutacao`; handoff e memória ao `documentador`. Implementação disjunta vai a
  `general-purpose`, em paralelo na MESMA mensagem, com arquivos DISJUNTOS escritos no briefing.
- **O briefing de cada agente** leva: o caminho absoluto da worktree, os arquivos que pode tocar, o que entregar,
  as formas proibidas e as lições do §0 (copiadas), o inner loop (`cargo-check-narrow.sh <crate>` e
  `cargo-test-narrow.sh <crate> <nome>`), «45 min IMPOSTOS» e «devolva ≤ 40 linhas: o que fez, commits, o que
  ficou aberto».
- **Escreva cada contrato novo** num §13 do plano (`docs/InputMap/01_plano.md`), no mesmo commit; os agentes não
  editam o plano, quem o actualiza é você.
- **Passou de ~300 mil tokens por passo:** delegue cada rodada inteira e faça o mínimo de passos.

═══════════════════════════════════════════════════════════════════
## 5 · Fecho da onda — `/pd-linha-fechar`

1. `cargo fmt` · `PH2D_SUITE=fecho bash scripts/ph2d-run.sh cargo clippy --workspace --all-targets -- -D warnings`
   · `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` · os testes do item e os
   portões tocados por nome · crate nova inteira com `PH2D_SUITE="motivo: crate NOVA"`.
2. **Provas de mutação** (`mutacao`). Leis novas: numérico ≠ dígito na ligação; tecla que o sistema toma não liga;
   hover, premido e foco presentes onde o inventário diz; anel de foco no campo numérico; sem animação com
   `reduced_motion`. As quatro leis da onda 1 têm de continuar a sangrar. Toda mutação que sobreviver vira teste.
3. **Auditoria ≥ 2 lentes independentes**, com o TEMPLATE da DIRETIVA §3: correcção e fios; aparência lado a
   lado. Achado confirmado se corrige na hora.
4. `git rebase main` (com log) e `bash scripts/censos-da-arvore-combinada.sh`.
5. **Handoff:** novo `docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_<data>_ONDA2.md` (≤ 15 KB, itens
   1–11 da DIRETRIZ §1.5.9, com a saída de `collision-surface.sh` e de `agent-loop-profile.sh`), apontando para o
   da onda 1.
6. Liste e apague `target/*/incremental` caminho a caminho. Construa o binário smoke 2×; a 2.ª saída
   («Finished», zero «Compiling») vai no handoff. Varra `pgrep -af 'ph2d|cargo|rustc'` e `fuser -v /dev/dri/*`.
7. **Relatório ao dono** (CLAUDE.md §0.8): curto, sem jargão, PT-BR, o que ele consegue FAZER agora. Smoke em
   passos numerados, um bloco por versão: (1) o comando com o `cd`, (2) onde clicar, (3) o que tem de acontecer,
   (4) como saber que deu errado. Fotos como links clicáveis relativos à raiz do repo principal
   (`[foto](Worktrees/line-InputMap/docs/InputMap/fotos/…png)`). Depois **PARE** e aguarde a ordem de integração.
