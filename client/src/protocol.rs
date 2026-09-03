pub enum FromBytes {
    ServerInformation(ServerInformation),
    MouseMove(MouseMove),
    MouseClick(MouseClick),
    UnknownInstruction(Vec<u8>)
}

impl FromBytes {
    pub fn parse(bytes: &[u8]) -> Self {
        match bytes[0] {
            0x00 => Self::ServerInformation(ServerInformation::from_bytes(&bytes[1..])),
            0x10 => Self::MouseMove(MouseMove::from_bytes(&bytes[1..])),
            0x11 => Self::MouseClick(MouseClick::from_bytes(&bytes[1..])),
            _ => Self::UnknownInstruction(bytes.into())
        }
    }
}

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

#[derive(Debug)]
pub struct ServerInformation {
    pub width: u32,
    pub height: u32
}

impl IntoBytes for ServerInformation {
    fn into_bytes(self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0x00];
        buf.extend_from_slice(&self.width.to_be_bytes());
        buf.extend_from_slice(&self.height.to_be_bytes());
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let w_bytes: [u8; 4] = (&bytes[0..4]).try_into().unwrap();
        let h_bytes: [u8; 4] = (&bytes[4..8]).try_into().unwrap();

        Self {
            width:  u32::from_be_bytes(w_bytes),
            height: u32::from_be_bytes(h_bytes)
        }
    }
}

pub struct MouseMove {
    pub x: u32,
    pub y: u32
}

impl IntoBytes for MouseMove {
    fn into_bytes(self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0x10];
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let w_bytes: [u8; 4] = (&bytes[0..4]).try_into().unwrap();
        let h_bytes: [u8; 4] = (&bytes[4..8]).try_into().unwrap();

        Self {
            x:  u32::from_be_bytes(w_bytes),
            y: u32::from_be_bytes(h_bytes)
        }
    }
}

pub enum MouseButton {
    Left,
    Right
}

impl From<MouseButton> for u8 {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => 0,
            MouseButton::Right => 0
        }
    }
}

impl TryFrom<u8> for MouseButton {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MouseButton::Left),
            1 => Ok(MouseButton::Right),
            _ => Err("Invalid u8".into())
        }
    }
}
pub enum MouseState {
    Released,
    Pressed
}

impl From<MouseState> for u8 {
    fn from(value: MouseState) -> Self {
        match value {
            MouseState::Released => 0,
            MouseState::Pressed => 1
        }
    }
}

impl TryFrom<u8> for MouseState {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MouseState::Released),
            1 => Ok(MouseState::Pressed),
            _ => Err("Invalid u8".into())
        }
    }
}


pub struct MouseClick {
    pub button: MouseButton,
    pub state: MouseState
}

impl IntoBytes for MouseClick {
    fn into_bytes(self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0x11, 0x00];
        
        buf[1] |= u8::from(self.button);
        buf[1] <<= 1;
        buf[1] |= u8::from(self.state);
        
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let button = (bytes[0] >> 1) & 1;
        let state = bytes[0] & 1;

        Self {
            button: button.try_into().unwrap(),
            state:  state.try_into().unwrap()
        }
    }
}