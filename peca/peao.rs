use crate::jogador::{self, Jogador};
use crate::peca::{Peca, PecaData};
use crate::posicao::{Linha, Posicao};

pub struct Peao {
    pub data: PecaData,
}

impl Peao {
    pub fn new(jogador: Jogador) -> Self {
        let p_data: PecaData = PecaData {
            jogador,
            nome: "Peão".to_string(),
        };
        Peao { data: p_data }
    }
}

impl Peca for Peao {
    fn mover(&self, posicao: &Posicao) -> bool {
        todo!()
    }

    fn possiveis_movimentos(&self) -> Vec<crate::posicao::Posicao> {
        todo!()
    }
}
