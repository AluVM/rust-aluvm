// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2021-2023 by
//     Dr Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023 UBIDECO Institute. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloc::boxed::Box;

use super::{
    DeleteFlag, FloatEqFlag, InsertFlag, InstructionSet, IntFlags, MergeFlag, RoundingFlag,
    SignFlag, SplitFlag,
};
use crate::data::{ByteStr, MaybeNumber, Step};
use crate::isa::{ExtendFlag, NoneEqFlag};
use crate::library::LibSite;
use crate::reg::{Reg16, Reg32, Reg8, RegA, RegA2, RegAF, RegAR, RegBlockAR, RegF, RegR, RegS};

/// Reserved instruction, which equal to [`ControlFlowOp::Fail`].
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, Default)]
#[display("rsrv:{0:02X}")]
pub struct ReservedOp(/** Reserved instruction op code value */ pub(super) u8);

/// Full set of instructions
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(inner)]
#[non_exhaustive]
pub enum Instr<Extension = ReservedOp>
where
    Extension: InstructionSet,
{
    /// Control-flow instructions. See [`ControlFlowOp`] for the details.
    // 0b00_000_***
    ControlFlow(ControlFlowOp),

    /// Instructions setting register values. See [`PutOp`] for the details.
    // 0b00_001_***
    Put(PutOp),

    /// Instructions moving and swapping register values. See [`PutOp`] for the details.
    // 0b00_010_***
    Move(MoveOp),

    /// Instructions comparing register values. See [`CmpOp`] for the details.
    // 0b00_011_***
    Cmp(CmpOp),

    /// Arithmetic instructions. See [`ArithmeticOp`] for the details.
    // 0b00_100_***
    Arithmetic(ArithmeticOp),

    /// Bit operations & boolean algebra instructions. See [`BitwiseOp`] for the details.
    // 0b00_101_***
    Bitwise(BitwiseOp),

    /// Operations on byte strings. See [`BytesOp`] for the details.
    // 0b00_110_***
    Bytes(BytesOp),

    /// Cryptographic hashing functions. See [`DigestOp`] for the details.
    // 0b01_000_***
    Digest(DigestOp),

    #[cfg(feature = "secp256k1")]
    /// Operations on Secp256k1 elliptic curve. See [`Secp256k1Op`] for the details.
    // 0b01_001_0**
    Secp256k1(Secp256k1Op),

    #[cfg(feature = "curve25519")]
    /// Operations on Curve25519 elliptic curve. See [`Curve25519Op`] for the details.
    // 0b01_001_1**
    Curve25519(Curve25519Op),

    /// Extension operations which can be provided by a host environment provided via generic
    /// parameter
    // 0b10_***_***
    ExtensionCodes(Extension),

    /// Reserved instruction for fututre use in core `ALU` ISA.
    ///
    /// Currently equal to [`ControlFlowOp::Fail`].
    ReservedInstruction(ReservedOp),

    // Reserved operations for the future use.
    //
    // When such an opcode is met in the bytecode the decoder MUST fail.
    // 0x11_***_***
    /// No-operation instruction.
    // #[value = 0b11_111_111]
    Nop,
}

/// Control-flow instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum ControlFlowOp {
    /// Completes program execution writing `false` to `st0` (indicating program failure). Does not
    /// modify value of call stack registers.
    #[display("fail")]
    Fail,

    /// Completes program execution writing `true` to `st0` (indicating program success). Does not
    /// modify value of call stack registers.
    #[display("succ")]
    Succ,

    /// Unconditionally jumps to an offset. Increments `cy0`.
    #[display("jmp     {0:#06X}")]
    Jmp(u16),

    /// Jumps to an offset if `st0` == true, otherwise does nothing. Increments `cy0`.
    #[display("jif     {0:#06X}")]
    Jif(u16),

    /// Jumps to other location in the current code with ability to return back (calls a
    /// subroutine). Increments `cy0` and pushes offset of the instruction which follows current
    /// one to `cs0`.
    #[display("routine {0:#06X}")]
    Routine(u16),

    /// Calls code from an external library identified by the hash of its code. Increments `cy0`
    /// and `cp0` and pushes offset of the instruction which follows current one to `cs0`.
    #[display("call    {0}")]
    Call(LibSite),

    /// Passes execution to other library without an option to return. Does not increment `cy0` and
    /// `cp0` counters and does not add anything to the call stack `cs0`.
    #[display("exec    {0}")]
    Exec(LibSite),

    /// Returns execution flow to the previous location from the top of `cs0`. Does not change the
    /// value in `cy0`. Decrements `cp0`.
    #[display("ret")]
    Ret,
}

/// Instructions setting register values
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
pub enum PutOp {
    /// Cleans a value of `A` register (sets it to undefined state)
    #[display("clr     {0}{1}")]
    ClrA(RegA, Reg32),

    /// Cleans a value of `F` register (sets it to undefined state)
    #[display("clr     {0}{1}")]
    ClrF(RegF, Reg32),

    /// Cleans a value of `R` register (sets it to undefined state)
    #[display("clr     {0}{1}")]
    ClrR(RegR, Reg32),

