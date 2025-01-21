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
mod storage;

extern crate alloc;
use alloc::vec;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println, Asymmetric, AttributeMemref, Digest, ObjHandle
};
use optee_utee::{Error, ErrorKind, Parameters, Result};
use proto::{Command, HASH_SIZE, KEY_PAIR_SIZE};
use optee_utee::{TransientObject, TransientObjectType};
use optee_utee::{AttributeId, ElementId, AttributeValue};

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

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");

    let cmd = Command::try_from(cmd_id).map_err(|e| {
        optee_utee::trace_println!("Unknown cmd {}, err is {}", cmd_id, e);
        ErrorKind::BadParameters
    })?;

    match cmd {
        Command::GenKeyPair => {
            gen_key_pair(params)
        }
        Command::DelkeyPair => {
            del_key_pair(params)
        }
        Command::Sign => {
            sign(params)
        }
        Command::Verify => {
            verify(params)
        }
        _ => Err(Error::new(ErrorKind::BadParameters)),
    }
}

fn gen_key_pair(params: &mut Parameters) -> Result<()> {

    let mut p0 = unsafe { params.0.as_memref().unwrap() };

    let key  =
        TransientObject::allocate(TransientObjectType::EcdsaKeypair, KEY_PAIR_SIZE).unwrap();
    
    let attr_curve = AttributeValue::from_value(AttributeId::EccCurve, ElementId::EccCurveNistP256 as u32, 0);
    key.generate_key(KEY_PAIR_SIZE, &[attr_curve.into()])?;

    let mut sk = vec![0u8; KEY_PAIR_SIZE/8];

    p0.buffer()[0] = 0x04;
    key.ref_attribute(AttributeId::EccPublicValueX, &mut p0.buffer()[1..33])?;
    key.ref_attribute(AttributeId::EccPublicValueY, &mut p0.buffer()[33..])?;

    key.ref_attribute(AttributeId::EccPrivateValue, &mut sk)?;

    trace_println!("key pair generated successfully");
    storage::create_raw_object(p0.buffer().to_vec(), &mut sk)
}

fn del_key_pair(params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref().unwrap() };
    trace_println!("delete key pair");
    storage::delete_object(p0.buffer().to_vec())
}

fn sign(params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref().unwrap() }; //key
    let mut p1 = unsafe { params.1.as_memref().unwrap() }; //message
    let mut p2 = unsafe { params.2.as_memref().unwrap() }; //signature

    //hash计算
    let mut hash = vec![0u8; HASH_SIZE];
    let dig = Digest::allocate(optee_utee::AlgorithmId::Sha256).unwrap();
    let _ =dig.do_final(&p1.buffer(), &mut hash);
    trace_println!("hash: {:?}", hash);

    //签名
    let mut signature = vec![0u8; KEY_PAIR_SIZE/4];

    let key = init_object(p0.buffer());

    let p256 = Asymmetric::allocate(
        optee_utee::AlgorithmId::EcDsaSha256, 
            optee_utee::OperationMode::Sign,
            KEY_PAIR_SIZE).unwrap();
    p256.set_key(&key)?;

    match p256.sign_digest(&[], &hash, &mut signature) {
        Ok(_len) => {
            trace_println!("signature: {:?}", signature);
            p2.buffer().copy_from_slice(&signature);
            Ok(())
        }
        Err(e) => {
            trace_println!("sign error: {:?}", e);
            Err(Error::new(ErrorKind::SignatureInvalid))
        }
        
    }

}

fn verify(params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref().unwrap() }; //pk
    let mut p1 = unsafe { params.1.as_memref().unwrap() }; //sign
    let mut p2 = unsafe { params.2.as_memref().unwrap() }; //content

    let message = p2.buffer();
    let sign = p1.buffer();

    let key = init_object(p0.buffer());

    //hash计算
    let mut hash = vec![0u8; HASH_SIZE];
    let dig = Digest::allocate(optee_utee::AlgorithmId::Sha256).unwrap();
    let _ =dig.do_final(&message, &mut hash);
    trace_println!("hash: {:?}", hash);

    let p256 = Asymmetric::allocate(
        optee_utee::AlgorithmId::EcDsaSha256, 
            optee_utee::OperationMode::Verify,
            KEY_PAIR_SIZE).unwrap();

    p256.set_key(&key)?;

    match p256.verify_digest(&[], &hash, &sign) {
        Ok(()) => {
            trace_println!("verify success");
            Ok(())
        }
        Err(e) => {
            trace_println!("verify error: {:?}", e);
            Err(Error::new(ErrorKind::SignatureInvalid))
        }
    }
}

fn init_object(pk: &[u8]) -> impl ObjHandle {
    let sk = storage::read_raw_object(pk.to_vec()).unwrap();
    let mut key  =
        TransientObject::allocate(TransientObjectType::EcdsaKeypair, KEY_PAIR_SIZE).unwrap();
    let attr_curve = AttributeValue::from_value(AttributeId::EccCurve, ElementId::EccCurveNistP256 as u32, 0);
    let sk_attr = AttributeMemref::from_ref(AttributeId::EccPrivateValue, &sk);
    let pk_x_attr = AttributeMemref::from_ref(AttributeId::EccPublicValueX, &pk[1..33]);
    let pk_y_attr = AttributeMemref::from_ref(AttributeId::EccPublicValueY, &pk[33..]);
    key.populate(&[attr_curve.into(), sk_attr.into(), pk_x_attr.into(), pk_y_attr.into()]).unwrap();
    key
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




