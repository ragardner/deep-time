use crate::{
    Drift, Dt, Every, LiteStr, Meridiem, Offset, Scale, Spacetime, Parts, TimeRange, Weekday,
};

impl Dt {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes.
    pub const WIRE_SIZE: usize = 19;

    /// Serializes this `Dt` into a fixed 18-byte little-endian buffer using the
    /// `attos: i128` + `scale: Scale` representation.
    ///
    /// ## Wire Format
    ///
    /// - Byte `0`: Version (`WIRE_VERSION`)
    /// - Bytes `[1..17]`: total attoseconds as little-endian `i128`
    /// - Byte `17`: scale as `u8` (enum discriminant)
    /// - Byte `18`: target as `u8` (enum discriminant)
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;
        buf[1..17].copy_from_slice(&self.attos.to_le_bytes());
        buf[17] = self.target as u8;
        buf
    }

    /// Deserializes a [`Dt`] from exactly 18 bytes of wire data.
    ///
    /// Returns `None` if the version byte is unknown, the length is wrong,
    /// or the scale byte is not a valid `Scale` variant.
    ///
    /// ## Wire Format
    ///
    /// - Byte `0`: Version (`WIRE_VERSION`)
    /// - Bytes `[1..17]`: total attoseconds as little-endian `i128`
    /// - Byte `17`: scale as `u8` (enum discriminant)
    /// - Byte `18`: target as `u8` (enum discriminant)
    ///
    /// ## Security
    ///
    /// Safe to call with completely untrusted input. Fixed-size format,
    /// no allocation, no `unsafe`, and no possibility of code execution.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let attos = i128::from_le_bytes([
            bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
            bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], bytes[16],
        ]);

        let scale = Scale::from_u8(bytes[17]);
        let target = Scale::from_u8(bytes[18]);

        Some(Dt::new(attos, scale, target))
    }
}

impl Drift {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes.
    pub const WIRE_SIZE: usize = 3 * Dt::WIRE_SIZE; // 3 × 17 = 51

    /// Serializes this `Drift` polynomial into a fixed buffer.
    ///
    /// The layout is the concatenation of the three `Dt` fields.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        let c = self.constant.to_wire_bytes();
        let r = self.rate.to_wire_bytes();
        let a = self.accel.to_wire_bytes();

        buf[0..Dt::WIRE_SIZE].copy_from_slice(&c);
        buf[Dt::WIRE_SIZE..2 * Dt::WIRE_SIZE].copy_from_slice(&r);
        buf[2 * Dt::WIRE_SIZE..].copy_from_slice(&a);
        buf
    }

    /// Deserializes a `Drift` from exactly `WIRE_SIZE` bytes of wire data.
    ///
    /// Returns `None` if any nested `Dt` fails validation or if the version
    /// byte is unknown.
    ///
    /// ## Security
    ///
    /// Composes the safety guarantees of
    /// [`from_wire_bytes`](docs.rs/deep-time/latest/deep_time/struct.Dt.html#method.from_wire_bytes).
    ///
    /// Fixed size and layered validation make it safe for untrusted input.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let constant = Dt::from_wire_bytes(&bytes[0..Dt::WIRE_SIZE])?;
        let rate = Dt::from_wire_bytes(&bytes[Dt::WIRE_SIZE..2 * Dt::WIRE_SIZE])?;
        let accel = Dt::from_wire_bytes(&bytes[2 * Dt::WIRE_SIZE..])?;

        Some(Self::new(constant, rate, accel))
    }
}

impl Spacetime {
    /// Size of the canonical wire representation in bytes (24 bytes).
    pub const WIRE_SIZE: usize = 24;

    /// Serializes this `Spacetime` snapshot into a fixed 24-byte buffer.
    ///
    /// All fields are stored as little-endian IEEE 754 `f64`.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0..8].copy_from_slice(&self.alpha.to_le_bytes());
        buf[8..16].copy_from_slice(&self.beta.to_le_bytes());
        buf[16..24].copy_from_slice(&self.kretschmann.to_le_bytes());
        buf
    }

    /// Deserializes a `Spacetime` from exactly 24 bytes.
    ///
    /// ## Security
    ///
    /// Accepts any `f64` bit pattern (including `NaN`/`Inf`) to match the
    /// type’s own invariants. Fixed size makes it immune to length-based
    /// attacks. Safe for untrusted input.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        let alpha = f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let beta = f64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let kretschmann = f64::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
        ]);
        Some(Self {
            alpha,
            beta,
            kretschmann,
        })
    }
}

impl Every {
    /// Size of the canonical wire representation in bytes (33 bytes).
    pub const WIRE_SIZE: usize = Dt::WIRE_SIZE + Dt::WIRE_SIZE;

