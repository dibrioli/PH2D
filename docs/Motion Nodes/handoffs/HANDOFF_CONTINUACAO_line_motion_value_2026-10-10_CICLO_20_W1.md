═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: ciclos 13–20 em `line/motion-value` sobre o `main` `2a925859d`, POR INTEGRAR. O ciclo 20
(O ACABAMENTO, [doc 129](../129_ciclo_20_o_acabamento.md)) está ABERTO: a W0 fechou (os oráculos
corridos e a crate `ph2d-lens` com as leis na CPU, 14 gates). Você abre a **W1**.

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -6 &&
        git status --short — ANTES de abrir qualquer ficheiro.

⛔ Fale com o Enio em PT-BR, curto, sem jargão (CLAUDE.md §0.8). Integrar/pushar só por ordem dele.
⭐ LEIA o doc 129 INTEIRO (é curto) — §2 tem as três medições que decidiram o desenho e §3 a lei de
   cada efeito. Protocolo do ciclo = doc 103 §1 (sete passos; o smoke é o tutorial em PDF).
⭐ CUSTO: leitura larga a agentes `explorador` em paralelo; gate batched a um `verificador`.
⚠️ Se o Enio trouxer um problema do que já está feito, CORRIJA antes de seguir.

O DESENHO (decidido e medido — não o reabra sem número novo):
- DUAS famílias, dois nós passthrough com a forma do `fx.glow` (doc 94 §0):
  · `fx.lens` — a LENTE sobre a CENA inteira, ordem FIXA aberração → vinheta → grade (+LUT `.cube`)
    → grão. Leis em `crates/ph2d-lens` (CPU, gateadas contra ImageMagick/ffmpeg);
  · `fx.filter` — os filtros da CAMADA do Motion, a ponte para a `FxOp` (doc 94: a ficha DERIVADA
    da `FxOp::SPECS`, nunca a lista à mão), na ordem do fio.
- ⛔ NÃO apender tipos à `FxOp`: é postcard posicional (bump de `PROJECT_SCHEMA`) e o painel do
  Vector corta em `MAX_FILTER_KINDS = 15` literal; a lente é da câmera, não de uma forma.
- ⭐ ONDE a lente corre: no ACUMULADOR DO MUNDO (`WorldRt`, `Bgra8Unorm` com vista sRGB de leitura,
  `crates/ph2d-render/src/world_rt.rs:38-40`, usos RENDER_ATTACHMENT|TEXTURE_BINDING|COPY_SRC) —
  é o único sítio onde a cena está inteira sem o chrome (antes do tonemap as formas do Motion da
  rota da CPU ainda não entraram: são do Vello). Costura: `shells/desktop/src/render_loop/
  present_chrome.rs`, DEPOIS da colagem da placa (`band_blit.blit(... Formas)`) e ANTES do vidro e
  do Vello. O modo em faixas tem de estar LIGADO quando há `fx.lens` (hoje a placa liga-o:
  `present.rs:299` `plan.banded |= self.motion_shell.placa.ativa()`; com o modo desligado o
  compositor lê o tonemap e a lente não teria onde correr). O sub-rect é o `scene_viewport` (o
  centro da vinheta é o da CENA). ⚠️ A catraca `the_shell_only_shrinks`: a lógica vai para
  `ph2d-app-motion`/`ph2d-render`; a shell só chama (e corta o que acrescenta).
- O WGSL é a MESMA lei da `ph2d-lens`: `Φ` de Abramowitz & Stegun 7.1.26 (não há `erf` no WGSL), a
  quadratura de 16 nós da vinheta, o `pcg3d` (inteiros de 32 bits: o MESMO bit), o OKLab por
  `concat!(ph2d_ease::wgsl_lib!(), ph2d_color::wgsl_perceptual!())` (o `pc_to_oklab`/`pc_from_oklab`
  do ciclo 18), a matriz do balanço calculada na CPU (`grade::matriz_balanco`) e passada como
  uniform, a LUT por `textureLoad` (sem filtragem de hardware: os pesos de 8 bits dela erram ~1/256)
  e amostragem tetraédrica à mão, a aberração com bilinear à mão (`textureLoad`, borda replicada).
  Leitura pela vista sRGB (o hardware descodifica), escrita codificada à mão (ou por uma vista sRGB).
  O passe lê uma CÓPIA do acumulador (não se lê e escreve a mesma textura).

