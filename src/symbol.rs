use std::result;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
/// Represents a symbol in the Implode bit stream.
pub enum Symbol {
    /// A literal byte or ASCII character.
    Literal(u8),
    /// A length/distance pair, according to LZ77.
    Pair {distance: u32, length: u32},
    /// The end symbol. Represents the end of the Implode data.
    End
}

impl Eq for Symbol {}

/// A simple trait that allows an object to be converted to bit form.
pub trait ToBits {
    /// Serializes this structure to at most 32 bits.
    fn to_bits(&self) -> u32;
    /// Returns the amount of bits in this structure when serialized.
    fn size_bits(&self) -> u32;
}

macro_rules! bitmask {
	($bits:expr) => ((1<<$bits)-1)
}

#[derive(Debug)]
/// Represents the output of bit decoding.
pub struct BitDecodeOutput<T> {
    /// The decoded object.
    pub decoded: T,
    /// Amount of bits consumed from the bit stream.
    pub used_bits: u32
}

#[derive(Debug)]
/// Represents an error decoding bits.
pub enum BitDecodeError {
    /// Error when the available bits are insufficent.
    /// Contains the amount of additional bits needed.
    NotEnoughBits(u32)
}

/// Result of calling an implode function.
pub type Result<T> = result::Result<T, BitDecodeError>;

/// Initialization data for the Length decode table.
pub struct LenInit {
    pub len_bits: [u8; 16],
    pub len_code: [u8; 16]
}

/// Initialization data for the Distance decode table.
pub struct DistInit {
    pub dist_bits: [u8; 64],
    pub dist_code: [u8; 64]
}

/// Initialization data for the Shannon-Fano literal decode table.
pub struct ShannonInit {
    /// Amount of bits for each code.
    pub shan_bits: [u8; 256],
    /// The codes.
    /// Use the byte you want to encode as the index.
    pub shan_code: [u16; 256]
}

/// Contains decoding data, used in decode_bits.
pub struct CodeTable {
    pub extra_len_bits: [u8; 16],
    pub len_base: [u16; 16],
    pub len_add: [u16; 16],
    pub len_bits: [u8; 16],
    pub len_codes: [u8; 256],

    pub dist_bits: [u8; 64],
    pub dist_codes: [u8; 256],
    
    pub shan_lut: [u8; 256],
    pub shan_4_lut: [u8; 256],
    pub shan_6_lut: [u8; 128],
    pub shan_8_lut: [u8; 256], 
}

/// Contains encoding data.
pub struct EncodeTable {
    pub lit_bits: [u8; 256],
    pub lit_code: [u16; 256],
    pub len_bits: [u8; 518],
    pub len_code: [u16; 518],
    pub dist_bits: [u8; 64],
    pub dist_code: [u8; 64],
    pub dict_bits: u32
}

