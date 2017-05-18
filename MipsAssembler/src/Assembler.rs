#![allow(non_snake_case)]

use std;
use std::collections::HashMap;

type OpCodeTable = HashMap<&'static str, u32>;

lazy_static! {
    static ref ALU_R_FUNCS : OpCodeTable = {
        let mut map = HashMap::new();
        map.insert("add", 32);
        map.insert("addu", 33);
        map.insert("sub", 34);
        map.insert("subu", 35);
        map.insert("and", 36);
        map.insert("or", 37);
        map.insert("xor", 38);
        map.insert("nor", 39);
        map.insert("slt", 42);
        map.insert("sltu", 43);
        map
    };
}

lazy_static! {
    static ref ALU_I_OPCODES : OpCodeTable = {
        let mut map = HashMap::new();
        map.insert("addi", 8);
        map.insert("addiu", 9);
        map.insert("slti", 10);
        map.insert("sltiu", 11);
        map.insert("andi", 12);
        map.insert("ori", 13);
        map.insert("xori", 14);
        map.insert("lui", 15);
        map
    };
}

lazy_static! {
    static ref JUMP_J_OPCODES : OpCodeTable = {
        let mut map =  HashMap::new();
        map.insert("j", 2);
        map.insert("jal", 3);
        map
    };
}

lazy_static! {
    static ref JUMP_I_OPCODES : OpCodeTable = {
        let mut map = HashMap::new();
        map.insert("beq", 4);
        map.insert("bne", 5);
        map.insert("blez", 6);
        map.insert("bgtz", 7);
        map
    };
}

lazy_static! {
    static ref REGISTERS : OpCodeTable = {
        let mut map: OpCodeTable =  HashMap::new();
        const REGISTER_NAMES : [&'static str; 32] = ["ZERO", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1",
            "t2", "t3", "t4", "t5", "t6", "t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "t8",
            "t9", "k0", "k1", "gp", "sp", "fp", "ra"];
        for (i, name) in REGISTER_NAMES.iter().enumerate() {
            map.insert(name, i as u32);
        }
        map
    };
}

lazy_static! {
    static ref LOAD_STORE_I_OPCODES : OpCodeTable = {
        let mut map: OpCodeTable = HashMap::new();
        map.insert("sw", 43);
        map.insert("lw", 35);
        map
    };
}

lazy_static! {
    static ref JUMP_R_FUNCTS : OpCodeTable = {
        let mut map: OpCodeTable = HashMap::new();
        map.insert("jr", 8);
        map.insert("jalr", 9);
        map
    };
}

#[derive(Debug)]
pub struct AsmErrorInfo {
    errorMessage: String,
    line: u32,
    col: u32,
}

impl AsmErrorInfo {
    fn new(errorMessage: String, line: u32, col: u32) -> AsmErrorInfo {
        AsmErrorInfo {
            errorMessage,
            line,
            col,
        }
    }
}

enum JCType {
    J,
    I,
}

