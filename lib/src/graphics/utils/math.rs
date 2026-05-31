pub fn gcd(number1: i32, number2: i32) -> i32 {
    if number2 == 0 { number1 } else { gcd(number2, number1 % number2) }
}

pub fn lcm(number1: i32, number2: i32) -> i64 {
    if number1 == 0 || number2 == 0 {
        0
    } else {
        (number1 as i64 * number2 as i64).abs() / gcd(number1, number2) as i64
    }
}