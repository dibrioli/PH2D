# SFX Lab — W0: a rodada de medição (05/11)

Decisão: [ADR-0198](../architecture/decisions/0198-o-motor-do-sfx-lab-e-kernels-proprios-sobre-det-math.md). Instrumento:
[`ferramentas/sonda_w0/corre.sh`](ferramentas/sonda_w0/corre.sh), projeto à parte para o FunDSP não entrar no `Cargo.lock` do app.
Uma rodada, intercalada no mesmo processo: 7 rodadas rotativas, perfil `smoke`, Ryzen 9950X, `load` 24–32 (outras linhas a compilar).

⚠️ As duas variantes têm a mesma estrutura (8 camadas: 2 modais de 32 modos, 2 ruídos, 2 tons, 2 FM), **não o mesmo som**: os filtros do
FunDSP são outros. A comparação é de custo e de determinismo, não de timbre. Nenhum som foi ouvido. Os kernels são escalares e sem otimização.

## Custo de `render` (ms; mínimo · mediana)

| Receita | 0,5 s | 2 s |
|---|---|---|
| Típica, kernels | 2,35 · 2,61 | **9,61 · 10,34** |
| Típica, FunDSP em blocos (SIMD) | 2,96 · 3,05 | 11,87 · 14,39 |
| Típica, FunDSP amostra a amostra | 34,88 · 44,12 | 165,47 · 173,66 |
| Pior (8 modais de 128 modos), kernels | 9,38 · 9,58 | **38,79 · 39,63** |
| Pior, FunDSP em blocos | 12,45 · 14,69 | 50,57 · 57,14 |
| Típica, kernels + rack (Compress, LowPass, Limiter) | 3,93 · 5,24 | 16,17 · 22,50 |

Por camada de kernels, 2 s (primeira rodada): modal 32 modos 5,20 · ruído 0,65 · tom 0,47 · FM 0,81. Montar a rede do FunDSP: 0,07 ms.

**O limite de 30 ms é proposta, não medição.** Medido ao lado dele: típica 9,6 ms passa; **o pior caso, 38,8 ms, passa do limite** e aciona o
kill-criterion do plano. A cura que ele pede cabe no desenho: as camadas se somam, então só a camada editada se renderiza de novo (no pior caso,
38,8 / 8 ≈ 4,9 ms). **Cache por camada entra antes de W3.** O número certo ainda é o do ouvido do dono, em W3.

## A semente muda quando se edita um vizinho?

O vizinho tem ganho zero, então qualquer diferença no resultado vem da semente. A linha «MUDOU» do FunDSP é o controlo positivo do detector.

| Alvo | Parâmetro do vizinho | Nó a mais no vizinho | Vizinho depois do alvo |
|---|---|---|---|
| Kernels (semente = receita + id da camada) | igual | igual | igual |
| FunDSP sem semente | igual | **MUDOU** | **MUDOU** |
| FunDSP com semente e fase fixadas | igual | igual | igual |
| FunDSP fixadas + `envelope` | igual | **MUDOU** | **MUDOU** |

⛔ A última linha decide: o `envelope` sorteia os pontos de amostragem pela posição na rede, e `seed`/`phase` não o alcançam. Todo som de jogo tem envelope.

## Bytes entre alvos (receita típica, 2 s)

| Alvo | Kernels | FunDSP amostra a amostra | FunDSP em blocos |
|---|---|---|---|
| x86-64 de base | `e262c4b2a017544c` | `274ed83b8cec805a` | `03c043d751830188` |
| x86-64-v3 (AVX2 e FMA no compilador) | `e262c4b2a017544c` | `274ed83b8cec805a` | **`c14cef12e4cf6b20`** |
| wasm32 no `node` | `e262c4b2a017544c` | `274ed83b8cec805a` | `03c043d751830188` |

⛔ **NÃO PROVADO: Intel e ARM.** Esta máquina é AMD, a linha não faz push e não há `qemu` aqui. O wasm32 é outra arquitetura de conjunto de
instruções, mas é evidência parcial: ele não executa num processador ARM. Para fechar, no Mac, depois da integração:
`cd docs/SfxLab/ferramentas/sonda_w0 && cargo run --profile smoke --bin so_kernels` tem de imprimir `kernels e262c4b2a017544c`.