    /// Unconditionally assigns a value to `A` register.
    ///
    /// NB: Bytecode does not contain the value (it is contained in the data segment), thus when
    ///     this instruction is assembled and the data are not present in the data segment (their
    ///     offset + length exceeds data segment size) the operation will set destination register
    ///     into undefined state and `st0` to `false`. Otherwise, `st0` value is not affected.
    #[display("put     {0}{1},{2}")]
    PutA(RegA, Reg32, Box<MaybeNumber>),

    /// Unconditionally assigns a value to `F` register
    ///
    /// NB: Bytecode does not contain the value (it is contained in the data segment), thus when
    ///     this instruction is assembled and the data are not present in the data segment (their
    ///     offset + length exceeds data segment size) the operation will set destination register
    ///     into undefined state and `st0` to `false`. Otherwise, `st0` value is not affected.
    #[display("put     {0}{1},{2}")]
    PutF(RegF, Reg32, Box<MaybeNumber>),

    /// Unconditionally assigns a value to `R` register
    ///
    /// NB: Bytecode does not contain the value (it is contained in the data segment), thus when
    ///     this instruction is assembled and the data are not present in the data segment (their
    ///     offset + length exceeds data segment size) the operation will set destination register
    ///     into undefined state and `st0` to `false`. Otherwise, `st0` value is not affected.
    #[display("put     {0}{1},{2}")]
    PutR(RegR, Reg32, Box<MaybeNumber>),

    /// Conditionally assigns a value to `A` register if the register is in uninitialized state.
    /// If the register is initialized and the value is not `None` sets `st0` to `false`.
    ///
    /// NB: Bytecode does not contain the value (it is contained in the data segment), thus when
    ///     this instruction is assembled and the data are not present in the data segment (their
    ///     offset + length exceeds data segment size) _and_ the destination register is
    ///     initialized, the operation will set destination register into undefined state and `st0`
    ///     to `false`. Otherwise, `st0` value is changed according to the general operation rules.
    #[display("putif   {0}{1},{2}")]
    PutIfA(RegA, Reg32, Box<MaybeNumber>),

    /// Conditionally assigns a value to `R` register if the register is in uninitialized state.
    /// If the register is initialized and the value is not `None` sets `st0` to `false`.
    ///
    /// NB: Bytecode does not contain the value (it is contained in the data segment), thus when
    ///     this instruction is assembled and the data are not present in the data segment (their
    ///     offset + length exceeds data segment size) _and_ the destination register is
    ///     initialized, the operation will set destination register into undefined state and `st0`
    ///     to `false`. Otherwise, `st0` value is changed according to the general operation rules.
    #[display("putif   {0}{1},{2}")]
    PutIfR(RegR, Reg32, Box<MaybeNumber>),
}

