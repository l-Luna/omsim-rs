use std::collections::HashMap;
use std::fmt::Debug;
use bitflags::bitflags;

// Puzzle and solution files

#[derive(Debug)]
pub struct Puzzle{
    pub name: String,
    pub creator_id: u64,
    pub reagents: Vec<Molecule>,
    pub products: Vec<Molecule>,
    pub product_multiplier: i32,
    pub permissions: Permissions,

    pub production_info: Option<ProductionInfo>
}

#[derive(Debug)]
pub struct Solution{
    pub name: String,
    pub metrics: Option<Metrics>,
    pub parts: Vec<Part>
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Metrics{
    pub cycles: i32,
    pub cost: i32,
    pub area: i32,
    pub instructions: i32
}

bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Permissions: u64{
        const SIMPLE_ARM = 1;
        const MULTI_ARMS = 2;
        const PISTON_ARM = 4;
        const TRACK = 8;
        const BONDER = 0x00000100;
        const UNBONDER = 0x00000200;
        const MULTI_BONDER = 0x00000400;
        const TRIPLEX_BONDER = 0x00000800;
        const CALCIFICATION = 0x00001000;
        const DUPLICATION = 0x00002000;
        const PROJECTION = 0x00004000;
        const PURIFICATION = 0x00008000;
        const ANIMISMUS = 0x00010000;
        const DISPOSAL = 0x00020000;
        const QUINTESSENCE = 0x00040000;
        const BERLO = 0x10000000;

        const GRAB_TURN_INSTRUCTIONS = 0x00400000;
        const DROP_INSTRUCTION = 0x00800000;
        const RESET_INSTRUCTION = 0x01000000;
        const REPEAT_INSTRUCTION = 0x02000000;
        const PIVOT_INSTRUCTIONS = 0x04000000;

        const DEFAULT_PERMISSIONS
            = Self::SIMPLE_ARM.bits()
            | Self::MULTI_ARMS.bits()
            | Self::PISTON_ARM.bits()
            | Self::TRACK.bits()
            | Self::BONDER.bits()
            | Self::UNBONDER.bits()
            | Self::MULTI_BONDER.bits()
            | Self::CALCIFICATION.bits()
            | Self::GRAB_TURN_INSTRUCTIONS.bits()
            | Self::DROP_INSTRUCTION.bits()
            | Self::RESET_INSTRUCTION.bits()
            | Self::REPEAT_INSTRUCTION.bits()
            | Self::PIVOT_INSTRUCTIONS.bits();

        // mark unknown bits as being potentially used
        const _ = !0;
    }
}

