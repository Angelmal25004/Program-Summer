const FREEZING_POINT_FAHRENHEIT: f64 = 32.0;

pub fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - FREEZING_POINT_FAHRENHEIT) * 5.0 / 9.0
}

pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + FREEZING_POINT_FAHRENHEIT
}

pub fn main() {
    // Start with the freezing point of water in Fahrenheit
    let mut temp_f: f64 = FREEZING_POINT_FAHRENHEIT;

    // Convert initial temperature and print
    let temp_c = fahrenheit_to_celsius(temp_f);
    println!("{:.2}°F = {:.2}°C", temp_f, temp_c);

    // Convert and print the next 5 integer Fahrenheit values
    for i in 1..=5 {
        let next_f = temp_f + i as f64;
        let next_c = fahrenheit_to_celsius(next_f);
        println!("{:.2}°F = {:.2}°C", next_f, next_c);
    }
}
