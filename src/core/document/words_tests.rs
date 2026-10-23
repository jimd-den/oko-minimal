mod tests {
    use super::super::words::Words;

    fn words_of(source: &str) -> Vec<String> {
        let mut words = Words::new(source).expect("a line of words");
        let mut taken = Vec::new();
        while let Some(word) = words.next() {
            taken.push(word.to_string());
        }
        taken
    }

    #[test]
    fn a_line_is_split_into_its_words() {
        assert_eq!(
            words_of("rect 4 4 4 4 #ff9d00"),
            ["rect", "4", "4", "4", "4", "#ff9d00"]
        );
    }

    #[test]
    fn a_label_is_one_word_however_many_spaces_it_holds() {
        assert_eq!(
            words_of("text 2 3 \"A B\" 2 #ff9d00"),
            ["text", "2", "3", "\"A B\"", "2", "#ff9d00"]
        );
    }

    #[test]
    fn a_label_without_a_closing_quote_is_refused() {
        let error = Words::new("text 0 0 \"A B").expect_err("an unclosed label");

        assert!(error.contains("closing quote"), "{error}");
    }

    #[test]
    fn a_word_may_be_looked_at_without_being_taken() {
        let words = Words::new("after 500ms to").expect("a line of words");

        assert_eq!(words.peek(0), Some("after"));
        assert_eq!(words.peek(1), Some("500ms"));
        assert_eq!(words.peek(2), Some("to"));
    }

    #[test]
    fn a_keyword_is_recognised_by_its_word() {
        let words = Words::new("after 500ms").expect("a line of words");

        assert!(words.is(0, "after"));
        assert!(!words.is(0, "to"));
    }

    #[test]
    fn a_label_holding_a_keyword_is_not_the_keyword() {
        let words = Words::new("\"to\" 1").expect("a line of words");

        assert!(!words.is(0, "to"));
    }

    #[test]
    fn a_coordinate_is_read_as_a_number() {
        let mut words = Words::new("12 -7").expect("a line of words");

        assert_eq!(words.coordinate(), Ok(12));
        assert_eq!(words.coordinate(), Ok(-7));
    }

    #[test]
    fn a_colour_is_read_from_six_hexadecimal_digits() {
        let mut words = Words::new("#ff9d00").expect("a line of words");

        assert_eq!(words.colour().map(|c| (c.r, c.g, c.b)), Ok((255, 157, 0)));
    }

    #[test]
    fn a_label_is_read_without_its_quotes() {
        let mut words = Words::new("\"A B\"").expect("a line of words");

        assert_eq!(words.label(), Ok("A B".to_string()));
    }

    #[test]
    fn a_size_must_be_positive() {
        let mut words = Words::new("0").expect("a line of words");

        assert!(words.extent().expect_err("a zero size").contains("1..="));
    }

    #[test]
    fn a_trailing_word_is_refused() {
        let mut words = Words::new("please").expect("a line of words");

        assert!(words.end().unwrap_err().contains("please"));
    }
}