/// Instructions moving and swapping register values
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum MoveOp {
    /// Move operation: moves value of one of the integer arithmetic registers into another integer
    /// arithmetic register of the same bit size, clearing its previous value and setting the
    /// source to `None`.
    #[display("mov     {0}{1},{0}{2}")]
    MovA(RegA, Reg32, Reg32),

    /// Duplicate operation: duplicates value of one of the integer arithmetic registers into
    /// another integer arithmetic register of the same bit size, clearing its previous value.
    #[display("dup     {0}{1},{0}{2}")]
    DupA(RegA, Reg32, Reg32),

    /// Swap operation: swaps value of two integer arithmetic registers of the same bit size.
    #[display("swp     {0}{1},{0}{2}")]
    SwpA(RegA, Reg32, Reg32),

    /// Move operation: moves value of one of the float arithmetic registers into another float
    /// arithmetic register of the same bit size, clearing its previous value and setting the
    /// source to `None`.
    #[display("mov     {0}{1},{0}{2}")]
    MovF(RegF, Reg32, Reg32),

    /// Duplicate operation: duplicates value of one of the float arithmetic registers into
    /// another float arithmetic register of the same bit size, clearing its previous value.
    #[display("dup     {0}{1},{0}{2}")]
    DupF(RegF, Reg32, Reg32),

    /// Swap operation: swaps value of two float arithmetic registers of the same bit size.
    #[display("swp     {0}{1},{0}{2}")]
    SwpF(RegF, Reg32, Reg32),

    /// Move operation: moves value of one of the general non-arithmetic registers into another
    /// general non- arithmetic register of the same bit size, clearing its previous value and
    /// setting the source to `None`.
    #[display("mov     {0}{1},{0}{2}")]
    MovR(RegR, Reg32, Reg32),

    /// Duplicate operation: duplicates value of one of the general non-arithmetic registers into
    /// another general non-arithmetic register of the same bit size, clearing its previous value.
    #[display("dup     {0}{1},{0}{2}")]
    DupR(RegR, Reg32, Reg32),

    // ----
    /// Copy operation: copies value from one of the integer arithmetic registers to a destination
    /// register treating value as unsigned: if the value does not fit destination bit dimension,
    /// truncates the most significant bits until they fit, setting `st0` value to `false`.
    /// Otherwise, the operation sets `st0` to `true`.
    #[display("cpy     {0}{1},{2}{3}")]
    CpyA(RegA, Reg32, RegA, Reg32),

    /// Conversion operation: copies value from one of the integer arithmetic registers to a
    /// destination register treating value as signed: if the value does not fit destination bit
    /// dimension, truncates the most significant non-sign bits until they fit, setting `st0`
    /// value to `false`. Otherwise, fills the difference between source and destination bit length
    /// with the value taken from the most significant source bit (sign bit) and sets `st0` to
    /// `true`.
    #[display("cnv     {0}{1},{2}{3}")]
    CnvA(RegA, Reg32, RegA, Reg32),

    /// Conversion operation: converts value from one of the float arithmetic registers to a
    /// destination register according to floating encoding rules. If the value does not fit
    /// destination bit dimension, truncates the most significant non-sign bits until they fit,
    /// setting `st0` value to `false`. Otherwise sets `st0` to `true`.
    #[display("cnv     {0}{1},{2}{3}")]
    CnvF(RegF, Reg32, RegF, Reg32),

    /// Copy operation: copies value from one of the general non-arithmetic registers to a
    /// destination register. If the value does not fit destination bit dimension,
    /// truncates the most significant bits until they fit, setting `st0` value to `false`.
    /// Otherwise, extends most significant bits with zeros and sets `st0` to `true`.
    #[display("cpy     {0}{1},{2}{3}")]
    CpyR(RegR, Reg32, RegR, Reg32),

    /// Swap-copy operation: swaps value one of the integer arithmetic registers with a value of an
    /// general non-arithmetic register. If any of the values do not fit destination bit
    /// dimensions, truncates the most significant bits until they fit, setting `st0` value to
    /// `false`. Otherwise, extends most significant bits with zeros and sets `st0` to `true`.
    #[display("spy     {0}{1},{2}{3}")]
    SpyAR(RegA, Reg32, RegR, Reg32),

    /// Conversion operation: converts value of an integer arithmetic register to a float register
    /// according to floating encoding rules. If the value does not fit destination bit dimension,
    /// truncates the most significant non-sign bits until they fit, setting `st0` value to
    /// `false`. Otherwise sets `st0` to `true`.
    ///
    /// NB: operation always treats integers as signed integers.
    #[display("cnv     {0}{1},{2}{3}")]
    CnvAF(RegA, Reg32, RegF, Reg32),

    /// Conversion operation: converts value of a float arithmetic register to an integer register
    /// according to floating encoding rules. If the value does not fit destination bit dimension,
    /// truncates the most significant non-sign bits until they fit, setting `st0` value to
    /// `false`. Otherwise sets `st0` to `true`.
    ///
    /// NB: operation always treats integers as signed integers.
    #[display("cnv     {0}{1},{2}{3}")]
    CnvFA(RegF, Reg32, RegA, Reg32),
}

/// Instructions comparing register values
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum CmpOp {
    /// Compares value of two integer arithmetic registers setting `st0` to `true` if the first
    /// parameter is greater (and not equal) than the second one. If at least one of the registers
    /// is set to `None`, sets `st0` to `false`.
    #[display("gt.{0}    {1}{2},{1}{3}")]
    GtA(SignFlag, RegA, Reg32, Reg32),

    /// Compares value of two integer arithmetic registers setting `st0` to `true` if the first
    /// parameter is lesser (and not equal) than the second one. If at least one of the registers
    /// is set to `None`, sets `st0` to `false`.
    #[display("lt.{0}    {1}{2},{1}{3}")]
    LtA(SignFlag, RegA, Reg32, Reg32),

    /// Compares value of two float arithmetic registers setting `st0` to `true` if the first
    /// parameter is greater (and not equal) than the second one. If at least one of the registers
    /// is set to `None`, sets `st0` to `false`.
    #[display("gt.{0}    {1}{2},{1}{3}")]
    GtF(FloatEqFlag, RegF, Reg32, Reg32),

    /// Compares value of two float arithmetic registers setting `st0` to `true` if the first
    /// parameter is lesser (and not equal) than the second one. If at least one of the registers
    /// is set to `None`, sets `st0` to `false`.
    #[display("lt.{0}    {1}{2},{1}{3}")]
    LtF(FloatEqFlag, RegF, Reg32, Reg32),

    // ----
    /// Compares value of two general non-arithmetic registers setting `st0` to `true` if the first
    /// parameter is greater (and not equal) than the second one. If at least one of the registers
    /// is set to `None`, sets `st0` to `false`.
    #[display("gt      {0}{1},{0}{2}")]
    GtR(RegR, Reg32, Reg32),

    /// Compares value of two general non-arithmetic registers setting `st0` to `true` if the first
    /// parameter is lesser (and not equal) than the second one. If at least one of the registers
    /// is set to `None`, sets `st0` to `false`.
    #[display("lt      {0}{1},{0}{2}")]
    LtR(RegR, Reg32, Reg32),

    /// Checks equality of value in two integer arithmetic (`A`) registers putting result into
    /// `st0`. None-equality flag specifies value for `st0` for the cases when both of the
    /// registers are in `None` state.
    #[display("eq.{0}    {1}{2},{1}{3}")]
    EqA(
        /** `st0` value if both of the registers are uninitialized */ NoneEqFlag,
        RegA,
        Reg32,
        Reg32,
    ),

    /// Checks equality of value in two float arithmetic (`F`) registers putting result into `st0`.
    /// If both registers are `None`, the `st0` is set to `false`.
    #[display("eq.{0}    {1}{2},{1}{3}")]
    EqF(FloatEqFlag, RegF, Reg32, Reg32),

    /// Checks equality of value in two non-arithmetic (`R`) registers putting result into `st0`.
    /// None-equality flag specifies value for `st0` for the cases when both of the registers
    /// are in `None` state.
    #[display("eq.{0}    {1}{2},{1}{3}")]
    EqR(
        /** `st0` value if both of the registers are uninitialized */ NoneEqFlag,
        RegR,
        Reg32,
        Reg32,
    ),

    // ---
    /// Checks if the value in `A` register is equal to zero, setting `st0` to `true` in this case.
    /// Otherwise, sets `st0` to false (including when the register is in the undefined state).
    #[display("ifz     {0}{1}")]
    IfZA(RegA, Reg32),

    /// Checks if the value in `R` register is equal to zero, setting `st0` to `true` in this case.
    /// Otherwise, sets `st0` to false (including when the register is in the undefined state).
    #[display("ifz     {0}{1}")]
    IfZR(RegR, Reg32),

    /// Checks if the value in `A` register is in an undefined state, setting `st0` to `true` in
    /// this case. Otherwise, sets `st0` to false.
    #[display("ifn     {0}{1}")]
    IfNA(RegA, Reg32),

    /// Checks if the value in `R` register is in an undefined state, setting `st0` to `true` in
    /// this case. Otherwise, sets `st0` to false.
    #[display("ifn     {0}{1}")]
    IfNR(RegR, Reg32),

    /// Takes value from `st0` and merges into the value of the destination `A` register. The merge
    /// operation is defined by the [`MergeFlag`] argument.
    #[display("st.{0}    {1}{2}")]
    St(MergeFlag, RegA, Reg8),

    /// Inverses value in `st0` register
    #[display("stinv")]
    StInv,
}

