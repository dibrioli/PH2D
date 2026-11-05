# Small local AI for game sound effects: verified shortlist and realistic roles (as of 2026-11-05)

Method: licence texts and model cards were downloaded directly from the repos. File sizes come from the Hugging Face API (`/api/models/<id>?blobs=true`, read 2026-11-05) or the Zenodo API, not from summaries. Limits on this method are listed in each Gaps section. The main one: the Stability model repos are gated, so their per-repo `LICENSE.md` returned HTTP 401. The public Community License page and the card's licence tag were read instead.

Classes used below, for a commercial, closed-distribution app:
- **USABLE**: code and weights are permissive in the text read.
- **CONDITIONAL**: allowed only under stated conditions, such as a revenue ceiling, notices or registration.
- **NO**: non-commercial, copyleft incompatible with closed distribution, or no licence for the weights.

This is an engineering reading of licence text, not legal advice.

## Objective: the shortlist, classified

### Takeaway
No text-to-audio model is both under 500 MB as a complete stack and free of licence conditions. The only serious commercial candidates are the Stability models (CONDITIONAL, 1.1 to 1.6 GB, optional download). Everything small and clean is non-generative: CLAP search, feature search, sound-matching a synth, vocoders, and a small grammar-constrained LLM.

### Cited Findings

Text-to-audio:

| Model | Files needed (real size) | Rate / length | Published speed | Code licence | Weights licence | Class |
|---|---|---|---|---|---|---|
| Stable Audio 3 Small-SFX | ONNX: DiT fp16 921 MB + SAME-S decoder fp8 59 MB + T5Gemma 620 MB = about 1.6 GB. Raw: 2,270 MB + 1,183 MB | 44.1 kHz stereo, up to 2 min | Mac CPU 0.70 s (5 s clip), 1.72 s (30 s), 5.92 s (120 s); H200 about 0.41-0.45 s | MIT | Stability Community + Gemma Terms | **CONDITIONAL** |
| Stable Audio Open Small | LiteRT: 346 + 313 + 440 MB = about 1.1 GB. Safetensors 1,677 MB | 44.1 kHz stereo, up to 11 s | About 75 ms H100; about 7 s mobile; 4.3 s M4 MacBook | MIT | Stability Community | **CONDITIONAL** |
| Stable Audio Open 1.0 | 4,854 MB | 44.1 kHz stereo, up to 47 s | 8.6 s on A40 (47 s clip, 100 steps) | MIT | Stability Community | **CONDITIONAL**, too large |
| EzAudio (L / XL) | 2,388 / 3,499 MB + VAE 183 MB | Not verified | Not found | MIT | MIT tag | **USABLE on paper**, too large, data provenance unverified |
| MeanAudio-S | 482 MB, plus files in the same repo: CLAP 2,352 MB, `v1-16.pth` 687 MB (VAE, judged by filename), BigVGAN 449 MB | Not verified | RTF 0.013 on RTX 3090, 1 step | MIT | MIT tag, trained on academic-only data | **NO in practice** (see legal section) |
| SoundCTM | Student 3,472 MB | 16 kHz | Not found | MIT | CC-BY-4.0 tag; repo also holds a teacher and an AudioLDM (NC-SA) checkpoint; trained on AudioCaps | **NO in practice** |
| TangoFlux | 3,437 + 625 MB | 44.1 kHz stereo, up to 30 s | 3.7 s on A40 (50 steps) | Repo licence below | Non-commercial | **NO** |
| Woosh (Sony) | Not read (v1.0.0 assets not listed) | 48 kHz | 4 NFE distilled variant | MIT / Apache | CC-BY-NC | **NO** |
| AudioGen medium | 3,679 + 236 MB | 16 kHz | Not found | MIT | CC-BY-NC 4.0 | **NO** |
| MusicGen small | 841 + 236 MB | Music only | Not found | MIT | CC-BY-NC 4.0 | **NO** |
| AudioLDM small v2 | About 1.68 GB | 16 kHz | Not found | MIT-style text | CC-BY-NC-SA 4.0 | **NO** |
| AudioLDM2 | Over 4 GB | 16 kHz | Large: 24.8 s on A40 (10 s clip, 200 steps) | Repo licence is CC-BY-NC-SA | CC-BY-NC-SA 4.0 | **NO** |
| MMAudio | Not read | 8 s default | About 6 GB GPU memory | MIT | CC-BY-NC 4.0 | **NO** |

