# HANDOFF — `line/audio` · W7: denoise ML nativo (DeepFilterNet via `tract`)

> **Para o próximo agente que assume a linha.** Este documento é auto-suficiente: leia-o inteiro
> antes de tocar em código. A decisão já está tomada e provada; sua tarefa é o **build-out**.
> **Autor do handoff:** agente anterior (2026-07-15). **Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio`.

---

## §0 — Protocolo (INEGOCIÁVEL — leia antes de tudo)

- **Modo L.** Você trabalha SÓ neste worktree. Cada mutação por **caminho absoluto**.
- **NÃO integra, NÃO faz push, NÃO roda `ship.sh`.** A linha fecha, escreve o handoff de
  fechamento e **PARA**. Integração/ship é **ordem explícita do Enio, no fim do dia** — nunca
  autônomo (CLAUDE.md §0.7).
- **Git:** `git add -- <seus paths>` (NUNCA `-A`/`.`/`stash`); `git commit --no-verify -F <arquivo>`
  (mensagem por `-F`, nunca `-m` com crase — o fish executa crase como comando). Acumule commits;
  não comite a cada fix.
- **UI em inglês** (HR-15): zero hex, zero `f32` literal de UI, zero string hardcoded — tokens/i18n.
- **`typos`** trata pt-BR como inglês mal escrito. Ou reescreva, ou adicione a palavra em
  `.typos.toml [default.extend-words]` (o repo já faz isso pra dezenas de palavras pt).
- **Inner loop = `cargo check -p <crate>`.** Teste/clippy/typos/deny **1× no fechamento**, não por task.

---

## §1 — A decisão (ADR-0123, ACEITO) e o número que a autorizou

O Enio pediu "padrão-ouro, custo à parte", e o **experimento de aceitação decidiu**. Sobre o
fixture do próprio autor do DeepFilterNet (voz real a 0 dB SNR, com a referência limpa pareada):

| sinal | SI-SDR vs limpo | ganho |
|---|---|---|
| entrada com ruído | 6,05 dB | — |
| nosso denoise W5 (`ph2d-audio-spectral`) | 7,94 dB | +1,89 dB |
| **DeepFilterNet3** | **20,01 dB** | **+13,96 dB** |

**DFN bate o W5 por +12 dB** (a barra do kill-criterion era +6). Leia o ADR inteiro:
[`docs/architecture/decisions/0123-audio-w7-ml-boundary-tract-native-denoise-reject-ort.md`](architecture/decisions/0123-audio-w7-ml-boundary-tract-native-denoise-reject-ort.md).
Pontos que o ADR já fechou e você **não re-litiga**:

- **`ort`/ONNX está BARRADO por contrato** (pré-release; baixa a runtime C++ da rede no build; 72
  crates + pilha TLS). Se você se pegar querendo `ort`, pare — o caminho é `tract`.
- **Demucs/stems fica FORA do workspace** (worker/serviço externo, ADR próprio se um dia). Não é
  seu escopo.
- **Licença do modelo: redistribuível.** O autor já empacota o peso dentro do crate MIT/Apache
  dele (`libDF` feature `default-model` faz `include_bytes!`), e distros/PipeWire distribuem. Não
  precisa re-verificar.

---

## §2 — A tarefa

Construir a crate **`ph2d-audio-ml`** (nova, isolada, feature `audio-ml` **OFF por default**) que
embrulha o runner do DeepFilterNet e expõe um denoise por IA como **operação de voz offline no
editor**, do lado do `ph2d-audio-spectral` (control-thread). O jogo de runtime (`ph2d-audio`)
**não** ganha nenhuma dep de ML.

Assinatura-alvo da crate (espelhe o `denoise` do W5):
```rust
// ph2d-audio-ml/src/lib.rs
pub fn denoise_ml(data: &SampleData, amount: f32) -> SampleData;
// amount 0 => retorna a entrada intocada, byte a byte (o neutro que todo efeito da rack promete).
// DFN aprende o ruído SOZINHO (não precisa de NoiseProfile como o W5).
```

---

## §3 — Fatos de de-risco (o ouro — já verificados, NÃO redescubra)

### API do runner (`libDF` / `df::tract`)
```rust
use df::tract::*;
let dfp = DfParams::default();                 // modelo embutido (feature "default-model")
let rp  = RuntimeParams::default_with_ch(1);   // mono
let mut model = DfTract::new(dfp, &rp)?;
let hop = model.hop_size;                       // 160 amostras @ 48 kHz
// processa em blocos de `hop`: entrada/saída são ndarray Array2<f32> shape [1, hop]
model.process(ibuf.view(), obuf.view_mut())?;   // retorna SNR local estimado (f32)
```
- **Taxa fixa 48 kHz.** Se o clipe não for 48 kHz, **resample na fronteira** (o `resample.rs` do
  W6 já existe na linha — windowed-sinc, fase linear). Volte à taxa original na saída.
- **hop = 160.** Alimente em blocos de 160; guarde o resto (`len % 160`) sem processar ou faça
  padding com zero e apare na saída.

### Fixação de versões (ISTO custou tempo no experimento — não repita)
- **`libDF` 0.5.7-pre no git HEAD NÃO compila** contra `tract 0.21.17` (API drift: `symbol_table`
  sumiu do `Graph<InferenceFact,...>`, 17 erros de tipo). **Use o tag `v0.5.6`** (o release cujo
  `libDF` casa com o `tract` que ele fixa), OU vendorize o `libDF` 0.5.6.
- **`kstring 2.0.3`** (transitivo via `liquid`/`tract-nnef`) exige **rustc 1.96** > nossa pin 1.95.
  **Pin em 2.0.2:** `cargo update -p kstring --precise 2.0.2`.
- O `deep_filter` **estável no crates.io (0.2.5) é SÓ DSP** (não denoiza). O que denoiza é o
  `libDF` 0.5.x, que **não está no crates.io** — daí git-pin no tag OU vendorizar.

### Como trazer o `libDF` (recomendação: git-pin p/ validar, vendorizar p/ fechar)
1. **Teste rápido** com git-pin no tag:
   ```toml
   deep_filter = { git = "https://github.com/Rikorose/DeepFilterNet", tag = "v0.5.6",
                   default-features = false, features = ["tract", "default-model"] }
   ndarray = "0.15"   # casar com o que o libDF usa
   ```
2. **Feche vendorizando** o `libDF` 0.5.6 (MIT/Apache, cópia com atribuição — é o padrão que a
   linha já usou no `ph2d-audio-opus`). Motivo: não deixar o CI refém do GitHub e não depender de
   um tag `-pre`. Confine tudo em `ph2d-audio-ml` (a dep `tract` **não** pode aparecer no
   `cargo tree` de `ph2d-audio` nem de `ph2d-host-desktop` com a feature OFF).

### O modelo
- `DeepFilterNet3_onnx.tar.gz` = **7,6 MB** (a `default-model` do `libDF` já o embute via
  `include_bytes!`). A variante `_ll` (low-latency, 34,7 MB) NÃO é necessária — use a padrão.

### CLI de referência (pra comparar o SEU wrapper)
Binário oficial (musl, x86_64, modelo embutido):
`https://github.com/Rikorose/DeepFilterNet/releases/download/v0.5.6/deep-filter-0.5.6-x86_64-unknown-linux-musl`
Uso: `./deep-filter -D -o out <arquivo.wav>` (`-D` compensa o atraso de STFT+lookahead — essencial
pra alinhar com a referência).