/// Arithmetic instructions.
///
/// All operations modify the value of `st0` register, setting it to `false` if the destination
/// is set to `None`. Otherwise, `st0` value is `true`, even if the overflow has occurred (when
/// `wrap` flag is provided).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Display)]
pub enum ArithmeticOp {
    /// Adds values from two integer arithmetic registers and puts result into the first register.
    #[display("add.{0}  {1}{2},{1}{3}")]
    AddA(IntFlags, RegA, Reg32, Reg32),

    /// Adds values from two float arithmetic registers and puts result into the first register.
    #[display("add.{0}   {1}{2},{1}{3}")]
    AddF(RoundingFlag, RegF, Reg32, Reg32),

    /// Subtracts values from two integer arithmetic registers and puts result into the first
    /// register.
    #[display("sub.{0}  {1}{2},{1}{3}")]
    SubA(IntFlags, RegA, Reg32, Reg32),

    /// Subtracts values from two float arithmetic registers and puts result into the first
    /// register.
    #[display("sub.{0}   {1}{2},{1}{3}")]
    SubF(RoundingFlag, RegF, Reg32, Reg32),

    /// Multiplies values from two integer arithmetic registers and puts result into the first
    /// register.
    #[display("mul.{0}  {1}{2},{1}{3}")]
    MulA(IntFlags, RegA, Reg32, Reg32),

    /// Multiplies values from two float arithmetic registers and puts result into the first
    /// register.
    #[display("mul.{0}   {1}{2},{1}{3}")]
    MulF(RoundingFlag, RegF, Reg32, Reg32),

    /// Divides values from two integer arithmetic registers and puts result into the first
    /// register.
    ///
    /// Since the division operation may not result in overflow, the overflow flag is used to
    /// indicate rounding of the result:
    ///
    /// Overflow flag is also defines behaviour for zero division `(x/0 if x > 0)`: whether the
    /// destination must be set to `0` (true) or to None (false).
    ///
    /// NB: Impossible arithmetic operation 0/0 always sets destination to `None`.
    #[display("div.{0}  {1}{2},{1}{3}")]
    DivA(IntFlags, RegA, Reg32, Reg32),

    /// Divides values from two float arithmetic registers and puts result into the first register.
    #[display("div.{0}   {1}{2},{1}{3}")]
    DivF(RoundingFlag, RegF, Reg32, Reg32),

    /// Modulo division.
    ///
    /// Puts a reminder of the division of source register on destination register into the
    /// the first register.
    #[display("rem     {0}{1},{2}{3}")]
    Rem(RegA, Reg32, RegA, Reg32),

    /// Increment/decrement register value on a given signed step.
    ///
    /// Sets the destination to `None` and `st0` to `false` in case of overflow.
    #[display("{2:#}     {0}{1},{2}")]
    Stp(RegA, Reg32, Step),

