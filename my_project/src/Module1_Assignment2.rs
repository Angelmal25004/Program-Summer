pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}

pub fn main() {
    // Array of 10 integers
    let numbers = [12, 3, 5, 7, 15, 20, 22, 9, 10, 30];

    // Analyze each number with FizzBuzz logic and even/odd
    println!("Analyzing numbers:");
    for &n in numbers.iter() {
        if n % 15 == 0 {
            println!("{}: FizzBuzz", n);
        } else if n % 3 == 0 {
            println!("{}: Fizz", n);
        } else if n % 5 == 0 {
            println!("{}: Buzz", n);
        } else if is_even(n) {
            println!("{}: Even", n);
        } else {
            println!("{}: Odd", n);
        }
    }

    // Sum of all numbers using a while loop
    let mut sum = 0;
    let mut idx = 0;
    while idx < numbers.len() {
        sum += numbers[idx];
        idx += 1;
    }
    println!("Sum of all numbers: {}", sum);

    // Find the largest number using a loop
    let mut largest = numbers[0];
    for &n in numbers.iter() {
        if n > largest {
            largest = n;
        }
    }
    println!("Largest number: {}", largest);
}