    /// Serializes this `Every` builder into a fixed 33-byte buffer.
    ///
    /// The layout is simply the concatenation of `start` (17 bytes) and `step` (16 bytes).
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        let start = self.start.to_wire_bytes();
        let step = self.step.to_wire_bytes();
        buf[0..17].copy_from_slice(&start);
        buf[17..33].copy_from_slice(&step);
        buf
    }

    /// Deserializes an `Every` builder from exactly 33 bytes.
    ///
    /// ## Security
    ///
    /// Safe for untrusted input. Fixed size with strict validation
    /// of the inner `Dt` and `Dt`.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        let start = Dt::from_wire_bytes(&bytes[0..17])?;
        let step = Dt::from_wire_bytes(&bytes[17..33])?;
        Some(Self { start, step })
    }
}

impl TimeRange {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes.
    /// Only the logical definition is stored (runtime state is not serialized).
    pub const WIRE_SIZE: usize = 1 + 2 * Dt::WIRE_SIZE + Dt::WIRE_SIZE + 1;

    /// Serializes this `TimeRange` into a fixed buffer.
    ///
    /// Only the logical definition is stored:
    /// - `start` + `end` + `step` + `inclusive` flag
    ///
    /// Runtime iterator state (`current`, `finished`) is **not** serialized.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;

        let start = self.start.to_wire_bytes();
        let end = self.end.to_wire_bytes();
        let step = self.step.to_wire_bytes();

        let tp_size = Dt::WIRE_SIZE;
        let span_size = Dt::WIRE_SIZE;

        buf[1..1 + tp_size].copy_from_slice(&start);
        buf[1 + tp_size..1 + 2 * tp_size].copy_from_slice(&end);
        buf[1 + 2 * tp_size..1 + 2 * tp_size + span_size].copy_from_slice(&step);
        buf[1 + 2 * tp_size + span_size] = if self.inclusive { 1 } else { 0 };

        buf
    }

    /// Deserializes a `TimeRange` from exactly `WIRE_SIZE` bytes.
    ///
    /// The iterator is reconstructed in its initial state
    /// (`current = start`, `finished = false`).
    ///
    /// Returns `None` if the version is unknown or any component is invalid.
    ///
    /// ## Security
    ///
    /// Safe for untrusted input. Fixed size with layered validation
    /// of all inner types. No runtime iterator state is accepted from the wire.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let tp_size = Dt::WIRE_SIZE;
        let span_size = Dt::WIRE_SIZE;

        let start = Dt::from_wire_bytes(&bytes[1..1 + tp_size])?;
        let end = Dt::from_wire_bytes(&bytes[1 + tp_size..1 + 2 * tp_size])?;
        let step = Dt::from_wire_bytes(&bytes[1 + 2 * tp_size..1 + 2 * tp_size + span_size])?;
        let inclusive = bytes[1 + 2 * tp_size + span_size] != 0;

        Some(Self::new(start, end, step, inclusive))
    }
}

impl Meridiem {
    pub const WIRE_SIZE: usize = 1;

    #[inline]
    pub const fn to_wire_byte(self) -> u8 {
        match self {
            Meridiem::AM => 0,
            Meridiem::PM => 1,
        }
    }

    #[inline]
    pub const fn from_wire_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Meridiem::AM),
            1 => Some(Meridiem::PM),
            _ => None,
        }
    }
}

impl Offset {
    pub const WIRE_SIZE: usize = 5; // tag (1) + i32 (4)

    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        match self {
            Offset::None => buf[0] = 0,
            Offset::Fixed(offset) => {
                buf[0] = 1;
                buf[1..5].copy_from_slice(&offset.to_le_bytes());
            }
        }
        buf
    }

    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        match bytes[0] {
            0 => Some(Offset::None),
            1 => {
                let offset = i32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Some(Offset::Fixed(offset))
            }
            _ => None,
        }
    }
}

impl Weekday {
    pub const WIRE_SIZE: usize = 1;

    #[inline]
    pub const fn to_wire_byte(self) -> u8 {
        self.wkday_sun_0_based()
    }

    #[inline]
    pub const fn from_wire_byte(b: u8) -> Option<Self> {
        Self::from_sunday_0_based(b)
    }
}

impl Parts {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Total size of the wire representation (119 bytes).
    pub const WIRE_SIZE: usize = 119;

    /// Serializes `Parts` into a fixed 119-byte buffer.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;

        let mut offset = 1usize;

        // year (sentinel = i64::MIN)
        let year = self.yr.unwrap_or(i64::MIN);
        buf[offset..offset + 8].copy_from_slice(&year.to_le_bytes());
        offset += 8;

        // month
        buf[offset] = self.mo.unwrap_or(u8::MAX);
        offset += 1;

        // day
        buf[offset] = self.day.unwrap_or(u8::MAX);
        offset += 1;

        // hour
        buf[offset] = self.hr;
        offset += 1;

        // minute
        buf[offset] = self.min;
        offset += 1;

        // second
        buf[offset] = self.sec;
        offset += 1;

