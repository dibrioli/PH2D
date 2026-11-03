# Prompt da linha REAL — UI em cartões (para colar numa janela nova, a partir do `main`)

Você vai abrir e conduzir a **linha de implementação real da UI em cartões** do PH2D. Uma linha de TESTE (`line/TrelloUITeste`, worktree `Worktrees/line-TrelloUITeste`, nunca integrada nem pushada) provou esta UI. O dono fechou-a em 02/11/2026: «Já conseguimos testar a UI e ela funciona muito bem e é linda». O seu trabalho é **trazer para o `main` o que foi aprovado e terminar o que falta**, sem perder nada do que o dono já aprovou no smoke.

---

## 0 — Quem é quem e como falar

- O **Enio** é o DONO do produto, não um engenheiro. Ele decide e testa. Responda-lhe **em português, curto e sem jargão** (nada de crate, gate, ADR ou nome de função, salvo se ele pedir) e diga o que ele consegue FAZER agora.
- **Smoke sempre em passos numerados:** (1) o comando completo com o `cd`; (2) onde clicar, com o nome que aparece na tela; (3) o que tem de acontecer; (4) como saber que deu errado. Fotografe a cena antes de a mandar (`docs/Components/ferramentas/fotografa_cena.sh`, com HOME isolado; nunca `spectacle`).
- **Decisão de PRODUTO** (como algo parece ou se comporta para o artista): pare SÓ esse item e pergunte numa linha, com opções (a ferramenta de pergunta com 2–4 escolhas). **Decisão TÉCNICA** é sua: escolha o padrão-ouro e diga-o no handoff.
- **O app é SÓ em inglês** (toda string de UI, dicas em Plain English). Os docs, os handoffs e os commits ficam em português técnico: o leitor deles é a próxima LLM.
- Você **nunca integra, nunca faz push, nunca faz ship** por conta própria. A linha acaba no HANDOFF.

## 1 — Abrir a linha (antes de ler qualquer coisa grande)

1. `/pd-linha-abrir`. Sugestão de nome: `line/CartoesUI` (worktree `Worktrees/line-CartoesUI`, a partir do `main` de agora). Depois: `cd <worktree> && pwd && git branch --show-current && git status --short && git log -1 --format=%h`.
2. `bash scripts/hw-profile.sh` decide o modo (workstation = Modo L).
3. **`CARGO_TARGET_DIR` próprio da worktree.** Partilhar entre worktrees troca os `.rlib` e a medição vale zero.
4. Traga a pasta de docs da linha de teste: `git checkout line/TrelloUITeste -- docs/CartoesUI` (o commit de referência da linha de teste é `50b58be84a` ou mais novo) e faça commit só dela.

## 2 — O que ler, nesta ordem (e mais nada no início)

1. `docs/CartoesUI/06_COMO_PORTAR_PARA_O_MAIN.md` — **como trazer o CÓDIGO** (estudo medido; ordem das famílias; comandos).
2. `docs/CartoesUI/05_ESPECIFICACAO_UI_COMPLETA.md` — **a LEI da UI**: a regra de referência (§0), a tabela «Smoke OK» (§1), o vocabulário, os soltos, o ABERTO (§8–§9).
3. `docs/CartoesUI/handoffs/HANDOFF_line_TrelloUITeste_2026-11-02_FECHO.md` — o fecho e as regras aprendidas.
4. Só quando precisar: `01_decisoes_do_dono.md` (cada decisão com a palavra do dono, D* e T*), `03_mapa_do_codigo.md` (onde mora cada coisa), `04_plano_de_construcao.md` (medições e contraprovas de cada bloco), `maquete/` (fotos aprovadas e maquetes HTML).
5. A memória do projeto (`project-memory/MEMORY.md`): leia o índice. Há decisões do dono que tocam nesta UI e que entraram no `main` por OUTRAS linhas depois de 29/10. Por exemplo: Effects/Filters e Fill/Stroke do Vector no layout do Figma (F43/F44), o rótulo que explica o campo, a unidade no rótulo e não na caixa, «todo arrastar-reordenar voa como o cartão do Kanban», o Auto Key que grava tudo e o Advanced do Vector à Figma. **Se o `main` já tem uma versão aprovada de um painel que os cartões também desenham, NÃO escolha sozinho**: mostre as duas ao dono e pergunte.

