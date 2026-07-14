# HANDOFF — continuação da linha `line/motion-value` (Motion Nodes)

**Data:** 2026-07-14 · **Para:** o **próximo agente-de-linha** (você) · **De:** o agente que fechou as
7 fatias da jornada de 2026-07-13 · **Modo:** **L** (worktree, DIRETRIZ §1.5)

> **A integração ANTERIOR está CONCLUÍDA.** As 7 fatias entraram no `main` (`4d203d48`, jornada de
> 6 linhas) e a `line/motion-value` foi **fast-forwardada até lá** — a linha está **limpa,
> sincronizada e verde**. Este documento **supersede** o
> `HANDOFF_line_motion_value_continuacao_2026-07-12.md` e o
> `HANDOFF_line_motion_integracao_2026-07-13.md` (aquele era pro integrador; já foi consumido).

---

## 0. ABERTURA DA LINHA (faça isto ANTES de qualquer coisa)

A worktree **JÁ EXISTE e já está sincronizada com o `main` integrado**. Você **não** cria nada.

```bash
# 1. o hardware define o MODO (tem que dizer `workstation`; se disser `constrained`, PARE)
cd /home/enio/Documentos/Projetos/PH2D && bash scripts/hw-profile.sh

# 2. entre na SUA worktree (todo read/edit/git/cargo acontece AQUI DENTRO, sempre)
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value
git branch --show-current          # DEVE imprimir: line/motion-value
git status -sb                     # DEVE estar limpa

# 3. rebase no main (início de TODA jornada)
git -C /home/enio/Documentos/Projetos/PH2D fetch origin && git rebase main

# 4. warm-up (o target/ desta worktree é próprio; 1º build frio demora — é ESPERADO)
cargo check -p ph2d-eval-motion -p ph2d-panel-motion-graph

# 5. leia INTEIROS, aqui dentro:
#    docs/IntegracaoMultiAgente/DIRETRIZ.md               -> §0, §1.5, §2, §6
#    docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md -> TUDO (e RELEIA a cada passo)
#    docs/Motion Nodes/01_plano_modulo_motion_nodes.md    -> §3 (roadmap M0..M5)
```

### As regras permanentes (Modo L — valem até o fim, SEM exceção)

| | Regra |
|---|---|
| **A** | **TODO** read/edit/git/cargo acontece **dentro da worktree**. O mesmo path relativo existe nas DUAS árvores — editar `crates/…` na raiz é editar a árvore **ERRADA**. Na dúvida: `pwd`. **Mutação de arquivo = SEMPRE caminho absoluto** ([[feedback_sed_relative_path_hits_primary_cwd]]). |
| **B** | Edite a pasta do módulo à vontade. **Foundational você PODE e DEVE tocar** (com cuidado). **PARE e reporte ao Enio** só se: (a) **contrato congelado** (CLAUDE.md §6 — exige ADR), ou (b) o rebase conflitar **FORA** dos seus arquivos. |
| **B'** | Ao **CRIAR** foundational novo, projete pra **ISOLAMENTO**: módulo **irmão** > engordar arquivo compartilhado; ponto de extensão **append-only**. Todo id/const/variant novo: pegue o próximo livre e **ANOTE no handoff**. |
| **C** | Commits locais frequentes: `git commit --no-verify`. **NUNCA** `push`. **NUNCA** `--force`. **NUNCA** `git add -A` fora dos seus paths. |
| **D** | Conflito em `Cargo.lock` ou arquivo **GERADO** (`ph2d-node-registry-init`): **NUNCA** resolva à mão — **regenere** (`cargo run -p ph2d-node-sync`). |
| **E** | Fechamento = **gate batched** (§6). Depois **PARE** — **NÃO integre nem faça ship**. |
| **F** | **Ship: NUNCA por conta própria.** Integrar/pushar sem ordem = **violação do protocolo**. |
| **G** | **UI canônica:** zero hex, zero `f32` literal de UI, zero string hardcoded (HR-15). **UI do app em INGLÊS.** |
| **H** | **Handoff de integração é entregável obrigatório** ao fechar (DIRETRIZ §1.5.9). Reporte *"linha pronta + handoff"* e **ESPERE**. |

---

## 1. O método desta linha (o Enio já ratificou — NÃO reinvente)

**O Enio diz "próximo" e VOCÊ escolhe a fatia.** Cada fatia:

