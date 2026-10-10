//! Parser for FreeRTOS identifiers, built on the token stream produced by
//! [`tokens::tokenize`] -- i.e. `&[Tokens] -> Result<Identifier, ParseErr>`
//!
//! Doc: https://freertos.org/Documentation/02-Kernel/06-Coding-guidelines/02-FreeRTOS-Coding-Standard-and-Style-Guide#naming-conventions

use std::fmt::Write;

use crate::tokens::{self, Token};

/// A parse error
// TODO: implement `std::error::Error` (+ `Display`) so this composes with
// `?`, `Box<dyn Error>`, etc. like a proper error type.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseErr {
    MissingPrefix,
    MissingIdent,
    UnknownPrefix,
    IllegalFunctionName,
    IllegalVariableName,
    IllegalMacroName,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum BaseTy {
    Char,
    Short,
    Long,
    Enum,
    Void,
    Custom,
}

impl BaseTy {
    /// Does this base type have a fixed-width `stdint.h` name (`int32_t`, ...)?
    const fn is_stdint(&self) -> bool {
        matches!(self, BaseTy::Long | BaseTy::Short | BaseTy::Char)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct Type {
    ty: BaseTy,
    /// Is this a pointer type?
    ptr: bool,
    /// Is this a signed type?
    sign: bool,
}

#[rustfmt::skip]
impl Type {
    pub const UINT32_T: Self = Self { ptr: false, sign: false, ty: BaseTy::Long };
    pub const INT32_T: Self  = Self { ptr: false, sign: true,  ty: BaseTy::Long };
    pub const UINT16_T: Self = Self { ptr: false, sign: false, ty: BaseTy::Short };
    pub const INT16_T: Self  = Self { ptr: false, sign: true,  ty: BaseTy::Short };
    pub const UINT8_T: Self  = Self { ptr: false, sign: false, ty: BaseTy::Char };
    pub const INT8_T: Self   = Self { ptr: false, sign: true,  ty: BaseTy::Char };
    pub const VOID: Self     = Self { ptr: false, sign: true,  ty: BaseTy::Void };
 }

impl Type {
    /// Convert a type to its pointer variant
    pub const fn to_ptr(mut self) -> Self {
        self.ptr = true;
        self
    }

    /// Write the Hungarian-notation type prefix for this type -- the
    /// inverse of [`Type::from_str`](std::str::FromStr::from_str).
    fn write_prefix(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ptr {
            f.write_char('p')?;
        }
        if !self.sign {
            f.write_char('u')?;
        }
        f.write_char(match self.ty {
            BaseTy::Long => 'l',
            BaseTy::Short => 's',
            BaseTy::Char => 'c',
            BaseTy::Enum => 'e',
            BaseTy::Void => 'v',
            BaseTy::Custom => 'x',
        })
    }
}

impl std::str::FromStr for Type {
    type Err = ParseErr;

    /// Decode a Hungarian-notation type prefix (`p`? `u`? then one of `l s c x e v`)
    /// into a [`Type`].
    ///
    /// Returns `Err` unless `s` is *entirely* consumed by the grammar, so
    /// e.g. `"config"` is never mistaken for the `"c"` (char) type prefix.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        let ptr = chars.as_str().starts_with('p');
        if ptr {
            chars.next();
        }

        let unsigned = chars.as_str().starts_with('u');
        if unsigned {
            chars.next();
        }
        let sign = !unsigned;

        let ty = match chars.next() {
            Some('l') => BaseTy::Long,
            Some('s') => BaseTy::Short,
            Some('c') => BaseTy::Char,
            Some('x') => BaseTy::Custom,
            Some('e') => BaseTy::Enum,
            Some('v') => BaseTy::Void,
            _ => return Err(ParseErr::UnknownPrefix),
        };

        // The whole prefix must be consumed, or this isn't a type prefix at
        // all (e.g. `config`, `port`, ...).
        if chars.next().is_none() {
            Ok(Type { ty, ptr, sign })
        } else {
            Err(ParseErr::UnknownPrefix)
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ty.is_stdint() {
            if self.ptr {
                f.write_char('*')?;
            }
            if !self.sign {
                f.write_char('u')?;
            }
        } else {
            if !self.sign {
                f.write_str("unsigned ")?;
            }
            if self.ptr {
                f.write_char('*')?;
            }
        }

        let base_name = match self.ty {
            BaseTy::Long => "int32_t",
            BaseTy::Short => "int16_t",
            BaseTy::Char => "int8_t",
            BaseTy::Enum => "enum",
            BaseTy::Void => "void",
            BaseTy::Custom => "custom",
        };
        f.write_str(base_name)
    }
}

/// A FreeRTOS variable
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Variable {
    /// Type of the variable
    ty: Type,
    /// Name of the variable
    name: String,
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ty.write_prefix(f)?;
        f.write_str(&self.name)
    }
}

/// A FreeRTOS function
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Function {
    /// Return type if not privileged, `None` otherwise
    ret_ty: Option<Type>,
    /// Name of the function
    name: String,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.ret_ty {
            None => write!(f, "prv{}", self.name),
            Some(ret_ty) => {
                ret_ty.write_prefix(f)?;
                f.write_str(&self.name)
            }
        }
    }
}

