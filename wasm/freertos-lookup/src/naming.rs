//! FreeRTOS naming convention
//!
//! Todo: Rewrite using recursive descent parser?
//! Doc: https://freertos.org/Documentation/02-Kernel/06-Coding-guidelines/02-FreeRTOS-Coding-Standard-and-Style-Guide#naming-conventions

type NamingParseError = ();

// #[derive(Debug, PartialEq, Eq)]
// pub enum _Type {
//     StdInt(_StdInt),
//     Enum,
//     Void,
//     Custom(Option<&'static str>),
// }

#[derive(Debug, Clone)]
pub enum Type {
    Value(Sign),
    Pointer(Sign),
}

impl TryFrom<&str> for Type {
    type Error = NamingParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (c, others) = value.split_at(1);
        match c.chars().nth(0).unwrap() {
            'p' => others.try_into().map(|x| Self::Pointer(x)),
            _ => value.try_into().map(|x| Self::Value(x)),
        }
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        match self {
            Type::Value(v) => match v {
                Sign::Signed(Base::Long) => "int32_t".to_string(),
                Sign::Signed(Base::Short) => "int16_t".to_string(),
                Sign::Signed(Base::Char) => "int8_t".to_string(),
                Sign::Signed(Base::Enum) => "enum".to_string(),
                Sign::Signed(Base::Void) => "void".to_string(),
                Sign::Signed(Base::Custom(t)) => t.unwrap_or("custom").to_string(),
                Sign::Unsigned(Base::Long) => "uint32_t".to_string(),
                Sign::Unsigned(Base::Short) => "uint16_t".to_string(),
                Sign::Unsigned(Base::Char) => "uint8_t".to_string(),
                Sign::Unsigned(Base::Enum) => unreachable!(),
                Sign::Unsigned(Base::Void) => unreachable!(),
                Sign::Unsigned(Base::Custom(t)) => format!("unsigned {}", t.unwrap_or("custom")),
            },
            Type::Pointer(v) => format!("*{}", Type::Value(v.clone()).to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Sign {
    Signed(Base),
    Unsigned(Base),
}

impl TryFrom<&str> for Sign {
    type Error = NamingParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (c, others) = value.split_at(1);
        match c.chars().nth(0).unwrap() {
            'u' => others.try_into().map(|x| Self::Unsigned(x)),
            _ => value.try_into().map(|x| Self::Signed(x)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Base {
    Long,
    Short,
    Char,
    Custom(Option<&'static str>),
    Enum,
    Void,
}

impl TryFrom<&str> for Base {
    type Error = NamingParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (c, _) = value.split_at(1);
        match c.chars().nth(0).unwrap() {
            'l' => Ok(Base::Long),
            's' => Ok(Base::Short),
            'c' => Ok(Base::Char),
            'e' => Ok(Base::Enum),
            'v' => Ok(Base::Void),
            'x' => Ok(Base::Custom(None)),
            _ => Err(()),
        }
    }
}

pub fn prefix_to_ty<'a>(name: impl AsRef<str>) -> String {
    let ty: Type = name.as_ref().try_into().unwrap();
    ty.to_string()
}

pub struct NonStdInt;

#[allow(non_snake_case)]
impl NonStdInt {
    pub(crate) const fn BaseType_t() -> Type {
        Type::Value(Sign::Signed(Base::Custom(Some("BaseType_t"))))
    }

    pub(crate) const fn TaskHandle_t() -> Type {
        Type::Value(Sign::Signed(Base::Custom(Some("TaskHandle_t"))))
    }

    pub(crate) const fn TickType_t() -> Type {
        Type::Value(Sign::Signed(Base::Custom(Some("TickType_t"))))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_base() {
        assert!(matches!("lVar".try_into().unwrap(), Base::Long));
        assert!(matches!("sVar".try_into().unwrap(), Base::Short));
        assert!(matches!("cVar".try_into().unwrap(), Base::Char));
        assert!(matches!("vVar".try_into().unwrap(), Base::Void));
        assert!(matches!("eVar".try_into().unwrap(), Base::Enum));
        assert!(matches!("xVar".try_into().unwrap(), Base::Custom(None)));
    }

    #[test]
    fn parse_sign() {
        assert!(matches!(
            "lVar".try_into().unwrap(),
            Sign::Signed(Base::Long)
        ));
        assert!(matches!(
            "ulVar".try_into().unwrap(),
            Sign::Unsigned(Base::Long)
        ));
        assert!(matches!(
            "xVar".try_into().unwrap(),
            Sign::Signed(Base::Custom(None))
        ));
        assert!(matches!(
            "uxVar".try_into().unwrap(),
            Sign::Unsigned(Base::Custom(None))
        ));
    }

    #[test]
    fn parse_pointer() {
        assert!(matches!(
            "lVar".try_into().unwrap(),
            Type::Value(Sign::Signed(Base::Long))
        ));
        assert!(matches!(
            "pulVar".try_into().unwrap(),
            Type::Pointer(Sign::Unsigned(Base::Long))
        ));
    }
}
