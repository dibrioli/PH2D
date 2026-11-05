# Briefing comum de uma fatia da nova UI em cartões do PH2D

PT-BR. O dono está ausente: NÃO pergunte nem proponha smoke; decida pelo padrão-ouro e registe em «Decisões tomadas sem o dono».

ÁRVORE: só /home/enio/Documentos/Projetos/PH2D/Worktrees/line-NovaUI (branch line/NovaUI). Comece por `cd` + `pwd` + `git branch --show-current`. NUNCA a raiz do repo. Sem push, sem integração.

LEIA: docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md; docs/CartoesUI/05_ESPECIFICACAO_UI_COMPLETA.md §2–§6, §10, §11; e o que a sua fatia indica. Referência de widget ausente, por ordem: os nossos painéis aprovados e a Widget Gallery (`PH2D_GALLERY_SMOKE=1`), Figma, Rive, app profissional com UI premiada (cite o prémio), Inkscape/Godot/Unity. Nunca texto a sair da borda ou cortado com «…» em botão (lei 7 de 05 §4). Idioma visual aprovado: docs/CartoesUI/maquete/f_hierarchy_ao_lado_do_inspector.png.

OUTROS AGENTES trabalham AGORA nesta árvore, noutras crates. Commit por caminho: `git add -- <paths>`, `git commit --no-verify -m "…" -- <paths>` (NUNCA -A, ., stash); antes, `git diff -- <ficheiro>` em ficheiro partilhado, para não levar nem desfazer linhas alheias. Ficheiro NOVO exige `git add` antes. `index.lock` = espere e repita. Última linha do commit: `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. `crates/ph2d-editor-core` só em passos pequenos que compilam logo. Mantenha a sua crate a compilar. Erro em crate alheia: espere e repita; não o conserte.

⛔ FICHEIROS TEMPORÁRIOS: scripts e sondas só numa pasta SUA, criada com `mktemp -d`, com o caminho completo. Nunca um nome fixo em `/tmp` (medido: um agente correu o `mut.py` de outro, que mutou crates alheias). Mutação nunca sob `timeout` externo que a possa matar a meio: use o prazo da `ph2d-run.sh`, e confira `git status` e o restauro no início do próximo passo (medido: um `timeout` deixou uma linha mutada). Mutação em ficheiro de outra crate é proibida. Mutação em `ph2d-editor-core` para todos os agentes: faça-a só em lote curto, com restauro por `try/finally`, e confira `git diff` no fim (medido: uma mutação correu por cima de um build e deixou uma linha mutada).

⛔⛔ LEI DA COMPACTAÇÃO (dono 04/11, docs/CartoesUI/05_ESPECIFICACAO_UI_COMPLETA.md §4): o layout é compacto como o do Figma. Irmãos curtos dividem a linha, nada grudado, uma grelha por cartão.

⛔⛔ MICRO-PESQUISA (dono 04/11, 05 §0): antes de mudar cada cartão, veja como o Figma e o Rive fazem (WebSearch/WebFetch na documentação oficial e imagens públicas; 1 a 3 consultas) e registe uma linha em docs/CartoesUI/16_REFERENCIAS_POR_CARTAO.md. Consulte a tabela antes.

LEIS: HR-15 (zero hex, zero f32 literal de UI; cor por `CardInk`; geometria em `metrics`; texto em ph2d-i18n; app só em inglês, Plain English). Unidade no rótulo, nunca na caixa. Campo que não se explica sozinho leva rótulo. Nenhum controlo morre: mesmos ids, mesmos eventos. `PH2D_CARDS=0` dá o painel de antes. Contratos §6 do CLAUDE.md intactos. Marcas de animação preservadas (Auto Key grava tudo). Todo arrastar-reordenar voa como o cartão do Kanban. Tecto de LOC por ficheiro (`panel_files_under_loc_cap`) e de crate: parta o que crescer. A shell não cresce. Catraca só desce; o que subir é medido e explicado na tabela. Silenciar diagnóstico é proibido.

TESTES: gesto real, por controlo, em `tests/it/*.rs` da crate (modelo: `crates/ph2d-panel-physics/tests/it/os_cartoes.rs`); clique em cada id produz o mesmo que `PH2D_CARDS=0`. Contraprova: tire a linha, veja vermelho; conte-as. Reforce o teste se um mutante sobreviver. Testes antigos continuam verdes.

COMPILAR/TESTAR: `bash scripts/cargo-check-narrow.sh <crate>`; `bash scripts/cargo-test-narrow.sh <crate> <nome>`; pesado por `bash scripts/ph2d-run.sh <cmd>`, perfil smoke. Sem suíte completa, sem clippy. Edite pela ferramenta Edit. No fim, por nome: `no_magic_numeric`, `hr12_widgets_a11y`, `the_shell_only_shrinks`, `architecture_panel_loc_cap`, `todo_campo_pintado_com_losango`, `quantas_entradas_tem_cada_painel`, `nenhum_nome_por_cima_do_controlo`, `onde_comeca_o_valor`, `a_marca_tem_a_altura_da_linha`, `toda_dica_dos_cartoes`.

OLHAR É OBRIGATÓRIO: foto (`docs/Components/ferramentas/fotografa_cena.sh`, nunca `spectacle`) e Read. Todo cartão aberto e visível ao menos numa foto (05 §11b); patch temporário restaurado para estados que a cena não semeia. A foto final é tirada DEPOIS das curas. Guarde poucas em docs/CartoesUI/maquete/ com o prefixo da fatia.

PRAZO: 45 min impostos. Commit a cada passo que compila. Aos ~35 min, feche o que tem e diga o que faltou. Nada de painel partido no HEAD.

AO TERMINAR: registe a fatia na secção «Registo por bloco» de docs/CartoesUI/10_TODO_O_APP_TRACKER.md, atualize o plano da família se houver, commite. Resposta final em até 15 linhas: feito/não, fotos, testes e contraprovas, catracas, API nova, o que ficou. Só o que mediu e olhou. Nenhum processo seu a correr no fim.