1. **REGRA-OURO — pesquise ANTES de codar.** O algoritmo padrão-ouro da indústria **e** o melhor
   nome (Houdini / Cavalry / After Effects / C4D MoGraph / Blender GN / Unreal). **Porte por
   SEMÂNTICA, não por código.** Sem pesquisa = fatia rejeitada.
2. **HR-5 — produção transcendental-free** (`sin`/`cos`/`exp`/`pow`/`atan2` proibidos; `sqrt` OK).
   Use as aproximações que já existem (`trig.rs` parabólico) — **copie o leaf**.
3. **Demo auto-playing pequena** no boot document (`shells/desktop/src/motion_demo_strobe.rs`).
   Regra permanente: **"simplifique o exemplo"**. Feature nova = **exemplo pronto pro smoke**, nunca
   *"monte você"*.
4. **Testes FALSIFICÁVEIS** (não "compila = verde"): o teste tem que **falhar** se a costura
   quebrar. Prove a **CORRENTE INTEIRA**. E **mute o código de produção** pra ver o gate ficar
   vermelho — mutação que sobrevive é gate frouxo (ou gate faltando).
5. **Nota-ADR numerada** em `docs/Motion Nodes/NN_<tema>_nota_adr.md` — **a próxima livre é a 63**.
6. **Gate de fechamento** (§6) → **commit** → **PARE**.

---

## 2. Onde a linha está (VERIFICADO em 2026-07-14, não é chute)

- **88 crates-nó registradas** (`cargo run -p ph2d-node-sync` diz o número).
- **M0 · M1 · M2 · M3: fechados.** O **editor F2 está COMPLETO**: busca no add-menu · Ctrl+G/Ctrl+Alt+G
  (subgrafos) · faca (`K`) · probe (`P`) · backdrops (+ paleta) · duplicate (Ctrl+D) · **F2 rename**.
- **Verde na árvore integrada** (rodado hoje, por exit code): contrato congelado **3/3** ·
  `ph2d-nodegraph` · `ph2d-panel-motion-graph` **89/89** · params + eval + os 2 nós novos ·
  shell `test(motion) or test(rename)` **85/85**.
- **A FILA DE FAN-OUT ACABOU.** O catálogo do plano está construído. **O que resta é decisão do
  Enio** (§3), não escolha do implementador.

### As 3 conquistas arquiteturais desta linha (memorize — são o padrão canônico)

1. **O canal de param não-f32** (doc 32, jornada anterior): o `NodeManifest` está **CONGELADO** e o
   `ParamSpec` é f32-only — mas os params **vivem no `Graph`**, não no manifest. Precisa de string /
   path / curva? **Use este padrão. NÃO bumpe o `NodeManifest`.**
2. **Um param dirigido é uma ARESTA que o manifesto não conhece** (doc 58). Porta dinâmica é
   impossível; aresta é **estado de documento**. Os 88 nós ficaram dirigíveis **sem uma linha de
   mudança em nenhum deles** — porque todos leem param pelo **mesmo funil** (`EvalCtx::param`).
3. **Nesting é uma dobra da VISTA** (doc 57): o `Graph` **nunca aninha**. O cook é **byte-idêntico**
   com e sem grupo.

> **É a 3ª vez que o contrato congelado ESCOLHE a arquitetura — e a 3ª vez que o resultado é melhor
> do que o que teríamos feito sem ele** ([[feedback_frozen_contract_can_pick_the_architecture]]).

---

## 3. A fila — **tudo aqui é DECISÃO DO ENIO** (pergunte, não escolha)

> **Jornada de 2026-07-14 (Enio: *"faça tudo que sobrou"*): 3 das 4 FECHARAM.** `motion.delay`
> (doc 63) · W4.T4 dock da timeline (doc 64) · `motion.path` + o **canal de externals** (doc 65).
> **Sobrou UMA, e ela mudou de forma** — leia a linha 1.