    /// Negates most significant bit
    #[display("neg     {0}{1}")]
    Neg(RegAF, Reg16),

    /// Replaces the register value with its absolute value
    #[display("abs     {0}{1}")]
    Abs(RegAF, Reg16),
}

/// Bit operations & boolean algebra instructions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum BitwiseOp {
    /// Bitwise AND operation
    #[display("and     {0}{1},{0}{2},{0}{3}")]
    And(RegAR, /** Source 1 */ Reg16, /** Source 2 */ Reg16, /** Operation destination */ Reg16),

    /// Bitwise OR operation
    #[display("or      {0}{1},{0}{2},{0}{3}")]
    Or(RegAR, /** Source 1 */ Reg16, /** Source 2 */ Reg16, /** Operation destination */ Reg16),

    /// Bitwise XOR operation
    #[display("xor     {0}{1},{0}{2},{0}{3}")]
    Xor(RegAR, /** Source 1 */ Reg16, /** Source 2 */ Reg16, /** Operation destination */ Reg16),

    /// Bitwise inversion
    #[display("not     {0}{1}")]
    Not(RegAR, Reg16),

    /// Left bit shift, filling added bits values with zeros. Sets `st0` value to the value of the
    /// most significant bit before the operation.
    ///
    /// This, [`BitwiseOp::ShrA`] and [`BitwiseOp::ShrR`] operations are encoded with the same
    /// instruction bitcode and differ only in their first two argument bits.
    #[display("shl     {0}{1},{2}{3}")]
    Shl(
        /** Which of `A` registers will have a shift value */ RegA2,
        /** Index of `u8` or `u16` register with bitshift value */ Reg32,
        /** Register to shift the value in */ RegAR,
        /** Source & destination register */ Reg32,
    ),

    /// Right bit shift for one of the integer arithmetic registers, filling added bits values with
    /// zeros (if `sign` flag is set to `false`) or ones (if `sign` flag is set to `true`).
    /// Sets `st0` value to the value of the least significant bit before the operation.
    ///
    /// This, [`BitwiseOp::Shl`] and [`BitwiseOp::ShrR`] operations are encoded with the same
    /// instruction bitcode and differ only in their first two argument bits.
    #[display("shr.{0}   {1}{2},{3}{4}")]
    ShrA(
        /** Sign flag */ SignFlag,
        /** Which of `A` registers will have a shift value */ RegA2,
        /** Index of `u8` or `u16` register with bitshift value */ Reg16,
        /** Family of `A` registers to shift */ RegA,
        /** Source & destination `A` register */ Reg32,
    ),

    /// Right bit shift for one of the general non-arithmetic registers, filling added bits values
    /// with zeros (if `sign` flag is set to `false`) or ones (if `sign` flag is set to `true`).
    /// Sets `st0` value to the value of the least significant bit before the operation.
    ///
    /// This, [`BitwiseOp::Shl`] and [`BitwiseOp::ShrA`] operations are encoded with the same
    /// instruction bitcode and differ only in their first two argument bits.
    #[display("shr     {0}{1},{2}{3}")]
    ShrR(
        /** Which of `A` registers will have a shift value */ RegA2,
        /** Index of `u8` or `u16` register with bitshift value */ Reg32,
        /** Family of `R` registers to shift */ RegR,
        /** Source & destination `R` register */ Reg32,
    ),

    /// Left bit shift, cycling the shifted values (most significant bit becomes least
    /// significant), putting the result into the first source register. Does not modify `st0`
    /// value.
    ///
    /// This and the next [`BitwiseOp::Scr`] operation are encoded with the same instruction
    /// bitcode and differ only in their first argument bit.
    #[display("scl     {0}{1},{2}{3}")]
    Scl(
        /** Which of `A` registers will have a shift value */ RegA2,
        /** Index of `u8` or `u16` register with bitshift value */ Reg32,
        /** Register to shift the value in */ RegAR,
        /** Source & destination register */ Reg32,
    ),

    /// Right bit shift, cycling the shifted values (least significant bit becomes nost
    /// significant), putting the result into the first source register. Does not modify `st0`
    /// value.
    ///
    /// This and the previous [`BitwiseOp::Scl`] operation are encoded with the same instruction
    /// bitcode and differ only in their first argument bit.
    #[display("scr     {0}{1},{2}{3}")]
    Scr(
        /** Which of `A` registers will have a shift value */ RegA2,
        /** Index of `u8` or `u16` register with bitshift value */ Reg32,
        /** Register to shift the value in */ RegAR,
        /** Source & destination register */ Reg32,
    ),

    /// Reverses bit order in the integer arithmetic register. Does not modify `st0` value.
    #[display("rev     {0}{1}")]
    RevA(RegA, Reg32),

    /// Reverses bit order in the generic non-arithmetic register. Does not modify `st0` value.
    #[display("rev     {0}{1}")]
    RevR(RegR, Reg32),
}