---

## §4 — Plano (3–4 rodadas; a rodada 1 é a única incerta)

1. **Motor.** `ph2d-audio-ml` compilando (fixar tract/kstring §3) + `denoise_ml` + **um teste que
   prova que o seu wrapper reproduz o resultado da CLI** no fixture do §7. **Se o wrapper não bater
   o +12 dB, a feature não existe** (kill-criterion do ADR §4).
2. **Integração UI.** Comando-irmão do Denoise do W5 (veja §5) + feature-gate `audio-ml` OFF.
3. **Gates + smoke** (veja §6) + split de LOC se `fx.rs`/algo estourar 700.
4. **(folga)** pro que a rodada 1 empurrar.

**~700–1000 LOC seus** (wrapper + costura + gates); o resto é dep/modelo.

---

## §5 — Onde pluga na UI (control-thread, offline)

O denoise do W5 é o modelo a copiar. Em [`shells/desktop/src/audio/editor.rs`](../shells/desktop/src/audio/editor.rs):
- `Cmd::Denoise => self.editor_denoise();` (linha ~467).
Em [`shells/desktop/src/audio/editor/spectral.rs`](../shells/desktop/src/audio/editor/spectral.rs):
- `editor_denoise()` (~225) chama `ph2d_audio_spectral::denoise(clip.data(), profile, amount)` e
  grava via `EditClip::commit_rendered` (**1 undo step**).

A operação ML é o mesmo desenho, **sem o passo de Learn** (DFN não precisa de profile):
- Novo `Cmd::DenoiseMl` (ou nome equivalente) → `editor_denoise_ml()` → `ph2d_audio_ml::denoise_ml(clip.data(), amount)` → `commit_rendered`.
- Botão no painel de áudio (English label, ex.: "AI Denoise"), **só visível/ativo com a feature
  `audio-ml`** (com a feature OFF, o botão nem existe — não pinte botão morto).
- `amount 0` = no-op byte-idêntico (o neutro da rack).

**Confinamento:** `ph2d-audio-ml` depende de `ph2d-audio` (pra `SampleData`), como a
`ph2d-audio-spectral`. A dep `tract` fica SÓ em `ph2d-audio-ml`. O shell faz a ponte, como já faz
com a spectral.

---

## §6 — Gates obrigatórios (ADR-0123 §4)

1. **`audio-ml` OFF = build intocado.** Gate: `tract`/`deep_filter` **não** aparece no
   `cargo tree -p ph2d-host-desktop` sem a feature. (E o build default não compila nada de ML.)
