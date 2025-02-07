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
mod format;
use openssl::aes::KeyError;
use optee_teec::{Context, Operation, ParamTmpRef, Session, Uuid};
use optee_teec::ParamNone;
use proto::{UUID, Command, KEY_PAIR_SIZE, MAC_SIZE};

fn generate_key(session: &mut Session) -> optee_teec::Result<()> {
    let mut pk = vec![0u8; KEY_PAIR_SIZE/8];
    let p0 = ParamTmpRef::new_output(&mut pk);

    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);

    session.invoke_command(Command::GenKeyPair as u32, &mut operation)?;

    println!(
        "Success plain text: 0x{}",
        hex::encode(&pk)
    );
    format::save_pem_to_file(&pk, "../pk").unwrap();
    Ok(())
}

fn doMac(session: &mut Session, data: Vec<u8>, key:Vec<u8>) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(&data);
    let p1 = ParamTmpRef::new_input(&key);

    let mut mac = vec![0u8; 16];
    let p2 = ParamTmpRef::new_output(&mut mac);

    let mut operation = Operation::new(0, p0, p1, p2, ParamNone);

    session.invoke_command(Command::Mac as u32, &mut operation)?;

    println!(
        "Success mac: 0x{}",
        hex::encode(&mac)
    );
    Ok(())
}

fn verify(session: &mut Session, data: Vec<u8>, key:Vec<u8>, mac: Vec<u8>) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(&data);
    let p1 = ParamTmpRef::new_input(&key);
    let p2 = ParamTmpRef::new_input(&mac);

    let mut operation = Operation::new(0, p0, p1, p2, ParamNone);

    session.invoke_command(Command::Verify as u32, &mut operation)?;

    println!(
        "Success verify"
    );
    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID).unwrap();
    let mut session = ctx.open_session(uuid)?;

    println!("Success");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::format::read_pem_from_file;

    use super::*;

    #[test]
    fn test_generate_key() {
        let mut ctx = Context::new().unwrap();
        let uuid = Uuid::parse_str(UUID).unwrap();
        let mut session = ctx.open_session(uuid).unwrap();

        generate_key(&mut session).unwrap();
    }

    #[test]
    fn test_doMac() {
        let mut ctx = Context::new().unwrap();
        let uuid = Uuid::parse_str(UUID).unwrap();
        let mut session = ctx.open_session(uuid).unwrap();
        let mut file = "/home/pengxu/rust-optee/aes-cbc-mac-nopad/pk/public_key_20250205_164300.pem";
        let key = read_pem_from_file(&mut file).unwrap();
        
        let data = vec![1u8; 16];
<<<<<<< HEAD
        doMac(&mut session, data, key.to_vec()).unwrap();
    }

    #[test]
    fn test_verify(){
        let mut ctx = Context::new().unwrap();
        let uuid = Uuid::parse_str(UUID).unwrap();
        let mut session = ctx.open_session(uuid).unwrap();
        let mut file = "/home/pengxu/rust-optee/aes-cbc-mac-nopad/pk/public_key_20250205_164300.pem";
        let key = read_pem_from_file(&mut file).unwrap();
        
        let data = vec![1u8; 16];
        let mac = vec![32, 23, 54, 209, 77, 55, 192, 125, 144, 202, 105, 40, 115, 197, 65, 244];

        verify(&mut session, data,key, mac).unwrap();
=======
        doMac(&mut session, data, key).unwrap();
    }

    #[test]
    fn test_verify() {
        let mut ctx = Context::new().unwrap();
        let uuid = Uuid::parse_str(UUID).unwrap();
        let mut session = ctx.open_session(uuid).unwrap();

        let data = vec![0u8; 16];
        let mac = vec![0u8; 16];
        verify(&mut session, data, mac).unwrap();
>>>>>>> 98046ba0b61f447b0d0f86e758c7f090bbeb6ef5
    }
}