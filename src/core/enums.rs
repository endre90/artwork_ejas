#[derive(Debug, Clone)]
pub enum Operator {
    UNDEFINED,
    EXTERNAL,
    Tasha,
    Sarah,
    Jessica,
    Alice,
    Gio,
    Patrik,
    Agnes,
    Vanessa,
    Sara,
    Kenny
}

// impl Default for BinID {
//     fn default() -> Self {
//         BinID::UNDEFINED
//     }
// }

// impl BinID {
//     pub fn from_u64(x: u64) -> BinID {
//         match x {
//             1 => BinID::ID1,
//             2 => BinID::ID2,
//             3 => BinID::ID3,
//             4 => BinID::ID4,
//             5 => BinID::ID5,
//             6 => BinID::ID6,
//             7 => BinID::ID7,
//             8 => BinID::ID8,
//             _ => BinID::UNDEFINED,
//         }
//     }
// }

// impl fmt::Display for BinID {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             BinID::UNDEFINED => write!(f, "undefined"),
//             BinID::ID1 => write!(f, "bin_1"),
//             BinID::ID2 => write!(f, "bin_2"),
//             BinID::ID3 => write!(f, "bin_3"),
//             BinID::ID4 => write!(f, "bin_4"),
//             BinID::ID5 => write!(f, "bin_5"),
//             BinID::ID6 => write!(f, "bin_6"),
//             BinID::ID7 => write!(f, "bin_7"),
//             BinID::ID8 => write!(f, "bin_8"),
//         }
//     }
// }