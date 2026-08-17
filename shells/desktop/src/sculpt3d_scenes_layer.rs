//! **A CENA DA DEMÃO** (`=33`) — a W8, o `layer.cc` do Blender.
//!
//! ⚠️ **Irmã das outras cenas e não parte delas**, pelo teto de LOC da shell e
//! pela mesma linha de corte: cada arquivo é a história de uma wave.
//!
//! ⚠️ **A cena NÃO arma o verbo nem a altura, e isso é METADE do smoke** — a
//! wave entrega um chip novo no seletor e uma row nova no painel, e uma cena que
//! os escolhesse por baixo do pano pularia exactamente a costura que ela existe
//! para provar. É a mesma cicatriz que as `=28`/`=29`/`=30`/`=32` herdaram do
//! `impasto_smoke` do Painter 2D.

/// `=33` — a cena da **DEMÃO**.
pub(crate) fn coat_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("33")
}

/// O roteiro da `=33`.
///
/// ⚠️ **A pergunta é de OLHO e ela NÃO é *"levantou barro?"*** — o Draw também
/// levanta. É que a demão **PARA**, e para numa altura que o artista escolheu:
/// medido, todo peso da pegada converge para a mesma altura (o falloff é uma
/// TAXA, não um perfil), e o teto não se move quando o pincel muda de tamanho.
pub(crate) fn announce() {
    if !coat_scene() {
        return;
    }
    eprintln!(
        "[sculpt3d] =33 A DEMAO (o verbo Layer, `layer.cc`).\n\
         [sculpt3d]    Ela deita uma camada de espessura ESCOLHIDA: insista quanto quiser\n\
         [sculpt3d]    que ela PARA na altura autorada, e o falloff decide so' quao DEPRESSA\n\
         [sculpt3d]    cada ponto la' chega -- nao qual altura ele atinge. Medido: peso 1,00\n\
         [sculpt3d]    fecha em 1 dab, 0,50 em 5, 0,25 em 10, 0,10 em 28.\n\
         [sculpt3d]    Abra o painel com a CRASE (`) e ache o seletor de verbo.\n\
         [sculpt3d]    (1) O CHIP e a ROW. Escolha LAYER. A row `Layer height` tem de aparecer\n\
         [sculpt3d]        (e so' nela). Se nao aparecer, PARE -- o resto nao diz nada.\n\
         [sculpt3d]    (2) O CONTROLE, e faca-o PRIMEIRO. Pegue o DRAW e esfregue o mesmo\n\
         [sculpt3d]        ponto vinte vezes: o barro sobe, e sobe, e sobe. Guarde a imagem.\n\
         [sculpt3d]    (3) Agora o LAYER, ao lado, e esfregue VINTE vezes. Ele sobe ate' a\n\
         [sculpt3d]        altura da row e PARA. Se continuar a subir, o teto nao chegou ao\n\
         [sculpt3d]        alvo: reporte.\n\
         [sculpt3d]    (4) O TOPO E' CHATO -- e' esta a leitura que separa demao de domo. Passe\n\
         [sculpt3d]        o Layer ate' fechar e olhe de RASPAO: a camada tem um PLATO com uma\n\
         [sculpt3d]        parede em volta, nao um monte. Um domo aqui significa que o falloff\n\
         [sculpt3d]        virou perfil: reporte.\n\
         [sculpt3d]    (5) O TETO NAO SEGUE O PINCEL. Sem mexer na row, aumente MUITO o raio e\n\
         [sculpt3d]        deite outra demao: ela cobre mais area com a MESMA espessura. (No\n\
         [sculpt3d]        Draw, o mesmo gesto deposita mais alto -- e' a diferenca inteira.)\n\
         [sculpt3d]    (6) O Ctrl CAVA a mesma camada, para dentro.\n\
         [sculpt3d]    (7) UM TRACO NOVO deita uma SEGUNDA camada por cima da primeira -- e' o\n\
         [sculpt3d]        que `demao` quer dizer, e e' a referencia (o estado dela morre no\n\
         [sculpt3d]        pen-up). Dentro de UM traco, insistir nao passa do teto.\n\
         [sculpt3d]    (8) A MASCARA e' um TETO e nao so' um freio. Mascare meia pegada e deite:\n\
         [sculpt3d]        o lado protegido para numa FRACAO da camada e fica la'.\n\
         [sculpt3d]    (9) A DUREZA -- o eixo do report, e ele agora tem numero. Suba o\n\
         [sculpt3d]        `Hardness` (a row e' `Pro`: o chip de nivel fica no topo). A demao\n\
         [sculpt3d]        vira uma MESA com PAREDE, nao um domo mais alto: medido, esfregando\n\
         [sculpt3d]        com h=0,90 o plato fecha em 99,9% da altura autorada e ondula 0,9%\n\
         [sculpt3d]        de UMA aresta da malha -- ele e' chato.\n\
         [sculpt3d]        AVISO: as listras na PAREDE sao a GRADE de quads da esfera, nao o\n\
         [sculpt3d]        pincel. O Blender escadeia igual na topologia dele, e a missao e'\n\
         [sculpt3d]        ser identico a ele: nao reporte a escada como defeito.\n\
         [sculpt3d]   (10) O AUTO SMOOTH -- o outro eixo do report. Suba-o ate' o fim e deite\n\
         [sculpt3d]        de novo: a demao SOBREVIVE (medido, relevo 0,0735 -> 0,0698 com o\n\
         [sculpt3d]        alisamento no maximo). Faca o CONTROLE ao lado com o DRAW no mesmo\n\
         [sculpt3d]        valor: ele e' ANIQUILADO (0,0874 -> 0,0002), e isso esta' CERTO --\n\
         [sculpt3d]        ele e' aditivo puro e nao tem meta para onde voltar.\n\
         [sculpt3d]   (11) FRONT FACES ONLY -- a caixa nova, abaixo do Accumulate. Ela e' o\n\
         [sculpt3d]        `use_frontface` do Blender e nasce DESMARCADA, como la'.\n\
         [sculpt3d]        Com dureza ALTA, deite uma demao larga sobre a CURVATURA da peca\n\
         [sculpt3d]        (nao no topo achatado) e compare marcada contra desmarcada:\n\
         [sculpt3d]        desmarcada e' uma MESA (a borda sobe tanto quanto o centro),\n\
         [sculpt3d]        marcada e' uma RAMPA que morre onde a peca vira de lado.\n\
         [sculpt3d]        Medido, UM dab: borda/centro 0,7828 desmarcada contra 0,3793\n\
         [sculpt3d]        marcada. Se as duas desenharem a mesma coisa, o interruptor nao\n\
         [sculpt3d]        chegou ao kernel -- reporte.\n\
         [sculpt3d]        AVISO: e' um dab, e o numero de dabs importa. O facing e' uma TAXA\n\
         [sculpt3d]        nesta lei, nao um perfil: esfregue trinta vezes e as duas CONVERGEM\n\
         [sculpt3d]        (medido, 0,9824 contra 0,9831) -- a demao para na altura autorada\n\
         [sculpt3d]        de qualquer maneira, e o facing so' decide quao depressa ela chega."
    );
}
