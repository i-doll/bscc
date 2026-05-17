//! Shared formatting helpers for exporters.
//!
//! Integer formatting honors `LC_NUMERIC` from the environment (via
//! `num-format`'s `SystemLocale`). Norwegian users get `4 433`; US users get
//! `4,433`; German users get `4.433`. POSIX/C locales (no grouping) yield
//! `4433` unchanged. When `SystemLocale` can't be loaded we fall back to
//! `Locale::en` so output is never just runs of digits in unusual setups.

use num_format::{Locale, SystemLocale, ToFormattedString};
use once_cell::sync::OnceCell;

enum Formatter {
    // SystemLocale is much larger than Locale; box it to keep the enum small.
    System(Box<SystemLocale>),
    Fallback(Locale),
}

fn formatter() -> &'static Formatter {
    static F: OnceCell<Formatter> = OnceCell::new();
    F.get_or_init(|| {
        SystemLocale::default()
            .map(|l| Formatter::System(Box::new(l)))
            .unwrap_or(Formatter::Fallback(Locale::en))
    })
}

/// Locale-aware integer formatting. Uses the system locale resolved at first
/// call; falls back to `Locale::en` if the environment can't be parsed.
pub fn fmt_int(n: u32) -> String {
    match formatter() {
        Formatter::System(loc) => n.to_formatted_string(loc.as_ref()),
        Formatter::Fallback(loc) => n.to_formatted_string(loc),
    }
}

/// `fmt_int` variant that takes any `num_format::Format`. Useful for tests
/// that need deterministic output regardless of `LC_NUMERIC`.
pub fn fmt_int_with<F: num_format::Format>(n: u32, loc: &F) -> String {
    n.to_formatted_string(loc)
}

/// Language family groupings. When the table/HTML exporter sees ≥2 members
/// of a family, it emits an extra sub-total row labeled with `display`.
pub struct Family {
    pub display: &'static str,
    pub members: &'static [&'static str],
}

pub const FAMILIES: &[Family] = &[
    Family {
        display: "TS + TSX",
        members: &["TypeScript", "TSX"],
    },
    Family {
        display: "JS + JSX",
        members: &["JavaScript", "JSX"],
    },
    Family {
        display: "C + C++",
        members: &["C", "C++"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use num_format::Locale;

    #[test]
    fn en_us_uses_commas() {
        assert_eq!(fmt_int_with(0, &Locale::en), "0");
        assert_eq!(fmt_int_with(123, &Locale::en), "123");
        assert_eq!(fmt_int_with(1234, &Locale::en), "1,234");
        assert_eq!(fmt_int_with(1_234_567, &Locale::en), "1,234,567");
    }

    #[test]
    fn fr_uses_narrow_no_break_space() {
        // French standard grouping uses U+202F NARROW NO-BREAK SPACE.
        let v = fmt_int_with(1_234_567, &Locale::fr);
        assert!(v.contains('\u{202f}'), "fr grouping = {v:?}");
    }

    #[test]
    fn de_uses_periods() {
        assert_eq!(fmt_int_with(1_234_567, &Locale::de), "1.234.567");
    }
}