2. **`no_ml_runtime_reaches_the_mixer`** — irmão do `no_codec_reaches_the_mixer` já existente:
   `ph2d-audio` (o mixer RT) não alcança `tract`. Grep de dep + teste.
3. **O wrapper reproduz o ganho** — teste sobre o fixture do §7: `si_sdr(clean, denoise_ml(noisy))`
   ≥ ~18 dB (a CLI deu 20; deixe folga de alinhamento). Este é o gate que impede o wrapper de estar
   silenciosamente quebrado (ex.: hop errado, resample errado) e ainda "passar".
4. **`amount 0` byte-idêntico** + **a rack não regride** (os 5 gates da rack seguem verdes).

**Cuidado (lição da linha):** um gate de presença precisa do irmão de ausência e vice-versa
([[feedback_absence_gate_needs_a_presence_sibling]]). "A IA não vaza pro mixer" (ausência) fica
verde no silêncio total — pareie com "a IA de fato denoiza" (presença, gate #3).

---

## §7 — O experimento de aceitação (RECEITA reproduzível)

O scratchpad da sessão anterior **não sobrevive** à sua sessão. Reproduza assim:

**Fixtures** (baixe do repo DFN):
```
https://raw.githubusercontent.com/Rikorose/DeepFilterNet/main/assets/clean_freesound_33711.wav
https://raw.githubusercontent.com/Rikorose/DeepFilterNet/main/assets/noisy_snr0.wav
```
(48 kHz mono, 10,6 s; `noisy` = `clean` + ruído a 0 dB, sample-alinhados.)

**Métrica: SI-SDR** (scale-invariant SDR, o padrão de denoise), com **alinhamento de atraso por
correlação cruzada** (DFN tem lookahead; sem alinhar, o SI-SDR despenca por fase):
1. `lag` = argmax da correlação cruzada de `est` vs `clean` numa janela central (±4096–8192).
2. `est` deslocado por `lag`; `alpha = <clean,est>/<clean,clean>`; `target = alpha*clean`;
   `SI-SDR = 10·log10(||target||² / ||est−target||²)`.
3. Para o **W5** (baseline honesto): aprenda o `NoiseProfile` do trecho **mais silencioso** do
   clipe (o uso real: o usuário seleciona um vão), varra `amount` 0,6/0,8/1,0.

Números a bater (a sessão anterior mediu): W5 ≈ +1,9 dB, DFN ≈ +14 dB. O **seu wrapper** deve
chegar perto dos +14 (paridade com a CLI). O código do medidor da sessão anterior está descrito
aqui em prosa de propósito — reescrevê-lo é trivial e evita depender de arquivo efêmero.

**Demo tocável na máquina do Enio** (se ainda existir): `~/ph2d_dfn_demo/{1_ANTES,2_DEPOIS,3_referencia}.wav`.

---

## §8 — Fechamento (rode 1× no fim, sobre o diff acumulado)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio
cargo test -p ph2d-audio-ml                                        # motor + o gate de paridade
cargo test -p ph2d-host-desktop --tests                            # ⚠️ --tests (LOC gate + integração)
cargo test -p ph2d-editor-core --tests                             # arch-gates
cargo build -p ph2d-host-desktop                                   # feature OFF: NÃO puxa tract
cargo build -p ph2d-host-desktop --features audio-ml               # feature ON: compila
cargo clippy --all-targets -p ph2d-audio-ml -p ph2d-host-desktop
rustup run 1.95 cargo fmt --all -- --check
typos
cargo deny check && cargo machete                                  # mexeu em deps -> obrigatório
```
Depois: **atualize o CLAUDE.md §5** (entrada de Áudio) e o handoff de fechamento; **feche a linha e
PARE**. Não integre.

**Exemplo pronto pro smoke** (o Enio testa no fim): monte uma cena/comando que carregue um clipe
ruidoso e rode o AI Denoise num clique, com `cd` junto na linha de comando
([[feedback_ready_to_smoke_example]] · [[feedback_run_command_include_cd]]).

---

## §9 — O que NÃO fazer

- **Não** use `ort`/ONNX (barrado, §1).
- **Não** deixe `tract` alcançar `ph2d-audio` (o mixer RT). Gate #2.
- **Não** siga o git HEAD do DeepFilterNet (não compila — §3). Tag v0.5.6 ou vendorize.
- **Não** pinte o botão de AI Denoise com a feature OFF (botão morto).
- **Não** integre nem faça push. Feche, escreva o handoff, PARE.

## Estado da linha ao entregar este handoff

- **11 commits à frente do `main`**, worktree limpa. Últimos: ADR-0123 (aceito, `4318f056`),
  o ADR proposto (`59debc44`), o rename Gate/Expander (`a5ec9d7a`), o import de `.opus` (`0da875cd`).
- **Nada de ML foi adicionado à linha ainda** — o experimento inteiro rodou fora do repo. Você
  começa a rodada 1 do zero, com todos os fatos de de-risco do §3 na mão.
