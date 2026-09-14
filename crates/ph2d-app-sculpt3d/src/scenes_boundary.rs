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
         [sculpt3d]    (1) Escolha `Move / Grab`, carregue perto da BORDA de cima e arraste\n\
         [sculpt3d]        para o lado.\n\
         [sculpt3d]        -> O barro vem ATRAS do dedo e faz um bico local. E' o que voce ja'\n\
         [sculpt3d]           conhece, e serve de termo de comparacao.\n\
         [sculpt3d]    (2) Ctrl+Z. Escolha `Boundary` e faca o MESMO arrasto, no mesmo sitio.\n\
         [sculpt3d]        -> A BOCA INTEIRA se dobra, e a dobra vai MORRENDO para dentro da\n\
         [sculpt3d]           tigela. Nao e' um bico: e' a beirada a virar.\n\
         [sculpt3d]    (3) Ctrl+Z. No painel, troque `Falloff along the edge` para `Radius`\n\
         [sculpt3d]        e repita.\n\
         [sculpt3d]        -> Agora so' um PEDACO da boca se move -- o que esta' perto de\n\
         [sculpt3d]           onde voce carregou. Com `Constant` era a boca toda.\n\
         [sculpt3d]    (4) Ctrl+Z. Troque para `Loop` e repita.\n\
         [sculpt3d]        -> A boca fica ONDULADA: a deformacao vai e volta ao longo dela.\n\
         [sculpt3d]           `Loop and Invert` faz as ondas trocarem de lado.\n\
         [sculpt3d]    (5) Ctrl+Z. Volte a `Constant` e arraste o `Origin offset` para cima.\n\
         [sculpt3d]        -> A dobra passa a entrar MUITO mais fundo na tigela, e fica mais\n\
         [sculpt3d]           forte. O pedaco da BOCA que se move nao muda -- so' a\n\
         [sculpt3d]           profundidade.\n\
         [sculpt3d]    (6) Ctrl+Z. Em `Deformation`, experimente os outros cinco:\n\
         [sculpt3d]          Expand   -> a boca ABRE ou FECHA, deslizando na propria superficie\n\
         [sculpt3d]          Inflate  -> a beirada engrossa para fora\n\
         [sculpt3d]          Grab     -> a boca segue a mao em qualquer direccao\n\
         [sculpt3d]          Twist    -> a boca RODA sobre o eixo da tigela\n\
         [sculpt3d]          Smooth   -> a boca ALISA-SE (⚠️ este age com o cursor PARADO)\n\
         [sculpt3d]\n\
         [sculpt3d]    DEU ERRADO SE: no passo (2) sair um bico local em vez da boca inteira;\n\
         [sculpt3d]    se nada mexer; se a dobra nao morrer para dentro da peca; ou se trocar\n\
         [sculpt3d]    o `Falloff along the edge` nao mudar QUANTO da boca se move.\n\
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
