use deep_time::*;

fn main() {
    let x = Dt::from(0, 0, Scale::TAI);
    let y = TSpan::new(0, 0);
    let z = x - y;
}
