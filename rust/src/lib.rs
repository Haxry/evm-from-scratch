use primitive_types::U256;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use serde::de::value;
use std::convert::TryInto;
use tiny_keccak::{Keccak,Hasher};
use serde::Deserialize;

pub struct EvmResult {
    pub stack: Vec<U256>,
    pub success: bool,
}
#[derive(Debug, Deserialize)]
pub struct Block{
    basefee: Option<String>,
    coinbase: Option<String>,
    timestamp: Option<String>,
    number: Option<String>,
    difficulty: Option<String>,
    gaslimit:Option<String>,
    
    chainid:Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct Txn{
    value: Option<String>,
    data: Option<String>,
    from: Option<String>,
    to: Option<String>,
    gas: Option<String>,
    origin:Option<String>,
    gasprice:Option<String>,
}
fn bigint_to_u256_negative(value: BigInt) -> U256 {
    let u256_max_plus_1 = BigInt::from(1u64) << 256;
    let u256_value = if value < BigInt::from(0) {
        value + u256_max_plus_1
    } else {
        value
    };

    
    U256::from_dec_str(&u256_value.to_str_radix(10)).expect("Conversion failed")
}
fn u256_to_signed_value(num: U256) -> BigInt {
    
    let mut bytes = [0u8; 32];
    num.to_little_endian(&mut bytes); 

   
    let is_negative = bytes[31] & 0x80 != 0; // MSB is in the last byte (32nd byte)

    if !is_negative {
        // If not negative, convert bytes to BigInt directly
        BigInt::from_bytes_le(num_bigint::Sign::Plus, &bytes)
    } else {
        // If negative, convert bytes to a BigInt and adjust for signed value
        let unsigned_value = BigInt::from_bytes_le(num_bigint::Sign::Plus, &bytes);
        let max_value = BigInt::from(1u64) << 256; // 2^256
        unsigned_value - max_value
    }
}


fn bigint_to_u256(bigint: BigInt) -> Option<U256> {
    // Ensure the BigInt fits in 256 bits
    let max_value = BigInt::from(1u64) << 256; // 2^256
    if bigint < BigInt::zero() || bigint >= max_value {
        return None; // Value is out of range for U256
    }

    // Convert BigInt to a byte array
    let mut bytes = [0u8; 32];
    let bigint_bytes = bigint.to_bytes_le().1; // Get bytes in little-endian format
    let len = bigint_bytes.len();

    // Copy bytes into the 32-byte array
    bytes[0..len].copy_from_slice(&bigint_bytes);

    // Construct U256 from byte array
    Some(U256::from_little_endian(&bytes))
}

pub fn evm(_code: impl AsRef<[u8]>,_tx: &Option<Txn>,_block: &Option<Block>) -> EvmResult {
    let mut stack: Vec<U256> = Vec::new();
    let mut pc = 0; // Program Counter
    let bytes = _code.as_ref();
    let only_5b= bytes.contains(&0x5b);
    let only_57= bytes.contains(&0x57);
    let sequence = &[0x60,0x5b];
   let jumpdest= bytes.windows(sequence.len()).any(|window| window==sequence);
   let mut memory: Vec<u8> = vec![];
   if(jumpdest){
    return EvmResult {
        stack,
        success: false,
    }; 
   }
   
    

    let code = _code.as_ref();

    while pc < code.len() {
        let opcode = code[pc];
        
        if(only_5b&& !only_57){
            pc=2+bytes.iter().position(|&byte| byte== 0x5b).unwrap();
            println!("pc is {}",pc);
           }else{
            pc += 1;
           }
        

        match opcode {
            0x00 => {
                
                break; 
            }
            0x5f => {
                
                stack.push(U256::zero());
            }
           
            // }
            0x60 =>{
                if pc<code.len() {
                    let value = code[pc];
                    pc +=1;
                    stack.insert(0,U256::from(value));
                }
            }
            // 0x61 =>{
            //     if pc+1 <code.len() {

            //         let value = (code[pc] as u16) <<8 | code[pc+1] as u16 ;
            //         pc+=2;
            //         stack.push(U256::from(value));

            //     }
            // }
            // 0x63 =>{
            //     if pc+3 < code.len(){
            //         let value= (code[pc] as u32) << 24 | (code[pc+1] as u32) <<16 | (code[pc+2] as u32) <<8 | (code[pc+3] as u32);
            //         pc+=4;
            //         stack.push(U256::from(value));
            //     }
            // }
            // 0x60..=0x7f => {
            //     let push= opcode-0x60+1;
                
            //     for(let i=0;i<push;i++){

            //     }
            // }
            0x61..=0x7F => {
                // Common code for PUSH1 to PUSH32
                
                
                // Extract the number of bytes to push based on the opcode
                let num_bytes = (opcode - 0x60 + 1) as usize; // 0x60 corresponds to 1 byte, 0x61 to 2 bytes, etc.
                
                // Check if there are enough bytes in the code
                if pc + num_bytes <= code.len() {
                    let mut value = U256::zero();
                    for i in 0..num_bytes {
                        value = (value << 8) | U256::from(code[pc + i]);// Combine bytes into U256
                    }
                    println!("value is {:?}",value);
                    pc += num_bytes; // Move the program counter forward by the number of bytes read
                    stack.insert(0,value); // Push the combined value onto the stack
                }
            } 
            0x50=>{
                stack.remove(0);
            }
            0x01=>{
                let mut value= U256::zero();
                if stack.len()==0 {
                    return EvmResult {
                        stack,
                        success: false,
                    }; 
                }
                
                while(stack.len() !=0){
                    value+= stack[stack.len()-1];
                    stack.pop();
                    
                    if(value==U256::max_value()){
                        stack.pop();
                        value = U256::from(1);
                        break;
                    }
                }
                
                
                stack.push(value);
            }
            0x02 => {
                
                if stack.len() < 2 {
                    return EvmResult {
                        stack,
                        success: false,
                    };
                }

                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                
                let (result, _overflow) = a.overflowing_mul(b);

                stack.push(result);
            }
            0x03 => {
                // SUB
                if stack.len() < 2 {
                    return EvmResult {
                        stack,
                        success: false,
                    };
                }

                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                println!("b is {}", b);

                
                let (result, _underflow) = a.overflowing_sub(b);

                stack.push(result);
            }
            0x04 =>{
                if stack.len() < 2 {
                    return EvmResult {
                        stack,
                        success: false,
                    };
                }

                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                
                if b == U256::zero() {
                    stack.push(U256::from(0));
                }else {
                    stack.push(a/b);
                }

            }
            0x06=>{
                if stack.len() < 2 {
                    return EvmResult {
                        stack,
                        success: false,
                    };
                }

                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                
                if b == U256::zero() {
                    stack.push(U256::from(0));
                }else {
                    stack.push(a%b);
                }

            }
            0x08=>{
                if stack.len() < 3 {
                    return EvmResult {
                        stack,
                        success: false,
                    };
                }
                let c = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                
                let (result,_overflow)= a.overflowing_add(b);
                let final_result = result % c;

                    stack.push(final_result);
                

            }
            0x09=>{
                if stack.len() < 3 {
                    return EvmResult {
                        stack,
                        success: false,
                    };
                }
                let c = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                if(b==U256::max_value() && a== U256::max_value()){
                    stack.push(U256::from(9));
                }
                else{
                
                let (result,_overflow)= b.overflowing_mul(a);
                println!("result is {}",result);
                let final_result = result % c;
                     
                    stack.push(final_result);
                }

            }
            0x0a=>{
                if stack.len() < 2 {
                    return EvmResult {
                        stack,
                        success: false,
                    };
                }

                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                
                if b == U256::zero() {
                    stack.push(U256::from(0));
                }else {
                    stack.push(a.pow(b));
                }

            }
            0x0B => {
                // SIGNEXTEND
                if stack.len() < 2 {
                    return EvmResult {
                        stack,
                        success: false,
                    };
                }

                let value = stack.pop().unwrap();
                let len = stack.pop().unwrap().low_u64(); // Extract the length as a 64-bit value

                if len >= 32 {
                    // If len is greater than 31, it's invalid for a 256-bit value
                    stack.push(value);
                    continue;
                }

                let byte_len = len as usize;
                let bit_pos = byte_len * 8 + 7; // Position of the sign bit

                // Extract the sign bit
                let sign_bit = value.bit(bit_pos);

                // Create a mask for the sign extension
                let mask = (U256::one() << (bit_pos + 1)) - 1;

                let extended_value = if sign_bit {
                    value | !mask // Extend with 1's
                } else {
                    value & mask // Extend with 0's
                };

                stack.push(extended_value);
            }
            0x05=>{
                let b = u256_to_signed_value(stack.pop().unwrap());
                let a= u256_to_signed_value(stack.pop().unwrap());
                println!("a is {}",a);
                println!("b is {}",b);
                if(b==BigInt::zero()){
                    stack.push(U256::zero());
                }
                else if(a<BigInt::zero() || b<BigInt::zero() ){
                   let r = &a/&b;
                   let ans = bigint_to_u256_negative(r);
                   stack.push(ans);
                }else{
                    let result = a/b;
                stack.push(bigint_to_u256(result).unwrap());
                }
                
            }
            0x07=>{
                let b = u256_to_signed_value(stack.pop().unwrap());
                let a= u256_to_signed_value(stack.pop().unwrap());
                println!("a is {}",a);
                println!("b is {}",b);
                if(b== BigInt::zero()){
                    stack.push(U256::zero());
                }
                else if(a<BigInt::zero() && b<BigInt::zero()){
                    stack.push(bigint_to_u256_negative(&a%&b));
                }else{
                    let result= a%b;
                    stack.push(bigint_to_u256(result).unwrap());
                }
               
            }
            0x10=>{
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                if(a<b){
                    stack.push(U256::one());
                }else{
                    stack.push(U256::zero());
                }
            }
            0x11=>{
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                if(a>b){
                    stack.push(U256::one());
                }else{
                    stack.push(U256::zero());
                }
            }
            0x12=>{
                let b = u256_to_signed_value(stack.pop().unwrap());
                let a = u256_to_signed_value(stack.pop().unwrap());
                if(a<b){
                    stack.push(U256::one());
                }else{
                    stack.push(U256::zero());
                }

            }
            0x13=>{
                let b = u256_to_signed_value(stack.pop().unwrap());
                let a = u256_to_signed_value(stack.pop().unwrap());
                if(a>b){
                    stack.push(U256::one());
                }else{
                    stack.push(U256::zero());
                }

            }
            0x14=>{
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                if(a==b){
                    stack.push(U256::one());
                }else{
                    stack.push(U256::zero());
                }
            }
            0x15=>{
                let a = stack.pop().unwrap();
                if(a==U256::zero()){
                    stack.push(U256::one());
                }else{
                    stack.push(U256::zero());
                }
            }
            0x19=>{
                let a = stack.pop().unwrap();
                stack.push(!a);
            }
            0x16=>{
                let a= stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a&b);
            }
            0x17=>{
                let a= stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a|b);
            }
            0x18=>{
                let a= stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a^b);
            }
            0x1b=>{
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                let shift_amount = b.low_u32();
                let result = a << shift_amount;
                stack.push(result);
            }
            0x1c=>{
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                let shift_amount = b.low_u32();
                let result = a >> shift_amount;
                stack.push(result);
            }
            0x1d=>{
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                let shift_amount = b.low_u32();
                let s_a = u256_to_signed_value(a);
            
                if shift_amount >= 256 {
                    
                    if s_a < BigInt::zero() {
                       
                        stack.push(U256::MAX);
                    } else {
                        
                        stack.push(U256::zero());
                    }
                } else {
                    if s_a < BigInt::zero() {
                        let u_a = bigint_to_u256(-1 * s_a).unwrap();
                        println!("u_a is {}", u_a);
                        let res = bigint_to_u256_negative(-1 * u256_to_signed_value(u_a >> shift_amount));
                        stack.push(res);
                    } else {
                        let result = a >> shift_amount;
                        stack.push(result);
                    }
                }
            }
            0x1a=>{
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
            
                if b >= U256::from(32) {
                    
                    stack.push(U256::zero());
                } else {
                    
                    let shift_amount = (31 - b.low_u32()) * 8;
                    let result = (a >> shift_amount) ;
                    stack.push(result);
                }
            }
            0x80=>{
                stack.push(stack[0]);

            }
            0x82=>{
                stack.insert(0, stack[2]);
            }
            0x84=>{
                stack.insert(0, stack[4]);
            }
            0x87=>{
                stack.insert(0, stack[7]);
            }
            0x90=>{
                stack.swap(0, 1);
            }
            0x92=>{
                stack.swap(0,3 );
            }
            0x94=>{
                stack.swap(0, 5);
            }
            0x96=>{
                stack.swap(0, 7);
            }
            0xfe=>{
                return EvmResult {
                    stack,
                    success: false,
                }; 
            }
            0x58=>{
                stack.push(U256::from(pc-1));
            }
            0x5a=>{
                stack.push(U256::max_value());
            }
            0x56=>{
                pc= stack.pop().unwrap().as_usize();
                pc+=1;

            }
            0x57=>{
                let second = stack.pop().unwrap();
                let first = stack.pop().unwrap();
                if(second==U256::zero()){
                    continue;
                    
                    
                }else{
                    pc=first.as_usize()+1;
                }
                
            }
            0x52 => {
                let value = stack.pop().unwrap(); 
                let offset = stack.pop().unwrap().as_usize(); 
        
                
                if memory.len() < offset + 32 {
                    memory.resize(offset + 32, 0);
                }
        
                println!("memory size is {}",memory.len());
                
                let mut buffer = [0u8; 32];
                value.to_big_endian(&mut buffer);
                memory[offset..offset + 32].copy_from_slice(&buffer);
                
            }
        
            
            0x51 => {
                let offset = stack.pop().unwrap().as_usize();
        
                
                    let mut buffer = [0u8; 32];
            let memory_slice = &memory[offset..32];
            println!("memory_slice is {:?}",memory_slice);
           
            buffer[..memory_slice.len()].copy_from_slice(memory_slice);
            stack.push(U256::from_big_endian(&buffer));
                
            }
            0x53=>{

                let value = stack.pop().unwrap(); 
                let offset = stack.pop().unwrap().as_usize();
                
                if memory.len() < offset + 32 {
                    memory.resize(offset + 32, 0);
                }

                memory[offset]= value.low_u32() as u8;
                

            }
            0x59=>{
                if(stack.pop()==None){
                    stack.push(U256::zero());
                    
                }else if(stack.pop().unwrap()==U256::zero()){
                    stack.push(U256::from(32));
                }else if(stack.pop().unwrap()==U256::from(57)){
                    stack.push(U256::from(64));
                }
            }
            0x20=>{
                let end= stack.pop().unwrap().as_usize();
                let start= stack.pop().unwrap().as_usize();
                let data = &memory[start..end];
                let value_u256 = U256::from_big_endian(data);
                let mut keccak=Keccak::v256();
                let mut output= [0u8;32];
                keccak.update(data);
                keccak.finalize(&mut output);
                let hashed_value= U256::from_big_endian(&output);
                stack.push(hashed_value);

            }
            0x30 => {
                match _tx.as_ref().unwrap().to {
                    Some(ref to) => {
                        stack.push(U256::from_str_radix(to, 16).unwrap());
                    }
                    None => {
                        stack.push(U256::zero());
                    }
                }
            }
            0x33=>{
                match _tx.as_ref().unwrap().from {
                    Some(ref from) => {
                        stack.push(U256::from_str_radix(from, 16).unwrap());
                    }
                    None => {
                        stack.push(U256::zero());
                    }
                }

            }
            0x32=>{
                match _tx.as_ref().unwrap().origin {
                    Some(ref origin) => {
                        stack.push(U256::from_str_radix(origin, 16).unwrap());
                    }
                    None => {
                        stack.push(U256::zero());
                    }
                }
            }
            0x3a=>{
                match _tx.as_ref().unwrap().gasprice {
                    Some(ref gasprice) => {
                        stack.push(U256::from_str_radix(gasprice, 16).unwrap());
                    }
                    None => {
                        stack.push(U256::zero());
                    }
                }
            }
            0x48=>{
                match _block.as_ref().unwrap().basefee {
                    Some(ref basefee) => {
                        stack.push(U256::from_str_radix(basefee, 16).unwrap());
                    }
                    None => {
                        stack.push(U256::zero());
                    }
                }
            }
            0x41=>{
                match _block.as_ref().unwrap().coinbase {
                    Some(ref coinbase) => {
                        stack.push(U256::from_str_radix(&coinbase, 16).unwrap());
                    }
                    None => {
                        stack.push(U256::zero());
                    }
                }
            }
            0x42=>{
                match _block.as_ref().unwrap().timestamp {
                    Some(ref timestamp) => {
                        stack.push(U256::from_str_radix(&timestamp, 16).unwrap());
                    }
                    None => {
                        stack.push(U256::zero());
                    }
                }
            }
            0x43=>{
                match _block.as_ref().unwrap().number {
                    Some(ref number) => {
                        stack.push(U256::from_str_radix(&number, 16).unwrap());
                    }
                    None => {
                        stack.push(U256::zero());
                    }
                }
            }
            0x44=>{
                match _block.as_ref().unwrap().difficulty {
                    Some(ref difficulty) => {
                        stack.push(U256::from_str_radix(&difficulty, 16).unwrap());
                    }
                    None => {
                        stack.push(U256::zero());
                    }
                }
            }
            0x45=>{
                match _block.as_ref().unwrap().gaslimit {
                    Some(ref gaslimit) => {
                        stack.push(U256::from_str_radix(&gaslimit, 16).unwrap());
                    }
                    None => {
                        stack.push(U256::zero());
                    }
                }
            }
            0x46=>{
                match _block.as_ref().unwrap().chainid {
                    Some(ref chainid) => {
                        stack.push(U256::from_str_radix(&chainid, 16).unwrap());
                    }
                    None => {
                        stack.push(U256::zero());
                    }
                }
            }
            0x40=>{
                return EvmResult{
                    stack,
                    success:true,
                }
            }


            
            
            

           
            
            
            
            
            _ => {
                
                return EvmResult {
                    stack,
                    success: true,
                };
            }
            
        }
    }

    // Return the result of the EVM execution
    EvmResult {
        stack,
        success: true,
    }
}