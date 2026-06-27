// This file contains code and data derived from ERFA (liberfa/erfa),
// which is licensed under the 3-clause BSD license.
// ERFA is derived from the IAU SOFA library.
//
// Copyright (C) 2013-2023, NumFOCUS Foundation.
// See the full license text in the ERFA repository: https://github.com/liberfa/erfa/blob/master/LICENSE

mod constants;
mod xls;
mod xpl;

use crate::{atan2, cos, sin};
use constants::*;
use core::f64::consts::PI;
use core::f64::consts::TAU;
use xls::*;
use xpl::*;

// ============================================================================
// Matrix and basic functions
// ============================================================================

const fn ir(r: &mut [[f64; 3]; 3]) {
    r[0][0] = 1.0;
    r[0][1] = 0.0;
    r[0][2] = 0.0;
    r[1][0] = 0.0;
    r[1][1] = 1.0;
    r[1][2] = 0.0;
    r[2][0] = 0.0;
    r[2][1] = 0.0;
    r[2][2] = 1.0;
}

const fn rz(psi: f64, r: &mut [[f64; 3]; 3]) {
    let s = sin(psi);
    let c = cos(psi);
    let a00 = c * r[0][0] + s * r[1][0];
    let a01 = c * r[0][1] + s * r[1][1];
    let a02 = c * r[0][2] + s * r[1][2];
    let a10 = -s * r[0][0] + c * r[1][0];
    let a11 = -s * r[0][1] + c * r[1][1];
    let a12 = -s * r[0][2] + c * r[1][2];
    r[0][0] = a00;
    r[0][1] = a01;
    r[0][2] = a02;
    r[1][0] = a10;
    r[1][1] = a11;
    r[1][2] = a12;
}

const fn rx(phi: f64, r: &mut [[f64; 3]; 3]) {
    let s = sin(phi);
    let c = cos(phi);
    let a10 = c * r[1][0] + s * r[2][0];
    let a11 = c * r[1][1] + s * r[2][1];
    let a12 = c * r[1][2] + s * r[2][2];
    let a20 = -s * r[1][0] + c * r[2][0];
    let a21 = -s * r[1][1] + c * r[2][1];
    let a22 = -s * r[1][2] + c * r[2][2];
    r[1][0] = a10;
    r[1][1] = a11;
    r[1][2] = a12;
    r[2][0] = a20;
    r[2][1] = a21;
    r[2][2] = a22;
}

const fn bpn2xy(rbpn: &[[f64; 3]; 3], x: &mut f64, y: &mut f64) {
    *x = rbpn[2][0];
    *y = rbpn[2][1];
}

const fn fw2m(gamb: f64, phib: f64, psi: f64, eps: f64, r: &mut [[f64; 3]; 3]) {
    ir(r);
    rz(gamb, r);
    rx(phib, r);
    rz(-psi, r);
    rx(-eps, r);
}

// ============================================================================
// Fundamental arguments
// ============================================================================

/// Mean anomaly of the Moon (IERS 2003)
pub const fn fal03(t: f64) -> f64 {
    (485868.249036
        + t * (1717915923.2178 + t * (31.8792 + t * (0.051635 + t * (-0.00024470)))) % TURNAS)
        * DAS2R
}

/// Mean anomaly of the Sun (IERS 2003)
pub const fn falp03(t: f64) -> f64 {
    (1287104.793048
        + t * (129596581.0481 + t * (-0.5532 + t * (0.000136 + t * (-0.00001149)))) % TURNAS)
        * DAS2R
}

/// Mean longitude of the Moon minus that of the ascending node (IERS 2003)
pub const fn faf03(t: f64) -> f64 {
    (335779.526232
        + t * (1739527262.8478 + t * (-12.7512 + t * (-0.001037 + t * (0.00000417)))) % TURNAS)
        * DAS2R
}

/// Mean longitude of the Moon's ascending node (IERS 2003)
pub const fn faom03(t: f64) -> f64 {
    (450160.398036
        + t * (-6962890.5431 + t * (7.4722 + t * (0.007702 + t * (-0.00005939)))) % TURNAS)
        * DAS2R
}

/// General accumulated precession in longitude
pub const fn fapa03(t: f64) -> f64 {
    (0.024381750 + 0.00000538691 * t) * t
}

/// Mean longitude of Jupiter (IERS Conventions 2003)
pub const fn faju03(t: f64) -> f64 {
    (0.599546497 + 52.9690962641 * t) % TAU
}

/// Mean longitude of Saturn (IERS Conventions 2003)
pub const fn fasa03(t: f64) -> f64 {
    (0.874016757 + 21.3299104960 * t) % TAU
}