Delegue leitura larga ao `explorador`; não leia ficheiros grandes na janela principal.

## 3 — A regra de referência (vinculativa, dono 02/11)

1. **Nós mesmos**: os painéis e cartões cujo smoke o dono APROVOU (05 §1). Antes de desenhar algo, procure se já existe aprovado e repita-o.
2. **Figma**: para todo widget ainda não criado, consulte o Figma PRIMEIRO.
3. **Rive**: se o Figma não resolve a complexidade do widget.
4. **Inkscape, Godot, Unity**: só se ainda não resolver.

Outro app que já faz a coisa é um ORÁCULO que se CORRE sobre entradas nossas, nunca um fonte que se lê (CLAUDE.md §0.9; triagem de licença por artefacto primeiro).

## 4 — Fase A: a régua antes de mexer

- Corra no `main` intocado (a sua worktree recém-aberta) os portões que a linha de teste viu vermelhos: tectos de LOC do `ph2d-editor-core` e dos painéis, o DAG dos módulos da fundação, os `seam_*` do `ph2d-panel-inspector`, `node_id_collisions::every_non_literal_hash_is_named` e `the_shell_only_shrinks`. **Escreva a lista exata em `docs/CartoesUI/04_plano_de_construcao.md`** (novo bloco «Linha real — régua do main»). A regra: nenhuma família portada pode acrescentar um vermelho a essa lista, e um vermelho que já lá estava não é seu.
- Meça também o tamanho de hoje de `shells/desktop` (a catraca), porque a linha de teste fez a shell crescer +149 linhas. Na linha real o saldo da shell tem de ser ≤ 0.

## 5 — Fase B: o porte (06 §3, família a família)

Base de comparação: `4e648ec2b4` (o pai do 1.º commit dos cartões, que NÃO está no `main`). Estado final: o HEAD de `line/TrelloUITeste`.

Para cada família:
1. `git diff 4e648ec2b4 line/TrelloUITeste -- <paths da família> > /tmp/.../familia.patch && git apply -3 /tmp/.../familia.patch`.
2. Conflitos: resolva contra o `main` de hoje com o teste da família como juiz. `--ours`/`--theirs` só com a razão escrita. O mecânico (rebase, `Cargo.lock` regenerado, recontagens) vai ao agente `integrador`.
3. `bash scripts/cargo-check-narrow.sh <crate>` até compilar sem avisos.
4. Testes da família **por nome** (`bash scripts/cargo-test-narrow.sh <crate> <nome>`). Crate nova, esquema, deps e código movido entre crates são o único `motivo:` para a crate inteira.
5. Commit com `git add --pathspec-from-file=<lista>` e `git commit --no-verify --pathspec-from-file=<lista>`. Na mensagem: «portado de line/TrelloUITeste `<commit>`, família X» e o que mudou no porte.

