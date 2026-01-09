use crate::peca::{Peca, PecaData};
use crate::posicao::Posicao;

pub struct Torre {
    pub data: PecaData,
}

impl Peca for Torre {
    fn mover(&self, posicao: &Posicao) -> bool {
        todo!()
    }

    fn possiveis_movimentos(&self) -> Vec<crate::posicao::Posicao> {
        todo!()
    }
}