/// Mean longitude of Uranus (IERS Conventions 2003)
pub const fn faur03(t: f64) -> f64 {
    (5.481293872 + 7.4781598567 * t) % TAU
}

/// Mean longitude of Mercury (IERS Conventions 2003)
pub const fn fame03(t: f64) -> f64 {
    (4.402608842 + 2608.7903141574 * t) % TAU
}

/// Mean longitude of Venus (IERS Conventions 2003)
pub const fn fave03(t: f64) -> f64 {
    (3.176146697 + 1021.3285546211 * t) % TAU
}

/// Mean longitude of Earth (IERS Conventions 2003)
pub const fn fae03(t: f64) -> f64 {
    (1.753470314 + 628.3075849991 * t) % TAU
}

/// Mean longitude of Mars (IERS Conventions 2003)
pub const fn fama03(t: f64) -> f64 {
    (6.203480913 + 334.0612426700 * t) % TAU
}

/// Mean elongation of the Moon from the Sun (MHB2000 / IERS 2003)
pub const fn fad03(t: f64) -> f64 {
    (1072260.703692
        + t * (1602961601.2090 + t * (-6.3706 + t * (0.006593 + t * (-0.00003169)))) % TURNAS)
        * DAS2R
}

/// Nutation
pub const fn nut00a(date1: f64, date2: f64, dpsi: &mut f64, deps: &mut f64) {
    let t = ((date1 - DJ00) + date2) / DJC;
    let u2r = DAS2R / 1e7;

    // Luni-Solar nutation
    let el = fal03(t);

    /* Mean anomaly of the Sun (MHB2000). */
    let elp = (1287104.79305
        + t * (129596581.0481 + t * (-0.5532 + t * (0.000136 + t * (-0.00001149)))) % TURNAS)
        * DAS2R;

    let f = faf03(t);

    /* Mean elongation of the Moon from the Sun (MHB2000). */
    let d = (1072260.70369
        + t * (1602961601.2090 + t * (-6.3706 + t * (0.006593 + t * (-0.00003169)))) % TURNAS)
        * DAS2R;

    let om = faom03(t);

    let mut dp = 0.0;
    let mut de = 0.0;

    let nls = XLS.len();
    {
        let mut i = nls;
        while i > 0 {
            i -= 1;

            let arg = ((XLS[i].nl as f64) * el
                + (XLS[i].nlp as f64) * elp
                + (XLS[i].nf as f64) * f
                + (XLS[i].nd as f64) * d
                + (XLS[i].nom as f64) * om)
                % TAU;

            let sarg = sin(arg);
            let carg = cos(arg);

            dp += (XLS[i].sp + XLS[i].spt * t) * sarg + XLS[i].cp * carg;
            de += (XLS[i].ce + XLS[i].cet * t) * carg + XLS[i].se * sarg;
        }
    }
    let dpsils = dp * u2r;
    let depsls = de * u2r;

    /* Mean anomaly of the Moon (MHB2000). */
    let al = (2.35555598 + 8328.6914269554 * t) % TAU;
    /* Mean longitude of the Moon minus that of the ascending node */
    /*(MHB2000). */
    let af = (1.627905234 + 8433.466158131 * t) % TAU;
    /* Mean elongation of the Moon from the Sun (MHB2000). */
    let ad = (5.198466741 + 7771.3771468121 * t) % TAU;
    /* Mean longitude of the ascending node of the Moon (MHB2000). */
    let aom = (2.18243920 - 33.757045 * t) % TAU;
    /* General accumulated precession in longitude (IERS 2003). */
    let apa = fapa03(t);
    /* Planetary longitudes, Mercury through Uranus (IERS 2003). */
    let alme = fame03(t);
    let alve = fave03(t);
    let alea = fae03(t);
    let alma = fama03(t);
    let alju = faju03(t);
    let alsa = fasa03(t);
    let alur = faur03(t);
    /* Neptune longitude (MHB2000). */
    let alne = (5.321159000 + 3.8127774000 * t) % TAU;

    dp = 0.0;
    de = 0.0;

    let npl = XPL.len();
    {
        let mut i = npl;
        while i > 0 {
            i -= 1;
            let arg = ((XPL[i].nl as f64) * al
                + (XPL[i].nf as f64) * af
                + (XPL[i].nd as f64) * ad
                + (XPL[i].nom as f64) * aom
                + (XPL[i].nme as f64) * alme
                + (XPL[i].nve as f64) * alve
                + (XPL[i].nea as f64) * alea
                + (XPL[i].nma as f64) * alma
                + (XPL[i].nju as f64) * alju
                + (XPL[i].nsa as f64) * alsa
                + (XPL[i].nur as f64) * alur
                + (XPL[i].nne as f64) * alne
                + (XPL[i].npa as f64) * apa)
                % TAU;
            let sarg = sin(arg);
            let carg = cos(arg);
            dp += (XPL[i].sp as f64) * sarg + (XPL[i].cp as f64) * carg;
            de += (XPL[i].se as f64) * sarg + (XPL[i].ce as f64) * carg;
        }
    }
    let dpsipl = dp * u2r;
    let depspl = de * u2r;

    *dpsi = dpsils + dpsipl;
    *deps = depsls + depspl;
}

