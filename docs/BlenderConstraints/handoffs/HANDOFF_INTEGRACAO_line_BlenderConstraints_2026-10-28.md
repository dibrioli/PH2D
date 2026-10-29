# HANDOFF DE INTEGRAÇÃO — `line/BlenderConstraints`, W1–W6 (28/10/2026)

> Linha FECHADA para integrar (DIRETRIZ §1.5.9). Smoke do dono: **OK** em todas as ondas (W6 em 28/10). Integrar só por ordem
> EXPLÍCITA do Enio. Desenho: [`01_plano.md`](../01_plano.md) · [`02_W2_ossos`](../02_W2_ossos.md) · [`03_W3_poderosas`](../03_W3_poderosas.md) ·
> [`04_W4_cad`](../04_W4_cad.md) · [`05_W5_diferenciais`](../05_W5_diferenciais.md) · [`06_W5_resto`](../06_W5_resto.md) ·
> [`07_W6_pontas`](../07_W6_pontas.md) — não duplicado aqui. História por onda: os `HANDOFF_CONTINUACAO_W*` desta pasta
> (o modelo do último: [`W5.5`](HANDOFF_CONTINUACAO_W5_5_2026-10-27.md)).

## 1. Superfície de colisão

| | |
|---|---|
| Linha | `line/BlenderConstraints`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-BlenderConstraints` |
| Merge-base | `c3662cd8e4` — o `main` **não andou** (`git rev-parse main` == merge-base) ⇒ a integração é **fast-forward** depois da suíte completa |
| Commits | **166** (`git log --oneline c3662cd8e4..HEAD`; 165 antes deste handoff) |
| Diff | **676 ficheiros, +115 641 / −2 358** (`git diff --shortstat c3662cd8e4 HEAD`) |

**Crates NOVAS (6)** — `Cargo.toml` do workspace e `Cargo.lock` colidem aqui:

| Crate | O que é |
|---|---|
| `ph2d-constraints` | a lei pura das constraints 2D: o que cada `Kind` faz a uma pose, sem saber o que é uma entidade |
| `ph2d-constraints-ecs` | a pilha `Constraints` de uma entidade (objecto ou osso), avaliada da pose autorada; relógio da mola, `depois_do_motion` |
| `ph2d-constraints-cola` | a COLA entre dois objectos: relação simétrica resolvida por grupo antes da pilha (ADR 0193) |
| `ph2d-app-constraints` | a família: cartão do Inspector, pick, gizmos, bake, roteador `PH2D_CONSTRAINT_SMOKE` (cenas 1–12) |
| `ph2d-hud-layout` | o HUD que se arruma: relações entre irmãos de um canvas pelo Cassowary incremental (`kasuari`; ADR 0194) |
| `ph2d-vec-constraints` | o solver das regras de desenho técnico do vetor (`kcl-ezpz =0.2.18`; QR esparsa para o posto) |

**Foundational tocado — tudo ADITIVO** (`git diff --shortstat c3662cd8e4 HEAD -- crates/<x>`):

| Crate | Delta | O quê (por que é aditivo) |
|---|---|---|
| `ph2d-ecs` | 18 ficheiros, +1105/−15 | `hud_arranjo` (`UiArranjo`), `canal_constraint` (canal **PICO = 52**), `canal_cad`, `timer.rs` (`TimersQuiet` + hook `on_remove` do `Timers`), canais de valor; componentes novos registados no FIM |
| `ph2d-runtime` | 3, +45/−4 | `SignalOrigin::Timer { notice }`, `SignalOutbox::read_notices`; quem não usa o campo lê o de antes |
| `ph2d-render` | 3, +63 | `sprite_repoe::repoe_no_presente`, `RenderInstance::pose_de` (a MESMA conta do extract, agora nomeada) |
| `ph2d-gpu-cook` | 2, +16/−8 | as tomadas levam o id da partícula; o sink que só a pilha pede lê-se em `P`+`id` (44 → 12 B/linha) |
| `ph2d-eval-motion` | 3, +57/−2 | `taps`; `scrub` repete o tique só com a timeline PARADA |
| `ph2d-component-desc` | 5, +59/−5 | catálogo: `UiArranjo` intrínseco oculto (`OtherEditor`, `Ref` `RefKind::Object` `Propagate`); `Constraints` `.hidden(Hidden::List)` |
| `ph2d-app-components` | 16, +1126/−1 | entrada nova em `REMAPPERS` (`instance_refs.rs`); passe `hud_arranjo` |
| `ph2d-app-macros` | 1, +5/−1 | `mundo/portas.rs` |
| `ph2d-editor-core`, `ph2d-app-vec`, `ph2d-timeline`, `ph2d-preview-drive` | 67 ficheiros no total, +4331/−160 | ferramentas Rules/HUD Layout, canais por vértice, `field_at`/`instante` |

A shell (`shells/desktop`) só **cresce em composição** e volta abaixo dos tectos (ver §4); o gizmo da constraint mora agora em
`shells/desktop/src/input_dispatch/despacho_constraints.rs`.

## 2. Contadores como DELTA

| Contador | Main (merge-base) | Linha | Delta |
|---|---|---|---|
| `PROJECT_SCHEMA` | **193** | **198** | +5: `193→194` pilha de constraints (W1, ADR 0192) · `194→195` id por VÉRTICE (W4.2) · `195→196` a COLA (W5, ADR 0193) · `196→197` `UiArranjo` (W5.5, ADR 0194) · `197→198` `TimersQuiet` (W6; irmão porque o postcard é posicional) |
| ADR | máx. **0191** | 0192, 0193, 0194 | +3 — **provisórios**, o integrador recompta (`git ls-tree c3662cd8e4 docs/architecture/decisions/`: nada ≥ 0192) |
| Cenas `PH2D_CONSTRAINT_SMOKE` | 0 | 1..12 | +12 |
| Crates | — | +6 | ver §1 |

⚠ Os handoffs W5.x dizem «196→197» e «Schema 196 provisório»: eram os números DA LINHA, mas o main estava em **193**. A escada
é `193→198` (confirmado em `shells/desktop/src/project_schema.rs`). Recontar com `python3 scripts/schema-recount.py <valor que o
degrau passa a ter no main>` se o main andar; a tripla `(…, 13, 22)` e o degrau `PROJECT_SCHEMA_GEOMETRIA_V23 = 194` /
`PROJECT_SCHEMA_SEM_CONSTRAINTS = 193` vêm no mesmo ficheiro.

## 3. O que um merge pode partir

- **Ordem do enum `Kind`** das constraints e a ordem dos componentes registados (postcard é posicional): se o main registar
  componentes ou degraus de schema entretanto, recontar (escada, tripla, hook `hoje[0] = 0xC4` em `project_geometria_v194_tests.rs`).
- `VEC_SCENE_SCHEMA_VERSION` `23 → 24` (`VecVertex.id: VertId`, `VecPath.next_vert` no FIM).
- Ids de painel/ícone: a tool **Rules** e a **HUD Layout** (`vector.mode.hud`, `vector.hud.*`) são modos da coluna do Vetor;
  enum de ícones e `ALL_ICONS` seguem a ordem alfabética dos svg; o catálogo de descritores segue `canonical_name`.
- O número de cenas do roteador (`max_level: 12`) e `smoke_particula::{FONTE, PARTICULA}` (gate lê o fonte do Motion).
- Tectos de ficheiro: `shape_handles_live.rs` está em **700 linhas exactas**; a shell no tecto.
- Hook de segurança: **nunca escreva a palavra «kill» numa mensagem de commit** («critério de morte»).

### Contratos congelados (§6) — NÃO tocados

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-BlenderConstraints
git diff c3662cd8e4 HEAD --stat -- crates/ph2d-nodegraph crates/ph2d-vector-doc crates/ph2d-imageio \
  crates/ph2d-editor-core/src/tool.rs crates/ph2d-editor-core/tests/it/architecture_tool_contract_surface.rs
```