/// The default code table, contains settings used by the standard PKWARE DCL.
/// Use this if you need to read files compressed by the DCL.
pub static DEFAULT_CODE_TABLE: CodeTable = CodeTable {
    extra_len_bits: [0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8],
    len_base:  [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 14, 22, 38, 70, 134, 262],
    len_add:  [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 4, 11, 26, 57, 120, 247],
    len_bits:  [3, 2, 3, 3, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 7, 7],
    len_codes: [15, 2, 5, 1, 8, 0, 3, 1, 10, 2, 4, 1, 6, 0, 3, 1, 12, 2, 5, 1, 7, 0, 3, 1, 9, 2, 4, 1, 6, 0, 3, 1, 
                13, 2, 5, 1, 8, 0, 3, 1, 10, 2, 4, 1, 6, 0, 3, 1, 11, 2, 5, 1, 7, 0, 3, 1, 9, 2, 4, 1, 6, 0, 3, 1, 
                14, 2, 5, 1, 8, 0, 3, 1, 10, 2, 4, 1, 6, 0, 3, 1, 12, 2, 5, 1, 7, 0, 3, 1, 9, 2, 4, 1, 6, 0, 3, 1, 
                13, 2, 5, 1, 8, 0, 3, 1, 10, 2, 4, 1, 6, 0, 3, 1, 11, 2, 5, 1, 7, 0, 3, 1, 9, 2, 4, 1, 6, 0, 3, 1, 
                15, 2, 5, 1, 8, 0, 3, 1, 10, 2, 4, 1, 6, 0, 3, 1, 12, 2, 5, 1, 7, 0, 3, 1, 9, 2, 4, 1, 6, 0, 3, 1, 
                13, 2, 5, 1, 8, 0, 3, 1, 10, 2, 4, 1, 6, 0, 3, 1, 11, 2, 5, 1, 7, 0, 3, 1, 9, 2, 4, 1, 6, 0, 3, 1, 
                14, 2, 5, 1, 8, 0, 3, 1, 10, 2, 4, 1, 6, 0, 3, 1, 12, 2, 5, 1, 7, 0, 3, 1, 9, 2, 4, 1, 6, 0, 3, 1, 
                13, 2, 5, 1, 8, 0, 3, 1, 10, 2, 4, 1, 6, 0, 3, 1, 11, 2, 5, 1, 7, 0, 3, 1, 9, 2, 4, 1, 6, 0, 3, 1],

    dist_bits:  [2, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 
                 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8],    
    dist_codes: [63, 6, 23, 0, 39, 2, 14, 0, 47, 4, 18, 0, 31, 1, 10, 0, 55, 5, 20, 0, 35, 2, 12, 0, 43, 3, 16, 0, 27, 1, 8, 0, 
				 59, 6, 21, 0, 37, 2, 13, 0, 45, 4, 17, 0, 29, 1, 9,  0, 51, 5, 19, 0, 33, 2, 11, 0, 41, 3, 15, 0, 25, 1, 7, 0, 
				 61, 6, 22, 0, 38, 2, 14, 0, 46, 4, 18, 0, 30, 1, 10, 0, 53, 5, 20, 0, 34, 2, 12, 0, 42, 3, 16, 0, 26, 1, 8, 0, 
				 57, 6, 21, 0, 36, 2, 13, 0, 44, 4, 17, 0, 28, 1, 9,  0, 49, 5, 19, 0, 32, 2, 11, 0, 40, 3, 15, 0, 24, 1, 7, 0, 
				 62, 6, 23, 0, 39, 2, 14, 0, 47, 4, 18, 0, 31, 1, 10, 0, 54, 5, 20, 0, 35, 2, 12, 0, 43, 3, 16, 0, 27, 1, 8, 0, 
				 58, 6, 21, 0, 37, 2, 13, 0, 45, 4, 17, 0, 29, 1, 9,  0, 50, 5, 19, 0, 33, 2, 11, 0, 41, 3, 15, 0, 25, 1, 7, 0, 
				 60, 6, 22, 0, 38, 2, 14, 0, 46, 4, 18, 0, 30, 1, 10, 0, 52, 5, 20, 0, 34, 2, 12, 0, 42, 3, 16, 0, 26, 1, 8, 0, 
				 56, 6, 21, 0, 36, 2, 13, 0, 44, 4, 17, 0, 28, 1, 9,  0, 48, 5, 19, 0, 32, 2, 11, 0, 40, 3, 15, 0, 24, 1, 7, 0],
    
    shan_lut: [0; 256], //todo
    shan_4_lut: [0; 256],//todo
    shan_6_lut: [0; 128],//todo
    shan_8_lut: [0; 256],//todo
};

/// Length value of end literal
pub const END: u32 = 517;

