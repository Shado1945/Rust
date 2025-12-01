pub struct Calculator {
    pub num1: i32,
    pub num2: i32,
    pub total: i32,
}

impl Calculator {
    pub fn addition(number1: i32, number2: i32) -> Calculator {
        Calculator {
            num1: number1,
            num2: number2,
            total: number1 + number2,
        }
    }
}