Resultado: **saída vazia** (zero ficheiros). Cobre `NodeOp`/`OpResolver`/`NodeManifest` (`ph2d-nodegraph`), `Tool=12`/
`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` (`tool.rs` + o gate `architecture_tool_contract_surface`), o data-model do
Vetor (`ph2d-vector-doc`, gate `architecture_vector_contract_surface`) e o `ColorProfile` (`ph2d-imageio`). O motor novo
`ph2d-vec-*` não é congelado. As crates de constraints ficaram fora do §6 (contrato próprio, ADR 0192).

## 4. Prova de fecho (onda W6 e a linha)

W6 (ordem do dono 27/10, smoke OK 28/10) fechou as seis pontas de [`07_W6_pontas.md`](../07_W6_pontas.md): remapeamento do
`UiArranjo` ao duplicar, `pico` (Atrair) no cartão, posto esparso, mola ao vivo com o tempo parado, Copy Particle e «Show
notice» por Timer. Commits: `580dc7a0c0`, `ad4629a408`, `fe0fa96a17`, `92df90d3d9`, `1b4ce6e5e4`, `164918f392`, `81dfe8ce23`,
`aac9c8e776`. O último devolveu a shell abaixo dos tectos (gizmo da constraint movido para `despacho_constraints.rs`;
`project_tests` 604 → 600; três `fase_*` de 201 → ≤ 200).