Ordem (a dependência manda):
1. Docs (já feito no §1).
2. Fundação nova: `ph2d-card-stack`, `ph2d-card-rope` (crates novas), `ph2d-tokens` (tokens `Card*`/`Kanban*`, `derive::card_tone`), `ph2d-i18n` (`cartoes.rs`, só inglês).
3. Kanban com a mola extraída (D19): o `main` mudou o Kanban depois de 29/10, por isso **refaça a extracção contra o Kanban do `main`** e não aplique o diff às cegas. Controlo: `PH2D_KANBAN_SMOKE=1` fica igual a antes.
4. Núcleo `ph2d-editor-core`: `widget/card_ink.rs`, `widget/card_field.rs`, `panel/cards*.rs`, `interaction/state/card_*_ops.rs` (lista, solto, instâncias e face T63, alfinete, retrato e ensaio T61), o despacho (arrasto, menus), as portas que ramificam no corpo do cartão (`property_row` + `escolha`/`marca`, `widget/dropdown`) e `icons`. ⚠️ `panel/cards.rs` tem ~1 400 linhas: parta-o POR RESPONSABILIDADE ao trazê-lo, porque nenhum ficheiro pode passar dos tectos.
5. Bones (piloto): `ph2d-panel-skeleton` + `ph2d-app-skeleton`/`ph2d-skeleton-live`.
6. Inspector em cartões: `ph2d-panel-inspector` (`cards.rs` 791 linhas e a fn `cards` 305 — parta; `cards_componentes.rs`, `retrato.rs`, `sections/rows*`, `event.rs` → `com_retrato`).
7. Estilos de cor do Vector (D28–D32): `ph2d-vec-scene` (o esquema subiu 25 → 26 na linha de teste; **o número da escada RECONTA-SE** com `python3 scripts/schema-recount.py`, nunca se copia), `ph2d-app-vec`, `ph2d-panel-vector`, `ph2d-tool-vector`, os testes de projecto da shell.
8. A cola na shell (`render_loop/snapshots_inspector.rs` e as fases de retrato/alfinete): a lógica vai para uma crate (HOWTO de partir uma família da shell; leia a §2 dele antes do 1.º ficheiro). Saldo da shell ≤ 0.
9. Cenas de smoke: `ph2d-app-vec/src/smoke_cartoes_inspector.rs` (`PH2D_VEC_CARTOES_SMOKE=1..7`), `PH2D_CARTAO_CORDA_SMOKE`. O número da próxima cena conta-se.
10. Os testes de `ph2d-panel-registry-init/tests/it/` (`o_*`, `os_*`, `toda_dica_dos_cartoes`, `a_lista_de_cartoes_flutua_no_hero`…) entram COM a família que testam, nunca todos no fim.

**Interruptor:** os cartões nascem ligados, e `PH2D_CARDS=0` volta ao Inspector de hoje (`ph2d-panel-inspector/src/cards.rs` `active()`, e o mesmo no Bones). **Pergunte ao dono numa linha** se no `main` ficam ligados para todos ou desligados até ele mandar.

## 6 — Fase C: provar que o porte não perdeu nada

- Corra as cenas da tabela «Smoke OK» do 05 §1 e **fotografe cada uma** no mesmo enquadramento das fotos aprovadas em `maquete/`, lado a lado (aprovada × portada). Diferença visível = defeito do porte, e cura-se antes de seguir.
- Mande ao dono UM smoke de regressão, em passos numerados, que percorra o que ele já aprovou: lista flutuante, campo Figma com arrasto no ícone, cartão solto, alfinete com corda, duas cópias, reset ↺, Ctrl+Z, estilos de cor e o T63 (cor e notas por cópia).
- **Só depois do OK dele** comece a Fase D.

## 7 — Fase D: o que falta construir (05 §8–§9)

Comece pelo **cartão Camera** (smoke do dono 02/11, palavra dele: «cartão câmera todo bugado, sem título, mal organizado, com cores diferentes do padrão, Ponto de animação mal posicionado»). MEÇA primeiro a causa de cada um dos quatro; o 05 só tem hipóteses. A família provável é a dos «chamadores que montam a linha à mão», e a cura é UMA lei para a família, nunca um remendo no Camera (CLAUDE.md §0.10). A família:
- os 11 `property_label_row` (o nome numa coluna que o chamador monta; converter pede o chamador);
- os 6 `Checkbox::new` diretos (`anchor_mount_row`, `slice_nine` ×2, `visibility` ×2, `script_controlo`);
- os `paint_text` diretos em `sections/`, cuja cor não passa pelo `CardInk` (o doc diz 76; uma recontagem deu 104 — reconte com critério escrito);
- o popover das listas suspensas, ainda no tom do Inspector;
- o título que falta (o corpo começa sem o cabeçalho) e a marca de animação que não está na linha do rótulo.

Depois, o resto do ABERTO do 05 §9: a dobra partilhada entre a cópia 1 e a lista; o solto colado ao topo do canvas, sob a barra de modos; a sprite, que ainda não tem cartões; e os painéis que a linha de teste NUNCA tocou (Hierarchy, Timeline, Painter, Pixel, Flip, Motion, Áudio, Código, Constraints, Luz 2D, Macros, Game Engine, Assets…). Cada painel novo segue a regra de referência do §3 e passa pelo dono antes de ser dado como feito.

