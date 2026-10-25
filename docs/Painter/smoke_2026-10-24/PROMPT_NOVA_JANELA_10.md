# Prompt da janela seguinte — line/PainterWatercolor, 25/10 (depois do smoke da janela 9)

```
Linha line/PainterWatercolor (Modo L), continuação de 25/10.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md e docs/Painter/smoke_2026-10-24/PROMPT_NOVA_JANELA_10.md inteiro. Delegue leitura e verificação.
```

⛔ Leis desta janela: **sem bateria por crate nem suíte por commit** (dono 25/10) — só testes por NOME +
clippy/fmt das crates tocadas; handoff/prompt só DEPOIS do smoke do dono.

## O que a janela 9 fez (commit `5291edbc75`; handoff em [`HANDOFF_line_PainterWatercolor_2026-10-25_JANELA9.md`](../handoffs/HANDOFF_line_PainterWatercolor_2026-10-25_JANELA9.md))

1. **Wet bloom = o tubo do Rebelle** (`crates/ph2d-wet-paint/src/trail/molhado.rs`, `trail/transfer.rs`): 3 mecanismos achados (aresta reta do disco; levantar/empurrar
   escrito fora do sujo; empurrão O(raio³) 212 ms/dab). Oráculo Rebelle em `oraculo_janela9/` (tubo 1,47–1,53×, largura constante). Pior quadro ao mesclar 215 → 11,9 ms
   (tela 1200×720, pincel 90/72). Tip pickup inocente. Gates novos; clippy/fmt limpos.
2. **Smoke do dono**: retângulos/manchas OK, resolvido. MAS «ainda severa queda de FPS na mistura de tinta molhada» na cena dele (Wet Flat, grade 2048², Size 122,
   Wet bloom 0,50, Tip pickup 0,46; fotos em `fotos_janela9/`) — a sonda da janela 9 NÃO a reproduz (1200×720, pincel de fábrica): causa por medir.
   Ordem nova do dono: «Auditoria geral dos principais parâmetros, suas ações e performance.»
3. Nome velho: o filtro `borracha_por_meio` casa ZERO testes (achar o atual).

## Abertos

1. **NOVO**: queda de FPS ao mesclar na cena do dono (por medir) + AUDITORIA GERAL dos parâmetros do Wet Paint (ação, leitor, custo).
2. Decisão do dono pendente: forma fechada no lugar do Newton do Rebelle (muda bits) para encurtar `a_largura_e_a_prometida`.
3. Decisão do dono pendente: Borracha da Watercolor que mantém a sessão — (a) buraco limpo ou (b) levantar com mata-borrão
   (mapa do modelo da sessão no handoff da janela 8, §5.3).
4. Da janela 7: Accumulate ON + passagem translúcida (Flow 0,3) sem Continuous escurece a cauda; ressalto ~+10 % na entrada da
   janela da cauda sem Continuous; Wet presets não gravam Press; S-curve, Light scatter, Speed lag; canal do «Reset to brush
   preset»; defeito 5 do Pen (Tilt smooth/Rotation); Dry brush (`ExtDryBrush`) knob morto; decisão 2 do dono (escondidos do
   Digital sem linha).
5. Da janela 4: macros «mundo» leem a sprite sem a tinta por assar; Borracha do Wet × Strength; preview da Borracha no Wet;
   Texture Eraser à Strength 1; a shell a 23 linhas do tecto.

## Próximo passo

Um só bloco (§0.10):
1. **Reproduzir a cena do dono sem janela** — preset Wet Flat, grade de fluido/fluxo 2048², Size 122, os knobs acima, amarelo por cima de uma segunda cor ainda
   molhada — e cronometrar por etapa E por caminho do dab (`tip_rows`, `molhado_estica`, `molhado_levanta`, depósito, transfer, etapas da sim, composite), misturando
   × papel limpo, intercalado, o MÍNIMO ([MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md)). Curar o que domina e medir de novo.
2. **A AUDITORIA do dono**: para cada parâmetro principal do cartão e do painel do Wet Paint (lista acima), uma tabela: o que escreve · quem lê · o leitor DECIDE?
   (controlo morto, CLAUDE.md §5.0) · efeito visível · custo na cena do dono. Curar no mesmo bloco os controlos mortos ou enganosos. Entregar ao dono a tabela em
   palavras simples, sem jargão.
3. Decisões pendentes do dono, inalteradas: forma fechada do Newton; Borracha da Watercolor (a)/(b).

OU integrar a linha no main, se o dono mandar (só por ordem dele, `/pd-integracao`; a suíte completa é do integrador).

## Como fechar

1. Um plano, uma rodada de medição, gates por nome, mutação das leis novas.
2. Só testes por nome + `clippy -D warnings` e `fmt` das crates tocadas. Sem suíte.
3. Smoke ao dono em passos numerados e sem jargão (comando com `cd`, onde clicar, o que acontece, como saber que falhou). Só depois
   do veredito: handoff e prompt.
