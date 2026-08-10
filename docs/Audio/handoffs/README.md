# `Audio` — handoffs (registro cronológico de sessão)

> **O que é esta pasta:** o registro de **como** o módulo foi construído — um arquivo por
> sessão de linha. O **pensamento** do módulo (planos, pesquisas, `BUGS_*`) fica **um nível
> acima**, em [`docs/Audio/`](..).
>
> ⚠️ **Isto NÃO é o estado atual do módulo.** O estado vivo é o **[`CLAUDE.md §5`](../../../CLAUDE.md)**;
> um handoff descreve o mundo **no dia em que foi escrito** e não é atualizado depois. Use-os
> para responder *"por que isto ficou assim?"* — nunca para decidir a próxima ação.

**14 handoffs** · **5** citados pelo CLAUDE.md §5 (marcados **◆** — são os que a
§5 aponta como o detalhe de mecanismo de uma integração).

| Data | | Arquivo | Papel | Assunto |
|---|---|---|---|---|
| — |  | [HANDOFF_INTEGRACAO_line_audio.md](HANDOFF_INTEGRACAO_line_audio.md) | integração | Handoff de integração — line/audio (DIRETRIZ §1.5.9) |
| — | ◆ | [HANDOFF_audio_line_continuation.md](HANDOFF_audio_line_continuation.md) | trabalho | HANDOFF DE CONTINUAÇÃO — linha line/audio |
| — | ◆ | [HANDOFF_audio_module.md](HANDOFF_audio_module.md) | trabalho | Módulo de Áudio (line/audio) |
| — |  | [HANDOFF_audio_variation_impl.md](HANDOFF_audio_variation_impl.md) | trabalho | HANDOFF DE INTEGRAÇÃO — linha line/audio (W6 + W4 rack) |
| — |  | [HANDOFF_audio_w4_integracao.md](HANDOFF_audio_w4_integracao.md) | integração | HANDOFF DE INTEGRAÇÃO — linha line/audio (W4 fechado) |
| — | ◆ | [HANDOFF_audio_w5_espectral.md](HANDOFF_audio_w5_espectral.md) | trabalho | Áudio W5 (Espectral) · linha line/audio |
| — |  | [HANDOFF_line_audio_w7_ml_denoise.md](HANDOFF_line_audio_w7_ml_denoise.md) | trabalho | line/audio · W7: denoise ML nativo (DeepFilterNet via tract) |
| — | ◆ | [HANDOFF_line_audio_w7_ml_denoise_CLOSURE.md](HANDOFF_line_audio_w7_ml_denoise_CLOSURE.md) | fechamento | HANDOFF (CLOSURE) — line/audio · W7: AI Denoise (DeepFilterNet via tract) |
| 2026-07-13 | ◆ | [HANDOFF_INTEGRACAO_line_audio_2026-07-13.md](HANDOFF_INTEGRACAO_line_audio_2026-07-13.md) | integração | Handoff de integração — line/audio (DIRETRIZ §1.5.9) |
| 2026-07-13 |  | [HANDOFF_line_audio_continuacao_2026-07-13.md](HANDOFF_line_audio_continuacao_2026-07-13.md) | continuação | continuação da line/audio (para o próximo agente) |
| 2026-07-14 |  | [HANDOFF_line_audio_continuacao_2026-07-14.md](HANDOFF_line_audio_continuacao_2026-07-14.md) | continuação | Handoff de continuação — line/audio (pós-integração de 2026-07-14) |
| 2026-07-16 |  | [HANDOFF_INTEGRACAO_line_audio_2026-07-16.md](HANDOFF_INTEGRACAO_line_audio_2026-07-16.md) | integração | Handoff de integração — line/audio → main (DIRETRIZ §1.5.9) |
| 2026-07-16 |  | [HANDOFF_line_audio_pricing_2026-07-16.md](HANDOFF_line_audio_pricing_2026-07-16.md) | trabalho | line/audio · Precificação fora do frame de edição (ADR-0125) |
| 2026-07-16 |  | [HANDOFF_line_audio_range_edit_2026-07-16.md](HANDOFF_line_audio_range_edit_2026-07-16.md) | trabalho | line/audio: a edição por-intervalo virou O(seleção) (ADR-0124) |

---
*Índice gerado na arrumação de 2026-08-10 (DIRETRIZ §1.5.9). Handoff novo entra aqui, não na
raiz de `docs/`.*
