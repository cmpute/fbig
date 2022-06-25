use std::str::FromStr;

use ibig::ibig;
use fbig::{FBig, DBig};

#[test]
fn test_print() {
    let f = FBig::from(1.2f32);
    // dbg!(&f);
    // println!("{}", f);
    // for i in 0..10 {
    //     println!(".{}, {:.*}", i, i, f);
    // }
    let f = FBig::from_ratio(ibig!(3), ibig!(16), 10);
    // dbg!(&f);
    // println!("{}", f);
    // for i in 0..10 {
    //     println!(".{}, {:.*}", i, i, f);
    // }

    let f = f.with_precision(100).into_decimal();
    dbg!(&f);
    println!("{}", f);
    for i in 0..10 {
        println!(".{}, {:.*}", i, i, f);
    }

    let f = DBig::from_str("121241431345234523452.234523452345234523534e-12").unwrap();
    println!("{}", f);
}
