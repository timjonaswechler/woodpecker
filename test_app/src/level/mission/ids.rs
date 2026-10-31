//! Indices are checked when an identity enters the gameplay interfaces.
macro_rules! identity {
    ($name:ident, $count:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(u8);
        impl $name {
            pub const ALL: [Self; $count] = {
                let mut ids = [Self(0); $count];
                let mut i = 0;
                while i < $count {
                    ids[i] = Self(i as u8);
                    i += 1;
                }
                ids
            };
            pub const fn new(index: usize) -> Option<Self> {
                if index < $count {
                    Some(Self(index as u8))
                } else {
                    None
                }
            }
            pub const fn index(self) -> usize {
                self.0 as usize
            }
        }
    };
}
identity!(PartId, 2);
identity!(ValveId, 2);
identity!(ScanId, 3);
identity!(CellId, 3);
identity!(TargetId, 3);
identity!(PortalId, 6);

identity!(CoinId, 24);