/*pub fn symbol_to_bits(symbol: Symbol, _table: &EncodeTable) -> u32 {
    match symbol {
        Symbol::Literal(byte) => ((byte as u32) << 1) | 1,
        Symbol::Pair{ distance, length} => unimplemented!(),
        Symbol::End => unimplemented!()
    }
}

pub fn symbol_bits(symbol: Symbol, table: &EncodeTable) -> u32 {
    match symbol {
        Symbol::Literal(byte) => table.lit_bits[byte as usize] as u32,
        Symbol::Pair{distance: dist, length: len} => {
            (table.len_bits[(len as usize)-1] as u32) + 
            if len==2 {
                (table.dist_bits[(dist >> 2) as usize] as u32) + 2
            } else {
                (table.dist_bits[(dist >> table.dict_bits) as usize] as u32) + table.dict_bits
            }
        },
        Symbol::End => table.len_bits[517] as u32
    }
}*/

/// Decodes the input bits into a symbol.
///
/// decode_bits is the core of the decompression. It takes in the bit buffer, code table, and dict_bits, and outputs 1 decoded symbol.
/// nbits should reflect the amount of bits in the bit buffer, but is only used for error checking. dict_bits is the real dict_bits - 6,
/// because it represents the literal dict bits. (6 bits are always coded using Shannon Fano codes.) For example, a dictionary size of 4096 (2^12)
/// would have a dict_bits value of 6 because 12 - 6 is 6. Beware that this value is limited from 4 to 6 in the PKWARE® implementation.
/// This implementation has no such restriction.
///
/// # Examples
///
/// ```
/// use implode;
/// 
/// assert_eq!(implode::Symbol::Literal(0), implode::decode_bits(0, 9, &implode::DEFAULT_CODE_TABLE));
/// ```
///
/// # Errors
/// 
/// If nbits is less than 1, returns BitDecodeError::NotEnoughBits(1).
/// If the bit stream indicates a literal, and nbits is less than 9, returns BitDecodeError::NotEnoughBits(9-nbits).
/// 
pub fn decode_bits(bits: u64, nbits: u32, table: &CodeTable, dict_bits: u32) -> Result<BitDecodeOutput<Symbol>> {
    let is_pair: u64 = bits&1;
    let next_byte: u64 = (bits>>1)&0xFF;
	let mut used_bits;

    let sym = if is_pair==1 {
	    let code: u32 = table.len_codes[next_byte as usize] as u32;
	    let code_bits: u32 = table.len_bits[code as usize] as u32;
	    let extra_bits: u32 = table.extra_len_bits[code as usize] as u32;
	    
		used_bits = code_bits + extra_bits + 1;
	    
		//Shift the bits over by 1+code_bits (used bits so far), and then create a bitmask from extra_bits.
		let extra: u64 = (bits>>(1+code_bits)) & bitmask!(extra_bits);
        
	    let length = if extra_bits>0 {table.len_add[code as usize] as u32} else {0} + code + (extra as u32);
        
        if length == END {
        	if nbits<used_bits {
		        return Err(BitDecodeError::NotEnoughBits(used_bits-nbits))
		    }
        	
        	return Ok(BitDecodeOutput::<Symbol>{decoded: Symbol::End, used_bits: used_bits})
        }
        
        // Now decode the distance. This is encoded like DEFLATE distances,
        // where there is a coded part and a literal part.
        
        let dist_code: u8 = table.dist_codes[((bits>>used_bits)&0xFF) as usize];
        
        let add_bits: u32 = if length==0 {2} else {dict_bits};
        used_bits = used_bits+(table.dist_bits[dist_code as usize] as u32);
        
        let distance: u32 = ((dist_code as u32) << add_bits) | (((bits>>used_bits) & bitmask!(add_bits)) as u32);
        
        used_bits = used_bits + add_bits;
        Symbol::Pair{distance: distance + 1, length: length + 2}
    } else {
    	used_bits = 9;
        Symbol::Literal(next_byte as u8)
    };

	if nbits<used_bits {
        return Err(BitDecodeError::NotEnoughBits(used_bits-nbits))
    }

    Ok(BitDecodeOutput::<Symbol>{decoded: sym, used_bits: used_bits})
}