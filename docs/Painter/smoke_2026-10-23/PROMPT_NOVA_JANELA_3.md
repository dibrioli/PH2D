# Prompt da janela seguinte — line/PainterWatercolor, as FERRAMENTAS do Painter (23/10)

> Estado: a biblioteca de pincéis (`0a87b88c60`), a janela de confirmar o Delete, o Stars pequeno sem degraus
> e a secção Jitter (`803dc5b694`) estão na linha, à espera do smoke do dono. A linha NÃO está integrada.
> Handoff: `docs/Painter/handoffs/HANDOFF_line_PainterWatercolor_2026-10-23_BIBLIOTECA.md`.

```
Linha line/PainterWatercolor (Modo L), continuação de 23/10.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, o handoff
docs/Painter/handoffs/HANDOFF_line_PainterWatercolor_2026-10-23_BIBLIOTECA.md (§3b e §4) e
`git log --oneline -8`. Delegue leitura e verificação a agentes (explorador, verificador, mutacao); a janela
principal decide e desenha. `git worktree list` antes de tocar a fundação (crates/ph2d-editor-core): se a
line/UIUX estiver aberta, os ficheiros do chrome são dela também — append-only, e diga no handoff.

⛔ REGRA DO DONO (23/10): a suíte COMPLETA só antes da INTEGRAÇÃO. Na linha, mesmo no fecho, só testes por
NOME (`bash scripts/cargo-test-narrow.sh <crate> <nome>`) e provas de mutação. Clippy -D warnings só das crates
tocadas, 1× no fim.

MISSÃO (palavras do dono): «Atualmente as tools do módulo painter estão na barra superior. Mas conforme foi
feito no módulo vector vamos colocar na barra lateral esquerda, com a mesma decoração do módulo vector. Outra
coisa: até agora o inspector do painter não se adequa perfeitamente à tool selecionada e várias opções do
pincel de pintura aparecem e não são usadas em outras tools. Vamos corrigir isso. Mais uma coisa: as tools
Blur, Eraser e Smear podem funcionar muito bem (e provavelmente já funcionam) com seções como Grain. Mantenha
e se necessário aprimore isso.»

As quatro tarefas são da MESMA família (qual ferramenta está na mão → o que a UI mostra e o que o motor lê):
um plano, um lote de portões (CLAUDE.md §0.10).

1. BARRA LATERAL ESQUERDA. As ferramentas do Painter saem da barra superior e vão para uma barra vertical à
   esquerda, IGUAL à do Vector: a mesma porta de desenho (não uma cópia), os mesmos ícones/tamanhos/realce do
   activo/dicas/atalhos e o mesmo lugar no layout. Primeiro MEÇA como o Vector a faz — onde mora (procure em
   crates/ph2d-panel-vector, crates/ph2d-app-vec, crates/ph2d-editor-core/src/screens/layout.rs e
   docs/UI_New_and_Simple/vetor_nova_ui/), se é uma porta partilhável ou código do Vector; se for do Vector,
   promova-a a porta partilhada e faça o Vector e o Painter usarem a MESMA (gate: os dois desenham pela mesma
   função). A barra superior do Painter fica só com o que não é ferramenta (diga o que fica e porquê). Atalhos
   de teclado e o estado activo continuam iguais. Foto antes/depois pelo teste, olhada.

2. O INSPECTOR SEGUE A FERRAMENTA. Faça o CENSO primeiro: para cada ferramenta do Painter (Brush/Paint,
   Eraser, Blur, Smear, Clone, Mask, Selection, Fill/ColorDrop, os modos do Wet Paint e do Impasto, …) × cada
   secção/linha do painel do pincel: a ferramenta LÊ essa linha? (o motor dela usa o campo?). Meça pelo código
   e pelo motor (uma linha que muda o traço daquela ferramenta é lida; uma que não muda nada é morta para ela) —
   a lei do «controlo morto» do CLAUDE.md §5.0: o painel escreve onde · quem lê · o leitor DECIDE?. Depois: cada
   ferramenta mostra SÓ o que lê; uma linha que só às vezes é lida segue o padrão `esmaecer` já existente
   (apagada com o porquê), nunca escondida sem razão. A tabela «ferramenta × secção» é UMA fonte (dados), não
   `if`s espalhados. Gate: para cada ferramenta, mexer em cada linha VISÍVEL muda o traço dela (ou a linha está
   esmaecida com o porquê), e nenhuma linha escondida mudaria o traço (o gate mostra os dois lados — controlo
   de não-vácuo). Fotos do painel com cada ferramenta.

3. BLUR, ERASER E SMEAR COM GRAIN (e as outras secções de ponta). Meça se hoje o Grain (e Shape, Dual, Jitter…)
   já age nessas três: pinte com e sem Grain e compare os píxeis. O que já age, mantenha e prove com gate; o que
   está no painel e não age, ou ligue de verdade (padrão-ouro: a ferramenta respeita a ponta inteira, como no
   Krita/Procreate) ou esmaeça com o porquê — decida pela qualidade, diga porquê. A tarefa 2 decide a
   visibilidade por esta medição, não por suposição.

4. O PREVIEW DAS FERRAMENTAS QUE MODIFICAM (dono 23/10: «O preview dessas outras tools que não são o pincel
   poderia ser aprimorado. Por exemplo: um traço de pintura na vertical e o traço da tool modificadora na
   horizontal.»). Para Eraser, Blur, Smear (e toda ferramenta que age sobre tinta que já está lá — confirme a
   lista no código): o preview vivo pinta PRIMEIRO um traço de tinta VERTICAL (cor e pincel fixos de
   referência, iguais em todas as ferramentas, para comparar) e por cima o traço da ferramenta na HORIZONTAL,
   cruzando-o — assim se vê o que ela faz à tinta. Pelo MESMO caminho que a tela (a lei do preview desta linha:
   `o_preview_pinta_cada_capacidade_nova_como_a_tela`), nunca uma simulação à parte. O preview do pincel de
   pintar fica como está. Gate: para cada ferramenta modificadora, o cruzamento muda os píxeis da faixa
   vertical (a ferramenta agiu) e fora do cruzamento a faixa fica igual à de referência; mexer numa linha do
   inspector dessa ferramenta muda o preview (liga à tarefa 2). Fotos do preview de cada uma, olhadas.

Portões red-first + prova de mutação para cada lei (a barra é a mesma porta do Vector; o inspector mostra o
que a ferramenta lê; Grain age no Blur/Eraser/Smear; o preview modificador cruza a tinta de referência). Fotos antes/depois pelo teste, olhadas.
Fecho: handoff da linha (técnico) + resposta ao Enio curta, sem jargão, smoke em passos numerados com o cd
(o binário: `bash scripts/ph2d-run.sh cargo build --profile smoke -p ph2d-host-desktop`).
```
