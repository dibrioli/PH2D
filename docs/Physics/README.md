# `Physics` — índice do módulo

> **Gerado por `bash scripts/doc-index.sh` — não edite à mão.** Uma lista mantida à
> mão envelhece na primeira semana; esta é derivada do primeiro `# ` de cada arquivo.
>
> O **pensamento** do módulo de Física: o plano de waves (que é o *tracker* das waves), a visão, os planos por família (joints, IK, polia, player de plataforma) e a auditoria contra as engines de referência. O registro de **como** foi construído fica em [`handoffs/`](handoffs/README.md).
>
> ⚠️ **Isto NÃO é o estado atual do módulo.** O estado vivo é o **[`CLAUDE.md §5`](../../CLAUDE.md)**;
> um doc descreve o mundo **no dia em que foi escrito** e não é atualizado depois. Use-os
> para responder *"por que isto ficou assim?"* — nunca para decidir a próxima ação.

**13 arquivos** · **3** citados pelo `CLAUDE.md` (marcados **◆**).

| # | | Arquivo | Papel | Assunto |
|---|---|---|---|---|
| 00 | ◆ | [00_plano_waves.md](00_plano_waves.md) | plano | 00 · Plano de waves — o motor de física global (`line/physics`) |
| 01 |   | [01_visao.md](01_visao.md) | porta de entrada | 01 · Visão — O motor de física global do PH2D (1 página) |
| 02 |   | [02_plano_joints_ui_authoring.md](02_plano_joints_ui_authoring.md) | plano | 02 — Joints: UI, autoria de âncoras e os tipos que faltam (plano pós-pesquisa) |
| 03 |   | [03_plano_ik.md](03_plano_ik.md) | plano | 03 — IK multibody: posar arrastando a ponta (W-IK) |
| 03 |   | [03_plano_polia.md](03_plano_polia.md) | plano | A POLIA — plano de redesenho |
| 04 |   | [04_plano_fk_e_modos_de_joint.md](04_plano_fk_e_modos_de_joint.md) | plano | 04 — FK + os cinco modos de joint (W-FK, W-JointTools) |
| 05 |   | [05_pesquisa_player_plataforma.md](05_pesquisa_player_plataforma.md) | pesquisa | Pesquisa — o PLAYER DE PLATAFORMA sobre um corpo Dynamic |
| 06 |   | [06_plano_player_plataforma.md](06_plano_player_plataforma.md) | plano | Plano — o PLAYER DE PLATAFORMA (Dynamic) |
| 07 |   | [07_plano_player_kinematico.md](07_plano_player_kinematico.md) | plano | Plano — o PLAYER CINEMÁTICO (o 2º modo) |
| 08 |   | [08_plano_features_faltantes.md](08_plano_features_faltantes.md) | plano | Plano 08 — o que falta ao PLAYER, medido contra o catálogo (2026-08-10) |
| 09 | ◆ | [09_auditoria_engines.md](09_auditoria_engines.md) | auditoria | Auditoria 09 — o Player medido contra Unity, Godot, Unreal e o tnua (2026-08-12) |
| 10 |   | [10_plano_fila_da_auditoria.md](10_plano_fila_da_auditoria.md) | plano | Plano 10 — a fila da auditoria 09, desenhada (2026-08-12) |
| — | ◆ | [BUGS_physics.md](BUGS_physics.md) | bugs | Bugs do módulo Physics — registro + soluções |

**Subpastas:** [`handoffs/`](handoffs/README.md)

---

⚠️ Um `Papel` `—` é um **achado**, não um defeito deste índice: é um doc cujo próprio
nome não diz o que ele é. Um arquivo **sem** ◆ não é lixo — é um doc que o roteador
(`CLAUDE.md`) não alcança, e essa era exactamente a medição que criou este índice.

