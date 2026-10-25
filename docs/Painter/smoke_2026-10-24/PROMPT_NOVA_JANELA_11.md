# Prompt da janela seguinte — line/PainterWatercolor, 25/10 (depois do smoke da janela 10)

```
Linha line/PainterWatercolor (Modo L), continuação de 25/10.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md e docs/Painter/smoke_2026-10-24/PROMPT_NOVA_JANELA_11.md inteiro. Delegue leitura e verificação.
```

⛔ Leis desta janela: **sem bateria por crate nem suíte por commit** (dono 25/10) — só testes por NOME +
clippy/fmt das crates tocadas; handoff/prompt só DEPOIS do smoke do dono.

## O que a janela 10 fez (commits `9dddc903e3`, `4ce1155a83`, `9d8514727b`; handoff em [`HANDOFF_line_PainterWatercolor_2026-10-25_JANELA10.md`](../handoffs/HANDOFF_line_PainterWatercolor_2026-10-25_JANELA10.md))

Tudo aprovado no smoke do dono («Smoke OK» / «ok»).

1. **Queda de FPS ao mesclar medida e curada**: a cena do dono (2048², Wet Flat, Size 122) reproduzida sem janela (`sonda_fps_cena_do_dono`). O esticão do Wet bloom (`molhado_estica`)
   passou a duas fases, com pouso paralelo idêntico ao bit; `Move` ao mesclar 11,0 → ~6–7,6 ms, pior 16,2 → ~8,8–11,4. O preset do Wet agora leva `PressureSCurve`, `PressureJitter` e `SpeedLag`.
2. **Composite Brush só no Digital** (cartão e traço, uma porta: `PaintMedia::offers_composite`).
3. **Saiu o «Basic brush» do painel** (duplicava o navegador de presets); o censo de controlos encolheu duas linhas.
4. **Auditoria dos controlos do Wet**: os principais leem e decidem; refutados «Press não grava» e «Speed lag não lido». Tabela entregue ao dono.
5. Perguntas do dono respondidas: baixar a taxa da sim não ajuda; «pigmento fino + água grossa» é NO-GO (≤ ~2 %, traz de volta a borda em degrau).

## Abertos

1. Decisão do dono pendente: forma fechada no lugar do Newton do Rebelle (muda bits) para encurtar `a_largura_e_a_prometida`.
2. Decisão do dono pendente: Borracha da Watercolor que mantém a sessão — (a) buraco limpo ou (b) levantar com mata-borrão
   (mapa do modelo da sessão no handoff da janela 8, §5.3).
3. **NOVO**: pouso da janela do transfer segue serial (~3,7 ms/`Move` ao mesclar a 2048²); só um arrasto que leia um instantâneo o paraleliza (muda bits ⇒ decisão do dono).
   Alternativa já ao alcance do dono: Grid Size 2.
4. **NOVO**: rota órfã `PAINTER_BRUSH_PRESET` / `apply_brush_preset` na ferramenta — avaliar se só resta a porta das macros.
5. Da janela 7: Accumulate ON + passagem translúcida (Flow 0,3) sem Continuous escurece a cauda; ressalto ~+10 % na entrada da janela da cauda sem Continuous;
   S-curve, Light scatter (não achado com esse nome), canal do «Reset to brush preset»; defeito 5 do Pen (Tilt smooth/Rotation); `ExtDryBrush` (escondido e lido);
   decisão 2 do dono (escondidos do Digital sem linha).
6. Da janela 4: macros «mundo» leem a sprite sem a tinta por assar; Borracha do Wet × Strength; preview da Borracha no Wet; Texture Eraser à Strength 1; a shell a 23 linhas do tecto.
7. Não verificados no Wet: Color, Solid, Type, «On empty».

Retirados: «Wet presets não gravam Press» (refutado), «Speed lag» (gravado), «filtro `borracha_por_meio`» (módulo `tool::paint::tests::borracha_por_meio_tests`,
testes `o_apagado_na_aguada_molhada_fica_apagado`, `o_follow_sem_lado_nao_muda_a_agua`, sonda `sonda_borracha_por_meio`).

## Próximo passo

Um só bloco (§0.10): os abertos acima que não dependem do dono (4, 5, 6, 7), agrupados por família, uma rodada de medição, gates por nome; as decisões 1–3 aguardam o dono.

OU integrar a linha no main, se o dono mandar (só por ordem dele, `/pd-integracao`; a suíte completa é do integrador).

## Como fechar

1. Um plano, uma rodada de medição, gates por nome, mutação das leis novas.
2. Só testes por nome + `clippy -D warnings` e `fmt` das crates tocadas. Sem suíte.
3. Smoke ao dono em passos numerados e sem jargão (comando com `cd`, onde clicar, o que acontece, como saber que falhou). Só depois
   do veredito: handoff e prompt.
