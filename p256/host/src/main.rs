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

use optee_teec::{Context, Operation, ParamTmpRef, Session, Uuid};
use optee_teec::ParamNone;
use proto::{Command, KEY_PAIR_SIZE, SIGNATURE_SIZE, UUID};

fn generate_key(session: &mut Session) -> optee_teec::Result<()> {
    let mut pk = vec![0u8; KEY_PAIR_SIZE/4+1];
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

fn del_key(session: &mut Session, key: Vec<u8>) -> optee_teec::Result<String> {
    let p0 = ParamTmpRef::new_input(&key);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);

    match session.invoke_command(Command::DelkeyPair as u32, &mut operation){
        Ok(()) => Ok("Success delete key".to_string()),
        Err(e) => Err(e)
    }

}

fn sign(session: &mut Session, key: Vec<u8>, data: Vec<u8>) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(&key);
    let p1 = ParamTmpRef::new_input(&data);
    let mut signature = vec![0u8; SIGNATURE_SIZE];
    let p2 = ParamTmpRef::new_output(&mut signature);

    let mut operation = Operation::new(0, p0, p1, p2, ParamNone);

    session.invoke_command(Command::Sign as u32, &mut operation)?;

    println!(
        "Success signature: 0x{}",
        hex::encode(&signature)
    );

    format::save_sign_to_file(&signature, "../sign").unwrap();
    Ok(())
}

fn verify(session: &mut Session, key: Vec<u8>, data: Vec<u8>, signature: Vec<u8>) -> optee_teec::Result<bool> {
    let p0 = ParamTmpRef::new_input(&key);
    let p1 = ParamTmpRef::new_input(&signature);
    let p2 = ParamTmpRef::new_input(&data);

    let mut operation = Operation::new(0, p0, p1, p2, ParamNone);

    match session.invoke_command(Command::Verify as u32, &mut operation){
        Ok(()) => Ok(true),
        Err(e) => Err(e)
    }

}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID).unwrap();
    let mut session = ctx.open_session(uuid)?;


    generate_key(&mut session)?;

    println!("Success");
    Ok(())
}


#[cfg(test)]
mod tests {
    use format::read_pem_from_file;

    use super::*;

    #[test]
    fn test_generate_key() {
        let mut ctx = Context::new().unwrap();
        let uuid = Uuid::parse_str(UUID).unwrap();
        let mut session = ctx.open_session(uuid).unwrap();

        generate_key(&mut session).unwrap();
    }

    #[test]
    fn test_del_key() {
        let mut ctx = Context::new().unwrap();
        let uuid = Uuid::parse_str(UUID).unwrap();
        let mut session = ctx.open_session(uuid).unwrap();

        let mut file = "/home/pengxu/rust-optee/p256/pk/public_key_20250121_113713.pem";
        let pk = read_pem_from_file(&mut file).unwrap();
        println!("pk: {:?}", pk);
        del_key(&mut session, pk).unwrap();
    }

    #[test]
    fn test_sign() {
        let mut ctx = Context::new().unwrap();
        let uuid = Uuid::parse_str(UUID).unwrap();
        let mut session = ctx.open_session(uuid).unwrap();

        let mut file = "/home/pengxu/rust-optee/p256/pk/public_key_20250206_151947.pem";
        let pk = read_pem_from_file(&mut file).unwrap();
        let data = vec![0u8; 32];
        sign(&mut session, pk, data).unwrap();
    }

    #[test]
    fn test_verify() {
        let mut ctx = Context::new().unwrap();
        let uuid = Uuid::parse_str(UUID).unwrap();
        let mut session = ctx.open_session(uuid).unwrap();

        let mut file = "/home/pengxu/rust-optee/p256/pk/public_key_20250206_151947.pem";
        let pk = read_pem_from_file(&mut file).unwrap();
        let data = vec![0u8; 32];
        let signature = format::read_sign_from_file("/home/pengxu/rust-optee/p256/sign/signature_20250206_152106.dat").unwrap();
        verify(&mut session, pk, data, signature).unwrap();
    }

}