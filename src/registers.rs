use bilge::prelude::*;

#[allow(dead_code)]
#[repr(u8)]
pub(crate) enum RegisterAddress {
    WhoAmI = 0x0F,
    Ctrl1Xl = 0x10,
    Ctrl2G = 0x11,
    Ctrl3C = 0x12,
    StatusReg = 0x1E,
    OutTempL = 0x20,
    OutTempH = 0x21,
    OutXLG = 0x22,  // gyro pitch rate low
    OutXHG = 0x23,  // gyro pitch rate high
    OutYLG = 0x24,  // gyro roll rate low
    OutYHG = 0x25,  // gyro roll rate high
    OutZLG = 0x26,  // gyro yaw rate low
    OutZHG = 0x27,  // gyro yaw rate high
    OutXLXl = 0x28, // accelerometer x low
    OutXHXl = 0x29, // accelerometer x high
    OutYLXl = 0x2A, // accelerometer y low
    OutYHXl = 0x2B, // accelerometer y high
    OutZLXl = 0x2C, // accelerometer z low
    OutZHXl = 0x2D, // accelerometer z high
}

#[bitsize(4)]
#[derive(FromBits, Clone, Copy)]
pub enum OdrXl {
    PowerDown = 0b0000,
    Hz12_5 = 0b0001,
    Hz26 = 0b0010,
    Hz52 = 0b0011,
    Hz104 = 0b0100,
    Hz208 = 0b0101,
    Hz416 = 0b0110,
    Hz833 = 0b0111,
    Hz1k66 = 0b1000,
    Hz3k33 = 0b1001,
    Hz6k66 = 0b1010,
    Hz1_6 = 0b1011,
    #[fallback]
    Reserved,
}

impl OdrXl {
    pub fn new_at_least(freq_hz: u16) -> Self {
        match freq_hz {
            0..12 => Self::Hz12_5,
            _ if freq_hz <= 12 => Self::Hz12_5,
            _ if freq_hz <= 26 => Self::Hz26,
            _ if freq_hz <= 52 => Self::Hz52,
            _ if freq_hz <= 104 => Self::Hz104,
            _ if freq_hz <= 208 => Self::Hz208,
            _ if freq_hz <= 416 => Self::Hz416,
            _ if freq_hz <= 833 => Self::Hz833,
            _ if freq_hz <= 1660 => Self::Hz1k66,
            _ if freq_hz <= 3330 => Self::Hz3k33,
            _ => Self::Hz6k66,
        }
    }
}

#[bitsize(2)]
#[derive(FromBits, Clone, Copy)]
pub enum FsXl {
    G2 = 0b00,
    G16 = 0b01,
    G4 = 0b10,
    G8 = 0b11,
}

impl FsXl {
    pub fn sensitivity_mg_lsb(&self) -> f32 {
        // direct datasheet values
        match self {
            FsXl::G2 => 0.061,
            FsXl::G4 => 0.122,
            FsXl::G8 => 0.244,
            FsXl::G16 => 0.488,
        }
    }
}

#[bitsize(1)]
#[derive(FromBits, Clone, Copy)]
pub enum Bw0Xl {
    Hz1k5 = 0b0,
    Hz400 = 0b1,
}

#[bitsize(8)]
#[derive(FromBits, Clone, Copy)]
pub(crate) struct Ctrl1Struct {
    pub(crate) bw0_xl: Bw0Xl,
    pub(crate) lpf1_bw_sel: bool,
    pub(crate) fs_xl: FsXl,
    pub(crate) odr_xl: OdrXl,
}

#[bitsize(4)]
#[derive(FromBits, Clone, Copy)]
pub enum OdrG {
    PowerDown = 0b0000,
    Hz12_5 = 0b0001,
    Hz26 = 0b0010,
    Hz52 = 0b0011,
    Hz104 = 0b0100,
    Hz208 = 0b0101,
    Hz416 = 0b0110,
    Hz833 = 0b0111,
    Hz1k66 = 0b1000,
    Hz3k33 = 0b1001,
    Hz6k66 = 0b1010,
    #[fallback]
    Reserved,
}

impl OdrG {
    pub fn new_at_least(freq_hz: u16) -> Self {
        match freq_hz {
            0..=12 => Self::Hz12_5,
            _ if freq_hz <= 26 => Self::Hz26,
            _ if freq_hz <= 52 => Self::Hz52,
            _ if freq_hz <= 104 => Self::Hz104,
            _ if freq_hz <= 208 => Self::Hz208,
            _ if freq_hz <= 416 => Self::Hz416,
            _ if freq_hz <= 833 => Self::Hz833,
            _ if freq_hz <= 1660 => Self::Hz1k66,
            _ if freq_hz <= 3330 => Self::Hz3k33,
            _ => Self::Hz6k66,
        }
    }
}

#[bitsize(3)]
#[derive(FromBits, Clone, Copy)]
pub enum FsG {
    Dps125 = 0b001,
    Dps245 = 0b000,
    Dps500 = 0b010,
    Dps1000 = 0b100,
    Dps2000 = 0b110,
    #[fallback]
    Reserved,
}

impl FsG {
    pub fn sensitivity_mdps_lsb(&self) -> f32 {
        // direct datasheet values
        match self {
            FsG::Dps125 => 4.375,
            FsG::Dps245 => 8.75,
            FsG::Dps500 => 17.50,
            FsG::Dps1000 => 35.0,
            FsG::Dps2000 => 70.0,
            FsG::Reserved => 0.0, // should not happen
        }
    }
}

#[bitsize(8)]
#[derive(FromBits, Clone, Copy)]
pub(crate) struct Ctrl2Struct {
    _reserved: u1,
    pub(crate) fs_g: FsG,
    pub(crate) odr_g: OdrG,
}

#[bitsize(8)]
#[derive(FromBits, Clone, Copy)]
pub(crate) struct Ctrl3CStruct {
    pub(crate) sw_reset: bool,
    pub(crate) ble: bool,
    pub(crate) if_inc: bool,
    pub(crate) sim: bool,
    pub(crate) pp_od: bool,
    pub(crate) h_lactive: bool,
    pub(crate) bdu: bool,
    pub(crate) boot: bool,
}
impl Ctrl3CStruct {
    pub fn default() -> Self {
        Self::new(false, false, true, false, false, false, false, false)
    }
}

#[bitsize(8)]
#[derive(FromBits, Clone, Copy)]
pub(crate) struct StatusRegStruct {
    pub(crate) xlda: bool,
    pub(crate) gda: bool,
    pub(crate) tda: bool,
    _reserved: u5,
}
