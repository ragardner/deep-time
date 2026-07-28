//! CCSDS unsegmented time code (CUC) as used for modern packet timestamps.
//!
//! [CCSDS 301.0-B-4](https://ccsds.org/Pubs/301x0b4e1.pdf) Level 1 **CUC** is still
//! the usual binary time field for onboard clocks and CCSDS Space Packet /
//! ECSS PUS secondary headers. A common industry layout is:
//!
//! - **4** coarse octets — whole seconds since **1958-01-01 00:00:00 TAI**
//! - **3** fine octets — binary fraction of a second (resolution \(2^{-24}\) s ≈ 60 ns)
//!
//! That 4+3 shape appears in public ICDs such as Sandia JAS telemetry. deep-time
//! encodes and decodes it with [`Dt::to_ccsds_cuc`] / [`Dt::from_ccsds_cuc`].
//!
//! For contrast, the crate’s private [`Dt::to_wire_bytes`] form keeps the full
//! attosecond value plus explicit scale and target (19 bytes). It is not a CCSDS
//! code and is only for deep-time ↔ deep-time use.
//!
//! ```text
//! cargo run --example ccsds_vs_wire --features wire
//! ```

use deep_time::{Dt, DtErr, Scale};

fn main() -> Result<(), DtErr> {
    // A contemporary TAI reading — CUC Level 1 is always on the 1958 TAI epoch.
    let t = Dt::from_ymd(2025, 4, 17, Scale::TAI, 14, 30, 45, 123_456_789_000_000_000);

    // ── CUC Level 1, 4 coarse + 3 fine (classic packet / OBC stamp) ─────────
    //
    // n_coarse = 4  → seconds since 1958 fit in four octets for current dates
    // n_frac   = 3  → ~60 ns steps; fine field is truncated, never rounded
    // extension = false → one-octet P-field (enough for 4+3)

    let (cuc, n) = t.to_ccsds_cuc(4, 3, false)?;
    assert_eq!(n, 8);

    // P-field: Code ID 001 (CUC Level 1), coarse−1 = 3, fine = 3  →  0x1f
    // T-field: coarse seconds 0x7e936f15, fine 0x1f9add = floor(0.123456789 × 2^24)
    assert_eq!(&cuc[..n], &[0x1f, 0x7e, 0x93, 0x6f, 0x15, 0x1f, 0x9a, 0xdd]);

    let back = Dt::from_ccsds_cuc(&cuc[..n])?;
    assert_eq!(back.scale, Scale::TAI);
    assert_eq!(back.target, Scale::TAI);

    // Fine field cannot hold full attoseconds: residual is under one 2^-24 tick.
    let lost = t.sub(back);
    assert!(lost.to_attos() > 0);
    assert!(lost.to_sec_f() < 1.0 / ((1u64 << 24) as f64));

    // ── Same instant on the private wire format (not CCSDS) ─────────────────
    //
    // version (1) + attos i128 LE (16) + scale (1) + target (1) = 19 bytes.
    // Scale and target survive; nothing is truncated.

    let stamped = t.with(Scale::TT).target(Scale::GPS);
    let wire = stamped.to_wire_bytes();
    assert_eq!(wire.len(), Dt::WIRE_SIZE);
    assert_eq!(Dt::WIRE_SIZE, 19);

    // PartialEq on Dt only compares attos; check scale/target explicitly.
    let recovered = Dt::from_wire_bytes(&wire);
    assert!(recovered.is_some());
    if let Some(recovered) = recovered {
        assert_eq!(recovered.attos, stamped.attos);
        assert_eq!(recovered.scale, Scale::TT);
        assert_eq!(recovered.target, Scale::GPS);
    }

    // CUC is smaller and interoperable; wire is larger and lossless for Dt.
    assert!(n < Dt::WIRE_SIZE);

    Ok(())
}
