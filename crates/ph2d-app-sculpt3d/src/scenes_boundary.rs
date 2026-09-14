//! **A CENA DO PINCEL DE CONTORNO** (`=42`) — a peça tem uma **boca aberta**, e
//! é ela que o pincel deforma.
//!
//! # ⚠️⚠️ Ela NÃO pode abrir numa peça fechada, e a razão é MEDIDA
//!
//! Este pincel só existe onde a malha **acaba**: numa casca fechada não há
//! aresta de borda nenhuma, a busca da âncora falha e o traço **não move um
//! único vértice** — medido no corpus do oráculo (`0` movidos numa esfera de
//! `738` vértices). *Uma cena de esfera mostraria a ferramenta a não fazer coisa
//! nenhuma*, que é o preço que a `=36` já pagou quando o dono respondeu *«do
//! modo como o objecto é não é possível testar»*.
//!
//! ⇒ ela abre numa **tigela**: meia esfera com o topo cortado. A borda é a boca,
//! e tudo o que o pincel faz acontece a partir dela.
//!
//! # ⚠️ O que a cena tem de deixar o dono COMPARAR
//!
//! O que separa este pincel de todos os outros é **de onde a região sai**: nos
//! outros ela sai do cursor e esmorece com a distância a ele; aqui ela sai da
//! **BORDA** e esmorece para **dentro** da peça. ⇒ o roteiro põe o `Move / Grab`
//! e o `Boundary` no mesmo sítio, na mesma ordem — o primeiro faz um bico onde
//! o dedo está, o segundo levanta a boca inteira.

/// `=42` — a cena do **PINCEL DE CONTORNO**.
///
/// ⚠️ **O número foi CONTADO no roteador** (`scenes::CENAS` estava em `41`).
pub(crate) fn boundary_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("42")
}

/// **A TIGELA** — meia esfera com a boca aberta para cima.
///
/// ⚠️ **O corte é em `y > 0` e não num paralelo qualquer:** a boca tem de ser
/// grande o bastante para o cursor lhe chegar com o raio de omissão, e larga o
/// bastante para a cadeia ter dezenas de vértices — com uma boca pequena a
/// deformação cabe num punhado de vértices e o dono não vê a onda entrar pela
/// peça. Medido: `48` vértices de borda, cadeia de `48`, e `144` vértices com
/// peso não-nulo no raio de omissão.
///
/// # ⛔⛔ E a COMPACTAÇÃO não é arrumação: sem ela esta cena não funciona
///
/// Escolher faces não tira posições do pool, e a metade deitada fora deixava
/// **`721` vértices órfãos** (`1 490` posições para `768` faces). Nenhuma face
/// os cita, logo o [`ph2d_mesh::Mesh::from_parts`] aceita-os sem um aviso e
/// nada na tela muda — mas **o cursor desta cena é *o ponto mais alto da
/// peça***, e o ponto mais alto passava a ser o **pólo norte da metade que não
/// existe**, a `1,0` da boca. A busca da âncora aterrava nesse órfão, que não
/// tem aresta nenhuma e portanto não é de borda, e a lei recusava o traço
/// inteiro (`SemBordaAoAlcance`): **`0` vértices movidos**, com a queixa a
/// apontar para o verbo.
///
/// *A aritmética é a porta [`ph2d_mesh::compact_for_faces`]* — ela nasceu no
/// importador de OBJ, onde o mesmo defeito daria a cada peça de um arquivo os
/// vértices de todas as outras.
pub(crate) fn tigela() -> ph2d_mesh::Mesh {
    let cheia = ph2d_mesh::shapes::uv_sphere(32, 48, 1.0);
    let pos = cheia.positions().to_vec();
    let escolhidas: Vec<ph2d_mesh::Face> = cheia
        .faces()
        .iter()
        .filter(|f| f.verts().iter().all(|&v| pos[v as usize][1] <= 0.0))
        .copied()
        .collect();
    let (pos, faces, _) = ph2d_mesh::compact_for_faces(&pos, &escolhidas);
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("a tigela é construída aqui")
}

