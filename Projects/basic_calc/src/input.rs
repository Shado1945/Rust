use std::io;

pub Struct Input {
    pub number1: i32,
    pub number2: i32
}

imp Input {
    pub fn user_input(prompt: &str, num_string: String) -> Input{
        println!("Please input a number: {}", num_string);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let res = match input.parse::<i32>() {
            Ok(n) => n,
            Err(e) => {
                error!("This Line is not a integer");
                None
            }
        }

    }
}