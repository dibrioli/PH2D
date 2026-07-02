// Documentação PT-BR do chrome (painel esquerdo + barra inferior), spec §16.
// Mesmo tratamento dos knobs de ajuste: emoji + título + explicação física +
// dica concreta; os quatro sliders de pincel também carregam as letras de
// correlação (D deposição · A água · F fluxo · R render). Conteúdo autoral,
// derivado da semântica da spec.

export const CHROME_DOCS = {
  colorWheel: {
    emoji: '🎨', title: 'Roda de cor (HSV)',
    doc: 'Ângulo = matiz, raio = saturação. Define a cor-base da ponta do pincel; na tela ela compõe por opacidade saturante (massa 100–400 deixa o papel transparecer) e o pincel sujo mistura o que atravessa.',
    tip: 'Aguadas luminosas: cores saturadas com pouca massa (pressão baixa) — a transparência vem da física, não do seletor.',
  },
  brightBar: {
    emoji: '☀️', title: 'Barra de brilho',
    doc: 'O componente V do HSV da cor atual. Escurecer aqui NÃO adiciona preto físico — só muda a cor do pigmento depositado.',
    tip: 'Sombras vivas: baixe o brilho e mantenha a saturação, em vez de misturar com cinza.',
  },
  palette: {
    emoji: '🧪', title: 'Pigmentos nomeados',
    doc: 'Oito presets de cor clássicos de aquarela. Clicar move a roda e a barra para o pigmento — são só cores; a física (granulação, sangria) é a mesma para todos.',
    tip: 'Misturas realistas: ative "Mistura de pigmento (K–M)" no painel experimental e sobreponha camadas finas.',
  },
  sliderSize: {
    emoji: '🖌️', title: 'Pincel (tamanho)',
    doc: 'Raio do toque: r = 2 + slider·33 px (2–35). A textura de cerdas é ancorada em pixels de tela, então o grão tem o mesmo tamanho físico em qualquer raio — pincéis grandes têm MAIS feixes, não feixes maiores.',
    tip: 'Atalhos [ e ] ajustam em passos de 0.03. Pincéis pequenos ficam sempre macios (a dureza é limitada por r/2).',
    corr: [['D', 3], ['A', 1]],
  },
  sliderPressure: {
    emoji: '⚖️', title: 'Pressão',
    doc: 'Base da pressão sintética (0.2–0.9): multiplica a intensidade do carimbo e DIVIDE a água — traço pesado é mais denso e mais seco; toque leve é mais molhado. A velocidade real do gesto modula por cima (devagar = forte).',
    tip: 'Lavado encharcado: pressão 0.3–0.4. Cobertura densa tipo guache: 0.8+.',
    corr: [['D', 3], ['A', 2]],
  },
  sliderWater: {
    emoji: '💧', title: 'Água',
    doc: 'Quanto líquido cada toque entrega (cresce com o QUADRADO do slider e é inverso à pressão). A água é o veículo: dela dependem sangria, escorrimento com inclinação e o anel escuro ao secar.',
    tip: 'Aquarela molhada que flui: slider no máximo; guache seco: 0.2–0.4 — com o depósito poroso, esfregue para encharcar a folha.',
    corr: [['A', 3], ['F', 2], ['D', 1]],
  },
  sliderErase: {
    emoji: '🩹', title: 'Borracha (força)',
    doc: 'Multiplica a força da ferramenta Apagar. A remoção é multiplicativa e barrada pelo grão do papel — apagar também granula, e nunca zera uma célula de um golpe.',
    tip: 'Levantar luzes suaves: 0.15–0.3 em várias passadas. Correção de verdade: 0.8+.',
    corr: [['D', 3], ['A', 1]],
  },
  shapeRound: {
    emoji: '⚪', title: 'Pincel redondo',
    doc: 'Disco pleno — o carimbo é isotrópico e ignora a direção do traço.',
    tip: 'O mais previsível para lavados e formas orgânicas.',
  },
  shapeFlat: {
    emoji: '▬', title: 'Pincel chato',
    doc: 'Elipse no referencial do traço (1 × 0.35 do raio): cheio ao longo do gesto, fino de lado — como a lâmina de um pincel chato.',
    tip: 'Traços de espessura variável: curve o gesto; o primeiro toque (sem direção) sai redondo.',
  },
  shapeFan: {
    emoji: '🪶', title: 'Pincel leque',
    doc: 'Elipse mais larga (1.4 × 0.5 do raio) no referencial do traço — pegada espalhada e rala, boa para texturas.',
    tip: 'Folhagens e texturas quebradas: pressão alta, água baixa, passadas rápidas.',
  },
  presetWet: {
    emoji: '🌊', title: 'Pincel molhado',
    doc: 'Preset: pressão 0.35, água 0.85. Toque leve e encharcado — deposita pouco pigmento com muito veículo, ideal para molhado-sobre-molhado.',
    tip: 'Pinte, incline a mesa e deixe a física escorrer os filetes.',
  },
  presetDry: {
    emoji: '🏜️', title: 'Pincel seco',
    doc: 'Preset: pressão 0.85, água 0.2. Denso e quase sem veículo — o grão do papel rejeita os vales e a cor sai quebrada nos picos.',
    tip: 'Textura de pincel seco sobre lavados já secos; quase não sangra.',
  },
  toolPaint: {
    emoji: '🖌️', title: 'Pintar',
    doc: 'Deposita pigmento suspenso + água pela trilha de duas etapas: a ponta capta cor da tela (pincel sujo), arrasta o filme sob o gesto e pousa com teto suave. Enquanto o ponteiro está na tela o fluido congela; ao soltar, a física corre.',
    tip: 'Tecla P. Gesto lento = pressão alta (denso e seco); gesto rápido = véu leve.',
  },
  toolErase: {
    emoji: '🧽', title: 'Apagar',
    doc: 'Remoção multiplicativa das duas camadas (suspensa e assentada) e da água, barrada pelo grão — picos do papel perdem mais que os vales.',
    tip: 'Tecla E. Use com força baixa para recuperar luzes sem buraco branco.',
  },
  toolSmear: {
    emoji: '👉', title: 'Esfregar',
    doc: 'Arrasta pigmento SUSPENSO e água do deslocamento anterior (amostra bilinear ponderada por massa). O assentado fica no lugar — igual borrar tinta fresca sobre tinta seca.',
    tip: 'Tecla S. O alcance cresce com a velocidade do gesto.',
  },
  toolBlend: {
    emoji: '🌀', title: 'Misturar',
    doc: 'Máscara saturante pela trilha: cada célula relaxa em direção à média da janela — inclusive o pigmento JÁ SECO se remistura. Esfregar o mesmo ponto acumula rumo à mistura total.',
    tip: 'Tecla N. Suaviza transições duras entre lavados secos.',
  },
  toolWet: {
    emoji: '💦', title: 'Molhar',
    doc: 'Goteja água limpa (sem pigmento) e marca a folha como úmida. Uma passada deixa um véu fino; passadas repetidas acumulam até o teto de água.',
    tip: 'Tecla W. Prepare a área antes de pintar para sangria molhado-sobre-molhado.',
  },
  toolDry: {
    emoji: '🔥', title: 'Secar',
    doc: 'Encolhe o filme e SELA o papel (umidade = 0): sangramentos futuros param na área secada. O filme nunca chega a zero exato.',
    tip: 'Tecla D. Desenhe uma barreira anti-sangria em volta do que quer proteger.',
  },
  toolBlow: {
    emoji: '🌬️', title: 'Soprar',
    doc: 'Injeta o deslocamento do cursor na velocidade persistente das células molhadas e arrasta a umidade — a simulação fica VIVA durante o gesto. Fraco e levemente anti-frente, como um canudo.',
    tip: 'Tecla B — ou arraste com o botão direito em qualquer ferramenta.',
  },
  tiltToggle: {
    emoji: '🎚️', title: 'Inclinação lig/desl',
    doc: 'Liga/desliga a gravidade sem perder a direção do dial. Desligada, a água só nivela e segue o grão.',
    tip: 'Desligue para lavados estáticos; religue para retomar o escorrimento.',
  },
  tiltDial: {
    emoji: '🧭', title: 'Dial de inclinação',
    doc: 'Arraste o pino: direção = para onde a folha pende (12 raios), raio = intensidade (8 anéis; o anel médio = o knob de gravidade). A gravidade entra SEM freio na velocidade persistente, escalada pela água local — por isso poças escorrem e véus não.',
    tip: 'Filetes finos: inclinação no anel máximo sobre área bem molhada; o freio de absorção seleciona os canais do grão.',
  },
  paperCold: {
    emoji: '📄', title: 'Prensado frio',
    doc: 'O papel padrão: feltro quase isotrópico, relevo médio (média 0.543, desvio 0.150). Granulação e capilaridade equilibradas.',
    tip: 'Duplo clique na tela troca entre 4 folhas do mesmo preset.',
  },
  paperRough: {
    emoji: '🧻', title: 'Rugoso',
    doc: 'Relevo fundo com grão fortemente VERTICAL (desvio 0.192): rejeita mais tinta nos vales e canaliza escorrimentos em filetes verticais.',
    tip: 'Melhor papel para drip art com inclinação — os sulcos viram trilhos.',
  },
  paperHot: {
    emoji: '🪞', title: 'Prensado quente',
    doc: 'Liso e raso (desvio 0.106): pouca granulação, bordas mais limpas, a água espalha com menos resistência.',
    tip: 'Detalhe fino e lavados uniformes; a textura quase não aparece.',
  },
  layerAdd: {
    emoji: '➕', title: 'Nova camada',
    doc: 'Até 4 camadas, cada uma com seu próprio campo de fluido/pigmento sobre o MESMO papel. Só a camada ativa simula.',
    tip: 'Separe o desenho de base dos lavados — dá para refazer um sem tocar o outro.',
  },
  layerRemove: {
    emoji: '➖', title: 'Remover camada',
    doc: 'Apaga a camada ativa (a última não pode ser removida). O histórico de desfazer é POR CAMADA.',
    tip: 'Sem pânico: cada camada guarda 6 passos de desfazer próprios.',
  },
  undo: {
    emoji: '↩️', title: 'Desfazer',
    doc: 'Volta o último traço ou ação de tela da camada ativa (anel de 6 passos por camada). Ctrl+Z.',
    tip: 'O estado inteiro do fluido volta — inclusive água em movimento.',
  },
  redo: {
    emoji: '↪️', title: 'Refazer',
    doc: 'Reaplica o passo desfeito. Ctrl+Shift+Z ou Ctrl+Y.',
    tip: 'Fazer um traço novo descarta a pilha de refazer.',
  },
  wetCanvas: {
    emoji: '🌫️', title: 'Molhar tela',
    doc: 'Eleva a UMIDADE da folha inteira (via máximo) sem injetar água — a simulação continua parada, mas o próximo traço sangra em qualquer lugar e "Ver umidade" acende.',
    tip: 'O gesto clássico antes de céus molhado-sobre-molhado.',
  },
  dryCanvas: {
    emoji: '☀️', title: 'Secar tela',
    doc: 'Seca tudo num golpe: assenta todo o pigmento suspenso, zera água, velocidades e umidade. Sem anéis de secagem — é instantâneo.',
    tip: 'Use quando quiser fixar o estado atual antes de uma nova campanha de lavados.',
  },
  fastDry: {
    emoji: '⏩', title: 'Secagem rápida',
    doc: 'Roda passadas aceleradas de evaporação+assentamento até o fluido acabar — as BORDAS ainda escurecem (o fator de borda age), só não há tempo de escorrer.',
    tip: 'O meio-termo: preserva o caráter de aquarela sem esperar a secagem real.',
  },
  showWet: {
    emoji: '👁️', title: 'Ver umidade',
    doc: 'Sobreposição de leitura: escurece frio onde a folha está úmida e acende um brilho de menisco na borda do filme. Não altera a pintura.',
    tip: 'Ligue para saber onde um traço novo vai sangrar antes de arriscar.',
  },
  clear: {
    emoji: '🗑️', title: 'Limpar',
    doc: 'Zera todo o estado dinâmico da camada ativa (pigmento, água, umidade). O papel fica.',
    tip: 'Entra no desfazer — dá para voltar.',
  },
  savePng: {
    emoji: '💾', title: 'Salvar PNG',
    doc: 'Exporta a composição visível (com papel) ou, desmarcando "papel", só o pigmento sobre fundo transparente com alfa real.',
    tip: 'PNG transparente é ideal para levar o lavado a outro app de composição.',
  },
  withPaper: {
    emoji: '📄', title: 'Exportar com papel',
    doc: 'Marcado: salva como você vê (base do papel + grão + relevo). Desmarcado: só as camadas de pigmento, alfa = cobertura real.',
    tip: 'Nome do arquivo muda: ph2d-wet-paint.png vs ph2d-wet-paint-pigment.png.',
  },
  toggleTuning: {
    emoji: '🎛️', title: 'Painel de ajustes',
    doc: 'Mostra/esconde o painel direito com os ~39 knobs de calibração ao vivo e o painel experimental (K–M).',
    tip: 'A borda esquerda do painel é arrastável para redimensionar.',
  },
  lang: {
    emoji: '🌐', title: 'EN/PT',
    doc: 'Alterna o idioma do chrome (botões e rótulos). As dicas técnicas — como esta — permanecem em PT-BR.',
    tip: 'A troca é imediata e não afeta a pintura.',
  },
};
