//! "Abstract Syntax Tree" for FreeRTOS identifiers
//!
use std::fmt::Write;

/// A FreeRTOS identifier
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Ident {
    Variable(Var),
    Function(Fun),
    Macro(Macro),
}

impl Ident {
    /// Get the name of this identifier -- i.e. the identifier w/o type or
    /// privilege related prefixes.
    pub fn name(&self) -> &str {
        match &self {
            Ident::Variable(var) => &var.name,
            Ident::Function(fun) => fun.name(),
            Ident::Macro(mac) => &mac.name,
        }
    }

    /// Get the (return) type of this identifier (if known)
    pub fn ty(&self) -> Option<Ty> {
        match &self {
            Ident::Variable(var) => Some(var.ty),
            Ident::Function(fun) => fun.ty(),
            Ident::Macro(_) => None,
        }
    }

    /// Get the full indentifier
    pub fn ident_str(&self) -> String {
        match self {
            Ident::Variable(var) => var.to_string(),
            Ident::Function(fun) => fun.to_string(),
            Ident::Macro(mac) => mac.to_string(),
        }
    }

    /// Get a string representation of the kind -- i.e. variable, function or macro
    pub const fn kind_str(&self) -> &str {
        match self {
            Ident::Variable(_) => "variable",
            Ident::Function(_) => "function",
            Ident::Macro(_) => "macro",
        }
    }

    /// Set the name of wrapped [BaseTy::Custom] if present
    pub fn fill_custom_type(&mut self, name: &'static str) -> bool {
        match self {
            Ident::Variable(var) => var.fill_custom_type(name),
            Ident::Function(fun) => fun.fill_custom_type(name),
            Ident::Macro(_) => false,
        }
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ident::Variable(var) => write!(f, "variable: {}", var),
            Ident::Function(fun) => write!(f, "function: {}", fun),
            Ident::Macro(mac) => write!(f, "macro: {}", mac),
        }
    }
}

/// A FreeRTOS variable
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Var {
    /// The type of the variable
    ty: Ty,
    /// The name of the variable
    name: String,
}

impl Var {
    pub const fn new(ty: Ty, name: String) -> Self {
        Self { ty, name }
    }

    /// Set the name of wrapped [BaseTy::Custom] if present
    fn fill_custom_type(&mut self, name: &'static str) -> bool {
        match self.ty.ty {
            BaseTy::Custom(_) => {
                self.ty.ty = BaseTy::Custom(Some(name));
                true
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.ty.compose())?;
        f.write_str(&self.name)
    }
}

/// A FreeRTOS function
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Fun {
    /// A privileged function
    Privileged { name: String },
    /// An function that follows the API naming convention
    Api { ret: Ty, name: String },
}

impl Fun {
    fn ty(&self) -> Option<Ty> {
        match self {
            Fun::Privileged { name: _ } => None,
            Fun::Api { ret, name: _ } => Some(ret.clone()),
        }
    }

    fn name(&self) -> &str {
        match self {
            Fun::Privileged { name } => &name,
            Fun::Api { ret: _, name } => &name,
        }
    }

    /// Set the name of wrapped [BaseTy::Custom] if present
    fn fill_custom_type(&mut self, name: &'static str) -> bool {
        match self {
            Fun::Privileged { name: _ } => false,
            Fun::Api { ret, name: _ } => match ret.ty {
                BaseTy::Custom(_) => {
                    ret.ty = BaseTy::Custom(Some(name));
                    true
                }
                _ => false,
            },
        }
    }
}

impl std::fmt::Display for Fun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fun::Privileged { name } => write!(f, "prv{}", name),
            Fun::Api { ret, name } => {
                f.write_str(&ret.compose())?;
                f.write_str(name)
            }
        }
    }
}

/// A FreeRTOS macro
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Macro {
    /// The file where the macro is defined
    file: String,
    /// The name of the macro
    name: String,
}

impl Macro {
    pub const fn new(file: String, name: String) -> Self {
        Self { file, name }
    }
}

impl std::fmt::Display for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file, self.name)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct Ty {
    /// Is this a pointer type?
    ptr: bool,
    /// Is this a signed type?
    sign: bool,
    /// The base type
    ty: BaseTy,
}

impl Ty {
    pub const UINT32_T: Self = Self::new_long(false);
    pub const INT32_T: Self = Self::new_long(true);
    pub const UINT16_T: Self = Self::new_short(false);
    pub const INT16_T: Self = Self::new_short(true);
    pub const UINT8_T: Self = Self::new_char(false);
    pub const INT8_T: Self = Self::new_char(true);
    pub const VOID: Self = Self::new_void();

    #[rustfmt::skip]
    pub const fn new_long(sign: bool) -> Self {
        Self { ptr: false, sign, ty: BaseTy::Long }
    }

    #[rustfmt::skip]
    pub const fn new_short(sign: bool) -> Self {
        Self { ptr: false, sign, ty: BaseTy::Short }
    }

    #[rustfmt::skip]
    pub const fn new_char(sign: bool) -> Self {
        Self { ptr: false, sign, ty: BaseTy::Char }
    }

    #[rustfmt::skip]
    pub const fn new_enum(sign: bool) -> Self {
        Self { ptr: false, sign, ty: BaseTy::Enum }
    }

    pub const fn new_void() -> Self {
        // Should a "default" void be signed?
        // Makes no sense but is consistent with parsing output
        // and allows for more generic formatting logic.
        Self {
            ptr: false,
            sign: true,
            ty: BaseTy::Void,
        }
    }

    /// Create a new custom type with optional name
    pub const fn new(sign: bool, name: Option<&'static str>) -> Self {
        Self {
            ptr: false,
            sign,
            ty: BaseTy::Custom(name),
        }
    }

    /// Convert a type to its pointer variant
    pub const fn ptr(mut self) -> Self {
        self.ptr = true;
        self
    }

    /// Compose an ident prefix for this type
    ///
    /// FIXME: move to parse module??
    fn compose(&self) -> String {
        let mut buf = String::new();
        if self.ptr {
            buf.push('p');
        }
        if !self.sign {
            buf.push('u');
        }
        buf.push(match self.ty {
            BaseTy::Long => 'l',
            BaseTy::Short => 's',
            BaseTy::Char => 'c',
            BaseTy::Enum => 'e',
            BaseTy::Void => 'v',
            BaseTy::Custom(_) => 'x',
        });
        buf
    }
}

impl std::fmt::Display for Ty {
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
            BaseTy::Custom(Some(n)) => n,
            BaseTy::Custom(None) => "non stdint type",
        };
        f.write_str(base_name)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum BaseTy {
    Long,
    Short,
    Char,
    Enum,
    Void,
    Custom(Option<&'static str>),
}

impl BaseTy {
    const fn is_stdint(&self) -> bool {
        match self {
            BaseTy::Long => true,
            BaseTy::Short => true,
            BaseTy::Char => true,
            _ => false,
        }
    }
}