| # | Item | Estado |
|---|---|---|
| 1 | **FX de PASSE** (glow/bloom/blur/vignette/levels/hue) | 🔴 **PRECISA DE DECISÃO** — [doc 66](Motion%20Nodes/66_fx_de_passe_a_premissa_do_plano_e_FALSA.md). **A premissa do plano é FALSA:** o compositor do Painter é **8-BIT** e o `game_rt` é `Rgba16Float` — passar o HDR por ele **destrói exatamente o que o bloom precisa** (os valores > 1.0). E o Motion **não é separável** no GPU (as instâncias dele entram no mesmo passe de sprites da cena). Duas formas possíveis, donos diferentes: **A** = post stack do FRAME (afeta TUDO — outro plano, outro dono) · **B** = RT próprio do Motion (blast radius zero). **Recomendo B.** |
| 2 | ~~**`motion.path`**~~ | ✅ **FECHADO** (doc 65) — e o que ele destravou é maior que ele: o **canal de externals** (`Cook::set_external` / `EvalCtx::external`), que é **como qualquer coisa que o APP possui entra no grafo**. |
| 3 | ~~**W4.T4 — dock da timeline**~~ | ✅ **FECHADO** (doc 64). Não faltava sistema; faltava **geometria**. |
| 4 | **GPU / M5** ([plano](plans/2026-07-gpu-resident-node-pipeline.md)) | 🔴 Exige **linha foundational DEDICADA** (`line/cook-parallel`, depois `line/gpu-nodes` **com ADR** — a Fase 1 **descongela o contrato**, CLAUDE.md §6). **É ordem do Enio abrir. NUNCA enxerte aqui.** |
| 5 | ~~**`motion.delay`**~~ | ✅ **FECHADO** (doc 63). O valor real era o modo **Blend** (lag **sem overshoot** — o que a mola não dá). |

### Gaps conhecidos (nomeados, não escondidos)

- **Backdrop órfão:** um backdrop cujos nós foram agrupados fica na raiz emoldurando nada. Decisão
  consciente (um backdrop não *possui* nada), mas dá pra questionar.
- **Reuso/instanciação** (datablock do Blender / Gizmo / HDA): **fora por desenho** (doc 57 §7) —
  exigiria mexer no contrato congelado.
- **O menu do card não oferece input já alimentado por DENTRO** (doc 57 §6.1) — decisão, não
  esquecimento.

---

## 4. Fan-out de nó novo — o checklist mecânico (o que quebra se esquecer)

1. `crates/ph2d-node-<familia>-<nome>/{Cargo.toml, src/lib.rs}` — molde: `ph2d-node-force-buoyancy`
   ou `ph2d-node-motion-distribute-poisson` (os mais recentes).
2. **`MANIFEST`** (`NodeManifest` const) + `impl NodeOp` + **`pub fn register(reg)`** com
   `register_ui` (display_name **inglês**, `NodeUiCategory`, `NodeSilhouette`) + `register_param_ui`
   (`&[ParamUiHint]` — label inglês, min/max/step, `ParamWidget`).
3. **`cargo run -p ph2d-node-sync`** → regenera `ph2d-node-registry-init`. **É o ÚNICO conflito de
   merge esperado no rebase — sempre REGENERE, nunca resolva à mão.**
4. `cargo check -p <crate>` (inner loop — **nada de test/clippy por task**).
5. Ligue na demo + teste falsificável em `motion_state_tests.rs`.

---

## 5. Cicatrizes (leia — custaram caro, não são teoria)

- **UM CLIQUE É UM PRESS QUE DESLIZOU.** O dispatcher classifica press-release **com qualquer
  movimento** como `End` (arrasto), não `Click` — e **mão humana sempre desliza 1px**. O add-menu
  ficou **inusável desde que existe** com **75 testes verdes**, porque todos mandavam Down e Up **na
  mesma coordenada**. **Todo gate de clique deste painel desliza 1px**
  ([[feedback_a_click_is_a_press_that_drifted]]).
- **O painel do grafo NÃO PINTAVA nos testes.** `HeroLayout::for_viewport` monta o centro **sem
  split** → o rect do grafo é **ZERO** e a pintura **retorna antes de desenhar**. Era por isso que
  ele não tinha gate de pintura — o buraco onde o bug cabia. Use
  `HeroLayout::for_viewport_split(...)` + `MockPanelHost::paint_with_layout`.
- **Cace a CAPACIDADE, não o símbolo.** Eu greppei um intent sem emissor e afirmei — **num commit e
  num ADR** — que a feature "nunca foi construída". **Era falso:** existia por **outro canal** (o
  painel de params). *"Quem emite X?"* e *"o usuário consegue fazer X?"* são perguntas diferentes, e
  só a segunda importa — e a resposta se dá **executando**, nunca por grep
  ([[feedback_stale_comment_and_dead_code_lie]] 3º caso).