## 8 — Protocolo de cada item (sem exceção)

1. **Meça antes de curar**: onde o defeito aterra ≠ o que o provoca. Escreva a medição no 04.
2. **Teste com cliques reais** no hero montado (`HeroScreen` + `dispatch_pointer`). Um teste de painel que só lê o pedido não basta: estenda-o até ao número ou à cor na caixa. Teste de clique num título: pegue a pega MAIS LARGA com aquele id (`iter_registrations`). O registo dos painéis é do PROCESSO, e testes paralelos partilham o `InspectorState`. Uma fixture com sprite desliga o Inspector em cartões (desarme-a). Um cartão abaixo da dobra pede rolar a lista no teste (`set_panel_scroll` + `_live`).
3. **Contraprova**: o teste fica VERMELHO com a cura desligada e verde ao repor (com `touch`). Nas leis que importam, o agente `mutacao` faz as provas. Uma mutação que sobrevive pede que o teste seja estendido.
4. **Foto antes/depois**: o «antes» tira-se com o patch reverso (`git diff > p; git apply -R p; foto; git apply p`), com a cura INTEIRA desligada. A cena tem de mostrar o cartão aberto e acima da dobra, porque a foto não clica (semeie um solto em `.ph2d/sections.txt` via `FOTO_HOME_SEMENTE`).
5. Atualize os docs a cada decisão: 01 (decisão com a palavra do dono), 03 (mapa), 04 (plano, medições e contraprovas), 05 (se a lei mudou).
6. Commit por item, com pathspec.

## 9 — Regras duras da casa (as que esta UI mais toca)

- **HR-15**: zero hex, zero `f32` literal de UI, zero string escrita à mão. A geometria vem de `ph2d_card_stack::metrics`, a cor do `CardInk` e dos tokens, o texto de `ph2d-i18n` (`cartoes.rs`).
- `PanelEvent=4` e os contratos do CLAUDE.md §6 ficam intactos.
- A shell e as `ph2d-app-*` não crescem; nenhum ficheiro passa de 600 linhas (parta por responsabilidade); os tectos de LOC somam entre linhas.
- `rustfmt` só nos ficheiros tocados (`--config skip_children=true` nos que têm submódulos); build `--profile smoke` SEM avisos.
- **Testes só por nome** durante o trabalho. A suíte COMPLETA só corre na integração, pelo integrador.
- Comando pesado: `bash scripts/ph2d-run.sh <cmd>`.
- Edite com a ferramenta `Edit`. Um script que grava tem de ter `assert` de contagem.
- Nunca `git add -A`, `git stash` ou `git checkout` de ficheiros alheios.
- Contexto pequeno: delegue leitura ao `explorador`, baterias ao `verificador`, contraprovas ao `mutacao`, rebase e recontagens ao `integrador`, docs e handoff ao `documentador`. Uma janela nova por onda de trabalho, e o handoff e o prompt da janela seguinte entregam-se ao dono SEM ele pedir, depois do smoke.

## 10 — Fecho de cada onda e da linha

- Fim de onda (o dono fez o smoke): `/pd-onda-fechar` — handoff curto em `docs/CartoesUI/handoffs/` (commits, smoke com a palavra do dono, ABERTO, regras aprendidas) e o prompt completo da janela seguinte.
- Fim da linha (antes de integrar): `/pd-linha-fechar` (o portão batched 1× sobre o diff acumulado), `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`, e a entrada do CLAUDE.md §5 (≤ 700 bytes) com o link do handoff. A integração é só por ordem explícita do Enio.

**Primeiro passo agora:** §1 (abrir a linha e trazer `docs/CartoesUI`), §2 (ler 06 → 05 → FECHO) e §4 (a régua do `main`). Depois diga ao dono, em duas linhas sem jargão, o plano do porte, e faça-lhe a pergunta do interruptor.
