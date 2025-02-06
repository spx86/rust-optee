// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#![no_std]
#![no_main]
#![feature(c_size_t)]

extern crate alloc;
use alloc::vec;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println, AlgorithmId, Asymmetric, AttributeId, AttributeMemref, AttributeValue, Mac, TransientObject, TransientObjectType
};
use optee_utee::{Error, ErrorKind, Parameters, Result};
use proto::{Command, KEY_PAIR_SIZE, MAC_SIZE};

#[panic_handler]
fn panic_handler(panic: &core::panic::PanicInfo<'_>) -> ! {
    trace_println!("TEE PANIC!!!! PanicInfo: {:#?}", panic);
    unsafe { 
        optee_utee_sys::TEE_Panic(optee_utee_sys::TEE_ERROR_BAD_STATE) 
    };
    loop {}
}


#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session() {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

fn gen_key_pair(params: &mut Parameters) -> Result<()> {

    let mut p0 = unsafe { params.0.as_memref().unwrap() };

    let key  =
        TransientObject::allocate(TransientObjectType::Aes, KEY_PAIR_SIZE).unwrap();

    key.generate_key(KEY_PAIR_SIZE, &[])?;

    let mut sk = vec![0u8; KEY_PAIR_SIZE/8];

    key.ref_attribute(AttributeId::SecretValue, &mut sk)?;

    p0.buffer().copy_from_slice(&sk);

    trace_println!("key pair generated successfully");
    trace_println!("key: {:?}", sk);
    // storage::create_raw_object(p0.buffer().to_vec(), &mut sk)
    Ok(())
}

fn do_mac(params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref().unwrap() }; //data
    let mut p1 = unsafe { params.1.as_memref().unwrap() }; //key
    let mut p2 = unsafe { params.2.as_memref().unwrap() }; //mac

    let data = p0.buffer();
    let aes_key = p1.buffer();
    let mut out = vec![0u8; MAC_SIZE];

    // let key = init_object(aes_key);

    // trace_println!("test1");

    // let aes = Mac::allocate(
    //     optee_utee::AlgorithmId::AesCbcMacNopad,
    //         KEY_PAIR_SIZE).unwrap();
    // aes.set_key(&key)?;

    // trace_println!("aes key: {:?}", aes_key);
    // aes.init(&[0u8; 0]);

    // trace_println!("mac data: {:?}", data);
    // aes.update(&[0u8; 0]);

    // trace_println!("test2");
    // aes.compute_final(&[0u8; 0], &mut out);

    let mut key: [u8; 20] = [
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35,
        0x36, 0x37, 0x38, 0x39, 0x30,];
    let mut out: [u8; 20] = [0u8; 20];
    match Mac::allocate(AlgorithmId::HmacSha1, key.len() * 8) {
        Err(e) => return Err(e),
        Ok(mac) => {
            match TransientObject::allocate(TransientObjectType::HmacSha1, key.len() * 8) {
                Err(e) => return Err(e),
                Ok(mut key_object) => {
                    let attr = AttributeMemref::from_ref(AttributeId::SecretValue, &key);
                    key_object.populate(&[attr.into()])?;
                    mac.set_key(&key_object)?;
                }
            }
            mac.init(&[0u8; 0]);
            mac.update(&[0u8; 8]);
            trace_println!("test2");
            mac.compute_final(&[0u8; 0], &mut out)?;
        }
    }

    trace_println!("test3");
    p2.buffer().copy_from_slice(&out);

    trace_println!("mac generated successfully");
    trace_println!("mac: {:?}", out);
    Ok(())
}

fn init_object(buffer: &[u8]) -> TransientObject {
    let mut key  =
        TransientObject::allocate(TransientObjectType::Aes, KEY_PAIR_SIZE).unwrap();
    let aes_key = AttributeMemref::from_ref(AttributeId::SecretValue, buffer);
    key.populate(&[aes_key.into()]).unwrap();
    key
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");

    let cmd = Command::try_from(cmd_id).map_err(|e| {
        optee_utee::trace_println!("Unknown cmd {}, err is {}", cmd_id, e);
        ErrorKind::BadParameters
    })?;

    
    match cmd {
        Command::GenKeyPair => {
            gen_key_pair(params);
            Ok(())
        }
        Command::Mac => {
            do_mac(params); 
            Ok(())
        }
        _ => Err(Error::new(ErrorKind::BadParameters)),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {
    trace_println!("the rust_eh_personality function should never be called");
    unsafe { 
        optee_utee_sys::TEE_Panic(optee_utee_sys::TEE_ERROR_BAD_STATE) 
    };
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _Unwind_Resume() {
    trace_println!("the _Unwind_Resume function should never be called");
    unsafe { 
        optee_utee_sys::TEE_Panic(optee_utee_sys::TEE_ERROR_BAD_STATE) 
    };
    loop {}
}




