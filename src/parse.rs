use std::backtrace::Backtrace;
use std::collections::{HashMap, HashSet};
use arrayref::array_ref;
use super::data::*;

pub fn parse_puzzle(data: &[u8]) -> Result<Puzzle, &'static str>{
    let mut parser = BaseParser::new(data);
    if parser.parse_int()? != 3{
        return Err("not an opus magnum puzzle");
    }
    let name = parser.parse_string()?;
    let creator_id = parser.parse_ulong()?;
    let permissions = Permissions::from_bits_retain(parser.parse_ulong()?);
    let reagents = parser.parse_list(|s| s.parse_molecule())?;
    let products = parser.parse_list(|s| s.parse_molecule())?;
    let product_multiplier = parser.parse_int()?;

    let production_info = if parser.parse_bool()?{
        let _shrink_left = parser.parse_bool()?; // visual, don't care
        let _shrink_right = parser.parse_bool()?;
        let isolation = parser.parse_bool()?;
        let chambers = parser.parse_list(|p| Ok(Chamber{
            pos: p.parse_b_hex_index()?,
            ty: ChamberType::from_name(&p.parse_string()?).ok_or("invalid chamber type")?
        }))?;
        let conduits = parser.parse_list(|p| Ok(Conduit{
            pos_a: p.parse_b_hex_index()?,
            pos_b: p.parse_b_hex_index()?,
            hexes: p.parse_list(|p| p.parse_b_hex_index())?
        }))?;
        // vial visuals also ignored
        Some(ProductionInfo{
            isolation,
            chambers,
            conduits
        })
    } else { None };

    Ok(Puzzle{ name, creator_id, reagents, products, permissions, product_multiplier, production_info })
}

pub fn parse_solution(data: &[u8]) -> Result<Solution, &'static str>{
    let mut parser = BaseParser::new(data);
    if parser.parse_int()? != 7 {
        return Err("not an opus magnum solution");
    }
    let puzzle_name = parser.parse_string()?;
    let name = parser.parse_string()?;
    let metrics = match parser.parse_int()? {
        0 => None,
        4 => {
            if parser.parse_int()? != 0 { return Err("invalid solution (0 != 0)") }
            let cycles = parser.parse_int()?;
            if parser.parse_int()? != 1 { return Err("invalid solution (1 != 1)") }
            let cost = parser.parse_int()?;
            if parser.parse_int()? != 2 { return Err("invalid solution (2 != 2)") }
            let area = parser.parse_int()?;
            if parser.parse_int()? != 3 { return Err("invalid solution (3 != 3)") }
            let instructions = parser.parse_int()?;
            Some(Metrics{ cycles, cost, area, instructions })
        },
        _ => return Err("invalid number of metrics")
    };
    let parts: Vec<Part> = parser.parse_list(|p| {
        let part_name = p.parse_string()?;
        if p.parse_byte()? != 1 { return Err("invalid solution part (1 != 1)") }
        let pos = p.parse_i_hex_index()?;
        let arm_length = p.parse_int()?;
        let rotation = p.parse_int()?;
        let index = p.parse_int()?;
        let instructions = p.parse_list(|p| {
            let idx = p.parse_int()?;
            let instr = p.parse_byte()?;
            Ok((Instruction::from_id(instr).ok_or("invalid instruction id")?, idx))
        })?;

        let track_hexes = if part_name == "track"{
            p.parse_list(|p| { p.parse_i_hex_index() })?
        }else{ Vec::new() };

        let arm_number = p.parse_int()? + 1;

        let (conduit_index, conduit_hexes) = if part_name == "pipe"{
            (p.parse_int()?, p.parse_list(|p| { p.parse_i_hex_index() })?)
        }else{ (0, Vec::new()) };

        Ok(Part{
            ty: PartType::from_name(&part_name).ok_or("invalid part type")?,
            pos,
            rotation,
            arm_number,
            arm_length,
            index,
            conduit_index,
            track_hexes,
            conduit_hexes,
            instructions
        })
    })?;
    Ok(Solution{ name, puzzle_name, metrics, parts })
}

// byte parsing

struct BaseParser<'a>{
    data: &'a [u8]
}

impl<'a> BaseParser<'a>{

    fn new(data: &'a [u8]) -> Self{
        Self{ data }
    }

