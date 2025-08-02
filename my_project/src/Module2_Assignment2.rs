pub fn most_frequent_word(text: &str) -> (String, usize) {
    let mut unique_words: Vec<String> = Vec::new();
    let mut counts: Vec<usize> = Vec::new();

    for word in text.split_whitespace() {
        // Find position of the word in unique_words
        if let Some(pos) = unique_words.iter().position(|w| w == word) {
            // Increment the count via mutable reference
            let count_ref = &mut counts[pos];
            *count_ref += 1;
        } else {
            unique_words.push(word.to_string());
            counts.push(1);
        }
    }

    // Determine the most frequent word
    let mut max_word = String::new();
    let mut max_count = 0;
    for (i, w) in unique_words.iter().enumerate() {
        if counts[i] > max_count {
            max_count = counts[i];
            max_word = w.clone();
        }
    }

    (max_word, max_count)
}

pub fn main() {
    let text = "the quick brown fox jumps over the lazy dog the quick brown fox";
    let (word, count) = most_frequent_word(text);
    println!("Most frequent word: \"{}\" ({} times)", word, count);
}
