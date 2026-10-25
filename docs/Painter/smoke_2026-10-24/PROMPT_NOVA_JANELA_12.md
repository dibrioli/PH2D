# Prompt da janela seguinte — line/PainterWatercolor, 25/10 (depois do smoke da janela 11)

```
Linha line/PainterWatercolor (Modo L), continuação de 25/10.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, DIRETRIZ §3.B e docs/Painter/smoke_2026-10-24/PROMPT_NOVA_JANELA_12.md inteiro. Delegue leitura e verificação.
```

⛔ Leis desta janela: **sem bateria por crate nem suíte por commit** (dono 25/10) — só testes por NOME +
clippy/fmt das crates tocadas; handoff/prompt só DEPOIS do smoke do dono.

## O que a janela 11 fez (18 commits `ec248aedbb`..`9b32db7a3d`; handoff em [`HANDOFF_line_PainterWatercolor_2026-10-25_JANELA11.md`](../handoffs/HANDOFF_line_PainterWatercolor_2026-10-25_JANELA11.md))

Aprovado no smoke do dono («Smoke OK»). Fechou os abertos 4–7 da janela 10: rota órfã do preset, «Reset to brush preset», defeito 5 do Pen, `ExtDryBrush` (lei viva, fixada), Wet Color/Solid/Type/On empty (sem defeito);
cauda do Digital sob Accumulate+Flow (146 → 72); macros «mundo» levam a tinta por assar; Borracha do Wet obedece ao Flow, preview certo; Texture Eraser à Strength 1; 11 vermelhos herdados curados
(`BrushSpec::fora_do_digital`). Risco: macros antigas que escolheram «Basic brush» reproduzem um no-op.

## NOVO — pedidos do dono no smoke de 25/10 (bloco principal desta janela)

> «Veja para wet paint que é um modo especial de pintura, as tools como Smear, blend, wet, dry, blow estão no painel do inspector quando deveriam estar na barra de ferramentas da esquerda. Smear tem água em seu efeito mas não encontrei onde ajustar a água de smear. É necessário configurar o inspector para cada tipo de tool, escondendo o que não faz parte do uso. Seria interessante criar alguns presets para Smear e Blend.»

- **N1** — as ferramentas do Wet Paint (Smear, Blend, Wet, Dry, Blow; ache o conjunto exato, p.ex. `pick_wet_tool` em `wetpaint_settings.rs`) saem do Inspector e vão à barra da esquerda enquanto o modo é Wet Paint.
  O Rebelle é a referência do dono: oráculo que se CORRE (`docs/_ComoInvestigarApps/`), nunca fonte que se lê.
- **N2** — expor a água do Smear. Ache se há knob escondido; controlo morto: o painel escreve onde · quem lê · o leitor DECIDE?
- **N3** — Inspector por ferramenta: mostrar só os controlos que o LEITOR da ferramenta usa (derive dos leitores; gate como o censo de controlos; cuidado com o controlo que mente).
- **N4** — presets de fábrica para Smear e Blend (ficheiros de preset como os de fábrica existentes; medir cada um contra o Rebelle se possível).

Um só bloco (§0.10): plano único, uma rodada de medição, gates por nome. UI por tokens/i18n (HR-15); a shell com folga ZERO (a cura é mover para `ph2d-app-painter`). Painel/chrome: DIRETRIZ §3.B.

## Abertos (dependem do dono ou menores)

1. Decisões do dono pendentes 1–3 (do PROMPT_11) — **o dono não se lembra das escolhas: re-explicar em linguagem simples**: (1) forma fechada no lugar do Newton do Rebelle (muda bits) para encurtar
   `a_largura_e_a_prometida`; (2) Borracha da Watercolor que mantém a sessão: (a) buraco limpo ou (b) levantar com mata-borrão (handoff da janela 8, §5.3); (3) arrasto do transfer lendo um instantâneo (paraleliza o pouso, muda bits) ou Grid Size 2.
2. Decisão 2 da janela 7: escondidos do Digital sem linha — `pigment_espectral`, `passo_pela_pressao`, `borda_px`, `tecto_por_dab`, `sub_passo_fino`, `particle_max_px`.
3. Da janela 4: Borracha do Wet sobre tinta seca ~1,9× mais larga que a do Digital.
4. Resíduos da cauda do Digital: ondulação (F,T) ±5; mergulho −3 após a entrada (F,F); pincéis de fábrica Accumulate+Flow<1+taper com cauda mais clara.

## Próximo passo

N1–N4 em bloco (acima). OU integrar a linha no main, se o dono mandar (só por ordem dele, `/pd-integracao`; a suíte completa é do integrador).

## Como fechar

1. Um plano, uma rodada de medição, gates por nome, mutação das leis novas.
2. Só testes por nome + `clippy -D warnings` e `fmt` das crates tocadas. Sem suíte.
3. Smoke ao dono em passos numerados e sem jargão (comando com `cd`, onde clicar, o que acontece, como saber que falhou). Só depois do veredito: handoff e prompt.