- **O gate leu DOIS REFERENCIAIS.** O sink é a cena **como DESENHADA** (o `motion.move` desloca a
  população antes do output) e as constantes da demo são de **SIMULAÇÃO**. Só peguei **olhando a
  trajetória** ([[feedback_derived_coordinate_seed_must_match_sample]]).
- **Número mágico duplicando constante = gate verde apontando pro vazio.** O gate do chão escrevia
  `-2.0 + 2.4` à mão; quando o leito se moveu, ele seguiu **verde** medindo água vazia. Gate **lê a
  constante**.
- **`| grep` mascara o exit code** do script. **Meça pelo exit code**
  ([[feedback_pipe_masks_script_exit_code]]).
- **Caps de LOC:** painel ≤ **600**, workspace ≤ **700**, fn ≤ 200. O `cargo fmt` **re-expande** →
  formate **ANTES** de medir. Estourou? **Extraia módulo irmão**, **nunca** allowlist.
- **`rustfmt` avulso quebra no `cook.rs`** (*"let chains are only allowed in Rust 2024"*) — use
  **`rustup run 1.95 rustfmt --edition 2024 <arquivo>`**.
- **Gate `no_tofu_glyphs`** escaneia **string literal** (não comentário): um `→` num `assert!`
  reprova.
- **A costura não-testada:** esquecer de ligar a **saída** do nó novo até o `motion.output` faz o
  grafo **VALIDAR** e cozinhar **0 instâncias**. Só um `assert_eq!(pos.len(), N)` pega.

---

## 6. Gate de fechamento da fatia (1× por fatia, **não** por task)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value

# contrato congelado — TEM que dar NodeManifest=8 / NodeOp=2 / OpResolver=1
cargo nextest run -p ph2d-nodegraph -E 'binary(architecture_contract_surface)'
# arch-gates (staleness do codegen, tofu, LOC, clamp, magic numbers)
cargo nextest run -p ph2d-editor-core -E 'test(loc_cap)'
cargo nextest run -p ph2d-host-desktop -E 'test(staleness) or test(no_tofu) or test(loc_cap) or test(clamp) or test(no_magic) or test(wiring_parity)'
# os testes de verdade
cargo nextest run -p ph2d-nodegraph -p ph2d-panel-motion-graph -p ph2d-panel-motion-params -p ph2d-eval-motion
cargo nextest run -p ph2d-host-desktop -E 'test(motion) or test(rename)'
# clippy no fim
cargo clippy -p <suas crates> --all-targets -- -D warnings
```

⚠️ **O gate da linha NÃO é o `ship.sh`.** O ship (fmt do workspace / clippy `--all-targets` /
machete / deny / audit / typos) roda **só na integração** e **sempre acha latentes** — é esperado
([[project_integrator_ship_catches_latents_budget_iterations]]). **Não rode ship. Não pushe.**

**Smoke (o Enio roda, não você — dê o comando pronto com o `cd` junto):**
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
```

---

## 7. O que a integração de 2026-07-13 ensinou (leia antes do próximo fechamento)

O [registro da jornada](REGISTRO_integracao_jornada_2026-07-13.md) (6 linhas → `main`) tem duas
lições que **te atingem**:

1. **A integração causou 2 bugs que o `merge-tree` não viu** — e **um deles envolveu esta linha**:
   a linha do Painter e a nossa adicionaram **as mesmas deps** (`ph2d-host` + `bumpalo`) ao
   `ph2d-ui-testkit/Cargo.toml`, **em pontos diferentes do arquivo**. O git fundiu sem conflito e o
   TOML ficou com **chave duplicada** → erro de parse. **Um merge limpo no texto pode estar quebrado
   por dentro** ([[feedback_clean_text_merge_can_be_semantically_broken]]).
2. **Uma branch não pode "consertar" um índice compartilhado a partir da própria base.** A linha de
   áudio apagou 4 linhas do `project-memory/MEMORY.md` que **eram da main** (a branch forkou antes
   delas). A única resolução correta de uma lista que **SOMA** é a **UNIÃO**, feita contra a main de
   **HOJE**.

> **O `CLAUDE.md` §5 foi atualizado por MIM** (a entrada de Motion estava dizendo "67 nós" e "editor
> F2/F3 ainda é TODO" — o integrador não aplicou a frase que o handoff deixou pronta). **Não
> re-aplique.**