/// O roteiro da `=42`.
///
/// ⚠️ **O passo (1) é o INDICADOR, e ele vem antes de qualquer arrasto** — pela
/// razão do irmão da pose: a região deste verbo não sai do cursor, e o anel
/// sozinho descreve-o mal. *Um artista que só vê o anel conclui que o pincel faz
/// um calombo redondo, e depois lê o resultado como defeito.*
pub(crate) fn announce() {
    if !boundary_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =42 O PINCEL DE CONTORNO -- levantar a BOCA de uma peca aberta\n\
         [sculpt3d]    Na tela esta' uma TIGELA: meia bola, com a boca virada para cima.\n\
         [sculpt3d]    A boca e' a borda -- e' de la' que este pincel trabalha.\n\
         [sculpt3d]\n\
         [sculpt3d]    Abra o painel com a CRASE (`). A fileira de pinceis esta' no topo; o\n\
         [sculpt3d]    novo chama-se `Boundary`, e esta' no FIM da fileira.\n\
         [sculpt3d]\n\
         [sculpt3d]    (1) Escolha `Boundary` e passe o rato SEM CARREGAR perto da boca.\n\
         [sculpt3d]        -> Acende uma FITA ao longo da beirada: e' exactamente o pedaco\n\
         [sculpt3d]           que vai dobrar. E sai dela uma LINHA para dentro da tigela,\n\
         [sculpt3d]           com uma bolinha na ponta: e' ate' onde a dobra entra, e a\n\
         [sculpt3d]           bolinha e' o eixo em torno do qual a beirada gira.\n\
         [sculpt3d]        -> Afaste o rato da boca (para o fundo da tigela): a fita some e\n\
         [sculpt3d]           fica so' uma bolinha VERMELHA. Isso quer dizer `daqui este\n\
         [sculpt3d]           pincel nao faz nada`.\n\
         [sculpt3d]    (2) Escolha `Move / Grab`, carregue perto da BORDA de cima e arraste\n\
         [sculpt3d]        para o lado.\n\
         [sculpt3d]        -> O barro vem ATRAS do dedo e faz um bico local. E' o que voce ja'\n\
         [sculpt3d]           conhece, e serve de termo de comparacao.\n\
         [sculpt3d]    (3) Ctrl+Z. Volte a `Boundary` e faca o MESMO arrasto, no mesmo sitio.\n\
         [sculpt3d]        -> A BEIRADA e' quem mais se move, e a dobra vai MORRENDO para\n\
         [sculpt3d]           dentro da tigela. Nao e' um bico, e nao e' o miolo: e' a boca\n\
         [sculpt3d]           a virar, com o fundo da tigela parado.\n\
         [sculpt3d]    (4) Ctrl+Z. No painel, troque `Falloff along the edge` para `Radius`.\n\
         [sculpt3d]        -> Antes de arrastar, olhe a FITA: agora so' um PEDACO dela\n\
         [sculpt3d]           acende -- o que esta' perto de onde voce aponta. Arraste e a\n\
         [sculpt3d]           peca faz exactamente o que a fita mostrou.\n\
         [sculpt3d]    (5) Ctrl+Z. Troque para `Loop` e repita.\n\
         [sculpt3d]        -> A fita fica MALHADA e a boca fica ONDULADA: a deformacao vai e\n\
         [sculpt3d]           volta ao longo dela. `Loop and Invert` troca as ondas de lado.\n\
         [sculpt3d]    (6) Ctrl+Z. Volte a `Constant` e arraste o `Origin offset` para cima,\n\
         [sculpt3d]        SEM carregar na peca.\n\
         [sculpt3d]        -> A LINHA para dentro da tigela fica mais comprida e a bolinha\n\
         [sculpt3d]           afunda. Arraste agora: a dobra entra muito mais fundo e fica\n\
         [sculpt3d]           mais forte. O pedaco da BOCA nao muda -- so' a profundidade.\n\
         [sculpt3d]    (7) Ctrl+Z. Em `Deformation`, experimente os outros cinco:\n\
         [sculpt3d]          Expand   -> a boca ABRE ou FECHA, deslizando na propria superficie\n\
         [sculpt3d]          Inflate  -> a beirada engrossa para fora\n\
         [sculpt3d]          Grab     -> a boca segue a mao em qualquer direccao\n\
         [sculpt3d]          Twist    -> a boca RODA sobre o eixo da tigela\n\
         [sculpt3d]          Smooth   -> a boca ALISA-SE (⚠️ este age com o cursor PARADO)\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: no passo (1) nao acender fita nenhuma perto da boca;\n\
         [sculpt3d]    se no passo (3) quem dobrar for o MIOLO em vez da beirada; se sair um\n\
         [sculpt3d]    bico local; se nada mexer; ou se trocar o `Falloff along the edge`\n\
         [sculpt3d]    nao mudar o tamanho da fita.\n\
         [sculpt3d]\n\
         [sculpt3d]    (Duas coisas que sao a LEI e nao defeito: arrastar na direccao da\n\
         [sculpt3d]     propria borda nao faz nada -- so' conta o quanto voce puxa para\n\
         [sculpt3d]     dentro ou para fora; e numa peca FECHADA, sem boca, este pincel nao\n\
         [sculpt3d]     faz nada nenhum.)"
    );
}

#[cfg(test)]
mod tests {
    use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

    /// ⛔ **O gate que as vizinhas `=39`/`=41` pagaram para existir:** duas cenas
    /// a reclamar o mesmo número deixam a segunda **inalcançável e muda**.
    #[test]
    fn a_cena_reclama_o_nivel_que_o_roteador_declara() {
        // ⭐ De COMPILAÇÃO, como as irmãs — ⚠️ e o `cargo check` é cego a ela:
        // só o build devolve o `E0080`.
        const {
            assert!(
                crate::scenes::CENAS >= 42,
                "o tecto do roteador tem de conter esta cena (=42)"
            );
        }
    }

    /// **A SONDA DO CUSTO DO INDICADOR**, na malha da própria cena.
    ///
    /// ⚠️ **`#[ignore]`: ela IMPRIME, não afirma.** O número que governa o
    /// produto é o do perfil em que o dono corre o smoke — corra-a com
    /// `--profile smoke` antes de citar qualquer linha da tabela do
    /// [`ph2d_sculpt3d::boundary_previa`]. *Uma tabela sem o perfil ao lado mede
    /// outro programa.*
    #[test]
    #[ignore]
    fn mede_o_indicador_desta_cena() {
        let malha = super::tigela();
        let alvo = malha
            .positions()
            .iter()
            .copied()
            .max_by(|a, c| a[1].total_cmp(&c[1]))
            .expect("a malha tem vértices");
        for raio in [0.3f32, 0.6, 1.0] {
            let b = Brush {
                verb: Verb::Boundary,
                radius: raio,
                ..Brush::default()
            };
            let mut s = SculptStroke::default();
            let t0 = std::time::Instant::now();
            let n = s
                .boundary_contorno(&malha, &b, Symmetry::default(), alvo)
                .borda()
                .len();
            let primeiro = t0.elapsed().as_secs_f64() * 1e3;
            let t1 = std::time::Instant::now();
            s.boundary_contorno(&malha, &b, Symmetry::default(), alvo);
            let repetido = t1.elapsed().as_secs_f64() * 1e6;
            println!(
                "tigela {:>5} verts · raio {raio:>4.2} · pedacos {n:>3} · \
                 1.a construcao {primeiro:>7.3} ms · quadro repetido {repetido:>7.2} us",
                malha.positions().len()
            );
        }
    }

    /// ⭐⭐ **A MALHA DESTA CENA É ESCOLHA MEDIDA, e este gate é a medição.**
    ///
    /// Numa peça FECHADA o pincel de contorno **não move nada** — não há aresta
    /// de borda, a busca da âncora falha e o traço inteiro é mudo (medido no
    /// corpus do oráculo). ⇒ pôr esta cena numa esfera daria ao dono uma
    /// ferramenta que parece partida.
    ///
    /// ⚠️ *Uma cena de smoke que ensina o contrário do que acontece é pior que
    /// uma cena ausente.* Este gate afirma as duas metades: a tigela **mexe**, e
    /// a esfera fechada **não** — que é o controlo que torna a primeira metade
    /// uma medição em vez de um número solto.
    #[test]
    fn o_contorno_move_a_boca_desta_cena_e_nao_move_uma_peca_fechada() {
        let mover = |mut malha: ph2d_mesh::Mesh| -> usize {
            let b = Brush {
                verb: Verb::Boundary,
                radius: 0.3,
                strength: 1.0,
                ..Brush::default()
            };
            // O ponto mais alto da boca — o cursor do roteiro.
            let alvo = malha
                .positions()
                .iter()
                .copied()
                .max_by(|a, c| a[1].total_cmp(&c[1]))
                .expect("a malha tem vértices");
            let mut s = SculptStroke::default();
            s.begin(&malha);
            let olho = [0.0, 0.0, -1.0];
            let mut total = 0;
            for k in 1..=4 {
                let d = 0.05 * f32::from(u8::try_from(k).unwrap_or(1));
                total = s.dab(
                    &mut malha,
                    &b,
                    &Dab::pulling(alvo, b.radius, olho, [0.0, d, d]),
                    Symmetry::default(),
                );
            }
            total
        };
        let movidos = mover(super::tigela());
        assert!(
            movidos > 30,
            "o contorno moveu só {movidos} vértices na boca da tigela — esta \
             cena mostraria uma ferramenta que parece partida"
        );
        // ⭐ O controlo: a MESMA malha fechada não move nada.
        let fechada = mover(ph2d_mesh::shapes::uv_sphere(32, 48, 1.0));
        assert_eq!(
            fechada, 0,
            "a esfera FECHADA moveu {fechada} vértices — ou a peça tem borda, \
             ou a lei deixou de recusar, e nos dois casos o gate acima deixa de \
             afirmar que a escolha da malha é a que importa"
        );
    }
}
