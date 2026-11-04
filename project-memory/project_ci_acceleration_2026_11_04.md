---
name: project-ci-acceleration-2026-11-04
description: "04/11: CI 60–101 → 27–30 min com cache; semente após o push; noite.yml; teto de CPU 50 vs 100 é decisão do DONO"
metadata:
  node_type: memory
  type: project
---

- O `rust-cache` põe no hash TODAS as toolchains instaladas; a action `cache-estavel` desinstala tudo menos o pin e só SALVA em `schedule`/`workflow_dispatch`. ⇒ a 1.ª corrida no `main` é FRIA: quem faz ship dispara `gh workflow run spike.yml --ref main` para semear.
- Corrida inteira 60–101 min → 27–30 min com cache (runs 41808153285, 41808157477). Sem cache nenhum: 43–58 min, o timeout 90 fica.
- `noite.yml` (cron `30 6 * * *`, 4 fatias, só lê a cache) corre a workspace inteira; o nível lento fica na integração até a noite ter histórico.
- A suíte começa pelo pré-voo estático (`PH2D_PREVOO=0` salta); vermelho de uma linha só = `falhas` + `b2`, nunca outra suíte inteira.
- **Pendente do DONO:** teto de CPU 50 % → 100 % na suíte da integração sem smoke dele: suíte inteira 1 308 → 831 s, verde (A×B intercalado 448,3 → 295,0 s). Nunca por omissão; `PH2D_CPU_PCT` reescreve a unidade da fatia, não se mede com outro agente na worktree.
- Recusados medidos: `RAYON_NUM_THREADS=4` (+33 %), hakari (0 · 0 · 0 unidades), sccache (só bucket fora do GitHub).
- Detalhe: `docs/DevOps/PESQUISA_CI_PROJETOS_GRANDES_2026-11-04.md` §Resultado.
