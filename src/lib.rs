//! Implementation of the PKWARE DCL Implode algorithm.
//! Note that this is different from the Imploding algorithm in PKZIP.
//! This implementation is based off of the one in zlib/contrib/blast.c, but
//! uses lookup tables to decode the Shannon-Fano codes instead, which is way faster.
//! This implementation uses no code from the PKWARE Data Compression Library (PKWARE DCL).
//!
//! ```
//! use implode::symbol::{Symbol, CodeTable, DEFAULT_CODE_TABLE, decode_bits};
//! use implode::exploder::Exploder;
//! use std::fs::File;
//! use std::io::Read;
//! use std::io::Write;
//! 
//! fn main() 
//! {
//! 	let mut f = File::open("/path/to/input").expect("File open failed");
//! 	let mut buf = [0u8; FILE_SIZE];
//! 	f.read(&mut buf);
//! 	let mut exploder = Exploder::new(&DEFAULT_CODE_TABLE);
//! 	
//! 	let mut ex = File::create("/path/to/output").expect("File create failed");
//! 	let mut cpos: u32 = 0;
//! 	let len = buf.len();
//! 	let mut iter = 0;
//! 	
//! 	while !exploder.ended {
//! 		
//! 		let abuf = &mut buf[cpos as usize .. len];
//! 		
//! 		let x = exploder.explode_block(abuf).unwrap();
//! 		cpos += x.0;
//! 		
//! 		//println!("{} {:?}",x.0, x.1);
//! 		let bf = x.1;
//! 	
//! 		ex.write(bf);
//! 		iter+=1;
//! 	}
//! 	
//! }

#[cfg(test)]
mod test;

pub mod symbol;
pub mod exploder;