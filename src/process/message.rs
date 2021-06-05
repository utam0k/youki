#[derive(Debug)]
pub enum Message {
    // ChildReady = 0x00,
    // InitReady = 0x01,
    ChildReady(i32),
    InitReady,
}

impl ToString for Message {
    fn to_string(&self) -> String {
        match &self {
            Message::ChildReady(pid) => format!("ChildReady={}", pid),
            Message::InitReady => "InitReady".into(),
        }
    }
}

// impl fmt::Display for Message {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:?}", self.to_string())
//     }
// }

// impl From<u8> for Message {
//     fn from(from: u8) -> Self {
//         match from {
//             0x00 => Message::ChildReady,
//             0x01 => Message::InitReady,
//             _ => panic!("unknown message."),
//         }
//     }
// }
