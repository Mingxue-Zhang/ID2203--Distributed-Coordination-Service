use std::time::Instant;


fn main() {
    let t = Instant::now();
    println!("now: {:?}", Instant::now().checked_duration_since(t).unwrap().as_secs_f64());

}