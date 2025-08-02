pub fn check_guess(guess: i32, secret: i32) -> i32 {
    if guess == secret {
        0
    } else if guess > secret {
        1
    } else {
        -1
    }
}

pub fn main() {
    // Hard-coded secret number
    let secret: i32 = 7;
    let mut attempts: i32 = 0;
    let mut guess: i32;

    println!("Welcome to the Guessing Game!");
    println!("Try to guess the secret number.");

    loop {
        // Simulate a user guess (incrementing for demo purposes)
        attempts += 1;
        guess = attempts; // e.g., guesses 1, 2, 3, ...

        match check_guess(guess, secret) {
            0 => {
                println!("Guess {}: {} is correct!", attempts, guess);
                break;
            }
            1 => println!("Guess {}: {} is too high.", attempts, guess),
            -1 => println!("Guess {}: {} is too low.", attempts, guess),
            _ => unreachable!(),
        }
    }

    println!("You found the secret number {} in {} attempts!", secret, attempts);
}