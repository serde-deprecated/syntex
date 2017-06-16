// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use extprim::i128::i128;
use extprim::u128::u128;

#[inline]
/// encodes an integer using unsigned leb128 encoding and stores
/// the result using a callback function.
///
/// The callback `write` is called once for each position
/// that is to be written to with the byte to be encoded
/// at that position.
pub fn write_unsigned_leb128_to<W>(mut value: u128, mut write: W) -> usize
    where W: FnMut(usize, u8)
{
    let mut position = 0;
    loop {
        let mut byte = (value & u128::from(0x7Fu64)).low64() as u8;
        value >>= 7;
        if value != u128::from(0u64) {
            byte |= 0x80;
        }

        write(position, byte);
        position += 1;

        if value == u128::from(0u64) {
            break;
        }
    }

    position
}

#[inline]
/// encodes an integer using signed leb128 encoding and stores
/// the result using a callback function.
///
/// The callback `write` is called once for each position
/// that is to be written to with the byte to be encoded
/// at that position.
pub fn write_signed_leb128_to<W>(mut value: i128, mut write: W) -> usize
    where W: FnMut(usize, u8)
{
    let mut position = 0;

    loop {
        let mut byte = (value.low64() as u8) & 0x7f;
        value >>= 7;
        let more = !((((value == i128::from(0)) && ((byte & 0x40) == 0)) ||
                      ((value == i128::from(-1)) && ((byte & 0x40) != 0))));

        if more {
            byte |= 0x80; // Mark this byte to show that more bytes will follow.
        }

        write(position, byte);
        position += 1;

        if !more {
            break;
        }
    }
    position
}