Sources for that table:
- Stable Audio 3: [repo README table](https://github.com/Stability-AI/stable-audio-3), [paper](https://arxiv.org/abs/2605.17991), [optimized files](https://huggingface.co/stabilityai/stable-audio-3-optimized), [code LICENSE](https://raw.githubusercontent.com/Stability-AI/stable-audio-3/main/LICENSE).
- Stable Audio Open Small: [model card](https://huggingface.co/stabilityai/stable-audio-open-small), [ARC paper](https://arxiv.org/abs/2505.08175), [Google/Arm timing](https://developers.googleblog.com/en/accelerating-on-device-ai-a-look-at-arm-and-google-ai-edge-optimization/).
- Stable Audio Open 1.0: [model card](https://huggingface.co/stabilityai/stable-audio-open-1.0), [timing table in the TangoFlux README](https://github.com/declare-lab/TangoFlux).
- Stability tools code: [LICENSE](https://raw.githubusercontent.com/Stability-AI/stable-audio-tools/main/LICENSE).
- EzAudio: [card](https://huggingface.co/OpenSound/EzAudio), [LICENSE](https://raw.githubusercontent.com/haidog-yaqub/EzAudio/main/LICENSE).
- MeanAudio: [README](https://github.com/xiquan-li/MeanAudio), [files](https://huggingface.co/AndreasXi/MeanAudio).
- SoundCTM: [README](https://github.com/sony/soundctm), [files](https://huggingface.co/Sony/soundctm), [LICENSE](https://raw.githubusercontent.com/sony/soundctm/main/LICENSE).
- TangoFlux: [LICENSE.md](https://raw.githubusercontent.com/declare-lab/TangoFlux/main/LICENSE.md), [card](https://huggingface.co/declare-lab/TangoFlux).
- Woosh: [README](https://github.com/SonyResearch/Woosh), [paper](https://arxiv.org/abs/2604.01929).
- AudioGen / MusicGen: [AudioGen card](https://huggingface.co/facebook/audiogen-medium), [MusicGen card](https://huggingface.co/facebook/musicgen-small), [code LICENSE](https://raw.githubusercontent.com/facebookresearch/audiocraft/main/LICENSE), [weights LICENSE](https://raw.githubusercontent.com/facebookresearch/audiocraft/main/LICENSE_weights).
- AudioLDM: [small v2 card](https://huggingface.co/cvssp/audioldm-s-full-v2), [AudioLDM2 card](https://huggingface.co/cvssp/audioldm2), [AudioLDM2 repo LICENSE](https://raw.githubusercontent.com/haoheliu/AudioLDM2/main/LICENSE), [training repo LICENSE](https://raw.githubusercontent.com/haoheliu/AudioLDM-training-finetuning/main/LICENSE). The training repo's LICENSE is MIT-form text carrying the odd copyright line "Scott Chacon and others".
- MMAudio: [README](https://github.com/hkchengrex/MMAudio).

Non-generative and supporting models:

| Model | Role | Real size | Code licence | Weights licence | Class |
|---|---|---|---|---|---|
| LAION-CLAP HTSAT unfused, ONNX | Text and audio search, tagging, scoring | Audio encoder 34.3 MB (quantized) to 117.5 MB; text encoder 126.6 MB (quantized) | CC0 | Apache-2.0 tag on source; ONNX mirror has no tag | **USABLE**, mixed-data caveat |
| PANNs MobileNetV2 / Cnn10 / Cnn14 | Tagging (AudioSet labels) | 20.8 / 25.2 / 327.4 MB | MIT | CC-BY-4.0 on Zenodo | **USABLE** with attribution |
| YAMNet | Tagging, 521 classes | 15.3 MB | Apache-2.0 | Same repo terms; hosted separately | **USABLE** |
| BEATs | Embeddings, tagging | Not verified | MIT | Not stated in README | **Unverified**, skip |
| Vocos 24 kHz | Mel to audio | 54.4 MB | MIT | MIT tag | **USABLE** |
| BigVGAN base 24 kHz / v2 44 kHz | Mel to audio | 56.3 / 489 MB | MIT | MIT tag | **USABLE** |
| Descript Audio Codec | Autoencoder at 44.1 kHz, latent morphing | 76 M params; file size not read | MIT | MIT, stated in README | **USABLE** |
| DeepFilterNet3 | Denoise (speech-oriented) | 7.6 MB, already in PH2D | MIT or Apache-2.0 | README's dual licence covers "all code"; weights not named separately | **USABLE**, low residual doubt |
| DDSP / DDSP-VST | Timbre transfer on pitched material | Not read | Apache-2.0 | Train your own | **USABLE** |
| RAVE | Timbre transfer | Not read | CC-BY-NC 4.0 | Depends on the model | **NO** |
| NoiseBandNet | SFX neural synthesis | Not read | AGPL-3.0 | Train your own | **NO** for closed app |
| Neutone SDK | Model wrapper | Not applicable | LGPL-2.1 | Per model | **CONDITIONAL**, needs TorchScript |
| Demucs | Music stems | Not read | MIT | "not clear" per maintainer | **NO** |
| Spleeter | Music stems | Not read | MIT | Not stated | **NO** until stated |
| SAM-Audio | Text-prompted isolation | 5.1 GB small, 14.9 GB large | SAM License | SAM License | **CONDITIONAL**, too large |
| Qwen3-0.6B Q4_K_M | Recipe LLM | 396.7 MB | llama.cpp MIT | Apache-2.0 | **USABLE** |
| SmolLM2-135M Q4_K_M | Tiny recipe LLM | 105.5 MB | llama.cpp MIT | Apache-2.0 | **USABLE** |
| Syntheon, TorchSynth, SynthAX, CTAG | Sound-matching method and code | Not applicable | Apache / Apache / MIT / MIT | Train your own | **USABLE** |

Sources for that table:
- CLAP: [ONNX files](https://huggingface.co/Xenova/clap-htsat-unfused), [source weights](https://huggingface.co/laion/clap-htsat-unfused), [code LICENSE](https://raw.githubusercontent.com/LAION-AI/CLAP/main/LICENSE).
- PANNs: [Zenodo](https://zenodo.org/records/3987831), [LICENSE](https://raw.githubusercontent.com/qiuqiangkong/audioset_tagging_cnn/master/LICENSE.MIT).
- YAMNet: [README](https://github.com/tensorflow/models/tree/master/research/audioset/yamnet), [LICENSE](https://raw.githubusercontent.com/tensorflow/models/master/LICENSE), [weights file](https://storage.googleapis.com/audioset/yamnet.h5).
- BEATs: [README](https://github.com/microsoft/unilm/tree/master/beats), [LICENSE](https://raw.githubusercontent.com/microsoft/unilm/master/LICENSE).
- Vocos: [card](https://huggingface.co/charactr/vocos-mel-24khz), [LICENSE](https://raw.githubusercontent.com/gemelo-ai/vocos/main/LICENSE).
- BigVGAN: [card](https://huggingface.co/nvidia/bigvgan_v2_24khz_100band_256x), [LICENSE](https://raw.githubusercontent.com/NVIDIA/BigVGAN/main/LICENSE).
- DAC: [README](https://github.com/descriptinc/descript-audio-codec), [LICENSE](https://raw.githubusercontent.com/descriptinc/descript-audio-codec/main/LICENSE), params from [Woosh paper table 1](https://arxiv.org/abs/2604.01929).
- DeepFilterNet: [README](https://github.com/Rikorose/DeepFilterNet), and `crates/ph2d-audio-ml/Cargo.toml` in this repo for the 7.6 MB figure.
- DDSP: [LICENSE](https://raw.githubusercontent.com/magenta/ddsp/main/LICENSE), [DDSP-VST LICENSE](https://raw.githubusercontent.com/magenta/ddsp-vst/main/LICENSE).
- RAVE: [LICENSE](https://raw.githubusercontent.com/acids-ircam/RAVE/master/LICENSE).
- NoiseBandNet: [LICENSE](https://raw.githubusercontent.com/adrianbarahona/noisebandnet/master/LICENSE).
- Neutone: [LICENSE](https://raw.githubusercontent.com/Neutone/neutone_sdk/main/LICENSE).
- Demucs: [README](https://github.com/adefossez/demucs).
- Spleeter: [README](https://github.com/deezer/spleeter).
- SAM-Audio: [LICENSE](https://raw.githubusercontent.com/facebookresearch/sam-audio/main/LICENSE), [small](https://huggingface.co/facebook/sam-audio-small), [large](https://huggingface.co/facebook/sam-audio-large).
- LLMs: [Qwen3 LICENSE](https://huggingface.co/Qwen/Qwen3-0.6B/raw/main/LICENSE), [Qwen3 GGUF](https://huggingface.co/unsloth/Qwen3-0.6B-GGUF), [SmolLM2 card](https://huggingface.co/HuggingFaceTB/SmolLM2-135M-Instruct), [SmolLM2 GGUF](https://huggingface.co/bartowski/SmolLM2-135M-Instruct-GGUF), [llama.cpp LICENSE](https://raw.githubusercontent.com/ggml-org/llama.cpp/master/LICENSE).
- Synth tools: [Syntheon](https://github.com/gudgud96/syntheon), [TorchSynth LICENSE](https://raw.githubusercontent.com/torchsynth/torchsynth/main/LICENSE), [SynthAX LICENSE](https://raw.githubusercontent.com/PapayaResearch/synthax/main/LICENSE), [CTAG LICENSE](https://raw.githubusercontent.com/PapayaResearch/ctag/main/LICENSE).

### Inferences
Suggested order of value for a game SFX tool, best first:
1. **Text prompt to editable synth recipe**, using a procedural synth, CLAP as the judge, and an evolutionary search (CTAG method). Under 300 MB, no generative weights, and the result stays editable.
2. **Library search, auto-tag and auto-name** with CLAP. 34 MB for audio-only similarity, 161 MB with text queries.
3. **Sound-match to synth parameters**, trained in-house on PH2D's own synth, so there are no third-party weights.
4. **Variation generation from one sound**, procedural first, then a DAC-style latent if measured to help.
5. **Optional text-to-audio pack** (Stable Audio 3 Small-SFX) as a separate user-accepted download, never in the installer.
6. **Small LLM to pick a recipe skeleton**, only if measurement beats a plain lookup table.

### Gaps
- No full-stack size is under 500 MB for any legally usable text-to-audio model.
- All speed figures are vendor-published. No independent x86 CPU or mid-range GPU benchmark was found for any of the Stability models. See the Rust section for the measurement this needs.

## Text-to-audio: what the licences actually say, and how fast it is

### Takeaway
Stable Audio 3 Small-SFX is the current best candidate: 2 minutes of 44.1 kHz stereo, sub-second to a few seconds on consumer hardware per the vendor, and an official ONNX export. It is not small (about 1.6 GB) and carries two licences, including a revenue trigger that terminates the licence automatically. Almost everything else in the assigned list is non-commercial.

### Cited Findings
- Stability Community License, threshold, verbatim: "If at any time You or Your Affiliate(s), either individually or in aggregate, generate more than USD $1,000,000 in annual revenue (or the equivalent thereof in Your local currency), regardless of whether that revenue is generated directly or indirectly from the Stability AI Materials or Derivative Works, any licenses granted to You under this Agreement shall terminate as of such date." — [Community License, last updated July 5, 2024](https://stability.ai/community-license-agreement)
- Commercial use requires registration, verbatim: "If You are using or distributing the Stability AI Materials for a Commercial Purpose, You must register with Stability AI." — [Community License](https://stability.ai/community-license-agreement)
- Distribution duties attach to "a product or service that uses any portion of them". You must provide a copy of the Agreement, keep a Notice file reading "This Stability AI Model is licensed under the Stability AI Community License, Copyright © Stability AI Ltd. All Rights Reserved", and "prominently display 'Powered by Stability AI' on a related website, user interface, blogpost, about page, or product documentation". — [Community License](https://stability.ai/community-license-agreement)
- Output cannot feed a competitor, verbatim: "You will not use the Stability AI Materials or Derivative Works, or any output or results ... to create or improve any foundational generative AI model (excluding the Models or Derivative Works)." — [Community License](https://stability.ai/community-license-agreement)
- Fine-tunes and LoRAs count as Derivative Works, which you own subject to Stability's ownership of the materials. — [Community License](https://stability.ai/community-license-agreement)
- The revenue threshold counts organisation revenue "regardless of the source of that revenue", and Stable Audio 3.0 is on the Community list. — [stability.ai/license](https://stability.ai/license)
- Stable Audio 3 also pulls in Gemma terms through its text encoder: the model "includes components under the Gemma Terms of Use". — [Small-SFX card](https://huggingface.co/stabilityai/stable-audio-3-small-sfx)
- Gemma distribution needs a copy of the agreement and a Notice file reading "Gemma is provided under and subject to the Gemma Terms of Use found at ai.google.dev/gemma/terms". — [Gemma Terms, modified April 1, 2026](https://ai.google.dev/gemma/terms)
- Gemma terms also state: "Google reserves the right to restrict (remotely or otherwise) usage of any of the Gemma Services" believed to violate the agreement, incorporate a Prohibited Use Policy, and say Google claims no rights in outputs. — [Gemma Terms](https://ai.google.dev/gemma/terms)
- Small-SFX parameter count conflicts across official sources: 433M in the [repo README](https://github.com/Stability-AI/stable-audio-3), 459M in [paper table 1](https://arxiv.org/abs/2605.17991) (diffusion transformer only; SAME-S adds 108M), and "0.6B" on the [card](https://huggingface.co/stabilityai/stable-audio-3-small-sfx).
- The README's speed table has one `small` row, not separate rows for Small-SFX and Small-Music, so the Small-SFX timings in the shortlist assume that row applies to it. Peak VRAM for `small` is 1.69 GB (5 s), 1.89 GB (30 s), and 2.40 GB (120 s). The Mac CPU column is footnoted "CPU-only via CoreML (Diffusion Transformer) + TFLite (SAME-S decoder)", so it is not a plain cross-platform CPU path. — [repo README](https://github.com/Stability-AI/stable-audio-3)
- Stable Audio Open Small is "up to 11s" stereo at 44.1 kHz. Its stated intended use is "research and experimentation", and it "should not be used on downstream applications without further risk evaluation and mitigation". — [card](https://huggingface.co/stabilityai/stable-audio-open-small)
- Its parameter count also conflicts: "0.5B" on the [card](https://huggingface.co/stabilityai/stable-audio-open-small) against 341M reported by [AlternativeTo](https://www.alternativeto.net/news/2025/5/stability-ai-open-sources-an-audio-model-that-can-run-on-smartphones-with-arm-cpus/) citing the announcement.
- Stable Audio Open Small on Arm: 14 s fell to 6.6 s on an SME2 Android device with one thread, and 10 s fell to 4.3 s on an M4 MacBook. — [Google/Arm](https://developers.googleblog.com/en/accelerating-on-device-ai-a-look-at-arm-and-google-ai-edge-optimization/)
- Arm reports about 10 s of audio in about 8 s on premium phones. — [Arm](https://developer.arm.com/community/arm-community-blogs/b/ai-blog/posts/audio-generation-arm-cpus-stable-audio-open-small-kleidiai)
- Stable Audio Open 1.0 "is better at generating sound effects and field recordings than music", and is "not able to generate realistic vocals". — [card](https://huggingface.co/stabilityai/stable-audio-open-1.0)
- TangoFlux, verbatim: "created for **non-commercial, research-only** purposes under the **UK data copyright exemption**", and "only academic uses are permitted for data sourced from WavCaps". — [LICENSE.md](https://raw.githubusercontent.com/declare-lab/TangoFlux/main/LICENSE.md)
- TangoFlux checkpoints "are for non-commercial research use only". — [card](https://huggingface.co/declare-lab/TangoFlux)
- AudioCraft, verbatim: "Code is released under MIT, model weights are released under CC-BY-NC 4.0." — [AudioGen model card](https://raw.githubusercontent.com/facebookresearch/audiocraft/main/model_cards/AUDIOGEN_MODEL_CARD.md)
- Woosh, verbatim: "The open weights in the releases page are released under the CC-BY-NC license." — [README](https://github.com/SonyResearch/Woosh)
- Woosh authors' metrics on AudioCaps-Test: Woosh-Flow-Public FD 109.1, TangoFlux 131.9, SAO 150.1; the 4-step DFlow gives FD 132.1. On InternalSFX-Test: 282.7, 378.1, 341.0. These are author-run. — [paper](https://arxiv.org/abs/2604.01929)
- Sony also trained private Woosh models on commercially licensed libraries, "around 1 M samples and 5500 h". Only the public-dataset weights are released. — [paper](https://arxiv.org/abs/2604.01929), [README](https://github.com/SonyResearch/Woosh)

### Inferences
- The revenue test binds whoever distributes. For the PH2D owner the test is his own and his affiliates' total revenue from any source. Crossing $1M ends the licence automatically, so the feature must be removable without breaking old projects (keep generated audio as plain assets).
- Each end user who downloads the model is plausibly a licensee in their own right, so a studio over $1M would need its own licence. The cleanest design is a separate opt-in download with both licences shown, off by default. A lawyer should confirm this before shipping.
- The Gemma remote-restriction wording needs legal review for an app promising local, offline use. Technically nothing in a local ONNX file can be restricted remotely, but the clause exists.
- 11 s (Open Small) already covers most one-shot SFX, but Stable Audio 3 Small-SFX is newer and SFX-specific, and has an official ONNX path. It should be tested first.

### Gaps
- Per-repo `LICENSE.md` for Stability's gated models was not read (HTTP 401). Someone must accept the gate and diff it against the public text. The tag `stable-audio-community` suggests it may be a distinct document.
- Whether Small-SFX's diffusion transformer used Freesound, AudioSparx or both was not confirmed. The card gives the family's data as 1,278,902 recordings (806,284 AudioSparx, 472,618 Freesound).
- `onnx/sa3-sm-sfx/dit_fp8` totals about 921 MB, the same as fp16. The reason is not explained in what was read.
- No published speed for AudioGen, EzAudio or SoundCTM on CPU or mid-range GPU was found. EzAudio's sample rate and clip length were not verified.
- No listening comparison between these models for game SFX specifically was found, and none was run.

## Timbre transfer and neural synthesis: can a designer train on their own sounds?

### Takeaway
Yes technically, but the best-known SFX-capable tools cannot be shipped: RAVE is non-commercial and NoiseBandNet is AGPL. DDSP is permissive but built around pitch. Vocoders are permissive and small, but they only turn spectrograms into audio. A designer-training feature would need PH2D's own implementation.

### Cited Findings
- RAVE repo LICENSE is headed "Creative Commons Attribution-NonCommercial 4.0 International". — [LICENSE](https://raw.githubusercontent.com/acids-ircam/RAVE/master/LICENSE)
- RAVE claims 48 kHz and about 20 times faster than real time on a standard laptop CPU. This is an author claim. — [paper](https://arxiv.org/abs/2111.05011)
- RAVE training minimum GPU memory by config: 5 GB (lightweight Raspberry Pi config), 6 GB (`onnx`), 8 GB (original continuous model), 16 GB (improved continuous model), 18 GB (discrete), 32 GB (style-transfer variant). Export is a separate step, and `--streaming` is needed for realtime. — [README](https://github.com/acids-ircam/RAVE)
- RAVE offers a "Noiseless v1 configuration for onnx usage". — [README](https://github.com/acids-ircam/RAVE)
- DDSP-VST training needs "as little as a few minutes of audio", and takes "a couple hours on a free Colab GPU" or "less than an hour on Colab Pro". Its pitch model has about 160k parameters. — [Magenta blog](https://magenta.tensorflow.org/ddsp-vst-blog)
- DDSP-VST reduces input to pitch and volume, and its audio path depends on a tiny CREPE pitch model. — [Magenta blog](https://magenta.tensorflow.org/ddsp-vst-blog)
- NoiseBandNet is aimed specifically at sound effects: trained on a folder of WAV files, it uses "2048 filters and a synthesis window of 32 samples", and offers loudness transfer and user-defined controls. — [site](https://www.adrianbarahonarios.com/noisebandnet/), [README](https://github.com/adrianbarahona/noisebandnet)
- NoiseBandNet's licence is AGPL-3.0. — [LICENSE](https://raw.githubusercontent.com/adrianbarahona/noisebandnet/master/LICENSE)
- Neutone SDK is LGPL-2.1 and converts models "to TorchScript". Its own notes list alternatives to TorchScript as still being explored. — [README](https://github.com/Neutone/neutone_sdk)
- Neutone offers a realtime Demucs model in its free VST/AU plugin. — [Demucs README](https://github.com/adefossez/demucs)
- Vocos generates "spectral coefficients" in one pass, with "an order of magnitude increase in speed" over time-domain neural vocoders. — [card](https://huggingface.co/charactr/vocos-mel-24khz), [paper](https://arxiv.org/abs/2306.00814)
- BigVGAN sizes: base 24 kHz has 14M params; v2 at 44 kHz has 122M. — [card](https://huggingface.co/nvidia/bigvgan_v2_24khz_100band_256x)
- BigVGAN speed work targets CUDA, and its card reports "1.5 - 3x faster speed on a single A100 GPU" with the custom kernel. — [card](https://huggingface.co/nvidia/bigvgan_v2_24khz_100band_256x)
- Descript Audio Codec: "Weights are released as part of this repo under MIT license", 44.1 kHz. — [README](https://github.com/descriptinc/descript-audio-codec)

### Inferences
- RAVE's non-commercial licence covers its code. A model exported from it inherits doubt at minimum. Treat "train with RAVE, ship the export" as not cleared.
- NoiseBandNet's published idea (a bank of noise filters driven by a small net) is simple, and a clean-room implementation from the paper under `/pd-cleanroom` could be a small drop-crate. Reading its AGPL code first would contaminate that.
- DDSP suits pitched sounds such as engines, whooshes with a clear tone, and creatures. It is a weak fit for impacts and noise, which are most of game SFX.
- DAC is the only permissively licensed high-rate autoencoder found. It is a candidate for "morph between two sounds" via latent interpolation, but it was designed for compression, so morph quality is unproven.

### Gaps
- RAVE and NoiseBandNet training time on a mid-range GPU and data amount were not found in the primary pages read.
- Neutone's plugin licence was not read.
- Vocos and BigVGAN quality on non-speech SFX was not measured, and their training data was not read in full.
- DAC's training data and weight file size were not read.

## Audio search and understanding: find similar, auto-tag, auto-name

### Takeaway
CLAP is the clear choice: the ONNX audio encoder is 34 to 118 MB, and it supports text queries, similarity and zero-shot tagging in one model. A tiny tagger such as YAMNet or PANNs MobileNet is worth adding only if a fixed label list proves more reliable for auto-naming.

### Cited Findings
- `Xenova/clap-htsat-unfused` ONNX sizes: audio encoder 117.5 MB, fp16 60.4 MB, quantized 34.3 MB; text encoder 501.5 MB, fp16 251.0 MB, quantized 126.6 MB; combined quantized 161.0 MB. — [HF files](https://huggingface.co/Xenova/clap-htsat-unfused)
- Source `laion/clap-htsat-unfused` carries an Apache-2.0 tag, with a 614.5 MB PyTorch file. — [card](https://huggingface.co/laion/clap-htsat-unfused)
- `lukewys/laion_clap` carries a CC0 tag, with checkpoints of 1.86 to 2.35 GB. — [card](https://huggingface.co/lukewys/laion_clap)
- LAION-CLAP code is CC0 1.0. — [LICENSE](https://raw.githubusercontent.com/LAION-AI/CLAP/main/LICENSE)
- CLAP's zero-shot ESC50 score is about 90% for the larger checkpoints. — [README](https://github.com/LAION-AI/CLAP)
- CLAP's authors state: "Due to copyright reasons, we cannot release the dataset we train this model on." — [README](https://github.com/LAION-AI/CLAP)
- LAION-Audio-630k includes BBC sound effects, Epidemic Sound, Audiostock and Freesound. — [dataset README](https://github.com/LAION-AI/audio-dataset/tree/main/laion-audio-630k)
- YAMNet predicts 521 AudioSet classes with a MobileNet v1 design; `yamnet.h5` is 15.3 MB. — [README](https://github.com/tensorflow/models/tree/master/research/audioset/yamnet), [file](https://storage.googleapis.com/audioset/yamnet.h5)
- The TensorFlow Models LICENSE says all files in `/research` are Apache-2.0. — [LICENSE](https://raw.githubusercontent.com/tensorflow/models/master/LICENSE)
- PANNs checkpoints on Zenodo are CC-BY-4.0: Cnn14 327.4 MB (mAP 0.431), MobileNetV2 20.8 MB (0.383), MobileNetV1 23.6 MB (0.389), Cnn10 25.2 MB (0.380), LeeNet11 3.0 MB (0.266). — [Zenodo](https://zenodo.org/records/3987831)
- BEATs code is MIT. — [LICENSE](https://raw.githubusercontent.com/microsoft/unilm/master/LICENSE)

### Inferences
- Feature pipeline: run the audio encoder on import, store a 512-dim vector per sound (an in-memory brute-force search is enough for thousands of clips), and compare with cosine similarity.
- For auto-naming, score each sound against a curated SFX vocabulary with CLAP text embeddings computed once at build time. Those could ship as a table, leaving only the 34 MB audio encoder in the app. Free-text search then needs the 127 MB text encoder, as an optional part.
- Search outputs no audio, so the risk from mixed training data is far lower than for a generator.

### Gaps
- Quality loss from the quantized CLAP audio encoder was not measured. It should be measured on PH2D's sounds against fp32.
- The `Xenova` ONNX mirror has no licence tag of its own. Exporting from the Apache-tagged source in-house removes this doubt.
- BEATs checkpoint size and weights licence were not confirmed in the README read.
- No independent comparison of CLAP against the others on game SFX retrieval was found.

## Separation and cleanup

### Takeaway
PH2D already ships the right small denoiser (DeepFilterNet). Music-stem separators are poor fits and have unclear weights licences. Prompted isolation exists (SAM-Audio) but at 5 GB and above it falls far outside the size target.

### Cited Findings
- Demucs maintainer, verbatim: "The license of the weights is not clear: MusDB contains some data that were only usable for research purpose. I sadly cannot help granting a license, both coming from Meta custom data, or from the original MusDB data." — [README](https://github.com/adefossez/demucs)
- Demucs needs "at least 3GB" of GPU RAM, and "about 7GB" with default arguments. — [README](https://github.com/adefossez/demucs)
- Spleeter's README says only "The code of **Spleeter** is MIT-licensed". It states no licence for weights, and has a note asking automated agents not to contact maintainers on licensing. It is TensorFlow-based. — [README](https://github.com/deezer/spleeter)
- DeepFilterNet: "All code in this repository is dual-licensed" MIT or Apache-2.0. Its stated purpose is real-time speech enhancement. — [README](https://github.com/Rikorose/DeepFilterNet)
- PH2D already vendors DeepFilterNet with tract, embedding the 7.6 MB DFN3 model. — `crates/ph2d-audio-ml/Cargo.toml` in this repo.
- SAM License grant is general, with no non-commercial clause in the grant text read: "a non-exclusive, worldwide, non-transferable and royalty-free limited license ... to use, reproduce, distribute, copy, create derivative works of, and make modifications to the SAM Materials". It also bars reverse engineering and military or ITAR-type uses. — [LICENSE](https://raw.githubusercontent.com/facebookresearch/sam-audio/main/LICENSE)
- SAM-Audio isolates "any sound in audio using text, visual, or temporal prompts". Checkpoints: small 5,100 MB, large 14,861 MB, gated by manual approval. — [README](https://github.com/facebookresearch/sam-audio), [small](https://huggingface.co/facebook/sam-audio-small)
- AudioSep code is MIT and works at 32 kHz. — [LICENSE](https://raw.githubusercontent.com/Audio-AGI/AudioSep/main/LICENSE), [README](https://github.com/Audio-AGI/AudioSep)

### Inferences
- A speech-trained denoiser may damage non-speech SFX (it could remove the wanted noise). Keep it for voice recordings and foley, with a clear label, and do not auto-apply it to generated sound.
- Classic DSP (gate, spectral subtraction) is likely adequate for cleaning generated SFX tails. This was not measured.

### Gaps
- DeepFilterNet's training datasets and weights licence were not separately confirmed.
- AudioSep's weights licence was not found in the README read, so it is not cleared.
- No small permissive general-sound separator under 500 MB was found.

## AI-free or tiny ML: which are feasible under 100 MB?

### Takeaway
Most of it is. Feature nearest-neighbour search, preset interpolation, a small autoencoder preset map, and parameter inference are all small because they run on PH2D's own synth and need no third-party weights. Only the CLAP-judged variant goes slightly over 100 MB, and only if free-text prompts are wanted.

### Cited Findings
- CTAG renders prompts through a 78-parameter virtual modular synth, scored by LAION-CLAP, using gradient-free evolutionary search. Gradients were avoided because they are unstable in the differentiable synth. — [paper](https://arxiv.org/html/2406.00294v1), [repo](https://github.com/PapayaResearch/ctag)
- CTAG timing on one V100 GPU: 300 iterations took about 28 s (population 25), 50 s (population 50), and 97 s (population 100). 50 iterations at population 25 took about 5.5 s. Prompts took 50 to over 300 iterations to reach their best result. — [paper](https://arxiv.org/html/2406.00294v1)
- CTAG user study (10 raters, 60 sounds each): correct identification was CTAG 56.0%, AudioLDM 34.0%, AudioGen 59.5%. CTAG sounds were rated more artistic (3.54/5), with lower confidence (2.99/5). — [paper](https://arxiv.org/html/2406.00294v1)
- CTAG uses the "audioset-best" CLAP checkpoint for audio under 10 s. — [paper](https://arxiv.org/html/2406.00294v1)
- Syntheon does parameter inference: "Given an audio sample, Syntheon infers the best parameter preset". It supports Vital, with Dexed in progress, and is Apache-2.0. — [README](https://github.com/gudgud96/syntheon), [LICENSE](https://raw.githubusercontent.com/gudgud96/syntheon/main/LICENSE)
- `preset-gen-vae` learns to program a Dexed FM synth from sound, using VAEs. — [README](https://github.com/gwendal-lv/preset-gen-vae)
- ISMIR 2025 best student paper: Hayes et al., inversion of Surge XT using a permutation-equivariant continuous normalizing flow. Equivalent parameter settings form several modes, which regression handles poorly. — [paper](https://arxiv.org/abs/2506.07199v1), [QMUL](https://www.seresearch.qmul.ac.uk/cfcs/news/5405/phd-student-ben-hayes-wins-best-student-paper-award/)
- Modulation discovery with differentiable DSP (WASPAA 2025) targets envelopes and LFOs, and the authors provide code and a VST. — [paper](https://arxiv.org/abs/2510.06204)
- DiffMoog is a differentiable modular synth with a sound-matching framework. — [paper](https://arxiv.org/html/2401.12570v1)
- Commercial proof that matching ships: Synplant 2's Genopatch builds a patch from any audio file. — [MusicRadar](https://musicradar.com/news/sonic-charge-synplant2-ai)
- LLM option: Qwen3-0.6B Q4_K_M is 396.7 MB under Apache-2.0, SmolLM2-135M Q4_K_M is 105.5 MB with an Apache-2.0 tag. — [Qwen3 GGUF](https://huggingface.co/unsloth/Qwen3-0.6B-GGUF), [LICENSE](https://huggingface.co/Qwen/Qwen3-0.6B/raw/main/LICENSE), [SmolLM2 GGUF](https://huggingface.co/bartowski/SmolLM2-135M-Instruct-GGUF)
- llama.cpp is MIT and its GBNF grammars "constrain model output", which can force valid recipe JSON. — [LICENSE](https://raw.githubusercontent.com/ggml-org/llama.cpp/master/LICENSE), [grammar guide](https://raw.githubusercontent.com/ggml-org/llama.cpp/master/grammars/README.md)

Feasibility under 100 MB (sizes in this list are engineering estimates unless cited above):
- Nearest-neighbour over hand-made features (MFCC, spectral shape, envelope): feasible, well under 1 MB, no ML.
- Nearest-neighbour over YAMNet or PANNs MobileNet embeddings: feasible at about 15 to 21 MB (cited).
- Same over quantized CLAP audio: feasible at 34.3 MB (cited).
- Preset or latent interpolation: feasible, with no model for presets. It only needs care with parameter types.
- Autoencoder preset map (2D map of the synth's parameter space, trained in-house): feasible, a small net that should be low MBs.
- Parameter inference trained on PH2D's own synth: feasible. A small convolutional net fits in tens of MB at most.
- CTAG-style text to recipe: 161 MB with quantized CLAP, so over 100 MB but under 500 MB. Audio-reference search alone is 34 MB.
- LLM recipe: 105.5 MB (SmolLM2) to 396.7 MB (Qwen3).

### Inferences
- CTAG's result is the strongest fact for this product. Listeners recognised its sounds nearly as often as AudioGen's and far more than AudioLDM's, using no generative audio model. The result is also an editable recipe, which fits a sound designer's tool and sidesteps most output-provenance risk.
- A native Rust synth renders candidates far cheaper than the JAX synth, so CLAP encoding will dominate. At population 25 and 300 iterations that is about 7,500 CLAP encodes, so GPU batching or a smaller search budget will be needed.
- Parameter inference needs training data, but a synth generates unlimited labelled data, so no third-party audio is needed. This is the cleanest legal position of any trained option. Multi-mode outputs (Hayes et al.) matter when several settings give the same sound.
- A small LLM has little to add beyond a curated prompt-to-template table plus CLAP search, unless it beats that in a measured test.

### Gaps
- The original InverSynth repo URL tried returned 404, so its licence is unverified. It should be treated as a paper only.
- The DiffMoog repo URL tried also returned 404, so its licence is unverified.
- Hayes et al. code is reported at benhayes.net/synth-perm in a search result; the page and its licence were not read.
- No published accuracy of tiny LLMs producing synth recipes was found.

## Training-data legal risk and output ownership

### Takeaway
Output ownership is granted by licence only "to the extent permitted by applicable law", and US guidance says prompts alone do not create authorship. Training-data risk differs sharply by model: Freesound-only CC mixes are the most defensible open option but still have attribution and retired-licence wrinkles. AudioSet and WavCaps-based models carry the worst provenance, even when their weights have an MIT tag.

### Cited Findings
- Stable Audio Open 1.0 data: 486,492 recordings; 472,618 from Freesound and the rest from FMA, "All audio files are licensed under CC0, CC BY, or CC Sampling+". Freesound split: 266,324 CC0, 194,840 CC-BY, 11,454 CC Sampling+. — [card](https://huggingface.co/stabilityai/stable-audio-open-1.0)
- That repo ships a 50.4 MB `freesound_dataset_attribution2.csv`. — [files](https://huggingface.co/stabilityai/stable-audio-open-1.0)
- Open Small's DiT was trained only on Freesound; the autoencoder on both sources. — [card](https://huggingface.co/stabilityai/stable-audio-open-small)
- Stable Audio 3 is described inconsistently: "All Stable Audio 3.0 models are trained on fully licensed data" in the [announcement](https://stability.ai/news-updates/meet-stable-audio-3-the-model-family-built-for-artistic-experimentation-with-open-weight-models), against "licensed and Creative Commons data" in the [paper](https://arxiv.org/abs/2605.17991).
- The Stable Audio 3 card lists 806,284 AudioSparx and 472,618 Freesound recordings. — [card](https://huggingface.co/stabilityai/stable-audio-3-small-sfx)
- CC Sampling+ 1.0 was retired on 2011-09-12. It excludes advertising and promotional uses except promotion of your own derivative, and requires crediting the original author. — [legal code](https://creativecommons.org/licenses/sampling+/1.0/legalcode)
- AudioSet: "The dataset is made available by Google Inc. under a Creative Commons Attribution 4.0 International (CC BY 4.0) license", and the page states no licence for the underlying YouTube videos. — [AudioSet](https://research.google.com/audioset/download.html)
- WavCaps, verbatim: "Only academic uses are allowed for WavCaps dataset." — [README](https://github.com/XinhaoMei/WavCaps)
- MeanAudio "Full" models were trained on AudioCaps, WavCaps, AudioSet, VGGSound and LP-MusicCaps, despite the MIT tag on the weights. — [README](https://github.com/xiquan-li/MeanAudio), [files](https://huggingface.co/AndreasXi/MeanAudio)
- AudioGen's data includes AudioSet, BBC sound effects, VGG-Sound, FSD50K and Sonniss Game Effects, among others. — [model card](https://raw.githubusercontent.com/facebookresearch/audiocraft/main/model_cards/AUDIOGEN_MODEL_CARD.md)
- Output, Stability: "As between You and Stability AI, You own any outputs generated from the Models or Derivative Works to the extent permitted by applicable law." — [Community License](https://stability.ai/community-license-agreement)
- US Copyright Office Part 2 (January 29, 2025): prompts alone do not give users sufficient control to be authors, though AI elements in a larger human-authored work do not undermine that larger work's protection. — [Reed Smith summary](https://www.reedsmith.com/de/perspectives/2025/02/copyright-office-anticipated-part-two-report-ai-addressing-copyrightability), [IPWatchdog](https://ipwatchdog.com/2025/01/29/part-two-copyright-office-ai-report-says-creative-prompting-doesnt-constitute-authorship/)
- UK Getty v Stability, [2025] EWHC 2863 (Ch): model weights were held not to be infringing copies, since they did not store Getty's works. An appeal was reported as possible. — [SCL](https://www.scl.org/high-court-issues-eagerly-awaited-judgment-in-getty-v-stability-ai-case/), [heise](https://heise.de/-11067232)
- Steam (January 2026) narrowed its disclosure to generative AI content that players consume, exempting behind-the-scenes efficiency tools. — [PC Gamer](https://www.pcgamer.com/software/ai/steam-updates-ai-disclosure-form-to-specify-that-its-focused-on-ai-generated-content-that-is-consumed-by-players-not-efficiency-tools-used-behind-the-scenes/)

### Inferences
- An MIT tag on weights covers only the publisher's rights in those weights. It cannot cure an academic-only condition in the training data. That is why MeanAudio and SoundCTM are rated NO in practice, despite tags that look permissive.
- Risk order, lowest first: procedural synth with no ML output; synth judged by CLAP (CLAP is not in the audio path); Stability Small-SFX; anything trained on AudioSet or WavCaps.
- Game developers using raw generated audio may hold thin or no copyright in it, which matters for asset exclusivity. Sounds edited or layered by a designer are in a stronger position. PH2D could keep a record of how each asset was made, to help with Steam disclosure.
- AI-generated SFX shipped in a game very likely count as player-consumed content on Steam. Synth-recipe output judged by CLAP is arguably different, but no source addresses this.

### Gaps
- Primary text of the US Copyright Office report was not read; law-firm summaries were used.
- Germany's GEMA v OpenAI (memorisation) was searched for and not returned, so it is not covered.
- No source addresses whether CC-BY attribution extends to model outputs. No ruling on audio models specifically was found.
- Valve's own form wording on sound effects was not read; coverage conflicts on enforcement.
- Memorisation or leakage testing for Stability's SFX models was not found.

## Inference from Rust: ONNX path and runtime needs

### Takeaway
`ort` is the practical choice for the optional Stability pack and any large model, with the caveat of a native dependency and a silent CPU fallback. `tract` suits small CPU models and is already in PH2D, but at an old version. `burn` is the only one aligned with PH2D's Vulkan/WebGPU direction, but transformer-sized imports are unproven. I found no published test of these audio models on any Rust runtime, so it has to be measured.

### Cited Findings
- `ort` 2.0.0-rc.15 wraps ONNX Runtime 1.31 (version from `cargo search` on 2026-11-05). — [README](https://github.com/pykeio/ort)
- `ort` execution providers include CUDA, TensorRT, DirectML, CoreML and WebGPU, and "binaries for CUDA ≥ 13.2" are provided. — [docs source](https://raw.githubusercontent.com/pykeio/ort/main/docs/content/perf/execution-providers.mdx)
- `ort` WebGPU EP, verbatim: "**experimental** and may produce incorrect results/crashes". — [docs source](https://raw.githubusercontent.com/pykeio/ort/main/docs/content/perf/execution-providers.mdx)
- `ort` warns it "will silently fail and fall back to executing on the CPU if all execution providers fail to register". — [docs source](https://raw.githubusercontent.com/pykeio/ort/main/docs/content/perf/execution-providers.mdx)
- `tract-onnx` latest is 0.23.10. Backends: CPU, Metal, CUDA, WASM (no Vulkan or AMD backend listed). — [README](https://github.com/sonos/tract)
- tract's ONNX loader registers DFT, STFT, MelWeightMatrix and Hann, Hamming and Blackman windows. — [source](https://raw.githubusercontent.com/sonos/tract/main/onnx/src/ops/fft.rs)
- PH2D is pinned to tract 0.19.16 through vendored DeepFilterNet (`libDF` v0.5.6), and uses wgpu 30.0.1. — `Cargo.lock` and `crates/ph2d-audio-ml/Cargo.toml` in this repo.
- Burn 0.23.0-pre.1 has CubeCL backends for CUDA, ROCm, Metal, Vulkan and WebGPU. `burn-onnx` imports ONNX "as native Rust code". — [README](https://github.com/tracel-ai/burn)
- burn-onnx's table lists STFT, DFT, MelWeightMatrix, HannWindow, Einsum, LayerNormalization, Loop and Scan as supported. — [ops table](https://raw.githubusercontent.com/tracel-ai/burn-onnx/main/SUPPORTED-ONNX-OPS.md)
- Candle 0.11.1 supports CUDA and Metal and includes EnCodec and Whisper examples, plus a separate `candle-onnx` crate. — [README](https://github.com/huggingface/candle)
- An independent MNIST test found candle failing on `auto_pad: SAME_UPPER` and burn producing wrong results, with ort at about 0.35 ms and tract at about 0.90 ms. It is dated (2024) and narrow, so it is weak evidence today. — [Rust Tokyo 2024 slides](https://speakerdeck.com/emergent/rust-tokyo-2024-comparing-onnx-crates)

Export status per model:

| Model | What exists | Best Rust route |
|---|---|---|
| Stable Audio 3 Small-SFX | Official ONNX (DiT, SAME-S, T5Gemma), TFLite, TensorRT — [files](https://huggingface.co/stabilityai/stable-audio-3-optimized) | `ort` |
| Stable Audio 3 Medium | Official ONNX plus a community CPU port, about 5.8 GB DiT — [port](https://huggingface.co/innermost47/stable-audio-open-3-medium-onnx) | `ort`, too large |
| Stable Audio Open Small | Official LiteRT only in repo; no ONNX file listed — [files](https://huggingface.co/stabilityai/stable-audio-open-small) | Own ONNX export needed |
| CLAP HTSAT | Community ONNX, split audio and text — [files](https://huggingface.co/Xenova/clap-htsat-unfused) | `ort` first, try `tract` and `burn` |
| RAVE | `onnx` config exists; licence blocks it anyway — [README](https://github.com/acids-ircam/RAVE) | Not applicable |
| DeepFilterNet | Already running on tract 0.19 in PH2D | Keep |
| YAMNet, PANNs, Vocos, BigVGAN, DAC | No official ONNX confirmed in what was read | Own export and test |

### Inferences
- Measurement plan, one batch per the project rule: on the workstation and one weaker machine, run Small-SFX and quantized CLAP through `ort` on CPU, CUDA and DirectML/WebGPU, intercalated in the same process, and take the minimum. Also try CLAP on `tract` 0.23 and `burn`. This replaces every missing x86 and mid-range GPU number in these notes.
- `ort` must assert which provider actually registered, or a "GPU" feature can silently run on CPU.
- `tract` 0.19 and 0.23 can coexist as separate Cargo dependencies, but that is two copies in the binary. Keeping new ML in its own crate behind a feature mirrors the existing ADR-0123 boundary.
- Burn's own WGPU path may conflict with PH2D's wgpu 30, since burn goes through CubeCL. The versions must be checked before committing.
- Generated audio from a GPU model is unlikely to be bit-identical across machines. Given the owner's same-result-on-any-machine rule for the Painter, generated SFX should be stored as an asset with seed and prompt, not re-generated on load. Synth recipes do not have this problem.

### Gaps
- No published Rust-runtime benchmark exists for any audio model here. Operator tables show support, not correct results at transformer scale.
- ONNX has STFT and DFT operators, but Vocos' inverse-STFT export path was not tested or sourced.
- Whether SA3's exported graphs run under `tract` or `burn` is unknown.
- Burn's current wgpu dependency version was not confirmed (its manifest pins CubeCL by git revision).
- The ONNX Runtime binary's size contribution to the installer was not measured.