/// Operations on byte strings.
///
/// All of these operations either set `st0` to `false`, if an exception occurred during their
/// execution, or do not modify `st0` register value. Since each of the exceptions can be predicted
/// with a code run by VM beforehand (unlike for arithmetic exceptions), the absence of `st0` value
/// change upon success allows batching multiple string operations and checking their final result,
/// while still maintaining ability to predict/detect which of the operations has failed.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum BytesOp {
    /// Put bytestring into a byte string register
    ///
    /// Data are kept in the separate data segment, thus when the instruction is parsed from the
    /// code segment it knows only data offset and length. If this offset or length exceeds the
    /// size of the data segment, the instruction truncates the string to the part that is present
    /// in the data segment (or zero-length string if the offset exceeds data segment length) and
    /// sets `st0` to `false`. Otherwise, `st0` is unaffected.
    #[display("put     {0},{1}")]
    Put(
        /** Destination `s` register index */ RegS,
        Box<ByteStr>,
        /** Indicates that the operation must set `st0` to false; i.e. string data are not
         * completely read from the data segment */
        bool,
    ),

    /// Move bytestring value between registers
    #[display("mov     {0},{1}")]
    Mov(/** Source `s` register index */ RegS, /** Destination `s` register index */ RegS),

    /// Swap bytestring value between registers
    #[display("swp     {0},{1}")]
    Swp(/** First `s` register index */ RegS, /** Second `s` register index */ RegS),

    /// Fill segment of bytestring with specific byte value, setting the length of the string in
    /// the destination register to specific value.
    ///
    /// The start offset is the least offset from one of the `a16` register provided in `offset`
    /// arguments, the end offset is the greatest one. If any of the offsets exceeds the length of
    /// the string in the destination register, operation behaviour is defined by the provided
    /// boolean flag:
    /// - if the flag is `true`, the string length is extended to the largest of the offsets and
    ///   all bytes between previous string length and start offset are filled with zeros, setting
    ///   `st0` value to `false`;
    /// - if the flag is `false`, the destination register is set to `None` and `st0` is set to
    ///   `false`.
    ///
    /// If both of the offsets lie within the length of the string, the `st0` register value is not
    /// modified.
    ///
    /// If any of the offsets or value registers are unset, sets `st0` to `false` and does not
    /// change destination value.
    #[display("fill.{4}    {0},a16{1},a16{2},a8{3}")]
    Fill(
        /** `s` register index */ RegS,
        /** `a16` register holding first offset */ Reg32,
        /** `a16` register holding second offset (exclusive) */ Reg32,
        /** `a8` register index holding the value */ Reg32,
        /** Exception handling flag */ ExtendFlag,
    ),

    /// Put length of the string into the destination register.
    ///
    /// If the string register is empty, or destination register can't fit the length, sets `st0`
    /// to `false` and destination register to `None`.
    #[display("len     {0},{1}{2}")]
    Len(/** `s` register index */ RegS, RegA, Reg32),

    /// Count number of byte occurrences from the `a8` register within the string and stores that
    /// value into destination `a16` register.
    ///
    /// If the string register is empty, or the source byte value register is uninitialized, sets
    /// `st0` to `false` and destination register to `None`.
    #[display("cnt     {0},a8{1},a16{2}")]
    Cnt(
        /** `s` register index */ RegS,
        /** `a8` register with the byte value */ Reg16,
        /** `a16` destination register index */ Reg16,
    ),

    /// Check equality of two strings, putting result into `st0`.
    ///
    /// If both of strings are uninitialized, `st0` assigned `true` value.
    #[display("eq      {0},{1}")]
    Eq(RegS, RegS),

    /// Compute offset and length of the `n`th fragment shared between two strings ("conjoint
    /// fragment"), putting it to the destination `a16` registers. If strings have no conjoint
    /// fragment sets destination to `None`.
    #[display("con     {0},{1},a16{2},a16{3},a16{4}")]
    Con(
        /** First source string register */ RegS,
        /** Second source string register */ RegS,
        /** Index of the conjoint fragment to match */ Reg32,
        /** `a16` register index to save the offset of the conjoint fragment */ Reg32,
        /** `a16` register index to save the length of the conjoint fragment */ Reg32,
    ),

    /// Count number of occurrences of one string within another putting result to `a16[0]`,
    ///
    /// If the first or the second string is `None`, sets `st0` to `false` and `a16[0]` to `None`.
    #[display("find    a16[0],{0},{1}")]
    Find(/** `s` register with string */ RegS, /** `s` register with matching fragment */ RegS),

    /// Extract byte string slice into general `r` register. The length of the extracted string is
    /// equal to the bit dimension of the destination register. If the bit size of the destination
    /// plus the initial offset exceeds string length the rest of the destination register bits is
    /// filled with zeros and `st0` is set to `false`. Otherwise, `st0` value is not modified.
    ///
    /// If the source string register - or offset register is uninitialized, sets destination to
    /// uninitialized state and `st0` to `false`.
    #[display("extr    {0},{1}{2},a16{3}")]
    Extr(/** `s` register index */ RegS, RegR, Reg16, /** `a16` register with offset */ Reg16),

    /// Inject general `R` register value at a given position to string register, replacing value
    /// of the corresponding bytes. If the insert offset is larger than the current length of the
    /// string, the length is extended and all bytes inbetween previous length and the new length
    /// are initialized with zeros. If the length of the inserted string plus insert offset exceeds
    /// the maximum string register length (2^16 bytes), than the destination register is set to
    /// `None` state and `st0` is set to `false`. Otherwise, `st0` value is not modified.
    #[display("inj     {0},{1}{2},{1}{3}")]
    Inj(
        /** `s` register index acting as the source and destination */ RegS,
        RegR,
        Reg16,
        /** `a16` register with offset */ Reg16,
    ),

    /// Join bytestrings from two registers into destination, overwriting its value. If the length
    /// of the joined string exceeds the maximum string register length (2^16 bytes), than the
    /// destination register is set to `None` state and `st0` is set to `false`. Otherwise,
    /// `st0` value is not modified.
    #[display("join    {0},{1},{2}")]
    Join(/** Source 1 */ RegS, /** Source 2 */ RegS, /** Destination */ RegS),

    /// Split bytestring at a given offset taken from `a16` register into two destination strings,
    /// overwriting their value. If offset exceeds the length of the string in the register,
    /// than the behaviour is determined by the [`SplitFlag`] value.
    ///
    /// <pre>
    /// +--------------------
    /// |       | ....
    /// +--------------------
    ///         ^       ^
    ///         |       +-- Split offset (`offset`)
    ///         +-- Source string length (`src_len`)
    ///
    /// `offset == 0`:
    ///   (1) first, second <- None; `st0` <- false
    ///   (2) first <- None, second <- `src_len > 0` ? src : None; `st0` <- false
    ///   (3) first <- None, second <- `src_len > 0` ? src : zero-len; `st0` <- false
    ///   (4) first <- zero-len, second <- `src_len > 0` ? src : zero-len
    /// `offset > 0 && offset > src_len`: `st0` always set to false
    ///   (1) first, second <- None
    ///   (5) first <- short, second <- None
    ///   (6) first <- short, second <- zero-len
    ///   (7) first <- zero-ext, second <- None
    ///   (8) first <- zero-ext, second <- zero-len
    /// `offset = src_len`:
    ///   (1) first, second <- None; `st0` <- false
    ///   (5,7) first <- ok, second <- None; `st0` <- false
    ///   (6,8) first <- ok, second <- zero-len
    /// `offset < src_len`: operation succeeds anyway, `st0` value is not changed
    /// </pre>
    ///
    /// Rule on `st0` changes: if at least one of the destination registers is set to `None`, or
    /// `offset` value exceeds source string length, `st0` is set to `false`; otherwise its value
    /// is not modified
    #[display("splt.{2}  {0},a16{1},{3},{4}")]
    Splt(
        SplitFlag,
        /** `a16` register index with offset value */ Reg32,
        /** Source */ RegS,
        /** Destination 1 */ RegS,
        /** Destination 2 */ RegS,
    ),

    /// Insert value from one of bytestring register at a given index of other bytestring register,
    /// shifting string bytes. If the destination register does not fit the length of the new
    /// string, or the offset exceeds the length of destination string operation behaviour is
    /// defined by the provided [`InsertFlag`].
    ///
    /// <pre>
    /// +--------------------
    /// |       | ....
    /// +--------------------
    ///         ^       ^
    ///         |       +-- Insert offset (`offset`)
    ///         +-- Destination string length (`dst_len`)
    ///
    /// `offset < dst_len && src_len + dst_len > 2^16`:
    ///   (6) Set destination to `None`
    ///   (7) Cut destination string part exceeding `2^16`
    ///   (8) Reduce `src_len` such that it will fit the destination
    /// `offset > dst_len && src_len + dst_len + offset <= 2^16`:
    ///   (1) Set destination to `None`
    ///   (2) Fill destination from `dst_let` to `offset` with zeros
    ///   (3) Use `src_len` instead of `offset`
    /// `offset > dst_len && src_len + dst_len + offset > 2^16`:
    ///   (4) Set destination to `None`
    ///   (5) Fill destination from `dst_let` to `offset` with zeros and cut source string part
    ///       exceeding `2^16`
    ///   (6-8) Use `src_len` instead of `offset` and use flag value from the first section
    /// </pre>
    ///
    /// In all of these cases `st0` is set to `false`. Otherwise, `st0` value is not modified.
    #[display("ins.{3}   {0},{1},a16{2}")]
    Ins(
        InsertFlag,
        /** `a16` register index with offset value for insert location */ Reg32,
        /** Source register */ RegS,
        /** Destination register */ RegS,
    ),

    /// Delete bytes in a given range, shifting the remaining bytes leftward. The start offset is
    /// the least offset from one of the `a16` register provided in `offset` arguments, the end
    /// offset is the greatest one. If any of the offsets exceeds the length of the string in
    /// the destination register, operation behaviour is defined by the provided [`DeleteFlag`]
    /// argument.
    ///
    /// <pre>
    /// +----------------------------------
    /// |                   | ....
    /// +----------------------------------
    ///     ^               ^       ^  
    ///     |               |       +-- End offset (`offset_end`)
    ///     |               +-- Source string length (`src_len`)
    ///     +-- Start offset (`offset_start`)
    ///
    /// `offset_start > src_len`:
    ///   (1) set destination to `None`
    ///   (2) set destination to zero-length string
    /// `offset_end > src_len && offset_start <= src_len`:
    ///   (1) set destination to `None`
    ///   (3) set destination to the fragment of the string `offset_start..src_len`
    ///   (4) set destination to the fragment of the string `offset_start..src_len` and extend
    ///       its length up to `offset_end - offset_start` with trailing zeros.
    /// </pre>
    ///
    /// `flag1` and `flag2` arguments indicate whether `st0` should be set to `false` if
    /// `offset_start > src_len` and `offset_end > src_len && offset_start <= src_len`.
    /// In all other cases, `st0` value is not modified.
    #[display("del.{0}   {7},{8},{1}{2},{3}{4},{5},{6}")]
    Del(
        DeleteFlag,
        RegA2,
        /** `a8` or `a16` register index with a first offset for delete location */ Reg32,
        RegA2,
        /** `a8` or `a16` register index with a second offset for delete location */ Reg32,
        /** `flag1` indicating `st0` value set to false if `offset_start > src_len` */ bool,
        /** `flag2` indicating `st0` value set to false if
         * `offset_end > src_len && offset_start <= src_len` */
        bool,
        /** Source `s` register */ RegS,
        /** Destination `s` register */ RegS,
    ),

    /// Revert byte order of the string.
    ///
    /// If the source string register is uninitialized, resets destination to the uninitialized
    /// state and sets `st0` to `false`.
    #[display("rev     {0},{1}")]
    Rev(/** Source */ RegS, /** Destination */ RegS),
}

