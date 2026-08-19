# Watercolor / fluid / wash — REMOVIDOS (histórico) — estado do §5 em 2026-08-18

> **Arquivo histórico.** Este é o texto do `CLAUDE.md` §5 **verbatim**, como estava em
> 2026-08-18, antes de o §5 voltar a ser um roteador. O **estado vivo** (o que está aberto,
> o handoff corrente) fica no [`CLAUDE.md`](../../../CLAUDE.md) §5; o **mecanismo** fica nos
> handoffs que este texto cita. Nada aqui foi editado — só recortado.

---

- **Watercolor/fluid/wash — REMOVIDOS** ([ADR-0096](../../architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md), Enio 2026-06-15): toda a simulação de aquarela (crate `ph2d-painter-wash`, sessões GPU `painter_wash_gpu`/`painter_canvas_gpu`, `wash_pipeline`/settle/bordas-molhadas) foi **deletada** e o canvas voltou a CPU-residente. Backups intactos em `backups/wash_2026-06-14` + `backups/watercolor_v2_2026-06-12`. SCHEMA_VERSION 2→3 (quebra dura de save, postcard posicional). Preservados: layer-stack + compositor GPU + efeitos/ajustes + o **brush default (`apply_stamps_wash` = blend-mode, NÃO sim)**. ADR-0096 supersede ADR-0085..0095 (mantidos como histórico). **Pivot:** mixer-brush (Procreate-style) + Kubelka–Munk/Mixbox, não fluido. **NOTA (2026-06-20):** o sucessor (Brush Engine, [ADR-0097](../../architecture/decisions/0097-brush-engine-procreate-parity-cpu-first-dab-pipeline.md)) **também foi removido depois** — toda a pintura saiu por [ADR-0099](../../architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md); o que sobra é o host de Layers + Efeitos (ver entrada acima). `docs/Novo Painter/` é histórico.