AS WAVES QUE FALTAM (doc 129 §4):
- **W1** — o passe `ph2d-render/src/lens.rs` (+ `lens.wgsl`): uniform com os quatro efeitos, a
  cópia, o triângulo de tela, o sub-rect. Gate de PARIDADE GPU × `ph2d_lens::aplica_cpu` (molde:
  `crates/ph2d-render/tests/it/fx_stack_*_gpu.rs`, `#[ignore]`, `PH2D_GPU=1 bash scripts/ph2d-run.sh
  cargo test … -- --ignored`) em cada efeito sozinho e nos quatro juntos; o grão ao BIT do hash.
  Byte-idêntico sem lente (gate). Prova o WGSL com `naga` (molde `fx_stack_tests.rs`).
- **W2** — o nó `fx.lens` (crate `ph2d-node-fx-lens`, molde `ph2d-node-fx-glow`: `from_graph` →
  `ph2d_lens::Lente`), quatro `ParamGroup` (Aberração · Vinheta · Grade · Grão), defaults por
  `register_param_defaults` (a vinheta nasce LIGADA, `amount 0,4`, para o artista ver ao largar o
  nó; o resto neutro), i18n (`ph2d-i18n` `node_catalog.rs`/`node_params.rs`), a linha de
  conferência (`motion_param_reach_tests.rs` `every_node_has_a_conference_row_or_is_named_in_the_debt`),
  a LUT por NOME (param de texto, `register_text_defaults`; quem lê o ficheiro é o `ph2d-app-motion`,
  com a recusa do `ErroCubo` dita ao ARTISTA no cartão — memória «recusa que só o terminal vê»), o
  tique do grão = o tique do relógio do Motion (scrub = o mesmo grão).
- **W3** — `fx.filter` (doc 94 §1 inteiro): a camada do Motion desenhada ISOLADA (sprites pela
  lista do quadro + formas da placa; ⚠️ o halo do glow NÃO é a mesma lista do passe fundido —
  `motion_glow_layer.rs:92-165`; meça antes de reusar), o Motion fora do passe fundido quando há
  `fx.filter` (`present.rs:227-240`), a ingestão/resolução LINEAR premultiplicada do `FxStackPass`
  (hoje é sRGB 8 bits, `fx_stack_shader.rs` `cs_ingest`/`cs_resolve`), o passe «esta textura sobre
  aquele alvo» (a metade B do doc 94 §2), e o gate do blur contra `crates/ph2d-lens/tests/fixtures/
  blur_linear_s3.txt` (já gravada). ⏳ A faixa do `radius` em MUNDO: medir, não escolher (doc 94 §1).
- **W4** — a cena `=135` (`MAX_DEMO_LEVEL` está em 134 em `motion_state_demo_router.rs:34`; reconte
  se o `main` mexeu; roteador `motion_state_demo_router_ciclos.rs`, molde `motion_state_jogo_demo.rs`
  + `_tests.rs` lido NA SAÍDA pela bomba): FORMAS + SIMULAÇÃO com campos, a lente e um filtro,
  FOTOGRAFADA com o controlo (`docs/Components/ferramentas/fotografa_cena.sh <VAR=valor> <larg>
  <alt> <saida.png> [espera_s]`), em tempos diferentes; o relógio do grupo (`measure_the_*_group`,
  `--release`, o `loadavg` ao lado); o tutorial 13 (`tutoriais/src/13_o_acabamento.html` →
  `bash scripts/tutorial-pdf.sh`) com o gate dos passos (molde `every_step_of_the_game_tutorial_is_reachable`).
- **W5** — provas de mutação (agente `mutacao`) sobre as leis que importam (o sentido da
  aberração, a quadratura, a ordem fixa, o tique do grão, a neutralidade byte-idêntica), o gate
  batched e o handoff de integração `HANDOFF_INTEGRACAO_..._CICLO_20.md` + a linha do §5.1 do
  CLAUDE.md.

ABERTOS NOMEADOS (herdados do ciclo 19, doc 128 §7): a dobra da fita acima de 32 MiB · uma linha de
COR inteira num gesto · vida e dano pedem dois objectos · `Bezier`/`Tint` OKLCH caros na CPU de
reserva · gate de gesto pelo caminho REAL do rato (BUGS #18).
═══════════════════════════════════════════════════════════════════
