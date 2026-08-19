# `plans` — planos de fan-out (multi-módulo)

> **Gerado por `bash scripts/doc-index.sh` — não edite à mão.** Uma lista mantida à
> mão envelhece na primeira semana; esta é derivada do primeiro `# ` de cada arquivo.
>
> Planos que **atravessam módulos** — as waves de nós, de imageio, de compressão de textura, de coerência cromática. Planos de UM módulo moram na pasta do módulo (`docs/<Módulo>/NN_plano_*.md`), não aqui.
>
> ⚠️ Um plano descreve o que se **pretendia** fazer. O que de facto ficou está no handoff da wave e no `CLAUDE.md §5` — vários destes têm waves fechadas e fan-out ainda aberto, e o plano não é atualizado quando isso muda.
>
> ⚠️ **Isto NÃO é o estado atual do módulo.** O estado vivo é o **[`CLAUDE.md §5`](../../CLAUDE.md)**;
> um doc descreve o mundo **no dia em que foi escrito** e não é atualizado depois. Use-os
> para responder *"por que isto ficou assim?"* — nunca para decidir a próxima ação.

**9 arquivos** · **2** citados pelo `CLAUDE.md` (marcados **◆**).

| | Arquivo | Papel | Assunto |
|---|---|---|---|
|   | [2026-05-color-eq-coerencia-cromatica.md](2026-05-color-eq-coerencia-cromatica.md) | — | Plano — Coerência cromática do tool Color Equalization |
|   | [2026-05-imageio-waves.md](2026-05-imageio-waves.md) | — | Plano de waves — Image I/O (neck → freeze → fan-out) |
| ◆ | [2026-05-node-waves.md](2026-05-node-waves.md) | — | Plano de waves — sistema de nós node-centric (neck → freeze → fan-out) |
|   | [2026-05-texture-compression-waves.md](2026-05-texture-compression-waves.md) | — | Plano de waves — Cooked Texture Compression Pipeline (ADR-0055) |
|   | [2026-05-ui-source-of-truth.md](2026-05-ui-source-of-truth.md) | — | Plano — UI single source of truth: Widget Gallery + Inspector + Hierarchy |
| ◆ | [2026-05-wave-11-carry-overs.md](2026-05-wave-11-carry-overs.md) | — | Wave 11 — Carry-overs from Wave 10 closure |
|   | [2026-06-14-wash-gpu-resident.md](2026-06-14-wash-gpu-resident.md) | — | Plano — Wash GPU-residente (reimplementação simplificada, padrão-ouro) |
|   | [2026-06-20-blindagem-implementacao.md](2026-06-20-blindagem-implementacao.md) | — | Plano de Blindagem da Implementação — PH2D |
|   | [2026-07-gpu-resident-node-pipeline.md](2026-07-gpu-resident-node-pipeline.md) | — | Plano — Motor de nós GPU-resident (o "animar milhões de cópias") |

---

⚠️ Um `Papel` `—` é um **achado**, não um defeito deste índice: é um doc cujo próprio
nome não diz o que ele é. Um arquivo **sem** ◆ não é lixo — é um doc que o roteador
(`CLAUDE.md`) não alcança, e essa era exactamente a medição que criou este índice.