pub struct Parser {
    row: u32,
    col: u32,
    currentLine: Vec<char>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            row: 0,
            col: 0,
            currentLine: Vec::new(),
        }
    }

    pub fn Reset(&mut self) {
        self.row = 0;
        self.col = 0;
        self.currentLine.clear();
    }

    fn SkipSpaces(&mut self) {
        let currentLine = &self.currentLine;
        let mut col = self.col as usize;
        loop {
            if col >= currentLine.len() || !currentLine[col].is_whitespace() {
                break;
            }
            col = col + 1;
        }
        self.col = col as u32;
    }

    fn IsLineEnd(&self) -> bool {
        let currentLine = &self.currentLine;
        assert!(self.col as usize <= currentLine.len());
        self.col as usize == currentLine.len()
    }

    fn PeekChar(&mut self) -> Option<char> {
        if !self.IsLineEnd() {
            return Some(self.currentLine[self.col as usize]);
        }
        None
    }

    fn Bump(&mut self, count: u32) {
        assert!(self.col + count <= self.currentLine.len() as u32);
        self.col = self.col + count;
    }

    //TODO: improve this.
    fn IsIdentChar(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '$' || c.is_digit(10) || c == '+' || c == '-' || c == '_' || c == '.'
    }

    fn EatWord(&mut self) -> Option<String> {
        self.SkipSpaces();
        let start = self.col as usize;
        let mut end = start as usize;
        loop {
            if self.PeekChar()
                   .into_iter()
                   .filter(|c| Parser::IsIdentChar(*c))
                   .next()
                   .is_some() {
                self.Bump(1);
                end = end + 1;
            } else {
                break;
            }
        }
        if end == start {
            None
        } else {
            let word = self.currentLine[start..end].iter().collect::<String>();
            Some(word)
        }
    }

    fn ParseRegister(&mut self) -> Result<u32, AsmErrorInfo> {
        self.SkipSpaces();
        self.ExpectChar('$')?;
        match self.PeekChar() {
            Some(c) if c.is_digit(10) => {
                let word = self.EatWord().unwrap();
                word.parse()
                    .or_else(|_| Err(self.MakeErrorInfoWithMessage(format!("Invalid register {}", word))))
            }
            Some(_) => {
                let word = self.EatWord().unwrap();
                REGISTERS
                    .get(&*word)
                    .cloned()
                    .ok_or(self.MakeErrorInfoWithMessage(format!("Couldn't found register {}", word)))
            }
            None => Err(self.MakeErrorInfoWithMessage("Register name expected, got nothing.".into())),
        }
    }

    fn ParseImm(&mut self, lower: i32, upper: i32) -> Result<i32, AsmErrorInfo> {
        self.SkipSpaces();
        if let Some(word) = self.EatWord() {
            word.parse()
                .or_else(|_| {
                             Err(self.MakeErrorInfoWithMessage(format!("Invalid immediate number
                {}.",
                                                                       word)))
                         })
                .and_then(|x| if x > upper || x < lower {
                              Err(self.MakeErrorInfoWithMessage(format!("Too large imm {}.", x)))
                          } else {
                              Ok(x)
                          })
        } else {
            Err(self.MakeErrorInfoWithMessage(String::from("Imm expected.")))
        }
    }

    fn ParseImm16(&mut self) -> Result<i16, AsmErrorInfo> {
        self.ParseImm(std::i16::MIN as i32, std::i16::MAX as i32)
            .map(|x| x as i16)
    }

    fn ExpectChar(&mut self, c: char) -> Result<char, AsmErrorInfo> {
        self.SkipSpaces();
        if let Some(fc) = self.PeekChar() {
            if fc == c {
                self.Bump(1);
                Ok(c)
            } else {
                Err(self.MakeErrorInfoWithMessage(format!("{} is expected, get {}.", c, fc)))
            }
        } else {
            Err(self.MakeErrorInfoWithMessage(format!("{} is expected, get nothing.", c)))
        }
    }

    fn EnsureEnd(&mut self) -> Result<(), AsmErrorInfo> {
        self.SkipSpaces();
        if self.IsLineEnd() {
            Ok(())
        } else {
            Err(self.MakeErrorInfoWithMessage(String::from("Expected nothing at this line.")))
        }
    }

    pub fn AsmLines<'a, I>(&mut self, lines: I) -> Result<Vec<u32>, AsmErrorInfo>
        where I: Iterator<Item = &'a str>
    {
        let mut instrs: Vec<u32> = Vec::new();
        let mut labels: HashMap<String, i32> = HashMap::new();
        let mut codeSize: i32 = 0;
        let mut relocations = Vec::new();
        for (i, line) in lines
                .map(|s| s.chars().take_while(|c| *c != '#').collect::<Vec<_>>())
                .enumerate() {
            if line.len() == 0 {
                continue;
            }
            self.row = i as u32;
            self.col = 0;
            self.currentLine = line;
            if let Some(word) = self.EatWord() {
                match &word[..] {
                    ".data" | ".code" | ".text" | ".globl" => (),
                    s => {
                        let op = &*s;

                        if let Some(rFuncs) = ALU_R_FUNCS.get(op) {
                            let rd = self.ParseRegister()?;
                            self.ExpectChar(',')?;
                            let rs = self.ParseRegister()?;
                            self.ExpectChar(',')?;
                            let rt = self.ParseRegister()?;
                            instrs.push(Parser::EmitFormatR(0, rs, rt, rd, 0, *rFuncs));
                            codeSize += 4;
                            continue;
                        }
                        if let Some(iOpCode) = ALU_I_OPCODES.get(op) {
                            let rd = self.ParseRegister()?;
                            self.ExpectChar(',')?;
                            let rs = self.ParseRegister()?;
                            self.ExpectChar(',')?;
                            let imm = self.ParseImm16()?;
                            instrs.push(Parser::EmitFormatI(*iOpCode, rs, rd, imm as u16));
                            codeSize += 4;
                            continue;
                        }
                        if let Some(iOpCode) = JUMP_I_OPCODES.get(op) {
                            let rs = self.ParseRegister()?;
                            self.ExpectChar(',')?;
                            let rt = self.ParseRegister()?;
                            self.ExpectChar(',')?;
                            if let Some(label) = self.EatWord() {
                                relocations.push((instrs.len() as u32, label, JCType::I));
                                instrs.push(Parser::EmitFormatI(*iOpCode, rs, rt, 0));
                                codeSize += 4;
                                continue;
                            } else {
                                return Err(self.MakeErrorInfoWithMessage("label expected.".into()));
                            }
                        }
                        if let Some(iOpCode) = LOAD_STORE_I_OPCODES.get(op) {
                            let rt = self.ParseRegister()?;
                            self.ExpectChar(',')?;
                            self.SkipSpaces();
                            let offset = match self.PeekChar() {
                                Some(c) if c.is_digit(10) => self.ParseImm16()?,
                                Some('(') => 0,
                                Some(_) | None => return Err(self.MakeErrorInfoWithMessage("Invalid character.".into())),
                            };
                            self.ExpectChar('(')?;
                            let rs = self.ParseRegister()?;
                            self.ExpectChar(')')?;
                            instrs.push(Parser::EmitFormatI(*iOpCode, rs, rt, offset as u16));
                            codeSize += 4;
                            continue;
                        }
                        if let Some(jOpCode) = JUMP_J_OPCODES.get(op) {
                            if let Some(dest) = self.EatWord() {
                                self.EnsureEnd()?;
                                relocations.push((instrs.len() as u32, dest, JCType::J));
                                instrs.push(Parser::EmitFormatJ(*jOpCode, 0));
                                codeSize += 4;
                            }
                            continue;
                        }
                        if let Some(funct) = JUMP_R_FUNCTS.get(op) {
                            let rs = self.ParseRegister()?;
                            instrs.push(Parser::EmitFormatR(0, rs, 0, 0, 0, *funct));
                            codeSize += 4;
                            continue;
                        }
                        //labels
                        let labelName = op;
                        if self.ExpectChar(':').is_err() {
                            return Err(self.MakeErrorInfoWithMessage(format!("Invalid instruction {}. Or you want a label? : is required.",
                                                                             op)));
                        }
                        labels.insert(labelName.into(), codeSize / 4);
                    }
                }
            }
        }

        for relocation in relocations {
            let instr = instrs[relocation.0 as usize];
            if let Some(pos) = labels.get(&relocation.1) {
                instrs[relocation.0 as usize] = match relocation.2 {
                    JCType::I => {
                        let offset = *pos as i32 - relocation.0 as i32;
                        Parser::ReemitFormatI(instr, offset as i16)
                    }
                    JCType::J => Parser::ReemitFormatJ(instr, *pos as u32),
                };
            } else {
                return Err(self.MakeErrorInfoWithMessage(format!("Label name {} isn't found.", relocation.1)));
            }

        }
        Ok(instrs)
    }

    pub fn AsmStr(&mut self, content: &str) -> Result<Vec<u32>, AsmErrorInfo> {
        self.AsmLines(content.lines())
    }

    fn MakeErrorInfoWithMessage(&self, message: String) -> AsmErrorInfo {
        AsmErrorInfo::new(message, self.row, self.col)
    }

    fn ReemitFormatJ(instr: u32, addr: u32) -> u32 {
        (instr & (!0 << 6)) | addr
    }

    fn ReemitFormatI(instr: u32, offset: i16) -> u32 {
        instr | ((offset as u32) & 0xFFFF)
    }

    fn EmitFormatR(opCode: u32, source: u32, target: u32, dest: u32, shamt: u32, funct: u32) -> u32 {
        opCode << 26 | source << 21 | target << 16 | dest << 11 | shamt << 6 | funct
    }

    fn EmitFormatI(opCode: u32, source: u32, target: u32, imm: u16) -> u32 {
        opCode << 26 | source << 21 | target << 16 | (imm & 0xFFFF) as u32
    }

    fn EmitFormatJ(opCode: u32, addr: u32) -> u32 {
        opCode << 26 | (addr & 0x3FFFFFF)
    }
}