/// A FreeRTOS macro
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Macro {
    /// Module with macro definition
    module: String,
    /// Name of the macro
    name: String,
}

impl std::fmt::Display for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.module, self.name)
    }
}

/// A FreeRTOS identifier
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Identifier {
    Var(Variable),
    Fun(Function),
    Mac(Macro),
}

impl Identifier {
    /// A short string naming this identifier's kind -- variable, function or macro
    pub(crate) fn kind_str(&self) -> &'static str {
        match self {
            Identifier::Var(_) => "variable",
            Identifier::Fun(_) => "function",
            Identifier::Mac(_) => "macro",
        }
    }

    /// The (return) type of this identifier, if it has one
    pub(crate) fn ty(&self) -> Option<Type> {
        match self {
            Identifier::Var(var) => Some(var.ty),
            Identifier::Fun(fun) => fun.ret_ty,
            Identifier::Mac(_) => None,
        }
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Var(var) => var.fmt(f),
            Identifier::Fun(fun) => fun.fmt(f),
            Identifier::Mac(mac) => mac.fmt(f),
        }
    }
}

impl std::str::FromStr for Identifier {
    type Err = ParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(&tokens::tokenize(s))
    }
}

fn parse(tokens: &[Token]) -> Result<Identifier, ParseErr> {
    let mut tokens = tokens.iter(); // Convert to iterator
    // Expect the first token to be a prefix
    let Some(Token::Prefix(prefix)) = tokens.next() else {
        return Err(ParseErr::MissingPrefix);
    };
    // Expect the second token to be a ident (name)
    let Some(Token::Ident(name)) = tokens.next() else {
        return Err(ParseErr::MissingIdent);
    };
    let name = name.to_owned();
    // If the third token is a '(' we're dealing with a fn
    let has_call = matches!(tokens.next(), Some(Token::LParam));

    // Try to parse prefix to type
    if let Ok(ty) = prefix.parse::<Type>() {
        // Parsing succeeded => variable or API function, differentiate using `has_call`
        if has_call {
            ensure(is_pascal_case(&name), ParseErr::IllegalFunctionName)?;
            Ok(Identifier::Fun(Function {
                ret_ty: Some(ty),
                name,
            }))
        } else {
            ensure(is_pascal_case(&name), ParseErr::IllegalVariableName)?;
            Ok(Identifier::Var(Variable { ty, name }))
        }
    } else {
        // Parsing failed => macro or priv function, differentiate using `is_priv` & valid macro nameing
        if prefix == "prv" {
            ensure(is_pascal_case(&name), ParseErr::IllegalFunctionName)?;
            Ok(Identifier::Fun(Function { ret_ty: None, name }))
        } else if is_snake_case(prefix) {
            ensure(is_screaming_snake_case(&name), ParseErr::IllegalMacroName)?;
            Ok(Identifier::Mac(Macro {
                module: prefix.to_owned(),
                name,
            }))
        } else {
            Err(ParseErr::UnknownPrefix)
        }
    }
}