pub const fn nut06a(date1: f64, date2: f64, dpsi: &mut f64, deps: &mut f64) {
    let t = ((date1 - DJ00) + date2) / DJC;
    let fj2 = -2.7774e-6 * t;

    let mut dp = 0.0;
    let mut de = 0.0;
    nut00a(date1, date2, &mut dp, &mut de);

    *dpsi = dp + dp * (0.4697e-6 + fj2);
    *deps = de + de * fj2;
}

// ============================================================================
// Precession, obliquity, bias
// ============================================================================

pub const fn obl06(date1: f64, date2: f64) -> f64 {
    let t = ((date1 - DJ00) + date2) / DJC;
    // eps0
    (84381.406
        + (-46.836769
            + (-0.0001831 + (0.00200340 + (-0.000000576 + (-0.0000000434) * t) * t) * t) * t)
            * t)
        * DAS2R
}

const fn pfw06(
    date1: f64,
    date2: f64,
    gamb: &mut f64,
    phib: &mut f64,
    psib: &mut f64,
    epsa: &mut f64,
) {
    /* Interval between fundamental date J2000.0 and given date (JC). */
    let t = ((date1 - DJ00) + date2) / DJC;
    /* P03 bias+precession angles. */
    *gamb = (-0.052928
        + (10.556378
            + (0.4932044 + (-0.00031238 + (-0.000002788 + (0.0000000260) * t) * t) * t) * t)
            * t)
        * DAS2R;
    *phib = (84381.412819
        + (-46.811016
            + (0.0511268 + (0.00053289 + (-0.000000440 + (-0.0000000176) * t) * t) * t) * t)
            * t)
        * DAS2R;
    *psib = (-0.041775
        + (5038.481484
            + (1.5584175 + (-0.00018522 + (-0.000026452 + (-0.0000000148) * t) * t) * t) * t)
            * t)
        * DAS2R;
    *epsa = obl06(date1, date2);
}

const fn pnm06a(date1: f64, date2: f64, rbpn: &mut [[f64; 3]; 3]) {
    let mut gamb = 0.0;
    let mut phib = 0.0;
    let mut psib = 0.0;
    let mut epsa = 0.0;
    let mut dp = 0.0;
    let mut de = 0.0;

    pfw06(date1, date2, &mut gamb, &mut phib, &mut psib, &mut epsa);
    nut06a(date1, date2, &mut dp, &mut de);
    fw2m(gamb, phib, psib + dp, epsa + de, rbpn);
}

const fn eors(rnpb: &[[f64; 3]; 3], s: f64) -> f64 {
    let x = rnpb[2][0];
    let ax = x / (1.0 + rnpb[2][2]);
    let xs = 1.0 - ax * x;
    let ys = -ax * rnpb[2][1];
    let zs = -x;
    let p = rnpb[0][0] * xs + rnpb[0][1] * ys + rnpb[0][2] * zs;
    let q = rnpb[1][0] * xs + rnpb[1][1] * ys + rnpb[1][2] * zs;
    if (p != 0.0) || (q != 0.0) {
        s - atan2(q, p)
    } else {
        s
    }
}

// ============================================================================
// CIO
// ============================================================================