        // attos
        let attos = self.attos;
        buf[offset..offset + 8].copy_from_slice(&attos.to_le_bytes());
        offset += 8;

        // offset (5 bytes)
        let offset_bytes = self.offset.unwrap_or_default().to_wire_bytes();
        buf[offset..offset + 5].copy_from_slice(&offset_bytes);
        offset += 5;

        // iana_name (49 bytes)
        if let Some(name) = &self.iana_name {
            let name_bytes = name.bytes;
            buf[offset..offset + 49].copy_from_slice(&name_bytes);
        }
        offset += 49;

        // scale
        buf[offset] = self.scale as u8;
        offset += 1;

        // weekday
        buf[offset] = self.wkday.map_or(255, |w| w.to_wire_byte());
        offset += 1;

        // day_of_year
        let doy = self.day_of_yr.unwrap_or(u16::MAX);
        buf[offset..offset + 2].copy_from_slice(&doy.to_le_bytes());
        offset += 2;

        // iso_week_year
        let iso_y = self.iso_wk_yr.unwrap_or(i64::MIN);
        buf[offset..offset + 8].copy_from_slice(&iso_y.to_le_bytes());
        offset += 8;

        // iso_week
        buf[offset] = self.iso_wk.unwrap_or(u8::MAX);
        offset += 1;

        // week_sun
        buf[offset] = self.wk_sun.unwrap_or(u8::MAX);
        offset += 1;

        // week_mon
        buf[offset] = self.wk_mon.unwrap_or(u8::MAX);
        offset += 1;

        // meridiem
        buf[offset] = self.meridiem.map_or(255, |m| m.to_wire_byte());
        offset += 1;

        // unix_timestamp_seconds
        let unix = self.unix_timestamp_seconds.unwrap_or(i64::MIN);
        buf[offset..offset + 8].copy_from_slice(&unix.to_le_bytes());

        buf
    }

    /// Deserializes `Parts` from exactly 119 bytes.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let mut dc = Parts::default();
        let mut offset = 1usize;

        // year (8 bytes)
        let year = i64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        if year != i64::MIN {
            dc.yr = Some(year);
        }
        offset += 8;

        // month (1 byte)
        let m = bytes[offset];
        if m != u8::MAX {
            dc.mo = Some(m);
        }
        offset += 1;

        // day (1 byte)
        let d = bytes[offset];
        if d != u8::MAX {
            dc.day = Some(d);
        }
        offset += 1;

        // hour (1 byte)
        dc.hr = bytes[offset];
        offset += 1;

        // minute (1 byte)
        dc.min = bytes[offset];
        offset += 1;

        // second (1 byte)
        dc.sec = bytes[offset];
        offset += 1;

        // attos (8 bytes)
        let attos = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        dc.attos = attos;
        offset += 8;

        // offset (5 bytes)
        if let Some(off) = Offset::from_wire_bytes(&bytes[offset..offset + 5]) {
            dc.offset = Some(off);
        }
        offset += 5;

        // iana_name (49 bytes)
        let iana_bytes = &bytes[offset..offset + 49];
        let name = LiteStr::<49>::from_bytes(iana_bytes);
        if !name.as_bytes().is_empty() {
            dc.iana_name = Some(name);
        }
        offset += 49;

        // scale (1 byte)
        dc.scale = Scale::from_u8(bytes[offset]);
        offset += 1;

        // weekday (1 byte)
        let wd_byte = bytes[offset];
        if wd_byte != 255
            && let Some(wd) = Weekday::from_wire_byte(wd_byte)
        {
            dc.wkday = Some(wd);
        }
        offset += 1;

        // day_of_year (2 bytes)
        let doy = u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?);
        if doy != u16::MAX {
            dc.day_of_yr = Some(doy);
        }
        offset += 2;

        // iso_week_year (8 bytes)
        let iso_y = i64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        if iso_y != i64::MIN {
            dc.iso_wk_yr = Some(iso_y);
        }
        offset += 8;

        // iso_week (1 byte)
        let iw = bytes[offset];
        if iw != u8::MAX {
            dc.iso_wk = Some(iw);
        }
        offset += 1;

        // week_sun (1 byte)
        let ws = bytes[offset];
        if ws != u8::MAX {
            dc.wk_sun = Some(ws);
        }
        offset += 1;

        // week_mon (1 byte)
        let wm = bytes[offset];
        if wm != u8::MAX {
            dc.wk_mon = Some(wm);
        }
        offset += 1;

        // meridiem (1 byte)
        let mer_byte = bytes[offset];
        if mer_byte != 255
            && let Some(m) = Meridiem::from_wire_byte(mer_byte)
        {
            dc.meridiem = Some(m);
        }
        offset += 1;

        // unix_timestamp_seconds (8 bytes)
        let unix = i64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        if unix != i64::MIN {
            dc.unix_timestamp_seconds = Some(unix);
        }

        Some(dc)
    }
}