/// Cryptographic hashing functions
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[non_exhaustive]
pub enum DigestOp {
    /// Computes RIPEMD160 hash value.
    ///
    /// Sets `st0` to `false` and destination register to `None` if the source register does not
    /// contain a value
    #[display("ripemd  {0},r160{1}")]
    Ripemd(
        /** Index of string register */ RegS,
        /** Index of `r160` register to save result to */ Reg16,
    ),

    /// Computes SHA256 hash value
    ///
    /// Sets `st0` to `false` and destination register to `None` if the source register does not
    /// contain a value
    #[display("sha2    {0},r256{1}")]
    Sha256(
        /** Index of string register */ RegS,
        /** Index of `r256` register to save result to */ Reg16,
    ),

    /// Computes SHA256 hash value
    ///
    /// Sets `st0` to `false` and destination register to `None` if the source register does not
    /// contain a value
    #[display("sha2    {0},r512{1}")]
    Sha512(
        /** Index of string register */ RegS,
        /** Index of `r512` register to save result to */ Reg16,
    ),
}

/// Operations on Secp256k1 elliptic curve
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum Secp256k1Op {
    /// Generates new elliptic curve point value saved into destination
    /// register in `r512` set using scalar value from the source `r256`
    /// register
    #[display("secpgen r256{0},r512{1}")]
    Gen(
        /** Register containing scalar */ Reg32,
        /** Destination register to put G * scalar */ Reg8,
    ),

    /// Multiplies elliptic curve point on a scalar
    #[display("secpmul {0}256{1},r512{2},r512{3}")]
    Mul(
        /** Use `a` or `r` register as scalar source */ RegBlockAR,
        /** Scalar register index */ Reg32,
        /** Source `r` register index containing EC point */ Reg32,
        /** Destination `r` register index */ Reg32,
    ),

    /// Adds two elliptic curve points
    #[display("secpadd r512{0},r512{1}")]
    Add(/** Source 1 */ Reg32, /** Source 2 and destination */ Reg8),

    /// Negates elliptic curve point
    #[display("secpneg r512{0},r512{1}")]
    Neg(/** Register hilding EC point to negate */ Reg32, /** Destination register */ Reg8),
}

/// Operations on Curve25519 elliptic curve
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum Curve25519Op {
    /// Generates new elliptic curve point value saved into destination
    /// register in `r512` set using scalar value from the source `r256`
    /// register
    #[display("edgen   r256{0},r256{1}")]
    Gen(
        /** Register containing scalar */ Reg32,
        /** Destination register to put G * scalar */ Reg8,
    ),

    /// Multiplies elliptic curve point on a scalar
    #[display("edmul   {0}256{1},r256{2},r256{3}")]
    Mul(
        /** Use `a` or `r` register as scalar source */ RegBlockAR,
        /** Scalar register index */ Reg32,
        /** Source `r` register index containing EC point */ Reg32,
        /** Destination `r` register index */ Reg32,
    ),

    /// Adds two elliptic curve points
    #[display("edadd   r512{0},r256{1},r256{2},{3}")]
    Add(
        /** Source 1 */ Reg32,
        /** Source 2 */ Reg32,
        /** Destination register */ Reg32,
        /** Allow overflows */ bool,
    ),

    /// Negates elliptic curve point
    #[display("edneg   r256{0},r256{1}")]
    Neg(/** Register hilding EC point to negate */ Reg32, /** Destination register */ Reg8),
}
