#[derive(Debug)]
pub struct Posicao {
    pub linha: Linha,
    pub coluna: Coluna,
}

impl Posicao {
    pub fn new(linha: Linha, coluna: Coluna) -> Self {
        Posicao {
            linha: linha,
            coluna: coluna,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coluna {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
}

impl From<i32> for Linha {
    fn from(value: i32) -> Self {
        match value {
            1 => Linha::A,
            2 => Linha::B,
            3 => Linha::C,
            4 => Linha::D,
            5 => Linha::E,
            6 => Linha::F,
            7 => Linha::G,
            8 => Linha::H,
            _ => panic!("Invalid i32 value: {} for Linha enum conversion", value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linha {
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
    F = 6,
    G = 7,
    H = 8,
}

impl From<i32> for Coluna {
    fn from(value: i32) -> Self {
        match value {
            1 => Coluna::One,
            2 => Coluna::Two,
            3 => Coluna::Three,
            4 => Coluna::Four,
            5 => Coluna::Five,
            6 => Coluna::Six,
            7 => Coluna::Seven,
            8 => Coluna::Eight,
            _ => panic!("Invalid i32 value: {} for Coluna enum conversion", value),
        }
    }
}