## Matemática do sistema (HR-5)

`nm -D --undefined-only` em cada binário, com o controlo positivo na última linha:

| Binário | Importa da libm do sistema |
|---|---|
| `so_kernels` | nada |
| `so_fundsp` | nada (ele usa o crate `libm`) |
| `sonda` (com o rack) | `atan2f cos cosf exp2 exp2f expf hypot hypotf log10f powf sinf tanf tanhf` |

## Auditoria do rack `ph2d_audio_edit::Effect`

38 variantes (`fx.rs:118`), `Effect::apply(&SampleData) -> SampleData` (`fx.rs:440`). **Nenhuma passa por `ph2d-det-math`**, e o crate se declara
fora da HR-5 (`Cargo.toml:14-15`). A nota do plano atribuía a isenção ao `ph2d-audio`; quem a declara é o `ph2d-audio-edit`, e o `ph2d-audio-spectral`.
O gate `o_motor_nao_chama_a_libm_do_sistema` cobre 11 crates, nenhum de áudio.

- **Sem matemática de plataforma (6):** Comb, Haas, Bitcrush, StereoWidth, DeClip, DeClick (este só `sqrt` e `powi`).
- **Com (32):** todos os outros. Quase todos por `BiquadCoeffs` ou `exp` de constante de tempo; Vibrato, Chorus, Flanger, Doubler, Tremolo, AutoPan e RingMod por `sin` por amostra.
- Nenhum usa `rand`, `HashMap`, relógio ou FFT. No mesmo processo o rack repete o hash (`76a0e60b79d4071d` duas vezes).
- `DetMath` não tem `exp2`, `log2` nem `tanh`, que o rack usaria: portar o rack pede antes essas funções.

⇒ **A garantia entre máquinas não vale para receita com efeito.** «O som aprovado nunca muda» não depende disso: o áudio aprovado fica guardado.

## Volume: qual é a porta única

Oráculos só executados, nunca linkados (triagem por artefato): `ffmpeg 2:9.0.2-2.1` é `GPL-3.0-only` (`--enable-gpl`), `sox 14.8.0.1-1` é `GPL-2.0-only`.

| Fixture | `integrated_lufs` | `ebur128` I | Dif. | Pico nosso / `sox` (dB) | `true_peak` / `ebur128` (dB) |
|---|---|---|---|---|---|
| Seno 1 kHz, −23 dBFS, 5 s | −23,25 | −23,00 | −0,25 | −23,00 / −23,00 | −23,00 / −23,00 |
| Receita típica, −3 dBFS, 2 s | −13,55 | −13,30 | −0,25 | −3,01 / −3,01 | −3,01 / −3,00 |
| Golpe curto, 150 ms | −26,25 | −70,00 | **+43,75** | −11,42 / −11,42 | −11,19 / −11,20 |
| 1 s de som + 9 s de silêncio | −39,94 | −29,50 | **−10,44** | −13,17 / −13,17 | −13,14 / −13,10 |

**Porta única: `ph2d_audio::dsp::integrated_lufs`** (`dsp/loudness.rs:50`); pico verdadeiro: `ph2d_audio_edit::true_peak`, que bate com o `ebur128` em até 0,04 dB.

Para W6, onde o gate contra o `ebur128` nasce:

1. Viés constante de −0,25 LU, a investigar nos filtros RBJ.
2. Sem o portão de silêncio do BS.1770: a linha de 9 s de silêncio erra 10,44 LU, e um som com cauda longa sofre o mesmo.
3. ⚠️ O `ebur128` devolve −70 para o que tem menos de 400 ms, a maior parte dos SFX. Para esses sons o oráculo não responde, e a regra de equilíbrio tem de ser escrita por nós.

Há também um `LoudnessMeter` momentâneo (400 ms, para o medidor ao vivo) e três RMS soltos; o medidor continua, os RMS ficam para W6.

## A sonda offline

O mixer já renderiza sem dispositivo: `AudioEngine::new(format) -> (AudioEngine, AudioRenderer)` e `AudioRenderer::render(out, frames)` (`engine.rs:72`, `:460`).
É com ela que a sonda de W1 conta os disparos.