// Production info

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionInfo{
    pub isolation: bool,
    pub chambers: Vec<Chamber>,
    pub conduits: Vec<Conduit>
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chamber{
    pub pos: HexIndex,
    pub ty: ChamberType
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conduit{
    pub pos_a: HexIndex,
    pub pos_b: HexIndex,
    pub hexes: Vec<HexIndex>
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChamberType{
    Small, SmallWide, SmallWider,
    Medium, MediumWide,
    Large
}

impl ChamberType{
    // it's not worth an extra dependency to autogen this
    pub fn from_name(name: &str) -> Option<ChamberType>{
        Some(match name{
            "Small" => ChamberType::Small,
            "SmallWide" => ChamberType::SmallWide,
            "SmallWider" => ChamberType::SmallWider,
            "Medium" => ChamberType::Medium,
            "MediumWide" => ChamberType::MediumWide,
            "Large" => ChamberType::Large,
            _ => return None
        })
    }
}

// Atoms and molecules

#[derive(Debug, PartialEq, Eq)]
pub struct Molecule{
    pub atoms: HashMap<HexIndex, Atom>,
    pub bonds: Vec<Bond>
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bond{
    pub start: HexIndex,
    pub end: HexIndex,
    pub ty: BondType
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Atom{
    #[default] Salt, Air, Earth, Fire, Water,
    Quicksilver, Vitae, Mors,
    Lead, Tin, Iron, Copper, Silver, Gold,
    Quintessence,
    Repeat
}

impl Atom {
    pub fn from_id(id: u8) -> Option<Atom>{
        Some(match id {
            1 => Atom::Salt,
            2 => Atom::Air,
            3 => Atom::Earth,
            4 => Atom::Fire,
            5 => Atom::Water,
            6 => Atom::Quicksilver,
            7 => Atom::Gold,
            8 => Atom::Silver,
            9 => Atom::Copper,
            10 => Atom::Iron,
            11 => Atom::Tin,
            12 => Atom::Lead,
            13 => Atom::Vitae,
            14 => Atom::Mors,
            15 => Atom::Repeat,
            16 => Atom::Quintessence,
            _ => return None
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum BondType{
    #[default] Normal,
    Triplex{ red: bool, black: bool, yellow: bool }
}

// Parts

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Part{
    pub ty: PartType,
    pub pos: HexIndex,
    pub rotation: i32,
    pub arm_number: i32,
    pub arm_length: i32,
    pub index: i32,
    pub conduit_index: i32,
    pub track_hexes: Vec<HexIndex>,
    pub conduit_hexes: Vec<HexIndex>,
    pub instructions: Vec<(Instruction, i32)>
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PartType{
    // IO
    Input, Output, PolymerOutput,
    // Mechanisms
    Arm, BiArm, TriArm, HexArm, PistonArm,
    Track, Berlo,
    // Glyphs
    Equilibrium, Bonding, MultiBonding, Debonding, Calcification,
    Projection, Purification,
    Duplication, Animismus,
    Unification, Dispersion,
    TriplexBonding,
    Disposal,
    // Misc
    Conduit
}

impl PartType {
    pub fn from_name(name: &str) -> Option<PartType>{
        Some(match name{
            "input" => PartType::Input,
            "out-std" => PartType::Output,
            "out-rep" => PartType::PolymerOutput,
            "arm1" => PartType::Arm,
            "arm2" => PartType::BiArm,
            "arm3" => PartType::TriArm,
            "arm6" => PartType::HexArm,
            "piston" => PartType::PistonArm,
            "track" => PartType::Track,
            "baron" => PartType::Berlo,
            "glyph-marker" => PartType::Equilibrium,
            "bonder" => PartType::Bonding,
            "bonder-speed" => PartType::MultiBonding,
            "unbonder" => PartType::Debonding,
            "glyph-calcification" => PartType::Calcification,
            "glyph-projection" => PartType::Projection,
            "glyph-purification" => PartType::Purification,
            "glyph-duplication" => PartType::Duplication,
            "glyph-life-and-death" => PartType::Animismus,
            "glyph-unification" => PartType::Unification,
            "glyph-dispersion" => PartType::Dispersion,
            "bonder-prisma" => PartType::TriplexBonding,
            "glyph-disposal" => PartType::Disposal,
            "pipe" => PartType::Conduit,
            _ => return None
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum Instruction{
    #[default]
    Blank,
    Grab, Drop,
    RotateClockwise, RotateAnticlockwise,
    Extend, Retract,
    PivotClockwise, PivotAnticlockwise,
    Advance, Retreat,
    PeriodOverride, Reset, Repeat
}

impl Instruction {
    pub fn from_id(id: u8) -> Option<Instruction>{
        Some(match id{
            b' ' => Instruction::Blank,
            b'G' => Instruction::Grab,
            b'g' => Instruction::Drop,
            b'R' => Instruction::RotateClockwise,
            b'r' => Instruction::RotateAnticlockwise,
            b'E' => Instruction::Extend,
            b'e' => Instruction::Retract,
            b'P' => Instruction::PivotClockwise,
            b'p' => Instruction::PivotAnticlockwise,
            b'A' => Instruction::Advance,
            b'a' => Instruction::Retreat,
            b'O' => Instruction::PeriodOverride,
            b'X' => Instruction::Reset, // incredible
            b'C' => Instruction::Repeat,
            _ => return None
        })
    }
}

// Misc

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct HexIndex{
    pub p: i32,
    pub q: i32
}