/// Return `Err(err)` unless `cond` holds.
fn ensure(cond: bool, err: ParseErr) -> Result<(), ParseErr> {
    if cond { Ok(()) } else { Err(err) }
}

/// Checks whether `s` is written in `PascalCase`.
fn is_pascal_case(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_alphanumeric())
}

/// Checks whether `s` is written in `snake_case`.
fn is_snake_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Checks whether `s` is written in `SCREAMING_SNAKE_CASE`.
fn is_screaming_snake_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn display_formats_stdint_types() {
        assert_eq!(Type::UINT32_T.to_string(), "uint32_t");
        assert_eq!(Type::INT32_T.to_string(), "int32_t");
        assert_eq!(Type::UINT16_T.to_string(), "uint16_t");
        assert_eq!(Type::INT16_T.to_string(), "int16_t");
        assert_eq!(Type::UINT8_T.to_string(), "uint8_t");
        assert_eq!(Type::INT8_T.to_string(), "int8_t");
    }

    #[test]
    fn display_formats_pointer_to_stdint_type() {
        assert_eq!(Type::UINT8_T.to_ptr().to_string(), "*uint8_t");
        assert_eq!(Type::INT32_T.to_ptr().to_string(), "*int32_t");
    }

    #[test]
    fn display_formats_void() {
        assert_eq!(Type::VOID.to_string(), "void");
    }

    #[test]
    fn display_formats_non_stdint_types() {
        // Non-stdint types put `unsigned`/`*` differently: `unsigned ` as a
        // word prefix, and the `*` after it rather than before.
        assert_eq!(
            Type {
                ty: BaseTy::Enum,
                ptr: false,
                sign: true
            }
            .to_string(),
            "enum"
        );
        assert_eq!(
            Type {
                ty: BaseTy::Enum,
                ptr: false,
                sign: false
            }
            .to_string(),
            "unsigned enum"
        );
        assert_eq!(
            Type {
                ty: BaseTy::Enum,
                ptr: true,
                sign: true
            }
            .to_string(),
            "*enum"
        );
    }

    #[test]
    fn display_formats_custom_type() {
        let ty = Type {
            ty: BaseTy::Custom,
            ptr: false,
            sign: true,
        };
        assert_eq!(ty.to_string(), "custom");
        assert_eq!(
            Variable {
                ty,
                name: "QueueHandle".to_string()
            }
            .to_string(),
            "xQueueHandle"
        );
        assert_eq!(
            Function {
                ret_ty: Some(ty),
                name: "TaskCreate".to_string()
            }
            .to_string(),
            "xTaskCreate"
        );
    }

    #[test]
    fn display_formats_variable() {
        assert_eq!(
            Variable {
                ty: Type::UINT32_T,
                name: "A".to_string()
            }
            .to_string(),
            "ulA"
        );
        assert_eq!(
            Variable {
                ty: Type::INT8_T.to_ptr(),
                name: "Counter".to_string()
            }
            .to_string(),
            "pcCounter"
        );
    }

    #[test]
    fn display_formats_api_function() {
        assert_eq!(
            Function {
                ret_ty: Some(Type::VOID),
                name: "QueueDelete".to_string()
            }
            .to_string(),
            "vQueueDelete"
        );
        assert_eq!(
            Function {
                ret_ty: Some(Type::UINT32_T),
                name: "TaskNotifyTake".to_string()
            }
            .to_string(),
            "ulTaskNotifyTake"
        );
        assert_eq!(
            Function {
                ret_ty: Some(Type::VOID.to_ptr()),
                name: "TimerGetTimerID".to_string()
            }
            .to_string(),
            "pvTimerGetTimerID"
        );
    }

    #[test]
    fn display_formats_privileged_function() {
        assert_eq!(
            Function {
                ret_ty: None,
                name: "YieldCore".to_string()
            }
            .to_string(),
            "prvYieldCore"
        );
    }

    #[test]
    fn display_formats_macro() {
        assert_eq!(
            Macro {
                module: "port".to_string(),
                name: "YIELD_CORE".to_string()
            }
            .to_string(),
            "portYIELD_CORE"
        );
        assert_eq!(
            Macro {
                module: "config".to_string(),
                name: "USE_MUTEXES".to_string()
            }
            .to_string(),
            "configUSE_MUTEXES"
        );
    }

    #[test]
    fn display_identifier_delegates_to_variant() {
        assert_eq!(
            Identifier::Var(Variable {
                ty: Type::UINT32_T,
                name: "A".to_string()
            })
            .to_string(),
            "ulA"
        );
        assert_eq!(
            Identifier::Fun(Function {
                ret_ty: None,
                name: "YieldCore".to_string()
            })
            .to_string(),
            "prvYieldCore"
        );
        assert_eq!(
            Identifier::Mac(Macro {
                module: "port".to_string(),
                name: "YIELD_CORE".to_string()
            })
            .to_string(),
            "portYIELD_CORE"
        );
    }

    #[test]
    fn parse_recognizes_variable() {
        // `A` alone trivially satisfies the macro-name (all-caps) check, so
        // this only classifies correctly if a decodable type prefix wins.
        assert_eq!(
            "ulA".parse::<Identifier>(),
            Ok(Identifier::Var(Variable {
                ty: Type::UINT32_T,
                name: "A".to_string()
            }))
        );
        assert_eq!(
            "pcCounter".parse::<Identifier>(),
            Ok(Identifier::Var(Variable {
                ty: Type::INT8_T.to_ptr(),
                name: "Counter".to_string()
            }))
        );
    }

    #[test]
    fn parse_recognizes_custom_type() {
        assert_eq!(
            "xQueueHandle".parse::<Identifier>(),
            Ok(Identifier::Var(Variable {
                ty: Type {
                    ty: BaseTy::Custom,
                    ptr: false,
                    sign: true
                },
                name: "QueueHandle".to_string()
            }))
        );
        assert_eq!(
            "xTaskCreate()".parse::<Identifier>(),
            Ok(Identifier::Fun(Function {
                ret_ty: Some(Type {
                    ty: BaseTy::Custom,
                    ptr: false,
                    sign: true
                }),
                name: "TaskCreate".to_string()
            }))
        );
    }

    #[test]
    fn parse_recognizes_privileged_function_without_call() {
        // `prv` is exclusively a function-privacy marker in FreeRTOS, so it
        // doesn't need a trailing `(...)` to be recognized as a function.
        assert_eq!(
            "prvCounter".parse::<Identifier>(),
            Ok(Identifier::Fun(Function {
                ret_ty: None,
                name: "Counter".to_string()
            }))
        );
    }

    #[test]
    fn parse_recognizes_api_function() {
        assert_eq!(
            "vQueueDelete()".parse::<Identifier>(),
            Ok(Identifier::Fun(Function {
                ret_ty: Some(Type::VOID),
                name: "QueueDelete".to_string()
            }))
        );
        assert_eq!(
            "ulTaskNotifyTake(x)".parse::<Identifier>(),
            Ok(Identifier::Fun(Function {
                ret_ty: Some(Type::UINT32_T),
                name: "TaskNotifyTake".to_string()
            }))
        );
        assert_eq!(
            "pvTimerGetTimerID()".parse::<Identifier>(),
            Ok(Identifier::Fun(Function {
                ret_ty: Some(Type::VOID.to_ptr()),
                name: "TimerGetTimerID".to_string()
            }))
        );
    }

    #[test]
    fn parse_recognizes_privileged_function() {
        assert_eq!(
            "prvYieldCore()".parse::<Identifier>(),
            Ok(Identifier::Fun(Function {
                ret_ty: None,
                name: "YieldCore".to_string()
            }))
        );
    }

    #[test]
    fn parse_recognizes_macro() {
        assert_eq!(
            "portYIELD_CORE".parse::<Identifier>(),
            Ok(Identifier::Mac(Macro {
                module: "port".to_string(),
                name: "YIELD_CORE".to_string()
            }))
        );
        assert_eq!(
            "configUSE_MUTEXES".parse::<Identifier>(),
            Ok(Identifier::Mac(Macro {
                module: "config".to_string(),
                name: "USE_MUTEXES".to_string()
            }))
        );
    }

    #[test]
    fn parse_recognizes_macro_with_digit_and_underscore_prefix() {
        assert_eq!(
            "arm_v8YIELD_CORE".parse::<Identifier>(),
            Ok(Identifier::Mac(Macro {
                module: "arm_v8".to_string(),
                name: "YIELD_CORE".to_string()
            }))
        );
    }

    #[test]
    fn parse_does_not_misclassify_macro_as_function() {
        // Regression: a naive type-prefix check could greedily match the
        // leading `c` of `config` as the char-type prefix, mangling the
        // rest into a bogus function name instead of recognizing the macro.
        assert_eq!(
            "configUSE_PREEMPTION".parse::<Identifier>(),
            Ok(Identifier::Mac(Macro {
                module: "config".to_string(),
                name: "USE_PREEMPTION".to_string()
            }))
        );
    }

    #[test]
    fn parse_fails_on_empty_token_stream() {
        assert_eq!(parse(&[]), Err(ParseErr::MissingPrefix));
    }

    #[test]
    fn parse_fails_on_incomplete_input() {
        assert_eq!("".parse::<Identifier>(), Err(ParseErr::MissingIdent));
        assert_eq!("(".parse::<Identifier>(), Err(ParseErr::MissingIdent));
    }

    #[test]
    fn parse_fails_without_type_prefix() {
        // No lowercase run precedes the leading capital, so this tokenizes
        // to an empty `Prefix` -- neither a known type, `prv`, nor snake_case.
        assert_eq!(
            "QueueDelete".parse::<Identifier>(),
            Err(ParseErr::UnknownPrefix)
        );
    }

    #[test]
    fn parse_fails_without_any_ident() {
        // No uppercase letter ever appears, so no `Ident` token is produced.
        assert_eq!("hello".parse::<Identifier>(), Err(ParseErr::MissingIdent));
    }

    #[test]
    fn parse_fails_on_illegal_api_function_name() {
        // The name has an underscore, so it isn't `PascalCase`.
        assert_eq!(
            "vQueue_Delete()".parse::<Identifier>(),
            Err(ParseErr::IllegalFunctionName)
        );
    }

    #[test]
    fn parse_fails_on_illegal_variable_name() {
        assert_eq!(
            "pcQueue_Delete".parse::<Identifier>(),
            Err(ParseErr::IllegalVariableName)
        );
    }

    #[test]
    fn parse_fails_on_illegal_privileged_function_name() {
        assert_eq!(
            "prvQueue_Delete()".parse::<Identifier>(),
            Err(ParseErr::IllegalFunctionName)
        );
    }

    #[test]
    fn parse_fails_on_illegal_macro_name() {
        // The name has a lowercase run, so it isn't `SCREAMING_SNAKE_CASE`.
        assert_eq!(
            "configUSE_mutexes".parse::<Identifier>(),
            Err(ParseErr::IllegalMacroName)
        );
    }
}
