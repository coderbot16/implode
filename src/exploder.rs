use symbol::{Symbol, CodeTable, decode_bits, BitDecodeError};
use std::result;
use std::cmp;

pub struct Exploder<'a> {
	pub table: &'a CodeTable,
	pub dict_bits: u32,
	pub need_mode: bool, 
	pub need_dict_bits: bool,
	pub ended: bool,
	pub need_swap: bool,
	
	pub buff: [u8; 8192],
	//XOR to be applied to offset
	pub offs_xor: u32,
	pub write_offs: u32,
	
	// state
	pub bits: u64,
	pub nbits: u32,
	pub remaining_len: u32,
	pub current_offs: u32,
}

pub type Result<T> = result::Result<T, ExplodeError>;

#[derive(Debug)]
pub enum ExplodeError {
	NeedMode,
	NeedDictBits,
	NeedMoreBytes,
	BitDecodeError,
}

impl<'a> Exploder<'a> {
	
	pub fn new(table: &'a CodeTable) -> Exploder {
		Exploder {
			table: table,
			dict_bits: 0,
			need_mode: true, 
			need_dict_bits: true,
			ended: false,
			need_swap: false,
			
			buff: [0u8; 8192],
			//XOR to be applied to offset
			offs_xor: 0,
			write_offs: 4096,
			
			// state
			bits: 0,
			nbits: 0,
			remaining_len: 0,
			current_offs: 0
		}
	}
	
	pub fn swap(&mut self) {
		self.offs_xor ^= 4096;
		self.write_offs = 4096;
		if self.remaining_len>0 {
			assert!(self.current_offs>=4096);
			self.current_offs -= 4096;
		}
	}
	
	pub fn reset(&mut self) {
		self.offs_xor = 0;
		self.write_offs = 4096;
		self.bits = 0;
		self.nbits = 0;
		self.remaining_len = 0;
		self.current_offs = 0;
		
		self.need_mode = true;
		self.need_dict_bits = true;
		self.dict_bits = 0;
	}
	
	fn handle_pair(&mut self, mut from_offs: u32, length: u32) -> bool {
		let mut write_offs = self.write_offs;
		let alen = cmp::min(length, 8192 - write_offs);
		
		for _ in 0..alen {			
			self.buff[(write_offs ^ self.offs_xor) as usize] = self.buff[(from_offs ^ self.offs_xor) as usize];
			write_offs+=1;
			from_offs+=1;
		}
		
		self.remaining_len = length-alen;
		self.current_offs = from_offs;
		self.write_offs = write_offs;
		self.remaining_len!=0
	}
	
	pub fn explode_block(&mut self, data: &[u8]) -> Result<(usize, &[u8])> {
		let mut pos: usize = 0;
		
		if self.need_mode {
			if data.len() < 1 {
				return Err(ExplodeError::NeedMode);
			}
			
			if data[0] == 1 {
				unimplemented!();
			}
			
			self.need_mode = false;
			pos = 1;
		}
		
		if self.need_dict_bits {
			if (data.len() as usize) - pos < 1 {
				return Err(ExplodeError::NeedDictBits);
			}
			
			self.dict_bits = data[pos as usize] as u32;
			
			self.need_dict_bits = false;
			pos += 1;
		}
		
		if self.need_swap {
			self.swap();
			
			if self.remaining_len != 0 {
				let co = self.current_offs;
				let rl = self.remaining_len;
				self.handle_pair(co, rl);
			}
			
			self.need_swap = false;
		}
		
		let mut bits = self.bits;
		let mut nbits = self.nbits;
		
		//println!("Buffer size: {}", data.len());
		
		loop {
			while nbits <= 56 && pos < data.len() {
				bits |= (data[pos as usize] as u64) << nbits;
				pos += 1;
				nbits += 8;
			}
			
			if self.write_offs == 8192 {
				//println!("need swap");
				self.need_swap = true;
				break
			}
			
			match decode_bits(bits, nbits, self.table, self.dict_bits) {
				Ok(res) => {
					nbits -= res.used_bits;
					bits >>= res.used_bits;
					match res.decoded {
						Symbol::Literal(byte) => {
							//println!("literal {} ({:X}) @ buf:{}/d:{}", byte, byte, self.write_offs ^ self.offs_xor, self.write_offs);
							self.buff[(self.write_offs ^ self.offs_xor) as usize] = byte;
							self.write_offs += 1;
						},
						Symbol::Pair{distance, length} => {
							//println!("pair distance={}, length={}", distance, length);
							let wo = self.write_offs;
							//println!("{}-{}", wo, distance);
							if self.handle_pair(wo-distance, length) {
								self.need_swap = true;
								//println!("need swap");
								break
							}
						},
						Symbol::End => {
							//println!("end");
							self.ended = true;
							break
						}
					}
				},
				Err(err) => match err {
					BitDecodeError::NotEnoughBits(_) => {/*println!("NEB: NEED {}, HAVE {} POS {}", x, nbits, pos);*/ break}
				}
			}
		}
		
		self.nbits = nbits;
		self.bits = bits;
		
		//TODO: If end of data, reduce size of return.
		// If offs_xor is 0, range is 4096..8192
		// If offs_xor is 4096, range is 0..4096
		
		//println!("{}, {}", self.offs_xor, self.write_offs);
		
		// Return the amount of data used from the input data and a slice to the output data.
		return Ok((pos, &self.buff[(4096-self.offs_xor) as usize .. (self.write_offs-self.offs_xor) as usize]));
	}
}