pub const fn s06(date1: f64, date2: f64, x: f64, y: f64) -> f64 {
    let t = ((date1 - DJ00) + date2) / DJC;

    let mut fa: [f64; 8] = [0.0; 8];
    fa[0] = fal03(t);
    fa[1] = falp03(t);
    fa[2] = faf03(t);
    fa[3] = fad03(t);
    fa[4] = faom03(t);
    fa[5] = fave03(t);
    fa[6] = fae03(t);
    fa[7] = fapa03(t);

    let mut w0 = SP[0];
    let mut w1 = SP[1];
    let mut w2 = SP[2];
    let mut w3 = SP[3];
    let mut w4 = SP[4];
    let w5 = SP[5];

    // s0 (t^0)
    {
        let mut i = S0.len();
        while i > 0 {
            i -= 1;
            let mut a = 0.0;
            let mut j = 0;
            while j < 8 {
                a += (S0[i].nfa[j] as f64) * fa[j];
                j += 1;
            }
            w0 += S0[i].s * sin(a) + S0[i].c * cos(a);
        }
    }

    // s1 (t^1)
    {
        let mut i = S1.len();
        while i > 0 {
            i -= 1;
            let mut a = 0.0;
            let mut j = 0;
            while j < 8 {
                a += (S1[i].nfa[j] as f64) * fa[j];
                j += 1;
            }
            w1 += S1[i].s * sin(a) + S1[i].c * cos(a);
        }
    }

    // s2 (t^2)
    {
        let mut i = S2.len();
        while i > 0 {
            i -= 1;
            let mut a = 0.0;
            let mut j = 0;
            while j < 8 {
                a += (S2[i].nfa[j] as f64) * fa[j];
                j += 1;
            }
            w2 += S2[i].s * sin(a) + S2[i].c * cos(a);
        }
    }

    // s3 (t^3)
    {
        let mut i = S3.len();
        while i > 0 {
            i -= 1;
            let mut a = 0.0;
            let mut j = 0;
            while j < 8 {
                a += (S3[i].nfa[j] as f64) * fa[j];
                j += 1;
            }
            w3 += S3[i].s * sin(a) + S3[i].c * cos(a);
        }
    }

    // s4 (t^4)
    {
        let mut i = S4.len();
        while i > 0 {
            i -= 1;
            let mut a = 0.0;
            let mut j = 0;
            while j < 8 {
                a += (S4[i].nfa[j] as f64) * fa[j];
                j += 1;
            }
            w4 += S4[i].s * sin(a) + S4[i].c * cos(a);
        }
    }

    (w0 + (w1 + (w2 + (w3 + (w4 + w5 * t) * t) * t) * t) * t) * DAS2R - x * y / 2.0
}

// ============================================================================
// Top-level EO06A
// ============================================================================

/// Equation of the origins, IAU 2006/2000A.
/// date1 + date2 = Julian Date (TT)
pub const fn earth_eo(date1: f64, date2: f64) -> f64 {
    let mut r = [[0.0_f64; 3]; 3];
    pnm06a(date1, date2, &mut r);
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;
    bpn2xy(&r, &mut x, &mut y);
    let s = s06(date1, date2, x, y);
    eors(&r, s)
}

/// Equation of the equinoxes, IAU 2006/2000A — eraEe06a
pub const fn earth_ee(date1: f64, date2: f64) -> f64 {
    let gst = gst06a(0.0, 0.0, date1, date2);
    let gmst = gmst06(0.0, 0.0, date1, date2);

    anpm(gst - gmst)
}

/// Normalize angle into range 0 <= a < 2π  (eraAnp)
const fn anp(a: f64) -> f64 {
    let mut w = a % TAU;
    if w < 0.0 {
        w += TAU;
    }
    w
}

/// Normalize angle into range -π <= a < π  (eraAnpm)
const fn anpm(a: f64) -> f64 {
    let mut w = a % TAU;
    if w.abs() >= PI {
        w -= TAU.copysign(a);
    }
    w
}

/// Earth Rotation Angle (IAU 2000) — eraEra00
const fn era00(dj1: f64, dj2: f64) -> f64 {
    /* Days since fundamental epoch. */
    let (d1, d2) = if dj1 < dj2 { (dj1, dj2) } else { (dj2, dj1) };
    let t = d1 + (d2 - DJ00);
    let f = (d1 % 1.0) + (d2 % 1.0);

    anp(TAU * (f + 0.7790572732640 + 0.00273781191135448 * t))
}

/// Greenwich apparent sidereal time, IAU 2006, given NPB matrix (eraGst06)
const fn gst06(uta: f64, utb: f64, tta: f64, ttb: f64, rnpb: &[[f64; 3]; 3]) -> f64 {
    let mut x = 0.0;
    let mut y = 0.0;
    bpn2xy(rnpb, &mut x, &mut y);

    let s = s06(tta, ttb, x, y);
    let era = era00(uta, utb);
    let eo = eors(rnpb, s);

    anp(era - eo)
}

/// Greenwich apparent sidereal time (IAU 2006/2000A) — eraGst06a
const fn gst06a(uta: f64, utb: f64, tta: f64, ttb: f64) -> f64 {
    let mut rnpb = [[0.0_f64; 3]; 3];

    pnm06a(tta, ttb, &mut rnpb);
    gst06(uta, utb, tta, ttb, &rnpb)
}

/// Greenwich mean sidereal time, IAU 2006 — eraGmst06
const fn gmst06(uta: f64, utb: f64, tta: f64, ttb: f64) -> f64 {
    // TT Julian centuries since J2000.0.
    let t = ((tta - DJ00) + ttb) / DJC;

    anp(era00(uta, utb)
        + (0.014506
            + (4612.156534
                + (1.3915817 + (-0.00000044 + (-0.000029956 + (-0.0000000368) * t) * t) * t) * t)
                * t)
            * DAS2R)
}
