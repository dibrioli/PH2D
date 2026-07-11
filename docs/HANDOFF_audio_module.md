# HANDOFF — Módulo de Áudio (`line/audio`)

> Para o próximo agente. Contém: (1) proposta/plano do módulo, (2) arquitetura,
> (3) a linha `line/audio` e a integração ao `main` (Modo L), (4) o histórico do
> BUG "meter vivo, sem som" (**RESOLVIDO**). Leia o §0 + o §1 antes de mexer.

---

## 0. TL;DR do estado atual (2026-07-10)

**A linha está verde**, mas **à frente do `main` integrado** (`54fc9ecf`) com commits
locais **não integrados**: `bdacc038` presets · `52b2df34` modulação · `ef0bcaab` fix
slider "0" · **W6 Bloco 1 loop points** (este). O último ponto de integração adicionou
`split audio.rs` (HR-18), fix `fmt` drift `style_edition 2024` e AVIF no CI. **Integração
e ship são decisão exclusiva do Enio** — a linha fecha, entrega este handoff e para.

**O bug "meter vivo, sem som" está RESOLVIDO** e **não era código**: o
WirePlumber guardava `mute:true` por-app em `stream-properties`. Detalhe +
receita em [`docs/Audio/BUGS_audio.md`](Audio/BUGS_audio.md) (Bug #1). O §4
abaixo é histórico — não re-investigue.

**O que já landou:**
- **W1** — `ph2d-audio-encode` (WAV writer) + `ph2d-audio-edit` (`PeakCache`,
  `EditClip`) + painel docado `ph2d-panel-audio-editor` + **overlay flutuante do
  waveform** (drag/resize) + **preview transport** na `AudioEngine` (voz
  dedicada `PREVIEW_VOICE_ID`, fora do pool do jogo).
- **W2** — ops offline (gain · normalize peak/LUFS · reverse · DC · invert ·
  trim · cut · silence · fade · zero-crossing) + **undo/redo timeline** +
  **seleção por arrasto** na waveform + **edição ao vivo** (hot-swap do buffer no
  preview sem parar o play, com loop).
- **W3 Bloco 1** — **rack de efeitos offline** (`fx.rs`): `Effect::{LowPass,
  HighPass, Compress, Saturate, Bitcrush, StereoWidth}`, reusando o kit
  `ph2d_audio::dsp` (Biquad, Compressor).
- **W3 Bloco 2** — **efeitos com cauda** (`TailEffect::{Reverb, Delay}`, reusando
  Freeverb + Delay do kit). Splice novo `ops::in_range_tail`: dentro do alvo o
  efeito **substitui**; a cauda **soma** por cima do que vem depois (áudio
  original se a seleção é no meio) e o **clipe cresce** se o alvo toca o fim.
  ⚠️ `tail_secs` precisa vencer a latência do próprio efeito — o comb mais curto
  do Freeverb é ~25 ms, então cauda curta rende **silêncio** (preset usa 2,5 s;
  há um teste fixando esse modo de falha).
- **W3 Bloco 3a** — **rack paramétrico**: um seletor (`◀ nome ▶`) + até 4 sliders
  + **Apply**, no lugar dos 8 botões de preset fixo (os defaults *são* os presets).
  **Divisão de responsabilidade:** o painel continua **UI-only** — guarda só um
  índice de efeito e sliders **normalizados 0..1**; o shell
  (`shells/desktop/src/audio/fx_params.rs`) dona as faixas reais, o mapeamento
  (**log** p/ Hz e tempo), a formatação (`3.0 kHz`, `250 ms`) e a construção do
  `Effect`/`TailEffect`. Compressor ganhou **auto-makeup**. O `paint` re-semeia os
  sliders com o preset **só quando o efeito muda** (mesmo guard da caixa de nome),
  pra não brigar com o arraste do usuário.
- **W3 Bloco 3a-bis — audição ao vivo (não-destrutiva)**: mexer no seletor ou num
  slider marca o rack como *dirty*; o shell renderiza o efeito sobre o clipe
  **pristino** e faz hot-swap no preview voice → **você ouve enquanto ajusta**, e
  a waveform do overlay mostra o resultado (cauda inclusa). **Apply** commita
  **exatamente o buffer que soou** (1 passo de undo); **Cancel** descarta. O clipe
  só muda no Apply. Invariantes:
  - `EditClip::render_effect`/`render_tail_effect` (puros) + `commit_rendered`
    garantem *what-you-hear-is-what-you-commit* (teste
    `rendered_audition_is_what_gets_committed`).
  - `fx_dirty` é setado **só por input real** (`cycle_fx_kind`, `set_fx_norm`) —
    o re-seed dos defaults usa `seed_fx_norm`, senão abrir o painel já começaria a
    auditar um efeito que ninguém pediu (teste no seam).
  - O change-gate `FxSig` inclui **a seleção**: mover a seleção re-alveja o efeito
    e re-renderiza. Qualquer OUTRA edição (gain/trim/undo…) cancela a audição
    antes, senão o clipe se moveria debaixo dela.
  - **Custo:** um render completo do target range por *mudança de parâmetro*
    (change-gated ⇒ no máximo 1 por frame). Selecionar um trecho escopa o render —
    é o que mantém clipes longos responsivos. Se um dia pesar (reverb com cauda
    longa em música de 3 min), medir antes de mexer.
- **W3 Bloco 3a-ter — CADEIA ao vivo (efeitos acumulam)**: a audição deixou de
  renderizar sempre do clipe pristino. Agora é `clipe → chain[0] → … → ativo`.
  Ao **trocar de efeito**, o estágio que o usuário **ajustou** é empilhado na
  `fx_chain` e o próximo audita **por cima** dele; um estágio só *navegado* pelas
  setas é descartado (senão passar pela lista empilharia presets). **Apply**
  commita a pilha inteira em **1 passo de undo**; **Cancel** joga tudo fora.
  - O flag `fx_touched` (painel) é setado só pelo arraste de slider; o **shell**
    o consome (empilha) e limpa — nunca o painel. Seam test prende isso.
  - `fx_base` = `clipe + chain`, **em cache**: arrastar o slider do efeito ativo
    re-renderiza **um** estágio, não a cadeia toda. Recomputa quando a cadeia ou o
    target range muda.
  - Ao trocar de efeito, o shell usa `default_norms(novo_kind)` **na hora** — o
    painel só re-semeia os sliders no paint seguinte, e sem isso o frame da troca
    auditaria o efeito novo com os parâmetros do antigo (glitch audível).
- **Defaults = ponto NEUTRO (Enio, 2026-07-09)**: selecionar um efeito (ou passar
  por ele com as setas) deixa o áudio **byte-idêntico** até o usuário girar algo.
  Não basta "quase": um filtro no topo da faixa ainda desloca fase, um compressor
  1:1 ainda arredonda, e uma reverb seca anexaria uma cauda de silêncio. Por isso
  cada efeito tem um **bypass explícito** (`Effect::is_bypass` /
  `TailEffect::is_bypass`), e `tail_frames()` devolve **0** quando `Mix = 0` (é o
  que impede a reverb seca de crescer o clipe).
  - Neutros: LowPass cutoff = **max** · HighPass cutoff = **min** · Compress
    ratio = **1:1** (auto-makeup colapsa pra 1) · Saturate drive = **0** (linear,
    não log, pra o zero ser alcançável) · Bitcrush **16 bits / 1×** · Widen
    width = **1.0** · Reverb/Echo **Mix = 0**.
  - ⚠️ `norm_to_real` devolve os **extremos exatos** (`n<=0 → min`, `n>=1 → max`):
    `exp(ln(20_000))` voltava `19_999.998`, o suficiente pra furar o bypass e fazer
    "efeito nenhum" filtrar o áudio. Foi o primeiro teste a falhar.
  - **Reset por-efeito**: ícone `IconId::Reset` (estilo `Plain`) ao lado do nome,
    devolve o efeito ATIVO ao neutro. Também *destoca* o estágio (`fx_touched`),
    senão trocar de efeito empilharia um estágio já neutro. Fica **dimmed** quando
    os sliders já estão no neutro (`fx_at_defaults`). Não mexe em `fx_dirty`: se há
    audição, o shell re-renderiza neutro (volta pra cadeia / clipe pristino).
  - Testes: `every_effect_is_a_no_op_at_its_defaults` (byte-idêntico + comprimento)
    e `a_nudge_off_neutral_changes_the_audio` (o bypass não engole edição real);
    `is_bypass_implies_exact_identity` na crate. Apply com o rack intocado **não**
    empilha um passo de undo no-op.
- **Compressor: make-up preservador de pico (Enio, 2026-07-09)** — subir o `ratio`
  **aumentava a amplitude** e caminhava pro clipping. Causa: o auto-makeup de
  livro-texto `(1/threshold)^(1−1/ratio)` é o ganho que traz um sinal de **fundo de
  escala** de volta a 0 dBFS, e cresce com o ratio; aplicado a material mais quieto
  ele só amplifica (threshold 0.3, pico 0.5: ratio 4 → 0.86; ratio 20 → 0.97).
  Agora o make-up é medido do próprio buffer: `peak_in / peak_out` (clampado em 8×).
  O gain computer só atenua, então o pico **nunca sobe** — subir o ratio reduz a
  faixa dinâmica, não a amplitude. O campo `makeup` saiu de `Effect::Compress`.
  - ⚠️ Foi preciso `Compressor::prime()` (novo, append-only em `ph2d-audio`): o
    `gain` começava em 1.0, então a **primeira amostra escapava sem compressão** —
    num trecho que já começa no pico isso dava `peak_out == peak_in`, o make-up não
    tinha nada a devolver, e ainda clicava na borda da seleção. Offline não há razão
    pra esse transiente de partida.
  - Teste `higher_ratio_lifts_rms_without_raising_the_peak`: pico nunca sobe (varre
    ratios × attacks) e, com attack rápido, ratio maior levanta o RMS.
- **Auditoria multiagêntica (2026-07-09)** — 6 lentes + verificação adversarial (3 céticos
  por achado). 8 achados, 5 confirmados, 3 refutados. Corrigidos:
  1. **Clique na borda da seleção ao filtrar** (4 lentes convergiram nele): `in_range` entrega
     ao op uma região ISOLADA, então o biquad começa com memória zerada — como se antes da
     seleção houvesse silêncio. A 1ª amostra filtrada colapsa pra ~`b0·x` enquanto a vizinha
     intocada está em nível cheio → degrau de escala cheia. Fix: splice novo **`in_range_warm`**
     (pre-roll do áudio ANTERIOR ao alvo, descartado depois) + `Effect::warmup_frames`
     dimensionado por `8·τ`, `τ ≈ Q/(π·f0)`, capado em 1 s. Sem seleção não há o que pré-rolar
     → byte-idêntico ao `in_range`.
  2. **Botão dimmed ainda despachava**: `button()` registrava o hit-rect incondicionalmente, então
     clicar no `Silence` "desabilitado" (sem seleção) caía em `target()` → **zerava o clipe
     inteiro**. Fix em 2 camadas: dimmed não registra hit **e** `event.rs` recusa armar range-ops
     sem seleção (essa metade é testável no seam).
  3. **Export durante audição** exportava o clipe pristino, não o que soa/aparece (waveform e
     duração já mostravam a audição). Fix: `editor_export` usa `editor_sounding()`.
  - Refutados (registrados de propósito): `normalize_lufs` sem clamp de pico **não é bug**
    (loudness-normalization legitimamente passa de ±1; use limiter depois); `equal_power_pan`
    com `cos/sin` no callback RT **não é bug** (roda 1× por comando e cacheia em `pan_gains`;
    `pan.rs:10` anota "HR-5 exempt"); `remove_dc` na seleção usar a média local é o
    comportamento documentado do `in_range`.
- **W3 Bloco 3b — cadeia VISÍVEL e EDITÁVEL (2026-07-09)**: a `fx_chain` deixou de ser
  um acumulador invisível do shell e virou o **modelo, dono do painel**. O rack agora é
  um rack de DAW: uma lista de estágios, **um selecionado** (é ele que o seletor e os
  sliders editam), com `+` (Add) · `🗑` (Remove) · `▲▼` (reordenar) no cabeçalho "Chain",
  **olho por-estágio** (bypass in-place, sem perder o ajuste) e um **Bypass global**
  (A/B: ouve/vê/exporta o clipe seco, cadeia intacta).
  - **Isso MATOU o `fx_touched`/`fx_last`/empilhamento implícito.** Antes, trocar de
    efeito empilhava o estágio "ajustado" e descartava o "só navegado" — regra sutil
    que ninguém conseguia inspecionar. Agora Add cria estágio, Remove tira, e ponto.
  - **Ownership:** painel dona `FX_CHAIN: Vec<FxStage{kind,norms,enabled}>` + `FX_SEL` +
    `FX_BYPASS`; shell publica a **tabela de kinds** (nomes + normals NEUTROS de cada
    kind) e renderiza. Isso mantém o painel UI-only: ele semeia um estágio novo
    transparente sem conhecer nenhuma faixa de DSP. ⚠️ O shell publica a tabela **antes**
    de ler `fx_chain()` — `ensure_chain()` materializa o 1º estágio a partir dela (sem a
    tabela, um estágio nasceria com normals 0.0 = Low-Pass em 20 Hz, audível).
  - **Cache de prefixo (`fx_head`)**: o clipe com `chain[..sel]` já renderizado. Arrastar
    um slider re-renderiza só `chain[sel..]` (normalmente 1 estágio). Teste
    `rendering_from_a_cached_head_matches_a_full_render` prende **byte-a-byte** o atalho
    contra o render completo, **com um efeito de cauda no meio** (onde um cache ingênuo
    sai de passo). Um drift ali = audição soa uma coisa, Apply commita outra.
  - **Apply fica DIMMED sob Bypass**: o que soa é o seco, então commitar não landaria
    nada. `editor_fx_apply` commita `editor_sounding()` (não a audição), então mesmo se o
    dim falhar o caminho concorda com ele — a invariante "what sounds is what commits"
    vale pro Play, waveform, Export e Apply pelo mesmo acessador.
  - Após Apply/Cancel/Load, `reset_fx_chain()` deixa **um estágio neutro**: re-renderizar
    uma cadeia já assada no clipe dobraria cada efeito.
  - **Ordem importa** (filtro antes da reverb ≠ depois) — `chain_order_changes_the_result`
    prende isso; é a razão de existirem ▲▼.
  - ⚠️ **Gotcha achado pelo teste:** `format().channels` é o enum `ChannelLayout`, não um
    número — `as usize` devolve o *discriminante* (Stereo = 1) e **compila**. Use
    `format().channel_count()`. Custou um falso "o áudio depois da seleção mudou".
  - `MAX_FX_STAGES = 6` (a lista cabe no painel docado, que **não rola**). Add para no
    teto; Remove é recusado no último estágio (o rack sempre tem o que editar) — dim
    **e** recusa no `event.rs`, as duas camadas.
  - Arquivos: `paint_fx.rs` (seletor/params/chain/commit) · `snapshot.rs` (estado da
    cadeia) · **`shells/desktop/src/audio/editor/fx_rack.rs`** (NOVO submódulo — o
    `editor.rs` ia estourar o teto HR-18; descendente de `editor`, então enxerga os
    campos privados de `AudioEditorRuntime`).
- **Scroll do painel (2026-07-09)** — o painel docado agora **rola** (roda do mouse +
  barra arrastável), porque a rack estourou a altura do dock. Nada de infra nova: o
  repo já tinha tudo (`store.panel_scroll`/`panel_content_h`/`panel_visible_h` +
  `dispatch_wheel` + `widget/scrollbar.rs` + `VectorScene::push_clip`); o painel só
  ignorava o offset. Espelha o `ph2d-panel-audio-mixer`.
  - ⚠️ **São QUATRO sites, e só três falham alto.** (1) id do thumb em
    `widget/scrollbar.rs`; (2) arm em `scrollbar_panel_for_id`
    (`interaction/dispatch/scroll.rs`) → roteia o DRAG; (3) o painter lê
    `panel_scroll` + publica `content_h`/`visible_h`; (4) o id em
    `cursor_over_hero_panel` (`shells/desktop/src/forwarding.rs`) → deixa a RODA
    chegar. **Esquecer (4) compila, pinta a barra e o thumb arrasta — mas a roda dá
    zoom na câmera.** O **Audio Mixer estava exatamente assim** e ninguém tinha
    notado; corrigido junto.
  - Gate nova: `shells/desktop/tests/scrollable_panels_intercept_the_wheel.rs` —
    lê os ids de `scrollbar_panel_for_id` e exige cada um dentro de um `inside(...)`
    de `cursor_over_hero_panel`. **Mutation-testada** (apagar `|| inside(AUDIO_MIXER_PANEL)`
    faz falhar). A 1ª versão escaneava a função inteira e passava mesmo assim — o
    `use ...::ids::{...}` no topo do corpo já continha o nome. Allowlist com 1 entrada:
    `PAINTER_BRUSH_STUDIO_PANEL` (crate deletada por ADR-0099; id vestigial).
  - **`ClippedHits`** (`clipped_hits.rs`, novo): o `HitIndex` é uma lista plana e
    global — o clip é só VISUAL. Sem isso, um widget rolado pra cima da barra de
    título continua **clicável e invisível**. Todo widget do corpo registra por ele;
    o thumb NÃO (mora na calha, fora do clip). 5 testes.
  - **Foundational tocado (anotar na integração):** `AUDIO_EDITOR_SCROLLBAR_ID =
    NodeId(834)` (append-only; 831 = dropdown, 832 = mixer, 833 = vector — próximo
    livre é **835**) + re-export em `widget/mod.rs` + arm em `scrollbar_panel_for_id`
    + teste `audio_editor_panel_scrollbar_thumb_drag_begins_and_scrolls` em
    `dispatch/tests/inputs.rs` + `AUDIO_MIXER_PANEL`/`AUDIO_EDITOR_PANEL` em
    `cursor_over_hero_panel`.
  - `paint.rs` está em **588/600 LOC** e `paint()` em ~150/200 (foi preciso extrair
    `paint_transport_section` / `sync_widget_buffers` / `paint_scroll_chrome` p/ o
    gate de fn). Próxima seção grande = split de arquivo, não allowlist.
- **W3 Bloco 4 — DSP essencial: EQ paramétrico + limiter true-peak (2026-07-09)**.
  A rack foi de 8 para **12 efeitos**: `Peak EQ` · `Low Shelf` · `High Shelf` ·
  `Limiter`. Ordem nova (tone → dynamics → character → space).
  - **EQ = zero DSP novo.** `ph2d_audio::dsp::BiquadCoeffs` **já tinha**
    `peak`/`lowshelf`/`highshelf` (RBJ cookbook), testados. Nenhuma mudança
    foundational. Neutro dos três = `gain_db == 0` (bypass explícito: um RBJ de 0 dB
    é identidade *algébrica*, mas ainda arredonda cada amostra).
  - **`truepeak.rs` (NOVO, pub `true_peak`)** — sobreamostragem 4× (interpolador
    windowed-sinc de fase fracionária, 12 taps/fase, gerado aqui; **não** é a tabela
    literal do BS.1770). Por quê: um seno em `fs/4` amostrado a 45° cai em `±0.707`
    em TODA amostra enquanto a onda contínua vai a `±1.0` — 3 dB de pico invisível
    pro medidor `max|x[n]|`. Invariantes gateadas: **fase 0 é impulso exato** (o
    true-peak nunca lê *abaixo* do sample-peak) e **cada fase tem ganho DC unitário**
    (senão um sinal constante interpola em ripple e inventa pico).
  - **`Effect::Limiter { ceiling_db, release_secs }`** — look-ahead, ganho
    **stereo-linked**, nunca amplifica. Neutro em `ceiling_db >= 0` (um teto em
    0 dBFS não tem o que pegar num buffer `[-1,1]`; a convenção é **−1 dBTP**, um
    passo abaixo do neutro).
    - **Por que não estoura**: `g[n] = min(1, ceiling/tp[n])` → **mínimo deslizante**
      em `±R` → duas médias-caixa com suporte somado `±R`. Após o mínimo,
      `g_min[n+k] <= g[n]` para todo `|k|<=R` (pois `n` está na janela que produziu
      `g_min[n+k]`), e o suavizador é uma média ponderada exatamente sobre esses `k`
      com pesos somando 1. Logo `g_smooth[n] <= g[n]` **em todo n**. O mínimo é o
      look-ahead; a suavização tira o clique; **nenhum desfaz o outro** — é por isso
      que os dois usam o MESMO raio.
    - `sliding_min` = deque monotônica O(n); `box_average` = prefix-sum em **f64**
      (um acumulador f32 deriva em milhões de frames).
    - `warmup_frames` do Limiter = a janela inteira de look-ahead (capada em 1 s),
      senão `in_range_warm` entrega uma região cuja curva de ganho começa em 1.0 e o
      primeiro pico da SELEÇÃO escapa. Reuso direto do splice da auditoria.
    - **Mutation-testado**: trocar `sliding_min(&need, radius)` por `need.clone()`
      derruba os 2 testes (true peak vaza pra 1.028 contra teto 0.891).
  - **`fx.rs` estourou o teto de 700 LOC** → dividido em `fx/{tone,dynamics,space}.rs`
    (o enum e os pontos neutros ficam no `fx.rs`). `ops::channels` virou `pub(crate)`.
  - **`fx_params.rs` virou tabela única** `KINDS: [FxKind; 12]` com **nome + specs +
    construtor na MESMA linha**. Antes eram três `match kind {}` paralelos: inserir um
    efeito no meio re-indexava um deles em silêncio — a rack nomearia "Compress",
    mostraria os sliders dele e construiria um Saturate. Agora é impossível.
  - **Custo**: o envelope true-peak é ~3 fases × 12 taps × canais por frame. Clipe de
    minutos numa audição ao vivo pesa; medir antes de otimizar (o render é
    change-gated e a seleção escopa o alvo).
- **W3 — Modulação: Chorus · Flanger · Phaser · Tremolo (2026-07-09)**. Rack: 14 →
  **18 efeitos** (grupo novo no fim, depois de Reverb/Echo). Todos length-preserving
  (splice `in_range_warm`), LFO senoidal, **zero dependência nova**, e **nada de
  painel/snapshot** — cada um é uma linha em `KINDS` que a maquinaria de rack já toca.
  - **Chorus/Flanger** = uma **linha de delay modulada** compartilhada
    (`modulated_delay`): a leitura passeia entre `base` e `base+depth` ms no rate do
    LFO; `feedback` recircula (pente do flanger); `mix` faz dry→wet. Chorus = base
    18 ms sem feedback; Flanger = base 2 ms com feedback. Leitura **fracionária**
    (interpolação linear — senão o tap modulado zippa). Canais L/R defasados 90° (sem
    isso o "stereo chorus" colapsa em mono — teste `stereo_channels_modulate_out_of_phase`).
  - **Phaser** = cascata de 4 all-pass de 1ª ordem com `fc` varrido pelo LFO
    (200 Hz→2 kHz log, escala por `depth`), somada ao dry → notches móveis.
  - **Tremolo** = modulação de amplitude, ganho `1..1-depth`; **em fase** nos dois
    canais (fora-de-fase seria auto-pan, outro efeito). Depth 0 = `×1.0` = identidade.
  - **Neutro/arm:** Chorus/Flanger/Phaser neutros em **Mix 0** (dry byte-idêntico via
    `_ => clone`); Tremolo em **Depth 0**. Os testes do shell (`turning_an_arming_knob`,
    `the_other_knobs_do_nothing`, `every_effect_is_a_no_op`) iteram os 18 e já cobrem.
  - **Warmup:** Chorus/Flanger pré-enchem a linha (`base+depth` ms) senão os primeiros
    ms do wet numa seleção do meio são silêncio (o efeito "sobe"). Phaser assenta em
    poucas amostras, Tremolo é memoryless → warmup 0.
  - **`fx.rs` estava em 656/700** → movi os ~300 LOC de teste pra **`fx/tests.rs`**
    (`mod tests;` resolve pra `src/fx/tests.rs`, sem `#[path]`; é filho de `fx` então
    enxerga os privados). fx.rs caiu pra 439 — folga durável pro roster crescer. DSP
    novo em `fx/modulation.rs`. Os arrays `tuned_effects`/`neutral_effects` foram 12→16.
  - **Fix de display (Enio): slider mostrava "0" onde o valor não era 0.** O `{:.0}`
    do `format_value` arredondava sub-1 pra zero — o Rate do LFO (mín 0.05 Hz, uma
    varredura LENTA, não parada) lia "0 Hz" em ~40% do curso, e o Gate Attack (0.5 ms)
    lia "0 ms". **Não é bug de slider** — o thumb já começa no mínimo real; era só o
    rótulo. Fix: sub-1 Hz → 2 casas ("0.05 Hz"), sub-1 ms → 1 casa ("0.5 ms"). Gate
    guard `no_slider_reads_a_false_zero` varre todo param×curso e prende (mutation-testado).
- **W3 — Presets de cadeia (2026-07-09)**. Salvar/carregar combinações de efeitos.
  - **Duas espécies, dois mecanismos.** **Factory presets** (curados, 7: Voice Cleanup ·
    Podcast · Telephone · Lo-Fi · Master Bus · Wide & Bright · Gate + Glue) são
    navegados no painel por um seletor `◀ nome ▶` + **Apply**. **User presets** são
    ARQUIVOS: **Save/Load** abrem `rfd::FileDialog` (mesmo padrão do Load/Export do WAV).
  - **Chaveado por NOME, nunca por índice.** A cadeia é `Vec<FxStage{kind,norms,enabled}>`
    e `kind` é índice em `KINDS` — que os blocos 4 e 5 já reordenaram. Um preset que
    guardasse índices viraria outro efeito. Factory specifica em **unidades reais por
    label** (`ovr("Cutoff", 90.0)`) e resolve por nome+label; o formato de arquivo é
    `Effect Name | on|off | n0 n1 …` (norms verbatim → round-trip exato).
    `parse_chain` **pula** linha/nome desconhecido (um preset velho carrega o que dá).
  - **Arquitetura:** `shells/desktop/src/audio/fx_presets.rs` (NOVO) dona a tabela +
    serialização (só o shell tem o mapa nome↔kind). O painel ganhou `set_fx_chain`
    (substitui a cadeia inteira, marca dirty → audita na hora, Apply commita) + o
    módulo irmão `presets.rs` (estado do seletor + 3 intents one-shot). O bridge publica
    os nomes e drena Apply/Save/Load. **Reusa 100% do fluxo de audição** — carregar um
    preset é como girar um slider.
  - **Testes:** `every_factory_stage_resolves` (todo nome+label existe — um typo dropava
    o estágio em silêncio), `factory_presets_are_audible` (nenhum resolve pra cadeia
    toda-neutra = no-op mudo), `user_preset_round_trips_including_disabled_stages`,
    `parse_is_keyed_by_name_not_index`, `parse_skips_junk_and_unknown_effects`. +2 no
    seam (seletor cicla / Apply arma sem tocar a cadeia; Save/Load armam os file-intents).
  - ⚠️ **`snapshot.rs` está CRAVADO em 600/600 LOC** (o `set_fx_chain` encostou no teto).
    Passa, mas o **próximo campo transborda** — a extração natural é mover o estado da
    FX-chain (`FX_CHAIN` + add/remove/move/select/toggle) pra um irmão `fx_chain.rs`.
    Vide [[project_painter_core_files_at_loc_cap]] e [[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]].
- **W3 Bloco 5 — Gate/Expander + De-Esser (2026-07-09)**. Rack: 12 → **14 efeitos**.
  Ordem final: `Low-Pass · High-Pass · Peak EQ · Low Shelf · High Shelf · Compress ·
  Gate · De-Esser · Limiter · Saturate · Bitcrush · Widen · Reverb · Echo` (pinada
  inteira pelo teste `the_kind_table_is_the_rack_layout`).
  - **`Effect::Gate`** = expansor descendente: acima do threshold passa; abaixo o ganho
    é `(level/threshold)^(ratio−1)`. **Um knob** vai de expansor suave (2:1) a gate duro
    (16:1) — sem modo. Neutro em `ratio == 1` (`^0` = 1 em todo nível). Floor −80 dB (um
    zero absoluto lê como dropout). ⚠️ **Attack ABRE e Release FECHA** — o oposto do
    compressor. Trocar os dois faz o gate *tremular* em cada passagem quieta; há teste
    (`the_gate_release_governs_how_slowly_it_closes`).
  - **`Effect::DeEss`** = **high-shelf dinâmico**, NÃO split-band. O desenho óbvio
    (`high = highpass(x)`, `low = x − high`, `out = low + g·high`) **está errado**:
    `x − highpass(x)` não é um lowpass. Um HP de 2ª ordem em 5 kHz passa 9 kHz com
    |H|≈0.96 *e fase*, então o complemento carrega uma cópia girada da banda alta —
    baixar `g` não remove a banda, faz um **comb**. Medido: energia da banda alta
    890 → **435** (ratio 2) → 439 (ratio 6) → **444** (ratio 12). *Apertar o de-esser
    deixava o "S" mais alto.* O shelf ataca a magnitude direto: cortar mais é sempre
    cortar mais. Teste `a_heavier_ratio_always_ducks_harder` prende a monotonicidade.
    - Coeficientes só recalculam quando o ganho move > 0.05 dB (`sin_cos`+`powf` custa
      ~30 amostras filtradas); `Biquad::set_coeffs` preserva o estado → sem clique.
    - Sidechain = HP da banda, stereo-linked, attack fixo em 1 ms (sibilância é
      transiente); só `Release` é exposto. `warmup_frames` cobre o biquad do detector.
  - **`arms: &'static [usize]` em `FxKind`** — os índices dos knobs que o `is_bypass`
    observa. Ninguém lê em runtime; existe pra **dois testes segurarem o `is_bypass`
    por fora**: `turning_an_arming_knob_wakes_the_effect_up` (mexer o arm acorda) e
    `the_other_knobs_do_nothing_while_the_effect_is_neutral` (mexer o resto NÃO acorda).
    Bitcrush tem **dois** arms (Bits ou Downsample). Mutation-testado: declarar 1 arm
    pro Bitcrush derruba a 2ª gate.
  - ⚠️ O `probe()` dos testes de `fx_params` usava `sin(hz * t)` — sem `2π`, então
    "220 Hz" eram 35 Hz de radianos no buffer inteiro e o sinal nunca saía de perto do
    seu offset DC. Um gate não tinha o que fechar. Corrigido pra `sin(TAU·hz·t)`.
  - **`fx_params.rs` estourou o cap de 600 do shell (HR-18)** → tabela extraída pra
    `audio/fx_params_table.rs` (447 + 262). O cap do shell é **600**, não os 700 do
    workspace — foi `shells/desktop/tests/file_loc_caps.rs` que pegou.
- **W6 Bloco 1 — Loop points + `smpl` chunk + audição click-free (2026-07-10)**.
  Primeiro bloco do **asset-prep de games**: uma **região de loop** no clipe (metadado,
  NÃO edição de undo), snap a zero-crossing, **audição contínua sem clique**, e o loop
  escrito no **`smpl` chunk** do WAV (sobrevive re-decode). Fim-a-fim: DSP → encode →
  painel → shell → overlay.
  - **DSP (`ph2d-audio-edit/src/loops.rs`)** — `crossfaded_loop(data, region, xfade)`:
    o **pre-loop crossfade** clássico. Um loop cru `[s,e)` salta `data[e-1]→data[s]` na
    volta; a cauda é fundida (equal-power sin/cos) nos `L` frames que **precedem** `s`,
    então o último frame pousa em `data[s-1]` e a volta vira o passo contínuo `s-1→s` do
    próprio sinal. `L = min(xfade, s, region_len)` → loop começando no frame 0 (sem
    lead-in) degrada pra cópia crua (o auto-snap do Set ajuda alinhando os pontos).
    Teste prova que a costura cai 10× vs. o loop cru.
  - **`EditClip`** ganhou `loop_region: Option<Range>` (metadado, mesma disciplina da
    `selection`: **sobrevive undo/redo**, só clampa quando um edit encurta o clipe, some
    no load) + `set_loop_from_selection`/`snap_loop_to_zero_crossing`/`clear_loop`/
    `loop_audition_buffer(xfade)`. **Não** entra na timeline de undo (não é sample data).
  - **Encode (`ph2d-audio-encode`)** — `WavMeta { loops: Vec<LoopRegion> }` +
    `encode_wav_with_meta`/`write_wav_with_meta` + `read_loop_regions` (walker RIFF
    próprio — o Symphonia ignora `smpl`). O chunk fica **ANTES do `data`** (leitor que
    para no `data` ainda o vê). `LoopRegion.end` é **half-open**; o `smpl` guarda o
    último frame **inclusivo** (`end-1`) — conversão só na fronteira. `encode_wav` sem
    loops é **byte-idêntico** ao arquivo antigo (teste de regressão).
  - **Áudio click-free reusa TODO o preview existente**: o loop toca o buffer
    crossfadeado em `set_preview_looping(true)` — **zero mudança na RT thread**.
    `replace_data` clampa o cursor, então hot-swap de buffer menor é seguro.
  - **UI unificada (revisão pós-smoke do Enio, 2026-07-10):** o design inicial tinha um
    botão **Audition** próprio + um **Snap Zero** — confundia com o toggle **Loop** que
    já existia, e um bug fazia o **Stop não funcionar** (a ponte relia `loop_audition()`
    a cada frame e re-disparava a audição logo após o Stop limpá-la). **Fix + simplificação:**
    - **Audition REMOVIDO** — o loop toca por **Loop (toggle) + Play**: `editor_toggle_play`
      vê `looping && has_loop` → toca a região crossfadeada em repeat; senão o clipe
      inteiro. Play é one-shot (não persiste), então **Stop/Pause funcionam** sem nada
      pra re-disparar. `editor_set_looping` fica guardado por `playing_loop_region` (o
      toggle Loop não desliga o loop por baixo).
    - **Snap REMOVIDO como botão** — dobrado no **Set Loop** (auto-snap a zero-crossing);
      move os pontos < 1 ms (invisível), só previne o clique. `editor_loop_live_update`
      faz hot-swap enquanto a região toca (o slider Crossfade atualiza ao vivo) e
      **nunca inicia** playback (por isso não briga com o Stop).
    - **Playhead corrigido:** a região toca como buffer próprio (frames 0..len), então
      `preview_frame` mapeado no clipe inteiro punha a linha no canto esquerdo, FORA dos
      brackets. `editor_playhead_frame` soma `loop_start` → a linha fica **dentro** do loop.
    - **Scrubbing:** arrastar a **régua** (strip de tempo) move o playhead
      (`editor_scrub_to_frame` → `seek_preview`, mapeado à região quando em loop). A régua
      é o novo hit-region publicado em `WaveView.ruler`; o corpo da waveform continua
      fazendo seleção. Estado `audio_scrub_drag` no `App`.
    - **Waveform "pro":** barras arredondadas espelhadas no centro (`BAR_PITCH`/`BAR_W`)
      + **shading played/unplayed** (Accent até o playhead, Text2 depois) — em vez do
      envelope min/max chapado. Silêncio vira um traço fino (piso 2 px).
  - **Painel** (`ph2d-panel-audio-editor`) — seção **Loop** (`paint_loop.rs`, sob a
    transport): readout `1.20–3.40s`/`No loop` · **Set Loop** (da seleção, auto-snap) ·
    **Clear** · slider **Crossfade**. Estado em `loop_state.rs` (thread-local): 1
    persistente (xfade norm) + 2 intents one-shot + `set_loop_span` shell→painel.
    Guardas: Set exige seleção; Clear exige loop (dim + recusa no seam).
  - **Overlay** — `draw_loop_region` desenha um **frame verde** (`ColorToken::Success`,
    distinto da banda azul de seleção) marcando o loop.
  - **Export** carrega o loop do clipe committed pro `WavMeta`, clampado ao buffer
    exportado.
  - ⚠️ **LOC dance (HR-18/panel gate):** o `cargo fmt` (style_edition 2024) re-expandiu
    a chamada nova e estourou `paint.rs` pra 603 → extraí `paint_edit_section` pra
    `paint_edit.rs` (paint.rs 493). E `apply_event` passou de 200 LOC → extraí
    `loop_click`. **Meça DEPOIS do fmt** (a memória avisa: fmt re-expande multi-arg).
  - **Ready-to-smoke:** `PH2D_AUDIO_LOOP_SMOKE=1` (em `main.rs` + `editor_loop_smoke`)
    põe um tom 220 Hz de 2 s no editor com um loop no terço do meio **não-snapado**
    (endpoints mid-phase = clicaria cru) → abrir o pill Audio Editor mostra os brackets
    verdes; ligar **Loop** + **Play** com Crossfade em 0 vs. default = o A/B click ↔
    click-free. Arrastar a régua move o playhead.
  - **Aberto (resto do W6):** containers de variação · markers/cue (`cue`/`LIST adtl`) ·
    codec/residência + export OGG/Opus · import por convenção.
- **W6 Bloco 3 — Force-to-mono + Batch LUFS (2026-07-10)**. Dois preparos de biblioteca
  de jogo, sem dep nova.
  - **Force Mono** (`ops::force_mono` re-exportado; downmix = média dos canais → mono,
    pra som 3D posicional que precisa ser mono pra panejar no espaço). **NÃO-destrutivo
    (revisão pós-feedback do Enio):** virou um **toggle de saída** igual ao Bypass do
    rack — o clipe fica estéreo (undo intacto) e o downmix é uma **view** (`mono_view`,
    cache invalidado por `(ptr,len)` da base) aplicada só em `editor_sounding` (Play /
    waveform / Export). Clicar de novo reverte instantâneo; live-switch do preview no
    toggle (`editor_toggle_force_mono` → refresh + hot-swap). Botão na seção Edit
    (`Invert | Force Mono`, **toggle** aceso via `mono_on`); reseta no Load. O overlay
    redesenha 1 lane sozinho (lê `channel_count` da view). O op destrutivo
    `EditClip::apply_force_mono` continua existindo (API + teste), só não é o que o
    botão usa.
  - **Batch LUFS** (`shells/desktop/src/audio/editor/batch.rs`): normaliza uma PASTA
    inteira a um alvo (−16 LUFS) escrevendo cópias em `<pasta>/normalized/` (PCM16).
    **Não-destrutivo** (originais intactos). Reusa decode + `normalize_lufs` (re-exportado
    de `ph2d-audio-edit`) + encode — zero dep nova. Botão **"Batch LUFS…"** na transport
    (sempre ativo — é op de pasta, independe do clipe carregado); a ponte abre
    `rfd::pick_folder`. Core testável = free fn `batch_lufs_dir` (o método
    `editor_batch_lufs` só delega), teste e2e cria tmpdir + tom quieto → confere cópia
    mais alta + `.txt` ignorado.
  - **Intent do batch** foi pra `loop_state.rs` (renomeado no doc pra "asset-prep panel
    state": loop + batch) — o `snapshot.rs` está em 600/600, então não dava pra add lá.
  - **Ready-to-smoke:** o `PH2D_AUDIO_LOOP_SMOKE=1` agora gera um tom **estéreo** (L 220 /
    R 223 Hz) → 2 lanes no overlay; **Force Mono** (toggle) colapsa em 1 e clicar de
    novo volta pra 2. Batch LUFS: clique o botão → escolha uma pasta com WAVs → cópias
    normalizadas em `normalized/`.
- **W6 Bloco 4 — Markers / cue points (2026-07-11)**. Pontos nomeados na timeline
  (transição/sync/sustain pro runtime do jogo), exportados nos chunks `cue`+`LIST/adtl`
  do WAV — **fecha o critério "smpl/cue sobrevivem re-decode"**. Sem dep nova.
  - **`EditClip`** ganhou `markers: Vec<Marker { frame, name }>` (metadado como o loop:
    sobrevive undo, clampa/dropa quando um edit encurta o clipe, some no load). Métodos
    `add_marker` (clamp + insert **ordenado por frame**, no-op se já há um no mesmo
    frame), `remove_marker_near(frame, window)` (o mais próximo dentro da janela),
    `clear_markers`. **`lib.rs` estava em 698/700** → extraí os testes pra
    `src/tests.rs` (`mod tests;`) antes de add o campo (senão estourava).
  - **Encode (`ph2d-audio-encode`)** — `WavMeta.markers` + chunks `cue ` (posições, id =
    índice) + `LIST/adtl` `labl` (nomes, casados por id) + `read_markers` (walker RIFF,
    junta posição↔nome, ordena por frame). Ficam ANTES do `data` (com o smpl); Symphonia
    ignora os dois. Teste prova cue+adtl round-trip, coexistência com smpl, e áudio
    decoda. `Marker` do encode = `{frame:u32, name}`; o shell converte.
  - **Painel** — seção **Markers** nova (`paint_markers_section` em `paint_loop.rs`, sob
    o Loop): readout `N markers`/`No markers` + **Add Marker** (no playhead) | **Delete**
    (o mais próximo). Estado em `loop_state.rs` (2 intents one-shot + `marker_count`).
  - **Shell** (`audio/editor/markers.rs`) — `editor_add_marker` (auto-nome `M{n}` no
    `editor_playhead_frame`), `editor_del_marker` (nearest, janela ~50 ms),
    `editor_markers`/`editor_marker_count`. Export carrega os markers pro `WavMeta`.
  - **Overlay** — `draw_markers`: linha fina **roxa** da régua ao fim da waveform + nome
    no topo (distinta do loop verde / seleção azul / playhead laranja).
  - **`apply_event` estourou 200 LOC** de novo → extraí `asset_click` (batch · mono ·
    markers), como o `loop_click`.
  - **Ready-to-smoke:** o `PH2D_AUDIO_LOOP_SMOKE=1` agora já vem com 2 markers (M1/M2) —
    flags roxas visíveis ao abrir o pill; Add põe um no playhead, Delete tira o mais
    perto, Export grava o `cue`+`adtl`.
  - **Aberto no W6 (à época):** ~~containers de variação~~ (Bloco 5, abaixo) · export OGG/Opus
    (dep + ADR) · import por convenção. Rename de marker (in-place TextInput) é follow-up.
- **W6 Bloco 5 — Variation containers (2026-07-11, `line/audio`, commit `ecd2587a` — NÃO
  integrado; handoff [`HANDOFF_audio_variation_impl.md`](HANDOFF_audio_variation_impl.md))**.
  Container estilo **Wwise Random/Sequence** / FMOD Multi-Instrument: um set de clipes que
  toca **um** por trigger (Random / Sequence / Shuffle-avoid-repeat) com jitter de pitch/gain
  por-play + **pesos** por-entry. **Autorado + auditado + salvo** no painel; o trigger em
  runtime segue bloqueado (§1), então a **audição É o consumidor vivo** e o **manifesto `.txt`**
  é o entregável (mesma forma dos presets de FX — NÃO virou entidade ECS / asset novo).
  - **Modelo puro** `ph2d-audio-edit/src/variation.rs`: `PickStrategy`/`VariationSet`/
    `VariationPicker` (splitmix64, pick ponderado, shuffle sem repetir seguido, jitter
    `2^(±st/12)`/`10^(±dB/20)` em `exp2`) + manifesto tolerante (`serialize`/`parse`, keyed
    by-content). Control-thread → HR-3/HR-5 não valem (aloca + transcendentais livres). 11 testes.
  - **Painel** (`variation_state.rs` + `paint_variation.rs`, UI-only): lista selecionável ·
    seletor `◀ estratégia ▶` · Add/Remove/Play · Weight ÷2/×2 · sliders Pitch/Gain jitter ·
    Save/Load. Estado thread-local (fora do `snapshot.rs`, que segue 600/600). `apply_event`
    ganhou `variation_click`; extraí `edit_cmd_for` p/ ficar sob 200 LOC/fn (fmt re-expandiu).
    5 testes de estado + **6 de seam**.
  - **Shell** (`audio/editor/variation.rs`): dona o `VariationSet` + cache de clipes decodados
    (index-aligned) + o picker; `editor_play_variation` toca o pick com jitter pela **preview
    voice** (one-shot transiente, não mexe no transporte). Ponte em `render_loop/mod.rs`
    (bloco aditivo após Markers). Novos campos em `AudioEditorRuntime`.
  - **Ready-to-smoke:** `PH2D_AUDIO_LOOP_SMOKE=1` semeia 4 blips (C-E-G-C) na seção Variations
    → **Play Variation** repetido cicla; troque a estratégia; suba o jitter; Weight ×2 enviesa.
  - **Aberto (variação):** enable-toggle por-entry na UI (modelo/manifesto já têm `enabled`) ·
    overlay não desenha o set (é set de arquivos, não timeline — proposital).
- **Atalhos:** `Ctrl+Z` undo · `Ctrl+Shift+Z` / `Ctrl+Y` redo (roteados ao
  `EditClip` quando o painel WAVE está aberto com clipe carregado).
- **Fix:** `cpal::Stream` é dropado no `on_close_request`, não no drop-cascade do
  `App` (segfault 139 de teardown).

**Invariante-chave do rack (duas famílias):** tudo age no **target range** (a
seleção, quando existe; senão o clipe inteiro), e a família decide o splice:
- **length-preserving** (`Effect`) → `apply_effect` → `ops::in_range`;
- **tail-extending** (`TailEffect`) → `apply_tail_effect` → `ops::in_range_tail`.

Efeito novo: escolha a família **antes** de escrever o DSP. Um efeito com cauda
enfiado no `in_range` tem a cauda **truncada** silenciosamente.

**Adicionar um efeito novo** (o caminho é curto agora): (1) a variante em
`fx.rs` (escolha a família!), (2) a tabela de params + o arm do `build` em
`audio/fx_params.rs`, (3) o nome em `FX_KINDS`. O painel **não muda** — ele lê
tudo do snapshot. `MAX_FX_PARAMS = 4`: um efeito com 5 params exige subir a
constante nos dois lados (painel + shell) e criar mais um id de slider.

**Próximo (plano vivo: [`docs/Audio/02_plano_implementacao_completo.md`](Audio/02_plano_implementacao_completo.md)):**
1. **W6 — resto do asset-prep** (loops + force-mono + batch LUFS + markers/cue já
   landaram): containers de variação (random/round-robin/avoid-repeat) · codec/residência
   + **export OGG/Opus** (1 dep nova → ADR + `deny`) · import por convenção. O essencial
   restante (variação) é DSP puro sem dep.
2. **Cluster FFT** (compartilha `realfft`/`rustfft`, 1 dep + 1 ADR): **reverb por
   convolução** (fecha os efeitos do W3) · **W5 espectral** (spectrograma, repair,
   denoise) · **W4 pitch/formant** (PSOLA/phase-vocoder clean-room).
3. **W4 voz** (parte não-FFT): de-hum, de-click/de-crackle, de-plosive, leveler/AGC.
4. **W7 ML** (opt-in, feature `audio-ml`): DeepFilterNet, Demucs.

**Dívida sinalizada:** `snapshot.rs` do painel está **600/600** (no teto). O estado da
FX-chain deveria sair pra um irmão `fx_chain.rs` **antes** do próximo bloco que mexa em
`snapshot.rs`. Este bloco não tocou lá (o estado de loop foi pra `loop_state.rs`), então
não bloqueou — mas fica anotado.

**Protocolo (Modo L):** trabalhe e comite **nesta linha** (`git commit
--no-verify`), **sem push**. Você **não integra nem faz ship** — fecha, escreve o
handoff de integração (DIRETRIZ §1.5.9) e **para**. Integração/ship são decisão
exclusiva do Enio, via agente integrador dedicado.

---

## 1. Proposta & escopo do módulo

Objetivo: um **subsistema de áudio em tempo real** para o engine PH2D
(`ph2d-audio`), platform-agnostic (HR-1), com uma **UI de mixer** estilo mesa de
som docada no Inspector.

Escopo escolhido: **subsistema** (`ph2d-audio`), não um domínio de nós. O domínio
de nós `Sound` (autoria via `ph2d-eval-sound`) é fase futura e **exige ADR**
(`Domain::Sound` é contrato congelado — Coord/Enio-only, DIRETRIZ §4). A API
Luau/MCP (jogo dispara sons) está **bloqueada**: o loop de script gameplay na
shell é placeholder (`shells/desktop/.../integration.rs`), não há tick por-frame
pra dirigir uma fila de comandos de áudio.

### O que JÁ está construído (fundamentos + features)

**Fundamentos (já integrados ao `main` numa jornada anterior):**
- Duas metades pela fronteira de thread: `AudioEngine` (control, aloca, devolve
  handles) ↔ `AudioRenderer` (audio thread, `Send`, no-alloc/no-free HR-3) via
  **2 ring buffers** lock-free bounded (`crossbeam-queue::ArrayQueue`).
- `VoiceId` opaco (HR-8), pool de 64 vozes com **voice stealing** (oldest-quietest).
- Playback: pitch/resample (interp linear, cursor f64), pan (equal-power), gain
  (`SmoothGain` linear-ramp), ADSR. Decode via **symphonia** (`ph2d-audio-decode`).
- `cpal` confinado na shell (`shells/desktop/src/audio.rs`).
- Meter (`AudioMeter`), `MemoryBudget` (HR-13), gate no-alloc por capacidade.

**Features (os 10 commits desta linha — §3):** sub-buses (Master/Music/SFX/UI/
Voice), faders dB-tapered, pan por strip, filtro (Tone) low-pass por strip, meter
VU (RMS + peak-hold + clip latching), **limiter** (soft-clip + envelope de
redução de ganho), **reverb** (Freeverb no master), **ducking** (Music/SFX
abaixam sob o bus Voice), **Play Test** (oscilador embutido pra testar sem
arquivo).

---

## 2. Arquitetura (o que você PRECISA saber pro bug)

### Crates
- `crates/ph2d-audio` — o motor (platform-agnostic). Sem `cpal`.
- `crates/ph2d-audio-decode` — decode symphonia → `SampleData`.
- `crates/ph2d-panel-audio-mixer` — o painel do mixer (UI-only, sem dep de
  `ph2d-audio`; fala com a shell por thread-locals de snapshot).
- `shells/desktop/src/audio.rs` — `AudioSystem` (cpal + engine).
- `shells/desktop/src/render_loop/mod.rs` — a **ponte** por-frame painel↔engine.

### Fluxo de sinal (control → audio → device)

```
Painel (thread-locals snapshot)  ──ponte por-frame──►  AudioSystem (shell)
   fader/pan/tone/mute/solo/                              set_master_*/set_bus_*
   limiter/reverb/ducking                                 (change-gated) → comandos
                                                                 │ command ring
                                                                 ▼
   AudioRenderer::render(out, frames)   ← cpal callback (audio thread) chama isto
     1. drena comandos → mixer.apply
     2. scratch.reset (master + bus_scratch)
     3. mixer.render(master, bus_scratch, &mut bus_peaks, &mut bus_rms, ...)
          · por sub-bus: render voices → filtro → fader → (peak/rms do BUS) → fold em master c/ pan
          · voices master-direct somam em master
          · LOOP MASTER: gain → filtro → reverb(se on) → pan → limiter(se on)  [in-place em master]
     4. peak/rms do MASTER a partir de `master` (pós-tudo) → meter.store
     5. write_out(out, master, ...)   ← copia master → out (clamp ±1)
                                                                 │
   cpal callback: scatter `out`(nossa fmt) → `data`(device, T::from_sample) ──► DEVICE
```

### ⚠️ Fato-chave pro diagnóstico (LEIA)

- O **meter do MASTER** (RMS/peak da UI, via `engine.levels()/rms()`) é calculado
  no **passo 4**, do buffer `master` **DEPOIS** de todo o processamento master
  (gain/filtro/reverb/pan/limiter). É **o MESMO buffer** que o `write_out` (passo
  5) manda pro device. **Logo: se o meter do MASTER se mexe, o `master` tem sinal
  e o device recebe sinal.**
- Os **meters dos SUB-BUSES** (Music/SFX/UI/Voice) leem `bus_peaks/bus_rms`,
  calculados **no fold, ANTES** do processamento master. **Podem se mexer mesmo
  com o master zerado.**

→ **Isso é o bisect central do bug (§4).**

Arquivos exatos:
- Loop master + meter master + `write_out`: `crates/ph2d-audio/src/engine.rs`
  (`AudioRenderer::render` ~linha 210-275; `write_out` ~linha 310).
- Loop dos sub-buses + processamento master in-place:
  `crates/ph2d-audio/src/mixer.rs` (`Mixer::render`).
- Scatter → device: `shells/desktop/src/audio.rs` (`build_stream`, o closure do
  callback com `T::from_sample`).

---

## 3. A linha `line/audio` e a integração futura ao `main`

**Modo L** (workstation, ADR-0106/0107): a linha tem **worktree + índice
próprios** em `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio`, branch
`line/audio`. Sem colisão de git; valem só conflitos de merge.

- **Comitar:** `git commit --no-verify -m "..."` (fast mode). **Sem `git push`.
  Sem integrar.** A integração ao `main` é **1× no fim**, quando o Enio mandar.
- **Fim de mensagem de commit:** `Co-Authored-By: Claude Opus 4.8 (1M context)
  <noreply@anthropic.com>`. (Msgs com `()`/backticks quebram no fish — use `git
  commit -F <arquivo>`.)
- **10 commits locais não-integrados** (`git log --oneline main..line/audio`):
  `9eae9fb6` dB-taper+solo+UI/Voice sub-buses · `a14e579b` VU meter · `1b881e61`
  Play Test · `ff245ee9` clip indicator · `3cac1cc6` limiter soft-clip ·
  `62df1af7` limiter audível (envelope) · `41d1c4e3` Tone por canal · `060f4b84`
  reverb Freeverb · `6d3011fa` ducking · `b5328b40` fix Play Test clipado.
  (Uma leva ANTERIOR de features de áudio JÁ foi integrada ao `main` numa jornada
  passada — o integrador achou até um 4º spot de `libasound2-dev` no CI. Estas 10
  são as novas desde então.)
- **Integração (quando o Enio pedir):** Modo L integra ao `main` por
  `scripts/foundational-integrate.sh` (gate da árvore combinada) + `--ff-only`;
  Mergiraf funde resíduo textual. Pontos de atenção do handoff anterior que
  seguem valendo (arquivos foundational que outras linhas também tocam):
  - `crates/ph2d-panel-registry-init/` (registro de painéis — re-rodar
    **panel-sync** na árvore combinada; blocos gerados não fundem textualmente).
  - `crates/ph2d-editor-core/src/widget/` + `showcase/` (widget `LevelMeter` +
    re-rodar **widget-sync**).
  - `.github/workflows/spike.yml` (`libasound2-dev` p/ ALSA — precisa sobreviver).
  - `ids/` (inspector/topbar), `screens/hero/` (paint z-order, fixture),
    `topbar/mod.rs` (extração `chip_name.rs` p/ ficar sob o LOC cap).
  - `deny.toml` já permite MPL-2.0 (symphonia). Workspace é glob (`crates/*`).
- **Contrato congelado NÃO tocado** (nada de ADR necessário pela linha): áudio não
  adiciona Tool/Node gateado. `SCHEMA_VERSION` intocado (não mexe em save).

---

## 4. O BUG — "grafos vivos, sem som audível" (RESOLVIDO 2026-07-08)

### ✅ Causa-raiz + fix (2026-07-08)
Confirmado o **Diagnóstico #1**: o meter do **Master mexe** → o sinal chega ao
`write_out` → o problema é **depois** dele (device/sistema). Inspeção do PipeWire
(`pactl`) revelou a máquina do Enio com saída ativa **7.1 de 8 canais**
(`alsa_output...HiFi_7_1__Speaker__sink`, s32le 8ch). O app abria o device no
**config nativo (8 canais)** e o scatter (`build_stream`) escrevia a mix estéreo só
em **FL/FR (canais 0,1)**, silêncio nos outros 6 — **bypassando o roteamento/upmix
estéreo→surround do PipeWire**. Todo app audível abre um stream **estéreo** (2ch) e
deixa o PipeWire mapear; o nosso não.
**Fix (`audio.rs::AudioSystem::new`):** pedir um stream **estéreo** quando o device
tem >2 canais **e** oferece uma config de 2ch (`supported_output_configs`), com
fallback pro nativo. Agora o PipeWire trata como qualquer app estéreo.
**Fator secundário (verificar se persistir):** há uma entrada **`module-stream-restore`
salva pro `ph2d-host-desktop`** — o PipeWire lembra rota/volume/**mute** do app entre
sessões (independente do painel). Se ainda mudo após o fix, no **pavucontrol** →
Playback → `ph2d-host-desktop` → **desmutar/rotear** (fica lembrado), ou limpar a
entrada de stream-restore.

### Sintoma (relato do Enio)
Toca o Play Test → **os meters/grafos do mixer se mexem** (algo está acontecendo)
mas **não há som audível** no device. (Antes disso houve um sintoma DIFERENTE já
resolvido: o botão Play Test tinha sido **empurrado pra fora da área visível** do
Inspector pelo crescimento do painel — corrigido em `b5328b40` movendo Play pro
topo + encolhendo os strips. Agora o Play é clicável, os meters mexem, mas sem som.)

### Timeline (pista forte de bisect)
- No commit do **reverb** (`060f4b84`) o Enio confirmou **"smoke ok"** → **tinha
  som**.
- Depois vieram só: **ducking** (`6d3011fa`, shell+painel) e **fix do Play
  clipado** (`b5328b40`, só paint do painel).
- **Ducking DESLIGADO (default) é comprovadamente no-op**: `update_ducking(false,
  _)` retorna `1.0`; a ponte faz `g = sub_gain[i] * 1.0`. O `fix do Play` só mexe
  em layout de paint. → **Nenhum dos dois deveria quebrar o áudio no caso
  default.** Isso é suspeito: ou (a) tem efeito ligado, ou (b) algo mais sutil.

### 🔍 Diagnóstico #1 (FAÇA PRIMEIRO — bisecta em 1 observação)
**O meter do strip MASTER se mexe, ou SÓ os de Music/SFX?**
- Pelo §2 (fato-chave): o meter do **Master** lê `master` pós-tudo, o mesmo que
  vai pro device. Se o **Master mexe** → o device recebe sinal → o problema é
  **depois do write_out** (scatter/cpal/device/sistema) ou conversão de amostra.
- Se **só Music/SFX mexem** e o **Master fica parado** → bug de **estágio
  master** (algo entre o fold e o write_out zera/NaN-a o `master`).

### Suspeitos (com localização)
Todos os defaults DEVERIAM ser passthrough, então instrumente pra achar o valor
real. Ordem sugerida:

1. **Efeitos ligados pelo usuário.** Peça pra testar num **launch limpo, TODOS os
   efeitos OFF, só Play Test**. Se com tudo OFF **tem som** → o culpado é um
   efeito (reverb instável/limiter/ducking). Se **sem som mesmo tudo OFF** →
   estágio master default ou device.

2. **Default do Tone/Cutoff NÃO é bypass real (imperfeição confirmada por
   leitura).** `engine.lowpass_coeffs` (`engine.rs`): `if cutoff_hz >= sr*0.5*0.9
   { identity } else { lowpass(...) }`. A 48 kHz, `sr*0.5*0.9 = 21600`. O default
   "aberto" do Tone é **20000 Hz** (`sub_tone_target()`/`master_cutoff_target()`),
   e 20000 < 21600 → aplica um **low-pass real de 20 kHz** em TODO sub-bus **e no
   master**, por default. Isso **não silencia** (só corta HF acima de 20 kHz), mas
   é um bug de default a corrigir (o "aberto" deveria mapear pra identity). **Cheque
   se não há um caminho onde o cutoff efetivo cai muito** (ex.: algum lugar
   mandando um Hz baixo). Confirme os Hz reais enviados (instrumento #2).

3. **Estágio master (`mixer.rs`, loop master).** Reverb/limiter só quando ON. Pan
   default `balance_gains(0.0) = [1,1]`. Gain default 1.0. **Cheque NaN/Inf**: se o
   reverb (feedback) ou o limiter produzir NaN, propaga pro `master`;
   `NaN.clamp(-1,1)` em `write_out` pode virar NaN → device muda pra silêncio/
   estouro. (Reverb OFF por default, mas confirme.)

4. **Caminho device (inalterado, mas verifique).** `write_out` (`engine.rs`) +
   scatter `T::from_sample` (`audio.rs`). Se o meter do Master mexe mas `out[]`
   sai zero/NaN, o bug está aqui.

5. **Ponte da shell mandando valor ruim.** `render_loop/mod.rs`: a cada frame
   manda `set_master_gain/pan/cutoff/limiter/reverb` + `set_bus_gain/pan/cutoff`.
   Todos os defaults parecem passthrough — **instrumente os valores reais**.

### Instrumentação sugerida (temporária, remova depois)
1. Em `AudioRenderer::render` (`engine.rs`), a cada ~100 blocos:
   `eprintln!` do `peak_l` do master **e** de `out[0..4]` **depois** do
   `write_out`. Prova se (a) o master tem sinal e (b) o buffer do device recebe.
2. Na ponte (`render_loop/mod.rs`), ~1×/s: `eprintln!` dos valores mandados
   (`master_gain_target()`, `master_cutoff_target()`, `master_pan_target()`,
   `limiter()`, `reverb_on/size/mix`, `ducking()`, e o `duck` calculado, `sub_tone`).
3. **Bisect por env-smoke (ótimo!):** o path de env NÃO passa pelo Play Test:
   - `PH2D_AUDIO_SMOKE=1` → tom 440 Hz no **bus SFX**.
   - `PH2D_AUDIO_FILE=<wav>` → arquivo no **bus Music**.
   Se o env-smoke **tem som** mas o Play Test não → o problema está nos geradores
   do Play Test (`pluck_loop`/`swell_loop`/`blip_loop`) ou no roteamento. Se o
   env-smoke **também é mudo** → estágio master/device (independe do Play Test).
4. **`git bisect`** entre `060f4b84` (reverb, tinha som) e `HEAD` (mudo). Só 2
   commits no meio (ducking, fix-clip) — build + smoke manual em cada.

### Hipóteses que JÁ foram descartadas por leitura (não gaste tempo)
- Ducking OFF atenuar Music/SFX: **não** (`update_ducking(false,_) == 1.0`).
- Play-clip-fix mexer em áudio: **não** (só paint do painel).
- Change-gate floodar/dropar comando de Play: **não** (ring 1024, drenado por bloco).

---

## 5. Como rodar / gates / convenções

- **Rodar (SEMPRE com o `cd` junto — preferência do Enio):**
  ```
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && cargo run -p ph2d-host-desktop
  ```
  Abre a janela → TopBar **MIX** abre o mixer → **Play Test** (topo da seção master).
  Env-smokes: prefixe `PH2D_AUDIO_SMOKE=1` ou `PH2D_AUDIO_FILE=/som.wav`.
- **Inner loop:** `cargo check -p <crate>`. **Não** `--workspace`.
- **Gate no fechamento** (rode do worktree):
  - `cargo test -p ph2d-audio` (34 unit + `tests/bus_routing.rs` prova roteamento,
    RMS≤peak, limiter, reverb tail, filtro).
  - `cargo test -p ph2d-panel-audio-mixer` (14 seam tests, `tests/seam.rs`).
  - `cargo test -p ph2d-editor-core --test no_magic_numeric --test no_literal_color
    --test hr15_no_hardcoded_ui_strings --test architecture_panel_loc_cap
    --test architecture_workspace_file_loc_cap` (UI/LOC gates — o painel toca-os).
  - `cargo clippy -p ph2d-audio -p ph2d-panel-audio-mixer -p ph2d-host-desktop --all-targets`.
  - `cargo build -p ph2d-host-desktop` (link completo — cpal/ALSA precisa
    `libasound2-dev`).
- **UI canônica (HR-15):** zero hex, zero `f32` literal de UI, zero string
  hardcoded. Constantes de áudio (Hz, dB, ratios) marcam `// LITERAL-PX-OK: <razão>`.
  Widgets pela **Widget Gallery** (o painel usa `Slider`/`LevelMeter` canônicos,
  não chrome improvisado). Labels em inglês.
- **LOC cap:** fn ≤200, arquivo ≤700. Se estourar, **extraia módulo/fn-irmã** (já
  foi feito: `paint()` → `paint_master_section`/`paint_labeled_slider`;
  `topbar/mod.rs` → `chip_name.rs`).

---

## 6. Mapa de arquivos-chave

**Core (`crates/ph2d-audio/src/`):**
- `engine.rs` — `AudioEngine`/`AudioRenderer`; **`render` (meter + write_out)** ⭐.
- `mixer.rs` — `Mixer::render` (**loop sub-bus + loop master**) ⭐; `soft_clip`,
  `balance_gains`, `set_reverb/limiter/bus_filter`.
- `bus.rs` — `BusId` (Master/Music/SFX/UI/Voice), `SUB_BUS_COUNT=4`, `SUB_BUSES`.
- `meter.rs` — `AudioMeter` (peak+RMS master + por-bus).
- `dsp/` — `biquad.rs`, `gain.rs` (`SmoothGain`), `envelope.rs`, `pan.rs`,
  **`reverb.rs`** (Freeverb).
- `voice.rs`/`pool.rs`/`buffer.rs`/`command.rs`/`format.rs`.
- `tests/bus_routing.rs` — testes de integração de roteamento/efeitos.

Também no core, adicionados pelo editor (append-only, isolados):
- `command.rs` — 6 variantes `AudioCommand::{Play,Seek,SetData,SetLooping,Pause,Stop}Preview`.
- `engine.rs` — `play_preview`/`seek_preview`/`set_preview_data`/`set_preview_looping`/
  `pause_preview`/`stop_preview`/`preview_frame`/`preview_playing`; const
  **`PREVIEW_VOICE_ID = VoiceId(u64::MAX)`** (voz de preview FORA do pool do jogo).
- `voice.rs` — `replace_data` (**hot-swap** do buffer mantendo o cursor → edição ao vivo).
- `meter.rs` — atomics `preview_frame: AtomicU64` / `preview_active: AtomicU32`.
- `dsp/loudness.rs` — `integrated_lufs()` (BS.1770 offline, usado pelo normalize LUFS).

**Edição offline (`crates/ph2d-audio-edit/src/`)** — control-thread only, HR-3/HR-5
NÃO se aplicam (aloca e usa `tanh`/`exp` livremente):
- `lib.rs` — `EditClip` (clipe + peak cache + seleção + **timeline de undo**,
  snapshots `Arc` baratos, cap 64); `apply_*` + **`apply_effect`**; `target()` =
  seleção ou clipe inteiro.
- `ops.rs` — gain/normalize peak+LUFS/reverse/invert/DC/trim/silence/delete/fade/
  `snap_to_zero_crossing` + **`in_range`** ⭐ (aplica op length-preserving só na
  região e reencaixa; é o que torna tudo selection-aware).
- `fx.rs` — **`Effect`** ⭐ (LowPass/HighPass/Compress/Saturate/Bitcrush/StereoWidth).
- `peaks.rs` — `PeakCache`/`column_peaks` (envelope min/max p/ a waveform).

**Encode (`crates/ph2d-audio-encode/src/`):** `write_wav`/`encode_wav`, `BitDepth`.

**Painéis (`crates/ph2d-panel-audio-{mixer,editor}/src/`):** ambos UI-only.
- mixer: `lib.rs` (ids `AMIX_*`/`SUB_*` + `snapshot`), `paint.rs`, `event.rs`,
  `populate.rs`, `fader.rs`, `tests/seam.rs`.
- editor: `lib.rs` (ids `AEDIT_*` + **`AudioEditCmd`**, o enum one-shot painel→shell),
  `paint.rs` (transporte + seções Edit/Range/Effects), `event.rs`, `populate.rs`,
  `snapshot.rs` (thread-locals), `tests/seam.rs`.

**Shell (`shells/desktop/src/`)** — `audio.rs` foi **dividido** (HR-18 shell cap):
- `audio.rs` — `AudioSystem` (cpal, `build_stream`/scatter ⭐, `update_ducking`,
  `set_master_*`/`set_bus_*` change-gated). **Prefere stream 2ch em device 7.1.**
- `audio/editor.rs` — runtime do editor: `editor_load/toggle_play/apply/…`,
  `WaveView` (rect+frames publicados p/ o hit-test da seleção).
- `audio/signals.rs` — geradores `sine_tone`/`pluck`/`swell`/`blip_loop`.
- `render_loop/mod.rs` — **a ponte por-frame** ⭐ (drena intents dos painéis → engine).
- `render_loop/audio_overlay.rs` — overlay flutuante do waveform (ruler, playhead,
  banda de seleção, handles de drag/resize).
- `input_dispatch.rs` — **seleção por arrasto** na waveform (SHELL-only) + **teardown
  do `cpal::Stream` no `on_close_request`** ⭐ (segfault 139).
- `input_handlers.rs` — **Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y** → undo/redo do `EditClip`.
- `main.rs` — env-smokes `PH2D_AUDIO_SMOKE`/`PH2D_AUDIO_FILE`.

**⚠️ Gotcha de wiring:** nenhum seam test cobre **atalho de teclado** — eles vivem
só em `input_handlers.rs`. Botão verde no seam ≠ atalho ligado (foi exatamente o
"undo não funciona" de 2026-07-09: os botões funcionavam, o Ctrl+Z não existia).

**Processo:** `CLAUDE.md` (roteador), `docs/IntegracaoMultiAgente/DIRETRIZ.md` +
`DIRETIVA_IMPLEMENTACAO.md`, `project-memory/MEMORY.md`.
