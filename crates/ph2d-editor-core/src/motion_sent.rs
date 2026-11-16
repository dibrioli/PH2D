//! **O que o cartão do objecto MOTION lê** (GEF 08 MI-8): quantas vezes cada nó «Send signal» dele
//! enviou nesta corrida. Só leitura — a contagem vem da ponte de CPU (`ph2d_node_send_signal::Envios`).

/// Um nó «Send signal» do objecto: o nome do sinal (o texto do nó) e as vezes nesta corrida.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MotionSent {
    pub signal: String,
    pub times: u64,
}

/// O instantâneo do cartão — só existe com o jogo a correr e pelo menos um «Send signal» no objecto.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorMotionSentInfo {
    pub entity_bits: u64,
    pub sent: Vec<MotionSent>,
}
