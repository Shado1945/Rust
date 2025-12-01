use crate::actions::Calculator;

mod actions;

fn main() {
    let total = Calculator::addition(5, 2);
    println!("{} + {} = {}", total.num1, total.num2, total.total);
}
