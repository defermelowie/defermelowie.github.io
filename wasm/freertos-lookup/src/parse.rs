//! Recursive descent parser for FreeRTOS identfiers
//!
//! Doc: https://freertos.org/Documentation/02-Kernel/06-Coding-guidelines/02-FreeRTOS-Coding-Standard-and-Style-Guide#naming-conventions

use crate::ast;
use scan::{ScanAction, Scanner};

/// Types that can be parsed from a [Scanner]
trait Parse: Sized {
    fn parse(sc: &mut Scanner) -> Option<Self>;
}

impl Parse for ast::Ty {
    fn parse(sc: &mut Scanner) -> Option<Self> {
        let pointer = sc.take('p').is_some();
        let signed = sc.take('u').is_none();

        sc.transform(|c| match c {
            'l' => Some(ast::Ty::new_long(signed)),
            's' => Some(ast::Ty::new_short(signed)),
            'c' => Some(ast::Ty::new_char(signed)),
            'x' => Some(ast::Ty::new(signed, None)),
            'e' => Some(ast::Ty::new_enum(signed)),
            'v' => Some(ast::Ty::new_void()),
            _ => None,
        })
        .map(|ty| if pointer { ty.ptr() } else { ty })
    }
}

impl Parse for ast::Var {
    fn parse(sc: &mut Scanner) -> Option<Self> {
        let ty = ast::Ty::parse(sc)?;
        let name = sc.scan(|str| ScanAction::Request(str.to_string()))?;
        Some(ast::Var::new(ty, name))
    }
}

impl Parse for ast::Fun {
    fn parse(sc: &mut Scanner) -> Option<Self> {
        // Store cursor such that we can later reset the scanner if needed
        let cursor = sc.cursor();

        // Try to parse as privileged function
        sc.set_cursor(cursor); // Reset cursor
        let prv = sc.scan(|str| match str {
            // Explicit prefix match: cheaper than target.starts_with(str) for short targets?
            "p" => ScanAction::Require,
            "pr" => ScanAction::Require,
            "prv" => ScanAction::Return(true),
            _ => ScanAction::Abort,
        });
        let name = prv.and_then(|_| sc.scan(|str| ScanAction::Request(str.to_string())));
        // FIXME: functions should have parameter list: uxGetTaskCounter(...)
        if let Some(name) = name {
            return Some(ast::Fun::Privileged { name });
        }

        // Try to parse as API function
        sc.set_cursor(cursor); // Reset cursor
        let ty = ast::Ty::parse(sc);
        let name = ty.and_then(|_| sc.scan(|str| ScanAction::Request(str.to_string())));
        // FIXME: functions should have parameter list: uxGetTaskCounter(...)
        if let Some(ret) = ty
            && let Some(name) = name
        {
            return Some(ast::Fun::Api { ret, name });
        }

        // Parsing failed
        None
    }
}

impl Parse for ast::Macro {
    fn parse(sc: &mut Scanner) -> Option<Self> {
        let file = sc.scan(|str| {
            if str.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                ScanAction::Request(dbg!(str.to_string()))
            } else {
                ScanAction::Abort
            }
        })?;
        let name = sc.scan(|str| {
            if str.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                ScanAction::Request(dbg!(str.to_string()))
            } else {
                ScanAction::Abort
            }
        })?;
        Some(ast::Macro::new(file, name))
    }
}

pub(crate) fn parse_ident<S: Into<Scanner>>(ident: S) -> Option<ast::Ident> {
    let mut sc = ident.into();
    let cursor = sc.cursor();

    // Try to parse as function
    sc.set_cursor(cursor); // Reset cursor
    if let Some(fun) = ast::Fun::parse(&mut sc) {
        return Some(ast::Ident::Function(fun));
    }

    // Try to parse as variable
    sc.set_cursor(cursor); // Reset cursor
    if let Some(var) = ast::Var::parse(&mut sc) {
        return Some(ast::Ident::Variable(var));
    }

    // Try to parse as macro
    sc.set_cursor(cursor); // Reset cursor
    if let Some(mac) = ast::Macro::parse(&mut sc) {
        return Some(ast::Ident::Macro(mac));
    }

    // Parsing failed
    None
}

/// Helper for writing parsers
mod scan {
    pub(super) enum ScanAction<T> {
        /// If next iteration returns [Self::Abort], return T without advancing
        /// the cursor. I.e. request more (optional) characters for further
        /// parsing.
        Request(T),

        /// If the next iteration returns [Self::Abort], return [None] without advancing
        /// the cursor. I.e. require more characters for successful parsing.
        Require,

        /// Immediately advance the cursor and return T.
        Return(T),

        /// Continue scanning for more characters if available.
        Continue,

        /// Abort scanning. Return the last request or [None] if parsing
        /// requires more characters.
        Abort,
    }

    /// Wrapper around cursor value
    #[derive(Clone, Copy)]
    pub(super) struct ScanCursor(usize);

    #[derive(Debug)]
    pub(super) struct Scanner {
        cursor: usize,
        chars: Vec<char>,
    }

    impl Scanner {
        /// Create a new scanner for a given string
        pub fn new<S: AsRef<str>>(string: S) -> Self {
            Self {
                cursor: 0,
                chars: string.as_ref().chars().collect(),
            }
        }