⚠ **Correcção ao handoff W5.5 (e W5.2–W5.4):** os três vermelhos rotulados «herdados do main» —
`shell_files_respect_hr18_loc_cap`, `shell_functions_respect_their_ceiling`, `a_altura_de_abertura_de_um_painel_so_encolhe` —
eram DESTA linha (a pilha de constraints do Inspector, a truncagem de rótulos e o crescimento da shell). Estão **verdes**.

**Medido** (W6 item 3; perfil `smoke`, variantes intercaladas no mesmo processo, mínimo; `loadavg` 19–58, outras linhas a
compilar — tabela completa em `07 §2`): juntar UMA regra a 200 pontos — escada 202: **17,3 → 0,47 ms**; misto 200: **33,1 →
1,32 ms** (outra sonda 0,87); posto denso 14,5 → esparso 0,15 / 0,52 ms. Leitura de partículas (10 k): corrente inteira 0,60 ms × só o
id 0,02 ms. Números do critério de morte da W5 (Valor 0,468 · behaviours 0,404 · HUD 0,456 ms / 1 000; tecto de grupo 64) em
[`W5.5`](HANDOFF_CONTINUACAO_W5_5_2026-10-27.md).

**Portões verdes no fecho:** baterias por nome sobre 17 pacotes · `clippy --all-targets -D warnings` + `rustfmt` nos 19 pacotes
tocados · clippy do `ph2d-host-desktop` verde depois da correcção de fecho · `file_loc_caps` (4/4), `fn_loc_caps`,
`the_shell_only_shrinks`, `a_altura_de_abertura_de_um_painel_so_encolhe`.

**Suíte completa — PARCIAL (28/10):** uma corrida de `foundational-integrate.sh` iniciada por engano foi parada pelo dono
(«não integre, aguarde sua vez») a **18 756 / 21 978** testes (o main NÃO foi tocado; `docs/PixelArt/` do primário, sujo de
outra linha, foi posto de lado e devolvido com sha256 idêntico). Ela achou **9 vermelhos, todos desta linha**, curados por nome:

| Vermelho | Causa | Cura (commit) |
|---|---|---|
| `register_ecs_components_populates_registry` (ecs), `registers_sprite_alongside_ecs` (render), `registers_luau_script_alongside_ecs` (script) | `UiArranjo` + `TimersQuiet` | 108 → **110** · 109 → **111**, delta +2 anotado (`d3f7df06cd`) |
| `the_shell_turns_the_signal_frame_before…`, `the_authored_intent_queue_has_a_drain…` (shell) | o dreno virou `read_notices` | as agulhas seguem-no (`d3f7df06cd`) |
| `every_ignore_in_the_motion_crate_names_its_class` | `#[ignore]` da sonda sem classe | `sonda: …` (`d3f7df06cd`) |
| `panel_files_under_loc_cap` | `sections/constraint.rs` 641, `state.rs` 616 | menu «Add» → `constraint_add.rs` (548); testes da tinta → `state_tint.rs` (576) (`45f47c853d`) |
| `panel_functions_under_loc_cap` | `ph2d-panel-vector::apply_event` 201 | os dois eventos desta linha num `if` (198) (`45f47c853d`) |
| `no_row_label_carries_its_own_unit` | «Speed (°/s)» | «Speed»; o campo leva `deg/s` por `Unidade::GrausPorSegundo` (guardado em graus, sem conversão; não segue o ângulo do projecto em radianos) (`45f47c853d`) |