    fn parse_byte(&mut self) -> Result<u8, &'static str>{
        if self.data.len() == 0{
            Err("not enough bytes")
        }else{
            let result = self.data[0];
            self.data = &self.data[1..];
            Ok(result)
        }
    }

    fn parse_sbyte(&mut self) -> Result<i8, &'static str>{
        if self.data.len() == 0{
            Err("not enough bytes")
        }else{
            let result = i8::from_be_bytes([self.data[0]]);
            self.data = &self.data[1..];
            Ok(result)
        }
    }

    fn parse_bool(&mut self) -> Result<bool, &'static str>{
        Ok(self.parse_byte()? != 0)
    }

    fn parse_int(&mut self) -> Result<i32, &'static str>{
        if self.data.len() >= 4{
            let result = i32::from_le_bytes(array_ref![self.data, 0, 4].clone());
            self.data = &self.data[4..];
            Ok(result)
        }else{
            println!("a {}", Backtrace::capture());
            Err("not enough bytes to read int")
        }
    }

    fn parse_long(&mut self) -> Result<i64, &'static str>{
        if self.data.len() >= 8{
            let result = i64::from_le_bytes(array_ref![self.data, 0, 8].clone());
            self.data = &self.data[8..];
            Ok(result)
        }else{
            Err("not enough bytes to read long")
        }
    }

    fn parse_ulong(&mut self) -> Result<u64, &'static str>{
        if self.data.len() >= 8{
            let result = u64::from_le_bytes(array_ref![self.data, 0, 8].clone());
            self.data = &self.data[8..];
            Ok(result)
        }else{
            Err("not enough bytes to read ulong")
        }
    }

    fn parse_list<T>(&mut self, f: fn(&mut Self) -> Result<T, &'static str>) -> Result<Vec<T>, &'static str>{
        let amount = self.parse_int()?;
        let mut result = Vec::with_capacity(amount as usize);
        for _ in 0..amount{
            result.push(f(self)?)
        }
        Ok(result)
    }

    fn parse_var_int(&mut self) -> Result<usize, &'static str>{
        let mut value: usize = 0;
        let mut shift: i32 = 0;
        while self.data.len() > 0{
            let next = self.parse_byte()?;
            value |= ((next & 0x7F) as usize) << shift;
            shift += 7;
            if (next & 0x80) != 1{
                break
            }
        }

        Ok(value)
    }

    fn parse_string(&mut self) -> Result<String, &'static str>{
        let length = self.parse_var_int()?;
        let result = String::from_utf8(Vec::from(&self.data[..length])).map_err(|_| "invalid utf8")?;
        self.data = &self.data[length..];
        Ok(result)
    }

    /// Parse a hex index represented with signed byte offsets, used in puzzles.
    fn parse_b_hex_index(&mut self) -> Result<HexIndex, &'static str>{
        Ok(HexIndex{ q: self.parse_sbyte()? as i32, r: self.parse_sbyte()? as i32 })
    }

    /// Parse a hex index represented with signed 32-bit integer offsets, used in solutions.
    fn parse_i_hex_index(&mut self) -> Result<HexIndex, &'static str>{
        Ok(HexIndex{ q: self.parse_int()?, r: self.parse_int()? })
    }

    fn parse_atom(&mut self) -> Result<Atom, &'static str>{
        Ok(Atom::from_id(self.parse_byte()?).ok_or("invalid atom type")?)
    }

    fn parse_bond_type(&mut self) -> Result<BondType, &'static str>{
        let ty = self.parse_byte()?;
        if ty == 1 {
            Ok(BondType::Normal)
        }else if (ty & 0b1111_000_1) != 0{
            Err("invalid bond type")
        }else{
            Ok(BondType::Triplex{ red: (ty & 0b10) != 0, black: (ty & 0b100) != 0, yellow: (ty & 0b1000) != 0 })
        }
    }

    fn parse_bond(&mut self) -> Result<Bond, &'static str>{
        Ok(Bond{ ty: self.parse_bond_type()?, start: self.parse_b_hex_index()?, end: self.parse_b_hex_index()? })
    }

    fn parse_molecule(&mut self) -> Result<Molecule, &'static str>{
        Ok(Molecule{
            atoms: HashMap::from_iter(self.parse_list(
                |s| {
                    let atom = s.parse_atom()?;
                    let index = s.parse_b_hex_index()?;
                    Ok((index, atom))
                }
            )?),
            bonds: HashSet::from_iter(self.parse_list(|s| s.parse_bond())?.iter().cloned())
        })
    }
}