        /// Get the current cursor position
        #[inline]
        pub fn cursor(&self) -> ScanCursor {
            ScanCursor(self.cursor)
        }

        /// (Re)set the cursor
        #[inline]
        pub fn set_cursor(&mut self, cursor: ScanCursor) {
            self.cursor = cursor.0
        }

        // Peek at the next character if it exists,
        // returns `None` otherwise
        #[inline]
        fn peek(&self) -> Option<&char> {
            self.chars.get(self.cursor)
        }

        /// Pop the next character from the buffer.
        /// Returns `None` if there is no next character.
        fn pop(&mut self) -> Option<&char> {
            // NOTE: can't reuse peek here since it would borrow entire self, blocking the cursor update
            match self.chars.get(self.cursor) {
                Some(c) => {
                    self.cursor += 1;
                    Some(c)
                }
                None => None,
            }
        }

        /// Returns `Some(c)` if and only if the next character matches `c`.
        /// Also advances the cursor in case of a match
        pub fn take(&mut self, c: char) -> Option<&char> {
            match self.peek() {
                Some(char) if char == &c => self.pop(),
                _ => None,
            }
        }

        /// Transforms a character into a value according to the provided function.
        /// Returns `None` if there is no next character or the provided function returns `None`.
        pub fn transform<T>(&mut self, cb: impl FnOnce(&char) -> Option<T>) -> Option<T> {
            let res = self.peek().map(cb).flatten();
            if res.is_some() {
                let _ = self.pop();
            }
            res
        }

        /// Scans a string of characters and transforms it into a value according to the provided function.
        pub fn scan<T>(&mut self, cb: impl Fn(&str) -> ScanAction<T>) -> Option<T> {
            let mut sequence = String::new();
            let mut require = false;
            let mut request = None;

            while let Some(ch) = self.peek() {
                sequence.push(*ch);

                match cb(&sequence) {
                    ScanAction::Return(result) => {
                        self.cursor += 1;
                        return Some(result);
                    }
                    ScanAction::Request(result) => {
                        self.cursor += 1;
                        require = false;
                        request = Some(result);
                    }
                    ScanAction::Require => {
                        self.cursor += 1;
                        require = true;
                    }
                    ScanAction::Continue => {
                        self.cursor += 1;
                    }
                    ScanAction::Abort => {
                        if require {
                            return None;
                        } else {
                            return request;
                        }
                    }
                }
            }
            // No more characters available
            if require {
                return None;
            } else {
                return request;
            }
        }
    }

    impl<S: AsRef<str>> From<S> for Scanner {
        fn from(value: S) -> Self {
            Scanner::new(value)
        }
    }
}

/// Test various [Parse] implementations
#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_parse_ty() {
        let ty = Ty::parse(&mut Scanner::new("ul")).unwrap();
        assert_eq!(ty, Ty::UINT32_T);

        let ty = Ty::parse(&mut Scanner::new("px")).unwrap();
        assert_eq!(ty, Ty::new(true, None).ptr());

        assert!(Ty::parse(&mut Scanner::new("prv")).is_none());
    }

    #[test]
    fn test_parse_var() {
        let var = Var::parse(&mut Scanner::new("ulA")).unwrap();
        assert_eq!(var, Var::new(Ty::UINT32_T, "A".to_string()));

        let var = Var::parse(&mut Scanner::new("pcCounter")).unwrap();
        assert_eq!(var, Var::new(Ty::INT8_T.ptr(), "Counter".to_string()));

        assert!(Var::parse(&mut Scanner::new("prvCounter")).is_none());
    }

    #[test]
    fn test_parse_fun() {
        let fun = Fun::parse(&mut Scanner::new("vQueueDelete")).unwrap();
        assert_eq!(
            fun,
            Fun::Api {
                ret: Ty::VOID,
                name: "QueueDelete".to_string()
            }
        );

        let fun = Fun::parse(&mut Scanner::new("ulTaskNotifyTake")).unwrap();
        assert_eq!(
            fun,
            Fun::Api {
                ret: Ty::UINT32_T,
                name: "TaskNotifyTake".to_string()
            }
        );

        let fun = Fun::parse(&mut Scanner::new("pvTimerGetTimerID")).unwrap();
        assert_eq!(
            fun,
            Fun::Api {
                ret: Ty::VOID.ptr(),
                name: "TimerGetTimerID".to_string()
            }
        );

        let fun = Fun::parse(&mut Scanner::new("prvYieldCore")).unwrap();
        assert_eq!(
            fun,
            Fun::Privileged {
                name: "YieldCore".to_string()
            }
        )
    }

    #[test]
    fn test_parse_macro() {
        let mac = Macro::parse(&mut Scanner::new("portYIELD_CORE")).unwrap();
        assert_eq!(
            mac,
            Macro::new("port".to_string(), "YIELD_CORE".to_string())
        );

        let mac = Macro::parse(&mut Scanner::new("configUSE_MUTEXES")).unwrap();
        assert_eq!(
            mac,
            Macro::new("config".to_string(), "USE_MUTEXES".to_string())
        );

        assert!(Macro::parse(&mut Scanner::new("ConfigUseMessageQueue")).is_none());
    }
}
