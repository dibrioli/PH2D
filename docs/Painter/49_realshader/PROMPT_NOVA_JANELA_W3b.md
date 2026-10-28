# Prompt da janela nova — line/PainterWatercolor: Óleo Real W3b (smoke do dono 27/10)

```
Linha line/PainterWatercolor (Modo L) — Óleo Real W3b: os três pedidos do smoke do dono (27/10, fim do dia). O dono está
presente e testa: respostas curtas, sem jargão. Trabalho por AGENTES (≤ 3 em paralelo, ficheiros disjuntos); a janela
decide e integra. ⛔ NÃO desligar o PC (ordem do dono 27/10: há outros agentes a trabalhar).
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, este ficheiro inteiro, a tabela de
docs/Painter/49_realshader/ESTADO_MISSAO_W3.md (só as linhas do fim, depois de «Fecho: clippy dos gates novos»: os BUG do
smoke) e docs/Painter/49_realshader/oraculo/README.md §BUG-bordas, §BUG-ressalto, §BUG-papel, §Transparency.
Depois execute os itens 1–3 abaixo.
```

## Estado ao abrir

- W3 fechada e corrigida durante o smoke (27/10): ressalto sem pigmento (`ca26a7738a`), grão do papel através da tinta
  grossa (`ca26a7738a`), Knife Textured sobreposta (`1acf37e292`, cortes `78e69511ab`), bordas `borda_suave`
  (`1ac47ce826`), arrasto 1×/quadro (`92e4455bbd`), Transparency + cache do campo de altura (`6233a4e2ef`). Metallic
  Splats e Oil Bristle saíram da fábrica por ordem do dono (`78e69511ab`): fábrica 52 / aprovados 26.
- Imagem que ficou do dono: docs/Painter/49_realshader/smoke_27_10/ (copie para lá as fotos que ele mandar).

## Os três pedidos do dono (palavras dele)

### 1. Bordas profissionais — «Smooth edges com resultados fracos. Aqui somos profissionais. Precisamos de um substituto profissional ao nano pixel do Rebelle.»

Na foto: bordas da tinta com pontinhos escuros soltos e uma orla borrada. A `borda_suave` (`1ac47ce826`) só reconstrói
o contorno por célula a partir da cobertura; o Smooth Edges antigo borra a máscara depois de sombreada. Padrão-ouro a
decidir MEDINDO (CLAUDE.md §0.0, tecto do hardware): a tinta na BORDA a resolução maior que a grelha — contorno
sub-célula (distância com sinal / iso-linha da massa interpolada) com super-amostragem analítica da cobertura por pixel
de ecrã, e cor/normal/sombra amostradas nesse contorno; ou grelha de borda adaptativa (só células de borda
subdivididas 4×4). Régua: `oraculo/bordas_regua.py` (travamento à grelha, energia do degrau) × fotos Rebelle a zoom
(tire fotos novas a 400 % pelo Rebelle aberto — receita project-memory/reference_rebelle_trial_installed_under_wine.md;
nunca o mate); pontinhos escuros isolados = 0 (gate). Custo ns/célula de borda.

### 2. Transparência VIVA, com todas as opções — «A transparência deixou a tinta pálida sem pigmento, como se fosse opacidade, diferente do resultado do Cycles/Eevee. Quando apontei o defeito da tinta não cobrir o papel, aquela aparência era mais interessante que a atual. A tinta era viva. Descubra o modo de ter todas as opções para o usuário.»

A Transparency de `6233a4e2ef` é ALFA (cobertura × (1 − T)): mistura a tinta com o papel → pálida. A aparência que o
dono achou viva (antes de `ca26a7738a`) = o papel e o grão vistos ATRAVÉS de um filme de cor: transmissão (filtro).
Desenho a medir contra o Cycles (Principled Transmission / volume absorvente fino sobre o papel, mesma malha) e Eevee:
- **Transparency** = transmissão pelo filme: luz atravessa a tinta, bate no papel, volta tingida —
  `cor ≈ papel × T_filme(cor, espessura)²` (Beer-Lambert pela espessura, a cor da tinta como absorção) + reflexo da
  superfície; saturada e viva, o grão do papel sombreia por baixo. Neutro 0 = opaca, ao bit.
- **Alpha** (a de hoje, renomeada): a mistura de cobertura — fica como opção.
- **Relevo do papel na tinta** (controlo à parte): quanto a superfície da tinta segue o grão do papel (tinta fina que
  assenta no dente) — a aparência de antes de `ca26a7738a` como escolha, não como defeito.
Os três no pincel (Tuning → Material, por célula) e no cartão Real Oil, nomes do Blender quando existirem
(Transmission? — decida pelo que o dono vê no Blender: ele disse «transparência como num shader do Blender»; diga-lhe o
nome escolhido). Aberto da `6233a4e2ef`: Transparency só no pincel ainda projecta sombra opaca no papel ao lado.

### 3. «Uma vez que o traço foi feito, se o usuário muda as configurações no painel, não afeta a tinta já colocada no papel — passa a valer para os próximos traços.»

Leia como LEI (confirme com o dono numa linha se for ambíguo no caso de algum controlo): controlos do PINCEL valem para os
traços seguintes; a tinta pousada guarda o que precisa por célula (material, Transparency, …). Faça o censo: que controlos
do painel do pincel (Wet Paint + Tuning) são lidos na COMPOSIÇÃO/no óleo em vez de no depósito (logo mudam a tinta antiga)
— `explorador` com a pergunta «o painel escreve onde · quem lê · lê ao pintar ou ao compor?». Cada um que muda o passado:
passa a ser guardado por célula no depósito (como `MaterialCelula`) ou, se é da tela inteira, vai para o cartão. Gate:
mudar cada controlo do pincel depois de um traço não muda um byte da tela.

## Regras

CLAUDE.md §0 e §2; testes POR NOME; clippy só no fecho (`PH2D_SUITE=fecho`); suíte completa NÃO; pesado por
`scripts/ph2d-run.sh`, `--profile smoke`; neutro ao bit com gate; mutação vista vermelha; OLHE as folhas; ≤ 2 rondas por
item; sem push nem integração. Smoke ao dono em passos numerados. Handoff de continuação só depois do veredito dele.