⇒ **Ao integrador:** a suíte INTEIRA continua obrigatória (dono 23/10) — **3 222 testes nunca correram** e os 9 curados só
correram por nome. Mudaram crates de que muitas dependem (`ph2d-ecs`, `ph2d-editor-core`, `ph2d-app-vec`, `ph2d-timeline`,
`ph2d-render`, `ph2d-runtime`).

## 5. Smokes (todos aprovados pelo dono)

Comando de todas as cenas (troque `<n>` por 1..12):

```
cd /home/enio/Documentos/Projetos/PH2D; env PH2D_CONSTRAINT_SMOKE=<n> cargo run -p ph2d-host-desktop --profile smoke
```

| n | O que mostra · como saber que deu errado |
|---|---|
| 1 | **Mão e espada**: a mão passa; no quadro 24 pega a espada do chão, no 48 larga-a noutro sítio. Errado: a espada salta ao pegar/largar |
| 2 | **Olho**: arraste a bola vermelha; a seta aponta para ela e a pupila corre atrás sem sair do olho. Errado: a pupila sai do olho |
| 3 | **Ponteiros**: o de cima pára em 45°, o de baixo copia a guia a 50 % e fica em 180°. Errado: 0° ou 170° |
| 4 | **Braço**: a IK de 2 ossos persegue a bola; a mão roda com o rodador |
| 5 | **Antena**: o corpo vai e volta; a ponta (mola) fica para trás, balança e assenta |
| 6 | **Régua de desenho**: arraste um canto no modo Edit e o resto obedece; no Play a altura do dente do meio estica e volta |
| 7 | **Caixa com tampa**: abre na tool Rules; dez regras à vista; o Play abre e fecha a tampa pelo ângulo |
| 8 | **Braço articulado** (a cola): manivela, biela e balancim presos por 4 pinos e uma mão soldada; arraste cada peça e as outras seguem |
| 9 | **Mostrador**: a bóia sobe e desce; o ponteiro segue com mola e a lâmpada acende com a altura |
| 10 | **Recreio**: limalha atraída pelo íman, moeda que treme mais perto dele, balão que flutua, lua em órbita |
| 11 | **Painel do jogo** (HUD): a pontuação sobe no Play e a moeda fica colada ao lado; estreite a janela e a pontuação encolhe antes de a moeda invadir o minimapa. Ao dar «Show notice» ao Timer o aviso não aparece a cada tique. **Ctrl+D mantém o arranjo** — teste apagando o HUD original (o HUD é colado ao ecrã, não se arrasta) |
| 12 | **Partículas e mola**: a Bola anda montada na partícula 1 da Fonte (Copy Particle) e fica na pose dela quando morre; com a timeline PARADA, arraste o corpo da antena e a ponta balança e assenta |

`PH2D_CONSTRAINT_SMOKE_TOCA=1` é um interruptor só para fotografar (abre a cena 12 a tocar, `smoke_particula.rs`) — não é o caminho do dono.

## 6. ⚠ Abertos (honestos)

**Da W6** — ainda UM quadro atrás para um objecto preso a uma partícula: a união da caixa da multi-selecção; a caixa de um dono
que rodou/escalou no passe tardio (só a translação se repõe); a malha de uma imagem presa ao esqueleto; o rank de um dono em `YSort`;
os números do Inspector. A via de **placa** do Copy Particle fica 1 quadro atrás (leitura sem espera). Uma sobreposição de instância
do `UiArranjo` é reescrita pelo mestre (mesma regra das juntas de física). O custo do re-posicionamento (`depois_do_motion`) NÃO foi
medido. A mola ao vivo arrastada e a ferramenta HUD por cliques NÃO se fotografam (eventos sintéticos não chegam ao Xwayland) —
provadas só por portões de costura.
«Frequency (Hz)» do cartão também traz a unidade no rótulo — o portão não a vê (`Hz` fora da lista) e não há unidade Hz para o campo.

