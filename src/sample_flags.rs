#[cfg(feature = "use_serde")]
use serde::Serialize;

#[cfg_attr(feature = "use_serde", derive(Serialize))]
#[cfg_attr(feature = "use_serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum IsLeading {
    Unknown = 0,
    LeadingDep = 1,
    NotLeading = 2,
    Leading = 3,
}
impl Default for IsLeading {
    fn default() -> Self {
        IsLeading::Unknown
    }
}
impl IsLeading {
    pub fn is_unknown(&self) -> bool {
        *self == Self::Unknown
    }
}

#[cfg_attr(feature = "use_serde", derive(Serialize))]
#[cfg_attr(feature = "use_serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum DependsOn {
    Unknown = 0,
    Depends = 1,
    NotDepends = 2,
}
impl Default for DependsOn {
    fn default() -> Self {
        DependsOn::Unknown
    }
}
impl DependsOn {
    pub fn is_unknown(&self) -> bool {
        *self == Self::Unknown
    }
}

#[cfg_attr(feature = "use_serde", derive(Serialize))]
#[cfg_attr(feature = "use_serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum DependedOn {
    Unknown = 0,
    Depended = 1,
    NotDepended = 2,
}
impl Default for DependedOn {
    fn default() -> Self {
        DependedOn::Unknown
    }
}
impl DependedOn {
    pub fn is_unknown(&self) -> bool {
        *self == Self::Unknown
    }
}

#[cfg_attr(feature = "use_serde", derive(Serialize))]
#[cfg_attr(feature = "use_serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum HasRedundancy {
    Unknown = 0,
    Redundant = 1,
    NotRedundant = 2,
}
impl Default for HasRedundancy {
    fn default() -> Self {
        HasRedundancy::Unknown
    }
}
impl HasRedundancy {
    pub fn is_unknown(&self) -> bool {
        *self == Self::Unknown
    }
}

#[cfg_attr(feature = "use_serde", derive(Serialize))]
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct SampleFlags {
    #[cfg_attr(
        feature = "use_serde",
        serde(default, skip_serializing_if = "IsLeading::is_unknown")
    )]
    pub is_leading: IsLeading,
    #[cfg_attr(
        feature = "use_serde",
        serde(default, skip_serializing_if = "DependsOn::is_unknown")
    )]
    pub depends_on: DependsOn,
    #[cfg_attr(
        feature = "use_serde",
        serde(default, skip_serializing_if = "DependedOn::is_unknown")
    )]
    pub depended_on: DependedOn,
    #[cfg_attr(
        feature = "use_serde",
        serde(default, skip_serializing_if = "HasRedundancy::is_unknown")
    )]
    pub has_redundancy: HasRedundancy,
    pub padding_value: u8,
    pub is_non_sync: bool,
    pub degredation_priority: u16,
}

impl SampleFlags {
    pub fn new(flags: u32) -> Self {
        let is_leading = match (flags >> 26) & 0b11 {
            0 => IsLeading::Unknown,
            1 => IsLeading::LeadingDep,
            2 => IsLeading::NotLeading,
            3 => IsLeading::Leading,
            _ => unreachable!(),
        };

        let depends_on = match (flags >> 24) & 0b11 {
            0 => DependsOn::Unknown,
            1 => DependsOn::Depends,
            2 => DependsOn::NotDepends,
            3 => panic!("got reserved"),
            _ => unreachable!(),
        };

        let depended_on = match (flags >> 22) & 0b11 {
            0 => DependedOn::Unknown,
            1 => DependedOn::Depended,
            2 => DependedOn::NotDepended,
            3 => panic!("got reserved"),
            _ => unreachable!(),
        };

        let has_redundancy = match (flags >> 20) & 0b11 {
            0 => HasRedundancy::Unknown,
            1 => HasRedundancy::Redundant,
            2 => HasRedundancy::NotRedundant,
            3 => panic!("got reserved"),
            _ => unreachable!(),
        };

        let padding_value = ((flags >> 17) & 0b111) as u8;

        let is_non_sync = (flags >> 16) != 0;

        SampleFlags {
            is_leading,
            depends_on,
            depended_on,
            has_redundancy,
            padding_value,
            is_non_sync,
            degredation_priority: flags as u16,
        }
    }
    pub fn to_u32(&self) -> u32 {
        let mut flags = 0;
        flags |= (self.is_leading as u32) << 26;
        flags |= (self.depends_on as u32) << 24;
        flags |= (self.depended_on as u32) << 22;
        flags |= (self.has_redundancy as u32) << 20;
        flags |= (self.padding_value as u32 & 0b111) << 17;
        flags |= (self.is_non_sync as u32) << 16;
        flags |= self.degredation_priority as u32;
        flags
    }
}
