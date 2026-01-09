use crate::peca::Peca;
use crate::posicao::Posicao;

pub struct Casa {
    pub posicao: Posicao,
    pub peca: OptionalPeca<Box<dyn Peca>>,
}

pub enum OptionalPeca<Peca> {
    None,
    Some(Peca),
}