**Herdados das ondas anteriores e ainda abertos (conferir ao integrar):** o Bake não dá feedback nem pede confirmação (W1d 2); o
desvio do Child Of não é animável (W1d 3); o bake avalia só curva→canais→pilha, sem física/motion (W1d 5); `bind_on_create` com
`PreviewDrive` vazio no meio da pilha (W1d 6); o cartão da Luz 2D fixa metros (W1d 7); Flip «desenho por ângulo» sem canal animável
(W3 1); entrada da Spring sob Arrange stack/Autoplay (W3 3); Action por POSIÇÃO do clip (W3 5); `MAX_PASSOS_DA_ORBITA = 100 000`
(W3 6). Fechados nesta janela: ponto de simulação (W3 2), mola com o relógio parado (W3 4), `pico` do Atrair, remapeamento do
`UiArranjo`, aviso «Signal» por tique.

## 7. Armadilhas para o integrador

- Crates de que muitas dependem mudaram ⇒ a suíte COMPLETA é obrigatória e é a única coisa que a linha não correu.
- Recontar `PROJECT_SCHEMA`/ADR só se o `main` tiver andado (hoje não andou): `python3 scripts/schema-recount.py <m>`.
- rustfmt/hook reformata ao gravar: âncoras de UMA linha em edições por script.
- Grelha do HUD: compare saídas do solver com `PARADA`, nunca `==`; `FIXO` = 2e6 (empate STRONG dependia do hash).
- Nunca a palavra «kill» na mensagem de commit.

## 8. Integração no main (29/10, ordem do dono, depois da GameEngineFuncional)

- Rebase sobre `23765c4475` (GEF no main): 11 commits conflitaram, resolvidos por estágios; cada commit comparado com o original
  (multiconjunto ±). Mergiraf largou o `mod constraints;` do `ph2d-i18n` e disse «Solved» — reposto. Tip antigo em `backup/BC-pre-rebase`.
- **Contados sobre o main do dia:** `PROJECT_SCHEMA` **198 → 203** (199 pilha · 200 id por vértice · 201 cola · 202 `UiArranjo` ·
  203 `TimersQuiet`; tripla `(203, 13, 24)`; `GEOMETRIA_V23` = 199, `SEM_CONSTRAINTS` = 198; 193–197 continuam no braço da GEF).
  Registo `ph2d-ecs` 113 + 2 = **115**; espelhos `ph2d-render`/`ph2d-script` 114 + 2 = **116**. Censo de comandos do Inspector
  92 + 8 = **100**. ADR 0192–0194 (o main acaba em 0191).
- **6 vermelhos da combinação** (cada lado verde sozinho), curados em `540b42e278`: o «Run From» da GEF entra na porta px/m; os
  testes da câmera e do Run From escrevem no seletor m/px; o v197 da GEF leva a cena vetorial v23; `ph2d-i18n/lib.rs` 703 → 700;
  `contradicao` do `ph2d-hud-layout` era 6/12 pela ordem interna do solver (agora 12/12); a cena 4 do CodeEditor = carga.
  Antes: `fase_snapshots_publish` 201 → 198 (o cartão publica por `ph2d_app_constraints::inspector::publica`).
- **Suíte (`nextest-impacted`, BASE=main, fecho):** 22 245 testes, **22 244 verdes**; o único vermelho, `a_cena_3_conduzida_passo_a_passo`
  (ph2d-host-code-scenes), passa sozinho (`loadavg` 19) — flake de carga já listado pela GEF. `cargo check --workspace
  --all-targets` com warnings negados limpo; registos sincronizados; censos da árvore combinada 122/122.
- `project-memory/MEMORY.md` fica igual ao do main (o primário tem mudanças vivas de outras sessões); os ponteiros desta linha
  entraram na cópia viva do primário, e a lição W4.6 do Auto Key voltou ao tópico com o `(49)`.
- `docs/PixelArt/` (não rastreado, de outra sessão) ficou intocado no primário; os passos do `foundational-integrate.sh` correram à mão.
