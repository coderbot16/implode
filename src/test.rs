use implode::symbol::{Symbol, CodeTable, DEFAULT_CODE_TABLE, decode_bits, BitDecodeError, END};

// For the lit_* tests, pass a value of 0xFFFF_FFFF as dict_bits, 
// so interpretation as a Pair would certainly cause a failed test.

#[test]
fn test_lit() {
    let mut iterations: u32 = 0;
    for b in 0..256 {
        let byte: u64 = b << 1;
        let out = decode_bits(byte, 9, &DEFAULT_CODE_TABLE, 0xFFFF_FFFF);
        let unwrapped = match out {
            Ok(symbol) => symbol,
            Err(err) => panic!("Err result returned from decode_bits: {:?}", err)
        };
        
        
        assert_eq!(Symbol::Literal(b as u8), unwrapped.decoded);
        assert_eq!(9, unwrapped.used_bits);
        iterations = iterations+1;
    }

    assert_eq!(iterations, 256);
}

#[test]
fn test_lit_insufficent_bits() {
    for x in 0..9 {
        match decode_bits(0, x, &DEFAULT_CODE_TABLE, 0xFFFF_FFFF) {
            Ok(_) => panic!("Ok result returned for insufficent nbits!"),
            Err(err) => match err {
                BitDecodeError::NotEnoughBits(n) => assert_eq!(9-x, n)
            }
        }
    }
}

#[test]
fn test_end() {
	let lit = 0xff01;
	// a dict_bits value of 0xFFFF_FFFF causes an overflow error when it then tries to wrongly
	// read a distance value after the end literal
	let decode = match decode_bits(lit, 16, &DEFAULT_CODE_TABLE, 0xFFFF_FFFF) {
		Ok(decoded) => decoded,
		Err(err) => panic!("Err result returned from decode_bits: {:?}", err)
	};
	
	assert_eq!(Symbol::End, decode.decoded);
	assert_eq!(16, decode.used_bits);
}

#[test]
// Bugs fixed from this test:
// #1 len_base not properly applied when extra_bits is not 0 but extra is 0
fn test_pair() {
	
	let len_code = [5, 3, 1, 6, 10, 2, 12, 20, 4, 24, 8, 48, 16, 32, 64, 0];
	let dist_code = [3, 13, 5, 25, 9, 17, 1, 62, 30, 46, 14, 54, 22, 38, 6, 58, 
					 26, 42, 10, 50, 18, 34, 66, 2, 124, 60, 92, 28, 108, 44, 76, 12, 
					 116, 52, 84, 20, 100, 36, 68, 4, 120, 56, 88, 24, 104, 40, 72, 8, 
					 240, 112, 176, 48, 208, 80, 144, 16, 224, 96, 160, 32, 192, 64, 128, 0];
	
	let mut len = 0;
	
	for i in 0..16 {
		for n in 0..(1 << DEFAULT_CODE_TABLE.extra_len_bits[i]) {
			let bits: u32 = (DEFAULT_CODE_TABLE.extra_len_bits[i] as u32) + (DEFAULT_CODE_TABLE.len_bits[i] as u32) + 1;
			let code: u64 = (n << ((DEFAULT_CODE_TABLE.len_bits[i] as u64) + 1)) | ((len_code[i]& 0xFFFF00FF) << 1) | 1;
			
			println!("# Length: code {}, bits {}, val {}", code, bits, len);
			
			if len == END
			{
				//This is the end length.
				return;
			}
			
			let mut dst = 0;
			
			for j in 0..64 {
				let dbits: u32 = bits + (DEFAULT_CODE_TABLE.dist_bits[j] as u32);
				let dcode: u64 = ((dist_code[j] as u64) << bits) | code;
				
				println!("# Dist: dcode {}, dbits {}, val {}", dcode, dbits, dst);
				
				if len == 0 {
					
					let mut subdst = 0;
					
					for x in 0..4 {
						println!("# Subdist: val {}", subdst);
						
						let s = match decode_bits((x << dbits) | dcode, dbits+2, &DEFAULT_CODE_TABLE, 6) {
				            Ok(symbol) => symbol,
				            Err(err) => panic!("Err result returned from decode_bits: {:?}", err)
						};
						
						assert_eq!(dbits+2, s.used_bits);
						assert_eq!(Symbol::Pair {distance: ((dst<<2) | subdst)+1, length: len+2}, s.decoded);
						
						subdst = subdst + 1;
					}
				} else {
					for dsize in 0..7 {
						
						let mut subdst = 0;
						
						for x in 0..(1<<dsize) {
							println!("# Subdist: val {}", subdst);
							
							let s = match decode_bits((x << dbits) | dcode, dbits+dsize, &DEFAULT_CODE_TABLE, dsize) {
					            Ok(symbol) => symbol,
					            Err(err) => panic!("Err result returned from decode_bits: {:?}", err)
							};
							
							assert_eq!(dbits+dsize, s.used_bits);
							assert_eq!(Symbol::Pair {distance: ((dst<<dsize) | subdst)+1, length: len+2}, s.decoded);
						
							subdst = subdst + 1;
						}
					}
				}
				
				dst = dst+1;
			}
			
			len = len+1;
		}
	}
}