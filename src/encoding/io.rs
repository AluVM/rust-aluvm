// AluRE: AluVM runtime environment.
// This is rust implementation of AluVM (arithmetic logic unit virtual machine).
//
// Designed & written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// This software is licensed under the terms of MIT License.
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify_num::{u1, u2, u3, u4, u5, u6, u7};

use crate::reg::{RegAR, Value};

// TODO: Make it sealed
pub trait Read {
    type Error;

    fn is_end(&self) -> bool;
    fn peek_u8(&self) -> Result<u8, Self::Error>;
    fn read_bool(&mut self) -> Result<bool, Self::Error>;
    fn read_u1(&mut self) -> Result<u1, Self::Error>;
    fn read_u2(&mut self) -> Result<u2, Self::Error>;
    fn read_u3(&mut self) -> Result<u3, Self::Error>;
    fn read_u4(&mut self) -> Result<u4, Self::Error>;
    fn read_u5(&mut self) -> Result<u5, Self::Error>;
    fn read_u6(&mut self) -> Result<u6, Self::Error>;
    fn read_u7(&mut self) -> Result<u7, Self::Error>;
    fn read_u8(&mut self) -> Result<u8, Self::Error>;
    fn read_u16(&mut self) -> Result<u16, Self::Error>;
    fn read_bytes32(&mut self) -> Result<[u8; 32], Self::Error>;
    fn read_slice(&mut self) -> Result<&[u8], Self::Error>;
    fn read_value(&mut self, reg: RegAR) -> Result<Value, Self::Error>;
}

pub trait Write {
    type Error;

    fn write_bool(&mut self, data: bool) -> Result<(), Self::Error>;
    fn write_u1(&mut self, data: impl Into<u1>) -> Result<(), Self::Error>;
    fn write_u2(&mut self, data: impl Into<u2>) -> Result<(), Self::Error>;
    fn write_u3(&mut self, data: impl Into<u3>) -> Result<(), Self::Error>;
    fn write_u4(&mut self, data: impl Into<u4>) -> Result<(), Self::Error>;
    fn write_u5(&mut self, data: impl Into<u5>) -> Result<(), Self::Error>;
    fn write_u6(&mut self, data: impl Into<u6>) -> Result<(), Self::Error>;
    fn write_u7(&mut self, data: impl Into<u7>) -> Result<(), Self::Error>;
    fn write_u8(&mut self, data: impl Into<u8>) -> Result<(), Self::Error>;
    fn write_u16(&mut self, data: impl Into<u16>) -> Result<(), Self::Error>;
    fn write_bytes32(&mut self, data: [u8; 32]) -> Result<(), Self::Error>;
    fn write_slice(&mut self, bytes: impl AsRef<[u8]>) -> Result<(), Self::Error>;
    fn write_value(&mut self, reg: RegAR, value: &Value) -> Result<(), Self::Error>;
}
