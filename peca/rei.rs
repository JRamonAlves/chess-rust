use crate::peca::{Peca, PecaData};
use crate::posicao::Posicao;

pub struct Rei {
    pub data: PecaData,
}

impl Peca for Rei {
    fn mover(&self, posicao: &Posicao) -> bool {
        todo!()
    }

    fn possiveis_movimentos(&self) -> Vec<crate::posicao::Posicao> {
        todo!()
    }
}
