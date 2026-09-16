//! Strongly typed identifiers.
//!
//! All identifiers are dense `u64` newtypes. They are cheap to copy, hash,
//! and order, which keeps flowsheet maps deterministic (`BTreeMap`) and
//! eliminates the "which string did I typo?" class of bugs.

use core::fmt;

macro_rules! define_id {
    ($(#[$doc:meta])* $name:ident, $prefix:literal) => {
        $(#[$doc])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
        pub struct $name(pub u64);

        impl $name {
            /// Raw numeric value.
            #[must_use]
            pub const fn value(self) -> u64 {
                self.0
            }
        }

        impl From<u64> for $name {
            fn from(v: u64) -> Self {
                Self(v)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!($prefix, "{}"), self.0)
            }
        }
    };
}

define_id!(
    /// Identifier of a material stream within a flowsheet.
    StreamId,
    "S"
);
define_id!(
    /// Identifier of a unit operation within a flowsheet.
    UnitId,
    "U"
);
define_id!(
    /// Index of a component within a property package's component list.
    ComponentId,
    "C"
);
define_id!(
    /// Identifier of a flowsheet.
    FlowsheetId,
    "F"
);
define_id!(
    /// Port index on a unit operation (0-based; convention: inlets first).
    PortId,
    "P"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_display_with_prefix() {
        assert_eq!(StreamId(7).to_string(), "S7");
        assert_eq!(UnitId(12).to_string(), "U12");
        assert_eq!(ComponentId(0).to_string(), "C0");
    }

    #[test]
    fn ids_order_numerically() {
        let mut ids = vec![StreamId(10), StreamId(2), StreamId(1)];
        ids.sort();
        assert_eq!(ids, vec![StreamId(1), StreamId(2), StreamId(10)]);
    }
}
