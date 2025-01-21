
extern crate alloc;

use core::mem;

use alloc::vec;
use optee_utee::{DataFlag, ObjectStorageConstants, PersistentObject};
use optee_utee::{Error, ErrorKind, Result};

pub fn delete_object(object_id: vec::Vec<u8>) -> Result<()> {
    let mut object_id = object_id[1..].to_vec();
    match PersistentObject::open(
        ObjectStorageConstants::Private,
        &mut object_id,
        DataFlag::ACCESS_READ | DataFlag::ACCESS_WRITE_META,
    ) {
        Err(e) => {
            return Err(e);
        }

        Ok(mut object) => {
            object.close_and_delete()?;
            mem::forget(object);
            return Ok(());
        }
    }
}

pub fn create_raw_object(object_id: vec::Vec<u8>, data_buffer: &mut vec::Vec<u8>) -> Result<()> {
    let obj_data_flag = DataFlag::ACCESS_READ
        | DataFlag::ACCESS_WRITE
        | DataFlag::ACCESS_WRITE_META
        | DataFlag::OVERWRITE;

    let mut init_data: [u8; 0] = [0; 0];
    let mut object_id = object_id[1..].to_vec();

    match PersistentObject::create(
        ObjectStorageConstants::Private,
        &mut object_id,
        obj_data_flag,
        None,
        &mut init_data,
    ) {
        Err(e) => {
            return Err(e);
        }

        Ok(mut object) => match object.write(&data_buffer) {
            Ok(()) => {
                return Ok(());
            }
            Err(e_write) => {
                object.close_and_delete()?;
                mem::forget(object);
                return Err(e_write);
            }
        },
    }
}

pub fn read_raw_object(object_id: vec::Vec<u8>) -> Result<vec::Vec<u8>> {
    let mut object_id = object_id[1..].to_vec();
    match PersistentObject::open(
        ObjectStorageConstants::Private,
        &mut object_id,
        DataFlag::ACCESS_READ | DataFlag::SHARE_READ,
    ) {
        Err(e) => {
            return Err(e);
        }

        Ok(object) => {
            let obj_info = object.info()?;

            let mut data_buffer = vec![0; obj_info.data_size() as usize];
            let read_bytes = object.read(&mut data_buffer).unwrap();
            if read_bytes != obj_info.data_size() as u32 {
                return Err(Error::new(ErrorKind::ExcessData));
            }

            return Ok(data_buffer);
        }
    }
}