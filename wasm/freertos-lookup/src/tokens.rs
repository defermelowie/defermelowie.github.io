//! Split FreeRTOS identifiers into their tokens

#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    Prefix(String),
    Ident(String),
    LParam,
    RParam,
    Comma,
}

impl Token {
    /// Extend a token that holds an inner string
    ///
    /// # Panics
    /// - Whenever `self` does not hold an inner string
    /// - Whenever `self` == [Token::Prefix] and `c` is uppercase
    fn extend(&mut self, c: char) {
        match self {
            Token::LParam => unreachable!(),
            Token::RParam => unreachable!(),
            Token::Comma => unreachable!(),
            Token::Ident(i) => i.push(c),
            Token::Prefix(i) => {
                assert!(!c.is_uppercase());
                i.push(c);
            }
        }
    }
}

/// Split an identifier, or a full function call, into [`Token`]s.
///
/// Whitespace is discarded; `(`, `)`, and `,` become their own tokens.
/// Never panics, even on malformed/untrusted input.
pub(crate) fn tokenize<S: AsRef<str>>(s: S) -> Vec<Token> {
    // All names are expected to start with a prefix; the token currently
    // being built is always `tokens`'s last element.
    let mut tokens = vec![Token::Prefix(String::new())];
    for c in s.as_ref().chars() {
        match c {
            '(' => tokens.push(Token::LParam), // opens parameter list in a function call
            ')' => tokens.push(Token::RParam), // closes parameter list in a function call
            ',' => tokens.push(Token::Comma),  // separates parameters in a function call
            c if c.is_whitespace() => (), // never part of an identifier, and not a token itself
            c if c.is_uppercase() => match tokens.last_mut() {
                Some(t @ Token::Ident(_)) => t.extend(c), // Ident tokens can have capitals in the middle
                _ => tokens.push(Token::Ident(c.to_string())),
            },
            // A char right after `(`/`)`/`,` starts a new prefix rather than
            // extending that token, since search input isn't guaranteed to
            // be a well-formed identifier.
            c => match tokens.last_mut() {
                Some(t @ (Token::Prefix(_) | Token::Ident(_))) => t.extend(c),
                _ => tokens.push(Token::Prefix(c.to_string())),
            },
        }
    }
    tokens
}

#[cfg(test)]
mod test {
    use super::{
        Token::Comma, Token::Ident, Token::LParam, Token::Prefix, Token::RParam, tokenize,
    };

    #[test]
    fn extend_prefix_accepts_non_uppercase() {
        let mut prefix_token = Prefix("hell".to_string());
        prefix_token.extend('o');
        assert_eq!(prefix_token, Prefix("hello".to_string()));
    }

    #[test]
    #[should_panic]
    fn extend_prefix_panics_on_uppercase() {
        Prefix("hell".to_string()).extend('O');
    }

    #[test]
    fn extend_ident_accepts_any_case() {
        // Lowercase char
        let mut ident_token = Ident("Star".to_string());
        ident_token.extend('t');
        assert_eq!(ident_token, Ident("Start".to_string()));

        // Uppercase char, mid-word
        let mut ident_token = Ident("Start".to_string());
        ident_token.extend('S');
        assert_eq!(ident_token, Ident("StartS".to_string()));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenize_splits_prefix_on_first_capital() {
        // Macros
        assert_eq!(tokenize("configUSE_PREEMPTION"), vec![Prefix("config".to_string()), Ident("USE_PREEMPTION".to_string())]);
        assert_eq!(tokenize("taskENTER_CRITICAL"), vec![Prefix("task".to_string()), Ident("ENTER_CRITICAL".to_string())]);
        assert_eq!(tokenize("portYIELD_CORE"), vec![Prefix("port".to_string()), Ident("YIELD_CORE".to_string())]);

        // Variables & API functions
        assert_eq!(tokenize("vTaskStartScheduler"), vec![Prefix("v".to_string()), Ident("TaskStartScheduler".to_string())]);
        assert_eq!(tokenize("uxTaskPriorityGet"), vec![Prefix("ux".to_string()), Ident("TaskPriorityGet".to_string())]);

        // Private functions
        assert_eq!(tokenize("prvCounter"), vec![Prefix("prv".to_string()), Ident("Counter".to_string())]);
    }

    #[test]
    fn tokenize_emits_lparam_rparam_tokens() {
        assert_eq!(
            tokenize("prvCheckPendingReadyList()"),
            vec![
                Prefix("prv".to_string()),
                Ident("CheckPendingReadyList".to_string()),
                LParam,
                RParam
            ]
        );
    }

    #[test]
    fn tokenize_does_not_panic_on_chars_after_paren() {
        assert_eq!(
            tokenize("vTaskDelay(xTicksToDelay)"),
            vec![
                Prefix("v".to_string()),
                Ident("TaskDelay".to_string()),
                LParam,
                Prefix("x".to_string()),
                Ident("TicksToDelay".to_string()),
                RParam,
            ]
        );
        assert_eq!(
            tokenize("foo()bar"),
            vec![
                Prefix("foo".to_string()),
                LParam,
                RParam,
                Prefix("bar".to_string()),
            ]
        );
    }

    #[test]
    fn tokenize_splits_multiple_params_on_comma() {
        assert_eq!(
            tokenize("prvSomeFunc(pxDest, xSrc)"),
            vec![
                Prefix("prv".to_string()),
                Ident("SomeFunc".to_string()),
                LParam,
                Prefix("px".to_string()),
                Ident("Dest".to_string()),
                Comma,
                Prefix("x".to_string()),
                Ident("Src".to_string()),
                RParam,
            ]
        );
    }

    #[test]
    fn tokenize_allows_digit_and_underscore_in_prefix() {
        assert_eq!(
            tokenize("arm_v8YIELD_CORE"),
            vec![
                Prefix("arm_v8".to_string()),
                Ident("YIELD_CORE".to_string())
            ]
        );
    }
}
